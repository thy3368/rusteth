//! 基于 EIP-1474 的以太坊 JSON-RPC 实现
//!
//! 本模块根据 EIP-1474 规范实现以太坊 JSON-RPC 2.0 接口。
//! 架构遵循整洁架构（Clean Architecture）原则，明确分离各层职责。

use async_trait::async_trait;
use ethereum_types::{Address, Bloom, H256, H64, U256, U64};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

// ============================================================================
// 领域层 - 核心类型（符合 EIP-1474 规范）
// ============================================================================

/// JSON-RPC 2.0 请求结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: RequestId,
}

/// JSON-RPC 2.0 响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcResponse {
    Success {
        jsonrpc: String,
        result: serde_json::Value,
        id: RequestId,
    },
    Error {
        jsonrpc: String,
        error: JsonRpcError,
        id: RequestId,
    },
}

/// 请求 ID（可以是字符串、数字或 null）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum RequestId {
    Number(u64),
    String(String),
    Null,
}

/// JSON-RPC 错误结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// 标准 JSON-RPC 错误代码（EIP-1474 规范）
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;        // 解析错误
    pub const INVALID_REQUEST: i32 = -32600;    // 无效请求
    pub const METHOD_NOT_FOUND: i32 = -32601;   // 方法未找到
    pub const INVALID_PARAMS: i32 = -32602;     // 无效参数
    pub const INTERNAL_ERROR: i32 = -32603;     // 内部错误
    pub const SERVER_ERROR: i32 = -32000;       // 服务器错误
}

// ============================================================================
// 以太坊类型（EIP-1474 数据结构）
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
    Latest,     // 最新区块
    Earliest,   // 创世区块
    Pending,    // 待处理区块
}

/// 以太坊区块结构（符合 EIP-1474，缓存行对齐优化性能）
#[repr(align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub number: U64,                      // 区块号
    pub hash: H256,                       // 区块哈希
    pub parent_hash: H256,                // 父区块哈希
    pub nonce: H64,                       // 工作量证明随机数
    pub sha3_uncles: H256,                // 叔块哈希
    pub logs_bloom: Bloom,                // 日志布隆过滤器
    pub transactions_root: H256,          // 交易树根
    pub state_root: H256,                 // 状态树根
    pub receipts_root: H256,              // 收据树根
    pub miner: Address,                   // 矿工地址
    pub difficulty: U256,                 // 难度
    pub total_difficulty: U256,           // 总难度
    #[serde(with = "hex_bytes")]
    pub extra_data: Vec<u8>,              // 额外数据（十六进制字符串）
    pub size: U256,                       // 区块大小
    pub gas_limit: U256,                  // Gas 限制
    pub gas_used: U256,                   // 已使用 Gas
    pub timestamp: U256,                  // 时间戳
    pub transactions: Vec<Transaction>,   // 交易列表
    pub uncles: Vec<H256>,                // 叔块哈希列表
}

/// 以太坊交易结构（符合 EIP-1474，缓存行对齐）
#[repr(align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub hash: H256,                       // 交易哈希
    pub nonce: U256,                      // 发送方交易序号
    pub block_hash: Option<H256>,         // 所属区块哈希
    pub block_number: Option<U64>,        // 所属区块号
    pub transaction_index: Option<U64>,   // 区块中的交易索引
    pub from: Address,                    // 发送方地址
    pub to: Option<Address>,              // 接收方地址（合约创建时为 None）
    pub value: U256,                      // 转账金额（wei）
    pub gas_price: U256,                  // Gas 价格
    pub gas: U256,                        // Gas 限制
    #[serde(with = "hex_bytes")]
    pub input: Vec<u8>,                   // 输入数据（十六进制字符串）
    pub v: U64,                           // 签名 v 值
    pub r: U256,                          // 签名 r 值
    pub s: U256,                          // 签名 s 值
}

/// 交易收据结构（符合 EIP-1474）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    pub transaction_hash: H256,           // 交易哈希
    pub transaction_index: U64,           // 交易索引
    pub block_hash: H256,                 // 区块哈希
    pub block_number: U64,                // 区块号
    pub from: Address,                    // 发送方地址
    pub to: Option<Address>,              // 接收方地址
    pub cumulative_gas_used: U256,        // 累计使用的 Gas
    pub gas_used: U256,                   // 本交易使用的 Gas
    pub contract_address: Option<Address>, // 合约地址（如果是合约创建）
    pub logs: Vec<Log>,                   // 日志列表
    pub logs_bloom: Bloom,                // 日志布隆过滤器
    pub status: U64,                      // 交易状态（1=成功，0=失败）
}

