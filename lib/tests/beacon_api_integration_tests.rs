//! Beacon API 集成测试
//!
//! 这些测试使用真实的以太坊主网 Beacon API 端点进行测试。
//!
//! # 运行测试
//!
//! 默认情况下这些测试被忽略（因为依赖外部服务）。要运行这些测试：
//! ```bash
//! cargo test --test beacon_api_integration_tests -- --ignored
//! ```
//!
//! # 主网 Beacon API 端点
//!
//! 我们使用公开的主网端点：
//! - https://www.lightclientdata.org (Lodestar)
//! - https://beaconstate.ethstaker.cc (Nimbus)

use lib::adapter::outbound::beacon_api_client::BeaconApiClient;
use lib::domain::service::beacon_api::{
    BeaconApi, BlockId, HealthStatus, ServiceError, StateId, ValidatorId, ValidatorStatus,
};

/// 主网公开 Beacon API 端点
/// 使用 ethstaker.cc 提供的公开端点（稳定性好）
const MAINNET_BEACON_API: &str = "https://beaconstate.ethstaker.cc";

/// 备用端点（如果主端点不可用）
const FALLBACK_BEACON_API: &str = "https://beaconstate.info";

/// 辅助函数：创建客户端
fn create_client() -> BeaconApiClient {
    BeaconApiClient::new(MAINNET_BEACON_API)
        .expect("Failed to create BeaconApiClient")
}

/// 辅助函数：创建备用客户端
fn create_fallback_client() -> BeaconApiClient {
    BeaconApiClient::new(FALLBACK_BEACON_API)
        .expect("Failed to create fallback BeaconApiClient")
}

// ================================================================================================
// 1. 基础信息查询测试
// ================================================================================================

#[tokio::test]
#[ignore] // 依赖外部服务
async fn test_get_genesis_mainnet() {
    let client = create_client();

    let result = client.get_genesis().await;

    assert!(result.is_ok(), "Failed to get genesis: {:?}", result.err());

    let genesis = result.unwrap();

    // 验证主网创世信息
    println!("Genesis time: {}", genesis.genesis_time);
    println!("Genesis validators root: {}", genesis.genesis_validators_root);
    println!("Genesis fork version: {}", genesis.genesis_fork_version);

    // 主网创世时间应该是 2020-12-01 12:00:23 UTC
    assert_eq!(
        genesis.genesis_time, "1606824023",
        "Incorrect mainnet genesis time"
    );

    // 主网创世 fork version
    assert_eq!(
        genesis.genesis_fork_version, "0x00000000",
        "Incorrect mainnet genesis fork version"
    );
}

#[tokio::test]
#[ignore]
async fn test_get_node_version_mainnet() {
    let client = create_client();

    let result = client.get_node_version().await;

    assert!(result.is_ok(), "Failed to get node version: {:?}", result.err());

    let version = result.unwrap();
    println!("Node version: {}", version.version);

    // 版本字符串应该非空
    assert!(!version.version.is_empty(), "Version string is empty");
}

