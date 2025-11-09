//! Discv5 本地节点示例
//!
//! 用于创建本地测试网络，验证节点发现功能
//!
//! 使用方式:
//! ```bash
//! # 终端1: 启动第一个节点（作为启动节点）
//! cargo run --example discv5_local_node -- 9000
//!
//! # 复制终端1输出的 ENR，然后在终端2运行:
//! cargo run --example discv5_local_node -- 9001 "enr:-IS4Q..."
//! ```

use node::inbound::client::{Discv5Client, DiscoveryConfig};
use std::env;
use std::time::Duration;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // 解析命令行参数
    let args: Vec<String> = env::args().collect();

    let port = if args.len() > 1 {
        args[1].parse::<u16>().unwrap_or(9000)
    } else {
        9000
    };

    let bootnodes = if args.len() > 2 {
        vec![args[2].clone()]
    } else {
        vec![]
    };

    info!("═══════════════════════════════════════");
    info!("  Discv5 本地测试节点");
    info!("═══════════════════════════════════════");
    info!("监听端口: {}", port);
    info!("启动节点数量: {}", bootnodes.len());
    info!("");

    // 创建配置
    let config = DiscoveryConfig::with_bootnodes(port, bootnodes.clone());

    // 创建客户端
    let client = Discv5Client::new(config).await?;

    // 显示本地节点信息
    let local_enr = client.local_enr().await;
    info!("✓ 节点已启动!");
    info!("");
    info!("═══════════════════════════════════════");
    info!("本地节点 ENR (用于其他节点连接):");
    info!("{}", local_enr.to_base64());
    info!("═══════════════════════════════════════");
    info!("Node ID: {}", local_enr.node_id());
    if let Some(socket) = local_enr.udp4_socket() {
        info!("Socket: {}", socket);
    }
    info!("");

    if bootnodes.is_empty() {
        info!("🔷 这是第一个节点 (启动节点)");
        info!("");
        info!("下一步: 在另一个终端运行第二个节点:");
        info!("cargo run --example discv5_local_node -- 9001 \"{}\"", local_enr.to_base64());
        info!("");
    } else {
        info!("🔷 这是第二个节点");
        info!("将连接到: {} 个启动节点", bootnodes.len());
        info!("");
    }

    // 启动节点发现
    client.start_discovery().await?;

    info!("节点发现已启动，监控中...");
    info!("");

    // 持续监控
    let mut last_count = 0;
    for i in 0..30 {
        tokio::time::sleep(Duration::from_secs(2)).await;

        let discovered = client.get_discovered_nodes().await;
        let peers = client.connected_peers().await;

        if discovered.len() != last_count {
            info!(
                "[{:3}秒] 📊 发现节点: {}, 连接对等节点: {}",
                i * 2,
                discovered.len(),
                peers
            );

            if !discovered.is_empty() {
                for (idx, node) in discovered.iter().enumerate() {
                    info!(
                        "  ✓ [{}] Node: {}, Bootnode: {}, Socket: {:?}",
                        idx + 1,
                        node.node_id,
                        node.is_bootnode,
                        node.socket_addr
                    );
                }
                info!("");
            }

            last_count = discovered.len();
        }
    }

    info!("");
    info!("═══════════════════════════════════════");
    info!("节点统计 (60秒后):");
    info!("═══════════════════════════════════════");
    let discovered = client.get_discovered_nodes().await;
    info!("总共发现: {} 个节点", discovered.len());
    info!("当前连接: {} 个对等节点", client.connected_peers().await);
    info!("");

    if discovered.is_empty() && !bootnodes.is_empty() {
        info!("⚠ 未发现节点，可能原因:");
        info!("  1. 启动节点未运行");
        info!("  2. ENR 不正确");
        info!("  3. 端口被防火墙阻止");
    } else if !discovered.is_empty() {
        info!("✅ 节点发现功能正常工作!");
        info!("");
        info!("发现的节点:");
        for (idx, node) in discovered.iter().enumerate() {
            info!("  [{}] {}", idx + 1, node.node_id);
        }
    }

    info!("");
    info!("按 Ctrl+C 退出...");

    // 保持运行
    tokio::signal::ctrl_c().await?;
    info!("正在关闭...");

    Ok(())
}
