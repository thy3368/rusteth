//! Beacon Chain API 领域定义
//!
//! 本模块定义了以太坊 Beacon Chain API 的领域模型和端口接口。
//! 遵循 Clean Architecture 原则，所有实体都是纯领域对象，不依赖外部框架。
//!
//! 标准参考: https://github.com/ethereum/beacon-APIs
//! 文档: /Users/hongyaotang/src/rusteth/app/node/docs/beacon

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

// ================================================================================================
// 类型别名 (Type Aliases)
// ================================================================================================

/// Slot 编号（十进制字符串）
pub type Slot = String;

/// Epoch 编号（十进制字符串）
pub type Epoch = String;

/// 验证者索引（十进制字符串）
pub type ValidatorIndex = String;

/// 根哈希（32字节，十六进制）
pub type Root = String;

/// BLS 公钥（48字节，十六进制）
pub type BlsPublicKey = String;

/// BLS 签名（96字节，十六进制）
pub type BlsSignature = String;

/// Gwei 数量（十进制字符串）
pub type Gwei = String;

/// Unix 时间戳（秒）
pub type UnixTimestamp = String;

/// 版本号（4字节，十六进制）
pub type Version = String;

// ================================================================================================
// 标识符枚举 (ID Enums)
// ================================================================================================

/// 状态标识符
///
/// 支持的格式:
/// - `head`: 当前头部状态
/// - `genesis`: 创世状态
/// - `finalized`: 最终确定状态
/// - `justified`: 最新合理状态
/// - `<slot>`: 特定 slot
/// - `0x<state_root>`: 特定状态根
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum StateId {
    Head,
    Genesis,
    Finalized,
    Justified,
    Slot(Slot),
    Root(Root),
}

/// 区块标识符
///
/// 支持的格式:
/// - `head`: 当前头部区块
/// - `genesis`: 创世区块
/// - `finalized`: 最终确定区块
/// - `<slot>`: 特定 slot
/// - `0x<block_root>`: 特定区块根
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum BlockId {
    Head,
    Genesis,
    Finalized,
    Slot(Slot),
    Root(Root),
}

/// 验证者标识符
///
/// 支持的格式:
/// - `<pubkey>`: BLS 公钥
/// - `<index>`: 验证者索引
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ValidatorId {
    PublicKey(BlsPublicKey),
    Index(ValidatorIndex),
}

// ================================================================================================
// 领域实体 (Domain Entities) - 缓存行对齐以优化性能
// ================================================================================================

/// 创世信息
///
/// 包含信标链创世时的关键信息
#[repr(align(64))] // 缓存行对齐
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisInfo {
    /// 创世时间（Unix 时间戳）
    pub genesis_time: UnixTimestamp,
    /// 创世验证者根哈希
    pub genesis_validators_root: Root,
    /// 创世分叉版本
    pub genesis_fork_version: Version,
}

/// 分叉信息
#[repr(align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fork {
    /// 上一个分叉版本
    pub previous_version: Version,
    /// 当前分叉版本
    pub current_version: Version,
    /// 分叉 epoch
    pub epoch: Epoch,
}

/// 最终性检查点
#[repr(align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalityCheckpoints {
    /// 上一个合理检查点
    pub previous_justified: Checkpoint,
    /// 当前合理检查点
    pub current_justified: Checkpoint,
    /// 最终确定检查点
    pub finalized: Checkpoint,
}

/// 检查点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Epoch
    pub epoch: Epoch,
    /// 根哈希
    pub root: Root,
}

/// 验证者
#[repr(align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    /// BLS 公钥
    pub pubkey: BlsPublicKey,
    /// 提款凭证
    pub withdrawal_credentials: Root,
    /// 有效余额（Gwei）
    pub effective_balance: Gwei,
    /// 是否被削减
    pub slashed: bool,
    /// 激活资格 epoch
    pub activation_eligibility_epoch: Epoch,
    /// 激活 epoch
    pub activation_epoch: Epoch,
    /// 退出 epoch
    pub exit_epoch: Epoch,
    /// 可提款 epoch
    pub withdrawable_epoch: Epoch,
}

