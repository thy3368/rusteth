//! 单命令处理器 - 插件化架构
//!
//! 定义单个命令的处理器接口，支持动态注册和查找
//! 遵循以下设计模式：
//! - Strategy Pattern: 每个命令有独立的处理策略
//! - Plugin Architecture: 支持运行时注册新的处理器
//! - Registry Pattern: 通过名称查找处理器

use crate::domain::commands::{CommandResult, EthCommand};
use crate::domain::command_handler::CommandError;
use async_trait::async_trait;
use std::sync::Arc;

/// 单命令处理器 Trait
///
/// 每个具体的命令类型都有一个独立的处理器实现
///
/// # 设计原则
/// - **单一职责**: 每个处理器只处理一种命令
/// - **可插拔**: 支持动态注册和替换
/// - **无状态**: 处理器本身不持有业务状态
///
/// # 使用示例
/// ```ignore
/// struct GetBlockNumberHandler { /* ... */ }
///
/// impl SingleCommandHandler for GetBlockNumberHandler {
///     fn command_name(&self) -> &'static str {
///         "eth_blockNumber"
///     }
///
///     async fn handle(&self, command: EthCommand) -> Result<CommandResult, CommandError> {
///         // 处理 GetBlockNumber 命令
///     }
/// }
/// ```
#[async_trait]
pub trait SingleCommandHandler: Send + Sync {
    /// 返回此处理器对应的命令名称
    ///
    /// 必须与 EthCommand::name() 返回的值一致
    fn command_name(&self) -> &'static str;

    /// 处理命令
    ///
    /// # 参数
    /// - `command`: 要处理的命令对象
    ///
    /// # 返回
    /// - `Ok(CommandResult)`: 处理成功
    /// - `Err(CommandError)`: 处理失败
    ///
    /// # 注意
    /// 实现时应该验证 command 的类型是否匹配此处理器
    async fn handle(&self, command: EthCommand) -> Result<CommandResult, CommandError>;

    /// 验证命令是否可以由此处理器处理（可选实现）
    ///
    /// 默认实现通过命令名称匹配
    fn can_handle(&self, command: &EthCommand) -> bool {
        command.name() == self.command_name()
    }
}

/// Handler Repository Trait - 处理器仓储
///
/// 管理所有注册的命令处理器，支持动态查找
///
/// # 设计原则
/// - **注册中心**: 维护命令名称到处理器的映射
/// - **动态查找**: 运行时根据命令名称查找处理器
/// - **线程安全**: 支持并发访问
pub trait HandlerRepository: Send + Sync {
    /// 根据命令名称查找处理器
    ///
    /// # 参数
    /// - `command_name`: 命令名称（如 "eth_blockNumber"）
    ///
    /// # 返回
    /// - `Some(handler)`: 找到对应的处理器
    /// - `None`: 未找到处理器
    fn query(&self, command_name: &str) -> Option<Arc<dyn SingleCommandHandler>>;

    /// 注册处理器
    ///
    /// # 参数
    /// - `handler`: 要注册的处理器
    ///
    /// # 注意
    /// 如果已存在相同命令名称的处理器，应该覆盖原有的
    fn register(&mut self, handler: Arc<dyn SingleCommandHandler>);

    /// 取消注册处理器
    ///
    /// # 参数
    /// - `command_name`: 要取消注册的命令名称
    ///
    /// # 返回
    /// - `true`: 成功取消注册
    /// - `false`: 未找到对应的处理器
    fn unregister(&mut self, command_name: &str) -> bool;

    /// 获取所有已注册的命令名称
    fn registered_commands(&self) -> Vec<String>;

    /// 获取已注册处理器的数量
    fn count(&self) -> usize {
        self.registered_commands().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock 处理器用于测试
    struct MockHandler {
        name: &'static str,
    }

    #[async_trait]
    impl SingleCommandHandler for MockHandler {
        fn command_name(&self) -> &'static str {
            self.name
        }

        async fn handle(&self, _command: EthCommand) -> Result<CommandResult, CommandError> {
            Ok(CommandResult::Unit)
        }
    }

    #[test]
    fn test_can_handle() {
        let handler = MockHandler {
            name: "eth_blockNumber",
        };
        let command = EthCommand::GetBlockNumber;

        assert!(handler.can_handle(&command));
    }
}
