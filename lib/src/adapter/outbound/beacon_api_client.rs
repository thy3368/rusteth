//! Beacon API RESTful 客户端实现
//!
//! 本模块实现了基于 HTTP/REST 的 Beacon API 客户端，遵循 Clean Architecture 原则。
//! 这是基础设施层的具体实现，实现了领域层定义的 BeaconApi trait。
//!
//! 参考标准: https://github.com/ethereum/beacon-APIs

use crate::domain::service::beacon_api::{
    Attestation, BeaconApi, BlockHeaderResponse, BlockId, ChainSpec, Committee, Epoch,
    FinalityCheckpoints, Fork, ForkSchedule, GenesisInfo, HealthStatus, NodeIdentity, NodeVersion,
    RepositoryError, Root, SignedBeaconBlock, SignedVoluntaryExit, Slot, StateId, SyncCommittee,
    SyncingStatus, ValidatorBalance, ValidatorId, ValidatorInfo, ValidatorStatus,
};
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ================================================================================================
// API 响应包装类型 (Response Wrappers)
// ================================================================================================

/// 标准 Beacon API 响应格式
///
/// 所有成功的 Beacon API 响应都遵循此格式：
/// ```json
/// {
///   "data": <实际数据>
/// }
/// ```
#[derive(Debug, Deserialize, Serialize)]
struct ApiResponse<T> {
    data: T,
}

/// API 错误响应格式
#[derive(Debug, Deserialize, Serialize)]
struct ApiError {
    code: u16,
    message: String,
    #[serde(default)]
    stacktraces: Vec<String>,
}

// ================================================================================================
// BeaconApiClient 结构体
// ================================================================================================

/// Beacon API RESTful 客户端
///
/// 使用 reqwest 实现的异步 HTTP 客户端，用于与 Beacon Node 通信。
///
/// # 设计原则
/// - **缓存行对齐**: 关键字段使用 `#[repr(align(64))]` 优化
/// - **不可变性**: 所有字段不可变，线程安全
/// - **低延迟**: 配置了合理的超时和连接池参数
#[repr(align(64))] // 缓存行对齐优化性能
pub struct BeaconApiClient {
    /// HTTP 客户端
    client: Client,
    /// Beacon Node 基础 URL
    base_url: String,
}

