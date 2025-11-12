//! JSON-RPC 2.0 协议类型定义
//!
//! 本模块定义符合 JSON-RPC 2.0 规范的核心类型。
//! 参考：https://www.jsonrpc.org/specification

use serde::{Deserialize, Serialize};

// ============================================================================
// JSON-RPC 2.0 核心类型
// ============================================================================

/// JSON-RPC 2.0 请求结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: RequestId,
}

/// JSON-RPC 2.0 响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcResponse {
    Success {
        jsonrpc: String,
        result: serde_json::Value,
        id: RequestId,
    },
    Error {
        jsonrpc: String,
        error: JsonRpcError,
        id: RequestId,
    },
}

/// 请求 ID（可以是字符串、数字或 null）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum RequestId {
    Number(u64),
    String(String),
    Null,
}

/// JSON-RPC 错误结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// 标准 JSON-RPC 错误代码（EIP-1474 规范）
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700; // 解析错误
    pub const INVALID_REQUEST: i32 = -32600; // 无效请求
    pub const METHOD_NOT_FOUND: i32 = -32601; // 方法未找到
    pub const INVALID_PARAMS: i32 = -32602; // 无效参数
    pub const INTERNAL_ERROR: i32 = -32603; // 内部错误
    pub const SERVER_ERROR: i32 = -32000; // 服务器错误
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_serialization() {
        // 测试请求 ID 的序列化
        let id_num = RequestId::Number(1);
        let json = serde_json::to_string(&id_num).unwrap();
        assert_eq!(json, "1");

        let id_str = RequestId::String("test".to_string());
        let json = serde_json::to_string(&id_str).unwrap();
        assert_eq!(json, "\"test\"");
    }

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse::Success {
            jsonrpc: "2.0".to_string(),
            result: serde_json::json!({"result": "ok"}),
            id: RequestId::Number(1),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("result"));
    }

    #[test]
    fn test_json_rpc_response_error() {
        let response = JsonRpcResponse::Error {
            jsonrpc: "2.0".to_string(),
            error: JsonRpcError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            },
            id: RequestId::Number(1),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("error"));
    }
}
