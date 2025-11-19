/// BuildBlockService 实现 - 区块构建服务
///
/// 职责：
/// 1. 编排区块构建流程
/// 2. 集成交易池、状态执行器
/// 3. 计算区块头字段
/// 4. 验证区块合法性
///
/// 参考: geth/miner/worker.go

use crate::domain::block_types::{
    Block, BlockHeader, BlockValidationError, BuildEnvironment, Withdrawal,
};
use crate::domain::receipt_types::TransactionReceipt;
use crate::domain::tx_types::DynamicFeeTx;
use crate::service::build_block_trait::BlockBuilder;
use crate::service::repo::transaction_repo::TxPool;
use async_trait::async_trait;
use ethereum_types::{Bloom, H256, U256, U64};
use std::sync::Arc;

/// Base Fee计算器 (EIP-1559)
///
/// 参考: https://eips.ethereum.org/EIPS/eip-1559
///
/// Base fee动态调整算法：
/// - 目标使用率: 50% (gas_target = gas_limit / 2)
/// - 使用率 > 50%: base fee上涨
/// - 使用率 < 50%: base fee下降
/// - 最大变化率: 12.5% (1/8)
pub struct BaseFeeCalculator;

impl BaseFeeCalculator {
    /// EIP-1559常量
    const BASE_FEE_MAX_CHANGE_DENOMINATOR: u64 = 8; // 最大变化率 12.5%
    const ELASTICITY_MULTIPLIER: u64 = 2; // 弹性乘数
    const INITIAL_BASE_FEE: u64 = 1_000_000_000; // 1 Gwei

    /// 计算下一个区块的base fee
    ///
    /// 算法 (EIP-1559):
    /// ```python
    /// gas_target = parent_gas_limit // ELASTICITY_MULTIPLIER
    /// if parent_gas_used > gas_target:
    ///     gas_used_delta = parent_gas_used - gas_target
    ///     base_fee_delta = max(
    ///         parent_base_fee * gas_used_delta // gas_target // BASE_FEE_MAX_CHANGE_DENOMINATOR,
    ///         1
    ///     )
    ///     return parent_base_fee + base_fee_delta
    /// elif parent_gas_used < gas_target:
    ///     gas_used_delta = gas_target - parent_gas_used
    ///     base_fee_delta = parent_base_fee * gas_used_delta // gas_target // BASE_FEE_MAX_CHANGE_DENOMINATOR
    ///     return max(parent_base_fee - base_fee_delta, 0)
    /// else:
    ///     return parent_base_fee
    /// ```
    pub fn calculate_base_fee(
        parent_gas_used: u64,
        parent_gas_limit: u64,
        parent_base_fee: U256,
    ) -> U256 {
        let gas_target = parent_gas_limit / Self::ELASTICITY_MULTIPLIER;

        if parent_gas_used == gas_target {
            // 使用量正好等于目标，base fee不变
            return parent_base_fee;
        }

        if parent_gas_used > gas_target {
            // 拥堵，增加base fee
            let gas_used_delta = parent_gas_used - gas_target;
            let base_fee_delta = std::cmp::max(
                (parent_base_fee * U256::from(gas_used_delta))
                    / U256::from(gas_target)
                    / U256::from(Self::BASE_FEE_MAX_CHANGE_DENOMINATOR),
                U256::one(),
            );
            parent_base_fee + base_fee_delta
        } else {
            // 空闲，降低base fee
            let gas_used_delta = gas_target - parent_gas_used;
            let base_fee_delta = (parent_base_fee * U256::from(gas_used_delta))
                / U256::from(gas_target)
                / U256::from(Self::BASE_FEE_MAX_CHANGE_DENOMINATOR);

            // Base fee最小为0
            parent_base_fee.saturating_sub(base_fee_delta)
        }
    }

    /// 获取初始base fee (创世区块)
    pub fn initial_base_fee() -> U256 {
        U256::from(Self::INITIAL_BASE_FEE)
    }
}

/// Gas Limit计算器
///
/// 参考: geth/consensus/misc/eip1559.go
///
/// Gas limit动态调整规则：
/// - 每个区块最多调整 parent_gas_limit / 1024
/// - 最小值为 5000 gas
/// - 根据使用率自适应调整
pub struct GasLimitCalculator;

