//定义 领域层 DynamicFeeTx (EIP-1559) 后续BlobTx (EIP-4844)，参考 geth  core/types/transaction.go;

use ethereum_types::{Address, H256, U256, U64};
use std::fmt;

/// EIP-1559 交易类型 (Type 2)
/// 参考: https://eips.ethereum.org/EIPS/eip-1559
#[repr(align(64))] // Cache-line alignment for performance
#[derive(Debug, Clone, PartialEq)]
pub struct DynamicFeeTx {
    /// 链ID，防止重放攻击
    pub chain_id: U64,
    /// 账户nonce
    pub nonce: U64,
    /// 最大优先费用（小费给矿工）
    pub max_priority_fee_per_gas: U256,
    /// 最大费用（包含base fee + priority fee）
    pub max_fee_per_gas: U256,
    /// Gas限制
    pub gas_limit: U64,
    /// 接收地址（None表示合约创建）
    pub to: Option<Address>,
    /// 转账金额
    pub value: U256,
    /// 交易数据/合约输入
    pub data: Vec<u8>,
    /// 访问列表 (EIP-2930)
    pub access_list: Vec<AccessListItem>,
    /// ECDSA签名 v值
    pub v: U64,
    /// ECDSA签名 r值
    pub r: U256,
    /// ECDSA签名 s值
    pub s: U256,
}

/// EIP-2930 访问列表项
#[derive(Debug, Clone, PartialEq)]
pub struct AccessListItem {
    pub address: Address,
    pub storage_keys: Vec<H256>,
}

/// 交易验证错误
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionValidationError {
    /// 无效的签名
    InvalidSignature,
    /// 无效的chain_id
    InvalidChainId { expected: U64, actual: U64 },
    /// Gas价格过低
    GasPriceTooLow { min: U256, actual: U256 },
    /// Max priority fee 超过 max fee
    PriorityFeeExceedsMaxFee,
    /// Gas限制过低
    InsufficientGas { min: u64, actual: u64 },
    /// Nonce过低（已使用）
    NonceTooLow { expected: U64, actual: U64 },
    /// 账户余额不足
    InsufficientBalance { required: U256, actual: U256 },
    /// 交易数据过大
    DataTooLarge { max: usize, actual: usize },
    /// RLP解码错误
    RlpDecodeError(String),
}

impl fmt::Display for TransactionValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidSignature => write!(f, "Invalid transaction signature"),
            Self::InvalidChainId { expected, actual } => {
                write!(f, "Invalid chain ID: expected {}, got {}", expected, actual)
            }
            Self::GasPriceTooLow { min, actual } => {
                write!(f, "Gas price too low: minimum {}, got {}", min, actual)
            }
            Self::PriorityFeeExceedsMaxFee => {
                write!(f, "Max priority fee exceeds max fee per gas")
            }
            Self::InsufficientGas { min, actual } => {
                write!(f, "Insufficient gas: minimum {}, got {}", min, actual)
            }
            Self::NonceTooLow { expected, actual } => {
                write!(f, "Nonce too low: expected {}, got {}", expected, actual)
            }
            Self::InsufficientBalance { required, actual } => {
                write!(f, "Insufficient balance: required {}, got {}", required, actual)
            }
            Self::DataTooLarge { max, actual } => {
                write!(f, "Transaction data too large: max {} bytes, got {}", max, actual)
            }
            Self::RlpDecodeError(msg) => write!(f, "RLP decode error: {}", msg),
        }
    }
}

impl std::error::Error for TransactionValidationError {}

impl DynamicFeeTx {
    /// 交易类型ID (EIP-1559)
    pub const TRANSACTION_TYPE: u8 = 2;

    /// 最小gas限制（标准转账）
    pub const MIN_GAS_LIMIT: u64 = 21000;

    /// 最大交易数据大小 (128KB)
    pub const MAX_DATA_SIZE: usize = 128 * 1024;

