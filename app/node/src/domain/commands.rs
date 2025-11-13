//! 领域层 - Command 定义（CQRS模式）
//!
//! 遵循CQRS（Command Query Responsibility Segregation）原则：
//! - Command: 表示改变系统状态的操作（写操作）
//! - Query: 表示读取系统状态的操作（读操作）
//!
//! ## 架构原则
//! - 纯领域对象，不依赖外部框架
//! - 不包含业务逻辑，仅作为数据载体
//! - 使用 enum 实现多态（静态分发）
//! - 支持 Send + Sync（线程安全）

use crate::domain::command_types::{BlockId, CallRequest, FilterOptions, SendTransactionRequest};
use ethereum_types::{Address, H256, U256, U64};
use std::fmt;

/// 命令处理错误
#[derive(Debug)]
pub enum CommandError {
    /// 不支持的命令
    UnsupportedCommand(String),
    /// 无效的参数
    InvalidParams(String),
    /// 资源未找到
    NotFound(String),
    /// 验证错误
    ValidationError(String),
    /// 内部错误
    InternalError(String),
    /// 网络错误
    NetworkError(String),
    /// 数据库错误
    DatabaseError(String),
    /// 超时
    Timeout(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedCommand(msg) => write!(f, "不支持的命令: {}", msg),
            Self::InvalidParams(msg) => write!(f, "无效参数: {}", msg),
            Self::NotFound(msg) => write!(f, "资源未找到: {}", msg),
            Self::ValidationError(msg) => write!(f, "验证失败: {}", msg),
            Self::InternalError(msg) => write!(f, "内部错误: {}", msg),
            Self::NetworkError(msg) => write!(f, "网络错误: {}", msg),
            Self::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            Self::Timeout(msg) => write!(f, "超时: {}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

/// 从 ServiceError 转换为 CommandError
impl From<crate::service::ethereum_service::ServiceError> for CommandError {
    fn from(err: crate::service::ethereum_service::ServiceError) -> Self {
        use crate::service::ethereum_service::ServiceError;
        match err {
            ServiceError::BlockNotFound => Self::NotFound("区块未找到".to_string()),
            ServiceError::TransactionNotFound => Self::NotFound("交易未找到".to_string()),
            ServiceError::ValidationError(msg) => Self::ValidationError(msg),
            ServiceError::InternalError(msg) => Self::InternalError(msg),
            ServiceError::Other(msg) => Self::InternalError(msg),
        }
    }
}

/// 以太坊 RPC 命令
///
/// 封装所有以太坊 JSON-RPC 操作的命令对象
/// 遵循命令模式（Command Pattern），将请求封装为对象
///
/// # 分类
/// - 查询命令（Query）: 只读操作，不改变状态
/// - 交易命令（Transaction）: 写操作，改变区块链状态
#[derive(Debug, Clone)]
pub enum EthCommand {
    // ========================================================================
    // 区块查询命令
    // ========================================================================
    /// 获取当前区块号
    GetBlockNumber,

    /// 根据区块号获取区块
    /// (区块ID, 是否返回完整交易)
    GetBlockByNumber(BlockId, bool),

    /// 根据区块哈希获取区块
    /// (区块哈希, 是否返回完整交易)
    GetBlockByHash(H256, bool),

    // ========================================================================
    // 交易查询命令
    // ========================================================================
    /// 根据交易哈希获取交易
    GetTransactionByHash(H256),

    /// 根据交易哈希获取交易收据
    GetTransactionReceipt(H256),

    // ========================================================================
    // 账户状态查询命令
    // ========================================================================
    /// 获取账户余额
    /// (地址, 区块ID)
    GetBalance(Address, BlockId),

    /// 获取存储值
    /// (地址, 存储位置, 区块ID)
    GetStorageAt(Address, U256, BlockId),

    /// 获取账户交易计数（nonce）
    /// (地址, 区块ID)
    GetTransactionCount(Address, BlockId),

    /// 获取合约代码
    /// (地址, 区块ID)
    GetCode(Address, BlockId),

    // ========================================================================
    // 合约调用命令
    // ========================================================================
    /// 执行只读合约调用
    /// (调用请求, 区块ID)
    Call(CallRequest, BlockId),

    /// 估算交易 Gas 消耗
    EstimateGas(CallRequest),

    /// 获取日志
    GetLogs(FilterOptions),

    // ========================================================================
    // 网络信息查询命令
    // ========================================================================
    /// 获取链 ID
    GetChainId,

    /// 获取当前 Gas 价格
    GetGasPrice,

    /// 获取网络版本
    GetNetVersion,

    /// 获取客户端版本
    GetClientVersion,

    // ========================================================================
    // EIP-1559 交易命令
    // ========================================================================
    /// 发送交易（需要签名）
    SendTransaction(SendTransactionRequest),

    /// 发送原始交易（已签名）
    /// (原始交易字节, 发送者地址)
    SendRawTransaction(Vec<u8>, Address),

    /// 获取费用历史
    /// (区块数量, 结束区块, 奖励百分位数)
    GetFeeHistory(U64, BlockId, Option<Vec<f64>>),

    /// 获取建议的最大优先费用
    GetMaxPriorityFeePerGas,
}

/// 命令执行结果
///
/// 封装命令执行后的返回值
/// 使用 enum 实现类型安全的多态返回值
#[derive(Debug, Clone)]
pub enum CommandResult {
    // ========================================================================
    // 基本类型结果
    // ========================================================================
    /// 空结果（无返回值）
    Unit,

    /// 布尔值
    Bool(bool),

    /// 字符串
    String(String),

    /// 字节数组
    Bytes(Vec<u8>),

    // ========================================================================
    // 以太坊类型结果
    // ========================================================================
    /// 64位无符号整数（区块号、nonce等）
    U64(U64),

    /// 256位无符号整数（余额、Gas价格等）
    U256(U256),

    /// 哈希值
    Hash(H256),

    /// 地址
    Address(Address),

    // ========================================================================
    // 复杂类型结果
    // ========================================================================
    /// 区块信息
    Block(Option<crate::domain::command_types::Block>),

    /// 交易信息
    Transaction(Option<crate::domain::command_types::Transaction>),

    /// 交易收据
    TransactionReceipt(Option<crate::domain::command_types::TransactionReceipt>),

    /// 日志列表
    Logs(Vec<crate::domain::command_types::Log>),

    /// 费用历史
    FeeHistory(crate::domain::command_types::FeeHistory),
}

impl EthCommand {
    /// 获取命令名称（用于日志和调试）
    pub fn name(&self) -> &'static str {
        match self {
            Self::GetBlockNumber => "eth_blockNumber",
            Self::GetBlockByNumber(..) => "eth_getBlockByNumber",
            Self::GetBlockByHash(..) => "eth_getBlockByHash",
            Self::GetTransactionByHash(..) => "eth_getTransactionByHash",
            Self::GetTransactionReceipt(..) => "eth_getTransactionReceipt",
            Self::GetBalance(..) => "eth_getBalance",
            Self::GetStorageAt(..) => "eth_getStorageAt",
            Self::GetTransactionCount(..) => "eth_getTransactionCount",
            Self::GetCode(..) => "eth_getCode",
            Self::Call(..) => "eth_call",
            Self::EstimateGas(..) => "eth_estimateGas",
            Self::GetLogs(..) => "eth_getLogs",
            Self::GetChainId => "eth_chainId",
            Self::GetGasPrice => "eth_gasPrice",
            Self::GetNetVersion => "net_version",
            Self::GetClientVersion => "web3_clientVersion",
            Self::SendTransaction(..) => "eth_sendTransaction",
            Self::SendRawTransaction(..) => "eth_sendRawTransaction",
            Self::GetFeeHistory(..) => "eth_feeHistory",
            Self::GetMaxPriorityFeePerGas => "eth_maxPriorityFeePerGas",
        }
    }

    /// 判断是否为写操作（会改变状态）
    pub fn is_write_operation(&self) -> bool {
        matches!(
            self,
            Self::SendTransaction(_) | Self::SendRawTransaction(_, _)
        )
    }

    /// 判断是否为读操作（只读）
    pub fn is_read_operation(&self) -> bool {
        !self.is_write_operation()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_name() {
        let cmd = EthCommand::GetBlockNumber;
        assert_eq!(cmd.name(), "eth_blockNumber");

        let cmd = EthCommand::GetBalance(
            Address::zero(),
            BlockId::Tag(crate::domain::command_types::BlockTag::Latest),
        );
        assert_eq!(cmd.name(), "eth_getBalance");
    }

    #[test]
    fn test_command_classification() {
        let read_cmd = EthCommand::GetBlockNumber;
        assert!(read_cmd.is_read_operation());
        assert!(!read_cmd.is_write_operation());

        let write_cmd = EthCommand::SendTransaction(SendTransactionRequest {
            from: Address::zero(),
            to: Some(Address::zero()),
            gas: None,
            gas_price: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            value: None,
            data: None,
            nonce: None,
        });
        assert!(write_cmd.is_write_operation());
        assert!(!write_cmd.is_read_operation());
    }
}
