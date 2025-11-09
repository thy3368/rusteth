//! Beacon API Mock 服务器示例
//!
//! 此示例演示如何使用 BeaconApiServer 和 Mock 实现创建一个独立的测试服务器。
//! 适合用于本地开发、测试和演示。
//!
//! # 使用方法
//!
//! ```bash
//! # 运行 Mock 服务器
//! cargo run --example beacon_server_mock
//!
//! # 在另一个终端测试
//! curl http://127.0.0.1:8080/eth/v1/beacon/genesis
//! curl http://127.0.0.1:8080/eth/v1/node/version
//! curl http://127.0.0.1:8080/eth/v1/node/health
//! ```

use lib::domain::service::beacon_api::*;
use async_trait::async_trait;
use std::sync::Arc;

// ================================================================================================
// Mock BeaconApi 实现 - 用于演示和测试
// ================================================================================================

/// Mock Beacon API 实现
///
/// 返回预定义的测试数据，不依赖真实的 Beacon Node
#[derive(Debug)]
pub struct MockBeaconApi {
    /// 节点版本
    version: String,
    /// 创世时间
    genesis_time: String,
}

impl MockBeaconApi {
    /// 创建新的 Mock 实现
    pub fn new() -> Self {
        Self {
            version: "MockBeacon/v1.0.0/rust".to_string(),
            genesis_time: "1606824023".to_string(), // 以太坊主网创世时间
        }
    }
}

#[async_trait]
impl BeaconApi for MockBeaconApi {
    // ============================================================================================
    // 1. 基础信息查询
    // ============================================================================================

    async fn get_genesis(&self) -> Result<GenesisInfo, ServiceError> {
        Ok(GenesisInfo {
            genesis_time: self.genesis_time.clone(),
            genesis_validators_root: "0x4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95".to_string(),
            genesis_fork_version: "0x00000000".to_string(),
        })
    }

    async fn get_node_version(&self) -> Result<NodeVersion, ServiceError> {
        Ok(NodeVersion {
            version: self.version.clone(),
        })
    }

    async fn get_node_health(&self) -> Result<HealthStatus, ServiceError> {
        Ok(HealthStatus::Healthy)
    }

    async fn get_syncing_status(&self) -> Result<SyncingStatus, ServiceError> {
        Ok(SyncingStatus {
            head_slot: "1000000".to_string(),
            sync_distance: "0".to_string(),
            is_syncing: false,
            is_optimistic: false,
            el_offline: false,
        })
    }

    async fn get_node_identity(&self) -> Result<NodeIdentity, ServiceError> {
        Ok(NodeIdentity {
            peer_id: "16Uiu2HAm1234567890abcdef".to_string(),
            enr: "enr:-Ku4QMock...".to_string(),
            p2p_addresses: vec!["/ip4/127.0.0.1/tcp/9000".to_string()],
            discovery_addresses: vec!["/ip4/127.0.0.1/udp/9000".to_string()],
            metadata: NodeMetadata {
                seq_number: "1".to_string(),
                attnets: "0x0000000000000000".to_string(),
                syncnets: "0x00".to_string(),
            },
        })
    }

    // ============================================================================================
    // 2. 配置查询
    // ============================================================================================

    async fn get_spec(&self) -> Result<ChainSpec, ServiceError> {
        Ok(ChainSpec {
            slots_per_epoch: "32".to_string(),
            seconds_per_slot: "12".to_string(),
            deposit_contract_address: "0x00000000219ab540356cBB839Cbe05303d7705Fa".to_string(),
            min_genesis_time: Some(self.genesis_time.clone()),
            config_name: Some("mainnet".to_string()),
            preset_base: Some("mainnet".to_string()),
            extra: std::collections::HashMap::new(),
        })
    }

