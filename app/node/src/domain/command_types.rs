//! 领域层 - 以太坊类型和命令定义
//!
//! 本模块包含：
//! 1. 以太坊核心数据结构（符合 EIP-1474 和 EIP-1559）
//! 2. CQRS 命令定义
//! 3. 命令错误类型
//!
//! ## 架构原则
//! - 纯领域对象，不依赖外部框架
//! - 缓存行对齐优化性能
//! - 遵循 CQRS 模式

use ethereum_types::{Address, Bloom, H256, H64, U256, U64};
use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// 命令错误类型
// ============================================================================

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

// ============================================================================
// 核心以太坊类型
// ============================================================================

/// 区块标识符 - 可以是区块号、"latest"、"earliest"、"pending"
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockId {
    Number(U64),
    Tag(BlockTag),
}

/// 区块标签枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlockTag {
    Latest,   // 最新区块
    Earliest, // 创世区块
    Pending,  // 待处理区块
}

/// 以太坊区块结构（符合 EIP-1474，缓存行对齐优化性能）
#[repr(align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub number: U64,             // 区块号
    pub hash: H256,              // 区块哈希
    pub parent_hash: H256,       // 父区块哈希
    pub nonce: H64,              // 工作量证明随机数
    pub sha3_uncles: H256,       // 叔块哈希
    pub logs_bloom: Bloom,       // 日志布隆过滤器
    pub transactions_root: H256, // 交易树根
    pub state_root: H256,        // 状态树根
    pub receipts_root: H256,     // 收据树根
    pub miner: Address,          // 矿工地址
    pub difficulty: U256,        // 难度
    pub total_difficulty: U256,  // 总难度
    #[serde(with = "hex_bytes")]
    pub extra_data: Vec<u8>, // 额外数据（十六进制字符串）
    pub size: U256,              // 区块大小
    pub gas_limit: U256,         // Gas 限制
    pub gas_used: U256,          // 已使用 Gas
    pub timestamp: U256,         // 时间戳
    pub transactions: Vec<Transaction>, // 交易列表
    pub uncles: Vec<H256>,       // 叔块哈希列表
}

/// 以太坊交易结构（符合 EIP-1474 和 EIP-1559，缓存行对齐）
#[repr(align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub hash: H256,                     // 交易哈希
    pub nonce: U256,                    // 发送方交易序号
    pub block_hash: Option<H256>,       // 所属区块哈希
    pub block_number: Option<U64>,      // 所属区块号
    pub transaction_index: Option<U64>, // 区块中的交易索引
    pub from: Address,                  // 发送方地址
    pub to: Option<Address>,            // 接收方地址（合约创建时为 None）
    pub value: U256,                    // 转账金额（wei）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<U256>, // Gas 价格（Legacy 交易使用）
    pub gas: U256,                      // Gas 限制
    #[serde(with = "hex_bytes")]
    pub input: Vec<u8>, // 输入数据（十六进制字符串）
    pub v: U64,                         // 签名 v 值
    pub r: U256,                        // 签名 r 值
    pub s: U256,                        // 签名 s 值
    // EIP-1559 字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<U256>, // EIP-1559: 每 gas 最大费用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<U256>, // EIP-1559: 每 gas 最大优先费用
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub transaction_type: Option<U64>, // 交易类型（0=Legacy, 2=EIP-1559）
}

/// 交易收据结构（符合 EIP-1474）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    pub transaction_hash: H256,            // 交易哈希
    pub transaction_index: U64,            // 交易索引
    pub block_hash: H256,                  // 区块哈希
    pub block_number: U64,                 // 区块号
    pub from: Address,                     // 发送方地址
    pub to: Option<Address>,               // 接收方地址
    pub cumulative_gas_used: U256,         // 累计使用的 Gas
    pub gas_used: U256,                    // 本交易使用的 Gas
    pub contract_address: Option<Address>, // 合约地址（如果是合约创建）
    pub logs: Vec<Log>,                    // 日志列表
    pub logs_bloom: Bloom,                 // 日志布隆过滤器
    pub status: U64,                       // 交易状态（1=成功，0=失败）
}

