/// 区块构建器接口 - 遵循Clean Architecture原则
///
/// 设计依据：
/// - EIP-1559: 费用市场和base fee动态调整
/// - EIP-3675: PoS共识机制
/// - EIP-4399: PREVRANDAO随机数
/// - Geth miner/worker.go: 区块构建流程
///
/// 参考文档：
/// - /consensus-specs/specs/gloas/builder.md
/// - /EIPs/EIPS/eip-1559.md
/// - /EIPs/EIPS/eip-3675.md

use crate::domain::block_types::{Block, BlockValidationError, BuildEnvironment};
use async_trait::async_trait;
use ethereum_types::U64;
use crate::domain::receipt_types::TransactionReceipt;
use crate::service::repo::block_repo::BlockRepositoryError;

/// 区块构建器接口
///
/// 职责：
/// 1. 从交易池选择交易 (按价格排序，装箱问题)
/// 2. 计算新区块的base fee (EIP-1559动态调整)
/// 3. 计算新区块的gas limit (动态调整，最大变化1/1024)
/// 4. 执行交易并更新状态
/// 5. 计算状态根、交易根、收据根
/// 6. 构建完整区块
///
/// 流程参考 geth/miner/worker.go:commitTransactions
#[async_trait]
pub trait BlockBuilder: Send + Sync {
    /// 构建新区块
    ///
    /// 步骤:
    /// 1. 计算base fee和gas limit
    /// 2. 从交易池获取候选交易
    /// 3. 选择并执行交易 (贪心算法 + 装箱)
    /// 4. 计算Merkle根和状态根
    /// 5. 组装区块头和区块
    async fn build_block(&self, env: BuildEnvironment) -> Result<Block, BlockValidationError>;

    /// 验证区块 (PoS规则)
    ///
    /// 参考 EIP-3675 和 geth/consensus/beacon/consensus.go
    async fn validate_block(&self, block: &Block) -> Result<(), BlockValidationError>;
}

/// 区块链状态管理接口（高层操作）
///
/// 职责：
/// - 管理链的当前状态（head block）
/// - 插入区块并验证
/// - 处理链重组
/// - 维护规范链
///
/// 参考 geth/core/blockchain.go 的 BlockChain 结构体
#[async_trait]
pub trait BlockChain: Send + Sync {
    /// 获取当前链头区块
    ///
    /// 参考: geth BlockChain.CurrentBlock()
    async fn current_block(&self) -> Result<Block, BlockRepositoryError>;

    /// 获取当前链头区块号
    async fn current_block_number(&self) -> Result<U64, BlockRepositoryError>;

    /// 获取创世区块
    ///
    /// 参考: geth BlockChain.Genesis()
    async fn genesis(&self) -> Result<Block, BlockRepositoryError>;

    /// 插入新区块到链中（带验证）
    ///
    /// 参考: geth BlockChain.InsertBlockWithoutSetHead
    ///
    /// 流程:
    /// 1. 验证区块基本规则（PoS规则、gas limit等）
    /// 2. 验证父区块存在
    /// 3. 执行区块中的交易
    /// 4. 验证状态根
    /// 5. 持久化区块和收据
    ///
    /// 注意: 此方法不更新链头，需要手动调用 set_head
    async fn insert_block(
        &self,
        block: Block,
        receipts: Vec<TransactionReceipt>,
    ) -> Result<(), BlockValidationError>;

    /// 写入区块并设置为链头
    ///
    /// 参考: geth BlockChain.WriteBlockAndSetHead
    ///
    /// 这是原子操作：
    /// 1. 验证并插入区块
    /// 2. 更新规范链指针
    /// 3. 更新内存中的链头
    /// 4. 发布 ChainHeadEvent
    async fn write_block_and_set_head(
        &self,
        block: Block,
        receipts: Vec<TransactionReceipt>,
    ) -> Result<(), BlockValidationError>;

    /// 设置链头到指定区块
    ///
    /// 参考: geth BlockChain.SetHead
    ///
    /// 用于：
    /// - 链重组
    /// - 快照同步完成
    /// - 回滚到安全点
    async fn set_head(&self, number: U64) -> Result<(), BlockRepositoryError>;

    /// 重置链到创世区块
    ///
    /// 参考: geth BlockChain.Reset()
    ///
    /// 警告: 删除所有区块数据
    async fn reset(&self) -> Result<(), BlockRepositoryError>;

    /// 获取从指定高度开始的区块链
    ///
    /// 用于同步和查询
    async fn get_blocks_from(
        &self,
        start: U64,
        count: usize,
    ) -> Result<Vec<Block>, BlockRepositoryError>;
}