impl GasLimitCalculator {
    /// Gas limit调整限制 (每个区块最多调整1/1024)
    const GAS_LIMIT_BOUND_DIVISOR: u64 = 1024;
    /// 最小gas limit
    const MIN_GAS_LIMIT: u64 = 5000;

    /// 计算下一个区块的gas limit
    ///
    /// 规则:
    /// - 每个区块gas limit最多调整 parent_gas_limit / 1024
    /// - 最小值为5000
    /// - 根据父区块使用率动态调整:
    ///   - 使用率 > 90%: 增加
    ///   - 使用率 < 50%: 减少
    ///   - 其他: 保持不变
    ///
    /// 参数:
    /// - parent_gas_used: 父区块的gas使用量
    /// - parent_gas_limit: 父区块的gas限制
    /// - desired_limit: 可选的目标gas limit (矿工/验证者配置)
    pub fn calculate_gas_limit(
        parent_gas_used: u64,
        parent_gas_limit: u64,
        desired_limit: Option<u64>,
    ) -> u64 {
        let delta = parent_gas_limit / Self::GAS_LIMIT_BOUND_DIVISOR;
        let mut limit = parent_gas_limit;

        // 如果指定了目标limit，向目标调整
        if let Some(desired) = desired_limit {
            if desired > parent_gas_limit {
                limit = std::cmp::min(parent_gas_limit + delta, desired);
            } else if desired < parent_gas_limit {
                limit = std::cmp::max(parent_gas_limit.saturating_sub(delta), desired);
            }
        } else {
            // 根据使用率自动调整
            let usage_ratio = (parent_gas_used as f64) / (parent_gas_limit as f64);

            if usage_ratio > 0.9 {
                // 高负载，增加gas limit
                limit = parent_gas_limit + delta;
            } else if usage_ratio < 0.5 {
                // 低负载，减少gas limit
                limit = parent_gas_limit.saturating_sub(delta);
            }
        }

        // 确保不低于最小值
        std::cmp::max(limit, Self::MIN_GAS_LIMIT)
    }

    /// 验证gas limit调整是否合法
    ///
    /// 检查:
    /// - 不低于最小值 (5000)
    /// - 调整幅度不超过 1/1024
    pub fn validate_gas_limit(
        parent_gas_limit: u64,
        current_gas_limit: u64,
    ) -> Result<(), BlockValidationError> {
        let delta = parent_gas_limit / Self::GAS_LIMIT_BOUND_DIVISOR;
        let max_limit = parent_gas_limit + delta;
        let min_limit = parent_gas_limit.saturating_sub(delta);

        if current_gas_limit < Self::MIN_GAS_LIMIT {
            return Err(BlockValidationError::GasLimitAdjustmentTooLarge {
                parent: parent_gas_limit,
                current: current_gas_limit,
            });
        }

        if current_gas_limit > max_limit || current_gas_limit < min_limit {
            return Err(BlockValidationError::GasLimitAdjustmentTooLarge {
                parent: parent_gas_limit,
                current: current_gas_limit,
            });
        }

        Ok(())
    }
}

/// 交易选择器
///
/// 负责从交易池选择最优交易组合 (装箱问题)
///
/// 参考: geth/miner/worker.go:commitTransactions
pub struct TransactionSelector;