/// 事件日志结构（符合 EIP-1474）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    pub removed: bool,                    // 是否因链重组被移除
    pub log_index: U256,                  // 日志索引
    pub transaction_index: U256,          // 交易索引
    pub transaction_hash: H256,           // 交易哈希
    pub block_hash: H256,                 // 区块哈希
    pub block_number: U64,                // 区块号
    pub address: Address,                 // 合约地址
    #[serde(with = "hex_bytes")]
    pub data: Vec<u8>,                    // 日志数据（十六进制字符串）
    pub topics: Vec<H256>,                // 日志主题
}

/// 调用/交易参数（符合 EIP-1474）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallRequest {
    pub from: Option<Address>,            // 发送方地址（可选）
    pub to: Address,                      // 目标地址
    pub gas: Option<U256>,                // Gas 限制（可选）
    pub gas_price: Option<U256>,          // Gas 价格（可选）
    pub value: Option<U256>,              // 转账金额（可选）
    #[serde(default, with = "hex_data")]
    pub data: Option<Vec<u8>>,            // 调用数据（十六进制字符串，可选）
}

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
                    hex::decode(s)
                        .map(Some)
                        .map_err(serde::de::Error::custom)
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

/// 日志过滤器参数（符合 EIP-1474）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterOptions {
    pub from_block: Option<BlockId>,      // 起始区块
    pub to_block: Option<BlockId>,        // 结束区块
    pub address: Option<Address>,         // 合约地址过滤
    pub topics: Option<Vec<Option<H256>>>, // 主题过滤
}

// ============================================================================
// 领域层 - 仓储接口（端口模式）
// ============================================================================

/// 以太坊状态仓储接口（符合 EIP-1474 规范）
#[async_trait]
pub trait EthereumRepository: Send + Sync {
    /// 获取当前区块号
    async fn get_block_number(&self) -> Result<U64, RepositoryError>;

    /// 根据区块号获取区块
    async fn get_block_by_number(&self, number: U64, full_tx: bool) -> Result<Option<Block>, RepositoryError>;

    /// 根据区块哈希获取区块
    async fn get_block_by_hash(&self, hash: H256, full_tx: bool) -> Result<Option<Block>, RepositoryError>;

    /// 根据交易哈希获取交易
    async fn get_transaction_by_hash(&self, hash: H256) -> Result<Option<Transaction>, RepositoryError>;

    /// 根据交易哈希获取交易收据
    async fn get_transaction_receipt(&self, hash: H256) -> Result<Option<TransactionReceipt>, RepositoryError>;

    /// 获取账户余额
    async fn get_balance(&self, address: Address, block: BlockId) -> Result<U256, RepositoryError>;

    /// 获取存储位置的值
    async fn get_storage_at(&self, address: Address, position: U256, block: BlockId) -> Result<H256, RepositoryError>;

    /// 获取账户交易数量（nonce）
    async fn get_transaction_count(&self, address: Address, block: BlockId) -> Result<U256, RepositoryError>;

    /// 获取合约代码
    async fn get_code(&self, address: Address, block: BlockId) -> Result<Vec<u8>, RepositoryError>;

    /// 执行调用（不创建交易）
    async fn call(&self, request: CallRequest, block: BlockId) -> Result<Vec<u8>, RepositoryError>;

    /// 估算 Gas 消耗
    async fn estimate_gas(&self, request: CallRequest) -> Result<U256, RepositoryError>;

    /// 根据过滤器获取日志
    async fn get_logs(&self, filter: FilterOptions) -> Result<Vec<Log>, RepositoryError>;
}

/// 仓储层错误类型
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("区块未找到")]
    BlockNotFound,
    #[error("交易未找到")]
    TransactionNotFound,
    #[error("内部错误: {0}")]
    InternalError(String),
}

// ============================================================================
// 用例层 - JSON-RPC 方法处理器
// ============================================================================

/// JSON-RPC 主处理器（遵循整洁架构）
pub struct EthJsonRpcHandler {
    repository: Arc<dyn EthereumRepository>,
}

impl EthJsonRpcHandler {
    pub fn new(repository: Arc<dyn EthereumRepository>) -> Self {
        Self { repository }
    }

    /// JSON-RPC 请求主分发方法
    pub async fn handle(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone();

        match self.execute_method(&request.method, request.params).await {
            Ok(result) => JsonRpcResponse::Success {
                jsonrpc: "2.0".to_string(),
                result,
                id,
            },
            Err(error) => JsonRpcResponse::Error {
                jsonrpc: "2.0".to_string(),
                error: self.map_error(error),
                id,
            },
        }
    }