    /// 验证交易基本字段（不包括状态相关验证）
    pub fn validate_basic(&self) -> Result<(), TransactionValidationError> {
        // 1. 验证 max_priority_fee <= max_fee_per_gas
        if self.max_priority_fee_per_gas > self.max_fee_per_gas {
            return Err(TransactionValidationError::PriorityFeeExceedsMaxFee);
        }

        // 2. 验证 gas_limit >= 最小值
        let min_gas = if self.to.is_none() || !self.data.is_empty() {
            // 合约创建或合约调用需要更多gas
            Self::MIN_GAS_LIMIT
        } else {
            Self::MIN_GAS_LIMIT
        };

        if self.gas_limit.as_u64() < min_gas {
            return Err(TransactionValidationError::InsufficientGas {
                min: min_gas,
                actual: self.gas_limit.as_u64(),
            });
        }

        // 3. 验证数据大小
        if self.data.len() > Self::MAX_DATA_SIZE {
            return Err(TransactionValidationError::DataTooLarge {
                max: Self::MAX_DATA_SIZE,
                actual: self.data.len(),
            });
        }

        // 4. 验证签名值有效性
        if self.v > U64::from(1) {
            return Err(TransactionValidationError::InvalidSignature);
        }

        Ok(())
    }

    /// 计算交易的最大成本 (max_fee_per_gas * gas_limit + value)
    pub fn max_cost(&self) -> U256 {
        self.max_fee_per_gas * U256::from(self.gas_limit.as_u64()) + self.value
    }

    /// 恢复发送者地址（需要验证签名）
    pub fn recover_sender(&self) -> Result<Address, TransactionValidationError> {
        // TODO: 实现ECDSA签名恢复
        // 这需要使用k256或secp256k1库进行椭圆曲线签名验证
        // 暂时返回错误，后续实现
        Err(TransactionValidationError::InvalidSignature)
    }

    /// 计算交易哈希
    ///
    /// 根据 EIP-2718 和 EIP-1559 规范：
    /// hash = keccak256(0x02 || rlp([chain_id, nonce, max_priority_fee_per_gas,
    ///                               max_fee_per_gas, gas_limit, to, value, data,
    ///                               access_list, v, r, s]))
    pub fn hash(&self) -> H256 {
        use rlp::RlpStream;
        use sha3::{Digest, Keccak256};

        // 构建 RLP 编码（12 个字段）
        let mut stream = RlpStream::new_list(12);
        stream.append(&self.chain_id);
        stream.append(&self.nonce);
        stream.append(&self.max_priority_fee_per_gas);
        stream.append(&self.max_fee_per_gas);
        stream.append(&self.gas_limit);

        // to 字段：None 表示合约创建，编码为空字节
        if let Some(to) = self.to {
            stream.append(&to);
        } else {
            stream.append(&vec![0u8; 0]); // 空字节数组
        }

        stream.append(&self.value);
        stream.append(&self.data);

        // access_list 编码
        stream.begin_list(self.access_list.len());
        for item in &self.access_list {
            stream.begin_list(2);
            stream.append(&item.address);
            stream.begin_list(item.storage_keys.len());
            for key in &item.storage_keys {
                stream.append(key);
            }
        }

        // 签名字段
        stream.append(&self.v);
        stream.append(&self.r);
        stream.append(&self.s);

        let rlp_encoded = stream.out();

        // 添加交易类型前缀 0x02（EIP-1559）
        let mut tx_bytes = vec![Self::TRANSACTION_TYPE];
        tx_bytes.extend_from_slice(&rlp_encoded);

        // 计算 keccak256 哈希
        let mut hasher = Keccak256::new();
        hasher.update(&tx_bytes);
        let hash_result = hasher.finalize();

        H256::from_slice(&hash_result)
    }
}

/// EIP-4844 Blob交易 (Type 3) - 预留接口
pub trait TransactionEip4844 {
    fn blob_hashes(&self) -> &[H256];
    fn max_fee_per_blob_gas(&self) -> U256;
}

