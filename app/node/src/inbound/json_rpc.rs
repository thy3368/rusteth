//! 基于 EIP-1474 EIP-1559 的以太坊 JSON-RPC 实现
//!
//! 本模块根据 EIP-1474 EIP-1559 规范实现以太坊 JSON-RPC 2.0 接口。
//! 架构遵循整洁架构（Clean Architecture）原则，明确分离各层职责。

use async_trait::async_trait;
use axum::handler::Handler;
use ethereum_types::U64;
use thiserror::Error;
use crate::inbound::json_rpc_trait::EthApiExecutor;
use crate::inbound::json_types::{error_codes, JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::service::ethereum_service::{EthereumService, ServiceError};
use crate::service::types::{BlockId, BlockTag, CallRequest, FilterOptions, SendTransactionRequest};
// ============================================================================
// 用例层 - JSON-RPC 方法处理器
// ============================================================================

/// JSON-RPC 主处理器（遵循整洁架构，使用静态分发）
#[derive(Clone)]
pub struct EthJsonRpcHandler<R> {
    pub(crate) service: R,
}

impl<R: EthereumService> EthJsonRpcHandler<R> {
    pub fn new(service: R) -> Self {
        Self { service }
    }

    /// JSON-RPC 请求主分发方法
    pub async fn handle(&self, request: JsonRpcRequest) -> JsonRpcResponse
    where
        Self: EthApiExecutor,
    {
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
    async fn execute_method(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, RpcMethodError>
    where
        Self: EthApiExecutor,
    {
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

            // EIP-1559 相关方法
            "eth_sendTransaction" => self.eth_send_transaction(params).await,
            "eth_sendRawTransaction" => self.eth_send_raw_transaction(params).await,
            "eth_feeHistory" => self.eth_fee_history(params).await,
            "eth_maxPriorityFeePerGas" => self.eth_max_priority_fee_per_gas().await,

            _ => Err(RpcMethodError::MethodNotFound(method.to_string())),
        }
    }

    /// 将内部错误映射为 JSON-RPC 错误
    fn map_error(&self, error: RpcMethodError) -> JsonRpcError {
        match error {
            RpcMethodError::MethodNotFound(method) => JsonRpcError {
                code: error_codes::METHOD_NOT_FOUND,
                message: format!("方法未找到: {}", method),
                data: None,
            },
            RpcMethodError::InvalidParams(msg) => JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: format!("无效参数: {}", msg),
                data: None,
            },
            RpcMethodError::ServiceError(err) => JsonRpcError {
                code: error_codes::SERVER_ERROR,
                message: format!("服务器错误: {}", err),
                data: None,
            },
            RpcMethodError::SerializationError(err) => JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("内部错误: {}", err),
                data: None,
            },
            RpcMethodError::UnsupportedFeature(feature) => JsonRpcError {
                code: error_codes::SERVER_ERROR,
                message: format!("不支持的功能: {}", feature),
                data: None,
            },
        }
    }
}

#[async_trait]
impl<R: EthereumService> EthApiExecutor for EthJsonRpcHandler<R> {
    /// eth_blockNumber - 返回当前区块号
    async fn eth_block_number(&self) -> Result<serde_json::Value, RpcMethodError> {
        let block_number = self.service.get_block_number().await?;
        Ok(serde_json::to_value(block_number)?)
    }

    /// eth_getBlockByNumber - 根据区块号返回区块信息
    async fn eth_get_block_by_number(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (BlockId, bool) = serde_json::from_value(params)?;
        let block_number = match params.0 {
            BlockId::Number(num) => num,
            BlockId::Tag(BlockTag::Latest) => self.service.get_block_number().await?,
            BlockId::Tag(BlockTag::Earliest) => U64::zero(),
            BlockId::Tag(BlockTag::Pending) => {
                return Err(RpcMethodError::UnsupportedFeature("待处理区块".to_string()))
            }
        };

        let block = self.service.get_block_by_number(block_number, params.1).await?;
        Ok(serde_json::to_value(block)?)
    }

    /// eth_getBlockByHash - 根据区块哈希返回区块信息
    async fn eth_get_block_by_hash(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (ethereum_types::H256, bool) = serde_json::from_value(params)?;
        let block = self.service.get_block_by_hash(params.0, params.1).await?;
        Ok(serde_json::to_value(block)?)
    }

    /// eth_getTransactionByHash - 根据交易哈希返回交易信息
    async fn eth_get_transaction_by_hash(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (ethereum_types::H256,) = serde_json::from_value(params)?;
        let tx = self.service.get_transaction_by_hash(params.0).await?;
        Ok(serde_json::to_value(tx)?)
    }

    /// eth_getTransactionReceipt - 根据交易哈希返回交易收据
    async fn eth_get_transaction_receipt(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (ethereum_types::H256,) = serde_json::from_value(params)?;
        let receipt = self.service.get_transaction_receipt(params.0).await?;
        Ok(serde_json::to_value(receipt)?)
    }