impl BeaconApiClient {
    /// 创建新的 Beacon API 客户端
    ///
    /// # 参数
    /// - `base_url`: Beacon Node 的基础 URL（例如 "http://localhost:5052"）
    ///
    /// # 返回
    /// 配置好的客户端实例
    ///
    /// # 性能优化
    /// - 启用连接池复用
    /// - 设置合理的超时时间（30秒）
    /// - 启用 gzip 压缩
    pub fn new(base_url: impl Into<String>) -> Result<Self, RepositoryError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10) // 连接池优化
            .gzip(true) // 启用 gzip 压缩
            .build()
            .map_err(|e| RepositoryError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            base_url: base_url.into(),
        })
    }

    /// 辅助方法：构建完整 URL
    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// 辅助方法：将 StateId 转换为 URL 路径参数
    fn state_id_to_path(&self, state_id: &StateId) -> String {
        match state_id {
            StateId::Head => "head".to_string(),
            StateId::Genesis => "genesis".to_string(),
            StateId::Finalized => "finalized".to_string(),
            StateId::Justified => "justified".to_string(),
            StateId::Slot(slot) => slot.clone(),
            StateId::Root(root) => root.clone(),
        }
    }

    /// 辅助方法：将 BlockId 转换为 URL 路径参数
    fn block_id_to_path(&self, block_id: &BlockId) -> String {
        match block_id {
            BlockId::Head => "head".to_string(),
            BlockId::Genesis => "genesis".to_string(),
            BlockId::Finalized => "finalized".to_string(),
            BlockId::Slot(slot) => slot.clone(),
            BlockId::Root(root) => root.clone(),
        }
    }

    /// 辅助方法：将 ValidatorId 转换为 URL 路径参数
    fn validator_id_to_path(&self, validator_id: &ValidatorId) -> String {
        match validator_id {
            ValidatorId::PublicKey(pubkey) => pubkey.clone(),
            ValidatorId::Index(index) => index.clone(),
        }
    }

    /// 辅助方法：执行 GET 请求并解析响应
    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, RepositoryError> {
        let url = self.build_url(path);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| RepositoryError::Internal(format!("HTTP request failed: {}", e)))?;

        let status = response.status();

        // 处理特殊 HTTP 状态码
        match status {
            StatusCode::OK => {
                // 200: 成功
                let api_response: ApiResponse<T> = response
                    .json()
                    .await
                    .map_err(|e| RepositoryError::Internal(format!("Failed to parse response: {}", e)))?;
                Ok(api_response.data)
            }
            StatusCode::NOT_FOUND => {
                // 404: 资源未找到
                Err(RepositoryError::NotFound(format!("Resource not found: {}", url)))
            }
            StatusCode::BAD_REQUEST => {
                // 400: 请求参数错误
                let error: ApiError = response
                    .json()
                    .await
                    .map_err(|e| RepositoryError::Internal(format!("Failed to parse error: {}", e)))?;
                Err(RepositoryError::InvalidParameter(error.message))
            }
            StatusCode::SERVICE_UNAVAILABLE => {
                // 503: 节点不健康
                Err(RepositoryError::Unhealthy)
            }
            StatusCode::PARTIAL_CONTENT => {
                // 206: 节点正在同步
                Err(RepositoryError::NotSynced)
            }
            _ => {
                // 其他错误
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Failed to read response body".to_string());
                Err(RepositoryError::Internal(format!(
                    "Unexpected status {}: {}",
                    status, body
                )))
            }
        }
    }

    /// 辅助方法：执行 POST 请求并解析响应
    async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, RepositoryError> {
        let url = self.build_url(path);

        let response = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| RepositoryError::Internal(format!("HTTP request failed: {}", e)))?;

        let status = response.status();

        match status {
            StatusCode::OK | StatusCode::ACCEPTED => {
                let api_response: ApiResponse<T> = response
                    .json()
                    .await
                    .map_err(|e| RepositoryError::Internal(format!("Failed to parse response: {}", e)))?;
                Ok(api_response.data)
            }
            StatusCode::BAD_REQUEST => {
                let error: ApiError = response
                    .json()
                    .await
                    .map_err(|e| RepositoryError::Internal(format!("Failed to parse error: {}", e)))?;
                Err(RepositoryError::InvalidParameter(error.message))
            }
            _ => {
                let body_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Failed to read response body".to_string());
                Err(RepositoryError::Internal(format!(
                    "Unexpected status {}: {}",
                    status, body_text
                )))
            }
        }
    }

    /// 辅助方法：执行 POST 请求（无响应体）
    async fn post_empty<B: Serialize>(&self, path: &str, body: &B) -> Result<(), RepositoryError> {
        let url = self.build_url(path);

        let response = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| RepositoryError::Internal(format!("HTTP request failed: {}", e)))?;

        let status = response.status();

        match status {
            StatusCode::OK | StatusCode::ACCEPTED | StatusCode::NO_CONTENT => Ok(()),
            StatusCode::BAD_REQUEST => {
                let error: ApiError = response
                    .json()
                    .await
                    .map_err(|e| RepositoryError::Internal(format!("Failed to parse error: {}", e)))?;
                Err(RepositoryError::InvalidParameter(error.message))
            }
            _ => {
                let body_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Failed to read response body".to_string());
                Err(RepositoryError::Internal(format!(
                    "Unexpected status {}: {}",
                    status, body_text
                )))
            }
        }
    }
}

// ================================================================================================
// BeaconApi Trait 实现
// ================================================================================================

#[async_trait]
impl BeaconApi for BeaconApiClient {
    // ============================================================================================
    // 1. 基础信息查询 (Basic Information)
    // ============================================================================================

