use node::service::command_dispatcher::CommandDispatcher;
use node::inbound::json_rpc::EthJsonRpcHandler;
use node::inbound::server::run_server;
use node::infrastructure::mock_repository::MockEthereumRepository;
use node::service::ethereum_service_impl::EthereumServiceImpl;
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

    println!("🏗️  构建 Clean Architecture 依赖链...\n");

    // 基础设施层 - 创建数据仓储
    println!("📦 [Infrastructure] MockEthereumRepository");
    let repo = MockEthereumRepository::new();

    // 服务层 - 创建业务服务
    println!("🔧 [Service] EthereumServiceImpl");
    let service = Arc::new(EthereumServiceImpl::new(repo));

    // 领域层 - 创建命令分发器
    println!("🚀 [Domain] CommandDispatcher");
    let dispatcher = CommandDispatcher::new(service);

    // 接口层 - 创建 JSON-RPC 处理器
    println!("🌐 [Interface] EthJsonRpcHandler");
    let rpc_handler = EthJsonRpcHandler::new(dispatcher);

    // 启动 HTTP 服务器
    let host = "127.0.0.1";
    let port = 8545;

    println!("\n✅ 依赖注入完成！\n");
    println!("🚀 RustEth 节点启动中...");
    println!("📡 JSON-RPC: http://{}:{}", host, port);
    println!("🏥 Health: http://{}:{}/health", host, port);
    println!("\n💡 测试命令：");
    println!(
        r#"curl -X POST http://{}:{} -H "Content-Type: application/json" --data '{{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}}'"#,
        host, port
    );
    println!("\n⚡ 架构：");
    println!("   ✓ Clean Architecture 三层架构");
    println!("   ✓ CQRS 命令查询分离");
    println!("   ✓ 极简设计，无过度抽象");

    run_server(host, port, rpc_handler).await?;

    Ok(())
}
