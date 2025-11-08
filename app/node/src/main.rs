mod inbound;
mod infrastructure;

use inbound::jsonrpc::EthJsonRpcHandler;
use inbound::server::run_server;
use infrastructure::mock_repository::MockEthereumRepository;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志追踪
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "node=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 基础设施层 - 创建仓储
    let repository = Arc::new(MockEthereumRepository::new());

    // 用例层 - 创建 RPC 处理器
    let rpc_handler = Arc::new(EthJsonRpcHandler::new(repository));

    // 接口层 - 运行 HTTP 服务器
    let host = "127.0.0.1";
    let port = 8545; // 标准以太坊 RPC 端口

    println!("🚀 RustEth 节点启动中...");
    println!("📡 以太坊 JSON-RPC 服务器监听地址：http://{}:{}", host, port);
    println!("🏥 健康检查：http://{}:{}/health", host, port);
    println!("\n💡 示例请求：");
    println!(r#"curl -X POST http://{}:{} -H "Content-Type: application/json" --data '{{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}}'"#, host, port);

    run_server(host, port, rpc_handler).await?;

    Ok(())
}