impl TransactionSelector {
    /// 选择交易 (贪心算法)
    ///
    /// 策略参考 geth/miner/worker.go:
    /// 1. 过滤: max_fee_per_gas >= base_fee
    /// 2. 排序: 按 effective_priority_fee 降序
    /// 3. 装箱: 累计gas不超过gas_limit
    ///
    /// 注意:
    /// - effective_priority_fee = min(max_priority_fee, max_fee - base_fee)
    /// - 优先选择给矿工小费高的交易
    /// - 区块填充到95%停止 (留一些缓冲空间)
    ///
    /// 参数:
    /// - candidates: 候选交易列表
    /// - gas_limit: 区块gas限制
    /// - base_fee: 当前base fee
    ///
    /// 返回: 选中的交易列表 (按优先级排序)
    pub fn select_transactions(
        candidates: Vec<DynamicFeeTx>,
        gas_limit: u64,
        base_fee: U256,
    ) -> Vec<DynamicFeeTx> {
        let mut selected = Vec::new();
        let mut total_gas: u64 = 0;

        // Step 1: 过滤低价交易
        let mut valid_txs: Vec<_> = candidates
            .into_iter()
            .filter(|tx| tx.max_fee_per_gas >= base_fee)
            .collect();

        // Step 2: 按effective priority fee降序排序
        valid_txs.sort_by(|a, b| {
            let a_priority = Self::effective_priority_fee(a, &base_fee);
            let b_priority = Self::effective_priority_fee(b, &base_fee);
            b_priority.cmp(&a_priority) // 降序
        });

        // Step 3: 贪心装箱
        for tx in valid_txs {
            let tx_gas = tx.gas_limit.as_u64();

            // 检查是否还有空间
            if total_gas + tx_gas <= gas_limit {
                total_gas += tx_gas;
                selected.push(tx);
            }

            // 如果区块已满95%，停止 (留一些缓冲)
            if total_gas >= gas_limit * 95 / 100 {
                break;
            }
        }

        selected
    }

    /// 计算effective priority fee (矿工实际收益)
    ///
    /// effective_priority_fee = min(max_priority_fee, max_fee - base_fee)
    ///
    /// 这是矿工/验证者实际能获得的小费金额
    fn effective_priority_fee(tx: &DynamicFeeTx, base_fee: &U256) -> U256 {
        let max_priority = tx.max_priority_fee_per_gas;
        let max_fee_minus_base = tx.max_fee_per_gas.saturating_sub(*base_fee);

        // min(max_priority_fee, max_fee - base_fee)
        if max_priority < max_fee_minus_base {
            max_priority
        } else {
            max_fee_minus_base
        }
    }
}

/// 区块构建服务实现
///
/// 设计原则：
/// - 无状态服务（状态由外部依赖管理）
/// - Erlang风格通信（通过trait抽象）
/// - 依赖注入（依赖接口而非具体实现）
pub struct BuildBlockService {
    /// 交易池（获取候选交易）
    tx_pool: Arc<dyn TxPool>,
    /// 期望的gas limit（矿工配置，None则自动调整）
    desired_gas_limit: Option<u64>,
}

impl BuildBlockService {
    /// 创建新的区块构建服务
    ///
    /// 参数:
    /// - tx_pool: 交易池实例
    /// - desired_gas_limit: 期望的gas limit（None则根据使用率自动调整）
    pub fn new(tx_pool: Arc<dyn TxPool>, desired_gas_limit: Option<u64>) -> Self {
        Self {
            tx_pool,
            desired_gas_limit,
        }
    }

    /// 计算新区块的base fee
    fn calculate_base_fee(&self, env: &BuildEnvironment) -> U256 {
        BaseFeeCalculator::calculate_base_fee(
            env.parent_gas_used.as_u64(),
            env.parent_gas_limit.as_u64(),
            env.parent_base_fee,
        )
    }

    /// 计算新区块的gas limit
    fn calculate_gas_limit(&self, env: &BuildEnvironment) -> u64 {
        GasLimitCalculator::calculate_gas_limit(
            env.parent_gas_used.as_u64(),
            env.parent_gas_limit.as_u64(),
            self.desired_gas_limit,
        )
    }

    /// 从交易池获取候选交易
    async fn get_candidate_transactions(
        &self,
        base_fee: U256,
    ) -> Result<Vec<DynamicFeeTx>, BlockValidationError> {
        // 从交易池获取最多1000笔交易（足够填满一个区块）
        self.tx_pool
            .get_pending(1000, Some(base_fee.as_u64()))
            .await
            .map_err(|e| {
                BlockValidationError::Other(format!("Failed to get pending transactions: {}", e))
            })
    }

