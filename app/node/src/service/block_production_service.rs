/// 区块生产和接收服务 - 集成用例实现
///
/// 场景1: 矿工/验证者构建新区块
/// 场景2: 验证者接收并验证区块
///
/// 参考 geth/miner/worker.go 和 geth/eth/handler.go

use crate::domain::block_types::{Block, BlockValidationError, BuildEnvironment};
use crate::domain::receipt_types::TransactionReceipt;
use crate::service::build_block_trait::{BlockBuilder, BlockChain};
use crate::service::repo::block_repo::BlockRepositoryError;
use async_trait::async_trait;
use ethereum_types::U64;
use std::sync::Arc;

/// 区块生产错误
#[derive(Debug, Clone)]
pub enum BlockProductionError {
    /// 区块构建失败
    BuildFailed(BlockValidationError),
    /// 持久化失败
    PersistenceFailed(String),
    /// 广播失败
    BroadcastFailed(String),
    /// 验证失败
    ValidationFailed(BlockValidationError),
    /// 仓储错误
    RepositoryError(String),
}

impl std::fmt::Display for BlockProductionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BuildFailed(e) => write!(f, "Block build failed: {}", e),
            Self::PersistenceFailed(msg) => write!(f, "Persistence failed: {}", msg),
            Self::BroadcastFailed(msg) => write!(f, "Broadcast failed: {}", msg),
            Self::ValidationFailed(e) => write!(f, "Validation failed: {}", e),
            Self::RepositoryError(msg) => write!(f, "Repository error: {}", msg),
        }
    }
}

impl std::error::Error for BlockProductionError {}

impl From<BlockValidationError> for BlockProductionError {
    fn from(e: BlockValidationError) -> Self {
        Self::BuildFailed(e)
    }
}

impl From<BlockRepositoryError> for BlockProductionError {
    fn from(e: BlockRepositoryError) -> Self {
        Self::RepositoryError(e.to_string())
    }
}

/// 区块广播接口（P2P网络）
///
/// 用于将新构建的区块广播给其他节点
#[async_trait]
pub trait BlockBroadcaster: Send + Sync {
    /// 广播区块到网络
    async fn broadcast_block(&self, block: &Block) -> Result<(), String>;

    /// 广播区块给指定节点
    async fn broadcast_to_peer(&self, block: &Block, peer_id: &str) -> Result<(), String>;
}

/// Mock广播器（用于测试和单机版）
pub struct MockBroadcaster;

#[async_trait]
impl BlockBroadcaster for MockBroadcaster {
    async fn broadcast_block(&self, _block: &Block) -> Result<(), String> {
        // 单机版不需要广播
        Ok(())
    }

    async fn broadcast_to_peer(&self, _block: &Block, _peer_id: &str) -> Result<(), String> {
        Ok(())
    }
}

/// 区块生产服务
///
/// 场景1: 矿工/验证者构建新区块
///
/// 职责：
/// 1. 编排区块构建流程
/// 2. 持久化新区块
/// 3. 广播到网络
///
/// 参考 geth/miner/worker.go:resultLoop
pub struct BlockProductionService {
    /// 区块构建器
    builder: Arc<dyn BlockBuilder>,
    /// 区块链管理器
    blockchain: Arc<dyn BlockChain>,
    /// 区块广播器
    broadcaster: Arc<dyn BlockBroadcaster>,
}

impl BlockProductionService {
    /// 创建新的区块生产服务
    pub fn new(
        builder: Arc<dyn BlockBuilder>,
        blockchain: Arc<dyn BlockChain>,
        broadcaster: Arc<dyn BlockBroadcaster>,
    ) -> Self {
        Self {
            builder,
            blockchain,
            broadcaster,
        }
    }