/// 事件日志结构（符合 EIP-1474）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    pub removed: bool,           // 是否因链重组被移除
    pub log_index: U256,         // 日志索引
    pub transaction_index: U256, // 交易索引
    pub transaction_hash: H256,  // 交易哈希
    pub block_hash: H256,        // 区块哈希
    pub block_number: U64,       // 区块号
    pub address: Address,        // 合约地址
    #[serde(with = "hex_bytes")]
    pub data: Vec<u8>, // 日志数据（十六进制字符串）
    pub topics: Vec<H256>,       // 日志主题
}

/// 调用/交易参数（符合 EIP-1474 和 EIP-1559）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallRequest {
    pub from: Option<Address>,   // 发送方地址（可选）
    pub to: Option<Address>,     // 目标地址（合约创建时为None）
    pub gas: Option<U256>,       // Gas 限制（可选）
    pub gas_price: Option<U256>, // Gas 价格（Legacy，可选）
    pub value: Option<U256>,     // 转账金额（可选）
    #[serde(default, with = "hex_data")]
    pub data: Option<Vec<u8>>, // 调用数据（十六进制字符串，可选）
    // EIP-1559 字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<U256>, // EIP-1559: 每 gas 最大费用（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<U256>, // EIP-1559: 每 gas 最大优先费用（可选）
}

/// 日志过滤器参数（符合 EIP-1474）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterOptions {
    pub from_block: Option<BlockId>,       // 起始区块
    pub to_block: Option<BlockId>,         // 结束区块
    pub address: Option<Address>,          // 合约地址过滤
    pub topics: Option<Vec<Option<H256>>>, // 主题过滤
}

/// EIP-1559 费用历史结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeHistory {
    pub oldest_block: U64,           // 最旧区块号
    pub base_fee_per_gas: Vec<U256>, // 每个区块的基础费用
    pub gas_used_ratio: Vec<f64>,    // 每个区块的 gas 使用比率
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reward: Option<Vec<Vec<U256>>>, // 可选：每个区块的奖励百分位数
}

/// 发送交易请求（用于 eth_sendTransaction）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendTransactionRequest {
    pub from: Address,           // 发送方地址
    pub to: Option<Address>,     // 接收方地址（合约创建时为 None）
    pub gas: Option<U256>,       // Gas 限制（可选）
    pub gas_price: Option<U256>, // Gas 价格（Legacy，可选）
    pub value: Option<U256>,     // 转账金额（可选）
    #[serde(default, with = "hex_data")]
    pub data: Option<Vec<u8>>, // 交易数据（可选）
    pub nonce: Option<U256>,     // Nonce（可选）
    // EIP-1559 字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<U256>, // EIP-1559: 最大费用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<U256>, // EIP-1559: 最大优先费用
}

// ============================================================================
// 序列化辅助模块
// ============================================================================

/// 自定义序列化模块：处理十六进制字符串和可选字节数组的转换
mod hex_data {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(data: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match data {
            Some(bytes) => {
                let hex_string = format!("0x{}", hex::encode(bytes));
                serializer.serialize_str(&hex_string)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) => {
                let s = s.trim_start_matches("0x");
                if s.is_empty() {
                    Ok(Some(vec![]))
                } else {
                    hex::decode(s).map(Some).map_err(serde::de::Error::custom)
                }
            }
            None => Ok(None),
        }
    }
}

/// 自定义序列化模块：处理十六进制字符串和必需字节数组的转换
mod hex_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_string = format!("0x{}", hex::encode(data));
        serializer.serialize_str(&hex_string)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        let s = s.trim_start_matches("0x");
        if s.is_empty() {
            Ok(vec![])
        } else {
            hex::decode(s).map_err(serde::de::Error::custom)
        }
    }
}

// ============================================================================
// CQRS 命令定义
// ============================================================================

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
    Block(Option<Block>),

    /// 交易信息
    Transaction(Option<Transaction>),

    /// 交易收据
    TransactionReceipt(Option<TransactionReceipt>),

    /// 日志列表
    Logs(Vec<Log>),

    /// 费用历史
    FeeHistory(FeeHistory),
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

        let cmd = EthCommand::GetBalance(Address::zero(), BlockId::Tag(BlockTag::Latest));
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
