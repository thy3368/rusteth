/// 区块构建器接口 - 遵循Clean Architecture原则
///
/// 设计依据：
/// - EIP-1559: 费用市场和base fee动态调整
/// - EIP-3675: PoS共识机制
/// - EIP-4399: PREVRANDAO随机数
/// - Geth miner/worker.go: 区块构建流程
///
/// 参考文档：
/// - /consensus-specs/specs/gloas/builder.md
/// - /EIPs/EIPS/eip-1559.md
/// - /EIPs/EIPS/eip-3675.md

use crate::domain::block_types::{Block, BlockValidationError, BuildEnvironment};
use crate::domain::tx_types::DynamicFeeTx;
use async_trait::async_trait;
use ethereum_types::U256;

/// 区块构建器接口
///
/// 职责：
/// 1. 从交易池选择交易 (按价格排序，装箱问题)
/// 2. 计算新区块的base fee (EIP-1559动态调整)
/// 3. 计算新区块的gas limit (动态调整，最大变化1/1024)
/// 4. 执行交易并更新状态
/// 5. 计算状态根、交易根、收据根
/// 6. 构建完整区块
///
/// 流程参考 geth/miner/worker.go:commitTransactions
#[async_trait]
pub trait BlockBuilder: Send + Sync {
    /// 构建新区块
    ///
    /// 步骤:
    /// 1. 计算base fee和gas limit
    /// 2. 从交易池获取候选交易
    /// 3. 选择并执行交易 (贪心算法 + 装箱)
    /// 4. 计算Merkle根和状态根
    /// 5. 组装区块头和区块
    async fn build_block(&self, env: BuildEnvironment) -> Result<Block, BlockValidationError>;

    /// 验证区块 (PoS规则)
    ///
    /// 参考 EIP-3675 和 geth/consensus/beacon/consensus.go
    async fn validate_block(&self, block: &Block) -> Result<(), BlockValidationError>;
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use ethereum_types::{Address, U64};

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

    #[test]
    fn test_initial_base_fee() {
        let initial = BaseFeeCalculator::initial_base_fee();
        assert_eq!(initial, U256::from(1_000_000_000u64));
    }
}