    /// 获取创世信息
    ///
    /// 端点: GET /eth/v1/beacon/genesis
    async fn get_genesis(&self) -> Result<GenesisInfo, RepositoryError> {
        self.get("/eth/v1/beacon/genesis").await
    }

    /// 获取节点版本
    ///
    /// 端点: GET /eth/v1/node/version
    async fn get_node_version(&self) -> Result<NodeVersion, RepositoryError> {
        self.get("/eth/v1/node/version").await
    }

    /// 获取节点健康状态
    ///
    /// 端点: GET /eth/v1/node/health
    ///
    /// 注意：此方法通过 HTTP 状态码判断健康状态：
    /// - 200: Healthy
    /// - 206: Syncing
    /// - 503: Unhealthy
    /// - 404: 端点不支持（返回 NotFound 错误）
    async fn get_node_health(&self) -> Result<HealthStatus, RepositoryError> {
        let url = self.build_url("/eth/v1/node/health");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| RepositoryError::Internal(format!("HTTP request failed: {}", e)))?;

        match response.status() {
            StatusCode::OK => Ok(HealthStatus::Healthy),
            StatusCode::PARTIAL_CONTENT => Ok(HealthStatus::Syncing),
            StatusCode::SERVICE_UNAVAILABLE => Ok(HealthStatus::Unhealthy),
            StatusCode::NOT_FOUND => {
                // 端点不支持（某些公开节点可能不提供此端点）
                Err(RepositoryError::NotFound(
                    "Health endpoint not supported by this node".to_string()
                ))
            }
            status => Err(RepositoryError::Internal(format!(
                "Unexpected health status: {}",
                status
            ))),
        }
    }

    /// 获取同步状态
    ///
    /// 端点: GET /eth/v1/node/syncing
    async fn get_syncing_status(&self) -> Result<SyncingStatus, RepositoryError> {
        self.get("/eth/v1/node/syncing").await
    }

    /// 获取节点身份信息
    ///
    /// 端点: GET /eth/v1/node/identity
    async fn get_node_identity(&self) -> Result<NodeIdentity, RepositoryError> {
        self.get("/eth/v1/node/identity").await
    }

    // ============================================================================================
    // 2. 配置查询 (Configuration)
    // ============================================================================================

    /// 获取链规范参数
    ///
    /// 端点: GET /eth/v1/config/spec
    async fn get_spec(&self) -> Result<ChainSpec, RepositoryError> {
        self.get("/eth/v1/config/spec").await
    }

    /// 获取分叉时间表
    ///
    /// 端点: GET /eth/v1/config/fork_schedule
    async fn get_fork_schedule(&self) -> Result<ForkSchedule, RepositoryError> {
        self.get("/eth/v1/config/fork_schedule").await
    }

    // ============================================================================================
    // 3. 区块头查询 (Block Headers)
    // ============================================================================================

    /// 获取区块头
    ///
    /// 端点: GET /eth/v1/beacon/headers/{block_id}
    async fn get_block_header(&self, block_id: BlockId) -> Result<BlockHeaderResponse, RepositoryError> {
        let block_path = self.block_id_to_path(&block_id);
        let path = format!("/eth/v1/beacon/headers/{}", block_path);
        self.get(&path).await
    }

    /// 获取区块头列表
    ///
    /// 端点: GET /eth/v1/beacon/headers?slot=<slot>&parent_root=<root>
    async fn get_block_headers(
        &self,
        slot: Option<Slot>,
        parent_root: Option<Root>,
    ) -> Result<Vec<BlockHeaderResponse>, RepositoryError> {
        let mut path = "/eth/v1/beacon/headers".to_string();
        let mut params = Vec::new();

        if let Some(s) = slot {
            params.push(format!("slot={}", s));
        }
        if let Some(root) = parent_root {
            params.push(format!("parent_root={}", root));
        }

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        self.get(&path).await
    }

    // ============================================================================================
    // 4. 区块查询 (Blocks)
    // ============================================================================================

    /// 获取信标区块
    ///
    /// 端点: GET /eth/v2/beacon/blocks/{block_id}
    async fn get_block(&self, block_id: BlockId) -> Result<SignedBeaconBlock, RepositoryError> {
        let block_path = self.block_id_to_path(&block_id);
        let path = format!("/eth/v2/beacon/blocks/{}", block_path);
        self.get(&path).await
    }

    /// 获取区块根哈希
    ///
    /// 端点: GET /eth/v1/beacon/blocks/{block_id}/root
    async fn get_block_root(&self, block_id: BlockId) -> Result<Root, RepositoryError> {
        let block_path = self.block_id_to_path(&block_id);
        let path = format!("/eth/v1/beacon/blocks/{}/root", block_path);

        #[derive(Deserialize)]
        struct RootResponse {
            root: Root,
        }

        let response: RootResponse = self.get(&path).await?;
        Ok(response.root)
    }

    /// 获取区块中的证明
    ///
    /// 端点: GET /eth/v1/beacon/blocks/{block_id}/attestations
    async fn get_block_attestations(&self, block_id: BlockId) -> Result<Vec<Attestation>, RepositoryError> {
        let block_path = self.block_id_to_path(&block_id);
        let path = format!("/eth/v1/beacon/blocks/{}/attestations", block_path);
        self.get(&path).await
    }

    /// 发布信标区块
    ///
    /// 端点: POST /eth/v1/beacon/blocks
    async fn publish_block(&self, block: SignedBeaconBlock) -> Result<(), RepositoryError> {
        self.post_empty("/eth/v1/beacon/blocks", &block).await
    }

    // ============================================================================================
    // 5. 状态查询 (States)
    // ============================================================================================

    /// 获取状态根哈希
    ///
    /// 端点: GET /eth/v1/beacon/states/{state_id}/root
    async fn get_state_root(&self, state_id: StateId) -> Result<Root, RepositoryError> {
        let state_path = self.state_id_to_path(&state_id);
        let path = format!("/eth/v1/beacon/states/{}/root", state_path);

        #[derive(Deserialize)]
        struct RootResponse {
            root: Root,
        }

        let response: RootResponse = self.get(&path).await?;
        Ok(response.root)
    }

    /// 获取分叉信息
    ///
    /// 端点: GET /eth/v1/beacon/states/{state_id}/fork
    async fn get_state_fork(&self, state_id: StateId) -> Result<Fork, RepositoryError> {
        let state_path = self.state_id_to_path(&state_id);
        let path = format!("/eth/v1/beacon/states/{}/fork", state_path);
        self.get(&path).await
    }

    /// 获取最终性检查点
    ///
    /// 端点: GET /eth/v1/beacon/states/{state_id}/finality_checkpoints
    async fn get_finality_checkpoints(&self, state_id: StateId) -> Result<FinalityCheckpoints, RepositoryError> {
        let state_path = self.state_id_to_path(&state_id);
        let path = format!("/eth/v1/beacon/states/{}/finality_checkpoints", state_path);
        self.get(&path).await
    }

    // ============================================================================================
    // 6. 验证者查询 (Validators)
    // ============================================================================================

    /// 获取验证者列表
    ///
    /// 端点: GET /eth/v1/beacon/states/{state_id}/validators?id=<id>&status=<status>
    async fn get_validators(
        &self,
        state_id: StateId,
        ids: Option<Vec<ValidatorId>>,
        statuses: Option<Vec<ValidatorStatus>>,
    ) -> Result<Vec<ValidatorInfo>, RepositoryError> {
        let state_path = self.state_id_to_path(&state_id);
        let mut path = format!("/eth/v1/beacon/states/{}/validators", state_path);
        let mut params = Vec::new();

        if let Some(id_list) = ids {
            for id in id_list {
                let id_str = self.validator_id_to_path(&id);
                params.push(format!("id={}", id_str));
            }
        }

        if let Some(status_list) = statuses {
            for status in status_list {
                let status_str = serde_json::to_string(&status)
                    .map_err(|e| RepositoryError::Internal(format!("Failed to serialize status: {}", e)))?
                    .trim_matches('"')
                    .to_string();
                params.push(format!("status={}", status_str));
            }
        }

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        self.get(&path).await
    }

    /// 获取单个验证者信息
    ///
    /// 端点: GET /eth/v1/beacon/states/{state_id}/validators/{validator_id}
    async fn get_validator(
        &self,
        state_id: StateId,
        validator_id: ValidatorId,
    ) -> Result<ValidatorInfo, RepositoryError> {
        let state_path = self.state_id_to_path(&state_id);
        let validator_path = self.validator_id_to_path(&validator_id);
        let path = format!(
            "/eth/v1/beacon/states/{}/validators/{}",
            state_path, validator_path
        );
        self.get(&path).await
    }

    /// 获取验证者余额列表
    ///
    /// 端点: GET /eth/v1/beacon/states/{state_id}/validator_balances?id=<id>
    async fn get_validator_balances(
        &self,
        state_id: StateId,
        ids: Option<Vec<ValidatorId>>,
    ) -> Result<Vec<ValidatorBalance>, RepositoryError> {
        let state_path = self.state_id_to_path(&state_id);
        let mut path = format!("/eth/v1/beacon/states/{}/validator_balances", state_path);

        if let Some(id_list) = ids {
            let mut params = Vec::new();
            for id in id_list {
                let id_str = self.validator_id_to_path(&id);
                params.push(format!("id={}", id_str));
            }
            if !params.is_empty() {
                path.push('?');
                path.push_str(&params.join("&"));
            }
        }

        self.get(&path).await
    }

    /// 批量查询验证者（POST）
    ///
    /// 端点: POST /eth/v1/beacon/states/{state_id}/validators
    async fn post_validators(
        &self,
        state_id: StateId,
        ids: Vec<ValidatorId>,
        statuses: Option<Vec<ValidatorStatus>>,
    ) -> Result<Vec<ValidatorInfo>, RepositoryError> {
        let state_path = self.state_id_to_path(&state_id);
        let path = format!("/eth/v1/beacon/states/{}/validators", state_path);

        #[derive(Serialize)]
        struct ValidatorsRequest {
            ids: Vec<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            statuses: Option<Vec<ValidatorStatus>>,
        }

        let ids_str: Vec<String> = ids.iter().map(|id| self.validator_id_to_path(id)).collect();
        let request = ValidatorsRequest {
            ids: ids_str,
            statuses,
        };

        self.post(&path, &request).await
    }

    // ============================================================================================
    // 7. 委员会查询 (Committees)
    // ============================================================================================

    /// 获取委员会信息
    ///
    /// 端点: GET /eth/v1/beacon/states/{state_id}/committees?epoch=<epoch>&index=<index>&slot=<slot>
    async fn get_committees(
        &self,
        state_id: StateId,
        epoch: Option<Epoch>,
        index: Option<String>,
        slot: Option<Slot>,
    ) -> Result<Vec<Committee>, RepositoryError> {
        let state_path = self.state_id_to_path(&state_id);
        let mut path = format!("/eth/v1/beacon/states/{}/committees", state_path);
        let mut params = Vec::new();

        if let Some(e) = epoch {
            params.push(format!("epoch={}", e));
        }
        if let Some(i) = index {
            params.push(format!("index={}", i));
        }
        if let Some(s) = slot {
            params.push(format!("slot={}", s));
        }

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        self.get(&path).await
    }

    /// 获取同步委员会
    ///
    /// 端点: GET /eth/v1/beacon/states/{state_id}/sync_committees?epoch=<epoch>
    async fn get_sync_committees(
        &self,
        state_id: StateId,
        epoch: Option<Epoch>,
    ) -> Result<SyncCommittee, RepositoryError> {
        let state_path = self.state_id_to_path(&state_id);
        let mut path = format!("/eth/v1/beacon/states/{}/sync_committees", state_path);

        if let Some(e) = epoch {
            path.push_str(&format!("?epoch={}", e));
        }

        self.get(&path).await
    }

    // ============================================================================================
    // 8. 交易池查询 (Pool)
    // ============================================================================================

    /// 获取待处理证明
    ///
    /// 端点: GET /eth/v1/beacon/pool/attestations?slot=<slot>&committee_index=<index>
    async fn get_pool_attestations(
        &self,
        slot: Option<Slot>,
        committee_index: Option<String>,
    ) -> Result<Vec<Attestation>, RepositoryError> {
        let mut path = "/eth/v1/beacon/pool/attestations".to_string();
        let mut params = Vec::new();

        if let Some(s) = slot {
            params.push(format!("slot={}", s));
        }
        if let Some(index) = committee_index {
            params.push(format!("committee_index={}", index));
        }

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        self.get(&path).await
    }

    /// 提交证明
    ///
    /// 端点: POST /eth/v1/beacon/pool/attestations
    async fn submit_pool_attestations(&self, attestations: Vec<Attestation>) -> Result<(), RepositoryError> {
        self.post_empty("/eth/v1/beacon/pool/attestations", &attestations)
            .await
    }

    /// 获取自愿退出
    ///
    /// 端点: GET /eth/v1/beacon/pool/voluntary_exits
    async fn get_pool_voluntary_exits(&self) -> Result<Vec<SignedVoluntaryExit>, RepositoryError> {
        self.get("/eth/v1/beacon/pool/voluntary_exits").await
    }

    /// 提交自愿退出
    ///
    /// 端点: POST /eth/v1/beacon/pool/voluntary_exits
    async fn submit_pool_voluntary_exit(&self, exit: SignedVoluntaryExit) -> Result<(), RepositoryError> {
        self.post_empty("/eth/v1/beacon/pool/voluntary_exits", &exit)
            .await
    }
}

