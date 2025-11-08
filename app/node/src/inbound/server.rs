//! 以太坊 JSON-RPC 的 HTTP 服务器实现
//!
//! 使用 Axum 构建的低延迟 HTTP 服务器，配置经过优化

use crate::inbound::jsonrpc::{EthJsonRpcHandler, JsonRpcRequest};
use axum::{
    extract::State,
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

/// HTTP 服务器状态
#[derive(Clone)]
pub struct ServerState {
    pub rpc_handler: Arc<EthJsonRpcHandler>,
}

/// 创建并配置 HTTP 服务器
pub fn create_server(rpc_handler: Arc<EthJsonRpcHandler>) -> Router {
    let state = ServerState { rpc_handler };

    // 为以太坊客户端配置 CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .route("/", post(handle_rpc_request))
        .route("/health", axum::routing::get(health_check))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors),
        )
        .with_state(state)
}

/// RPC 请求主处理器
async fn handle_rpc_request(
    State(state): State<ServerState>,
    Json(request): Json<JsonRpcRequest>,
) -> Response {
    let response = state.rpc_handler.handle(request).await;
    Json(response).into_response()
}

/// 健康检查端点
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// 运行服务器（优化配置）
pub async fn run_server(host: &str, port: u16, rpc_handler: Arc<EthJsonRpcHandler>) -> anyhow::Result<()> {
    let app = create_server(rpc_handler);
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("以太坊 JSON-RPC 服务器启动于 {}", addr);
    info!("健康检查可访问 http://{}/health", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn test_health_check() {
        // 测试健康检查端点
        let response = health_check().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