/// EIP-2930 访问列表交易 (Type 1) - 预留接口
pub trait TransactionEip2930 {
    fn access_list(&self) -> &[AccessListItem];
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 创建测试用的最小交易
    fn create_minimal_tx() -> DynamicFeeTx {
        DynamicFeeTx {
            chain_id: U64::from(1),
            nonce: U64::from(0),
            max_priority_fee_per_gas: U256::from(1_000_000_000u64),
            max_fee_per_gas: U256::from(2_000_000_000u64),
            gas_limit: U64::from(21000),
            to: Some(Address::from_low_u64_be(0x1234)),
            value: U256::from(1_000_000_000_000_000_000u64),
            data: vec![],
            access_list: vec![],
            v: U64::from(0),
            r: U256::from(1),
            s: U256::from(1),
        }
    }

    #[test]
    fn test_transaction_hash_deterministic() {
        // 交易哈希应该是确定性的
        let tx1 = create_minimal_tx();
        let tx2 = create_minimal_tx();

        let hash1 = tx1.hash();
        let hash2 = tx2.hash();

        assert_eq!(hash1, hash2, "相同交易应该生成相同的哈希");
        assert_ne!(hash1, H256::zero(), "哈希不应该为零");
    }

    #[test]
    fn test_transaction_hash_different_for_different_tx() {
        let tx1 = create_minimal_tx();

        let mut tx2 = create_minimal_tx();
        tx2.nonce = U64::from(1); // 修改 nonce

        let hash1 = tx1.hash();
        let hash2 = tx2.hash();

        assert_ne!(hash1, hash2, "不同交易应该生成不同的哈希");
    }

    #[test]
    fn test_transaction_hash_with_contract_creation() {
        let mut tx = create_minimal_tx();
        tx.to = None; // 合约创建
        tx.data = vec![0x60, 0x60, 0x60, 0x40]; // 合约字节码

        let hash = tx.hash();
        assert_ne!(hash, H256::zero());
    }

    #[test]
    fn test_transaction_hash_with_access_list() {
        let mut tx = create_minimal_tx();
        tx.access_list = vec![
            AccessListItem {
                address: Address::from_low_u64_be(0x5678),
                storage_keys: vec![
                    H256::from_low_u64_be(1),
                    H256::from_low_u64_be(2),
                ],
            },
        ];

        let hash = tx.hash();
        assert_ne!(hash, H256::zero());

        // 验证带访问列表的交易哈希与不带访问列表的不同
        let tx_without_access_list = create_minimal_tx();
        assert_ne!(hash, tx_without_access_list.hash());
    }

    #[test]
    fn test_validate_basic_success() {
        let tx = create_minimal_tx();
        assert!(tx.validate_basic().is_ok());
    }

    #[test]
    fn test_validate_basic_priority_fee_exceeds_max_fee() {
        let mut tx = create_minimal_tx();
        tx.max_priority_fee_per_gas = U256::from(3_000_000_000u64);
        tx.max_fee_per_gas = U256::from(2_000_000_000u64);

        let result = tx.validate_basic();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransactionValidationError::PriorityFeeExceedsMaxFee
        ));
    }

    #[test]
    fn test_validate_basic_insufficient_gas() {
        let mut tx = create_minimal_tx();
        tx.gas_limit = U64::from(20000); // 低于最小值 21000

        let result = tx.validate_basic();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransactionValidationError::InsufficientGas { .. }
        ));
    }

    #[test]
    fn test_validate_basic_data_too_large() {
        let mut tx = create_minimal_tx();
        tx.data = vec![0u8; DynamicFeeTx::MAX_DATA_SIZE + 1];

        let result = tx.validate_basic();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransactionValidationError::DataTooLarge { .. }
        ));
    }

    #[test]
    fn test_max_cost_calculation() {
        let tx = create_minimal_tx();

        // max_cost = max_fee_per_gas * gas_limit + value
        let expected = U256::from(2_000_000_000u64) * U256::from(21000)
            + U256::from(1_000_000_000_000_000_000u64);

        assert_eq!(tx.max_cost(), expected);
    }

    #[test]
    fn test_max_cost_with_zero_value() {
        let mut tx = create_minimal_tx();
        tx.value = U256::zero();

        let expected = U256::from(2_000_000_000u64) * U256::from(21000);
        assert_eq!(tx.max_cost(), expected);
    }
}
