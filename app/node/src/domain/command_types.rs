//! 领域核心类型定义
//!
//! 符合 EIP-1474 和 EIP-1559 规范的以太坊数据结构

use ethereum_types::{Address, Bloom, H256, H64, U256, U64};
use serde::{Deserialize, Serialize};

// ============================================================================
// 核心以太坊类型
// ============================================================================

/// 区块标识符 - 可以是区块号、"latest"、"earliest"、"pending"
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockId {
    Number(U64),
    Tag(BlockTag),
}

/// 区块标签枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlockTag {
    Latest,   // 最新区块
    Earliest, // 创世区块
    Pending,  // 待处理区块
}

/// 以太坊区块结构（符合 EIP-1474，缓存行对齐优化性能）
#[repr(align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub number: U64,             // 区块号
    pub hash: H256,              // 区块哈希
    pub parent_hash: H256,       // 父区块哈希
    pub nonce: H64,              // 工作量证明随机数
    pub sha3_uncles: H256,       // 叔块哈希
    pub logs_bloom: Bloom,       // 日志布隆过滤器
    pub transactions_root: H256, // 交易树根
    pub state_root: H256,        // 状态树根
    pub receipts_root: H256,     // 收据树根
    pub miner: Address,          // 矿工地址
    pub difficulty: U256,        // 难度
    pub total_difficulty: U256,  // 总难度
    #[serde(with = "hex_bytes")]
    pub extra_data: Vec<u8>, // 额外数据（十六进制字符串）
    pub size: U256,              // 区块大小
    pub gas_limit: U256,         // Gas 限制
    pub gas_used: U256,          // 已使用 Gas
    pub timestamp: U256,         // 时间戳
    pub transactions: Vec<Transaction>, // 交易列表
    pub uncles: Vec<H256>,       // 叔块哈希列表
}

/// 以太坊交易结构（符合 EIP-1474 和 EIP-1559，缓存行对齐）
#[repr(align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub hash: H256,                     // 交易哈希
    pub nonce: U256,                    // 发送方交易序号
    pub block_hash: Option<H256>,       // 所属区块哈希
    pub block_number: Option<U64>,      // 所属区块号
    pub transaction_index: Option<U64>, // 区块中的交易索引
    pub from: Address,                  // 发送方地址
    pub to: Option<Address>,            // 接收方地址（合约创建时为 None）
    pub value: U256,                    // 转账金额（wei）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<U256>, // Gas 价格（Legacy 交易使用）
    pub gas: U256,                      // Gas 限制
    #[serde(with = "hex_bytes")]
    pub input: Vec<u8>, // 输入数据（十六进制字符串）
    pub v: U64,                         // 签名 v 值
    pub r: U256,                        // 签名 r 值
    pub s: U256,                        // 签名 s 值
    // EIP-1559 字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<U256>, // EIP-1559: 每 gas 最大费用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<U256>, // EIP-1559: 每 gas 最大优先费用
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub transaction_type: Option<U64>, // 交易类型（0=Legacy, 2=EIP-1559）
}

/// 交易收据结构（符合 EIP-1474）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    pub transaction_hash: H256,            // 交易哈希
    pub transaction_index: U64,            // 交易索引
    pub block_hash: H256,                  // 区块哈希
    pub block_number: U64,                 // 区块号
    pub from: Address,                     // 发送方地址
    pub to: Option<Address>,               // 接收方地址
    pub cumulative_gas_used: U256,         // 累计使用的 Gas
    pub gas_used: U256,                    // 本交易使用的 Gas
    pub contract_address: Option<Address>, // 合约地址（如果是合约创建）
    pub logs: Vec<Log>,                    // 日志列表
    pub logs_bloom: Bloom,                 // 日志布隆过滤器
    pub status: U64,                       // 交易状态（1=成功，0=失败）
}

/// 事件日志结构（符合 EIP-1474）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    pub removed: bool,           // 是否因链重组被移除
    pub log_index: U256,         // 日志索引
    pub transaction_index: U256, // 交易索引
    pub transaction_hash: H256,  // 交易哈希
    pub block_hash: H256,        // 区块哈希
    pub block_number: U64,       // 区块号
    pub address: Address,        // 合约地址
    #[serde(with = "hex_bytes")]
    pub data: Vec<u8>, // 日志数据（十六进制字符串）
    pub topics: Vec<H256>,       // 日志主题
}

/// 调用/交易参数（符合 EIP-1474 和 EIP-1559）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallRequest {
    pub from: Option<Address>,   // 发送方地址（可选）
    pub to: Option<Address>,     // 目标地址（合约创建时为None）
    pub gas: Option<U256>,       // Gas 限制（可选）
    pub gas_price: Option<U256>, // Gas 价格（Legacy，可选）
    pub value: Option<U256>,     // 转账金额（可选）
    #[serde(default, with = "hex_data")]
    pub data: Option<Vec<u8>>, // 调用数据（十六进制字符串，可选）
    // EIP-1559 字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<U256>, // EIP-1559: 每 gas 最大费用（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<U256>, // EIP-1559: 每 gas 最大优先费用（可选）
}

/// 日志过滤器参数（符合 EIP-1474）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterOptions {
    pub from_block: Option<BlockId>,       // 起始区块
    pub to_block: Option<BlockId>,         // 结束区块
    pub address: Option<Address>,          // 合约地址过滤
    pub topics: Option<Vec<Option<H256>>>, // 主题过滤
}

/// EIP-1559 费用历史结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeHistory {
    pub oldest_block: U64,           // 最旧区块号
    pub base_fee_per_gas: Vec<U256>, // 每个区块的基础费用
    pub gas_used_ratio: Vec<f64>,    // 每个区块的 gas 使用比率
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reward: Option<Vec<Vec<U256>>>, // 可选：每个区块的奖励百分位数
}

/// 发送交易请求（用于 eth_sendTransaction）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendTransactionRequest {
    pub from: Address,           // 发送方地址
    pub to: Option<Address>,     // 接收方地址（合约创建时为 None）
    pub gas: Option<U256>,       // Gas 限制（可选）
    pub gas_price: Option<U256>, // Gas 价格（Legacy，可选）
    pub value: Option<U256>,     // 转账金额（可选）
    #[serde(default, with = "hex_data")]
    pub data: Option<Vec<u8>>, // 交易数据（可选）
    pub nonce: Option<U256>,     // Nonce（可选）
    // EIP-1559 字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<U256>, // EIP-1559: 最大费用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<U256>, // EIP-1559: 最大优先费用
}

// ============================================================================
// 序列化辅助模块
// ============================================================================

/// 自定义序列化模块：处理十六进制字符串和可选字节数组的转换
mod hex_data {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(data: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match data {
            Some(bytes) => {
                let hex_string = format!("0x{}", hex::encode(bytes));
                serializer.serialize_str(&hex_string)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) => {
                let s = s.trim_start_matches("0x");
                if s.is_empty() {
                    Ok(Some(vec![]))
                } else {
                    hex::decode(s).map(Some).map_err(serde::de::Error::custom)
                }
            }
            None => Ok(None),
        }
    }
}

/// 自定义序列化模块：处理十六进制字符串和必需字节数组的转换
mod hex_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_string = format!("0x{}", hex::encode(data));
        serializer.serialize_str(&hex_string)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        let s = s.trim_start_matches("0x");
        if s.is_empty() {
            Ok(vec![])
        } else {
            hex::decode(s).map_err(serde::de::Error::custom)
        }
    }
}
