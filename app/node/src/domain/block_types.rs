/// 区块相关类型定义 - 领域层实体
///
/// 参考标准：
/// - EIP-1559: 费用市场
/// - EIP-3675: PoS共识
/// - EIP-4399: PREVRANDAO
/// - EIP-4844: Blob交易
/// - EIP-4895: 验证者提款
/// - Geth: core/types/block.go

use crate::domain::entity_types::DynamicFeeTx;
use ethereum_types::{Address, Bloom, H256, U256, U64};
use std::fmt;

/// 区块头信息
///
/// 参考: geth/core/types/block.go - Header struct
#[repr(align(64))] // 缓存行对齐优化
#[derive(Debug, Clone, PartialEq)]
pub struct BlockHeader {
    /// 父区块哈希
    pub parent_hash: H256,
    /// Ommers (叔块) 哈希 - PoS后固定为空列表的哈希
    /// Keccak256(RLP([])) = 0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347
    pub ommers_hash: H256,
    /// 矿工/验证者地址 (fee recipient)
    pub fee_recipient: Address,
    /// 状态根 (world state Merkle-Patricia Trie root)
    pub state_root: H256,
    /// 交易根 (transactions Merkle-Patricia Trie root)
    pub transactions_root: H256,
    /// 收据根 (receipts Merkle-Patricia Trie root)
    pub receipts_root: H256,
    /// Bloom过滤器 (用于快速日志查询)
    pub logs_bloom: Bloom,
    /// 难度值 - PoS后固定为0
    pub difficulty: U256,
    /// 区块号
    pub number: U64,
    /// Gas限制
    pub gas_limit: U64,
    /// Gas使用量
    pub gas_used: U64,
    /// 时间戳 (Unix时间)
    pub timestamp: U64,
    /// 额外数据 (最大32字节)
    pub extra_data: Vec<u8>,
    /// MixHash/PrevRandao - PoS后存储RANDAO值
    pub mix_hash: H256,
    /// Nonce - PoS后固定为0x0000000000000000
    pub nonce: u64,
    /// Base fee per gas (EIP-1559)
    pub base_fee_per_gas: Option<U256>,
    /// 提取根 (withdrawals root, EIP-4895)
    pub withdrawals_root: Option<H256>,
    /// Blob gas使用量 (EIP-4844)
    pub blob_gas_used: Option<U64>,
    /// Excess blob gas (EIP-4844)
    pub excess_blob_gas: Option<U64>,
    /// 父区块的Beacon根 (EIP-4788)
    pub parent_beacon_block_root: Option<H256>,
}

impl BlockHeader {
    /// 计算区块头哈希 (Keccak256)
    ///
    /// TODO: 实现完整的RLP编码和哈希计算
    /// hash = keccak256(rlp([parent_hash, ommers_hash, ..., parent_beacon_block_root]))
    pub fn hash(&self) -> H256 {
        // 暂时返回零值，后续实现RLP编码
        H256::zero()
    }

    /// 获取空ommers列表的哈希值 (PoS固定值)
    pub fn empty_ommers_hash() -> H256 {
        H256::from_slice(
            &hex::decode("1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347")
                .expect("Valid hex string"),
        )
    }

    /// 验证PoS区块头 (EIP-3675)
    ///
    /// PoS区块头固定字段验证：
    /// - difficulty = 0
    /// - nonce = 0
    /// - ommers_hash = Keccak256(RLP([]))
    /// - extra_data <= 32字节
    pub fn validate_pos_header(&self) -> Result<(), BlockValidationError> {
        // PoS固定字段验证
        if self.difficulty != U256::zero() {
            return Err(BlockValidationError::InvalidDifficulty {
                expected: U256::zero(),
                actual: self.difficulty,
            });
        }

        if self.nonce != 0 {
            return Err(BlockValidationError::InvalidNonce {
                expected: 0,
                actual: self.nonce,
            });
        }

        // ommers_hash必须是空列表的RLP哈希
        if self.ommers_hash != Self::empty_ommers_hash() {
            return Err(BlockValidationError::InvalidOmmersHash);
        }

        // extra_data长度限制
        if self.extra_data.len() > 32 {
            return Err(BlockValidationError::ExtraDataTooLarge {
                max: 32,
                actual: self.extra_data.len(),
            });
        }

        Ok(())
    }
}

