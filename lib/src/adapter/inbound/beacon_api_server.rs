//! Beacon API RESTful 服务端实现
//!
//! 本模块实现了基于 Axum 的 Beacon API RESTful 服务端，遵循 Clean Architecture 原则。
//! 这是接口适配层的实现,将 HTTP 请求转换为领域用例调用。
//!
//! # 架构说明
//! - **接口层**: 接收 HTTP 请求,处理参数验证和响应格式化
//! - **用例层**: BeaconApi trait 定义的业务逻辑
//! - **依赖倒置**: 服务端依赖 BeaconApi trait 抽象,而非具体实现
//!
//! # 性能优化
//! - 缓存行对齐的数据结构
//! - 零拷贝设计
//! - 高性能 JSON 序列化
//!
//! 参考标准: https://github.com/ethereum/beacon-APIs

use crate::domain::service::beacon_api::{
    Attestation, BeaconApi, BlockHeaderResponse, BlockId, ChainSpec, Committee,
    FinalityCheckpoints, Fork, ForkSchedule, GenesisInfo, HealthStatus, NodeIdentity, NodeVersion,
    ServiceError, SignedBeaconBlock, SignedVoluntaryExit, StateId, SyncCommittee,
    SyncingStatus, ValidatorBalance, ValidatorId, ValidatorInfo,
};
use crate::adapter::beacon_api_types::{
    ApiError, ApiResponse, AttestationsQuery, BlockHeadersQuery, CommitteesQuery,
    PostValidatorsRequest, RootResponse, SyncCommitteesQuery, ValidatorBalancesQuery, ValidatorsQuery,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;

// ================================================================================================
// 错误处理 (Error Handling)
// ================================================================================================

/// HTTP 错误响应
///
/// 将领域层错误转换为 HTTP 响应
impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            ServiceError::NotFound(msg) => (StatusCode::NOT_FOUND, 404, msg.clone()),
            ServiceError::InvalidParameter(msg) => (StatusCode::BAD_REQUEST, 400, msg.clone()),
            ServiceError::Unhealthy => (
                StatusCode::SERVICE_UNAVAILABLE,
                503,
                "Node is unhealthy".to_string(),
            ),
            ServiceError::NotSynced => (
                StatusCode::PARTIAL_CONTENT,
                206,
                "Node is syncing".to_string(),
            ),
            ServiceError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                500,
                format!("Internal error: {}", msg),
            ),
        };

        let error = ApiError::new(code, message);
        (status, Json(error)).into_response()
    }
}

// ================================================================================================
// BeaconApiServer 结构体
// ================================================================================================

/// Beacon API RESTful 服务端
///
/// 使用 Axum 实现的高性能异步 HTTP 服务器。
///
/// # 设计原则
/// - **缓存行对齐**: 关键字段使用 `#[repr(align(64))]` 优化
/// - **依赖倒置**: 依赖 BeaconApi trait 而非具体实现
/// - **低延迟**: 零拷贝设计,最小化内存分配
#[repr(align(64))]
pub struct BeaconApiServer<T: BeaconApi> {
    /// BeaconApi 实现(可以是本地实现或远程代理)
    beacon_api: Arc<T>,
}