/// 验证者状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidatorStatus {
    /// 已存款，等待激活
    PendingInitialized,
    /// 激活队列中
    PendingQueued,
    /// 正在验证
    ActiveOngoing,
    /// 正在退出
    ActiveExiting,
    /// 被削减
    ActiveSlashed,
    /// 已退出（未被削减）
    ExitedUnslashed,
    /// 已退出（被削减）
    ExitedSlashed,
    /// 可提款
    WithdrawalPossible,
    /// 已提款
    WithdrawalDone,
}

/// 验证者信息（包含状态）
#[repr(align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    /// 验证者索引
    pub index: ValidatorIndex,
    /// 余额（Gwei）
    pub balance: Gwei,
    /// 状态
    pub status: ValidatorStatus,
    /// 验证者数据
    pub validator: Validator,
}

/// 信标区块头
#[repr(align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconBlockHeader {
    /// Slot
    pub slot: Slot,
    /// 提议者索引
    pub proposer_index: ValidatorIndex,
    /// 父区块根
    pub parent_root: Root,
    /// 状态根
    pub state_root: Root,
    /// 区块体根
    pub body_root: Root,
}

/// 签名的信标区块头
#[repr(align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedBeaconBlockHeader {
    /// 区块头消息
    pub message: BeaconBlockHeader,
    /// BLS 签名
    pub signature: BlsSignature,
}

/// 区块头响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeaderResponse {
    /// 区块根
    pub root: Root,
    /// 是否规范
    pub canonical: bool,
    /// 签名的区块头
    pub header: SignedBeaconBlockHeader,
}

/// 信标区块（简化版）
///
/// 注意：完整的信标区块结构非常复杂，这里提供简化版本
/// 实际实现需要根据不同分叉版本（Phase0, Altair, Bellatrix, Capella, Deneb）
/// 定义不同的区块结构
#[repr(align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconBlock {
    /// Slot
    pub slot: Slot,
    /// 提议者索引
    pub proposer_index: ValidatorIndex,
    /// 父区块根
    pub parent_root: Root,
    /// 状态根
    pub state_root: Root,
    /// 区块体
    pub body: BeaconBlockBody,
}

/// 信标区块体（简化版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconBlockBody {
    /// RANDAO 显示
    pub randao_reveal: BlsSignature,
    /// ETH1 数据
    pub eth1_data: Eth1Data,
    /// Graffiti（32字节）
    pub graffiti: String,
    /// 提议者削减
    pub proposer_slashings: Vec<ProposerSlashing>,
    /// 证明者削减
    pub attester_slashings: Vec<AttesterSlashing>,
    /// 证明
    pub attestations: Vec<Attestation>,
    /// 存款
    pub deposits: Vec<Deposit>,
    /// 自愿退出
    pub voluntary_exits: Vec<SignedVoluntaryExit>,
}

/// 签名的信标区块
#[repr(align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedBeaconBlock {
    /// 区块消息
    pub message: BeaconBlock,
    /// BLS 签名
    pub signature: BlsSignature,
}

/// ETH1 数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Eth1Data {
    /// 存款根
    pub deposit_root: Root,
    /// 存款数量
    pub deposit_count: String,
    /// 区块哈希
    pub block_hash: Root,
}

/// 证明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    /// 聚合位
    pub aggregation_bits: String, // 位向量，十六进制
    /// 证明数据
    pub data: AttestationData,
    /// 签名
    pub signature: BlsSignature,
}

/// 证明数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationData {
    /// Slot
    pub slot: Slot,
    /// 委员会索引
    pub index: String,
    /// 信标区块根
    pub beacon_block_root: Root,
    /// 源检查点
    pub source: Checkpoint,
    /// 目标检查点
    pub target: Checkpoint,
}

