//! 模拟以太坊仓储实现
//!
//! 这是一个简单的内存实现，用于测试和开发

use ethereum_types::{Address, Bloom, H256, H64, U256, U64};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::service::types::{Block, Transaction, TransactionReceipt};

/// 模拟的内存以太坊仓储（支持 Clone 用于静态分发）
#[derive(Clone)]
pub struct MockEthereumRepository {
    pub(crate) blocks: Arc<RwLock<HashMap<U64, Block>>>,
    pub(crate) transactions: Arc<RwLock<HashMap<H256, Transaction>>>,
    pub(crate) receipts: Arc<RwLock<HashMap<H256, TransactionReceipt>>>,
    pub(crate) current_block_number: Arc<RwLock<U64>>,
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

        self.blocks
            .write()
            .unwrap()
            .insert(U64::zero(), genesis_block);
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
