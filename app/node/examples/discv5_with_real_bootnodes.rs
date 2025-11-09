//! Discv5 使用真实启动节点示例
//!
//! 演示如何连接到真实的以太坊网络节点
//!
//! 运行方式:
//! ```bash
//! cargo run --example discv5_with_real_bootnodes
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

    info!("=== 使用真实启动节点的 Discv5 示例 ===");
    info!("");

    // 方法1: 使用 enode 格式的启动节点（需要转换为 ENR）
    info!("注意: 以太坊主网的启动节点主要使用 enode 格式");
    info!("Discv5 需要 ENR 格式的启动节点");
    info!("");

    // 方法2: 手动指定一些公开的 ENR 启动节点
    // 这些是示例 ENR，可能需要替换为最新的有效节点
    let bootnodes = vec![
        // 注意: 以下是示例格式，实际使用时需要从可靠来源获取最新的 ENR
        // 可以从以下来源获取:
        // 1. 运行自己的节点并获取其 ENR
        // 2. 从其他以太坊客户端获取
        // 3. 从社区维护的列表获取
    ];

    if bootnodes.is_empty() {
        warn!("⚠ 当前未配置任何启动节点");
        warn!("");
        warn!("要发现以太坊网络中的节点，你需要:");
        warn!("");
        warn!("方法1 - 运行两个本地节点测试:");
        warn!("  终端1: cargo run --example discv5_local_node -- 9000");
        warn!("  终端2: cargo run --example discv5_local_node -- 9001 <node1_enr>");
        warn!("");
        warn!("方法2 - 从 Geth 获取 enode 并转换为 ENR:");
        warn!("  1. 查看 Geth bootnodes:");
        warn!("     https://github.com/ethereum/go-ethereum/blob/master/params/bootnodes.go");
        warn!("  2. 使用工具转换 enode 为 ENR");
        warn!("");
        warn!("方法3 - 连接到已知的公共节点:");
        warn!("  获取公共节点的 ENR 并添加到 bootnodes");
        warn!("");
    }

    // 创建配置
    let config = DiscoveryConfig::with_bootnodes(9000, bootnodes);

    info!("创建 Discv5 客户端...");
    let client = Discv5Client::new(config).await?;

    // 显示本地节点信息
    let local_enr = client.local_enr().await;
    info!("本地节点已启动!");
    info!("═══════════════════════════════════════");
    info!("本地 ENR (可分享给其他节点):");
    info!("{}", local_enr.to_base64());
    info!("═══════════════════════════════════════");
    info!("本地 Node ID: {}", local_enr.node_id());

    if let Some(socket) = local_enr.udp4_socket() {
        info!("监听地址: {}", socket);
    }
    info!("");

    // 启动节点发现
    info!("启动节点发现进程...");
    client.start_discovery().await?;

    // 等待一段时间
    info!("等待 60 秒，观察节点发现情况...");
    for i in 1..=6 {
        tokio::time::sleep(Duration::from_secs(10)).await;

        let discovered = client.get_discovered_nodes().await;
        let peers = client.connected_peers().await;

        info!(
            "[{}0秒] 已发现: {} 个节点, 连接: {} 个对等节点",
            i, discovered.len(), peers
        );

        if !discovered.is_empty() {
            info!("  ✓ 发现的节点示例:");
            for (idx, node) in discovered.iter().take(3).enumerate() {
                info!(
                    "    [{}] ID: {}, Socket: {:?}",
                    idx + 1,
                    node.node_id,
                    node.socket_addr
                );
            }
        }
    }

    // 最终统计
    info!("");
    info!("═══════════════════════════════════════");
    info!("最终统计:");
    let discovered = client.get_discovered_nodes().await;
    info!("总共发现: {} 个节点", discovered.len());
    info!("当前连接: {} 个对等节点", client.connected_peers().await);
    info!("═══════════════════════════════════════");

    if discovered.is_empty() {
        warn!("");
        warn!("未发现任何节点，可能的原因:");
        warn!("1. 启动节点列表为空或无效");
        warn!("2. 网络连接问题");
        warn!("3. 防火墙阻止 UDP 端口 9000");
        warn!("4. 启动节点已离线");
        warn!("");
        warn!("建议: 运行本地测试网络（见下方）");
    }

    info!("");
    info!("示例运行完毕!");
    Ok(())
}