/// 提议者削减
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposerSlashing {
    /// 签名的区块头 1
    pub signed_header_1: SignedBeaconBlockHeader,
    /// 签名的区块头 2
    pub signed_header_2: SignedBeaconBlockHeader,
}

/// 证明者削减
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttesterSlashing {
    /// 索引证明 1
    pub attestation_1: IndexedAttestation,
    /// 索引证明 2
    pub attestation_2: IndexedAttestation,
}

/// 索引证明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedAttestation {
    /// 证明索引
    pub attesting_indices: Vec<ValidatorIndex>,
    /// 证明数据
    pub data: AttestationData,
    /// 签名
    pub signature: BlsSignature,
}

/// 存款
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deposit {
    /// Merkle 证明
    pub proof: Vec<Root>,
    /// 存款数据
    pub data: DepositData,
}

/// 存款数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositData {
    /// 公钥
    pub pubkey: BlsPublicKey,
    /// 提款凭证
    pub withdrawal_credentials: Root,
    /// 金额（Gwei）
    pub amount: Gwei,
    /// 签名
    pub signature: BlsSignature,
}

/// 自愿退出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoluntaryExit {
    /// Epoch
    pub epoch: Epoch,
    /// 验证者索引
    pub validator_index: ValidatorIndex,
}

/// 签名的自愿退出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedVoluntaryExit {
    /// 退出消息
    pub message: VoluntaryExit,
    /// 签名
    pub signature: BlsSignature,
}

/// 验证者余额
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorBalance {
    /// 验证者索引
    pub index: ValidatorIndex,
    /// 余额（Gwei）
    pub balance: Gwei,
}

/// 委员会信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Committee {
    /// Slot
    pub slot: Slot,
    /// 委员会索引
    pub index: String,
    /// 验证者索引列表
    pub validators: Vec<ValidatorIndex>,
}

/// 同步委员会
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCommittee {
    /// 验证者索引列表
    pub validators: Vec<ValidatorIndex>,
    /// 聚合公钥列表
    pub validator_aggregates: Vec<Vec<ValidatorIndex>>,
}

/// 链规范参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainSpec {
    /// 每个 epoch 的 slot 数
    #[serde(rename = "SLOTS_PER_EPOCH")]
    pub slots_per_epoch: String,
    /// 每个 slot 的秒数
    #[serde(rename = "SECONDS_PER_SLOT")]
    pub seconds_per_slot: String,
    /// 存款合约地址
    #[serde(rename = "DEPOSIT_CONTRACT_ADDRESS")]
    pub deposit_contract_address: String,
    /// 最小创世时间（可选，某些节点可能不返回）
    #[serde(rename = "MIN_GENESIS_TIME", skip_serializing_if = "Option::is_none")]
    pub min_genesis_time: Option<UnixTimestamp>,
    /// 配置名称（如 "mainnet"）
    #[serde(rename = "CONFIG_NAME", skip_serializing_if = "Option::is_none")]
    pub config_name: Option<String>,
    /// 预设基础（如 "mainnet"）
    #[serde(rename = "PRESET_BASE", skip_serializing_if = "Option::is_none")]
    pub preset_base: Option<String>,
    /// 其他参数使用 flatten 处理
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// 分叉时间表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkSchedule {
    /// 分叉列表
    pub forks: Vec<ForkInfo>,
}

/// 分叉信息（用于时间表）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkInfo {
    /// 分叉版本
    pub version: Version,
    /// 分叉 epoch
    pub epoch: Epoch,
    /// 分叉名称（如 "phase0", "altair", "bellatrix"）
    pub name: String,
}

/// 节点版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeVersion {
    /// 客户端版本字符串
    pub version: String,
}

/// 节点健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// 节点正常（HTTP 200）
    Healthy,
    /// 节点正在同步（HTTP 206）
    Syncing,
    /// 节点不健康（HTTP 503）
    Unhealthy,
}

