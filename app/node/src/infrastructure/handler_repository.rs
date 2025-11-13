//! Handler Repository 实现
//!
//! 提供基于内存的处理器注册中心实现

use crate::domain::single_command_handler::{HandlerRepository, SingleCommandHandler};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 内存版 Handler Repository
///
/// 使用 HashMap 存储命令名称到处理器的映射
/// 使用 RwLock 保证线程安全
#[derive(Clone)]
pub struct InMemoryHandlerRepository {
    handlers: Arc<RwLock<HashMap<String, Arc<dyn SingleCommandHandler>>>>,
}

impl InMemoryHandlerRepository {
    /// 创建新的空仓储
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 批量注册处理器
    ///
    /// # 参数
    /// - `handlers`: 处理器列表
    pub fn register_batch(&mut self, handlers: Vec<Arc<dyn SingleCommandHandler>>) {
        let mut map = self.handlers.write().unwrap();
        for handler in handlers {
            map.insert(handler.command_name().to_string(), handler);
        }
    }
}

impl Default for InMemoryHandlerRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl HandlerRepository for InMemoryHandlerRepository {
    fn query(&self, command_name: &str) -> Option<Arc<dyn SingleCommandHandler>> {
        self.handlers
            .read()
            .unwrap()
            .get(command_name)
            .cloned()
    }

    fn register(&mut self, handler: Arc<dyn SingleCommandHandler>) {
        let command_name = handler.command_name().to_string();
        self.handlers.write().unwrap().insert(command_name, handler);
    }

    fn unregister(&mut self, command_name: &str) -> bool {
        self.handlers
            .write()
            .unwrap()
            .remove(command_name)
            .is_some()
    }

    fn registered_commands(&self) -> Vec<String> {
        self.handlers
            .read()
            .unwrap()
            .keys()
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::command_handler::CommandError;
    use crate::domain::commands::{CommandResult, EthCommand};
    use async_trait::async_trait;

    struct TestHandler {
        name: &'static str,
    }

    #[async_trait]
    impl SingleCommandHandler for TestHandler {
        fn command_name(&self) -> &'static str {
            self.name
        }

        async fn handle(&self, _command: EthCommand) -> Result<CommandResult, CommandError> {
            Ok(CommandResult::Unit)
        }
    }

    #[test]
    fn test_register_and_query() {
        let mut repo = InMemoryHandlerRepository::new();
        let handler = Arc::new(TestHandler {
            name: "eth_blockNumber",
        });

        repo.register(handler.clone());

        let found = repo.query("eth_blockNumber");
        assert!(found.is_some());
        assert_eq!(found.unwrap().command_name(), "eth_blockNumber");
    }

    #[test]
    fn test_unregister() {
        let mut repo = InMemoryHandlerRepository::new();
        let handler = Arc::new(TestHandler {
            name: "eth_blockNumber",
        });

        repo.register(handler);
        assert_eq!(repo.count(), 1);

        let removed = repo.unregister("eth_blockNumber");
        assert!(removed);
        assert_eq!(repo.count(), 0);
    }

    #[test]
    fn test_registered_commands() {
        let mut repo = InMemoryHandlerRepository::new();

        repo.register(Arc::new(TestHandler {
            name: "eth_blockNumber",
        }));
        repo.register(Arc::new(TestHandler {
            name: "eth_getBalance",
        }));

        let commands = repo.registered_commands();
        assert_eq!(commands.len(), 2);
        assert!(commands.contains(&"eth_blockNumber".to_string()));
        assert!(commands.contains(&"eth_getBalance".to_string()));
    }

    #[test]
    fn test_register_batch() {
        let mut repo = InMemoryHandlerRepository::new();

        let handlers: Vec<Arc<dyn SingleCommandHandler>> = vec![
            Arc::new(TestHandler {
                name: "eth_blockNumber",
            }),
            Arc::new(TestHandler {
                name: "eth_getBalance",
            }),
        ];

        repo.register_batch(handlers);
        assert_eq!(repo.count(), 2);
    }
}