/// 完整区块 (包含头和交易)
///
/// 参考: geth/core/types/block.go - Block struct
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    /// 区块头
    pub header: BlockHeader,
    /// 交易列表
    pub transactions: Vec<DynamicFeeTx>,
    /// 提取列表 (EIP-4895, PoS后的验证者提款)
    pub withdrawals: Vec<Withdrawal>,
}

impl Block {
    /// 获取区块哈希
    pub fn hash(&self) -> H256 {
        self.header.hash()
    }

    /// 获取区块号
    pub fn number(&self) -> U64 {
        self.header.number
    }

    /// 获取gas使用量
    pub fn gas_used(&self) -> U64 {
        self.header.gas_used
    }

    /// 获取gas限制
    pub fn gas_limit(&self) -> U64 {
        self.header.gas_limit
    }

    /// 获取base fee
    pub fn base_fee(&self) -> Option<U256> {
        self.header.base_fee_per_gas
    }
}

/// 提款信息 (EIP-4895)
///
/// 参考: https://eips.ethereum.org/EIPS/eip-4895
#[derive(Debug, Clone, PartialEq)]
pub struct Withdrawal {
    /// 提款索引 (全局递增)
    pub index: U64,
    /// 验证者索引
    pub validator_index: U64,
    /// 接收地址 (执行层地址)
    pub address: Address,
    /// 金额 (单位: Gwei)
    pub amount: U64,
}

/// 区块构建环境
///
/// 封装构建新区块所需的所有父区块信息和外部输入
#[derive(Debug, Clone)]
pub struct BuildEnvironment {
    /// 父区块哈希
    pub parent_hash: H256,
    /// 父区块号
    pub parent_number: U64,
    /// 父区块的gas使用量
    pub parent_gas_used: U64,
    /// 父区块的gas限制
    pub parent_gas_limit: U64,
    /// 父区块的base fee
    pub parent_base_fee: U256,
    /// 当前时间戳
    pub timestamp: U64,
    /// Fee recipient地址 (矿工/验证者)
    pub fee_recipient: Address,
    /// PrevRandao值 (来自信标链)
    pub prev_randao: H256,
    /// 提款列表
    pub withdrawals: Vec<Withdrawal>,
    /// 父区块的Beacon根
    pub parent_beacon_block_root: Option<H256>,
}

/// 区块验证错误
#[derive(Debug, Clone, PartialEq)]
pub enum BlockValidationError {
    /// 无效的难度值
    InvalidDifficulty { expected: U256, actual: U256 },
    /// 无效的nonce
    InvalidNonce { expected: u64, actual: u64 },
    /// 无效的ommers哈希
    InvalidOmmersHash,
    /// Extra data过大
    ExtraDataTooLarge { max: usize, actual: usize },
    /// Gas使用量超过限制
    GasLimitExceeded { limit: u64, used: u64 },
    /// 无效的base fee
    InvalidBaseFee { expected: U256, actual: U256 },
    /// 交易执行失败
    TransactionExecutionFailed(String),
    /// 无效的状态根
    InvalidStateRoot { expected: H256, actual: H256 },
    /// Gas limit调整超出范围
    GasLimitAdjustmentTooLarge { parent: u64, current: u64 },
    /// 其他错误
    Other(String),
}

