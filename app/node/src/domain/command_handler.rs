//! 领域层 - CommandHandler trait 定义
//!
//! 遵循 CQRS 模式和 Erlang 风格的消息传递机制：
//! - Handler 接收 Command，返回 CommandResult
//! - 支持异步处理（async/await）
//! - 无状态接口，状态由具体实现管理
//! - 通过静态分发实现零成本抽象

use crate::domain::commands::{CommandResult, EthCommand};
use async_trait::async_trait;
use thiserror::Error;

/// 命令处理错误
///
/// 封装命令执行过程中可能发生的所有错误
#[derive(Debug, Error, Clone)]
pub enum CommandError {
    /// 命令不支持
    #[error("不支持的命令: {0}")]
    UnsupportedCommand(String),

    /// 参数无效
    #[error("无效的参数: {0}")]
    InvalidParams(String),

    /// 资源未找到
    #[error("资源未找到: {0}")]
    NotFound(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    InternalError(String),

    /// 网络错误
    #[error("网络错误: {0}")]
    NetworkError(String),

    /// 数据库错误
    #[error("数据库错误: {0}")]
    DatabaseError(String),

    /// 验证失败
    #[error("验证失败: {0}")]
    ValidationError(String),

    /// 超时
    #[error("操作超时: {0}")]
    Timeout(String),
}

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

/// 命令处理器 Trait
///
/// 定义命令处理的核心接口，遵循以下原则：
/// - **无状态接口**: trait 本身不持有可变状态
/// - **消息传递**: 类似 Erlang 的 Actor 模型，通过消息（Command）通信
/// - **单一职责**: 只负责命令的路由和执行
/// - **可测试性**: 易于 mock 和单元测试
///
/// # 设计模式
/// - Command Pattern: 将请求封装为对象
/// - Strategy Pattern: 不同的实现策略可以互换
/// - Actor Pattern: 消息传递，状态隔离
///
/// # 使用示例
/// ```ignore
/// let handler = MyCommandHandler::new(dependencies);
/// let command = EthCommand::GetBlockNumber;
/// let result = handler.ask(command).await?;
/// ```
#[async_trait]
pub trait CommandHandler: Send + Sync {
    /// 处理命令（Erlang 风格的 ask）
    ///
    /// # 参数
    /// - `command`: 要执行的命令对象
    ///
    /// # 返回
    /// - `Ok(CommandResult)`: 命令执行成功，返回结果
    /// - `Err(CommandError)`: 命令执行失败，返回错误信息
    ///
    /// # 语义
    /// - 类似 Erlang 的 `gen_server:call/2`
    /// - 同步等待结果返回（虽然是异步实现）
    /// - 保证请求-响应的对应关系
    async fn ask(&self, command: EthCommand) -> Result<CommandResult, CommandError>;

    /// 批量处理命令（可选优化）
    ///
    /// 默认实现：串行执行所有命令
    /// 具体实现可以重写以支持并行或批量优化
    async fn ask_batch(
        &self,
        commands: Vec<EthCommand>,
    ) -> Vec<Result<CommandResult, CommandError>> {
        let mut results = Vec::with_capacity(commands.len());
        for command in commands {
            results.push(self.ask(command).await);
        }
        results
    }

    /// 健康检查（可选）
    ///
    /// 用于检查 Handler 是否正常运行
    /// 默认实现总是返回健康
    async fn health_check(&self) -> Result<(), CommandError> {
        Ok(())
    }
}

/// 空操作命令处理器（用于测试）
///
/// 实现一个什么都不做的 Handler，用于测试和占位
#[derive(Debug, Clone)]
pub struct NoOpCommandHandler;

#[async_trait]
impl CommandHandler for NoOpCommandHandler {
    async fn ask(&self, _command: EthCommand) -> Result<CommandResult, CommandError> {
        Err(CommandError::UnsupportedCommand(
            "NoOpCommandHandler 不支持任何命令".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_handler() {
        let handler = NoOpCommandHandler;
        let result = handler.ask(EthCommand::GetBlockNumber).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CommandError::UnsupportedCommand(_)
        ));
    }

    #[tokio::test]
    async fn test_health_check() {
        let handler = NoOpCommandHandler;
        let result = handler.health_check().await;
        assert!(result.is_ok());
    }
}
