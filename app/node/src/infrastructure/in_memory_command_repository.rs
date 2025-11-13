//! 内存命令仓储实现
//!
//! 用于开发和测试环境的简单内存存储实现

use crate::domain::command_repository::{
    CommandRecord, CommandRepository, CommandRepositoryError, CommandStatus,
};
use crate::domain::commands::EthCommand;
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 内存命令仓储
#[derive(Clone)]
pub struct InMemoryCommandRepository {
    /// 命令记录存储（record_id -> CommandRecord）
    records: Arc<RwLock<HashMap<String, CommandRecord>>>,
}

impl InMemoryCommandRepository {
    /// 创建新的内存仓储实例
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryCommandRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommandRepository for InMemoryCommandRepository {
    async fn save(&self, command: EthCommand) -> Result<String, CommandRepositoryError> {
        // 生成唯一记录ID
        let record_id = Uuid::new_v4().to_string();

        // 创建命令记录
        let record = CommandRecord {
            id: record_id.clone(),
            command,
            timestamp: Utc::now(),
            request_id: None, // 可以在后续版本中从上下文获取
            status: CommandStatus::Pending,
        };

        // 存储记录
        let mut records = self.records.write().await;
        records.insert(record_id.clone(), record);

        Ok(record_id)
    }

    async fn update_status(
        &self,
        record_id: &str,
        status: CommandStatus,
    ) -> Result<(), CommandRepositoryError> {
        let mut records = self.records.write().await;

        match records.get_mut(record_id) {
            Some(record) => {
                record.status = status;
                Ok(())
            }
            None => Err(CommandRepositoryError::QueryError(format!(
                "记录未找到: {}",
                record_id
            ))),
        }
    }

    async fn find_by_id(
        &self,
        record_id: &str,
    ) -> Result<Option<CommandRecord>, CommandRepositoryError> {
        let records = self.records.read().await;
        Ok(records.get(record_id).cloned())
    }

    async fn find_by_time_range(
        &self,
        start: chrono::DateTime<Utc>,
        end: chrono::DateTime<Utc>,
    ) -> Result<Vec<CommandRecord>, CommandRepositoryError> {
        let records = self.records.read().await;
        let mut result: Vec<CommandRecord> = records
            .values()
            .filter(|record| record.timestamp >= start && record.timestamp <= end)
            .cloned()
            .collect();

        // 按时间戳排序
        result.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(result)
    }

    async fn find_recent(
        &self,
        limit: usize,
    ) -> Result<Vec<CommandRecord>, CommandRepositoryError> {
        let records = self.records.read().await;
        let mut result: Vec<CommandRecord> = records.values().cloned().collect();

        // 按时间戳倒序排序
        result.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // 限制返回数量
        result.truncate(limit);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::commands::EthCommand;
    use ethereum_types::U64;

    #[tokio::test]
    async fn test_save_command() {
        let repo = InMemoryCommandRepository::new();
        let command = EthCommand::GetBlockNumber;

        let record_id = repo.save(command).await.unwrap();
        assert!(!record_id.is_empty());

        // 验证可以查询到记录
        let record = repo.find_by_id(&record_id).await.unwrap();
        assert!(record.is_some());

        let record = record.unwrap();
        assert_eq!(record.id, record_id);
        assert!(matches!(record.command, EthCommand::GetBlockNumber));
        assert_eq!(record.status, CommandStatus::Pending);
    }

    #[tokio::test]
    async fn test_update_status() {
        let repo = InMemoryCommandRepository::new();
        let command = EthCommand::GetChainId;

        let record_id = repo.save(command).await.unwrap();

        // 更新状态为成功
        repo.update_status(&record_id, CommandStatus::Success)
            .await
            .unwrap();

        let record = repo.find_by_id(&record_id).await.unwrap().unwrap();
        assert_eq!(record.status, CommandStatus::Success);

        // 更新状态为失败
        repo.update_status(&record_id, CommandStatus::Failed("测试错误".to_string()))
            .await
            .unwrap();

        let record = repo.find_by_id(&record_id).await.unwrap().unwrap();
        assert!(matches!(record.status, CommandStatus::Failed(_)));
    }

    #[tokio::test]
    async fn test_find_recent() {
        let repo = InMemoryCommandRepository::new();

        // 保存多个命令
        for _ in 0..5 {
            repo.save(EthCommand::GetBlockNumber).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // 查询最近3条
        let recent = repo.find_recent(3).await.unwrap();
        assert_eq!(recent.len(), 3);

        // 验证按时间倒序排列
        for i in 0..recent.len() - 1 {
            assert!(recent[i].timestamp >= recent[i + 1].timestamp);
        }
    }

    #[tokio::test]
    async fn test_find_by_time_range() {
        let repo = InMemoryCommandRepository::new();

        let start = Utc::now();

        // 保存命令
        repo.save(EthCommand::GetBlockNumber).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let middle = Utc::now();

        repo.save(EthCommand::GetChainId).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let end = Utc::now();

        // 查询整个时间范围
        let all_records = repo.find_by_time_range(start, end).await.unwrap();
        assert_eq!(all_records.len(), 2);

        // 查询部分时间范围
        let partial_records = repo.find_by_time_range(middle, end).await.unwrap();
        assert_eq!(partial_records.len(), 1);
    }
}
