use crate::infrastructure::mock_repository::MockEthereumRepository;
use crate::service::ethereum_service::{EthereumService, ServiceError};
use async_trait::async_trait;
use ethereum_types::{Address, H256, U256, U64};
use crate::service::types::{Block, BlockId, CallRequest, FeeHistory, FilterOptions, Log, SendTransactionRequest, Transaction, TransactionReceipt};

pub struct EthereumServiceImpl {
    pub repo: MockEthereumRepository,
}

impl EthereumServiceImpl {
    pub fn new(repo: MockEthereumRepository) -> _ {
        todo!()
    }
}

#[async_trait]
impl EthereumService for EthereumServiceImpl {
    async fn get_block_number(&self) -> Result<U64, ServiceError> {
        Ok(*self.current_block_number.read().unwrap())
    }

    async fn get_block_by_number(
        &self,
        number: U64,
        _full_tx: bool,
    ) -> Result<Option<Block>, ServiceError> {
        Ok(repo.blocks.read().unwrap().get(&number).cloned())
    }

    async fn get_block_by_hash(
        &self,
        hash: H256,
        _full_tx: bool,
    ) -> Result<Option<Block>, ServiceError> {
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
    ) -> Result<Option<Transaction>, ServiceError> {
        Ok(self.transactions.read().unwrap().get(&hash).cloned())
    }

    async fn get_transaction_receipt(
        &self,
        hash: H256,
    ) -> Result<Option<TransactionReceipt>, ServiceError> {
        Ok(self.receipts.read().unwrap().get(&hash).cloned())
    }

    async fn get_balance(&self, _address: Address, _block: BlockId) -> Result<U256, ServiceError> {
        // 模拟：返回 1 ETH
        Ok(U256::from(1_000_000_000_000_000_000u64))
    }

    async fn get_storage_at(
        &self,
        _address: Address,
        _position: U256,
        _block: BlockId,
    ) -> Result<H256, ServiceError> {
        // 模拟：返回零值
        Ok(H256::zero())
    }

    async fn get_transaction_count(
        &self,
        _address: Address,
        _block: BlockId,
    ) -> Result<U256, ServiceError> {
        // 模拟：返回 nonce 0
        Ok(U256::zero())
    }

    async fn get_code(&self, _address: Address, _block: BlockId) -> Result<Vec<u8>, ServiceError> {
        // 模拟：返回空代码
        Ok(vec![])
    }

    async fn call(&self, _request: CallRequest, _block: BlockId) -> Result<Vec<u8>, ServiceError> {
        // 模拟：返回空结果
        Ok(vec![])
    }

    async fn estimate_gas(&self, _request: CallRequest) -> Result<U256, ServiceError> {
        // 模拟：返回 21000 gas（标准转账）
        Ok(U256::from(21000u64))
    }

    async fn get_logs(&self, _filter: FilterOptions) -> Result<Vec<Log>, ServiceError> {
        // 模拟：返回空日志列表
        Ok(vec![])
    }

    // EIP-1559 相关方法实现

    async fn send_transaction(
        &self,
        request: SendTransactionRequest,
    ) -> Result<H256, ServiceError> {
        // 模拟：生成交易哈希（基于输入参数的简单组合）
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        request.from.as_bytes().hash(&mut hasher);
        if let Some(to) = request.to {
            to.as_bytes().hash(&mut hasher);
        }
        if let Some(value) = request.value {
            let mut value_bytes = [0u8; 32];
            value.to_big_endian(&mut value_bytes);
            value_bytes.hash(&mut hasher);
        }

        let hash_value = hasher.finish();
        let mut hash = [0u8; 32];
        hash[0..8].copy_from_slice(&hash_value.to_be_bytes());
        let tx_hash = H256::from(hash);

        // 创建模拟交易
        let tx = Transaction {
            hash: tx_hash,
            nonce: request.nonce.unwrap_or(U256::zero()),
            block_hash: None,
            block_number: None,
            transaction_index: None,
            from: request.from,
            to: request.to,
            value: request.value.unwrap_or(U256::zero()),
            gas_price: request.gas_price,
            gas: request.gas.unwrap_or(U256::from(21000)),
            input: request.data.unwrap_or_default(),
            v: U64::zero(),
            r: U256::zero(),
            s: U256::zero(),
            max_fee_per_gas: request.max_fee_per_gas,
            max_priority_fee_per_gas: request.max_priority_fee_per_gas,
            transaction_type: if request.max_fee_per_gas.is_some() {
                Some(U64::from(2)) // EIP-1559
            } else {
                Some(U64::from(0)) // Legacy
            },
        };

        self.add_transaction(tx);
        Ok(tx_hash)
    }

    async fn send_raw_transaction(&self, raw_tx: Vec<u8>) -> Result<H256, ServiceError> {
        // 模拟：对原始交易数据进行哈希
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        raw_tx.hash(&mut hasher);
        let hash_value = hasher.finish();
        let mut hash = [0u8; 32];
        hash[0..8].copy_from_slice(&hash_value.to_be_bytes());
        Ok(H256::from(hash))
    }

    async fn fee_history(
        &self,
        block_count: U64,
        _newest_block: BlockId,
        reward_percentiles: Option<Vec<f64>>,
    ) -> Result<FeeHistory, ServiceError> {
        let current_block = self.get_block_number().await?;
        let oldest_block = if current_block >= block_count {
            current_block - block_count + U64::from(1)
        } else {
            U64::zero()
        };

        let count = block_count.as_u64() as usize;

        // 模拟：生成基础费用（EIP-1559）
        let base_fee_per_gas: Vec<U256> = (0..count + 1)
            .map(|i| U256::from(20_000_000_000u64 + i as u64 * 1_000_000_000u64))
            .collect();

        // 模拟：生成 gas 使用比率
        let gas_used_ratio: Vec<f64> = (0..count).map(|i| 0.5 + (i as f64 * 0.05)).collect();

        // 模拟：生成奖励（如果请求）
        let reward = reward_percentiles.map(|percentiles| {
            (0..count)
                .map(|_| {
                    percentiles
                        .iter()
                        .map(|&p| U256::from((p * 1_000_000_000.0) as u64))
                        .collect()
                })
                .collect()
        });

        Ok(FeeHistory {
            oldest_block,
            base_fee_per_gas,
            gas_used_ratio,
            reward,
        })
    }

    async fn max_priority_fee_per_gas(&self) -> Result<U256, ServiceError> {
        // 模拟：返回 2 Gwei 作为建议的优先费用
        Ok(U256::from(2_000_000_000u64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_repository() {
        use crate::inbound::json_rpc::BlockTag;

        let repo = EthereumServiceImpl::new();

        // 测试区块号
        let block_num = repo.get_block_number().await.unwrap();
        assert_eq!(block_num, U64::zero());

        // 测试创世区块
        let genesis = repo.get_block_by_number(U64::zero(), false).await.unwrap();
        assert!(genesis.is_some());
        assert_eq!(genesis.unwrap().number, U64::zero());

        // 测试余额
        let balance = repo
            .get_balance(Address::zero(), BlockId::Tag(BlockTag::Latest))
            .await
            .unwrap();
        assert_eq!(balance, U256::from(1_000_000_000_000_000_000u64));
    }
}
