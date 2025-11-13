/// 交易内存池接口 - 遵循Clean Architecture原则
/// 参考 geth txpool 和相关EIP标准
///
/// 设计原则：
/// - 无状态接口：TxPool不保存状态，只定义行为
/// - 线程安全：所有方法都是Send + Sync
/// - 异步操作：支持高并发场景

use crate::domain::entity_types::DynamicFeeTx;
use async_trait::async_trait;
use ethereum_types::{Address, H256};

/// 交易池错误
#[derive(Debug, Clone, PartialEq)]
pub enum TxPoolError {
    /// 交易已存在
    AlreadyExists(H256),
    /// 交易池已满
    PoolFull { current: usize, max: usize },
    /// Nonce间隙（当前nonce之前有未处理的交易）
    NonceGap { expected: u64, actual: u64 },
    /// 替换交易gas价格过低
    ReplacementUnderpriced { current: String, required: String },
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for TxPoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::AlreadyExists(hash) => write!(f, "Transaction already exists: {:?}", hash),
            Self::PoolFull { current, max } => {
                write!(f, "Transaction pool full: {}/{}", current, max)
            }
            Self::NonceGap { expected, actual } => {
                write!(f, "Nonce gap: expected {}, got {}", expected, actual)
            }
            Self::ReplacementUnderpriced { current, required } => {
                write!(f, "Replacement underpriced: current {}, required {}", current, required)
            }
            Self::Other(msg) => write!(f, "TxPool error: {}", msg),
        }
    }
}

impl std::error::Error for TxPoolError {}

/// 交易池统计信息
#[derive(Debug, Clone, PartialEq)]
pub struct TxPoolStats {
    /// 待处理交易数
    pub pending: usize,
    /// 队列中交易数（nonce间隙）
    pub queued: usize,
    /// 总容量
    pub capacity: usize,
}

/// 交易内存池接口
///
/// 交易状态管理：
/// - Pending: 可以被打包的交易（nonce连续）
/// - Queued: 等待中的交易（nonce有间隙）
#[async_trait]
pub trait TxPool: Send + Sync {
    /// 添加新交易到池中
    ///
    /// 行为：
    /// - 如果nonce连续，放入pending
    /// - 如果nonce有间隙，放入queued
    /// - 如果是替换交易，检查gas价格是否足够高
    async fn add(&self, tx: DynamicFeeTx, sender: Address) -> Result<H256, TxPoolError>;

    /// 根据哈希获取交易
    async fn get(&self, hash: &H256) -> Result<Option<DynamicFeeTx>, TxPoolError>;

    /// 获取账户的所有待处理交易
    async fn get_pending_by_sender(&self, sender: Address) -> Result<Vec<DynamicFeeTx>, TxPoolError>;

    /// 获取可打包的交易（按gas价格排序）
    ///
    /// 参数：
    /// - max_count: 最多返回多少笔交易
    /// - base_fee: 当前区块的base fee，用于过滤
    async fn get_pending(&self, max_count: usize, base_fee: Option<u64>) -> Result<Vec<DynamicFeeTx>, TxPoolError>;

    /// 移除交易（已打包或过期）
    async fn remove(&self, hash: &H256) -> Result<(), TxPoolError>;

    /// 批量移除交易
    async fn remove_batch(&self, hashes: &[H256]) -> Result<(), TxPoolError>;

    /// 获取池统计信息
    async fn stats(&self) -> Result<TxPoolStats, TxPoolError>;

    /// 清空交易池
    async fn clear(&self) -> Result<(), TxPoolError>;
}
