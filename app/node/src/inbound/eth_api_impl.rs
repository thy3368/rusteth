//! 以太坊 JSON-RPC API 实现
//!
//! 本模块实现了 EthApiExecutor trait 的具体业务逻辑。
//! 遵循整洁架构原则，依赖于仓储接口而非具体实现。

use async_trait::async_trait;
use ethereum_types::U64;
use crate::inbound::eth_api_trait::EthApiExecutor;
use crate::inbound::json_rpc::{
    BlockId, BlockTag, EthereumRepository, RpcMethodError, EthJsonRpcHandler
};

/// EthJsonRpcHandler 的 EthApiExecutor trait 实现
#[async_trait]
impl<R: EthereumRepository> EthApiExecutor for EthJsonRpcHandler<R> {
    /// eth_blockNumber - 返回当前区块号
    async fn eth_block_number(&self) -> Result<serde_json::Value, RpcMethodError> {
        let block_number = self.repository.get_block_number().await?;
        Ok(serde_json::to_value(block_number)?)
    }

    /// eth_getBlockByNumber - 根据区块号返回区块信息
    async fn eth_get_block_by_number(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (BlockId, bool) = serde_json::from_value(params)?;
        let block_number = match params.0 {
            BlockId::Number(num) => num,
            BlockId::Tag(BlockTag::Latest) => self.repository.get_block_number().await?,
            BlockId::Tag(BlockTag::Earliest) => U64::zero(),
            BlockId::Tag(BlockTag::Pending) => {
                return Err(RpcMethodError::UnsupportedFeature("待处理区块".to_string()))
            }
        };

        let block = self.repository.get_block_by_number(block_number, params.1).await?;
        Ok(serde_json::to_value(block)?)
    }

    /// eth_getBlockByHash - 根据区块哈希返回区块信息
    async fn eth_get_block_by_hash(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (ethereum_types::H256, bool) = serde_json::from_value(params)?;
        let block = self.repository.get_block_by_hash(params.0, params.1).await?;
        Ok(serde_json::to_value(block)?)
    }

    /// eth_getTransactionByHash - 根据交易哈希返回交易信息
    async fn eth_get_transaction_by_hash(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (ethereum_types::H256,) = serde_json::from_value(params)?;
        let tx = self.repository.get_transaction_by_hash(params.0).await?;
        Ok(serde_json::to_value(tx)?)
    }

    /// eth_getTransactionReceipt - 根据交易哈希返回交易收据
    async fn eth_get_transaction_receipt(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (ethereum_types::H256,) = serde_json::from_value(params)?;
        let receipt = self.repository.get_transaction_receipt(params.0).await?;
        Ok(serde_json::to_value(receipt)?)
    }

    /// eth_getBalance - 返回账户余额
    async fn eth_get_balance(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (ethereum_types::Address, BlockId) = serde_json::from_value(params)?;
        let balance = self.repository.get_balance(params.0, params.1).await?;
        Ok(serde_json::to_value(balance)?)
    }

    /// eth_getStorageAt - 返回指定位置的存储值
    async fn eth_get_storage_at(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (ethereum_types::Address, ethereum_types::U256, BlockId) =
            serde_json::from_value(params)?;
        let value = self.repository.get_storage_at(params.0, params.1, params.2).await?;
        Ok(serde_json::to_value(value)?)
    }

    /// eth_getTransactionCount - 返回账户的交易数量（nonce）
    async fn eth_get_transaction_count(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (ethereum_types::Address, BlockId) = serde_json::from_value(params)?;
        let count = self.repository.get_transaction_count(params.0, params.1).await?;
        Ok(serde_json::to_value(count)?)
    }

    /// eth_getCode - 返回合约代码
    async fn eth_get_code(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (ethereum_types::Address, BlockId) = serde_json::from_value(params)?;
        let code = self.repository.get_code(params.0, params.1).await?;
        Ok(serde_json::to_value(hex::encode(code))?)
    }

    /// eth_call - 执行调用（不创建交易）
    async fn eth_call(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (crate::inbound::json_rpc::CallRequest, BlockId) =
            serde_json::from_value(params)?;
        let result = self.repository.call(params.0, params.1).await?;
        Ok(serde_json::to_value(hex::encode(result))?)
    }

    /// eth_estimateGas - 估算交易的 Gas 消耗
    async fn eth_estimate_gas(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (crate::inbound::json_rpc::CallRequest,) =
            serde_json::from_value(params)?;
        let gas = self.repository.estimate_gas(params.0).await?;
        Ok(serde_json::to_value(gas)?)
    }

    /// eth_getLogs - 返回匹配过滤器的日志
    async fn eth_get_logs(
        &self,
        params: serde_json::Value
    ) -> Result<serde_json::Value, RpcMethodError> {
        let params: (crate::inbound::json_rpc::FilterOptions,) =
            serde_json::from_value(params)?;
        let logs = self.repository.get_logs(params.0).await?;
        Ok(serde_json::to_value(logs)?)
    }

    /// eth_chainId - 返回链 ID
    async fn eth_chain_id(&self) -> Result<serde_json::Value, RpcMethodError> {
        Ok(serde_json::to_value(U64::from(1))?) // 主网 = 1
    }

    /// eth_gasPrice - 返回当前 Gas 价格
    async fn eth_gas_price(&self) -> Result<serde_json::Value, RpcMethodError> {
        Ok(serde_json::to_value(ethereum_types::U256::from(20_000_000_000u64))?) // 20 Gwei
    }

    /// net_version - 返回网络 ID
    async fn net_version(&self) -> Result<serde_json::Value, RpcMethodError> {
        Ok(serde_json::to_value("1")?)
    }

    /// web3_clientVersion - 返回客户端版本
    async fn web3_client_version(&self) -> Result<serde_json::Value, RpcMethodError> {
        Ok(serde_json::to_value("rusteth/0.1.0")?)
    }
}
