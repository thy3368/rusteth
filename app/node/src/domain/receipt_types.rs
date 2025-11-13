/// 交易收据相关类型定义 - 领域层实体
///
/// 参考标准：
/// - Geth: core/types/receipt.go
/// - EIP-658: 收据中的状态字段
/// - EIP-2718: 类型化交易收据

use ethereum_types::{Address, Bloom, H256, U64};

/// 交易收据
///
/// 记录交易执行的结果和状态变更
/// 参考: geth/core/types/receipt.go
#[repr(align(64))] // 缓存行对齐优化
#[derive(Debug, Clone, PartialEq)]
pub struct TransactionReceipt {
    /// 交易哈希
    pub transaction_hash: H256,
    /// 交易在区块中的索引
    pub transaction_index: U64,
    /// 累计gas使用量 (从区块开始累计)
    pub cumulative_gas_used: U64,
    /// 本交易的gas使用量
    pub gas_used: U64,
    /// 执行状态 (1=成功, 0=失败) - EIP-658
    pub status: U64,
    /// 日志Bloom过滤器 (用于快速日志查询)
    pub logs_bloom: Bloom,
    /// 事件日志列表
    pub logs: Vec<Log>,
}

impl TransactionReceipt {
    /// 创建新的交易收据
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        transaction_hash: H256,
        transaction_index: u64,
        cumulative_gas_used: u64,
        gas_used: u64,
        success: bool,
        logs_bloom: Bloom,
        logs: Vec<Log>,
    ) -> Self {
        Self {
            transaction_hash,
            transaction_index: U64::from(transaction_index),
            cumulative_gas_used: U64::from(cumulative_gas_used),
            gas_used: U64::from(gas_used),
            status: if success { U64::one() } else { U64::zero() },
            logs_bloom,
            logs,
        }
    }

    /// 判断交易是否成功执行
    pub fn is_success(&self) -> bool {
        self.status == U64::one()
    }

    /// 计算收据哈希 (用于Merkle-Patricia Trie)
    ///
    /// TODO: 实现完整的RLP编码和哈希计算
    pub fn hash(&self) -> H256 {
        // 暂时返回零值,后续实现RLP编码
        H256::zero()
    }
}

/// 事件日志
///
/// EVM合约事件的日志记录
/// 参考: geth/core/types/log.go
#[derive(Debug, Clone, PartialEq)]
pub struct Log {
    /// 合约地址 (发出事件的合约)
    pub address: Address,
    /// 主题列表 (最多4个,第一个通常是事件签名)
    pub topics: Vec<H256>,
    /// 日志数据 (ABI编码的参数)
    pub data: Vec<u8>,
}

impl Log {
    /// 创建新的日志
    pub fn new(address: Address, topics: Vec<H256>, data: Vec<u8>) -> Self {
        Self {
            address,
            topics,
            data,
        }
    }

    /// 获取事件签名 (第一个topic)
    pub fn event_signature(&self) -> Option<H256> {
        self.topics.first().copied()
    }

    /// 验证主题数量是否合法 (最多4个)
    pub fn validate_topics_count(&self) -> Result<(), ReceiptValidationError> {
        if self.topics.len() > 4 {
            return Err(ReceiptValidationError::TooManyTopics {
                max: 4,
                actual: self.topics.len(),
            });
        }
        Ok(())
    }
}

/// 收据验证错误
#[derive(Debug, Clone, PartialEq)]
pub enum ReceiptValidationError {
    /// 主题数量过多
    TooManyTopics { max: usize, actual: usize },
    /// 无效的状态值
    InvalidStatus { actual: U64 },
    /// Gas使用量不一致
    GasUsedMismatch { expected: u64, actual: u64 },
}

impl std::fmt::Display for ReceiptValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::TooManyTopics { max, actual } => {
                write!(f, "Too many topics: max {}, got {}", max, actual)
            }
            Self::InvalidStatus { actual } => {
                write!(f, "Invalid status value: {} (must be 0 or 1)", actual)
            }
            Self::GasUsedMismatch { expected, actual } => {
                write!(f, "Gas used mismatch: expected {}, got {}", expected, actual)
            }
        }
    }
}

impl std::error::Error for ReceiptValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_creation() {
        let receipt = TransactionReceipt::new(
            H256::random(),
            0,
            21000,
            21000,
            true,
            Bloom::zero(),
            vec![],
        );

        assert!(receipt.is_success());
        assert_eq!(receipt.gas_used, U64::from(21000));
        assert_eq!(receipt.transaction_index, U64::zero());
    }

    #[test]
    fn test_receipt_failure() {
        let receipt = TransactionReceipt::new(
            H256::random(),
            1,
            50000,
            50000,
            false,
            Bloom::zero(),
            vec![],
        );

        assert!(!receipt.is_success());
        assert_eq!(receipt.status, U64::zero());
    }

    #[test]
    fn test_log_creation() {
        let log = Log::new(Address::random(), vec![H256::random()], vec![1, 2, 3, 4]);

        assert_eq!(log.topics.len(), 1);
        assert_eq!(log.data.len(), 4);
        assert!(log.event_signature().is_some());
    }

    #[test]
    fn test_log_topics_validation() {
        // 合法的主题数量
        let valid_log = Log::new(
            Address::random(),
            vec![H256::random(), H256::random(), H256::random()],
            vec![],
        );
        assert!(valid_log.validate_topics_count().is_ok());

        // 过多的主题
        let invalid_log = Log::new(
            Address::random(),
            vec![
                H256::random(),
                H256::random(),
                H256::random(),
                H256::random(),
                H256::random(), // 第5个主题
            ],
            vec![],
        );
        assert!(invalid_log.validate_topics_count().is_err());
    }

    #[test]
    fn test_event_signature() {
        let event_sig = H256::random();
        let log = Log::new(Address::random(), vec![event_sig, H256::random()], vec![]);

        assert_eq!(log.event_signature(), Some(event_sig));
    }

    #[test]
    fn test_empty_topics() {
        let log = Log::new(Address::random(), vec![], vec![1, 2, 3]);

        assert_eq!(log.event_signature(), None);
        assert!(log.validate_topics_count().is_ok());
    }
}
