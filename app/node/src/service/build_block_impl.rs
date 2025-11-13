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
use crate::service::build_block_trait::{
    BaseFeeCalculator, BlockBuilder, GasLimitCalculator, TransactionSelector,
};
use crate::service::repo::transaction_repo::TxPool;
use async_trait::async_trait;
use ethereum_types::{Bloom, H256, U256, U64};
use std::sync::Arc;

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
}
