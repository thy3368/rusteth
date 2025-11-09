//! Discv5 节点发现示例
//!
//! 演示如何使用 Discv5Client 发现以太坊网络中的其他节点
//!
//! 运行方式:
//! ```bash
//! cargo run --example discv5_example
//! ```

use node::inbound::client::{Discv5Client, DiscoveryConfig};
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("=== Ethereum Discv5 节点发现示例 ===");

    // 配置节点发现
    let config = DiscoveryConfig {
        listen_addr: "0.0.0.0".parse().unwrap(),
        listen_port: 9000,
        bootnodes: DiscoveryConfig::default().bootnodes, // 使用默认的以太坊主网启动节点
        enable_ipv6: false,
        query_parallelism: 3,
        query_timeout: Duration::from_secs(60),
    };

    info!("创建 Discv5 客户端...");
    let client = Discv5Client::new(config).await?;

    // 显示本地节点信息
    let local_enr = client.local_enr().await;
    info!("本地节点 ENR: {}", local_enr.to_base64());
    info!("本地节点 ID: {}", local_enr.node_id());

    // 启动节点发现
    info!("启动节点发现进程...");
    client.start_discovery().await?;

    // 等待一段时间让节点发现运行
    info!("等待节点发现运行 30 秒...");
    tokio::time::sleep(Duration::from_secs(30)).await;

    // 查看发现的节点
    let discovered = client.get_discovered_nodes().await;
    info!("已发现 {} 个节点:", discovered.len());

    for (i, node) in discovered.iter().enumerate().take(10) {
        info!(
            "  [{}] Node ID: {}, Socket: {:?}, Bootnode: {}",
            i + 1,
            node.node_id,
            node.socket_addr,
            node.is_bootnode
        );
    }

    // 查找额外的随机节点
    info!("查找 5 个额外的随机节点...");
    let random_nodes = client.find_random_nodes(5).await;
    info!("找到 {} 个随机节点", random_nodes.len());

    // 显示连接统计
    info!("当前连接的对等节点数: {}", client.connected_peers().await);

    info!("节点发现示例运行完毕");
    Ok(())
}