    /// 执行特定的 RPC 方法
    async fn execute_method(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, RpcMethodError> {
        match method {
            // EIP-1474 标准方法
            "eth_blockNumber" => self.eth_block_number().await,
            "eth_getBlockByNumber" => self.eth_get_block_by_number(params).await,
            "eth_getBlockByHash" => self.eth_get_block_by_hash(params).await,
            "eth_getTransactionByHash" => self.eth_get_transaction_by_hash(params).await,
            "eth_getTransactionReceipt" => self.eth_get_transaction_receipt(params).await,
            "eth_getBalance" => self.eth_get_balance(params).await,
            "eth_getStorageAt" => self.eth_get_storage_at(params).await,
            "eth_getTransactionCount" => self.eth_get_transaction_count(params).await,
            "eth_getCode" => self.eth_get_code(params).await,
            "eth_call" => self.eth_call(params).await,
            "eth_estimateGas" => self.eth_estimate_gas(params).await,
            "eth_getLogs" => self.eth_get_logs(params).await,
            "eth_chainId" => self.eth_chain_id().await,
            "eth_gasPrice" => self.eth_gas_price().await,
            "net_version" => self.net_version().await,
            "web3_clientVersion" => self.web3_client_version().await,

            _ => Err(RpcMethodError::MethodNotFound(method.to_string())),
        }
    }

    // ========================================================================
    // EIP-1474 方法实现
    // ========================================================================

    /// eth_blockNumber - 返回当前区块号
    async fn eth_block_number(&self) -> Result<serde_json::Value, RpcMethodError> {
        let block_number = self.repository.get_block_number().await?;
        Ok(serde_json::to_value(block_number)?)
    }

    /// eth_getBlockByNumber - 根据区块号返回区块信息
    async fn eth_get_block_by_number(&self, params: serde_json::Value) -> Result<serde_json::Value, RpcMethodError> {
        let params: (BlockId, bool) = serde_json::from_value(params)?;
        let block_number = match params.0 {
            BlockId::Number(num) => num,
            BlockId::Tag(BlockTag::Latest) => self.repository.get_block_number().await?,
            BlockId::Tag(BlockTag::Earliest) => U64::zero(),
            BlockId::Tag(BlockTag::Pending) => return Err(RpcMethodError::UnsupportedFeature("待处理区块".to_string())),
        };

        let block = self.repository.get_block_by_number(block_number, params.1).await?;
        Ok(serde_json::to_value(block)?)
    }

    /// eth_getBlockByHash - 根据区块哈希返回区块信息
    async fn eth_get_block_by_hash(&self, params: serde_json::Value) -> Result<serde_json::Value, RpcMethodError> {
        let params: (H256, bool) = serde_json::from_value(params)?;
        let block = self.repository.get_block_by_hash(params.0, params.1).await?;
        Ok(serde_json::to_value(block)?)
    }

    /// eth_getTransactionByHash - 根据交易哈希返回交易信息
    async fn eth_get_transaction_by_hash(&self, params: serde_json::Value) -> Result<serde_json::Value, RpcMethodError> {
        let params: (H256,) = serde_json::from_value(params)?;
        let tx = self.repository.get_transaction_by_hash(params.0).await?;
        Ok(serde_json::to_value(tx)?)
    }

    /// eth_getTransactionReceipt - 根据交易哈希返回交易收据
    async fn eth_get_transaction_receipt(&self, params: serde_json::Value) -> Result<serde_json::Value, RpcMethodError> {
        let params: (H256,) = serde_json::from_value(params)?;
        let receipt = self.repository.get_transaction_receipt(params.0).await?;
        Ok(serde_json::to_value(receipt)?)
    }

    /// eth_getBalance - 返回账户余额
    async fn eth_get_balance(&self, params: serde_json::Value) -> Result<serde_json::Value, RpcMethodError> {
        let params: (Address, BlockId) = serde_json::from_value(params)?;
        let balance = self.repository.get_balance(params.0, params.1).await?;
        Ok(serde_json::to_value(balance)?)
    }

    /// eth_getStorageAt - 返回指定位置的存储值
    async fn eth_get_storage_at(&self, params: serde_json::Value) -> Result<serde_json::Value, RpcMethodError> {
        let params: (Address, U256, BlockId) = serde_json::from_value(params)?;
        let value = self.repository.get_storage_at(params.0, params.1, params.2).await?;
        Ok(serde_json::to_value(value)?)
    }

    /// eth_getTransactionCount - 返回账户的交易数量（nonce）
    async fn eth_get_transaction_count(&self, params: serde_json::Value) -> Result<serde_json::Value, RpcMethodError> {
        let params: (Address, BlockId) = serde_json::from_value(params)?;
        let count = self.repository.get_transaction_count(params.0, params.1).await?;
        Ok(serde_json::to_value(count)?)
    }

    /// eth_getCode - 返回合约代码
    async fn eth_get_code(&self, params: serde_json::Value) -> Result<serde_json::Value, RpcMethodError> {
        let params: (Address, BlockId) = serde_json::from_value(params)?;
        let code = self.repository.get_code(params.0, params.1).await?;
        Ok(serde_json::to_value(hex::encode(code))?)
    }

    /// eth_call - 执行调用（不创建交易）
    async fn eth_call(&self, params: serde_json::Value) -> Result<serde_json::Value, RpcMethodError> {
        let params: (CallRequest, BlockId) = serde_json::from_value(params)?;
        let result = self.repository.call(params.0, params.1).await?;
        Ok(serde_json::to_value(hex::encode(result))?)
    }

    /// eth_estimateGas - 估算交易的 Gas 消耗
    async fn eth_estimate_gas(&self, params: serde_json::Value) -> Result<serde_json::Value, RpcMethodError> {
        let params: (CallRequest,) = serde_json::from_value(params)?;
        let gas = self.repository.estimate_gas(params.0).await?;
        Ok(serde_json::to_value(gas)?)
    }

    /// eth_getLogs - 返回匹配过滤器的日志
    async fn eth_get_logs(&self, params: serde_json::Value) -> Result<serde_json::Value, RpcMethodError> {
        let params: (FilterOptions,) = serde_json::from_value(params)?;
        let logs = self.repository.get_logs(params.0).await?;
        Ok(serde_json::to_value(logs)?)
    }

    /// eth_chainId - 返回链 ID
    async fn eth_chain_id(&self) -> Result<serde_json::Value, RpcMethodError> {
        Ok(serde_json::to_value(U64::from(1))?) // 主网 = 1
    }

    /// eth_gasPrice - 返回当前 Gas 价格
    async fn eth_gas_price(&self) -> Result<serde_json::Value, RpcMethodError> {
        Ok(serde_json::to_value(U256::from(20_000_000_000u64))?) // 20 Gwei
    }

    /// net_version - 返回网络 ID
    async fn net_version(&self) -> Result<serde_json::Value, RpcMethodError> {
        Ok(serde_json::to_value("1")?)
    }

    /// web3_clientVersion - 返回客户端版本
    async fn web3_client_version(&self) -> Result<serde_json::Value, RpcMethodError> {
        Ok(serde_json::to_value("rusteth/0.1.0")?)
    }

    /// 将内部错误映射为 JSON-RPC 错误
    fn map_error(&self, error: RpcMethodError) -> JsonRpcError {
        match error {
            RpcMethodError::MethodNotFound(method) => JsonRpcError {
                code: -32601,
                message: format!("方法未找到: {}", method),
                data: None,
            },
            RpcMethodError::InvalidParams(msg) => JsonRpcError {
                code: -32602,
                message: format!("无效参数: {}", msg),
                data: None,
            },
            RpcMethodError::RepositoryError(err) => JsonRpcError {
                code: -32000,
                message: format!("服务器错误: {}", err),
                data: None,
            },
            RpcMethodError::SerializationError(err) => JsonRpcError {
                code: -32603,
                message: format!("内部错误: {}", err),
                data: None,
            },
            RpcMethodError::UnsupportedFeature(feature) => JsonRpcError {
                code: -32000,
                message: format!("不支持的功能: {}", feature),
                data: None,
            },
        }
    }
}

/// RPC 方法错误类型
#[derive(Debug, Error)]
pub enum RpcMethodError {
    #[error("方法未找到: {0}")]
    MethodNotFound(String),
    #[error("无效参数: {0}")]
    InvalidParams(String),
    #[error("仓储错误: {0}")]
    RepositoryError(#[from] RepositoryError),
    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("不支持的功能: {0}")]
    UnsupportedFeature(String),
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_serialization() {
        // 测试请求 ID 的序列化
        let id_num = RequestId::Number(1);
        let json = serde_json::to_string(&id_num).unwrap();
        assert_eq!(json, "1");

        let id_str = RequestId::String("test".to_string());
        let json = serde_json::to_string(&id_str).unwrap();
        assert_eq!(json, "\"test\"");
    }

    #[test]
    fn test_block_id_parsing() {
        // 测试区块标识符的解析
        let latest: BlockId = serde_json::from_str("\"latest\"").unwrap();
        assert!(matches!(latest, BlockId::Tag(BlockTag::Latest)));

        let number: BlockId = serde_json::from_str("\"0x1\"").unwrap();
        assert!(matches!(number, BlockId::Number(_)));
    }
}