    /// 选择并执行交易
    ///
    /// 返回: (选中的交易, 总gas使用量, 收据列表)
    async fn select_and_execute_transactions(
        &self,
        candidates: Vec<DynamicFeeTx>,
        gas_limit: u64,
        base_fee: U256,
    ) -> Result<(Vec<DynamicFeeTx>, u64, Vec<TransactionReceipt>), BlockValidationError> {
        // Step 1: 使用贪心算法选择交易
        let selected_txs =
            TransactionSelector::select_transactions(candidates, gas_limit, base_fee);

        // Step 2: 执行交易并累计gas使用量
        let mut receipts = Vec::new();
        let mut total_gas_used: u64 = 0;
        let mut executed_txs = Vec::new();

        for tx in selected_txs {
            // TODO: 集成revm执行交易
            // 目前先使用简化逻辑：假设每笔交易使用其gas_limit
            let gas_used = tx.gas_limit.as_u64();

            // 检查是否超出区块gas limit
            if total_gas_used + gas_used > gas_limit {
                break; // 区块已满
            }

            // 创建收据
            let receipt = TransactionReceipt::new(
                tx.hash(),
                receipts.len() as u64,
                total_gas_used + gas_used,
                gas_used,
                true, // 假设交易成功
                Bloom::zero(),
                vec![],
            );

            total_gas_used += gas_used;
            receipts.push(receipt);
            executed_txs.push(tx);
        }

        Ok((executed_txs, total_gas_used, receipts))
    }

    /// 计算交易根 (Merkle-Patricia Trie)
    ///
    /// TODO: 实现完整的MPT计算
    /// 参考: geth/core/types/derive_sha.go
    fn calculate_transactions_root(&self, _transactions: &[DynamicFeeTx]) -> H256 {
        // 暂时返回零值，后续实现完整的MPT
        H256::zero()
    }

    /// 计算收据根 (Merkle-Patricia Trie)
    ///
    /// TODO: 实现完整的MPT计算
    fn calculate_receipts_root(&self, _receipts: &[TransactionReceipt]) -> H256 {
        // 暂时返回零值，后续实现完整的MPT
        H256::zero()
    }

    /// 计算状态根 (Merkle-Patricia Trie)
    ///
    /// TODO: 集成状态数据库，实现完整的状态根计算
    /// 参考: geth/core/state/statedb.go
    fn calculate_state_root(&self) -> H256 {
        // 暂时返回零值，后续实现完整的状态根计算
        H256::zero()
    }

    /// 计算日志Bloom过滤器
    ///
    /// TODO: 实现完整的Bloom过滤器计算
    /// 参考: geth/core/types/bloom9.go
    fn calculate_logs_bloom(&self, _receipts: &[TransactionReceipt]) -> Bloom {
        // 暂时返回零值
        Bloom::zero()
    }

    /// 计算提取根 (Withdrawals Root)
    ///
    /// EIP-4895: 验证者提款
    fn calculate_withdrawals_root(&self, withdrawals: &[Withdrawal]) -> Option<H256> {
        if withdrawals.is_empty() {
            None
        } else {
            // TODO: 实现完整的MPT计算
            Some(H256::zero())
        }
    }

    /// 构建区块头
    fn build_header(
        &self,
        env: &BuildEnvironment,
        base_fee: U256,
        gas_limit: u64,
        gas_used: u64,
        transactions_root: H256,
        receipts_root: H256,
        state_root: H256,
        logs_bloom: Bloom,
        withdrawals_root: Option<H256>,
    ) -> BlockHeader {
        BlockHeader {
            parent_hash: env.parent_hash,
            ommers_hash: BlockHeader::empty_ommers_hash(), // PoS: 固定为空
            fee_recipient: env.fee_recipient,
            state_root,
            transactions_root,
            receipts_root,
            logs_bloom,
            difficulty: U256::zero(), // PoS: 固定为0
            number: env.parent_number + 1,
            gas_limit: U64::from(gas_limit),
            gas_used: U64::from(gas_used),
            timestamp: env.timestamp,
            extra_data: vec![],                    // 可以添加矿工/验证者签名
            mix_hash: env.prev_randao,             // PoS: 存储RANDAO值
            nonce: 0,                              // PoS: 固定为0
            base_fee_per_gas: Some(base_fee),      // EIP-1559
            withdrawals_root,                      // EIP-4895
            blob_gas_used: None,                   // EIP-4844: 暂不支持
            excess_blob_gas: None,                 // EIP-4844: 暂不支持
            parent_beacon_block_root: env.parent_beacon_block_root, // EIP-4788
        }
    }
}

