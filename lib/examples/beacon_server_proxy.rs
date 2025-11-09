//! Beacon API 代理服务器示例
//!
//! 此示例演示如何使用 BeaconApiServer 和 BeaconApiClient 创建一个代理服务器,
//! 将本地端点转发到远程 Beacon Node。
//!
//! # 使用方法
//!
//! ```bash
//! # 运行代理服务器
//! cargo run --example beacon_server_proxy
//!
//! # 在另一个终端测试
//! curl http://127.0.0.1:8080/eth/v1/beacon/genesis
//! ```

use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // 从环境变量读取配置,或使用默认值
    let beacon_node_url = std::env::var("BEACON_NODE_URL")
        .unwrap_or_else(|_| "http://localhost:5052".to_string());

    let server_addr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    tracing::info!("Connecting to Beacon Node at: {}", beacon_node_url);

    // 创建 BeaconApiClient 连接到远程 Beacon Node
    let beacon_client = Arc::new(
        lib::adapter::outbound::beacon_api_client::BeaconApiClient::new(&beacon_node_url)?
    );

    // 创建 BeaconApiServer (代理模式)
    let server = lib::adapter::inbound::beacon_api_server::BeaconApiServer::new(beacon_client);

    // 构建 Axum Router
    let app = server.router();

    // 绑定 TCP 监听器
    let listener = tokio::net::TcpListener::bind(&server_addr).await?;

    tracing::info!("Beacon API Proxy Server listening on http://{}", server_addr);
    tracing::info!("Available endpoints:");
    tracing::info!("  - GET  /eth/v1/beacon/genesis");
    tracing::info!("  - GET  /eth/v1/node/version");
    tracing::info!("  - GET  /eth/v1/node/health");
    tracing::info!("  - GET  /eth/v1/node/syncing");
    tracing::info!("  - GET  /eth/v1/beacon/headers/head");
    tracing::info!("  - GET  /eth/v2/beacon/blocks/head");
    tracing::info!("  - ... and more (see Beacon API spec)");

    // 启动 HTTP 服务器
    axum::serve(listener, app).await?;

    Ok(())
}