/// 同步状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncingStatus {
    /// 头部 slot
    pub head_slot: Slot,
    /// 同步距离
    pub sync_distance: String,
    /// 是否正在同步
    pub is_syncing: bool,
    /// 是否乐观
    pub is_optimistic: bool,
    /// 执行层是否乐观
    pub el_offline: bool,
}

/// 节点身份信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIdentity {
    /// 节点 ID（libp2p peer_id）
    pub peer_id: String,
    /// ENR（Ethereum Node Record）
    pub enr: String,
    /// P2P 地址列表
    pub p2p_addresses: Vec<String>,
    /// Discovery 地址列表
    pub discovery_addresses: Vec<String>,
    /// 元数据
    pub metadata: NodeMetadata,
}

/// 节点元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// 序列号
    pub seq_number: String,
    /// 支持的 attnets 位向量
    pub attnets: String,
    /// 支持的 syncnets 位向量
    pub syncnets: String,
}

// ================================================================================================
// 错误类型 (Error Types)
// ================================================================================================

/// Repository 错误
#[derive(Debug, Clone)]
pub enum RepositoryError {
    /// 资源未找到
    NotFound(String),
    /// 无效参数
    InvalidParameter(String),
    /// 内部错误
    Internal(String),
    /// 节点未同步
    NotSynced,
    /// 节点不健康
    Unhealthy,
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
            Self::NotSynced => write!(f, "Node is not synced"),
            Self::Unhealthy => write!(f, "Node is unhealthy"),
        }
    }
}

impl std::error::Error for RepositoryError {}

// ================================================================================================
// Repository Trait（端口接口）- 遵循 Clean Architecture
// ================================================================================================

/// Beacon API Repository Trait
///
/// 定义信标链数据访问的抽象接口。
/// 所有具体实现（如基于 HTTP 的客户端、Mock 实现）都应实现此 trait。
///
/// 此 trait 遵循 Clean Architecture 的依赖倒置原则：
/// - 用例层依赖此抽象接口
/// - 基础设施层提供具体实现
#[async_trait]
pub trait BeaconApi: Send + Sync {
    // ============================================================================================
    // 1. 基础信息查询 (Basic Information)
    // ============================================================================================

    /// 获取创世信息
    ///
    /// 端点: GET /eth/v1/beacon/genesis
    async fn get_genesis(&self) -> Result<GenesisInfo, RepositoryError>;

    /// 获取节点版本
    ///
    /// 端点: GET /eth/v1/node/version
    async fn get_node_version(&self) -> Result<NodeVersion, RepositoryError>;

    /// 获取节点健康状态
    ///
    /// 端点: GET /eth/v1/node/health
    async fn get_node_health(&self) -> Result<HealthStatus, RepositoryError>;

    /// 获取同步状态
    ///
    /// 端点: GET /eth/v1/node/syncing
    async fn get_syncing_status(&self) -> Result<SyncingStatus, RepositoryError>;

    /// 获取节点身份信息
    ///
    /// 端点: GET /eth/v1/node/identity
    async fn get_node_identity(&self) -> Result<NodeIdentity, RepositoryError>;

    // ============================================================================================
    // 2. 配置查询 (Configuration)
    // ============================================================================================

    /// 获取链规范参数
    ///
    /// 端点: GET /eth/v1/config/spec
    async fn get_spec(&self) -> Result<ChainSpec, RepositoryError>;

    /// 获取分叉时间表
    ///
    /// 端点: GET /eth/v1/config/fork_schedule
    async fn get_fork_schedule(&self) -> Result<ForkSchedule, RepositoryError>;

    // ============================================================================================
    // 3. 区块头查询 (Block Headers)
    // ============================================================================================

    /// 获取当前链头区块头
    ///
    /// 端点: GET /eth/v1/beacon/headers/head
    async fn get_block_header(&self, block_id: BlockId) -> Result<BlockHeaderResponse, RepositoryError>;

    /// 获取区块头列表
    ///
    /// 端点: GET /eth/v1/beacon/headers
    async fn get_block_headers(
        &self,
        slot: Option<Slot>,
        parent_root: Option<Root>,
    ) -> Result<Vec<BlockHeaderResponse>, RepositoryError>;