    async fn get_fork_schedule(&self) -> Result<ForkSchedule, ServiceError> {
        Ok(ForkSchedule {
            forks: vec![
                ForkInfo {
                    version: "0x00000000".to_string(),
                    epoch: "0".to_string(),
                    name: "phase0".to_string(),
                },
                ForkInfo {
                    version: "0x01000000".to_string(),
                    epoch: "74240".to_string(),
                    name: "altair".to_string(),
                },
                ForkInfo {
                    version: "0x02000000".to_string(),
                    epoch: "144896".to_string(),
                    name: "bellatrix".to_string(),
                },
            ],
        })
    }

    // ============================================================================================
    // 3. 区块头查询
    // ============================================================================================

    async fn get_block_header(&self, _block_id: BlockId) -> Result<BlockHeaderResponse, ServiceError> {
        Ok(BlockHeaderResponse {
            root: "0xabcd1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab".to_string(),
            canonical: true,
            header: SignedBeaconBlockHeader {
                message: BeaconBlockHeader {
                    slot: "1000000".to_string(),
                    proposer_index: "12345".to_string(),
                    parent_root: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
                    state_root: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
                    body_root: "0x567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234".to_string(),
                },
                signature: "0x1234567890abcdef".to_string(),
            },
        })
    }

    async fn get_block_headers(
        &self,
        _slot: Option<Slot>,
        _parent_root: Option<Root>,
    ) -> Result<Vec<BlockHeaderResponse>, ServiceError> {
        Ok(vec![])
    }

    // ============================================================================================
    // 4. 区块查询
    // ============================================================================================

    async fn get_block(&self, _block_id: BlockId) -> Result<SignedBeaconBlock, ServiceError> {
        // 返回简化的模拟区块
        Err(ServiceError::NotFound("Mock implementation does not provide full block data".to_string()))
    }

    async fn get_block_root(&self, _block_id: BlockId) -> Result<Root, ServiceError> {
        Ok("0xabcd1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab".to_string())
    }

    async fn get_block_attestations(&self, _block_id: BlockId) -> Result<Vec<Attestation>, ServiceError> {
        Ok(vec![])
    }

    async fn publish_block(&self, _block: SignedBeaconBlock) -> Result<(), ServiceError> {
        Ok(())
    }

    // ============================================================================================
    // 5. 状态查询
    // ============================================================================================