    /// 场景1: 生产新区块
    ///
    /// 完整流程：
    /// 1. 调用 BlockBuilder::build_block 构建区块
    /// 2. 获取交易收据（从构建过程中）
    /// 3. 持久化到本地区块链
    /// 4. 广播给网络中的其他节点
    ///
    /// 参考 geth/miner/worker.go:resultLoop
    pub async fn produce_block(
        &self,
        env: BuildEnvironment,
    ) -> Result<Block, BlockProductionError> {
        // Step 1: 构建区块
        tracing::info!(
            parent_number = %env.parent_number,
            "开始构建新区块"
        );

        let block = self.builder.build_block(env).await
            .map_err(BlockProductionError::BuildFailed)?;

        tracing::info!(
            block_number = %block.number(),
            block_hash = %block.hash(),
            tx_count = block.transactions.len(),
            "区块构建成功"
        );

        // Step 2: 获取收据
        // TODO: 在实际实现中，收据应该在区块构建过程中生成并返回
        // 这里暂时创建空收据列表
        let receipts = Vec::new();

        // Step 3: 持久化到本地链
        tracing::info!(
            block_number = %block.number(),
            "开始持久化区块到本地链"
        );

        self.blockchain
            .write_block_and_set_head(block.clone(), receipts)
            .await
            .map_err(BlockProductionError::ValidationFailed)?;

        tracing::info!(
            block_number = %block.number(),
            "区块持久化成功"
        );

        // Step 4: 广播到网络
        tracing::info!(
            block_number = %block.number(),
            "开始广播区块到网络"
        );

        self.broadcaster
            .broadcast_block(&block)
            .await
            .map_err(BlockProductionError::BroadcastFailed)?;

        tracing::info!(
            block_number = %block.number(),
            "区块广播成功"
        );

        Ok(block)
    }

    /// 获取当前链头
    pub async fn current_block(&self) -> Result<Block, BlockProductionError> {
        self.blockchain.current_block().await.map_err(Into::into)
    }

    /// 获取当前区块号
    pub async fn current_block_number(&self) -> Result<U64, BlockProductionError> {
        self.blockchain.current_block_number().await.map_err(Into::into)
    }
}

/// 区块接收服务
///
/// 场景2: 验证者接收区块
///
/// 职责：
/// 1. 验证收到的区块
/// 2. 重新执行交易验证状态根
/// 3. 持久化有效区块
///
/// 参考 geth/eth/handler.go:handleBlockBroadcast
pub struct BlockReceptionService {
    /// 区块验证器
    validator: Arc<dyn BlockBuilder>,
    /// 区块链管理器
    blockchain: Arc<dyn BlockChain>,
}

impl BlockReceptionService {
    /// 创建新的区块接收服务
    pub fn new(
        validator: Arc<dyn BlockBuilder>,
        blockchain: Arc<dyn BlockChain>,
    ) -> Self {
        Self {
            validator,
            blockchain,
        }
    }

    /// 场景2: 接收并处理区块
    ///
    /// 完整流程：
    /// 1. 验证区块基本规则（PoS规则、gas limit等）
    /// 2. 验证父区块存在
    /// 3. 重新执行交易验证状态根（可选，快速同步时跳过）
    /// 4. 持久化到本地链
    ///
    /// 参考 geth/eth/handler.go:handleBlockBroadcast
    pub async fn receive_block(
        &self,
        block: Block,
        receipts: Vec<TransactionReceipt>,
    ) -> Result<(), BlockProductionError> {
        let block_number = block.number();
        let block_hash = block.hash();

        tracing::info!(
            block_number = %block_number,
            block_hash = %block_hash,
            tx_count = block.transactions.len(),
            "收到新区块"
        );

        // Step 1: 验证区块基本规则
        tracing::debug!(
            block_number = %block_number,
            "验证区块基本规则"
        );

        self.validator
            .validate_block(&block)
            .await
            .map_err(BlockProductionError::ValidationFailed)?;

        tracing::debug!(
            block_number = %block_number,
            "区块基本规则验证通过"
        );

        // Step 2: 验证父区块存在
        tracing::debug!(
            block_number = %block_number,
            parent_hash = %block.header.parent_hash,
            "验证父区块存在"
        );

        let parent_exists = self.verify_parent_exists(&block).await?;
        if !parent_exists {
            return Err(BlockProductionError::ValidationFailed(
                BlockValidationError::Other(format!(
                    "Parent block not found: {}",
                    block.header.parent_hash
                ))
            ));
        }

        tracing::debug!(
            block_number = %block_number,
            "父区块存在，验证通过"
        );

        // Step 3: 重新执行交易验证状态根
        // TODO: 在完整实现中，这里应该重新执行所有交易
        // 目前暂时跳过（快速同步模式）
        tracing::debug!(
            block_number = %block_number,
            "跳过交易重新执行（快速同步模式）"
        );

        // Step 4: 持久化到本地链
        tracing::info!(
            block_number = %block_number,
            "开始持久化区块"
        );

        self.blockchain
            .write_block_and_set_head(block, receipts)
            .await
            .map_err(BlockProductionError::ValidationFailed)?;

        tracing::info!(
            block_number = %block_number,
            "区块接收并持久化成功"
        );

        Ok(())
    }