    // ============================================================================================
    // 4. 区块查询 (Blocks)
    // ============================================================================================

    /// 获取信标区块
    ///
    /// 端点: GET /eth/v2/beacon/blocks/{block_id}
    async fn get_block(&self, block_id: BlockId) -> Result<SignedBeaconBlock, RepositoryError>;

    /// 获取区块根哈希
    ///
    /// 端点: GET /eth/v1/beacon/blocks/{block_id}/root
    async fn get_block_root(&self, block_id: BlockId) -> Result<Root, RepositoryError>;

    /// 获取区块中的证明
    ///
    /// 端点: GET /eth/v1/beacon/blocks/{block_id}/attestations
    async fn get_block_attestations(&self, block_id: BlockId) -> Result<Vec<Attestation>, RepositoryError>;

    /// 发布信标区块
    ///
    /// 端点: POST /eth/v1/beacon/blocks
    async fn publish_block(&self, block: SignedBeaconBlock) -> Result<(), RepositoryError>;

    // ============================================================================================
    // 5. 状态查询 (States)
    // ============================================================================================

    /// 获取状态根哈希
    ///
    /// 端点: GET /eth/v1/beacon/states/{state_id}/root
    async fn get_state_root(&self, state_id: StateId) -> Result<Root, RepositoryError>;

    /// 获取分叉信息
    ///
    /// 端点: GET /eth/v1/beacon/states/{state_id}/fork
    async fn get_state_fork(&self, state_id: StateId) -> Result<Fork, RepositoryError>;

    /// 获取最终性检查点
    ///
    /// 端点: GET /eth/v1/beacon/states/{state_id}/finality_checkpoints
    async fn get_finality_checkpoints(&self, state_id: StateId) -> Result<FinalityCheckpoints, RepositoryError>;

    // ============================================================================================
    // 6. 验证者查询 (Validators)
    // ============================================================================================

    /// 获取验证者列表
    ///
    /// 端点: GET /eth/v1/beacon/states/{state_id}/validators
    async fn get_validators(
        &self,
        state_id: StateId,
        ids: Option<Vec<ValidatorId>>,
        statuses: Option<Vec<ValidatorStatus>>,
    ) -> Result<Vec<ValidatorInfo>, RepositoryError>;

    /// 获取单个验证者信息
    ///
    /// 端点: GET /eth/v1/beacon/states/{state_id}/validators/{validator_id}
    async fn get_validator(
        &self,
        state_id: StateId,
        validator_id: ValidatorId,
    ) -> Result<ValidatorInfo, RepositoryError>;

    /// 获取验证者余额列表
    ///
    /// 端点: GET /eth/v1/beacon/states/{state_id}/validator_balances
    async fn get_validator_balances(
        &self,
        state_id: StateId,
        ids: Option<Vec<ValidatorId>>,
    ) -> Result<Vec<ValidatorBalance>, RepositoryError>;

    /// 批量查询验证者（POST）
    ///
    /// 端点: POST /eth/v1/beacon/states/{state_id}/validators
    async fn post_validators(
        &self,
        state_id: StateId,
        ids: Vec<ValidatorId>,
        statuses: Option<Vec<ValidatorStatus>>,
    ) -> Result<Vec<ValidatorInfo>, RepositoryError>;

    // ============================================================================================
    // 7. 委员会查询 (Committees)
    // ============================================================================================

    /// 获取委员会信息
    ///
    /// 端点: GET /eth/v1/beacon/states/{state_id}/committees
    async fn get_committees(
        &self,
        state_id: StateId,
        epoch: Option<Epoch>,
        index: Option<String>,
        slot: Option<Slot>,
    ) -> Result<Vec<Committee>, RepositoryError>;

    /// 获取同步委员会
    ///
    /// 端点: GET /eth/v1/beacon/states/{state_id}/sync_committees
    async fn get_sync_committees(
        &self,
        state_id: StateId,
        epoch: Option<Epoch>,
    ) -> Result<SyncCommittee, RepositoryError>;

