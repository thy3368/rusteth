//! 模拟以太坊仓储实现
//!
//! 这是一个简单的内存实现，用于测试和开发

use crate::inbound::json_rpc::{
    Block, BlockId, CallRequest, EthereumRepository, FilterOptions, Log,
    RepositoryError, Transaction, TransactionReceipt,
};
use async_trait::async_trait;
use ethereum_types::{Address, Bloom, H256, H64, U256, U64};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 模拟的内存以太坊仓储（支持 Clone 用于静态分发）
#[derive(Clone)]
pub struct MockEthereumRepository {
    blocks: Arc<RwLock<HashMap<U64, Block>>>,
    transactions: Arc<RwLock<HashMap<H256, Transaction>>>,
    receipts: Arc<RwLock<HashMap<H256, TransactionReceipt>>>,
    current_block_number: Arc<RwLock<U64>>,
}

impl MockEthereumRepository {
    pub fn new() -> Self {
        let repo = Self {
            blocks: Arc::new(RwLock::new(HashMap::new())),
            transactions: Arc::new(RwLock::new(HashMap::new())),
            receipts: Arc::new(RwLock::new(HashMap::new())),
            current_block_number: Arc::new(RwLock::new(U64::from(0))),
        };

        // 初始化创世区块
        repo.initialize_genesis();
        repo
    }

    fn initialize_genesis(&self) {
        let genesis_block = Block {
            number: U64::zero(),
            hash: H256::zero(),
            parent_hash: H256::zero(),
            nonce: H64::zero(),
            sha3_uncles: H256::zero(),
            logs_bloom: Bloom::zero(),
            transactions_root: H256::zero(),
            state_root: H256::zero(),
            receipts_root: H256::zero(),
            miner: Address::zero(),
            difficulty: U256::zero(),
            total_difficulty: U256::zero(),
            extra_data: vec![],
            size: U256::zero(),
            gas_limit: U256::from(8_000_000u64),
            gas_used: U256::zero(),
            timestamp: U256::from(0),
            transactions: vec![],
            uncles: vec![],
        };

        self.blocks.write().unwrap().insert(U64::zero(), genesis_block);
    }

    /// 添加模拟区块（用于测试）
    pub fn add_block(&self, block: Block) {
        let number = block.number;
        self.blocks.write().unwrap().insert(number, block);
        *self.current_block_number.write().unwrap() = number;
    }

    /// 添加模拟交易（用于测试）
    pub fn add_transaction(&self, tx: Transaction) {
        self.transactions.write().unwrap().insert(tx.hash, tx);
    }

    /// 添加模拟收据（用于测试）
    pub fn add_receipt(&self, receipt: TransactionReceipt) {
        self.receipts
            .write()
            .unwrap()
            .insert(receipt.transaction_hash, receipt);
    }
}

impl Default for MockEthereumRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EthereumRepository for MockEthereumRepository {
    async fn get_block_number(&self) -> Result<U64, RepositoryError> {
        Ok(*self.current_block_number.read().unwrap())
    }

    async fn get_block_by_number(
        &self,
        number: U64,
        _full_tx: bool,
    ) -> Result<Option<Block>, RepositoryError> {
        Ok(self.blocks.read().unwrap().get(&number).cloned())
    }

    async fn get_block_by_hash(
        &self,
        hash: H256,
        _full_tx: bool,
    ) -> Result<Option<Block>, RepositoryError> {
        Ok(self
            .blocks
            .read()
            .unwrap()
            .values()
            .find(|b| b.hash == hash)
            .cloned())
    }

    async fn get_transaction_by_hash(
        &self,
        hash: H256,
    ) -> Result<Option<Transaction>, RepositoryError> {
        Ok(self.transactions.read().unwrap().get(&hash).cloned())
    }

    async fn get_transaction_receipt(
        &self,
        hash: H256,
    ) -> Result<Option<TransactionReceipt>, RepositoryError> {
        Ok(self.receipts.read().unwrap().get(&hash).cloned())
    }

    async fn get_balance(
        &self,
        _address: Address,
        _block: BlockId,
    ) -> Result<U256, RepositoryError> {
        // 模拟：返回 1 ETH
        Ok(U256::from(1_000_000_000_000_000_000u64))
    }

    async fn get_storage_at(
        &self,
        _address: Address,
        _position: U256,
        _block: BlockId,
    ) -> Result<H256, RepositoryError> {
        // 模拟：返回零值
        Ok(H256::zero())
    }

    async fn get_transaction_count(
        &self,
        _address: Address,
        _block: BlockId,
    ) -> Result<U256, RepositoryError> {
        // 模拟：返回 nonce 0
        Ok(U256::zero())
    }

    async fn get_code(
        &self,
        _address: Address,
        _block: BlockId,
    ) -> Result<Vec<u8>, RepositoryError> {
        // 模拟：返回空代码
        Ok(vec![])
    }

    async fn call(
        &self,
        _request: CallRequest,
        _block: BlockId,
    ) -> Result<Vec<u8>, RepositoryError> {
        // 模拟：返回空结果
        Ok(vec![])
    }

    async fn estimate_gas(&self, _request: CallRequest) -> Result<U256, RepositoryError> {
        // 模拟：返回 21000 gas（标准转账）
        Ok(U256::from(21000u64))
    }

    async fn get_logs(&self, _filter: FilterOptions) -> Result<Vec<Log>, RepositoryError> {
        // 模拟：返回空日志列表
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_repository() {
        use crate::inbound::json_rpc::BlockTag;

        let repo = MockEthereumRepository::new();

        // 测试区块号
        let block_num = repo.get_block_number().await.unwrap();
        assert_eq!(block_num, U64::zero());

        // 测试创世区块
        let genesis = repo.get_block_by_number(U64::zero(), false).await.unwrap();
        assert!(genesis.is_some());
        assert_eq!(genesis.unwrap().number, U64::zero());

        // 测试余额
        let balance = repo.get_balance(Address::zero(), BlockId::Tag(BlockTag::Latest)).await.unwrap();
        assert_eq!(balance, U256::from(1_000_000_000_000_000_000u64));
    }
}
