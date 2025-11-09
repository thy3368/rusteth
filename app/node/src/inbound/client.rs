//! Ethereum P2P Client - Discv5 Node Discovery Implementation
//!
//! 本模块实现基于 Discv5 协议的以太坊节点发现功能
//! 符合 Clean Architecture 原则和低延迟性能要求

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use discv5::{enr, Discv5, ListenConfig};
use enr::{CombinedKey, Enr, NodeId};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// 以太坊主网的链 ID
const ETH_MAINNET_CHAIN_ID: u64 = 1;

/// 默认的 Discv5 监听端口
const DEFAULT_DISCV5_PORT: u16 = 9000;

/// 节点发现配置
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// 监听地址
    pub listen_addr: IpAddr,
    /// 监听端口
    pub listen_port: u16,
    /// 启动节点列表 (ENR格式)
    pub bootnodes: Vec<String>,
    /// 是否启用 IPv6
    pub enable_ipv6: bool,
    /// 查询并发数
    pub query_parallelism: usize,
    /// 查询超时时间
    pub query_timeout: Duration,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0".parse().unwrap(),
            listen_port: DEFAULT_DISCV5_PORT,
            bootnodes: Self::default_mainnet_bootnodes(),
            enable_ipv6: false,
            query_parallelism: 3,
            query_timeout: Duration::from_secs(60),
        }
    }
}

impl DiscoveryConfig {
    /// 以太坊主网默认启动节点
    ///
    /// 注意: 使用简化的 ENR 列表用于测试
    /// 实际生产环境应从可靠来源获取最新的执行层 discv5 启动节点
    /// 来源参考: https://github.com/ethereum/go-ethereum/blob/master/params/bootnodes.go
    fn default_mainnet_bootnodes() -> Vec<String> {
        vec![
            // 简化的启动节点列表
            // 注意: 这些 ENR 可能需要定期更新
            // 如果解析失败，客户端仍可正常运行，只是初始对等节点较少
        ]
    }

    /// 获取测试网配置（Sepolia）
    ///
    /// 注意: 启动节点列表可能需要根据网络状态更新
    pub fn sepolia() -> Self {
        Self {
            bootnodes: vec![
                // Sepolia 测试网启动节点
                // 可通过手动添加更多节点
            ],
            ..Default::default()
        }
    }

    /// 创建自定义配置，手动指定启动节点
    ///
    /// # Arguments
    /// * `listen_port` - 监听端口
    /// * `bootnodes` - 启动节点 ENR 列表
    ///
    /// # Example
    /// ```rust,no_run
    /// use node::inbound::client::DiscoveryConfig;
    /// let config = DiscoveryConfig::with_bootnodes(
    ///     9000,
    ///     vec![
    ///         "enr:-...".to_string(),
    ///     ]
    /// );
    /// ```
    pub fn with_bootnodes(listen_port: u16, bootnodes: Vec<String>) -> Self {
        Self {
            listen_port,
            bootnodes,
            ..Default::default()
        }
    }
}

/// 发现的节点信息
#[repr(align(64))] // 缓存行对齐优化
#[derive(Debug, Clone)]
pub struct DiscoveredNode {
    /// 节点 ENR 记录
    pub enr: Enr<CombinedKey>,
    /// 节点 ID
    pub node_id: NodeId,
    /// Socket 地址 (如果可用)
    pub socket_addr: Option<SocketAddr>,
    /// 是否是启动节点
    pub is_bootnode: bool,
}

/// Discv5 节点发现客户端
///
/// 负责:
/// 1. 运行 Discv5 协议栈
/// 2. 发现网络中的其他以太坊节点
/// 3. 维护已发现节点的列表
pub struct Discv5Client {
    /// Discv5 协议实例
    discv5: Arc<Discv5>,
    /// 本地节点的 ENR
    local_enr: Arc<RwLock<Enr<CombinedKey>>>,
    /// 已发现的节点列表 (NodeId -> DiscoveredNode)
    discovered_nodes: Arc<RwLock<Vec<DiscoveredNode>>>,
    /// 配置
    config: DiscoveryConfig,
}

