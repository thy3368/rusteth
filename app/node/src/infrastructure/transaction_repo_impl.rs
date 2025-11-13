/// 交易池内存实现
/// 采用Erlang风格的无状态设计：服务与状态分离

use crate::domain::tx_types::DynamicFeeTx;
use crate::service::repo::transaction_repo::{TxPool, TxPoolError, TxPoolStats};
use async_trait::async_trait;
use ethereum_types::{Address, H256, U256};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};

/// 交易池配置
#[derive(Debug, Clone)]
pub struct TxPoolConfig {
    /// 最大交易数
    pub max_pending: usize,
    /// 最大队列交易数
    pub max_queued: usize,
    /// 替换交易的最小价格涨幅（10% = 110）
    pub price_bump_percent: u64,
}

impl Default for TxPoolConfig {
    fn default() -> Self {
        Self {
            max_pending: 4096,
            max_queued: 1024,
            price_bump_percent: 110, // 10% bump
        }
    }
}

/// 交易池状态（独立的数据结构）
/// 遵循Erlang风格：状态与行为分离
struct TxPoolState {
    /// 所有交易映射 hash -> (tx, sender)
    transactions: HashMap<H256, (DynamicFeeTx, Address)>,
    /// Pending交易：按sender分组，每个sender的交易按nonce排序
    pending: HashMap<Address, BTreeMap<u64, H256>>,
    /// Queued交易：按sender分组
    queued: HashMap<Address, BTreeMap<u64, H256>>,
}

impl TxPoolState {
    fn new() -> Self {
        Self {
            transactions: HashMap::new(),
            pending: HashMap::new(),
            queued: HashMap::new(),
        }
    }

    fn total_count(&self) -> usize {
        self.transactions.len()
    }

    fn pending_count(&self) -> usize {
        self.pending.values().map(|txs| txs.len()).sum()
    }

    fn queued_count(&self) -> usize {
        self.queued.values().map(|txs| txs.len()).sum()
    }
}

/// 交易池实现（无状态服务）
/// 所有状态存储在Arc<RwLock<TxPoolState>>中
#[derive(Clone)]
pub struct TxPoolImpl {
    config: TxPoolConfig,
    state: Arc<RwLock<TxPoolState>>,
}

impl TxPoolImpl {
    pub fn new(config: TxPoolConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(TxPoolState::new())),
        }
    }

    /// 计算交易哈希
    ///
    /// 使用 DynamicFeeTx::hash() 方法计算标准的 EIP-1559 交易哈希
    /// hash = keccak256(0x02 || rlp([...]))
    fn compute_tx_hash(&self, tx: &DynamicFeeTx) -> H256 {
        tx.hash()
    }

    /// 检查是否需要价格提升（替换交易）
    fn needs_price_bump(&self, old_tx: &DynamicFeeTx, new_tx: &DynamicFeeTx) -> bool {
        let old_price = old_tx.max_fee_per_gas;
        let required_price = old_price * U256::from(self.config.price_bump_percent) / U256::from(100);
        new_tx.max_fee_per_gas >= required_price
    }
}

impl Default for TxPoolImpl {
    fn default() -> Self {
        Self::new(TxPoolConfig::default())
    }
}

#[async_trait]
impl TxPool for TxPoolImpl {
    async fn add(&self, tx: DynamicFeeTx, sender: Address) -> Result<H256, TxPoolError> {
        let tx_hash = self.compute_tx_hash(&tx);
        let nonce = tx.nonce.as_u64();

        let mut state = self.state.write().unwrap();

        // 检查是否已存在
        if let Some((existing_tx, _)) = state.transactions.get(&tx_hash) {
            // 如果是替换交易，检查价格提升
            if !self.needs_price_bump(existing_tx, &tx) {
                return Err(TxPoolError::ReplacementUnderpriced {
                    current: format!("{}", existing_tx.max_fee_per_gas),
                    required: format!("{}", tx.max_fee_per_gas),
                });
            }
            // 允许替换
        }

        // 检查容量
        if state.total_count() >= self.config.max_pending + self.config.max_queued {
            return Err(TxPoolError::PoolFull {
                current: state.total_count(),
                max: self.config.max_pending + self.config.max_queued,
            });
        }

        // 存储交易
        state.transactions.insert(tx_hash, (tx.clone(), sender));

        // 决定放入pending还是queued
        // 简化逻辑：先都放pending，实际应该检查nonce连续性
        let sender_pending = state.pending.entry(sender).or_insert_with(BTreeMap::new);
        sender_pending.insert(nonce, tx_hash);

        Ok(tx_hash)
    }

    async fn get(&self, hash: &H256) -> Result<Option<DynamicFeeTx>, TxPoolError> {
        let state = self.state.read().unwrap();
        Ok(state.transactions.get(hash).map(|(tx, _)| tx.clone()))
    }

    async fn get_pending_by_sender(&self, sender: Address) -> Result<Vec<DynamicFeeTx>, TxPoolError> {
        let state = self.state.read().unwrap();

        let mut result = Vec::new();
        if let Some(sender_txs) = state.pending.get(&sender) {
            for hash in sender_txs.values() {
                if let Some((tx, _)) = state.transactions.get(hash) {
                    result.push(tx.clone());
                }
            }
        }

        Ok(result)
    }

