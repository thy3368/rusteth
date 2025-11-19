/// 区块持久化接口 - 参考 geth/core/blockchain.go
///
/// 设计参考：
/// - geth/core/blockchain.go: BlockChain 主要接口
/// - geth/core/rawdb: 底层数据库操作
///
/// Clean Architecture 分层：
/// - BlockRepository trait: 领域层接口（底层持久化）
/// - BlockChain trait: 用例层接口（链状态管理）
/// - 具体实现: 基础设施层

use crate::domain::block_types::{Block, BlockValidationError};
use crate::domain::receipt_types::TransactionReceipt;
use async_trait::async_trait;
use ethereum_types::{H256, U256, U64};
use std::sync::Arc;

/// 区块持久化错误
#[derive(Debug, Clone, PartialEq)]
pub enum BlockRepositoryError {
    /// 区块未找到
    BlockNotFound { hash: H256 },
    /// 区块号未找到
    BlockNumberNotFound { number: U64 },
    /// 数据库错误
    DatabaseError(String),
    /// 序列化错误
    SerializationError(String),
    /// 区块已存在
    BlockAlreadyExists { hash: H256 },
}

impl std::fmt::Display for BlockRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BlockNotFound { hash } => write!(f, "Block not found: {}", hash),
            Self::BlockNumberNotFound { number } => write!(f, "Block number not found: {}", number),
            Self::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::BlockAlreadyExists { hash } => write!(f, "Block already exists: {}", hash),
        }
    }
}

impl std::error::Error for BlockRepositoryError {}

/// 区块持久化接口（底层存储）
///
/// 职责：
/// - 区块的CRUD操作
/// - 收据的存储和查询
/// - 区块号到哈希的映射
///
/// 参考 geth/core/rawdb 的设计
#[async_trait]
pub trait BlockRepository: Send + Sync {
    /// 保存区块及其收据
    ///
    /// 参考: geth rawdb.WriteBlock + WriteReceipts
    ///
    /// 原子操作：
    /// 1. 写入区块头
    /// 2. 写入区块体（交易列表）
    /// 3. 写入收据
    /// 4. 写入区块号->哈希映射
    async fn save_block(
        &self,
        block: &Block,
        receipts: &[TransactionReceipt],
        total_difficulty: U256,
    ) -> Result<(), BlockRepositoryError>;

    /// 根据哈希获取区块
    ///
    /// 参考: geth rawdb.ReadBlock
    async fn get_block_by_hash(&self, hash: &H256) -> Result<Option<Block>, BlockRepositoryError>;

    /// 根据区块号获取区块
    ///
    /// 参考: geth rawdb.ReadBlockByNumber
    async fn get_block_by_number(&self, number: U64) -> Result<Option<Block>, BlockRepositoryError>;

    /// 根据哈希获取收据
    ///
    /// 参考: geth rawdb.ReadReceipts
    async fn get_receipts_by_hash(
        &self,
        hash: &H256,
    ) -> Result<Vec<TransactionReceipt>, BlockRepositoryError>;

    /// 获取区块的总难度
    ///
    /// 参考: geth rawdb.ReadTd
    async fn get_total_difficulty(&self, hash: &H256) -> Result<Option<U256>, BlockRepositoryError>;

    /// 根据区块号获取区块哈希
    ///
    /// 参考: geth rawdb.ReadCanonicalHash
    async fn get_canonical_hash(&self, number: U64) -> Result<Option<H256>, BlockRepositoryError>;

    /// 设置规范链的区块号->哈希映射
    ///
    /// 参考: geth rawdb.WriteCanonicalHash
    async fn set_canonical_hash(&self, number: U64, hash: H256) -> Result<(), BlockRepositoryError>;

    /// 删除规范链的区块号映射（用于链重组）
    ///
    /// 参考: geth rawdb.DeleteCanonicalHash
    async fn delete_canonical_hash(&self, number: U64) -> Result<(), BlockRepositoryError>;
}


/// 内存版区块存储（用于测试和单机版）
///
/// 使用 Arc + Mutex 实现线程安全
pub struct InMemoryBlockRepository {
    // TODO: 实现内存版本
    // - blocks: HashMap<H256, Block>
    // - receipts: HashMap<H256, Vec<TransactionReceipt>>
    // - block_numbers: HashMap<U64, H256>
    // - total_difficulties: HashMap<H256, U256>
    // - current_head: AtomicPtr<H256>
}

impl InMemoryBlockRepository {
    pub fn new() -> Self {
        Self {}
    }

    pub fn with_genesis(_genesis: Block) -> Self {
        // TODO: 初始化创世区块
        Self {}
    }
}

#[async_trait]
impl BlockRepository for InMemoryBlockRepository {
    async fn save_block(
        &self,
        _block: &Block,
        _receipts: &[TransactionReceipt],
        _total_difficulty: U256,
    ) -> Result<(), BlockRepositoryError> {
        todo!("实现内存版本的区块保存")
    }

    async fn get_block_by_hash(&self, _hash: &H256) -> Result<Option<Block>, BlockRepositoryError> {
        todo!("实现内存版本的区块查询")
    }

    async fn get_block_by_number(&self, _number: U64) -> Result<Option<Block>, BlockRepositoryError> {
        todo!("实现内存版本的区块查询")
    }

    async fn get_receipts_by_hash(
        &self,
        _hash: &H256,
    ) -> Result<Vec<TransactionReceipt>, BlockRepositoryError> {
        todo!("实现内存版本的收据查询")
    }

    async fn get_total_difficulty(&self, _hash: &H256) -> Result<Option<U256>, BlockRepositoryError> {
        todo!("实现内存版本的难度查询")
    }

    async fn get_canonical_hash(&self, _number: U64) -> Result<Option<H256>, BlockRepositoryError> {
        todo!("实现内存版本的规范哈希查询")
    }

    async fn set_canonical_hash(&self, _number: U64, _hash: H256) -> Result<(), BlockRepositoryError> {
        todo!("实现内存版本的规范哈希设置")
    }

    async fn delete_canonical_hash(&self, _number: U64) -> Result<(), BlockRepositoryError> {
        todo!("实现内存版本的规范哈希删除")
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::blockchain_impl::BlockChainImpl;

    #[test]
    fn test_block_repository_error_display() {
        let err = BlockRepositoryError::BlockNotFound {
            hash: H256::zero(),
        };
        assert!(err.to_string().contains("Block not found"));
    }

    #[tokio::test]
    async fn test_in_memory_repository_creation() {
        let _repo = InMemoryBlockRepository::new();
        // 基本创建测试
    }

    #[tokio::test]
    async fn test_blockchain_impl_creation() {
        let repo = Arc::new(InMemoryBlockRepository::new());
        let _blockchain = BlockChainImpl::new(repo);
        // 基本创建测试
    }
}