impl fmt::Display for BlockValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidDifficulty { expected, actual } => {
                write!(f, "Invalid difficulty: expected {}, got {}", expected, actual)
            }
            Self::InvalidNonce { expected, actual } => {
                write!(f, "Invalid nonce: expected {}, got {}", expected, actual)
            }
            Self::InvalidOmmersHash => write!(f, "Invalid ommers hash (must be empty in PoS)"),
            Self::ExtraDataTooLarge { max, actual } => {
                write!(f, "Extra data too large: max {} bytes, got {}", max, actual)
            }
            Self::GasLimitExceeded { limit, used } => {
                write!(f, "Gas limit exceeded: limit {}, used {}", limit, used)
            }
            Self::InvalidBaseFee { expected, actual } => {
                write!(f, "Invalid base fee: expected {}, got {}", expected, actual)
            }
            Self::TransactionExecutionFailed(msg) => {
                write!(f, "Transaction execution failed: {}", msg)
            }
            Self::InvalidStateRoot { expected, actual } => {
                write!(f, "Invalid state root: expected {:?}, got {:?}", expected, actual)
            }
            Self::GasLimitAdjustmentTooLarge { parent, current } => {
                write!(
                    f,
                    "Gas limit adjustment too large: parent {}, current {}",
                    parent, current
                )
            }
            Self::Other(msg) => write!(f, "Block validation error: {}", msg),
        }
    }
}

impl std::error::Error for BlockValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_ommers_hash() {
        let expected = H256::from_slice(
            &hex::decode("1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347")
                .unwrap(),
        );
        assert_eq!(BlockHeader::empty_ommers_hash(), expected);
    }

    #[test]
    fn test_pos_header_validation() {
        let valid_header = BlockHeader {
            parent_hash: H256::zero(),
            ommers_hash: BlockHeader::empty_ommers_hash(),
            fee_recipient: Address::zero(),
            state_root: H256::zero(),
            transactions_root: H256::zero(),
            receipts_root: H256::zero(),
            logs_bloom: Bloom::zero(),
            difficulty: U256::zero(), // PoS
            number: U64::one(),
            gas_limit: U64::from(30_000_000),
            gas_used: U64::from(15_000_000),
            timestamp: U64::from(1234567890),
            extra_data: vec![],
            mix_hash: H256::random(),
            nonce: 0, // PoS
            base_fee_per_gas: Some(U256::from(1_000_000_000u64)),
            withdrawals_root: None,
            blob_gas_used: None,
            excess_blob_gas: None,
            parent_beacon_block_root: None,
        };

        assert!(valid_header.validate_pos_header().is_ok());

        // 测试无效难度
        let invalid_difficulty = BlockHeader {
            difficulty: U256::from(1000),
            ..valid_header.clone()
        };
        assert!(invalid_difficulty.validate_pos_header().is_err());

        // 测试无效nonce
        let invalid_nonce = BlockHeader {
            nonce: 12345,
            ..valid_header.clone()
        };
        assert!(invalid_nonce.validate_pos_header().is_err());

        // 测试无效ommers_hash
        let invalid_ommers = BlockHeader {
            ommers_hash: H256::random(),
            ..valid_header.clone()
        };
        assert!(invalid_ommers.validate_pos_header().is_err());
    }

    #[test]
    fn test_block_methods() {
        let header = BlockHeader {
            parent_hash: H256::zero(),
            ommers_hash: BlockHeader::empty_ommers_hash(),
            fee_recipient: Address::zero(),
            state_root: H256::zero(),
            transactions_root: H256::zero(),
            receipts_root: H256::zero(),
            logs_bloom: Bloom::zero(),
            difficulty: U256::zero(),
            number: U64::from(12345),
            gas_limit: U64::from(30_000_000),
            gas_used: U64::from(20_000_000),
            timestamp: U64::from(1234567890),
            extra_data: vec![],
            mix_hash: H256::zero(),
            nonce: 0,
            base_fee_per_gas: Some(U256::from(1_000_000_000u64)),
            withdrawals_root: None,
            blob_gas_used: None,
            excess_blob_gas: None,
            parent_beacon_block_root: None,
        };

        let block = Block {
            header: header.clone(),
            transactions: vec![],
            withdrawals: vec![],
        };

        assert_eq!(block.number(), U64::from(12345));
        assert_eq!(block.gas_used(), U64::from(20_000_000));
        assert_eq!(block.gas_limit(), U64::from(30_000_000));
        assert_eq!(block.base_fee(), Some(U256::from(1_000_000_000u64)));
    }
}
