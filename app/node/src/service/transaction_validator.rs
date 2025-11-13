/// 交易验证器实现 - 包含状态相关的验证逻辑
/// 遵循Clean Architecture原则，验证器依赖抽象接口而非具体实现
///
/// # 架构分层
/// - Trait定义: domain::transaction_validator_trait (领域层抽象)
/// - 具体实现: service::transaction_validator (服务层实现)
/// - 状态查询: AccountStateProvider trait (基础设施层接口)

use crate::domain::entity_types::{DynamicFeeTx, TransactionValidationError};
use crate::service::transaction_validator_trait::TransactionValidator as TransactionValidatorTrait;
use async_trait::async_trait;
use ethereum_types::{Address, U256, U64};

/// 账户状态查询接口 (抽象层)
/// 用于验证器查询账户状态，不依赖具体实现
#[async_trait]
pub trait AccountStateProvider: Send + Sync {
    /// 获取账户余额
    async fn get_balance(&self, address: Address) -> Result<U256, StateError>;

    /// 获取账户nonce
    async fn get_nonce(&self, address: Address) -> Result<U64, StateError>;

    /// 检查地址是否为合约
    async fn is_contract(&self, address: Address) -> Result<bool, StateError>;
}

/// 状态查询错误
#[derive(Debug, Clone)]
pub enum StateError {
    /// 账户不存在
    AccountNotFound(Address),
    /// 数据库错误
    DatabaseError(String),
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::AccountNotFound(addr) => write!(f, "Account not found: {:?}", addr),
            Self::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            Self::Other(msg) => write!(f, "State error: {}", msg),
        }
    }
}

impl std::error::Error for StateError {}

/// 交易验证器配置
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    /// 链ID
    pub chain_id: U64,
    /// 最小gas价格
    pub min_gas_price: U256,
    /// 当前区块的base fee (EIP-1559)
    pub base_fee_per_gas: U256,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            chain_id: U64::from(1), // 主网
            min_gas_price: U256::from(1_000_000_000u64), // 1 Gwei
            base_fee_per_gas: U256::from(20_000_000_000u64), // 20 Gwei
        }
    }
}

/// 交易验证器
/// 负责完整的交易验证流程：基本验证 + 状态验证
pub struct TransactionValidator<S: AccountStateProvider> {
    config: ValidatorConfig,
    state_provider: S,
}

impl<S: AccountStateProvider> TransactionValidator<S> {
    pub fn new(config: ValidatorConfig, state_provider: S) -> Self {
        Self {
            config,
            state_provider,
        }
    }

    /// 完整验证交易（基本验证 + 状态验证）
    /// 这是入池前必须调用的验证方法
    pub async fn validate_transaction(
        &self,
        tx: &DynamicFeeTx,
        sender: Address,
    ) -> Result<(), TransactionValidationError> {
        // 1. 基本验证（无状态）
        tx.validate_basic()?;

        // 2. Chain ID验证
        self.validate_chain_id(tx)?;

        // 3. Gas价格验证
        self.validate_gas_price(tx)?;

        // 4. 状态验证（需要查询账户状态）
        self.validate_state(tx, sender).await?;

        Ok(())
    }

    /// 验证Chain ID
    fn validate_chain_id(&self, tx: &DynamicFeeTx) -> Result<(), TransactionValidationError> {
        if tx.chain_id != self.config.chain_id {
            return Err(TransactionValidationError::InvalidChainId {
                expected: self.config.chain_id,
                actual: tx.chain_id,
            });
        }
        Ok(())
    }

