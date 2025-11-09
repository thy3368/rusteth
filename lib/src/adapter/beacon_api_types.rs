//! Beacon API 通用类型定义
//!
//! 本模块包含客户端和服务端共享的数据结构,避免重复定义。
//!
//! # 共享类型
//! - **ApiResponse**: 标准 Beacon API 响应格式
//! - **ApiError**: API 错误响应格式
//! - **RootResponse**: Root 响应格式
//! - **查询参数类型**: 用于构建请求查询字符串

use serde::{Deserialize, Serialize};

use crate::domain::service::beacon_api::{Epoch, Root, Slot, ValidatorStatus};

// ================================================================================================
// API 响应包装类型 (Response Wrappers)
// ================================================================================================

/// 标准 Beacon API 响应格式
///
/// 所有成功的 Beacon API 响应都遵循此格式:
/// ```json
/// {
///   "data": <实际数据>
/// }
/// ```
///
/// # 性能优化
/// - 泛型设计,零成本抽象
/// - Serialize/Deserialize 自动派生
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

impl<T> ApiResponse<T> {
    /// 创建新的 API 响应
    ///
    /// # 参数
    /// - `data`: 响应数据
    ///
    /// # 返回
    /// 包装后的响应
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

// ================================================================================================
// API 错误响应格式
// ================================================================================================

/// API 错误响应格式
///
/// 遵循 Beacon API 标准错误响应格式:
/// ```json
/// {
///   "code": <HTTP 状态码>,
///   "message": "<错误消息>",
///   "stacktraces": []
/// }
/// ```
///
/// # 设计说明
/// - `stacktraces` 字段为可选,生产环境通常为空
/// - `code` 字段对应 HTTP 状态码(400, 404, 500 等)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiError {
    /// HTTP 状态码
    pub code: u16,
    /// 错误消息
    pub message: String,
    /// 堆栈跟踪(通常为空)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stacktraces: Vec<String>,
}

impl ApiError {
    /// 创建新的 API 错误
    ///
    /// # 参数
    /// - `code`: HTTP 状态码
    /// - `message`: 错误消息
    ///
    /// # 返回
    /// 错误响应对象
    pub fn new(code: u16, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            stacktraces: Vec::new(),
        }
    }

    /// 创建带堆栈跟踪的错误
    ///
    /// # 参数
    /// - `code`: HTTP 状态码
    /// - `message`: 错误消息
    /// - `stacktraces`: 堆栈跟踪列表
    ///
    /// # 返回
    /// 错误响应对象
    pub fn with_stacktraces(code: u16, message: impl Into<String>, stacktraces: Vec<String>) -> Self {
        Self {
            code,
            message: message.into(),
            stacktraces,
        }
    }
}

// ================================================================================================
// 特定响应类型
// ================================================================================================

/// Root 响应格式
///
/// 用于返回单个 root 哈希值的端点:
/// - GET /eth/v1/beacon/blocks/{block_id}/root
/// - GET /eth/v1/beacon/states/{state_id}/root
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RootResponse {
    pub root: Root,
}

impl RootResponse {
    /// 创建新的 Root 响应
    pub fn new(root: Root) -> Self {
        Self { root }
    }
}

// ================================================================================================
// 查询参数类型 (Query Parameter Types)
// ================================================================================================

/// 区块头查询参数
///
/// 用于端点: GET /eth/v1/beacon/headers?slot=<slot>&parent_root=<root>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockHeadersQuery {
    /// Slot 编号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<Slot>,
    /// 父区块根哈希
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_root: Option<Root>,
}

/// 验证者查询参数
///
/// 用于端点: GET /eth/v1/beacon/states/{state_id}/validators?id=<id>&status=<status>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidatorsQuery {
    /// 验证者 ID 列表
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub id: Vec<String>,
    /// 验证者状态列表
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub status: Vec<ValidatorStatus>,
}

/// 委员会查询参数
///
/// 用于端点: GET /eth/v1/beacon/states/{state_id}/committees?epoch=<epoch>&index=<index>&slot=<slot>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommitteesQuery {
    /// Epoch 编号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epoch: Option<Epoch>,
    /// 委员会索引
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<String>,
    /// Slot 编号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<Slot>,
}

/// 同步委员会查询参数
///
/// 用于端点: GET /eth/v1/beacon/states/{state_id}/sync_committees?epoch=<epoch>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncCommitteesQuery {
    /// Epoch 编号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epoch: Option<Epoch>,
}

/// 证明池查询参数
///
/// 用于端点: GET /eth/v1/beacon/pool/attestations?slot=<slot>&committee_index=<index>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttestationsQuery {
    /// Slot 编号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<Slot>,
    /// 委员会索引
    #[serde(skip_serializing_if = "Option::is_none")]
    pub committee_index: Option<String>,
}

/// 验证者余额查询参数
///
/// 用于端点: GET /eth/v1/beacon/states/{state_id}/validator_balances?id=<id>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidatorBalancesQuery {
    /// 验证者 ID 列表
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub id: Vec<String>,
}

// ================================================================================================
// 请求体类型 (Request Body Types)
// ================================================================================================

/// POST 验证者请求体
///
/// 用于端点: POST /eth/v1/beacon/states/{state_id}/validators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostValidatorsRequest {
    /// 验证者 ID 列表
    pub ids: Vec<String>,
    /// 验证者状态列表(可选)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statuses: Option<Vec<ValidatorStatus>>,
}

// ================================================================================================
// 测试模块
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_creation() {
        let response = ApiResponse::new(42);
        assert_eq!(response.data, 42);
    }

    #[test]
    fn test_api_response_serialization() {
        let response = ApiResponse::new("test_data".to_string());
        let json = serde_json::to_string(&response).unwrap();
        assert_eq!(json, r#"{"data":"test_data"}"#);
    }

    #[test]
    fn test_api_error_creation() {
        let error = ApiError::new(404, "Not found");
        assert_eq!(error.code, 404);
        assert_eq!(error.message, "Not found");
        assert!(error.stacktraces.is_empty());
    }

    #[test]
    fn test_api_error_serialization() {
        let error = ApiError::new(500, "Internal error");
        let json = serde_json::to_string(&error).unwrap();
        // stacktraces 应该被跳过(因为是空的)
        assert_eq!(json, r#"{"code":500,"message":"Internal error"}"#);
    }

    #[test]
    fn test_api_error_with_stacktraces() {
        let error = ApiError::with_stacktraces(
            500,
            "Internal error",
            vec!["line1".to_string(), "line2".to_string()],
        );
        assert_eq!(error.stacktraces.len(), 2);

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("stacktraces"));
    }

    #[test]
    fn test_root_response() {
        let response = RootResponse::new("0x1234".to_string());
        assert_eq!(response.root, "0x1234");
    }

    #[test]
    fn test_validators_query_default() {
        let query = ValidatorsQuery {
            id: vec![],
            status: vec![],
        };

        let json = serde_json::to_string(&query).unwrap();
        // 空 Vec 应该被跳过
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_post_validators_request() {
        let request = PostValidatorsRequest {
            ids: vec!["123".to_string(), "456".to_string()],
            statuses: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("ids"));
        assert!(!json.contains("statuses")); // None 应该被跳过
    }
}