    /// eth_getBalance - 返回账户余额
    async fn eth_get_balance(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (ethereum_types::Address, BlockId) = serde_json::from_value(params)?;
        let balance = self.service.get_balance(params.0, params.1).await?;
        Ok(serde_json::to_value(balance)?)
    }

    /// eth_getStorageAt - 返回指定位置的存储值
    async fn eth_get_storage_at(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (ethereum_types::Address, ethereum_types::U256, BlockId) =
            serde_json::from_value(params)?;
        let value = self.service.get_storage_at(params.0, params.1, params.2).await?;
        Ok(serde_json::to_value(value)?)
    }

    /// eth_getTransactionCount - 返回账户的交易数量（nonce）
    async fn eth_get_transaction_count(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (ethereum_types::Address, BlockId) = serde_json::from_value(params)?;
        let count = self.service.get_transaction_count(params.0, params.1).await?;
        Ok(serde_json::to_value(count)?)
    }

    /// eth_getCode - 返回合约代码
    async fn eth_get_code(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (ethereum_types::Address, BlockId) = serde_json::from_value(params)?;
        let code = self.service.get_code(params.0, params.1).await?;
        Ok(serde_json::to_value(hex::encode(code))?)
    }

    /// eth_call - 执行调用（不创建交易）
    async fn eth_call(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (CallRequest, BlockId) =
            serde_json::from_value(params)?;
        let result = self.service.call(params.0, params.1).await?;
        Ok(serde_json::to_value(hex::encode(result))?)
    }

    /// eth_estimateGas - 估算交易的 Gas 消耗
    async fn eth_estimate_gas(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (CallRequest,) =
            serde_json::from_value(params)?;
        let gas = self.service.estimate_gas(params.0).await?;
        Ok(serde_json::to_value(gas)?)
    }

    /// eth_getLogs - 返回匹配过滤器的日志
    async fn eth_get_logs(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (FilterOptions,) =
            serde_json::from_value(params)?;
        let logs = self.service.get_logs(params.0).await?;
        Ok(serde_json::to_value(logs)?)
    }

    /// eth_chainId - 返回链 ID
    async fn eth_chain_id(&self) -> Result<serde_json::Value, RpcMethodError> {
        Ok(serde_json::to_value(U64::from(1))?) // 主网 = 1
    }

    /// eth_gasPrice - 返回当前 Gas 价格
    async fn eth_gas_price(&self) -> Result<serde_json::Value, RpcMethodError> {
        Ok(serde_json::to_value(ethereum_types::U256::from(20_000_000_000u64))?) // 20 Gwei
    }

    /// net_version - 返回网络 ID
    async fn net_version(&self) -> Result<serde_json::Value, RpcMethodError> {
        Ok(serde_json::to_value("1")?)
    }

    /// web3_clientVersion - 返回客户端版本
    async fn web3_client_version(&self) -> Result<serde_json::Value, RpcMethodError> {
        Ok(serde_json::to_value("rusteth/0.1.0")?)
    }

    // EIP-1559 相关方法实现

    /// eth_sendTransaction - 发送交易
    async fn eth_send_transaction(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (SendTransactionRequest,) =
            serde_json::from_value(params)?;
        let tx_hash = self.service.send_transaction(params.0).await?;
        Ok(serde_json::to_value(tx_hash)?)
    }

    /// eth_sendRawTransaction - 发送已签名的原始交易
    async fn eth_send_raw_transaction(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (String,) = serde_json::from_value(params)?;
        // 解析十六进制字符串
        let raw_tx = hex::decode(params.0.trim_start_matches("0x"))
            .map_err(|e| RpcMethodError::InvalidParams(format!("无效的十六进制数据: {}", e)))?;
        let tx_hash = self.service.send_raw_transaction(raw_tx).await?;
        Ok(serde_json::to_value(tx_hash)?)
    }

    /// eth_feeHistory - 返回历史费用信息
    async fn eth_fee_history(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (U64, BlockId, Option<Vec<f64>>) =
            serde_json::from_value(params)?;
        let fee_history = self.service
            .fee_history(params.0, params.1, params.2)
            .await?;
        Ok(serde_json::to_value(fee_history)?)
    }

    /// eth_maxPriorityFeePerGas - 返回建议的最大优先费用
    async fn eth_max_priority_fee_per_gas(&self) -> Result<serde_json::Value, RpcMethodError> {
        let fee = self.service.max_priority_fee_per_gas().await?;
        Ok(serde_json::to_value(fee)?)
    }
}

/// RPC 方法错误类型
#[derive(Debug, Error)]
pub enum RpcMethodError {
    #[error("方法未找到: {0}")]
    MethodNotFound(String),
    #[error("无效参数: {0}")]
    InvalidParams(String),
    #[error("服务错误: {0}")]
    ServiceError(#[from] ServiceError),
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
    use crate::inbound::json_types::RequestId;
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
