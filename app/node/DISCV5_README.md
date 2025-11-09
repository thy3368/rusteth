# Discv5 节点发现实现

基于 discv5 0.7 实现的以太坊节点发现功能，符合 Clean Architecture 原则和低延迟性能要求。

## 功能特性

### 核心功能

1. **节点发现**: 自动发现以太坊网络中的其他节点
2. **ENR 管理**: 完整的 Ethereum Node Record 支持
3. **启动节点**: 内置以太坊主网和 Sepolia 测试网启动节点
4. **周期性查询**: 自动周期性执行节点发现
5. **缓存行对齐**: `DiscoveredNode` 结构体 64 字节对齐优化

### 性能优化

- **缓存行对齐**: `#[repr(align(64))]` 优化数据结构
- **无锁读写**: 使用 `Arc<RwLock<>>` 实现高效并发访问
- **零拷贝**: 最小化数据拷贝
- **异步设计**: 完全异步实现，避免阻塞

## API 文档

### `Discv5Client`

主要客户端结构体，负责节点发现。

#### 创建客户端

```rust
use node::inbound::client::{Discv5Client, DiscoveryConfig};
use std::time::Duration;

let config = DiscoveryConfig {
    listen_addr: "0.0.0.0".parse().unwrap(),
    listen_port: 9000,
    bootnodes: DiscoveryConfig::default().bootnodes, // 使用默认启动节点
    enable_ipv6: false,
    query_parallelism: 3,
    query_timeout: Duration::from_secs(60),
};

let client = Discv5Client::new(config).await?;
```

#### 启动节点发现

```rust
// 启动节点发现进程
client.start_discovery().await?;

// 等待节点被发现
tokio::time::sleep(Duration::from_secs(30)).await;

// 获取已发现的节点
let nodes = client.get_discovered_nodes().await;
println!("发现了 {} 个节点", nodes.len());
```

#### 主动查找节点

```rust
// 查找指定数量的随机节点
let random_nodes = client.find_random_nodes(10).await;

for node in random_nodes {
    println!("节点 ID: {}", node.node_id);
    println!("Socket 地址: {:?}", node.socket_addr);
    println!("是否启动节点: {}", node.is_bootnode);
}
```

#### 获取本地节点信息

```rust
// 获取本地节点的 ENR
let local_enr = client.local_enr().await;
println!("本地 ENR: {}", local_enr.to_base64());
println!("本地 Node ID: {}", local_enr.node_id());

// 获取连接的对等节点数
let peer_count = client.connected_peers().await;
println!("连接的节点数: {}", peer_count);
```

### `DiscoveryConfig`

节点发现配置结构体。

#### 字段说明

- `listen_addr: IpAddr` - 监听地址 (默认: `0.0.0.0`)
- `listen_port: u16` - 监听端口 (默认: `9000`)
- `bootnodes: Vec<String>` - 启动节点 ENR 列表
- `enable_ipv6: bool` - 是否启用 IPv6 (默认: `false`)
- `query_parallelism: usize` - 查询并发数 (默认: `3`)
- `query_timeout: Duration` - 查询超时时间 (默认: `60s`)

#### 预定义配置

```rust
// 以太坊主网配置（默认）
let mainnet_config = DiscoveryConfig::default();

// Sepolia 测试网配置
let sepolia_config = DiscoveryConfig::sepolia();
```

### `DiscoveredNode`

发现的节点信息结构体。

#### 字段说明

- `enr: Enr<CombinedKey>` - 节点的 ENR 记录
- `node_id: NodeId` - 节点 ID
- `socket_addr: Option<SocketAddr>` - Socket 地址（如果可用）
- `is_bootnode: bool` - 是否是启动节点

## 使用示例

### 基本示例

```rust
use node::inbound::client::{Discv5Client, DiscoveryConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 创建配置
    let config = DiscoveryConfig::default();

    // 2. 创建客户端
    let client = Discv5Client::new(config).await?;

    // 3. 启动节点发现
    client.start_discovery().await?;

    // 4. 等待发现节点
    tokio::time::sleep(Duration::from_secs(30)).await;

    // 5. 查看发现的节点
    let nodes = client.get_discovered_nodes().await;
    println!("发现了 {} 个节点", nodes.len());

    Ok(())
}
```

### 运行示例程序

项目包含一个完整的示例程序：

```bash
# 编译并运行示例
cargo run --example discv5_example

# 仅编译示例
cargo build --example discv5_example
```

### 自定义配置示例

```rust
use std::net::IpAddr;

let config = DiscoveryConfig {
    listen_addr: "192.168.1.100".parse().unwrap(),
    listen_port: 9001, // 自定义端口
    bootnodes: vec![
        // 添加自定义启动节点
        "enr:-Ku4QHqVeJ...".to_string(),
    ],
    enable_ipv6: true, // 启用 IPv6
    query_parallelism: 5, // 增加并发数
    query_timeout: Duration::from_secs(120), // 更长的超时
};

let client = Discv5Client::new(config).await?;
```