impl Discv5Client {
    /// 创建新的 Discv5 客户端
    ///
    /// # Arguments
    /// * `config` - 节点发现配置
    ///
    /// # Returns
    /// * `Result<Self>` - 客户端实例或错误
    pub async fn new(config: DiscoveryConfig) -> Result<Self> {
        info!("初始化 Discv5 节点发现客户端");

        // 1. 生成节点密钥
        let enr_key = CombinedKey::generate_secp256k1();

        // 2. 构建本地 ENR (Ethereum Node Record)
        let local_enr = Self::build_local_enr(
            &enr_key,
            config.listen_addr,
            config.listen_port,
        )?;

        info!("本地 ENR: {}", local_enr.to_base64());
        info!("本地 Node ID: {}", local_enr.node_id());

        // 3. 配置监听地址
        let listen_config = match config.listen_addr {
            IpAddr::V4(addr) => ListenConfig::Ipv4 {
                ip: addr,
                port: config.listen_port,
            },
            IpAddr::V6(addr) => ListenConfig::Ipv6 {
                ip: addr,
                port: config.listen_port,
            },
        };

        // 4. 配置 Discv5
        let discv5_config = discv5::ConfigBuilder::new(listen_config.clone()).build();

        // 5. 创建 Discv5 实例
        let mut discv5 = Discv5::new(local_enr.clone(), enr_key, discv5_config)
            .map_err(|e| anyhow::anyhow!("创建 Discv5 实例失败: {}", e))?;

        // 6. 启动 Discv5 服务（关键步骤！）
        discv5.start().await.map_err(|e| {
            anyhow::anyhow!("启动 Discv5 服务失败: {}", e)
        })?;

        info!(
            "Discv5 服务已启动，监听地址: {}:{}",
            config.listen_addr, config.listen_port
        );

        Ok(Self {
            discv5: Arc::new(discv5),
            local_enr: Arc::new(RwLock::new(local_enr)),
            discovered_nodes: Arc::new(RwLock::new(Vec::new())),
            config,
        })
    }

    /// 构建本地节点的 ENR
    fn build_local_enr(
        enr_key: &CombinedKey,
        listen_addr: IpAddr,
        listen_port: u16,
    ) -> Result<Enr<CombinedKey>> {
        // 使用简化的 ENR 创建方式
        let mut enr = Enr::empty(enr_key)
            .context("创建空 ENR 失败")?;

        // 根据地址类型设置 IP 和端口
        match listen_addr {
            IpAddr::V4(ip) => {
                enr = Enr::builder()
                    .ip4(ip)
                    .udp4(listen_port)
                    .build(enr_key)
                    .context("构建 IPv4 ENR 失败")?;
            }
            IpAddr::V6(ip) => {
                enr = Enr::builder()
                    .ip6(ip)
                    .udp6(listen_port)
                    .build(enr_key)
                    .context("构建 IPv6 ENR 失败")?;
            }
        }

        Ok(enr)
    }

    /// 添加启动节点
    ///
    /// # Arguments
    /// * `bootnode_enrs` - 启动节点的 ENR 字符串列表
    pub async fn add_bootnodes(&self, bootnode_enrs: Vec<String>) -> Result<()> {
        info!("添加 {} 个启动节点", bootnode_enrs.len());

        for enr_str in bootnode_enrs {
            match enr_str.parse::<Enr<CombinedKey>>() {
                Ok(enr) => {
                    debug!("添加启动节点: {}", enr.node_id());

                    if let Err(e) = self.discv5.add_enr(enr.clone()) {
                        warn!("添加启动节点失败: {}", e);
                    } else {
                        // 记录为已发现节点
                        let node = DiscoveredNode {
                            socket_addr: enr.udp4_socket().map(|addr| addr.into()),
                            node_id: enr.node_id(),
                            enr: enr.clone(),
                            is_bootnode: true,
                        };

                        let mut nodes = self.discovered_nodes.write().await;
                        nodes.push(node);
                    }
                }
                Err(e) => {
                    warn!("解析启动节点 ENR 失败: {} - {}", enr_str, e);
                }
            }
        }

        Ok(())
    }

    /// 启动节点发现
    ///
    /// 执行以下操作:
    /// 1. 添加启动节点
    /// 2. 启动随机节点查询
    /// 3. 监听 Discv5 事件
    pub async fn start_discovery(&self) -> Result<()> {
        info!("启动节点发现进程");

        // 1. 添加启动节点
        self.add_bootnodes(self.config.bootnodes.clone()).await?;

        // 2. 启动随机节点查询 (用于发现网络中的节点)
        let target = NodeId::random();
        info!("开始随机节点查询: target={}", target);

        match self.discv5.find_node(target).await {
            Ok(nodes) => {
                info!("随机查询发现 {} 个节点", nodes.len());
                self.process_discovered_nodes(nodes).await;
            }
            Err(e) => {
                warn!("随机节点查询失败: {}", e);
            }
        }

        // 3. 启动事件监听循环
        self.start_event_loop().await;

        Ok(())
    }