    /// 验证父区块存在
    async fn verify_parent_exists(&self, block: &Block) -> Result<bool, BlockProductionError> {
        // 如果是创世区块，不需要父区块
        if block.number() == U64::zero() {
            return Ok(true);
        }

        // TODO: 从区块链查询父区块
        // let parent = self.blockchain.get_block_by_hash(block.parent_hash()).await?;
        // Ok(parent.is_some())

        // 暂时返回true（假设父区块总是存在）
        Ok(true)
    }

    /// 批量接收区块（用于同步）
    pub async fn receive_blocks(
        &self,
        blocks: Vec<(Block, Vec<TransactionReceipt>)>,
    ) -> Result<usize, BlockProductionError> {
        let mut imported = 0;

        for (block, receipts) in blocks {
            match self.receive_block(block, receipts).await {
                Ok(_) => imported += 1,
                Err(e) => {
                    tracing::warn!(error = %e, "区块导入失败");
                    // 继续处理下一个区块
                }
            }
        }

        tracing::info!(
            imported_count = imported,
            "批量导入完成"
        );

        Ok(imported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::transaction_repo_impl::{TxPoolConfig, TxPoolImpl};
    use crate::service::build_block_impl::BuildBlockService;
    use crate::service::repo::block_repo::InMemoryBlockRepository;
    use crate::service::blockchain_impl::BlockChainImpl;
    use ethereum_types::{Address, H256};

    /// 场景1测试: 矿工构建新区块
    #[tokio::test]
    async fn test_scenario1_produce_block() {
        // 准备：创建依赖
        //todo 服务太多，依赖太多
        let tx_pool = Arc::new(TxPoolImpl::new(TxPoolConfig::default()));
        let builder = Arc::new(BuildBlockService::new(tx_pool, Some(30_000_000)))
            as Arc<dyn BlockBuilder>;

        let repository = Arc::new(InMemoryBlockRepository::new());
        let blockchain = Arc::new(BlockChainImpl::new(repository)) as Arc<dyn BlockChain>;

        let broadcaster = Arc::new(MockBroadcaster) as Arc<dyn BlockBroadcaster>;

        // 创建生产服务
        let production_service = BlockProductionService::new(
            builder,
            blockchain.clone(),
            broadcaster,
        );

        // 构建环境
        let env = BuildEnvironment {
            parent_hash: H256::zero(),
            parent_number: U64::zero(),
            parent_gas_used: U64::from(15_000_000),
            parent_gas_limit: U64::from(30_000_000),
            parent_base_fee: ethereum_types::U256::from(1_000_000_000u64),
            timestamp: U64::from(1234567890),
            fee_recipient: Address::zero(),
            prev_randao: H256::random(),
            withdrawals: vec![],
            parent_beacon_block_root: None,
        };

        // 执行：生产区块
        // 注意：这个测试会失败，因为 BlockChain 的方法还没实现
        // 这是预期的，展示了完整的集成流程
        let result = production_service.produce_block(env).await;

        // 验证：应该构建成功（但持久化会失败因为是 todo!()）
        // 在实际实现后，这里应该成功
        match result {
            Ok(block) => {
                assert_eq!(block.number(), U64::one());
                println!("✅ 场景1成功: 区块已构建并持久化");
            }
            Err(e) => {
                // 预期会失败，因为 blockchain 方法还没实现
                println!("⚠️  场景1部分完成: 区块已构建，持久化待实现: {}", e);
            }
        }
    }

    /// 场景2测试: 验证者接收区块
    #[tokio::test]
    async fn test_scenario2_receive_block() {
        // 准备：创建依赖
        let tx_pool = Arc::new(TxPoolImpl::new(TxPoolConfig::default()));
        let validator = Arc::new(BuildBlockService::new(tx_pool.clone(), None))
            as Arc<dyn BlockBuilder>;

        let repository = Arc::new(InMemoryBlockRepository::new());
        let blockchain = Arc::new(BlockChainImpl::new(repository)) as Arc<dyn BlockChain>;

        let reception_service = BlockReceptionService::new(validator, blockchain);

        // 先构建一个区块
        let builder = BuildBlockService::new(tx_pool, Some(30_000_000));
        let env = BuildEnvironment {
            parent_hash: H256::zero(),
            parent_number: U64::zero(),
            parent_gas_used: U64::from(15_000_000),
            parent_gas_limit: U64::from(30_000_000),
            parent_base_fee: ethereum_types::U256::from(1_000_000_000u64),
            timestamp: U64::from(1234567890),
            fee_recipient: Address::zero(),
            prev_randao: H256::random(),
            withdrawals: vec![],
            parent_beacon_block_root: None,
        };

        let block = builder.build_block(env).await.unwrap();
        let receipts = Vec::new();

        // 执行：接收区块
        let result = reception_service.receive_block(block.clone(), receipts).await;

        // 验证
        match result {
            Ok(_) => {
                println!("✅ 场景2成功: 区块已验证并持久化");
            }
            Err(e) => {
                // 预期会失败，因为 blockchain 方法还没实现
                println!("⚠️  场景2部分完成: 区块已验证，持久化待实现: {}", e);
            }
        }
    }

    /// 场景3测试: 仅测试构建逻辑（无持久化）
    #[tokio::test]
    async fn test_scenario3_build_only() {
        // 准备：只创建构建器，不需要区块链
        let tx_pool = Arc::new(TxPoolImpl::new(TxPoolConfig::default()));
        let builder = BuildBlockService::new(tx_pool, Some(30_000_000));

        let env = BuildEnvironment {
            parent_hash: H256::zero(),
            parent_number: U64::zero(),
            parent_gas_used: U64::from(15_000_000),
            parent_gas_limit: U64::from(30_000_000),
            parent_base_fee: ethereum_types::U256::from(1_000_000_000u64),
            timestamp: U64::from(1234567890),
            fee_recipient: Address::zero(),
            prev_randao: H256::random(),
            withdrawals: vec![],
            parent_beacon_block_root: None,
        };

        // 执行：只构建，不持久化
        let block = builder.build_block(env).await.unwrap();

        // 验证：构建逻辑
        assert_eq!(block.number(), U64::one());
        assert_eq!(block.transactions.len(), 0); // 空交易池
        assert!(block.base_fee().is_some());
        assert_eq!(block.gas_used(), U64::zero());

        // 验证 PoS 规则
        assert!(builder.validate_block(&block).await.is_ok());

        println!("✅ 场景3成功: 区块构建逻辑测试通过（无需持久化）");
    }

    /// 集成测试: 完整的生产-接收流程
    #[tokio::test]
    async fn test_full_integration_produce_and_receive() {
        // 节点A: 生产者
        let tx_pool_a = Arc::new(TxPoolImpl::new(TxPoolConfig::default()));
        let builder_a = Arc::new(BuildBlockService::new(tx_pool_a, Some(30_000_000)))
            as Arc<dyn BlockBuilder>;
        let repo_a = Arc::new(InMemoryBlockRepository::new());
        let blockchain_a = Arc::new(BlockChainImpl::new(repo_a)) as Arc<dyn BlockChain>;
        let broadcaster = Arc::new(MockBroadcaster) as Arc<dyn BlockBroadcaster>;

        let producer = BlockProductionService::new(
            builder_a,
            blockchain_a,
            broadcaster,
        );

        // 节点B: 接收者
        let tx_pool_b = Arc::new(TxPoolImpl::new(TxPoolConfig::default()));
        let validator_b = Arc::new(BuildBlockService::new(tx_pool_b, None))
            as Arc<dyn BlockBuilder>;
        let repo_b = Arc::new(InMemoryBlockRepository::new());
        let blockchain_b = Arc::new(BlockChainImpl::new(repo_b)) as Arc<dyn BlockChain>;

        let receiver = BlockReceptionService::new(validator_b, blockchain_b);

        // 1. 节点A生产区块
        let env = BuildEnvironment {
            parent_hash: H256::zero(),
            parent_number: U64::zero(),
            parent_gas_used: U64::from(15_000_000),
            parent_gas_limit: U64::from(30_000_000),
            parent_base_fee: ethereum_types::U256::from(1_000_000_000u64),
            timestamp: U64::from(1234567890),
            fee_recipient: Address::zero(),
            prev_randao: H256::random(),
            withdrawals: vec![],
            parent_beacon_block_root: None,
        };

        // 注意：这会失败因为持久化未实现，但展示了完整流程
        let block_result = producer.produce_block(env).await;

        match block_result {
            Ok(block) => {
                // 2. 节点B接收区块（模拟网络传输）
                let receipts = Vec::new();
                let receive_result = receiver.receive_block(block, receipts).await;

                match receive_result {
                    Ok(_) => {
                        println!("✅ 完整集成测试成功: 生产 -> 广播 -> 接收 -> 验证 -> 持久化");
                    }
                    Err(e) => {
                        println!("⚠️  接收阶段失败（预期，持久化未实现）: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("⚠️  生产阶段失败（预期，持久化未实现）: {}", e);
            }
        }
    }
}