## 技术细节

### 架构设计

项目严格遵循 Clean Architecture 原则：

```
Discv5Client (Domain Layer)
    ↓ 使用
discv5 库 (Infrastructure Layer)
    ↓ 通过
UDP/TCP 网络协议
```

### ENR (Ethereum Node Record)

ENR 是以太坊节点的身份记录，包含：

- 节点公钥
- IP 地址和端口
- 自定义键值对（如链配置）
- 序列号和签名

### 节点发现流程

1. **初始化**: 生成本地 ENR 和节点密钥
2. **连接启动节点**: 添加预配置的启动节点
3. **随机查询**: 执行随机节点 ID 查询
4. **周期性发现**: 每 60 秒执行一次节点查询
5. **维护路由表**: 自动维护已发现节点列表

### 性能考虑

#### 缓存行对齐

```rust
#[repr(align(64))]  // 64 字节缓存行对齐
#[derive(Debug, Clone)]
pub struct DiscoveredNode {
    pub enr: Enr<CombinedKey>,
    pub node_id: NodeId,
    pub socket_addr: Option<SocketAddr>,
    pub is_bootnode: bool,
}
```

#### 并发访问

- 使用 `Arc` 实现共享所有权
- 使用 `RwLock` 实现高效读写锁
- 避免在关键路径上的内存分配

## 测试

### 运行单元测试

```bash
# 运行所有测试
cargo test

# 仅运行 discv5 客户端测试
cargo test client::tests
```

### 测试覆盖

- ✅ 客户端创建
- ✅ 本地 ENR 生成
- ✅ 节点发现功能
- ✅ 并发访问

## 故障排除

### 常见问题

**问题1: ServiceNotStarted 错误** ✅ 已修复

```
错误信息: ServiceNotStarted
原因: Discv5 实例未调用 start() 方法

解决方案:
已在 v0.1.0 中修复，Discv5Client::new() 自动启动服务
```

**问题2: ENR 签名验证失败**

```
错误信息: Invalid ENR: Custom("Invalid Signature")
原因: 启动节点 ENR 过时或格式不兼容

解决方案:
1. 使用 DiscoveryConfig::with_bootnodes() 指定自定义启动节点
2. 从可靠来源获取最新的 ENR
3. 运行自己的启动节点
4. 使用空启动节点列表（仅本地测试）
```

**问题3: No known_closest_peers 警告**

```
警告信息: No known_closest_peers found. Return empty result without sending query.
原因: 路由表为空，没有已知节点

解决方案:
这是正常的，当启动节点列表为空时会出现此警告。
添加有效的启动节点即可解决。
```

**问题4: 无法连接到启动节点**

```
解决方案:
1. 检查网络连接
2. 确认防火墙允许 UDP 端口 9000
3. 验证启动节点 ENR 是否正确和最新
4. 使用执行层的 discv5 节点（非共识层）
```

**问题5: 发现的节点数量为 0**

```
解决方案:
1. 添加有效的启动节点 ENR
2. 等待更长时间（至少 30-60 秒）
3. 检查日志中的错误信息
4. 尝试增加 query_parallelism
5. 确认使用正确的网络配置（主网/测试网）
```

**问题6: 端口已被占用**

```
解决方案:
修改配置使用不同的端口:
let config = DiscoveryConfig {
    listen_port: 9001,  // 使用其他端口
    ..Default::default()
};
```

### 获取有效的启动节点

由于以太坊网络的启动节点经常更新，建议：

1. **从官方源获取**:
   ```bash
   # 查看 Geth 最新启动节点
   curl https://raw.githubusercontent.com/ethereum/go-ethereum/master/params/bootnodes.go
   ```

2. **运行自己的节点**:
   ```bash
   # 启动一个节点并获取其 ENR
   cargo run --example discv5_custom_nodes
   # 复制输出的 ENR 供其他节点使用
   ```

3. **手动指定**:
   ```rust
   let config = DiscoveryConfig::with_bootnodes(
       9000,
       vec![
           "enr:-IS4Q...".to_string(),  // 从可靠来源获取
       ]
   );
   ```

## 依赖项

- `discv5 = "0.7"` - Discv5 协议实现
- `enr = "0.12"` - ENR 记录支持
- `tokio` - 异步运行时
- `tracing` - 日志记录

## 参考资料

- [Discv5 规范](https://github.com/ethereum/devp2p/blob/master/discv5/discv5.md)
- [ENR 规范](https://github.com/ethereum/devp2p/blob/master/enr.md)
- [discv5 Rust 实现](https://github.com/sigp/discv5)
- [以太坊开发文档](https://ethereum.org/developers/docs/)

## 许可证

遵循项目主许可证。