impl<T: BeaconApi + 'static> BeaconApiServer<T> {
    /// 创建新的 Beacon API 服务器
    ///
    /// # 参数
    /// - `beacon_api`: BeaconApi trait 实现
    ///
    /// # 返回
    /// 服务器实例
    pub fn new(beacon_api: Arc<T>) -> Self {
        Self { beacon_api }
    }

    /// 构建 Axum 路由
    ///
    /// # 返回
    /// 配置好的 Router
    pub fn router(self) -> Router {
        let state = Arc::new(self);

        Router::new()
            // ============================================================================================
            // 1. 基础信息查询端点 (Basic Information Endpoints)
            // ============================================================================================
            .route("/eth/v1/beacon/genesis", get(Self::get_genesis))
            .route("/eth/v1/node/version", get(Self::get_node_version))
            .route("/eth/v1/node/health", get(Self::get_node_health))
            .route("/eth/v1/node/syncing", get(Self::get_syncing_status))
            .route("/eth/v1/node/identity", get(Self::get_node_identity))
            // ============================================================================================
            // 2. 配置查询端点 (Configuration Endpoints)
            // ============================================================================================
            .route("/eth/v1/config/spec", get(Self::get_spec))
            .route("/eth/v1/config/fork_schedule", get(Self::get_fork_schedule))
            // ============================================================================================
            // 3. 区块头端点 (Block Headers Endpoints)
            // ============================================================================================
            .route("/eth/v1/beacon/headers", get(Self::get_block_headers))
            .route(
                "/eth/v1/beacon/headers/:block_id",
                get(Self::get_block_header),
            )
            // ============================================================================================
            // 4. 区块端点 (Blocks Endpoints)
            // ============================================================================================
            .route("/eth/v2/beacon/blocks/:block_id", get(Self::get_block))
            .route(
                "/eth/v1/beacon/blocks/:block_id/root",
                get(Self::get_block_root),
            )
            .route(
                "/eth/v1/beacon/blocks/:block_id/attestations",
                get(Self::get_block_attestations),
            )
            .route("/eth/v1/beacon/blocks", post(Self::publish_block))
            // ============================================================================================
            // 5. 状态端点 (States Endpoints)
            // ============================================================================================
            .route(
                "/eth/v1/beacon/states/:state_id/root",
                get(Self::get_state_root),
            )
            .route(
                "/eth/v1/beacon/states/:state_id/fork",
                get(Self::get_state_fork),
            )
            .route(
                "/eth/v1/beacon/states/:state_id/finality_checkpoints",
                get(Self::get_finality_checkpoints),
            )
            // ============================================================================================
            // 6. 验证者端点 (Validators Endpoints)
            // ============================================================================================
            .route(
                "/eth/v1/beacon/states/:state_id/validators",
                get(Self::get_validators).post(Self::post_validators),
            )
            .route(
                "/eth/v1/beacon/states/:state_id/validators/:validator_id",
                get(Self::get_validator),
            )
            .route(
                "/eth/v1/beacon/states/:state_id/validator_balances",
                get(Self::get_validator_balances),
            )
            // ============================================================================================
            // 7. 委员会端点 (Committees Endpoints)
            // ============================================================================================
            .route(
                "/eth/v1/beacon/states/:state_id/committees",
                get(Self::get_committees),
            )
            .route(
                "/eth/v1/beacon/states/:state_id/sync_committees",
                get(Self::get_sync_committees),
            )
            // ============================================================================================
            // 8. 交易池端点 (Pool Endpoints)
            // ============================================================================================
            .route(
                "/eth/v1/beacon/pool/attestations",
                get(Self::get_pool_attestations).post(Self::submit_pool_attestations),
            )
            .route(
                "/eth/v1/beacon/pool/voluntary_exits",
                get(Self::get_pool_voluntary_exits).post(Self::submit_pool_voluntary_exit),
            )
            .with_state(state)
    }

    // ============================================================================================
    // 处理器方法 - 1. 基础信息查询
    // ============================================================================================

    /// GET /eth/v1/beacon/genesis
    async fn get_genesis(
        State(server): State<Arc<Self>>,
    ) -> Result<Json<ApiResponse<GenesisInfo>>, ServiceError> {
        let genesis = server.beacon_api.get_genesis().await?;
        Ok(Json(ApiResponse::new(genesis)))
    }

    /// GET /eth/v1/node/version
    async fn get_node_version(
        State(server): State<Arc<Self>>,
    ) -> Result<Json<ApiResponse<NodeVersion>>, ServiceError> {
        let version = server.beacon_api.get_node_version().await?;
        Ok(Json(ApiResponse::new(version)))
    }

    /// GET /eth/v1/node/health
    ///
    /// 注意: 健康状态通过 HTTP 状态码返回
    /// - 200: Healthy
    /// - 206: Syncing
    /// - 503: Unhealthy
    async fn get_node_health(State(server): State<Arc<Self>>) -> Response {
        match server.beacon_api.get_node_health().await {
            Ok(HealthStatus::Healthy) => StatusCode::OK.into_response(),
            Ok(HealthStatus::Syncing) => StatusCode::PARTIAL_CONTENT.into_response(),
            Ok(HealthStatus::Unhealthy) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
            Err(e) => e.into_response(),
        }
    }

    /// GET /eth/v1/node/syncing
    async fn get_syncing_status(
        State(server): State<Arc<Self>>,
    ) -> Result<Json<ApiResponse<SyncingStatus>>, ServiceError> {
        let status = server.beacon_api.get_syncing_status().await?;
        Ok(Json(ApiResponse::new(status)))
    }

    /// GET /eth/v1/node/identity
    async fn get_node_identity(
        State(server): State<Arc<Self>>,
    ) -> Result<Json<ApiResponse<NodeIdentity>>, ServiceError> {
        let identity = server.beacon_api.get_node_identity().await?;
        Ok(Json(ApiResponse::new(identity)))
    }

    // ============================================================================================
    // 处理器方法 - 2. 配置查询
    // ============================================================================================

    /// GET /eth/v1/config/spec
    async fn get_spec(
        State(server): State<Arc<Self>>,
    ) -> Result<Json<ApiResponse<ChainSpec>>, ServiceError> {
        let spec = server.beacon_api.get_spec().await?;
        Ok(Json(ApiResponse::new(spec)))
    }

    /// GET /eth/v1/config/fork_schedule
    async fn get_fork_schedule(
        State(server): State<Arc<Self>>,
    ) -> Result<Json<ApiResponse<ForkSchedule>>, ServiceError> {
        let schedule = server.beacon_api.get_fork_schedule().await?;
        Ok(Json(ApiResponse::new(schedule)))
    }

    // ============================================================================================
    // 处理器方法 - 3. 区块头查询
    // ============================================================================================

    /// GET /eth/v1/beacon/headers/:block_id
    async fn get_block_header(
        State(server): State<Arc<Self>>,
        Path(block_id_str): Path<String>,
    ) -> Result<Json<ApiResponse<BlockHeaderResponse>>, ServiceError> {
        let block_id = Self::parse_block_id(&block_id_str)?;
        let header = server.beacon_api.get_block_header(block_id).await?;
        Ok(Json(ApiResponse::new(header)))
    }

    /// GET /eth/v1/beacon/headers?slot=<slot>&parent_root=<root>
    async fn get_block_headers(
        State(server): State<Arc<Self>>,
        Query(query): Query<BlockHeadersQuery>,
    ) -> Result<Json<ApiResponse<Vec<BlockHeaderResponse>>>, ServiceError> {
        let headers = server
            .beacon_api
            .get_block_headers(query.slot, query.parent_root)
            .await?;
        Ok(Json(ApiResponse::new(headers)))
    }

    // ============================================================================================
    // 处理器方法 - 4. 区块查询
    // ============================================================================================

    /// GET /eth/v2/beacon/blocks/:block_id
    async fn get_block(
        State(server): State<Arc<Self>>,
        Path(block_id_str): Path<String>,
    ) -> Result<Json<ApiResponse<SignedBeaconBlock>>, ServiceError> {
        let block_id = Self::parse_block_id(&block_id_str)?;
        let block = server.beacon_api.get_block(block_id).await?;
        Ok(Json(ApiResponse::new(block)))
    }

    /// GET /eth/v1/beacon/blocks/:block_id/root
    async fn get_block_root(
        State(server): State<Arc<Self>>,
        Path(block_id_str): Path<String>,
    ) -> Result<Json<ApiResponse<RootResponse>>, ServiceError> {
        let block_id = Self::parse_block_id(&block_id_str)?;
        let root = server.beacon_api.get_block_root(block_id).await?;
        Ok(Json(ApiResponse::new(RootResponse { root })))
    }

    /// GET /eth/v1/beacon/blocks/:block_id/attestations
    async fn get_block_attestations(
        State(server): State<Arc<Self>>,
        Path(block_id_str): Path<String>,
    ) -> Result<Json<ApiResponse<Vec<Attestation>>>, ServiceError> {
        let block_id = Self::parse_block_id(&block_id_str)?;
        let attestations = server.beacon_api.get_block_attestations(block_id).await?;
        Ok(Json(ApiResponse::new(attestations)))
    }

    /// POST /eth/v1/beacon/blocks
    async fn publish_block(
        State(server): State<Arc<Self>>,
        Json(block): Json<SignedBeaconBlock>,
    ) -> Result<StatusCode, ServiceError> {
        server.beacon_api.publish_block(block).await?;
        Ok(StatusCode::OK)
    }

    // ============================================================================================
    // 处理器方法 - 5. 状态查询
    // ============================================================================================

    /// GET /eth/v1/beacon/states/:state_id/root
    async fn get_state_root(
        State(server): State<Arc<Self>>,
        Path(state_id_str): Path<String>,
    ) -> Result<Json<ApiResponse<RootResponse>>, ServiceError> {
        let state_id = Self::parse_state_id(&state_id_str)?;
        let root = server.beacon_api.get_state_root(state_id).await?;
        Ok(Json(ApiResponse::new(RootResponse { root })))
    }

    /// GET /eth/v1/beacon/states/:state_id/fork
    async fn get_state_fork(
        State(server): State<Arc<Self>>,
        Path(state_id_str): Path<String>,
    ) -> Result<Json<ApiResponse<Fork>>, ServiceError> {
        let state_id = Self::parse_state_id(&state_id_str)?;
        let fork = server.beacon_api.get_state_fork(state_id).await?;
        Ok(Json(ApiResponse::new(fork)))
    }

    /// GET /eth/v1/beacon/states/:state_id/finality_checkpoints
    async fn get_finality_checkpoints(
        State(server): State<Arc<Self>>,
        Path(state_id_str): Path<String>,
    ) -> Result<Json<ApiResponse<FinalityCheckpoints>>, ServiceError> {
        let state_id = Self::parse_state_id(&state_id_str)?;
        let checkpoints = server.beacon_api.get_finality_checkpoints(state_id).await?;
        Ok(Json(ApiResponse::new(checkpoints)))
    }

    // ============================================================================================
    // 处理器方法 - 6. 验证者查询
    // ============================================================================================

    /// GET /eth/v1/beacon/states/:state_id/validators?id=<id>&status=<status>
    async fn get_validators(
        State(server): State<Arc<Self>>,
        Path(state_id_str): Path<String>,
        Query(query): Query<ValidatorsQuery>,
    ) -> Result<Json<ApiResponse<Vec<ValidatorInfo>>>, ServiceError> {
        let state_id = Self::parse_state_id(&state_id_str)?;

        let ids = if query.id.is_empty() {
            None
        } else {
            Some(
                query
                    .id
                    .iter()
                    .map(|s| Self::parse_validator_id(s))
                    .collect::<Result<Vec<_>, _>>()?,
            )
        };

        let statuses = if query.status.is_empty() {
            None
        } else {
            Some(query.status)
        };

        let validators = server.beacon_api.get_validators(state_id, ids, statuses).await?;
        Ok(Json(ApiResponse::new(validators)))
    }

    /// GET /eth/v1/beacon/states/:state_id/validators/:validator_id
    async fn get_validator(
        State(server): State<Arc<Self>>,
        Path((state_id_str, validator_id_str)): Path<(String, String)>,
    ) -> Result<Json<ApiResponse<ValidatorInfo>>, ServiceError> {
        let state_id = Self::parse_state_id(&state_id_str)?;
        let validator_id = Self::parse_validator_id(&validator_id_str)?;
        let validator = server.beacon_api.get_validator(state_id, validator_id).await?;
        Ok(Json(ApiResponse::new(validator)))
    }

    /// GET /eth/v1/beacon/states/:state_id/validator_balances?id=<id>
    async fn get_validator_balances(
        State(server): State<Arc<Self>>,
        Path(state_id_str): Path<String>,
        Query(query): Query<ValidatorBalancesQuery>,
    ) -> Result<Json<ApiResponse<Vec<ValidatorBalance>>>, ServiceError> {
        let state_id = Self::parse_state_id(&state_id_str)?;

        let ids = if query.id.is_empty() {
            None
        } else {
            Some(
                query
                    .id
                    .iter()
                    .map(|s| Self::parse_validator_id(s))
                    .collect::<Result<Vec<_>, _>>()?,
            )
        };

        let balances = server.beacon_api.get_validator_balances(state_id, ids).await?;
        Ok(Json(ApiResponse::new(balances)))
    }

    /// POST /eth/v1/beacon/states/:state_id/validators
    async fn post_validators(
        State(server): State<Arc<Self>>,
        Path(state_id_str): Path<String>,
        Json(request): Json<PostValidatorsRequest>,
    ) -> Result<Json<ApiResponse<Vec<ValidatorInfo>>>, ServiceError> {
        let state_id = Self::parse_state_id(&state_id_str)?;

        let ids = request
            .ids
            .iter()
            .map(|s| Self::parse_validator_id(s))
            .collect::<Result<Vec<_>, _>>()?;

        let validators = server
            .beacon_api
            .post_validators(state_id, ids, request.statuses)
            .await?;
        Ok(Json(ApiResponse::new(validators)))
    }

    // ============================================================================================
    // 处理器方法 - 7. 委员会查询
    // ============================================================================================

    /// GET /eth/v1/beacon/states/:state_id/committees?epoch=<epoch>&index=<index>&slot=<slot>
    async fn get_committees(
        State(server): State<Arc<Self>>,
        Path(state_id_str): Path<String>,
        Query(query): Query<CommitteesQuery>,
    ) -> Result<Json<ApiResponse<Vec<Committee>>>, ServiceError> {
        let state_id = Self::parse_state_id(&state_id_str)?;
        let committees = server
            .beacon_api
            .get_committees(state_id, query.epoch, query.index, query.slot)
            .await?;
        Ok(Json(ApiResponse::new(committees)))
    }

    /// GET /eth/v1/beacon/states/:state_id/sync_committees?epoch=<epoch>
    async fn get_sync_committees(
        State(server): State<Arc<Self>>,
        Path(state_id_str): Path<String>,
        Query(query): Query<SyncCommitteesQuery>,
    ) -> Result<Json<ApiResponse<SyncCommittee>>, ServiceError> {
        let state_id = Self::parse_state_id(&state_id_str)?;
        let sync_committee = server
            .beacon_api
            .get_sync_committees(state_id, query.epoch)
            .await?;
        Ok(Json(ApiResponse::new(sync_committee)))
    }

    // ============================================================================================
    // 处理器方法 - 8. 交易池查询
    // ============================================================================================

    /// GET /eth/v1/beacon/pool/attestations?slot=<slot>&committee_index=<index>
    async fn get_pool_attestations(
        State(server): State<Arc<Self>>,
        Query(query): Query<AttestationsQuery>,
    ) -> Result<Json<ApiResponse<Vec<Attestation>>>, ServiceError> {
        let attestations = server
            .beacon_api
            .get_pool_attestations(query.slot, query.committee_index)
            .await?;
        Ok(Json(ApiResponse::new(attestations)))
    }

    /// POST /eth/v1/beacon/pool/attestations
    async fn submit_pool_attestations(
        State(server): State<Arc<Self>>,
        Json(attestations): Json<Vec<Attestation>>,
    ) -> Result<StatusCode, ServiceError> {
        server.beacon_api.submit_pool_attestations(attestations).await?;
        Ok(StatusCode::OK)
    }

    /// GET /eth/v1/beacon/pool/voluntary_exits
    async fn get_pool_voluntary_exits(
        State(server): State<Arc<Self>>,
    ) -> Result<Json<ApiResponse<Vec<SignedVoluntaryExit>>>, ServiceError> {
        let exits = server.beacon_api.get_pool_voluntary_exits().await?;
        Ok(Json(ApiResponse::new(exits)))
    }

    /// POST /eth/v1/beacon/pool/voluntary_exits
    async fn submit_pool_voluntary_exit(
        State(server): State<Arc<Self>>,
        Json(exit): Json<SignedVoluntaryExit>,
    ) -> Result<StatusCode, ServiceError> {
        server.beacon_api.submit_pool_voluntary_exit(exit).await?;
        Ok(StatusCode::OK)
    }

    // ============================================================================================
    // 辅助方法 - 参数解析
    // ============================================================================================

    /// 解析 StateId 参数
    fn parse_state_id(s: &str) -> Result<StateId, ServiceError> {
        match s {
            "head" => Ok(StateId::Head),
            "genesis" => Ok(StateId::Genesis),
            "finalized" => Ok(StateId::Finalized),
            "justified" => Ok(StateId::Justified),
            _ => {
                if s.starts_with("0x") {
                    Ok(StateId::Root(s.to_string()))
                } else {
                    // 假设是 slot 编号
                    Ok(StateId::Slot(s.to_string()))
                }
            }
        }
    }

    /// 解析 BlockId 参数
    fn parse_block_id(s: &str) -> Result<BlockId, ServiceError> {
        match s {
            "head" => Ok(BlockId::Head),
            "genesis" => Ok(BlockId::Genesis),
            "finalized" => Ok(BlockId::Finalized),
            _ => {
                if s.starts_with("0x") {
                    Ok(BlockId::Root(s.to_string()))
                } else {
                    // 假设是 slot 编号
                    Ok(BlockId::Slot(s.to_string()))
                }
            }
        }
    }

    /// 解析 ValidatorId 参数
    fn parse_validator_id(s: &str) -> Result<ValidatorId, ServiceError> {
        if s.starts_with("0x") {
            Ok(ValidatorId::PublicKey(s.to_string()))
        } else {
            // 假设是索引
            Ok(ValidatorId::Index(s.to_string()))
        }
    }
}