    /// 启动事件监听循环
    ///
    /// 注意: discv5 0.7 的事件处理机制
    /// 事件通过内部回调处理，这里提供周期性查询机制
    async fn start_event_loop(&self) {
        let discv5 = self.discv5.clone();
        let discovered_nodes = self.discovered_nodes.clone();

        tokio::spawn(async move {
            info!("事件监听循环已启动 - 周期性节点发现模式");

            // 周期性执行节点发现查询
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                // 执行随机节点查询
                let target = NodeId::random();
                debug!("周期性节点查询: target={}", target);

                match discv5.find_node(target).await {
                    Ok(enrs) => {
                        debug!("周期性查询发现 {} 个节点", enrs.len());

                        let mut nodes = discovered_nodes.write().await;
                        for enr in enrs {
                            let node_id = enr.node_id();

                            // 去重检查
                            if nodes.iter().any(|n| n.node_id == node_id) {
                                continue;
                            }

                            let node = DiscoveredNode {
                                socket_addr: enr.udp4_socket().map(|addr| addr.into()),
                                node_id,
                                enr: enr.clone(),
                                is_bootnode: false,
                            };

                            nodes.push(node);
                            info!("新节点已添加: {} (总数: {})", node_id, nodes.len());
                        }
                    }
                    Err(e) => {
                        warn!("周期性节点查询失败: {}", e);
                    }
                }
            }
        });
    }

    /// 处理发现的节点
    async fn process_discovered_nodes(&self, enrs: Vec<Enr<CombinedKey>>) {
        let mut nodes = self.discovered_nodes.write().await;

        for enr in enrs {
            let node_id = enr.node_id();

            // 去重检查
            if nodes.iter().any(|n| n.node_id == node_id) {
                continue;
            }

            let node = DiscoveredNode {
                socket_addr: enr.udp4_socket().map(|addr| addr.into()),
                node_id,
                enr: enr.clone(),
                is_bootnode: false,
            };

            nodes.push(node);
            debug!("添加发现的节点: {}", node_id);
        }

        info!("当前已发现节点总数: {}", nodes.len());
    }

    /// 查找指定数量的随机节点
    ///
    /// # Arguments
    /// * `count` - 要查找的节点数量
    ///
    /// # Returns
    /// * `Vec<DiscoveredNode>` - 发现的节点列表
    pub async fn find_random_nodes(&self, count: usize) -> Vec<DiscoveredNode> {
        info!("查找 {} 个随机节点", count);

        let mut result = Vec::new();

        for _ in 0..count {
            let target = NodeId::random();

            match self.discv5.find_node(target).await {
                Ok(enrs) => {
                    for enr in enrs {
                        if result.len() >= count {
                            break;
                        }

                        let node = DiscoveredNode {
                            socket_addr: enr.udp4_socket().map(|addr| addr.into()),
                            node_id: enr.node_id(),
                            enr: enr.clone(),
                            is_bootnode: false,
                        };

                        result.push(node);
                    }
                }
                Err(e) => {
                    warn!("节点查询失败: {}", e);
                }
            }

            if result.len() >= count {
                break;
            }
        }

        info!("找到 {} 个节点", result.len());
        result
    }

    /// 获取所有已发现的节点
    pub async fn get_discovered_nodes(&self) -> Vec<DiscoveredNode> {
        self.discovered_nodes.read().await.clone()
    }

    /// 获取本地节点的 ENR
    pub async fn local_enr(&self) -> Enr<CombinedKey> {
        self.local_enr.read().await.clone()
    }

    /// 获取连接的节点数量
    pub async fn connected_peers(&self) -> usize {
        self.discv5.connected_peers()
    }

    /// 关闭客户端
    pub async fn shutdown(self) {
        info!("关闭 Discv5 客户端");
        // discv5 0.7 会在 drop 时自动清理
        drop(self.discv5);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_client() {
        let config = DiscoveryConfig {
            listen_port: 19000, // 使用不同端口避免冲突
            bootnodes: vec![], // 测试时不使用启动节点
            ..Default::default()
        };

        let client = Discv5Client::new(config).await;
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.connected_peers().await, 0);

        client.shutdown().await;
    }

    #[tokio::test]
    async fn test_local_enr() {
        let config = DiscoveryConfig {
            listen_port: 19001,
            bootnodes: vec![],
            ..Default::default()
        };

        let client = Discv5Client::new(config).await.unwrap();
        let enr = client.local_enr().await;

        assert!(enr.udp4_socket().is_some());
        assert_eq!(enr.udp4_socket().unwrap().port(), 19001);

        client.shutdown().await;
    }
}