#[async_trait]
impl BlockBuilder for BuildBlockService {
    /// 构建新区块
    ///
    /// 完整流程:
    /// 1. 计算base fee (EIP-1559动态调整)
    /// 2. 计算gas limit (动态调整)
    /// 3. 从交易池获取候选交易
    /// 4. 选择并执行交易 (贪心算法 + 装箱)
    /// 5. 计算Merkle根 (transactions_root, receipts_root, state_root)
    /// 6. 计算logs bloom过滤器
    /// 7. 组装区块头
    /// 8. 返回完整区块
    async fn build_block(&self, env: BuildEnvironment) -> Result<Block, BlockValidationError> {
        // Step 1: 计算base fee
        let base_fee = self.calculate_base_fee(&env);

        // Step 2: 计算gas limit
        let gas_limit = self.calculate_gas_limit(&env);

        // 验证gas limit调整合法性
        GasLimitCalculator::validate_gas_limit(env.parent_gas_limit.as_u64(), gas_limit)?;

        // Step 3: 获取候选交易
        let candidates = self.get_candidate_transactions(base_fee).await?;

        // Step 4: 选择并执行交易
        let (transactions, gas_used, receipts) = self
            .select_and_execute_transactions(candidates, gas_limit, base_fee)
            .await?;

        // Step 5: 计算Merkle根
        let transactions_root = self.calculate_transactions_root(&transactions);
        let receipts_root = self.calculate_receipts_root(&receipts);
        let state_root = self.calculate_state_root();

        // Step 6: 计算logs bloom
        let logs_bloom = self.calculate_logs_bloom(&receipts);

        // Step 7: 计算withdrawals root
        let withdrawals_root = self.calculate_withdrawals_root(&env.withdrawals);

        // Step 8: 构建区块头
        let header = self.build_header(
            &env,
            base_fee,
            gas_limit,
            gas_used,
            transactions_root,
            receipts_root,
            state_root,
            logs_bloom,
            withdrawals_root,
        );

        // Step 9: 组装完整区块
        let block = Block {
            header,
            transactions,
            withdrawals: env.withdrawals.clone(),
        };

        Ok(block)
    }

