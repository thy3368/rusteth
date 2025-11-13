use super::ethereum_service::{EthereumService, ServiceError};
use super::types::{
    Block, BlockId, BlockTag, CallRequest, FeeHistory, FilterOptions, Log, SendTransactionRequest,
    Transaction, TransactionReceipt,
};
use crate::domain::command_handler::{CommandError, CommandHandler};
use crate::domain::commands::{CommandResult, EthCommand};
use crate::domain::single_command_handler::HandlerRepository;
use crate::infrastructure::handler_repository::InMemoryHandlerRepository;
use crate::infrastructure::mock_repository::MockEthereumRepository;
use crate::infrastructure::transaction_repo_impl::TxPoolImpl;
use async_trait::async_trait;
use ethereum_types::{Address, H256, U256, U64};

#[derive(Clone)]
pub struct EthereumServiceImpl {
    pub repo: MockEthereumRepository,
    pub tx_pool: TxPoolImpl,
    /// 动态处理器仓储（支持插件化扩展）
    pub handler_repo: InMemoryHandlerRepository,
}

impl EthereumServiceImpl {
    pub fn new(repo: MockEthereumRepository) -> Self {
        Self {
            repo,
            tx_pool: TxPoolImpl::default(),
            handler_repo: InMemoryHandlerRepository::new(),
        }
    }

    /// 创建带有自定义 handler repository 的服务实例
    pub fn with_handler_repo(
        repo: MockEthereumRepository,
        handler_repo: InMemoryHandlerRepository,
    ) -> Self {
        Self {
            repo,
            tx_pool: TxPoolImpl::default(),
            handler_repo,
        }
    }
}