    async fn get_state_root(&self, _state_id: StateId) -> Result<Root, ServiceError> {
        Ok("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string())
    }

    async fn get_state_fork(&self, _state_id: StateId) -> Result<Fork, ServiceError> {
        Ok(Fork {
            previous_version: "0x01000000".to_string(),
            current_version: "0x02000000".to_string(),
            epoch: "144896".to_string(),
        })
    }

    async fn get_finality_checkpoints(&self, _state_id: StateId) -> Result<FinalityCheckpoints, ServiceError> {
        Ok(FinalityCheckpoints {
            previous_justified: Checkpoint {
                epoch: "31249".to_string(),
                root: "0xabcd1234".to_string(),
            },
            current_justified: Checkpoint {
                epoch: "31250".to_string(),
                root: "0x1234abcd".to_string(),
            },
            finalized: Checkpoint {
                epoch: "31248".to_string(),
                root: "0x5678efab".to_string(),
            },
        })
    }

    // ============================================================================================
    // 6. 验证者查询
    // ============================================================================================

    async fn get_validators(
        &self,
        _state_id: StateId,
        _ids: Option<Vec<ValidatorId>>,
        _statuses: Option<Vec<ValidatorStatus>>,
    ) -> Result<Vec<ValidatorInfo>, ServiceError> {
        Ok(vec![])
    }

    async fn get_validator(
        &self,
        _state_id: StateId,
        _validator_id: ValidatorId,
    ) -> Result<ValidatorInfo, ServiceError> {
        Err(ServiceError::NotFound("Validator not found".to_string()))
    }

    async fn get_validator_balances(
        &self,
        _state_id: StateId,
        _ids: Option<Vec<ValidatorId>>,
    ) -> Result<Vec<ValidatorBalance>, ServiceError> {
        Ok(vec![])
    }

    async fn post_validators(
        &self,
        _state_id: StateId,
        _ids: Vec<ValidatorId>,
        _statuses: Option<Vec<ValidatorStatus>>,
    ) -> Result<Vec<ValidatorInfo>, ServiceError> {
        Ok(vec![])
    }

    // ============================================================================================
    // 7. 委员会查询
    // ============================================================================================

    async fn get_committees(
        &self,
        _state_id: StateId,
        _epoch: Option<Epoch>,
        _index: Option<String>,
        _slot: Option<Slot>,
    ) -> Result<Vec<Committee>, ServiceError> {
        Ok(vec![])
    }

    async fn get_sync_committees(
        &self,
        _state_id: StateId,
        _epoch: Option<Epoch>,
    ) -> Result<SyncCommittee, ServiceError> {
        Ok(SyncCommittee {
            validators: vec![],
            validator_aggregates: vec![],
        })
    }

    // ============================================================================================
    // 8. 交易池查询
    // ============================================================================================

    async fn get_pool_attestations(
        &self,
        _slot: Option<Slot>,
        _committee_index: Option<String>,
    ) -> Result<Vec<Attestation>, ServiceError> {
        Ok(vec![])
    }

    async fn submit_pool_attestations(&self, _attestations: Vec<Attestation>) -> Result<(), ServiceError> {
        Ok(())
    }

    async fn get_pool_voluntary_exits(&self) -> Result<Vec<SignedVoluntaryExit>, ServiceError> {
        Ok(vec![])
    }

    async fn submit_pool_voluntary_exit(&self, _exit: SignedVoluntaryExit) -> Result<(), ServiceError> {
        Ok(())
    }
}

// ================================================================================================
// 主函数 - 启动服务器
// ================================================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // 从环境变量读取配置
    let server_addr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    tracing::info!("Starting Beacon API Mock Server");

    // 创建 Mock BeaconApi 实现
    let mock_beacon_api = Arc::new(MockBeaconApi::new());

    // 创建 BeaconApiServer
    let server = lib::adapter::inbound::beacon_api_server::BeaconApiServer::new(mock_beacon_api);

    // 构建 Axum Router
    let app = server.router();

    // 绑定 TCP 监听器
    let listener = tokio::net::TcpListener::bind(&server_addr).await?;

    tracing::info!("🚀 Beacon API Mock Server listening on http://{}", server_addr);
    tracing::info!("");
    tracing::info!("📋 Available endpoints:");
    tracing::info!("  基础信息:");
    tracing::info!("    GET  /eth/v1/beacon/genesis");
    tracing::info!("    GET  /eth/v1/node/version");
    tracing::info!("    GET  /eth/v1/node/health");
    tracing::info!("    GET  /eth/v1/node/syncing");
    tracing::info!("    GET  /eth/v1/node/identity");
    tracing::info!("");
    tracing::info!("  配置查询:");
    tracing::info!("    GET  /eth/v1/config/spec");
    tracing::info!("    GET  /eth/v1/config/fork_schedule");
    tracing::info!("");
    tracing::info!("  区块查询:");
    tracing::info!("    GET  /eth/v1/beacon/headers/head");
    tracing::info!("    GET  /eth/v1/beacon/blocks/head/root");
    tracing::info!("");
    tracing::info!("  状态查询:");
    tracing::info!("    GET  /eth/v1/beacon/states/head/root");
    tracing::info!("    GET  /eth/v1/beacon/states/head/fork");
    tracing::info!("    GET  /eth/v1/beacon/states/head/finality_checkpoints");
    tracing::info!("");
    tracing::info!("💡 测试命令:");
    tracing::info!("  curl http://{}/eth/v1/beacon/genesis", server_addr);
    tracing::info!("  curl http://{}/eth/v1/node/version", server_addr);
    tracing::info!("");

    // 启动 HTTP 服务器
    axum::serve(listener, app).await?;

    Ok(())
}