    /// 验证Gas价格
    fn validate_gas_price(&self, tx: &DynamicFeeTx) -> Result<(), TransactionValidationError> {
        // EIP-1559: max_fee_per_gas 必须 >= base_fee
        if tx.max_fee_per_gas < self.config.base_fee_per_gas {
            return Err(TransactionValidationError::GasPriceTooLow {
                min: self.config.base_fee_per_gas,
                actual: tx.max_fee_per_gas,
            });
        }

        // 检查是否满足节点的最小gas价格要求
        if tx.max_fee_per_gas < self.config.min_gas_price {
            return Err(TransactionValidationError::GasPriceTooLow {
                min: self.config.min_gas_price,
                actual: tx.max_fee_per_gas,
            });
        }

        Ok(())
    }

    /// 验证状态相关约束（余额、nonce）
    async fn validate_state(
        &self,
        tx: &DynamicFeeTx,
        sender: Address,
    ) -> Result<(), TransactionValidationError> {
        // 1. 验证nonce
        let current_nonce = self
            .state_provider
            .get_nonce(sender)
            .await
            .map_err(|e| {
                TransactionValidationError::RlpDecodeError(format!("Failed to get nonce: {}", e))
            })?;

        if tx.nonce < current_nonce {
            return Err(TransactionValidationError::NonceTooLow {
                expected: current_nonce,
                actual: tx.nonce,
            });
        }

        // 2. 验证余额
        let balance = self
            .state_provider
            .get_balance(sender)
            .await
            .map_err(|e| {
                TransactionValidationError::RlpDecodeError(format!(
                    "Failed to get balance: {}",
                    e
                ))
            })?;

        let max_cost = tx.max_cost();
        if balance < max_cost {
            return Err(TransactionValidationError::InsufficientBalance {
                required: max_cost,
                actual: balance,
            });
        }

        Ok(())
    }

    /// 快速验证（仅基本验证，用于快速拒绝明显无效的交易）
    pub fn validate_basic_internal(&self, tx: &DynamicFeeTx) -> Result<(), TransactionValidationError> {
        tx.validate_basic()?;
        self.validate_chain_id(tx)?;
        self.validate_gas_price(tx)?;
        Ok(())
    }
}

/// 实现 TransactionValidator trait
/// 通过静态分发（泛型）实现零成本抽象
#[async_trait]
impl<S: AccountStateProvider> TransactionValidatorTrait for TransactionValidator<S> {
    async fn validate_transaction(
        &self,
        tx: &DynamicFeeTx,
        sender: Address,
    ) -> Result<(), TransactionValidationError> {
        // 委托给内部实现
        self.validate_transaction(tx, sender).await
    }