// #[async_trait]
// impl EthereumService for EthereumServiceImpl {
//     async fn get_block_number(&self) -> Result<U64, ServiceError> {
//         Ok(*self.repo.current_block_number.read().unwrap())
//     }
//
//     async fn get_block_by_number(
//         &self,
//         number: U64,
//         _full_tx: bool,
//     ) -> Result<Option<Block>, ServiceError> {
//         Ok(self.repo.blocks.read().unwrap().get(&number).cloned())
//     }
//
//     async fn get_block_by_hash(
//         &self,
//         hash: H256,
//         _full_tx: bool,
//     ) -> Result<Option<Block>, ServiceError> {
//         Ok(self
//             .repo
//             .blocks
//             .read()
//             .unwrap()
//             .values()
//             .find(|b| b.hash == hash)
//             .cloned())
//     }
//
//     async fn get_transaction_by_hash(
//         &self,
//         hash: H256,
//     ) -> Result<Option<Transaction>, ServiceError> {
//         Ok(self.repo.transactions.read().unwrap().get(&hash).cloned())
//     }
//
//     async fn get_transaction_receipt(
//         &self,
//         hash: H256,
//     ) -> Result<Option<TransactionReceipt>, ServiceError> {
//         Ok(self.repo.receipts.read().unwrap().get(&hash).cloned())
//     }
//
//     async fn get_balance(&self, _address: Address, _block: BlockId) -> Result<U256, ServiceError> {
//         // 模拟：返回 1 ETH
//         Ok(U256::from(1_000_000_000_000_000_000u64))
//     }
//
//     async fn get_storage_at(
//         &self,
//         _address: Address,
//         _position: U256,
//         _block: BlockId,
//     ) -> Result<H256, ServiceError> {
//         // 模拟：返回零值
//         Ok(H256::zero())
//     }
//
//     async fn get_transaction_count(
//         &self,
//         _address: Address,
//         _block: BlockId,
//     ) -> Result<U256, ServiceError> {
//         // 模拟：返回 nonce 0
//         Ok(U256::zero())
//     }
//
//     async fn get_code(&self, _address: Address, _block: BlockId) -> Result<Vec<u8>, ServiceError> {
//         // 模拟：返回空代码
//         Ok(vec![])
//     }
//
//     async fn call(&self, _request: CallRequest, _block: BlockId) -> Result<Vec<u8>, ServiceError> {
//         // 模拟：返回空结果
//         Ok(vec![])
//     }
//
//     async fn estimate_gas(&self, _request: CallRequest) -> Result<U256, ServiceError> {
//         // 模拟：返回 21000 gas（标准转账）
//         Ok(U256::from(21000u64))
//     }
//
//     async fn get_logs(&self, _filter: FilterOptions) -> Result<Vec<Log>, ServiceError> {
//         // 模拟：返回空日志列表
//         Ok(vec![])
//     }
//
//     // EIP-1559 相关方法实现
//
//     async fn send_transaction(
//         &self,
//         request: SendTransactionRequest,
//     ) -> Result<H256, ServiceError> {
//         // 模拟：生成交易哈希（基于输入参数的简单组合）
//         use std::collections::hash_map::DefaultHasher;
//         use std::hash::{Hash, Hasher};
//
//         let mut hasher = DefaultHasher::new();
//         request.from.as_bytes().hash(&mut hasher);
//         if let Some(to) = request.to {
//             to.as_bytes().hash(&mut hasher);
//         }
//         if let Some(value) = request.value {
//             let mut value_bytes = [0u8; 32];
//             value.to_big_endian(&mut value_bytes);
//             value_bytes.hash(&mut hasher);
//         }
//
//         let hash_value = hasher.finish();
//         let mut hash = [0u8; 32];
//         hash[0..8].copy_from_slice(&hash_value.to_be_bytes());
//         let tx_hash = H256::from(hash);
//
//         // 创建模拟交易
//         let tx = Transaction {
//             hash: tx_hash,
//             nonce: request.nonce.unwrap_or(U256::zero()),
//             block_hash: None,
//             block_number: None,
//             transaction_index: None,
//             from: request.from,
//             to: request.to,
//             value: request.value.unwrap_or(U256::zero()),
//             gas_price: request.gas_price,
//             gas: request.gas.unwrap_or(U256::from(21000)),
//             input: request.data.unwrap_or_default(),
//             v: U64::zero(),
//             r: U256::zero(),
//             s: U256::zero(),
//             max_fee_per_gas: request.max_fee_per_gas,
//             max_priority_fee_per_gas: request.max_priority_fee_per_gas,
//             transaction_type: if request.max_fee_per_gas.is_some() {
//                 Some(U64::from(2)) // EIP-1559
//             } else {
//                 Some(U64::from(0)) // Legacy
//             },
//         };
//
//         self.repo.add_transaction(tx);
//         Ok(tx_hash)
//     }
//
//     async fn send_raw_transaction(
//         &self,
//         tx: crate::domain::entity_types::DynamicFeeTx,
//         sender: Address,
//     ) -> Result<H256, ServiceError> {
//         // Service层职责：业务逻辑处理
//         // 1. 交易验证（基本验证 + 状态验证）
//         // 2. 入池管理
//         // 3. 事件发布
//
//         use crate::service::repo::transaction_repo::TxPool;
//
//         // Step 1: 基本验证（无状态）
//         tx.validate_basic().map_err(|e| {
//             ServiceError::ValidationError(format!("Basic validation failed: {}", e))
//         })?;
//
//         // Step 2: 状态验证（nonce、余额等）
//         // TODO: 使用TransactionValidator进行完整验证
//         // 需要实现AccountStateProvider trait
//         //
//         // let validator_config = ValidatorConfig {
//         //     chain_id: U64::from(1),
//         //     min_gas_price: U256::from(1_000_000_000u64),
//         //     base_fee_per_gas: U256::from(20_000_000_000u64),
//         // };
//         // let validator = TransactionValidator::new(validator_config, &self.repo);
//         // validator.validate_transaction(&tx, sender).await.map_err(|e| {
//         //     ServiceError::ValidationError(format!("State validation failed: {}", e))
//         // })?;
//
//         // Step 3: 加入交易池
//         let tx_hash = self.tx_pool.add(tx, sender).await.map_err(|e| {
//             ServiceError::Other(format!("Failed to add transaction to pool: {}", e))
//         })?;
//
//         // TODO: 发布事件通知P2P网络广播交易
//
//         Ok(tx_hash)
//     }
//
//     async fn fee_history(
//         &self,
//         block_count: U64,
//         _newest_block: BlockId,
//         reward_percentiles: Option<Vec<f64>>,
//     ) -> Result<FeeHistory, ServiceError> {
//         let current_block = self.get_block_number().await?;
//         let oldest_block = if current_block >= block_count {
//             current_block - block_count + U64::from(1)
//         } else {
//             U64::zero()
//         };
//
//         let count = block_count.as_u64() as usize;
//
//         // 模拟：生成基础费用（EIP-1559）
//         let base_fee_per_gas: Vec<U256> = (0..count + 1)
//             .map(|i| U256::from(20_000_000_000u64 + i as u64 * 1_000_000_000u64))
//             .collect();
//
//         // 模拟：生成 gas 使用比率
//         let gas_used_ratio: Vec<f64> = (0..count).map(|i| 0.5 + (i as f64 * 0.05)).collect();
//
//         // 模拟：生成奖励（如果请求）
//         let reward = reward_percentiles.map(|percentiles| {
//             (0..count)
//                 .map(|_| {
//                     percentiles
//                         .iter()
//                         .map(|&p| U256::from((p * 1_000_000_000.0) as u64))
//                         .collect()
//                 })
//                 .collect()
//         });
//
//         Ok(FeeHistory {
//             oldest_block,
//             base_fee_per_gas,
//             gas_used_ratio,
//             reward,
//         })
//     }
//
//     async fn max_priority_fee_per_gas(&self) -> Result<U256, ServiceError> {
//         // 模拟：返回 2 Gwei 作为建议的优先费用
//         Ok(U256::from(2_000_000_000u64))
//     }
// }