    async fn get_pending(&self, max_count: usize, base_fee: Option<u64>) -> Result<Vec<DynamicFeeTx>, TxPoolError> {
        let state = self.state.read().unwrap();

        let mut all_pending = Vec::new();

        // 收集所有pending交易
        for sender_txs in state.pending.values() {
            for hash in sender_txs.values() {
                if let Some((tx, _)) = state.transactions.get(hash) {
                    // 如果设置了base_fee，过滤掉max_fee_per_gas太低的交易
                    if let Some(base) = base_fee {
                        if tx.max_fee_per_gas < U256::from(base) {
                            continue;
                        }
                    }
                    all_pending.push(tx.clone());
                }
            }
        }

        // 按max_fee_per_gas降序排序（矿工收益最大化）
        all_pending.sort_by(|a, b| b.max_fee_per_gas.cmp(&a.max_fee_per_gas));

        // 限制数量
        all_pending.truncate(max_count);

        Ok(all_pending)
    }

    async fn remove(&self, hash: &H256) -> Result<(), TxPoolError> {
        let mut state = self.state.write().unwrap();

        if let Some((tx, sender)) = state.transactions.remove(hash) {
            let nonce = tx.nonce.as_u64();

            // 从pending移除
            if let Some(sender_pending) = state.pending.get_mut(&sender) {
                sender_pending.remove(&nonce);
                if sender_pending.is_empty() {
                    state.pending.remove(&sender);
                }
            }

            // 从queued移除
            if let Some(sender_queued) = state.queued.get_mut(&sender) {
                sender_queued.remove(&nonce);
                if sender_queued.is_empty() {
                    state.queued.remove(&sender);
                }
            }
        }

        Ok(())
    }

    async fn remove_batch(&self, hashes: &[H256]) -> Result<(), TxPoolError> {
        for hash in hashes {
            self.remove(hash).await?;
        }
        Ok(())
    }

    async fn stats(&self) -> Result<TxPoolStats, TxPoolError> {
        let state = self.state.read().unwrap();
        Ok(TxPoolStats {
            pending: state.pending_count(),
            queued: state.queued_count(),
            capacity: self.config.max_pending + self.config.max_queued,
        })
    }

    async fn clear(&self) -> Result<(), TxPoolError> {
        let mut state = self.state.write().unwrap();
        state.transactions.clear();
        state.pending.clear();
        state.queued.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tx_types::AccessListItem;

    fn create_test_tx(nonce: u64, max_fee: u64) -> DynamicFeeTx {
        DynamicFeeTx {
            chain_id: ethereum_types::U64::from(1),
            nonce: ethereum_types::U64::from(nonce),
            max_priority_fee_per_gas: U256::from(1_000_000_000u64),
            max_fee_per_gas: U256::from(max_fee),
            gas_limit: ethereum_types::U64::from(21000),
            to: Some(Address::from_low_u64_be(0x1234)),
            value: U256::from(1_000_000_000_000_000_000u64),
            data: vec![],
            access_list: vec![],
            v: ethereum_types::U64::from(0),
            r: U256::from(1),
            s: U256::from(1),
        }
    }

    #[tokio::test]
    async fn test_add_and_get_transaction() {
        let pool = TxPoolImpl::default();
        let sender = Address::from_low_u64_be(0x5678);
        let tx = create_test_tx(0, 50_000_000_000);

        let hash = pool.add(tx.clone(), sender).await.unwrap();

        let retrieved = pool.get(&hash).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().nonce, tx.nonce);
    }

    #[tokio::test]
    async fn test_get_pending_transactions() {
        let pool = TxPoolImpl::default();
        let sender = Address::from_low_u64_be(0x5678);

        // 添加多笔交易
        pool.add(create_test_tx(0, 50_000_000_000), sender).await.unwrap();
        pool.add(create_test_tx(1, 60_000_000_000), sender).await.unwrap();
        pool.add(create_test_tx(2, 40_000_000_000), sender).await.unwrap();

        let pending = pool.get_pending(10, None).await.unwrap();
        assert_eq!(pending.len(), 3);

        // 验证按价格排序（降序）
        assert!(pending[0].max_fee_per_gas >= pending[1].max_fee_per_gas);
        assert!(pending[1].max_fee_per_gas >= pending[2].max_fee_per_gas);
    }

    #[tokio::test]
    async fn test_remove_transaction() {
        let pool = TxPoolImpl::default();
        let sender = Address::from_low_u64_be(0x5678);
        let tx = create_test_tx(0, 50_000_000_000);

        let hash = pool.add(tx, sender).await.unwrap();
        assert!(pool.get(&hash).await.unwrap().is_some());

        pool.remove(&hash).await.unwrap();
        assert!(pool.get(&hash).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_pool_stats() {
        let pool = TxPoolImpl::default();
        let sender = Address::from_low_u64_be(0x5678);

        pool.add(create_test_tx(0, 50_000_000_000), sender).await.unwrap();
        pool.add(create_test_tx(1, 60_000_000_000), sender).await.unwrap();

        let stats = pool.stats().await.unwrap();
        assert_eq!(stats.pending, 2);
        assert_eq!(stats.queued, 0);
    }
}