#[tokio::test]
#[ignore]
async fn test_get_node_health_mainnet() {
    let client = create_client();

    let result = client.get_node_health().await;

    // 公开端点可能不支持此端点，返回 404
    match result {
        Ok(health) => {
            println!("Node health status: {:?}", health);
            // 节点应该是健康的或正在同步
            assert!(
                matches!(health, HealthStatus::Healthy | HealthStatus::Syncing),
                "Node is unhealthy: {:?}",
                health
            );
        }
        Err(ServiceError::NotFound(msg)) => {
            println!("Health endpoint not supported: {}", msg);
            // 这是预期的，某些公开节点不提供此端点
        }
        Err(e) => {
            panic!("Unexpected error getting node health: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_get_syncing_status_mainnet() {
    let client = create_client();

    let result = client.get_syncing_status().await;

    assert!(result.is_ok(), "Failed to get syncing status: {:?}", result.err());

    let sync_status = result.unwrap();
    println!("Head slot: {}", sync_status.head_slot);
    println!("Sync distance: {}", sync_status.sync_distance);
    println!("Is syncing: {}", sync_status.is_syncing);
    println!("Is optimistic: {}", sync_status.is_optimistic);
    println!("EL offline: {}", sync_status.el_offline);

    // Head slot 应该是合理的数字
    let head_slot: u64 = sync_status.head_slot.parse().expect("Invalid head slot");
    assert!(head_slot > 0, "Head slot should be greater than 0");
}

#[tokio::test]
#[ignore]
async fn test_get_node_identity_mainnet() {
    let client = create_client();

    let result = client.get_node_identity().await;

    assert!(result.is_ok(), "Failed to get node identity: {:?}", result.err());

    let identity = result.unwrap();
    println!("Peer ID: {}", identity.peer_id);
    println!("ENR: {}", identity.enr);
    println!("P2P addresses: {:?}", identity.p2p_addresses);

    // Peer ID 应该非空
    assert!(!identity.peer_id.is_empty(), "Peer ID is empty");
    assert!(!identity.enr.is_empty(), "ENR is empty");
}

// ================================================================================================
// 2. 配置查询测试
// ================================================================================================

#[tokio::test]
#[ignore]
async fn test_get_spec_mainnet() {
    let client = create_client();

    let result = client.get_spec().await;

    assert!(result.is_ok(), "Failed to get spec: {:?}", result.err());

    let spec = result.unwrap();
    println!("SLOTS_PER_EPOCH: {}", spec.slots_per_epoch);
    println!("SECONDS_PER_SLOT: {}", spec.seconds_per_slot);
    println!("DEPOSIT_CONTRACT_ADDRESS: {}", spec.deposit_contract_address);
    if let Some(ref min_genesis_time) = spec.min_genesis_time {
        println!("MIN_GENESIS_TIME: {}", min_genesis_time);
    }
    if let Some(ref config_name) = spec.config_name {
        println!("CONFIG_NAME: {}", config_name);
    }
    println!("Extra fields: {}", spec.extra.len());

    // 验证主网参数
    assert_eq!(spec.slots_per_epoch, "32", "Incorrect SLOTS_PER_EPOCH");
    assert_eq!(spec.seconds_per_slot, "12", "Incorrect SECONDS_PER_SLOT");

    // 主网存款合约地址
    assert_eq!(
        spec.deposit_contract_address.to_lowercase(),
        "0x00000000219ab540356cbb839cbe05303d7705fa",
        "Incorrect deposit contract address"
    );
}

#[tokio::test]
#[ignore]
async fn test_get_fork_schedule_mainnet() {
    let client = create_client();

    let result = client.get_fork_schedule().await;

    assert!(result.is_ok(), "Failed to get fork schedule: {:?}", result.err());

    let schedule = result.unwrap();
    println!("Fork schedule:");
    for fork in &schedule.forks {
        println!("  - {}: epoch {} (version: {})", fork.name, fork.epoch, fork.version);
    }

    // 主网应该至少有 Phase 0, Altair, Bellatrix, Capella, Deneb 分叉
    assert!(schedule.forks.len() >= 5, "Fork schedule should have at least 5 forks");

    // 验证 Phase 0
    let phase0 = schedule.forks.iter().find(|f| f.name == "phase0");
    assert!(phase0.is_some(), "Phase 0 fork not found");
    assert_eq!(phase0.unwrap().epoch, "0", "Phase 0 should start at epoch 0");
}

// ================================================================================================
// 3. 区块查询测试
// ================================================================================================

#[tokio::test]
#[ignore]
async fn test_get_block_header_head_mainnet() {
    let client = create_client();

    let result = client.get_block_header(BlockId::head()).await;

    assert!(result.is_ok(), "Failed to get block header: {:?}", result.err());

    let header = result.unwrap();
    println!("Block root: {}", header.root);
    println!("Canonical: {}", header.canonical);
    println!("Slot: {}", header.header.message.slot);
    println!("Proposer index: {}", header.header.message.proposer_index);

    // 验证基本属性
    assert!(header.canonical, "Head block should be canonical");
    assert!(!header.root.is_empty(), "Block root should not be empty");

    let slot: u64 = header.header.message.slot.parse().expect("Invalid slot");
    assert!(slot > 0, "Slot should be greater than 0");
}

#[tokio::test]
#[ignore]
async fn test_get_block_header_finalized_mainnet() {
    let client = create_client();

    let result = client.get_block_header(BlockId::finalized()).await;

    assert!(result.is_ok(), "Failed to get finalized block header: {:?}", result.err());

    let header = result.unwrap();
    println!("Finalized block root: {}", header.root);
    println!("Finalized slot: {}", header.header.message.slot);

    assert!(header.canonical, "Finalized block should be canonical");
}

#[tokio::test]
#[ignore]
async fn test_get_block_root_mainnet() {
    let client = create_client();

    let result = client.get_block_root(BlockId::genesis()).await;

    assert!(result.is_ok(), "Failed to get block root: {:?}", result.err());

    let root = result.unwrap();
    println!("Genesis block root: {}", root);

    // 根哈希应该是 0x 开头的 66 字符（0x + 64 hex）
    assert!(root.starts_with("0x"), "Root should start with 0x");
    assert_eq!(root.len(), 66, "Root should be 66 characters long");
}

#[tokio::test]
#[ignore]
async fn test_get_block_attestations_mainnet() {
    let client = create_client();

    // 获取 finalized 区块的证明
    let result = client.get_block_attestations(BlockId::finalized()).await;

    assert!(result.is_ok(), "Failed to get block attestations: {:?}", result.err());

    let attestations = result.unwrap();
    println!("Number of attestations: {}", attestations.len());

    // 最终确定的区块应该有证明
    if !attestations.is_empty() {
        let first_att = &attestations[0];
        println!("First attestation slot: {}", first_att.data.slot);
        println!("Committee index: {}", first_att.data.index);
    }
}

// ================================================================================================
// 4. 状态查询测试
// ================================================================================================

#[tokio::test]
#[ignore]
async fn test_get_state_root_mainnet() {
    let client = create_client();

    let result = client.get_state_root(StateId::head()).await;

    assert!(result.is_ok(), "Failed to get state root: {:?}", result.err());

    let root = result.unwrap();
    println!("State root: {}", root);

    assert!(root.starts_with("0x"), "State root should start with 0x");
    assert_eq!(root.len(), 66, "State root should be 66 characters long");
}

#[tokio::test]
#[ignore]
async fn test_get_state_fork_mainnet() {
    let client = create_client();

    let result = client.get_state_fork(StateId::head()).await;

    assert!(result.is_ok(), "Failed to get state fork: {:?}", result.err());

    let fork = result.unwrap();
    println!("Previous version: {}", fork.previous_version);
    println!("Current version: {}", fork.current_version);
    println!("Epoch: {}", fork.epoch);

    // 当前版本应该是 Deneb 或更新版本
    assert!(!fork.current_version.is_empty(), "Current version should not be empty");
}

#[tokio::test]
#[ignore]
async fn test_get_finality_checkpoints_mainnet() {
    let client = create_client();

    let result = client.get_finality_checkpoints(StateId::head()).await;

    assert!(result.is_ok(), "Failed to get finality checkpoints: {:?}", result.err());

    let checkpoints = result.unwrap();
    println!("Previous justified epoch: {}", checkpoints.previous_justified.epoch);
    println!("Current justified epoch: {}", checkpoints.current_justified.epoch);
    println!("Finalized epoch: {}", checkpoints.finalized.epoch);

    // 验证检查点的合理性
    let prev_epoch: u64 = checkpoints.previous_justified.epoch.parse().unwrap();
    let curr_epoch: u64 = checkpoints.current_justified.epoch.parse().unwrap();
    let final_epoch: u64 = checkpoints.finalized.epoch.parse().unwrap();

    assert!(curr_epoch >= prev_epoch, "Current justified should be >= previous justified");
    assert!(curr_epoch >= final_epoch, "Current justified should be >= finalized");
}

// ================================================================================================
// 5. 验证者查询测试
// ================================================================================================

#[tokio::test]
#[ignore]
async fn test_get_validator_by_index_mainnet() {
    let client = create_client();

    // 查询第 0 个验证者（创世验证者之一）
    let result = client
        .get_validator(StateId::head(), ValidatorId::from_index("0".to_string()))
        .await;

    assert!(result.is_ok(), "Failed to get validator: {:?}", result.err());

    let validator = result.unwrap();
    println!("Validator 0 index: {}", validator.index);
    println!("Validator 0 balance: {} Gwei", validator.balance);
    println!("Validator 0 status: {:?}", validator.status);
    println!("Validator 0 pubkey: {}", validator.validator.pubkey);
    println!("Validator 0 effective_balance: {}", validator.validator.effective_balance);

    // 验证基本属性
    assert_eq!(validator.index, "0", "Validator index should be 0");
    assert!(!validator.validator.pubkey.is_empty(), "Pubkey should not be empty");
}

#[tokio::test]
#[ignore]
async fn test_get_validators_active_mainnet() {
    let client = create_client();

    // 查询活跃的验证者（限制查询数量以避免超时）
    let result = client
        .get_validators(
            StateId::head(),
            Some(vec![
                ValidatorId::from_index("0".to_string()),
                ValidatorId::from_index("1".to_string()),
                ValidatorId::from_index("2".to_string()),
            ]),
            Some(vec![ValidatorStatus::ActiveOngoing]),
        )
        .await;

    assert!(result.is_ok(), "Failed to get validators: {:?}", result.err());

    let validators = result.unwrap();
    println!("Number of active validators queried: {}", validators.len());

    // 应该至少有一些验证者
    assert!(!validators.is_empty(), "Should have at least some validators");

    // 验证第一个验证者的状态
    if let Some(first) = validators.first() {
        assert_eq!(
            first.status,
            ValidatorStatus::ActiveOngoing,
            "Validator should be active"
        );
    }
}

#[tokio::test]
#[ignore]
async fn test_get_validator_balances_mainnet() {
    let client = create_client();

    // 查询前 5 个验证者的余额
    let result = client
        .get_validator_balances(
            StateId::head(),
            Some(vec![
                ValidatorId::from_index("0".to_string()),
                ValidatorId::from_index("1".to_string()),
                ValidatorId::from_index("2".to_string()),
                ValidatorId::from_index("3".to_string()),
                ValidatorId::from_index("4".to_string()),
            ]),
        )
        .await;

    assert!(result.is_ok(), "Failed to get validator balances: {:?}", result.err());

    let balances = result.unwrap();
    println!("Number of balances: {}", balances.len());

    for balance in &balances {
        println!("Validator {} balance: {} Gwei", balance.index, balance.balance);
    }

    assert_eq!(balances.len(), 5, "Should have 5 balances");

    // 余额应该是合理的正数
    for balance in &balances {
        let balance_gwei: u64 = balance.balance.parse().expect("Invalid balance");
        assert!(balance_gwei > 0, "Balance should be greater than 0");
    }
}

#[tokio::test]
#[ignore]
async fn test_post_validators_batch_mainnet() {
    let client = create_client();

    // 批量查询验证者
    let result = client
        .post_validators(
            StateId::head(),
            vec![
                ValidatorId::from_index("0".to_string()),
                ValidatorId::from_index("1".to_string()),
                ValidatorId::from_index("2".to_string()),
            ],
            None,
        )
        .await;

    assert!(result.is_ok(), "Failed to post validators: {:?}", result.err());

    let validators = result.unwrap();
    println!("Batch query returned {} validators", validators.len());

    assert_eq!(validators.len(), 3, "Should have 3 validators");
}

// ================================================================================================
// 6. 委员会查询测试
// ================================================================================================

#[tokio::test]
#[ignore]
async fn test_get_committees_mainnet() {
    let client = create_client();

    // 查询当前 epoch 的委员会
    let result = client
        .get_committees(StateId::head(), None, None, None)
        .await;

    assert!(result.is_ok(), "Failed to get committees: {:?}", result.err());

    let committees = result.unwrap();
    println!("Number of committees: {}", committees.len());

    if !committees.is_empty() {
        let first = &committees[0];
        println!("First committee slot: {}", first.slot);
        println!("First committee index: {}", first.index);
        println!("First committee size: {}", first.validators.len());
    }

    // 应该有委员会
    assert!(!committees.is_empty(), "Should have committees");
}

#[tokio::test]
#[ignore]
async fn test_get_sync_committees_mainnet() {
    let client = create_client();

    let result = client.get_sync_committees(StateId::head(), None).await;

    assert!(result.is_ok(), "Failed to get sync committees: {:?}", result.err());

    let sync_committee = result.unwrap();
    println!("Sync committee size: {}", sync_committee.validators.len());

    // 同步委员会应该有 512 个验证者
    assert_eq!(
        sync_committee.validators.len(),
        512,
        "Sync committee should have 512 validators"
    );
}

// ================================================================================================
// 7. 交易池查询测试
// ================================================================================================

#[tokio::test]
#[ignore]
async fn test_get_pool_attestations_mainnet() {
    let client = create_client();

    let result = client.get_pool_attestations(None, None).await;

    // 交易池可能为空，这是正常的
    match result {
        Ok(attestations) => {
            println!("Pool attestations: {}", attestations.len());
        }
        Err(e) => {
            println!("Note: Pool attestations query failed (this might be normal): {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_get_pool_voluntary_exits_mainnet() {
    let client = create_client();

    let result = client.get_pool_voluntary_exits().await;

    // 自愿退出池通常为空
    match result {
        Ok(exits) => {
            println!("Pool voluntary exits: {}", exits.len());
        }
        Err(e) => {
            println!("Note: Pool voluntary exits query failed (this might be normal): {:?}", e);
        }
    }
}

// ================================================================================================
// 8. 容错性测试
// ================================================================================================

#[tokio::test]
#[ignore]
async fn test_fallback_endpoint() {
    // 测试备用端点是否可用
    let client = create_fallback_client();

    let result = client.get_genesis().await;

    assert!(result.is_ok(), "Fallback endpoint should work: {:?}", result.err());

    let genesis = result.unwrap();
    assert_eq!(
        genesis.genesis_time, "1606824023",
        "Fallback endpoint should return correct genesis"
    );
}

#[tokio::test]
#[ignore]
async fn test_invalid_block_id() {
    let client = create_client();

    // 查询一个不存在的 slot
    let result = client
        .get_block_header(BlockId::from_slot("99999999999".to_string()))
        .await;

    // 应该返回错误
    assert!(result.is_err(), "Should fail for invalid block ID");
}

#[tokio::test]
#[ignore]
async fn test_invalid_validator_id() {
    let client = create_client();

    // 查询一个不存在的验证者索引
    let result = client
        .get_validator(
            StateId::head(),
            ValidatorId::from_index("999999999999999".to_string()),
        )
        .await;

    // 应该返回 NotFound 错误
    assert!(result.is_err(), "Should fail for invalid validator ID");
}

// ================================================================================================
// 9. 性能测试
// ================================================================================================

#[tokio::test]
#[ignore]
async fn test_concurrent_requests() {
    use tokio::time::{Duration, Instant};

    use std::sync::Arc;

    let client = Arc::new(create_client());
    let start = Instant::now();

    // 并发执行 5 个请求
    let (r1, r2, r3, r4, r5) = tokio::join!(
        async { client.get_genesis().await },
        async { client.get_node_version().await },
        async { client.get_spec().await },
        async { client.get_state_root(StateId::head()).await },
        async { client.get_block_header(BlockId::head()).await },
    );

    let results = (r1, r2, r3, r4, r5);

    let duration = start.elapsed();

    println!("Concurrent requests completed in: {:?}", duration);

    // 所有请求都应该成功
    assert!(results.0.is_ok(), "Genesis request failed");
    assert!(results.1.is_ok(), "Version request failed");
    assert!(results.2.is_ok(), "Spec request failed");
    assert!(results.3.is_ok(), "State root request failed");
    assert!(results.4.is_ok(), "Block header request failed");

    // 并发请求应该比串行快
    assert!(
        duration < Duration::from_secs(10),
        "Concurrent requests should complete within 10 seconds"
    );
}