    fn validate_basic(
        &self,
        tx: &DynamicFeeTx,
    ) -> Result<(), TransactionValidationError> {
        // 委托给内部实现
        self.validate_basic_internal(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    /// Mock状态提供者用于测试
    struct MockStateProvider {
        balances: Arc<RwLock<HashMap<Address, U256>>>,
        nonces: Arc<RwLock<HashMap<Address, U64>>>,
    }

    impl MockStateProvider {
        fn new() -> Self {
            Self {
                balances: Arc::new(RwLock::new(HashMap::new())),
                nonces: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        fn set_balance(&self, addr: Address, balance: U256) {
            self.balances.write().unwrap().insert(addr, balance);
        }

        fn set_nonce(&self, addr: Address, nonce: U64) {
            self.nonces.write().unwrap().insert(addr, nonce);
        }
    }

    #[async_trait]
    impl AccountStateProvider for MockStateProvider {
        async fn get_balance(&self, address: Address) -> Result<U256, StateError> {
            self.balances
                .read()
                .unwrap()
                .get(&address)
                .copied()
                .ok_or(StateError::AccountNotFound(address))
        }

        async fn get_nonce(&self, address: Address) -> Result<U64, StateError> {
            Ok(self
                .nonces
                .read()
                .unwrap()
                .get(&address)
                .copied()
                .unwrap_or(U64::zero()))
        }

        async fn is_contract(&self, _address: Address) -> Result<bool, StateError> {
            Ok(false)
        }
    }

    fn create_valid_tx() -> DynamicFeeTx {
        DynamicFeeTx {
            chain_id: U64::from(1),
            nonce: U64::from(0),
            max_priority_fee_per_gas: U256::from(1_000_000_000u64),
            max_fee_per_gas: U256::from(50_000_000_000u64), // 50 Gwei
            gas_limit: U64::from(21000),
            to: Some(Address::from_low_u64_be(0x1234)),
            value: U256::from(1_000_000_000_000_000_000u64), // 1 ETH
            data: vec![],
            access_list: vec![],
            v: U64::from(0),
            r: U256::from(1),
            s: U256::from(1),
        }
    }

    #[tokio::test]
    async fn test_valid_transaction() {
        let mock_state = MockStateProvider::new();
        let sender = Address::from_low_u64_be(0x5678);

        // 设置足够的余额
        mock_state.set_balance(sender, U256::from(2_000_000_000_000_000_000u64)); // 2 ETH
        mock_state.set_nonce(sender, U64::from(0));

        let validator = TransactionValidator::new(ValidatorConfig::default(), mock_state);
        let tx = create_valid_tx();

        let result = validator.validate_transaction(&tx, sender).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_insufficient_balance() {
        let mock_state = MockStateProvider::new();
        let sender = Address::from_low_u64_be(0x5678);

        // 设置不足的余额
        mock_state.set_balance(sender, U256::from(500_000_000_000_000_000u64)); // 0.5 ETH
        mock_state.set_nonce(sender, U64::from(0));

        let validator = TransactionValidator::new(ValidatorConfig::default(), mock_state);
        let tx = create_valid_tx();

        let result = validator.validate_transaction(&tx, sender).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransactionValidationError::InsufficientBalance { .. }
        ));
    }

    #[tokio::test]
    async fn test_nonce_too_low() {
        let mock_state = MockStateProvider::new();
        let sender = Address::from_low_u64_be(0x5678);

        mock_state.set_balance(sender, U256::from(2_000_000_000_000_000_000u64));
        mock_state.set_nonce(sender, U64::from(5)); // 当前nonce是5

        let validator = TransactionValidator::new(ValidatorConfig::default(), mock_state);
        let mut tx = create_valid_tx();
        tx.nonce = U64::from(3); // 交易nonce是3，太低

        let result = validator.validate_transaction(&tx, sender).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransactionValidationError::NonceTooLow { .. }
        ));
    }

    #[tokio::test]
    async fn test_gas_price_too_low() {
        let mock_state = MockStateProvider::new();
        let sender = Address::from_low_u64_be(0x5678);

        mock_state.set_balance(sender, U256::from(2_000_000_000_000_000_000u64));
        mock_state.set_nonce(sender, U64::from(0));

        let config = ValidatorConfig {
            chain_id: U64::from(1),
            min_gas_price: U256::from(1_000_000_000u64),
            base_fee_per_gas: U256::from(100_000_000_000u64), // 100 Gwei
        };

        let validator = TransactionValidator::new(config, mock_state);
        let mut tx = create_valid_tx();
        tx.max_fee_per_gas = U256::from(50_000_000_000u64); // 只有50 Gwei，低于base fee

        let result = validator.validate_transaction(&tx, sender).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransactionValidationError::GasPriceTooLow { .. }
        ));
    }

    #[test]
    fn test_priority_fee_exceeds_max_fee() {
        let tx = DynamicFeeTx {
            chain_id: U64::from(1),
            nonce: U64::from(0),
            max_priority_fee_per_gas: U256::from(50_000_000_000u64), // 50 Gwei
            max_fee_per_gas: U256::from(20_000_000_000u64), // 20 Gwei (更低!)
            gas_limit: U64::from(21000),
            to: Some(Address::zero()),
            value: U256::zero(),
            data: vec![],
            access_list: vec![],
            v: U64::from(0),
            r: U256::from(1),
            s: U256::from(1),
        };

        let result = tx.validate_basic();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransactionValidationError::PriorityFeeExceedsMaxFee
        ));
    }
}
