use std::sync::Arc;
use async_trait::async_trait;
use ethereum_types::U64;
use crate::domain::block_types::{Block, BlockValidationError};
use crate::domain::receipt_types::TransactionReceipt;
use crate::service::build_block_trait::BlockChain;
use crate::service::repo::block_repo::{BlockRepository, BlockRepositoryError};

/// 区块链实现（管理链状态）
///
/// 参考 geth/core/blockchain.go
pub struct BlockChainImpl {
    /// 底层区块存储
    repository: Arc<dyn BlockRepository>,
    // TODO: 添加区块验证器（来自 BuildBlockService）
    // validator: Arc<dyn BlockBuilder>,
}

impl BlockChainImpl {
    pub fn new(repository: Arc<dyn BlockRepository>) -> Self {
        Self { repository }
    }

    /// 创建带创世区块的链
    pub async fn new_with_genesis(
        repository: Arc<dyn BlockRepository>,
        _genesis: Block,
    ) -> Result<Self, BlockRepositoryError> {
        // TODO: 初始化创世区块
        Ok(Self { repository })
    }
}

#[async_trait]
impl BlockChain for BlockChainImpl {
    async fn current_block(&self) -> Result<Block, BlockRepositoryError> {
        // TODO: 从缓存或数据库获取当前区块
        todo!("实现获取当前区块")
    }

    async fn current_block_number(&self) -> Result<U64, BlockRepositoryError> {
        todo!("实现获取当前区块号")
    }

    async fn genesis(&self) -> Result<Block, BlockRepositoryError> {
        // 创世区块总是 #0
        let block: Option<Block> = self.repository
            .get_block_by_number(U64::zero())
            .await?;
        block.ok_or_else(|| BlockRepositoryError::BlockNumberNotFound { number: U64::zero() })
    }

    async fn insert_block(
        &self,
        _block: Block,
        _receipts: Vec<TransactionReceipt>,
    ) -> Result<(), BlockValidationError> {
        // TODO: 实现区块插入逻辑
        // 1. 验证区块
        // 2. 检查父区块
        // 3. 执行交易
        // 4. 持久化
        todo!("实现区块插入")
    }

    async fn write_block_and_set_head(
        &self,
        _block: Block,
        _receipts: Vec<TransactionReceipt>,
    ) -> Result<(), BlockValidationError> {
        // TODO: 实现写入并设置链头
        // 1. insert_block
        // 2. set_canonical_hash
        // 3. 更新内存缓存
        todo!("实现写入并设置链头")
    }

    async fn set_head(&self, _number: U64) -> Result<(), BlockRepositoryError> {
        // TODO: 设置链头到指定高度
        todo!("实现设置链头")
    }

    async fn reset(&self) -> Result<(), BlockRepositoryError> {
        // TODO: 重置链到创世区块
        todo!("实现链重置")
    }

    async fn get_blocks_from(
        &self,
        _start: U64,
        _count: usize,
    ) -> Result<Vec<Block>, BlockRepositoryError> {
        // TODO: 批量获取区块
        todo!("实现批量获取区块")
    }
}
