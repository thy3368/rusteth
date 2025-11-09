//! EIP-1474 规范的 JSON-RPC 客户端实现
//!
//! 本模块提供高性能的以太坊 JSON-RPC 客户端,用于调用远端 RPC 服务。
//! 遵循整洁架构原则,位于基础设施层,实现了 `EthApiExecutor` trait。

use crate::inbound::eth_api_trait::EthApiExecutor;
use crate::inbound::json_rpc::RpcMethodError;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// JSON-RPC 请求结构
#[derive(Debug, Clone, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    method: String,
    params: Value,
    id: u64,
}

/// JSON-RPC 响应结构
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: Option<u64>,  // 错误响应中 id 可能为 null
}

/// JSON-RPC 错误结构
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// 以太坊 JSON-RPC 客户端
///
/// 高性能实现,遵循低延迟标准:
/// - 连接池复用减少握手开销
/// - 原子递增的请求 ID 避免锁竞争
/// - 可配置的超时和重试策略
pub struct EthApiClient {
    /// HTTP 客户端(内部使用连接池)
    client: Client,
    /// 远端 RPC 端点 URL
    rpc_url: String,
    /// 请求 ID 计数器(原子递增)
    request_id: Arc<AtomicU64>,
}

impl EthApiClient {
    /// 创建新的 JSON-RPC 客户端
    ///
    /// # 参数
    /// - `rpc_url`: 远端 RPC 端点 URL (例如: "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY")
    ///
    /// # 性能优化
    /// - 自动协商 HTTP/2 或 HTTP/1.1
    /// - 配置连接池(默认最多 10 个连接)
    /// - 设置合理的超时时间(30秒)
    pub fn new(rpc_url: String) -> Result<Self, RpcMethodError> {
        let client = Client::builder()
            .pool_max_idle_per_host(10) // 连接池优化
            .timeout(Duration::from_secs(30)) // 请求超时
            .pool_idle_timeout(Duration::from_secs(90)) // 空闲连接保持时间
            .build()
            .map_err(|e| RpcMethodError::InvalidParams(format!("创建 HTTP 客户端失败: {}", e)))?;

        Ok(Self {
            client,
            rpc_url,
            request_id: Arc::new(AtomicU64::new(1)),
        })
    }

    /// 发送 JSON-RPC 请求
    ///
    /// # 低延迟设计
    /// - 原子递增请求 ID (无锁竞争)
    /// - 直接序列化避免中间分配
    /// - 异步非阻塞 I/O
    async fn send_request(&self, method: &str, params: Value) -> Result<Value, RpcMethodError> {
        // 原子递增请求 ID
        let id = self.request_id.fetch_add(1, Ordering::Relaxed);

        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            method: method.to_string(),
            params,
            id,
        };

        // 发送 HTTP POST 请求
        let response = self
            .client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                RpcMethodError::InvalidParams(format!("HTTP 请求失败: {}", e))
            })?;

        // 检查 HTTP 状态码
        if !response.status().is_success() {
            return Err(RpcMethodError::InvalidParams(format!(
                "HTTP 错误: {}",
                response.status()
            )));
        }

        // 解析 JSON-RPC 响应
        let rpc_response: JsonRpcResponse = response.json().await.map_err(|e| {
            RpcMethodError::InvalidParams(format!("解析响应失败: {}", e))
        })?;

        // 处理 JSON-RPC 错误
        if let Some(error) = rpc_response.error {
            return Err(RpcMethodError::InvalidParams(format!(
                "RPC 错误 [{}]: {}",
                error.code, error.message
            )));
        }

        // 返回结果
        rpc_response.result.ok_or_else(|| {
            RpcMethodError::InvalidParams("响应中缺少 result 字段".to_string())
        })
    }
}

#[async_trait]
impl EthApiExecutor for EthApiClient {
    /// eth_blockNumber - 返回当前区块号
    async fn eth_block_number(&self) -> Result<Value, RpcMethodError> {
        self.send_request("eth_blockNumber", serde_json::json!([])).await
    }

    /// eth_getBlockByNumber - 根据区块号返回区块信息
    async fn eth_get_block_by_number(&self, params: Value) -> Result<Value, RpcMethodError> {
        self.send_request("eth_getBlockByNumber", params).await
    }

    /// eth_getBlockByHash - 根据区块哈希返回区块信息
    async fn eth_get_block_by_hash(&self, params: Value) -> Result<Value, RpcMethodError> {
        self.send_request("eth_getBlockByHash", params).await
    }

    /// eth_getTransactionByHash - 根据交易哈希返回交易信息
    async fn eth_get_transaction_by_hash(&self, params: Value) -> Result<Value, RpcMethodError> {
        self.send_request("eth_getTransactionByHash", params).await
    }

    /// eth_getTransactionReceipt - 根据交易哈希返回交易收据
    async fn eth_get_transaction_receipt(&self, params: Value) -> Result<Value, RpcMethodError> {
        self.send_request("eth_getTransactionReceipt", params).await
    }

    /// eth_getBalance - 返回账户余额
    async fn eth_get_balance(&self, params: Value) -> Result<Value, RpcMethodError> {
        self.send_request("eth_getBalance", params).await
    }

    /// eth_getStorageAt - 返回指定位置的存储值
    async fn eth_get_storage_at(&self, params: Value) -> Result<Value, RpcMethodError> {
        self.send_request("eth_getStorageAt", params).await
    }

    /// eth_getTransactionCount - 返回账户的交易数量(nonce)
    async fn eth_get_transaction_count(&self, params: Value) -> Result<Value, RpcMethodError> {
        self.send_request("eth_getTransactionCount", params).await
    }

    /// eth_getCode - 返回合约代码
    async fn eth_get_code(&self, params: Value) -> Result<Value, RpcMethodError> {
        self.send_request("eth_getCode", params).await
    }

    /// eth_call - 执行调用(不创建交易)
    async fn eth_call(&self, params: Value) -> Result<Value, RpcMethodError> {
        self.send_request("eth_call", params).await
    }

    /// eth_estimateGas - 估算交易的 Gas 消耗
    async fn eth_estimate_gas(&self, params: Value) -> Result<Value, RpcMethodError> {
        self.send_request("eth_estimateGas", params).await
    }

    /// eth_getLogs - 返回匹配过滤器的日志
    async fn eth_get_logs(&self, params: Value) -> Result<Value, RpcMethodError> {
        self.send_request("eth_getLogs", params).await
    }

    /// eth_chainId - 返回链 ID
    async fn eth_chain_id(&self) -> Result<Value, RpcMethodError> {
        self.send_request("eth_chainId", serde_json::json!([])).await
    }

    /// eth_gasPrice - 返回当前 Gas 价格
    async fn eth_gas_price(&self) -> Result<Value, RpcMethodError> {
        self.send_request("eth_gasPrice", serde_json::json!([])).await
    }

    /// net_version - 返回网络 ID
    async fn net_version(&self) -> Result<Value, RpcMethodError> {
        self.send_request("net_version", serde_json::json!([])).await
    }

    /// web3_clientVersion - 返回客户端版本
    async fn web3_client_version(&self) -> Result<Value, RpcMethodError> {
        self.send_request("web3_clientVersion", serde_json::json!([])).await
    }
}
