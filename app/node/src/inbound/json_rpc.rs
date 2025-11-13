//! 基于 EIP-1474 EIP-1559 的以太坊 JSON-RPC 实现
//!
//! 本模块根据 EIP-1474 EIP-1559 规范实现以太坊 JSON-RPC 2.0 接口。
//! 架构遵循整洁架构（Clean Architecture）原则，明确分离各层职责。

use crate::domain::command_dispatcher::CommandDispatcher;
use crate::domain::commands::CommandError;
use crate::inbound::command_mapper::{CommandMapper, CommandMapperError};
use crate::inbound::json_types::{error_codes, JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::inbound::result_mapper::{ResultMapper, ResultMapperError};
use crate::service::ethereum_service::EthereumService;

// ============================================================================
// 用例层 - JSON-RPC 方法处理器
// ============================================================================

/// JSON-RPC 主处理器
///
/// # CQRS 架构
/// 采用命令查询职责分离（CQRS）模式：
/// 1. JSON-RPC Request → Domain Command (通过 CommandMapper)
/// 2. Domain Command → Dispatcher.ask() → CommandResult (领域层处理)
/// 3. CommandResult → JSON Response (通过 ResultMapper)
#[derive(Clone)]
pub struct EthJsonRpcHandler<S: EthereumService> {
    dispatcher: CommandDispatcher<S>,
    // TODO: 增加 command_repo 用于命令持久化/审计/溯源
    // command_repo: Arc<dyn CommandRepository>,
}

impl<S: EthereumService> EthJsonRpcHandler<S> {
    pub fn new(dispatcher: CommandDispatcher<S>) -> Self {
        Self {
            dispatcher,
            // TODO: 传入 command_repo 参数
        }
    }

    /// JSON-RPC 请求主分发方法（CQRS 模式）
    ///
    /// # 处理流程
    /// ```text
    /// JSON-RPC Request
    ///     ↓
    /// [CommandMapper] 转换为领域 Command
    ///     ↓
    /// [Handler.ask()] 执行命令，返回 CommandResult
    ///     ↓
    /// [ResultMapper] 转换为 JSON Response
    ///     ↓
    /// JSON-RPC Response
    /// ```
    pub async fn handle(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone();

        // Step 1: 将 JSON-RPC request 转换为领域 Command
        let command = match CommandMapper::map_to_command(&request.method, request.params) {
            Ok(cmd) => cmd,
            Err(err) => {
                return JsonRpcResponse::Error {
                    jsonrpc: "2.0".to_string(),
                    error: Self::map_mapper_error(err),
                    id,
                }
            }
        };

        // TODO: 增加 commandRepo 用于命令持久化
        // let commandRepo = self.command_repo.clone();
        // commandRepo.save(&command).await?;

        // Step 2: 通过 Dispatcher 处理命令（Erlang 风格的 ask）
        // Dispatcher 内部会动态查找并分发命令到具体的 Handler
        let result = self.dispatcher.ask(command).await;

        // Step 3: 将 CommandResult 转换为 JSON Response
        match result {
            Ok(command_result) => match ResultMapper::map_to_json(command_result) {
                Ok(json_value) => JsonRpcResponse::Success {
                    jsonrpc: "2.0".to_string(),
                    result: json_value,
                    id,
                },
                Err(err) => JsonRpcResponse::Error {
                    jsonrpc: "2.0".to_string(),
                    error: Self::map_result_error(err),
                    id,
                },
            },
            Err(err) => JsonRpcResponse::Error {
                jsonrpc: "2.0".to_string(),
                error: Self::map_command_error(err),
                id,
            },
        }
    }

    /// 将 CommandMapperError 映射为 JSON-RPC 错误
    fn map_mapper_error(error: CommandMapperError) -> JsonRpcError {
        match error {
            CommandMapperError::UnsupportedMethod(method) => JsonRpcError {
                code: error_codes::METHOD_NOT_FOUND,
                message: format!("方法未找到: {}", method),
                data: None,
            },
            CommandMapperError::InvalidParams(msg) => JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: format!("无效参数: {}", msg),
                data: None,
            },
            CommandMapperError::JsonError(err) => JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: format!("JSON 解析错误: {}", err),
                data: None,
            },
        }
    }

    /// 将 CommandError 映射为 JSON-RPC 错误
    fn map_command_error(error: CommandError) -> JsonRpcError {

        match error {
            CommandError::UnsupportedCommand(msg) => JsonRpcError {
                code: error_codes::METHOD_NOT_FOUND,
                message: format!("不支持的命令: {}", msg),
                data: None,
            },
            CommandError::InvalidParams(msg) => JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: format!("无效参数: {}", msg),
                data: None,
            },
            CommandError::NotFound(msg) => JsonRpcError {
                code: error_codes::SERVER_ERROR,
                message: format!("资源未找到: {}", msg),
                data: None,
            },
            CommandError::ValidationError(msg) => JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: format!("验证失败: {}", msg),
                data: None,
            },
            CommandError::InternalError(msg) => JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("内部错误: {}", msg),
                data: None,
            },
            CommandError::NetworkError(msg) => JsonRpcError {
                code: error_codes::SERVER_ERROR,
                message: format!("网络错误: {}", msg),
                data: None,
            },
            CommandError::DatabaseError(msg) => JsonRpcError {
                code: error_codes::SERVER_ERROR,
                message: format!("数据库错误: {}", msg),
                data: None,
            },
            CommandError::Timeout(msg) => JsonRpcError {
                code: error_codes::SERVER_ERROR,
                message: format!("超时: {}", msg),
                data: None,
            },
        }
    }

    /// 将 ResultMapperError 映射为 JSON-RPC 错误
    fn map_result_error(error: ResultMapperError) -> JsonRpcError {
        match error {
            ResultMapperError::SerializationError(err) => JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("序列化错误: {}", err),
                data: None,
            },
            ResultMapperError::TypeMismatch(msg) => JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("类型不匹配: {}", msg),
                data: None,
            },
        }
    }
}


// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::mock_repository::MockEthereumRepository;
    use crate::inbound::json_types::RequestId;
    use crate::service::ethereum_service_impl::EthereumServiceImpl;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_handle_simple_request() {
        let mock_repo = MockEthereumRepository::new();
        let service = Arc::new(EthereumServiceImpl::new(mock_repo));
        let dispatcher = CommandDispatcher::new(service);
        let rpc_handler = EthJsonRpcHandler::new(dispatcher);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "eth_blockNumber".to_string(),
            params: serde_json::json!([]),
            id: RequestId::Number(1),
        };

        let response = rpc_handler.handle(request).await;
        assert!(matches!(response, JsonRpcResponse::Success { .. }));
    }

    #[test]
    fn test_request_id_serialization() {
        let id_num = RequestId::Number(1);
        let json = serde_json::to_string(&id_num).unwrap();
        assert_eq!(json, "1");

        let id_str = RequestId::String("test".to_string());
        let json = serde_json::to_string(&id_str).unwrap();
        assert_eq!(json, "\"test\"");
    }
}
