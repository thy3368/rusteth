//! 命令仓储 - 用于命令持久化、审计和溯源
//!
//! 实现 Event Sourcing 模式，记录所有执行的命令以支持：
//! - 审计追踪
//! - 命令重放
//! - 故障恢复
//! - 性能分析

use crate::domain::command_types::EthCommand;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::fmt;

/// 命令记录 - 包含命令和元数据
#[derive(Debug, Clone)]
pub struct CommandRecord {
    /// 记录ID
    pub id: String,
    /// 执行的命令
    pub command: EthCommand,
    /// 执行时间戳
    pub timestamp: DateTime<Utc>,
    /// 请求ID（来自JSON-RPC）
    pub request_id: Option<String>,
    /// 执行结果状态
    pub status: CommandStatus,
}

/// 命令执行状态
#[derive(Debug, Clone, PartialEq)]
pub enum CommandStatus {
    /// 待执行
    Pending,
    /// 执行成功
    Success,
    /// 执行失败
    Failed(String),
}

/// 命令仓储错误类型
#[derive(Debug)]
pub enum CommandRepositoryError {
    /// 存储错误
    StorageError(String),
    /// 查询错误
    QueryError(String),
    /// 序列化错误
    SerializationError(String),
}

impl fmt::Display for CommandRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StorageError(msg) => write!(f, "存储错误: {}", msg),
            Self::QueryError(msg) => write!(f, "查询错误: {}", msg),
            Self::SerializationError(msg) => write!(f, "序列化错误: {}", msg),
        }
    }
}

impl std::error::Error for CommandRepositoryError {}

/// 命令仓储接口
///
/// # 职责
/// - 持久化所有执行的命令
/// - 支持命令查询和审计
/// - 提供命令重放能力
#[async_trait]
pub trait CommandRepository: Send + Sync {
    /// 保存命令（接受原始 EthCommand）
    ///
    /// # 参数
    /// - `command`: 要保存的以太坊命令
    ///
    /// # 返回
    /// - 成功时返回生成的记录ID
    async fn save(&self, command: EthCommand) -> Result<String, CommandRepositoryError>;

    /// 更新命令执行状态
    ///
    /// # 参数
    /// - `record_id`: 记录ID
    /// - `status`: 新的执行状态
    async fn update_status(
        &self,
        record_id: &str,
        status: CommandStatus,
    ) -> Result<(), CommandRepositoryError>;

    /// 根据ID查询命令记录
    async fn find_by_id(
        &self,
        record_id: &str,
    ) -> Result<Option<CommandRecord>, CommandRepositoryError>;

    /// 查询指定时间范围内的命令记录
    async fn find_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CommandRecord>, CommandRepositoryError>;

    /// 获取最近N条命令记录
    async fn find_recent(
        &self,
        limit: usize,
    ) -> Result<Vec<CommandRecord>, CommandRepositoryError>;
}