    // ============================================================================================
    // 8. 交易池查询 (Pool)
    // ============================================================================================

    /// 获取待处理证明
    ///
    /// 端点: GET /eth/v1/beacon/pool/attestations
    async fn get_pool_attestations(
        &self,
        slot: Option<Slot>,
        committee_index: Option<String>,
    ) -> Result<Vec<Attestation>, RepositoryError>;

    /// 提交证明
    ///
    /// 端点: POST /eth/v1/beacon/pool/attestations
    async fn submit_pool_attestations(&self, attestations: Vec<Attestation>) -> Result<(), RepositoryError>;

    /// 获取自愿退出
    ///
    /// 端点: GET /eth/v1/beacon/pool/voluntary_exits
    async fn get_pool_voluntary_exits(&self) -> Result<Vec<SignedVoluntaryExit>, RepositoryError>;

    /// 提交自愿退出
    ///
    /// 端点: POST /eth/v1/beacon/pool/voluntary_exits
    async fn submit_pool_voluntary_exit(&self, exit: SignedVoluntaryExit) -> Result<(), RepositoryError>;
}

// ================================================================================================
// 辅助函数和实用工具
// ================================================================================================

impl StateId {
    /// 创建 head 状态 ID
    pub fn head() -> Self {
        Self::Head
    }

    /// 创建 finalized 状态 ID
    pub fn finalized() -> Self {
        Self::Finalized
    }

    /// 创建 genesis 状态 ID
    pub fn genesis() -> Self {
        Self::Genesis
    }

    /// 从 slot 创建状态 ID
    pub fn from_slot(slot: Slot) -> Self {
        Self::Slot(slot)
    }

    /// 从根哈希创建状态 ID
    pub fn from_root(root: Root) -> Self {
        Self::Root(root)
    }
}

impl BlockId {
    /// 创建 head 区块 ID
    pub fn head() -> Self {
        Self::Head
    }

    /// 创建 finalized 区块 ID
    pub fn finalized() -> Self {
        Self::Finalized
    }

    /// 创建 genesis 区块 ID
    pub fn genesis() -> Self {
        Self::Genesis
    }

    /// 从 slot 创建区块 ID
    pub fn from_slot(slot: Slot) -> Self {
        Self::Slot(slot)
    }

    /// 从根哈希创建区块 ID
    pub fn from_root(root: Root) -> Self {
        Self::Root(root)
    }
}

impl ValidatorId {
    /// 从公钥创建验证者 ID
    pub fn from_pubkey(pubkey: BlsPublicKey) -> Self {
        Self::PublicKey(pubkey)
    }

    /// 从索引创建验证者 ID
    pub fn from_index(index: ValidatorIndex) -> Self {
        Self::Index(index)
    }
}

// ================================================================================================
// 测试模块
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_id_creation() {
        let head = StateId::head();
        assert!(matches!(head, StateId::Head));

        let finalized = StateId::finalized();
        assert!(matches!(finalized, StateId::Finalized));

        let slot = StateId::from_slot("12345".to_string());
        assert!(matches!(slot, StateId::Slot(_)));
    }

    #[test]
    fn test_block_id_creation() {
        let head = BlockId::head();
        assert!(matches!(head, BlockId::Head));

        let genesis = BlockId::genesis();
        assert!(matches!(genesis, BlockId::Genesis));
    }

    #[test]
    fn test_validator_id_creation() {
        let by_index = ValidatorId::from_index("12345".to_string());
        assert!(matches!(by_index, ValidatorId::Index(_)));

        let by_pubkey = ValidatorId::from_pubkey("0x1234".to_string());
        assert!(matches!(by_pubkey, ValidatorId::PublicKey(_)));
    }

    #[test]
    fn test_validator_status_serialization() {
        let status = ValidatorStatus::ActiveOngoing;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"active_ongoing\"");
    }
}