    /// 验证区块 (PoS规则)
    ///
    /// 验证项:
    /// 1. PoS固定字段 (difficulty=0, nonce=0, ommers_hash=empty)
    /// 2. Gas limit调整合法性
    /// 3. Base fee计算正确性
    /// 4. 交易执行正确性
    /// 5. Merkle根正确性
    async fn validate_block(&self, block: &Block) -> Result<(), BlockValidationError> {
        // Step 1: 验证PoS区块头
        block.header.validate_pos_header()?;

        // Step 2: 验证gas limit (需要父区块信息)
        // TODO: 需要从数据库获取父区块
        // GasLimitCalculator::validate_gas_limit(parent_gas_limit, block.header.gas_limit.as_u64())?;

        // Step 3: 验证base fee
        // TODO: 需要父区块信息计算期望的base fee
        // let expected_base_fee = BaseFeeCalculator::calculate_base_fee(...);
        // if block.header.base_fee_per_gas != Some(expected_base_fee) {
        //     return Err(BlockValidationError::InvalidBaseFee { expected, actual });
        // }

        // Step 4: 验证gas used不超过gas limit
        if block.header.gas_used > block.header.gas_limit {
            return Err(BlockValidationError::GasLimitExceeded {
                limit: block.header.gas_limit.as_u64(),
                used: block.header.gas_used.as_u64(),
            });
        }

        // Step 5: 验证交易执行和Merkle根
        // TODO: 重新执行所有交易，验证状态根、交易根、收据根
        // let expected_state_root = self.calculate_state_root();
        // if block.header.state_root != expected_state_root {
        //     return Err(BlockValidationError::InvalidStateRoot { expected, actual });
        // }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::transaction_repo_impl::{TxPoolImpl, TxPoolConfig};
    use ethereum_types::Address;

    #[tokio::test]
    async fn test_build_empty_block() {
        // 创建空交易池
        let tx_pool = Arc::new(TxPoolImpl::new(TxPoolConfig::default()));

        // 创建区块构建服务
        let builder = BuildBlockService::new(tx_pool, Some(30_000_000));

        // 构建环境
        let env = BuildEnvironment {
            parent_hash: H256::zero(),
            parent_number: U64::zero(),
            parent_gas_used: U64::from(15_000_000),
            parent_gas_limit: U64::from(30_000_000),
            parent_base_fee: U256::from(1_000_000_000u64),
            timestamp: U64::from(1234567890),
            fee_recipient: Address::zero(),
            prev_randao: H256::random(),
            withdrawals: vec![],
            parent_beacon_block_root: None,
        };

        // 构建区块
        let block = builder.build_block(env).await.unwrap();

        // 验证
        assert_eq!(block.number(), U64::one());
        assert_eq!(block.transactions.len(), 0); // 空交易
        assert_eq!(block.gas_used(), U64::zero());
        assert!(block.base_fee().is_some());

        // TODO: 集成区块存储后添加保存逻辑
        // let block_repo = BlockRepo::new();
        // block_repo.save(block);
    }

    #[tokio::test]
    async fn test_base_fee_calculation() {
        let tx_pool = Arc::new(TxPoolImpl::new(TxPoolConfig::default()));
        let builder = BuildBlockService::new(tx_pool, None);

        // 父区块使用率 > 50%
        let env_high_usage = BuildEnvironment {
            parent_hash: H256::zero(),
            parent_number: U64::zero(),
            parent_gas_used: U64::from(20_000_000), // 66% usage
            parent_gas_limit: U64::from(30_000_000),
            parent_base_fee: U256::from(1_000_000_000u64),
            timestamp: U64::from(1234567890),
            fee_recipient: Address::zero(),
            prev_randao: H256::zero(),
            withdrawals: vec![],
            parent_beacon_block_root: None,
        };

        let new_base_fee = builder.calculate_base_fee(&env_high_usage);

        // Base fee应该上涨
        assert!(new_base_fee > env_high_usage.parent_base_fee);
    }

    #[tokio::test]
    async fn test_gas_limit_calculation() {
        let tx_pool = Arc::new(TxPoolImpl::new(TxPoolConfig::default()));
        let builder = BuildBlockService::new(tx_pool, None);

        // 高使用率场景
        let env = BuildEnvironment {
            parent_hash: H256::zero(),
            parent_number: U64::zero(),
            parent_gas_used: U64::from(28_000_000), // 93% usage
            parent_gas_limit: U64::from(30_000_000),
            parent_base_fee: U256::from(1_000_000_000u64),
            timestamp: U64::from(1234567890),
            fee_recipient: Address::zero(),
            prev_randao: H256::zero(),
            withdrawals: vec![],
            parent_beacon_block_root: None,
        };

        let new_gas_limit = builder.calculate_gas_limit(&env);

        // Gas limit应该增加
        assert!(new_gas_limit > env.parent_gas_limit.as_u64());

        // 但不能超过1/1024的调整幅度
        let max_increase = env.parent_gas_limit.as_u64() / 1024;
        assert!(new_gas_limit <= env.parent_gas_limit.as_u64() + max_increase);
    }

    #[tokio::test]
    async fn test_validate_pos_block() {
        let tx_pool = Arc::new(TxPoolImpl::new(TxPoolConfig::default()));
        let builder = BuildBlockService::new(tx_pool, None);

        let env = BuildEnvironment {
            parent_hash: H256::zero(),
            parent_number: U64::zero(),
            parent_gas_used: U64::from(15_000_000),
            parent_gas_limit: U64::from(30_000_000),
            parent_base_fee: U256::from(1_000_000_000u64),
            timestamp: U64::from(1234567890),
            fee_recipient: Address::zero(),
            prev_randao: H256::random(),
            withdrawals: vec![],
            parent_beacon_block_root: None,
        };

        // 构建区块
        let block = builder.build_block(env).await.unwrap();

        // 验证区块
        assert!(builder.validate_block(&block).await.is_ok());
    }

    // ========== BaseFeeCalculator 单元测试 ==========

    #[test]
    fn test_base_fee_increase() {
        // 父区块使用率 > 50%，base fee应上涨
        let parent_gas_used = 20_000_000;
        let parent_gas_limit = 30_000_000;
        let parent_base_fee = U256::from(1_000_000_000u64); // 1 Gwei

        let new_base_fee = BaseFeeCalculator::calculate_base_fee(
            parent_gas_used,
            parent_gas_limit,
            parent_base_fee,
        );

        assert!(new_base_fee > parent_base_fee);
    }

    #[test]
    fn test_base_fee_decrease() {
        // 父区块使用率 < 50%，base fee应下降
        let parent_gas_used = 5_000_000;
        let parent_gas_limit = 30_000_000;
        let parent_base_fee = U256::from(1_000_000_000u64);

        let new_base_fee = BaseFeeCalculator::calculate_base_fee(
            parent_gas_used,
            parent_gas_limit,
            parent_base_fee,
        );

        assert!(new_base_fee < parent_base_fee);
    }

    #[test]
    fn test_base_fee_unchanged() {
        // 父区块使用率正好50%，base fee应不变
        let parent_gas_limit = 30_000_000;
        let parent_gas_used = parent_gas_limit / 2;
        let parent_base_fee = U256::from(1_000_000_000u64);

        let new_base_fee = BaseFeeCalculator::calculate_base_fee(
            parent_gas_used,
            parent_gas_limit,
            parent_base_fee,
        );

        assert_eq!(new_base_fee, parent_base_fee);
    }

    #[test]
    fn test_initial_base_fee() {
        let initial = BaseFeeCalculator::initial_base_fee();
        assert_eq!(initial, U256::from(1_000_000_000u64));
    }

    // ========== GasLimitCalculator 单元测试 ==========

    #[test]
    fn test_gas_limit_adjustment() {
        let parent_gas_limit = 30_000_000;
        let parent_gas_used = 28_000_000; // 93% usage

        let new_gas_limit =
            GasLimitCalculator::calculate_gas_limit(parent_gas_used, parent_gas_limit, None);

        // 应该增加，但不超过 parent + parent/1024
        let max_increase = parent_gas_limit / 1024;
        assert!(new_gas_limit <= parent_gas_limit + max_increase);
        assert!(new_gas_limit > parent_gas_limit);
    }

    #[test]
    fn test_gas_limit_validation() {
        let parent_gas_limit = 30_000_000;
        let valid_limit = parent_gas_limit + (parent_gas_limit / 1024);
        let invalid_limit = parent_gas_limit + (parent_gas_limit / 512); // 超过最大调整

        assert!(GasLimitCalculator::validate_gas_limit(parent_gas_limit, valid_limit).is_ok());
        assert!(GasLimitCalculator::validate_gas_limit(parent_gas_limit, invalid_limit).is_err());
    }

    // ========== TransactionSelector 单元测试 ==========

    #[test]
    fn test_transaction_selection() {
        let base_fee = U256::from(1_000_000_000u64); // 1 Gwei
        let gas_limit = 30_000_000;

        let tx1 = DynamicFeeTx {
            chain_id: U64::one(),
            nonce: U64::zero(),
            max_priority_fee_per_gas: U256::from(2_000_000_000u64), // 2 Gwei
            max_fee_per_gas: U256::from(3_000_000_000u64),          // 3 Gwei
            gas_limit: U64::from(21000),
            to: Some(Address::zero()),
            value: U256::zero(),
            data: vec![],
            access_list: vec![],
            v: U64::zero(),
            r: U256::zero(),
            s: U256::zero(),
        };

        let tx2 = DynamicFeeTx {
            max_priority_fee_per_gas: U256::from(1_000_000_000u64), // 1 Gwei (更低)
            ..tx1.clone()
        };

        let candidates = vec![tx2.clone(), tx1.clone()]; // 故意打乱顺序

        let selected = TransactionSelector::select_transactions(candidates, gas_limit, base_fee);

        // 应该优先选择priority fee更高的tx1
        assert_eq!(selected.len(), 2);
        assert_eq!(
            selected[0].max_priority_fee_per_gas,
            tx1.max_priority_fee_per_gas
        );
    }
}
