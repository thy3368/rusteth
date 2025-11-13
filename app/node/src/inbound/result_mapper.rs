//! Domain CommandResult → JSON-RPC Response 映射器
//!
//! 负责将内部领域 CommandResult 转换为外部 JSON 响应格式
//! 遵循 Clean Architecture 原则：
//! - 接口层负责数据格式转换
//! - 将领域对象序列化为 JSON

use crate::domain::commands::CommandResult;
use thiserror::Error;

/// Result 映射错误
#[derive(Debug, Error)]
pub enum ResultMapperError {
    /// JSON 序列化错误
    #[error("JSON 序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// 结果类型不匹配
    #[error("结果类型不匹配: {0}")]
    TypeMismatch(String),
}

/// CommandResult → JSON Value 映射器
pub struct ResultMapper;

impl ResultMapper {
    /// 将领域 CommandResult 转换为 JSON 格式
    ///
    /// # 参数
    /// - `result`: 领域层返回的命令结果
    ///
    /// # 返回
    /// - `Ok(serde_json::Value)`: 转换成功
    /// - `Err(ResultMapperError)`: 转换失败
    pub fn map_to_json(result: CommandResult) -> Result<serde_json::Value, ResultMapperError> {
        match result {
            // 基本类型结果
            CommandResult::Unit => Ok(serde_json::json!(null)),

            CommandResult::Bool(b) => Ok(serde_json::to_value(b)?),

            CommandResult::String(s) => Ok(serde_json::to_value(s)?),

            CommandResult::Bytes(bytes) => {
                // 字节数组转十六进制字符串
                let hex_string = format!("0x{}", hex::encode(bytes));
                Ok(serde_json::to_value(hex_string)?)
            }

            // 以太坊类型结果
            CommandResult::U64(value) => Ok(serde_json::to_value(value)?),

            CommandResult::U256(value) => Ok(serde_json::to_value(value)?),

            CommandResult::Hash(hash) => Ok(serde_json::to_value(hash)?),

            CommandResult::Address(address) => Ok(serde_json::to_value(address)?),

            // 复杂类型结果
            CommandResult::Block(block) => Ok(serde_json::to_value(block)?),

            CommandResult::Transaction(tx) => Ok(serde_json::to_value(tx)?),

            CommandResult::TransactionReceipt(receipt) => Ok(serde_json::to_value(receipt)?),

            CommandResult::Logs(logs) => Ok(serde_json::to_value(logs)?),

            CommandResult::FeeHistory(fee_history) => Ok(serde_json::to_value(fee_history)?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethereum_types::U256;

    #[test]
    fn test_map_unit_result() {
        let result = ResultMapper::map_to_json(CommandResult::Unit);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::json!(null));
    }

    #[test]
    fn test_map_string_result() {
        let result = ResultMapper::map_to_json(CommandResult::String("test".to_string()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::json!("test"));
    }

    #[test]
    fn test_map_u256_result() {
        let result = ResultMapper::map_to_json(CommandResult::U256(U256::from(12345)));
        assert!(result.is_ok());
    }

    #[test]
    fn test_map_bytes_result() {
        let bytes = vec![0x01, 0x02, 0x03];
        let result = ResultMapper::map_to_json(CommandResult::Bytes(bytes));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::json!("0x010203"));
    }
}