// ================================================================================================
// 测试模块
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = BeaconApiClient::new("http://localhost:5052");
        assert!(client.is_ok());
    }

    #[test]
    fn test_url_building() {
        let client = BeaconApiClient::new("http://localhost:5052").unwrap();
        assert_eq!(
            client.build_url("/eth/v1/beacon/genesis"),
            "http://localhost:5052/eth/v1/beacon/genesis"
        );
    }

    #[test]
    fn test_state_id_conversion() {
        let client = BeaconApiClient::new("http://localhost:5052").unwrap();

        assert_eq!(client.state_id_to_path(&StateId::Head), "head");
        assert_eq!(client.state_id_to_path(&StateId::Genesis), "genesis");
        assert_eq!(client.state_id_to_path(&StateId::Finalized), "finalized");
        assert_eq!(client.state_id_to_path(&StateId::Justified), "justified");
        assert_eq!(
            client.state_id_to_path(&StateId::Slot("12345".to_string())),
            "12345"
        );
        assert_eq!(
            client.state_id_to_path(&StateId::Root("0xabcd".to_string())),
            "0xabcd"
        );
    }

    #[test]
    fn test_block_id_conversion() {
        let client = BeaconApiClient::new("http://localhost:5052").unwrap();

        assert_eq!(client.block_id_to_path(&BlockId::Head), "head");
        assert_eq!(client.block_id_to_path(&BlockId::Genesis), "genesis");
        assert_eq!(client.block_id_to_path(&BlockId::Finalized), "finalized");
        assert_eq!(
            client.block_id_to_path(&BlockId::Slot("12345".to_string())),
            "12345"
        );
    }

    #[test]
    fn test_validator_id_conversion() {
        let client = BeaconApiClient::new("http://localhost:5052").unwrap();

        assert_eq!(
            client.validator_id_to_path(&ValidatorId::Index("12345".to_string())),
            "12345"
        );
        assert_eq!(
            client.validator_id_to_path(&ValidatorId::PublicKey("0xabcd".to_string())),
            "0xabcd"
        );
    }
}