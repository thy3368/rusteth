//! 命令分发器 - 统一的命令处理入口
//!
//! 直接处理所有 ETH RPC 命令，使用 EthereumService 执行业务逻辑

use crate::domain::command_types::{CommandError, CommandResult, EthCommand};
use crate::service::ethereum_service_trait::EthereumService;
use crate::domain::command_types::BlockTag;
use ethereum_types::U64;
use std::sync::Arc;

/// 命令分发器
#[derive(Clone)]
pub struct CommandDispatcher<S: EthereumService> {
    service: Arc<S>,
}

impl<S: EthereumService> CommandDispatcher<S> {
    /// 创建新的命令分发器
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }

    /// 处理命令
    pub async fn ask(&self, command: EthCommand) -> Result<CommandResult, CommandError> {
        //开始消费command
        match command {
            // ============ 区块查询命令 ============
            EthCommand::GetBlockNumber => {
                let result = self.service.get_block_number().await?;
                Ok(CommandResult::U64(result))
            }

            EthCommand::GetBlockByNumber(block_id, full_tx) => {
                let number = match block_id {
                    crate::domain::command_types::BlockId::Number(num) => num,
                    crate::domain::command_types::BlockId::Tag(BlockTag::Latest) => {
                        self.service.get_block_number().await?
                    }
                    crate::domain::command_types::BlockId::Tag(BlockTag::Earliest) => U64::zero(),
                    crate::domain::command_types::BlockId::Tag(BlockTag::Pending) => {
                        return Err(CommandError::UnsupportedCommand("待处理区块".to_string()))
                    }
                };
                let result = self.service.get_block_by_number(number, full_tx).await?;
                Ok(CommandResult::Block(result))
            }

            EthCommand::GetBlockByHash(hash, full_tx) => {
                let result = self.service.get_block_by_hash(hash, full_tx).await?;
                Ok(CommandResult::Block(result))
            }

            // ============ 交易查询命令 ============
            EthCommand::GetTransactionByHash(hash) => {
                let result = self.service.get_transaction_by_hash(hash).await?;
                Ok(CommandResult::Transaction(result))
            }

            EthCommand::GetTransactionReceipt(hash) => {
                let result = self.service.get_transaction_receipt(hash).await?;
                Ok(CommandResult::TransactionReceipt(result))
            }

            // ============ 账户状态查询命令 ============
            EthCommand::GetBalance(address, block_id) => {
                let result = self.service.get_balance(address, block_id).await?;
                Ok(CommandResult::U256(result))
            }

            EthCommand::GetStorageAt(address, position, block_id) => {
                let result = self
                    .service
                    .get_storage_at(address, position, block_id)
                    .await?;
                Ok(CommandResult::Hash(result))
            }

            EthCommand::GetTransactionCount(address, block_id) => {
                let result = self
                    .service
                    .get_transaction_count(address, block_id)
                    .await?;
                Ok(CommandResult::U256(result))
            }

            EthCommand::GetCode(address, block_id) => {
                let result = self.service.get_code(address, block_id).await?;
                Ok(CommandResult::Bytes(result))
            }

            // ============ 合约调用命令 ============
            EthCommand::Call(request, block_id) => {
                let result = self.service.call(request, block_id).await?;
                Ok(CommandResult::Bytes(result))
            }

            EthCommand::EstimateGas(request) => {
                let result = self.service.estimate_gas(request).await?;
                Ok(CommandResult::U256(result))
            }

            EthCommand::GetLogs(filter) => {
                let result = self.service.get_logs(filter).await?;
                Ok(CommandResult::Logs(result))
            }

            // ============ 网络信息查询命令 ============
            EthCommand::GetChainId => Ok(CommandResult::U64(U64::from(1))),

            EthCommand::GetGasPrice => Ok(CommandResult::U256(ethereum_types::U256::from(
                20_000_000_000u64,
            ))),

            EthCommand::GetNetVersion => Ok(CommandResult::String("1".to_string())),

            EthCommand::GetClientVersion => Ok(CommandResult::String("rusteth/0.1.0".to_string())),

            // ============ EIP-1559 交易命令 ============
            EthCommand::SendTransaction(request) => {
                let result = self.service.send_transaction(request).await?;
                Ok(CommandResult::Hash(result))
            }

            EthCommand::SendRawTransaction(raw_tx, sender) => {
                use crate::inbound::transaction_decoder::decode_raw_transaction;

                let tx = decode_raw_transaction(&raw_tx)
                    .map_err(|e| CommandError::InvalidParams(format!("RLP解码失败: {}", e)))?;

                let result = self.service.send_raw_transaction(tx, sender).await?;
                Ok(CommandResult::Hash(result))
            }

            EthCommand::GetFeeHistory(block_count, newest_block, reward_percentiles) => {
                let result = self
                    .service
                    .fee_history(block_count, newest_block, reward_percentiles)
                    .await?;
                Ok(CommandResult::FeeHistory(result))
            }

            EthCommand::GetMaxPriorityFeePerGas => {
                let result = self.service.max_priority_fee_per_gas().await?;
                Ok(CommandResult::U256(result))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::mock_repository::MockEthereumRepository;
    use crate::service::ethereum_service_impl::EthereumServiceImpl;

    #[tokio::test]
    async fn test_dispatcher() {
        let mock_repo = MockEthereumRepository::new();
        let service = Arc::new(EthereumServiceImpl::new(mock_repo));
        let dispatcher = CommandDispatcher::new(service);

        let result = dispatcher.ask(EthCommand::GetBlockNumber).await;
        assert!(result.is_ok());

        if let Ok(CommandResult::U64(num)) = result {
            assert_eq!(num, U64::zero());
        }
    }
}
