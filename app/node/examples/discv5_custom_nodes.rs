//! Discv5 自定义节点示例
//!
//! 演示如何手动指定启动节点
//!
//! 运行方式:
//! ```bash
//! cargo run --example discv5_custom_nodes
//! ```

use node::inbound::client::{Discv5Client, DiscoveryConfig};
use std::time::Duration;
use tracing::{info, warn, Level};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("=== Discv5 自定义节点示例 ===");
    info!("");
    info!("注意: 此示例演示如何手动添加启动节点");
    info!("由于当前使用空的启动节点列表，服务会启动但无法发现节点");
    info!("");

    // 方法1: 使用默认配置（无启动节点）
    info!("方法1: 使用默认配置");
    let config1 = DiscoveryConfig::default();
    let client1 = Discv5Client::new(config1).await?;
    info!("✓ 客户端创建成功（无启动节点）");
    info!("  本地 Node ID: {}", client1.local_enr().await.node_id());
    info!("");

    // 方法2: 手动指定启动节点（示例 - 需要替换为真实 ENR）
    info!("方法2: 手动指定启动节点");
    info!("如需添加真实节点，请替换以下 ENR：");

    let custom_bootnodes = vec![
        // 这里添加你的自定义启动节点 ENR
        // 示例格式: "enr:-IS4Q...".to_string(),
    ];

    if custom_bootnodes.is_empty() {
        warn!("  ⚠ 未配置任何启动节点");
        warn!("  提示: 在生产环境中，你应该:");
        warn!("    1. 从可靠来源获取最新的 ENR");
        warn!("    2. 运行自己的启动节点");
        warn!("    3. 从已知节点获取更多对等节点");
    }

    let config2 = DiscoveryConfig::with_bootnodes(9001, custom_bootnodes);
    let client2 = Discv5Client::new(config2).await?;
    info!("✓ 自定义客户端创建成功");
    info!("  监听端口: 9001");
    info!("");

    // 方法3: 完全自定义配置
    info!("方法3: 完全自定义配置");
    let config3 = DiscoveryConfig {
        listen_addr: "0.0.0.0".parse().unwrap(),
        listen_port: 9002,
        bootnodes: vec![],
        enable_ipv6: false,
        query_parallelism: 5, // 增加并发数
        query_timeout: Duration::from_secs(120), // 更长的超时
    };

    let client3 = Discv5Client::new(config3).await?;
    info!("✓ 高级自定义客户端创建成功");
    info!("  查询并发数: 5");
    info!("  查询超时: 120秒");
    info!("");

    // 获取 ENR 信息
    info!("=== 本地节点信息 ===");
    let enr1 = client1.local_enr().await;
    info!("节点1 ENR: {}", enr1.to_base64());
    info!("节点1 ID:  {}", enr1.node_id());

    if let Some(socket) = enr1.udp4_socket() {
        info!("节点1 Socket: {}", socket);
    }
    info!("");

    // 演示服务状态
    info!("=== 服务状态 ===");
    info!("节点1 连接数: {}", client1.connected_peers().await);
    info!("节点2 连接数: {}", client2.connected_peers().await);
    info!("节点3 连接数: {}", client3.connected_peers().await);
    info!("");

    info!("=== 如何获取启动节点 ===");
    info!("1. 从以太坊客户端获取:");
    info!("   - Geth: https://github.com/ethereum/go-ethereum/blob/master/params/bootnodes.go");
    info!("   - 注意: 需要执行层的 discv5 节点，而非共识层节点");
    info!("");
    info!("2. 运行自己的启动节点:");
    info!("   cargo run --example discv5_custom_nodes");
    info!("   # 复制输出的 ENR，添加到其他节点的配置中");
    info!("");
    info!("3. 从已连接的节点获取更多对等节点:");
    info!("   let nodes = client.find_random_nodes(10).await;");
    info!("");

    info!("示例运行完毕！");
    info!("所有服务均已正常启动（无 ServiceNotStarted 错误）");

    Ok(())
}