// ================================================================================================
// 测试模块
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::domain::service::beacon_api::{Epoch, Root, Slot, ValidatorStatus};

    #[test]
    fn test_parse_state_id() {
        type Server = BeaconApiServer<MockBeaconApi>;

        assert_eq!(
            Server::parse_state_id("head").unwrap(),
            StateId::Head
        );
        assert_eq!(
            Server::parse_state_id("genesis").unwrap(),
            StateId::Genesis
        );
        assert_eq!(
            Server::parse_state_id("finalized").unwrap(),
            StateId::Finalized
        );
        assert_eq!(
            Server::parse_state_id("justified").unwrap(),
            StateId::Justified
        );
        assert_eq!(
            Server::parse_state_id("12345").unwrap(),
            StateId::Slot("12345".to_string())
        );
        assert_eq!(
            Server::parse_state_id("0xabcd1234").unwrap(),
            StateId::Root("0xabcd1234".to_string())
        );
    }

    #[test]
    fn test_parse_block_id() {
        type Server = BeaconApiServer<MockBeaconApi>;

        assert_eq!(
            Server::parse_block_id("head").unwrap(),
            BlockId::Head
        );
        assert_eq!(
            Server::parse_block_id("genesis").unwrap(),
            BlockId::Genesis
        );
        assert_eq!(
            Server::parse_block_id("finalized").unwrap(),
            BlockId::Finalized
        );
        assert_eq!(
            Server::parse_block_id("12345").unwrap(),
            BlockId::Slot("12345".to_string())
        );
        assert_eq!(
            Server::parse_block_id("0xabcd1234").unwrap(),
            BlockId::Root("0xabcd1234".to_string())
        );
    }

    #[test]
    fn test_parse_validator_id() {
        type Server = BeaconApiServer<MockBeaconApi>;

        assert_eq!(
            Server::parse_validator_id("12345").unwrap(),
            ValidatorId::Index("12345".to_string())
        );
        assert_eq!(
            Server::parse_validator_id("0xabcd1234").unwrap(),
            ValidatorId::PublicKey("0xabcd1234".to_string())
        );
    }

    // Mock implementation for testing
    struct MockBeaconApi;

    #[async_trait]
    impl BeaconApi for MockBeaconApi {
        async fn get_genesis(&self) -> Result<GenesisInfo, ServiceError> {
            unimplemented!()
        }

        async fn get_node_version(&self) -> Result<NodeVersion, ServiceError> {
            unimplemented!()
        }

        async fn get_node_health(&self) -> Result<HealthStatus, ServiceError> {
            unimplemented!()
        }

        async fn get_syncing_status(&self) -> Result<SyncingStatus, ServiceError> {
            unimplemented!()
        }

        async fn get_node_identity(&self) -> Result<NodeIdentity, ServiceError> {
            unimplemented!()
        }

        async fn get_spec(&self) -> Result<ChainSpec, ServiceError> {
            unimplemented!()
        }

        async fn get_fork_schedule(&self) -> Result<ForkSchedule, ServiceError> {
            unimplemented!()
        }

        async fn get_block_header(&self, _block_id: BlockId) -> Result<BlockHeaderResponse, ServiceError> {
            unimplemented!()
        }

        async fn get_block_headers(&self, _slot: Option<Slot>, _parent_root: Option<Root>) -> Result<Vec<BlockHeaderResponse>, ServiceError> {
            unimplemented!()
        }

        async fn get_block(&self, _block_id: BlockId) -> Result<SignedBeaconBlock, ServiceError> {
            unimplemented!()
        }

        async fn get_block_root(&self, _block_id: BlockId) -> Result<Root, ServiceError> {
            unimplemented!()
        }

        async fn get_block_attestations(&self, _block_id: BlockId) -> Result<Vec<Attestation>, ServiceError> {
            unimplemented!()
        }

        async fn publish_block(&self, _block: SignedBeaconBlock) -> Result<(), ServiceError> {
            unimplemented!()
        }

        async fn get_state_root(&self, _state_id: StateId) -> Result<Root, ServiceError> {
            unimplemented!()
        }

        async fn get_state_fork(&self, _state_id: StateId) -> Result<Fork, ServiceError> {
            unimplemented!()
        }

        async fn get_finality_checkpoints(&self, _state_id: StateId) -> Result<FinalityCheckpoints, ServiceError> {
            unimplemented!()
        }

        async fn get_validators(&self, _state_id: StateId, _ids: Option<Vec<ValidatorId>>, _statuses: Option<Vec<ValidatorStatus>>) -> Result<Vec<ValidatorInfo>, ServiceError> {
            unimplemented!()
        }

        async fn get_validator(&self, _state_id: StateId, _validator_id: ValidatorId) -> Result<ValidatorInfo, ServiceError> {
            unimplemented!()
        }

        async fn get_validator_balances(&self, _state_id: StateId, _ids: Option<Vec<ValidatorId>>) -> Result<Vec<ValidatorBalance>, ServiceError> {
            unimplemented!()
        }

        async fn post_validators(&self, _state_id: StateId, _ids: Vec<ValidatorId>, _statuses: Option<Vec<ValidatorStatus>>) -> Result<Vec<ValidatorInfo>, ServiceError> {
            unimplemented!()
        }

        async fn get_committees(&self, _state_id: StateId, _epoch: Option<Epoch>, _index: Option<String>, _slot: Option<Slot>) -> Result<Vec<Committee>, ServiceError> {
            unimplemented!()
        }

        async fn get_sync_committees(&self, _state_id: StateId, _epoch: Option<Epoch>) -> Result<SyncCommittee, ServiceError> {
            unimplemented!()
        }

        async fn get_pool_attestations(&self, _slot: Option<Slot>, _committee_index: Option<String>) -> Result<Vec<Attestation>, ServiceError> {
            unimplemented!()
        }

        async fn submit_pool_attestations(&self, _attestations: Vec<Attestation>) -> Result<(), ServiceError> {
            unimplemented!()
        }

        async fn get_pool_voluntary_exits(&self) -> Result<Vec<SignedVoluntaryExit>, ServiceError> {
            unimplemented!()
        }

        async fn submit_pool_voluntary_exit(&self, _exit: SignedVoluntaryExit) -> Result<(), ServiceError> {
            unimplemented!()
        }
    }
}