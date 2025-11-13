//! 以太坊 JSON-RPC API 接口定义
//!
//! 本模块定义了符合 EIP-1474 规范的以太坊 JSON-RPC 方法接口。
//! 这是一个端口（Port）定义，遵循整洁架构的依赖倒置原则。

use async_trait::async_trait;
use thiserror::Error;

/// RPC 方法错误类型（客户端使用）
#[derive(Debug, Error)]
pub enum RpcMethodError {
    #[error("方法未找到: {0}")]
    MethodNotFound(String),
    #[error("无效参数: {0}")]
    InvalidParams(String),
    #[error("服务错误: {0}")]
    ServiceError(String),
    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("不支持的功能: {0}")]
    UnsupportedFeature(String),
    #[error("内部错误: {0}")]
    InternalError(String),
}

/// 以太坊 JSON-RPC API 执行接口
///
/// 定义了所有符合 EIP-1474 规范的 JSON-RPC 方法。
/// 实现此 trait 的类型需要提供具体的业务逻辑。
#[async_trait]
pub trait EthApiExecutor: Send + Sync {
    /// eth_blockNumber - 返回当前区块号
    async fn eth_block_number(&self) -> Result<serde_json::Value, RpcMethodError>;

    /// eth_getBlockByNumber - 根据区块号返回区块信息
    async fn eth_get_block_by_number(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError>;

    /// eth_getBlockByHash - 根据区块哈希返回区块信息
    async fn eth_get_block_by_hash(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError>;

    /// eth_getTransactionByHash - 根据交易哈希返回交易信息
    async fn eth_get_transaction_by_hash(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError>;

    /// eth_getTransactionReceipt - 根据交易哈希返回交易收据
    async fn eth_get_transaction_receipt(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError>;

    /// eth_getBalance - 返回账户余额
    async fn eth_get_balance(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError>;

    /// eth_getStorageAt - 返回指定位置的存储值
    async fn eth_get_storage_at(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError>;

    /// eth_getTransactionCount - 返回账户的交易数量（nonce）
    async fn eth_get_transaction_count(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError>;

    /// eth_getCode - 返回合约代码
    async fn eth_get_code(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError>;

    /// eth_call - 执行调用（不创建交易）
    async fn eth_call(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError>;

    /// eth_estimateGas - 估算交易的 Gas 消耗
    async fn eth_estimate_gas(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError>;

    /// eth_getLogs - 返回匹配过滤器的日志
    async fn eth_get_logs(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError>;

    /// eth_chainId - 返回链 ID
    async fn eth_chain_id(&self) -> Result<serde_json::Value, RpcMethodError>;

    /// eth_gasPrice - 返回当前 Gas 价格
    async fn eth_gas_price(&self) -> Result<serde_json::Value, RpcMethodError>;

    /// net_version - 返回网络 ID
    async fn net_version(&self) -> Result<serde_json::Value, RpcMethodError>;

    /// web3_clientVersion - 返回客户端版本
    async fn web3_client_version(&self) -> Result<serde_json::Value, RpcMethodError>;

    // EIP-1559 相关方法

    /// eth_sendTransaction - 发送交易（返回交易哈希）
    async fn eth_send_transaction(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError>;

    /// eth_sendRawTransaction - 发送已签名的原始交易
    async fn eth_send_raw_transaction(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError>;

    /// eth_feeHistory - 返回历史费用信息（EIP-1559）
    async fn eth_fee_history(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError>;

    /// eth_maxPriorityFeePerGas - 返回建议的最大优先费用（EIP-1559）
    async fn eth_max_priority_fee_per_gas(&self) -> Result<serde_json::Value, RpcMethodError>;
}
