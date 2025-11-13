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
    pub fn hash(&self) -> H256 {
        // TODO: 实现完整的EIP-1559交易哈希计算
        // hash = keccak256(0x02 || rlp([chain_id, nonce, ...]))
        H256::zero()
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
