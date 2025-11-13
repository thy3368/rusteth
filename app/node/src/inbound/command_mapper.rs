//! JSON-RPC Request → Domain Command 映射器
//!
//! 负责将外部 JSON-RPC 请求转换为内部领域 Command 对象
//! 遵循 Clean Architecture 原则：
//! - 接口层负责数据格式转换
//! - 领域层处理业务逻辑
//! - 两层之间通过 Command 进行解耦

use crate::domain::commands::EthCommand;
use crate::service::types::{BlockId, BlockTag, CallRequest, FilterOptions, SendTransactionRequest};
use ethereum_types::{Address, H256, U256, U64};
use thiserror::Error;

/// Command 映射错误
#[derive(Debug, Error)]
pub enum CommandMapperError {
    /// JSON 解析错误
    #[error("JSON 解析错误: {0}")]
    JsonError(#[from] serde_json::Error),

    /// 方法不支持
    #[error("不支持的方法: {0}")]
    UnsupportedMethod(String),

    /// 参数无效
    #[error("参数无效: {0}")]
    InvalidParams(String),
}

/// JSON-RPC 请求转 Domain Command 映射器
pub struct CommandMapper;

impl CommandMapper {
    /// 将 JSON-RPC 方法和参数转换为领域 Command
    ///
    /// # 参数
    /// - `method`: JSON-RPC 方法名
    /// - `params`: JSON 格式的参数
    ///
    /// # 返回
    /// - `Ok(EthCommand)`: 转换成功
    /// - `Err(CommandMapperError)`: 转换失败
    pub fn map_to_command(
        method: &str,
        params: serde_json::Value,
    ) -> Result<EthCommand, CommandMapperError> {
        match method {
            // 区块查询方法
            "eth_blockNumber" => Ok(EthCommand::GetBlockNumber),

            "eth_getBlockByNumber" => {
                let params: (BlockId, bool) = serde_json::from_value(params)?;
                Ok(EthCommand::GetBlockByNumber(params.0, params.1))
            }

            "eth_getBlockByHash" => {
                let params: (H256, bool) = serde_json::from_value(params)?;
                Ok(EthCommand::GetBlockByHash(params.0, params.1))
            }

            // 交易查询方法
            "eth_getTransactionByHash" => {
                let params: (H256,) = serde_json::from_value(params)?;
                Ok(EthCommand::GetTransactionByHash(params.0))
            }

            "eth_getTransactionReceipt" => {
                let params: (H256,) = serde_json::from_value(params)?;
                Ok(EthCommand::GetTransactionReceipt(params.0))
            }

            // 账户状态查询方法
            "eth_getBalance" => {
                let params: (Address, BlockId) = serde_json::from_value(params)?;
                Ok(EthCommand::GetBalance(params.0, params.1))
            }

            "eth_getStorageAt" => {
                let params: (Address, U256, BlockId) = serde_json::from_value(params)?;
                Ok(EthCommand::GetStorageAt(params.0, params.1, params.2))
            }

            "eth_getTransactionCount" => {
                let params: (Address, BlockId) = serde_json::from_value(params)?;
                Ok(EthCommand::GetTransactionCount(params.0, params.1))
            }

            "eth_getCode" => {
                let params: (Address, BlockId) = serde_json::from_value(params)?;
                Ok(EthCommand::GetCode(params.0, params.1))
            }

            // 合约调用方法
            "eth_call" => {
                let params: (CallRequest, BlockId) = serde_json::from_value(params)?;
                Ok(EthCommand::Call(params.0, params.1))
            }

            "eth_estimateGas" => {
                let params: (CallRequest,) = serde_json::from_value(params)?;
                Ok(EthCommand::EstimateGas(params.0))
            }

            "eth_getLogs" => {
                let params: (FilterOptions,) = serde_json::from_value(params)?;
                Ok(EthCommand::GetLogs(params.0))
            }

            // 网络信息查询方法
            "eth_chainId" => Ok(EthCommand::GetChainId),

            "eth_gasPrice" => Ok(EthCommand::GetGasPrice),

            "net_version" => Ok(EthCommand::GetNetVersion),

            "web3_clientVersion" => Ok(EthCommand::GetClientVersion),

            // EIP-1559 交易方法
            "eth_sendTransaction" => {
                let params: (SendTransactionRequest,) = serde_json::from_value(params)?;
                Ok(EthCommand::SendTransaction(params.0))
            }

            "eth_sendRawTransaction" => {
                let params: (String,) = serde_json::from_value(params)?;

                // 解析十六进制字符串
                let raw_tx = hex::decode(params.0.trim_start_matches("0x")).map_err(|e| {
                    CommandMapperError::InvalidParams(format!("无效的十六进制数据: {}", e))
                })?;

                // 签名恢复 - TODO: 实现真实的签名恢复
                let sender = Address::from_low_u64_be(0x9999); // Mock sender

                Ok(EthCommand::SendRawTransaction(raw_tx, sender))
            }

            "eth_feeHistory" => {
                let params: (U64, BlockId, Option<Vec<f64>>) = serde_json::from_value(params)?;
                Ok(EthCommand::GetFeeHistory(params.0, params.1, params.2))
            }

            "eth_maxPriorityFeePerGas" => Ok(EthCommand::GetMaxPriorityFeePerGas),

            // 不支持的方法
            _ => Err(CommandMapperError::UnsupportedMethod(method.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_simple_command() {
        let result = CommandMapper::map_to_command("eth_blockNumber", serde_json::json!([]));
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), EthCommand::GetBlockNumber));
    }

    #[test]
    fn test_map_command_with_params() {
        let params = serde_json::json!(["0x0000000000000000000000000000000000000000", "latest"]);
        let result = CommandMapper::map_to_command("eth_getBalance", params);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), EthCommand::GetBalance(..)));
    }

    #[test]
    fn test_unsupported_method() {
        let result = CommandMapper::map_to_command("unsupported_method", serde_json::json!([]));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CommandMapperError::UnsupportedMethod(_)
        ));
    }
}