/// 实现 CommandHandler trait，支持 CQRS 模式
///
/// 混合策略：
/// 1. 优先从 handler_repo 动态查找（支持插件化）
/// 2. 如果未找到，则使用内置的 match 分发（向后兼容）
#[async_trait]
impl CommandHandler for EthereumServiceImpl {
    async fn ask(&self, command: EthCommand) -> Result<CommandResult, CommandError> {
        // Step 1: 尝试从 handler repository 动态查找
        if let Some(handler) = self.handler_repo.query(command.name()) {
            return handler.handle(command).await;
        }

        // Step 2: 使用内置的默认实现（向后兼容）
        self.handle_builtin_command(command).await
    }
}

impl EthereumServiceImpl {
    /// 内置命令处理（默认实现）
    ///
    /// 当 handler repository 中未找到对应处理器时调用
    async fn handle_builtin_command(
        &self,
        command: EthCommand,
    ) -> Result<CommandResult, CommandError> {
        match command {
            // 区块查询命令
            EthCommand::GetBlockNumber => {
                let result = self.get_block_number().await?;
                Ok(CommandResult::U64(result))
            }

            EthCommand::GetBlockByNumber(block_id, full_tx) => {
                let number = match block_id {
                    BlockId::Number(num) => num,
                    BlockId::Tag(BlockTag::Latest) => self.get_block_number().await?,
                    BlockId::Tag(BlockTag::Earliest) => U64::zero(),
                    BlockId::Tag(BlockTag::Pending) => {
                        return Err(CommandError::UnsupportedCommand(
                            "待处理区块".to_string(),
                        ))
                    }
                };
                let result = self.get_block_by_number(number, full_tx).await?;
                Ok(CommandResult::Block(result))
            }

            EthCommand::GetBlockByHash(hash, full_tx) => {
                let result = self.get_block_by_hash(hash, full_tx).await?;
                Ok(CommandResult::Block(result))
            }

            // 交易查询命令
            EthCommand::GetTransactionByHash(hash) => {
                let result = self.get_transaction_by_hash(hash).await?;
                Ok(CommandResult::Transaction(result))
            }

            EthCommand::GetTransactionReceipt(hash) => {
                let result = self.get_transaction_receipt(hash).await?;
                Ok(CommandResult::TransactionReceipt(result))
            }

            // 账户状态查询命令
            EthCommand::GetBalance(address, block_id) => {
                let result = self.get_balance(address, block_id).await?;
                Ok(CommandResult::U256(result))
            }

            EthCommand::GetStorageAt(address, position, block_id) => {
                let result = self.get_storage_at(address, position, block_id).await?;
                Ok(CommandResult::Hash(result))
            }

            EthCommand::GetTransactionCount(address, block_id) => {
                let result = self.get_transaction_count(address, block_id).await?;
                Ok(CommandResult::U256(result))
            }

            EthCommand::GetCode(address, block_id) => {
                let result = self.get_code(address, block_id).await?;
                Ok(CommandResult::Bytes(result))
            }

            // 合约调用命令
            EthCommand::Call(request, block_id) => {
                let result = self.call(request, block_id).await?;
                Ok(CommandResult::Bytes(result))
            }

            EthCommand::EstimateGas(request) => {
                let result = self.estimate_gas(request).await?;
                Ok(CommandResult::U256(result))
            }

            EthCommand::GetLogs(filter) => {
                let result = self.get_logs(filter).await?;
                Ok(CommandResult::Logs(result))
            }

            // 网络信息查询命令
            EthCommand::GetChainId => Ok(CommandResult::U64(U64::from(1))),

            EthCommand::GetGasPrice => {
                Ok(CommandResult::U256(U256::from(20_000_000_000u64)))
            }

            EthCommand::GetNetVersion => Ok(CommandResult::String("1".to_string())),

            EthCommand::GetClientVersion => {
                Ok(CommandResult::String("rusteth/0.1.0".to_string()))
            }

            // EIP-1559 交易命令
            EthCommand::SendTransaction(request) => {
                let result = self.send_transaction(request).await?;
                Ok(CommandResult::Hash(result))
            }

            EthCommand::SendRawTransaction(raw_tx, sender) => {
                // 需要先解码 raw_tx
                use crate::inbound::transaction_decoder::decode_raw_transaction;

                let tx = decode_raw_transaction(&raw_tx).map_err(|e| {
                    CommandError::InvalidParams(format!("RLP解码失败: {}", e))
                })?;

                let result = self.send_raw_transaction(tx, sender).await?;
                Ok(CommandResult::Hash(result))
            }

            EthCommand::GetFeeHistory(block_count, newest_block, reward_percentiles) => {
                let result = self
                    .fee_history(block_count, newest_block, reward_percentiles)
                    .await?;
                Ok(CommandResult::FeeHistory(result))
            }

            EthCommand::GetMaxPriorityFeePerGas => {
                let result = self.max_priority_fee_per_gas().await?;
                Ok(CommandResult::U256(result))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_repository() {
        use crate::service::types::BlockTag;

        let mock_repo = MockEthereumRepository::new();
        let service = EthereumServiceImpl::new(mock_repo);

        // 测试区块号
        let block_num = service.get_block_number().await.unwrap();
        assert_eq!(block_num, U64::zero());

        // 测试创世区块
        let genesis = service
            .get_block_by_number(U64::zero(), false)
            .await
            .unwrap();
        assert!(genesis.is_some());
        assert_eq!(genesis.unwrap().number, U64::zero());

        // 测试余额
        let balance = service
            .get_balance(Address::zero(), BlockId::Tag(BlockTag::Latest))
            .await
            .unwrap();
        assert_eq!(balance, U256::from(1_000_000_000_000_000_000u64));
    }
}
