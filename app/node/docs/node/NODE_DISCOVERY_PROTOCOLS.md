# 以太坊节点发现协议详解

## 📚 概述

以太坊使用两个主要的节点发现协议：
- **Discovery v4 (discv4)**: 执行层节点发现
- **Discovery v5 (discv5)**: 共识层节点发现（更先进）

两者都基于 **Kademlia DHT** (分布式哈希表) 算法，使用 **UDP** 传输。

---

## 🔍 Discovery v4 (执行层)

### 标准来源
- **规范**: https://github.com/ethereum/devp2p/blob/master/discv4.md
- **传输层**: UDP
- **端口**: 30303 (默认)

---

### 核心概念

#### 节点 ID
```
Node ID = keccak256(secp256k1_public_key)  // 64字节 (512位)

示例:
public_key = 0x0404...  // 65字节 (未压缩)
node_id = keccak256(public_key[1..])  // 跳过0x04前缀
```

#### 节点距离

使用 **XOR 距离** (Kademlia 标准):

```rust
fn distance(node_a: &NodeId, node_b: &NodeId) -> U512 {
    let xor = node_a ^ node_b;
    U512::from_be_bytes(xor)
}

// 距离越小，节点越"接近"
// distance(a, a) = 0
// distance(a, b) = distance(b, a)
```

#### K-Bucket 路由表

```
路由表结构:
  - 256 个 bucket (每个对应一个比特位距离)
  - 每个 bucket 最多 16 个节点
  - 最近看到的节点放在 bucket 前面 (LRU)

Bucket 0:  距离在 [2^0, 2^1) 的节点
Bucket 1:  距离在 [2^1, 2^2) 的节点
Bucket 2:  距离在 [2^2, 2^3) 的节点
...
Bucket 255: 距离在 [2^255, 2^256) 的节点
```

**Rust 实现**:
```rust
const BUCKET_SIZE: usize = 16;
const NUM_BUCKETS: usize = 256;

struct KBucket {
    nodes: Vec<Node>,  // 最多 16 个
    last_updated: Instant,
}

struct RoutingTable {
    local_node_id: NodeId,
    buckets: [KBucket; NUM_BUCKETS],
}

impl RoutingTable {
    fn bucket_index(&self, node_id: &NodeId) -> usize {
        let distance = self.local_node_id ^ node_id;
        // 找到最高位的 1
        255 - distance.leading_zeros() as usize
    }

    fn add_node(&mut self, node: Node) {
        let index = self.bucket_index(&node.id);
        let bucket = &mut self.buckets[index];

        // 如果节点已存在，移到前面 (LRU)
        if let Some(pos) = bucket.nodes.iter().position(|n| n.id == node.id) {
            bucket.nodes.remove(pos);
            bucket.nodes.insert(0, node);
            return;
        }

        // 如果 bucket 未满，直接添加
        if bucket.nodes.len() < BUCKET_SIZE {
            bucket.nodes.insert(0, node);
            bucket.last_updated = Instant::now();
            return;
        }

        // Bucket 已满，Ping 最后一个节点
        let last_node = bucket.nodes.last().unwrap();
        if !self.ping(last_node) {
            // 最后一个节点无响应，替换
            bucket.nodes.pop();
            bucket.nodes.insert(0, node);
        }
    }
}
```

---

### 消息格式

所有 Discovery v4 消息格式：

```
UDP Packet = packet-header || packet-data

packet-header = hash || signature || packet-type
hash = keccak256(signature || packet-type || packet-data)
signature = sign(keccak256(packet-type || packet-data), private_key)
packet-type = 0x01 (Ping) | 0x02 (Pong) | 0x03 (FindNode) | 0x04 (Neighbors)

packet-data = RLP([field1, field2, ...])
```

**签名验证**:
```rust
fn verify_packet(packet: &[u8]) -> Result<(NodeId, u8, Vec<u8>)> {
    // 1. 解析
    let hash = &packet[0..32];
    let signature = &packet[32..96];  // 65字节
    let packet_type = packet[96];
    let packet_data = &packet[97..];

    // 2. 验证哈希
    let computed_hash = keccak256([signature, &[packet_type], packet_data].concat());
    if computed_hash != hash {
        return Err(Error::InvalidHash);
    }

    // 3. 恢复公钥
    let message_hash = keccak256([[packet_type].as_ref(), packet_data].concat());
    let public_key = ecrecover(signature, &message_hash)?;
    let node_id = keccak256(&public_key[1..]);  // 跳过0x04

    Ok((node_id, packet_type, packet_data.to_vec()))
}
```

---

### 1. Ping (0x01)

**用途**: 探测节点是否在线，交换端点信息

**消息格式**:
```
Ping {
  version: 4,                    // 协议版本
  from: Endpoint,                // 发送方端点
  to: Endpoint,                  // 接收方端点
  expiration: unix_timestamp,    // 消息过期时间
}

Endpoint {
  ip: [u8; 4] or [u8; 16],      // IPv4 或 IPv6
  udp_port: u16,                 // UDP 端口
  tcp_port: u16,                 // TCP 端口 (可选，为0表示无)
}
```

**RLP 编码**:
```
Ping = [version, from, to, expiration]
from = [ip, udp_port, tcp_port]
to = [ip, udp_port, tcp_port]
```

**发送 Ping**:
```rust
async fn send_ping(
    socket: &UdpSocket,
    target: SocketAddr,
    local_endpoint: Endpoint,
    target_endpoint: Endpoint,
) -> Result<H256> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() + 60;

    let ping = vec![
        rlp::encode(&4u8),                     // version
        rlp::encode(&local_endpoint),          // from
        rlp::encode(&target_endpoint),         // to
        rlp::encode(&expiration),              // expiration
    ];

    let packet_data = rlp::encode_list::<Vec<u8>, _>(&ping);
    let packet = build_packet(0x01, &packet_data)?;

    socket.send_to(&packet, target).await?;

    // 返回 Ping 哈希用于匹配 Pong
    let ping_hash = keccak256([[0x01].as_ref(), &packet_data].concat());
    Ok(ping_hash)
}
```

---

### 2. Pong (0x02)

**用途**: 响应 Ping，确认节点在线

**消息格式**:
```
Pong {
  to: Endpoint,                  // 接收方端点 (回显 Ping 的 from)
  ping_hash: H256,               // 对应的 Ping 消息哈希
  expiration: unix_timestamp,    // 消息过期时间
}
```

**处理 Ping 并发送 Pong**:
```rust
async fn handle_ping(
    socket: &UdpSocket,
    sender: SocketAddr,
    ping: Ping,
) -> Result<()> {
    // 1. 验证过期时间
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    if ping.expiration < now {
        return Err(Error::Expired);
    }

    // 2. 更新路由表
    routing_table.add_node(Node {
        id: sender_node_id,
        endpoint: ping.from,
        last_seen: Instant::now(),
    });

    // 3. 计算 ping_hash
    let ping_hash = keccak256(received_packet[97..]);  // packet_type || packet_data

    // 4. 发送 Pong
    let expiration = now + 60;
    let pong = vec![
        rlp::encode(&ping.from),               // to (echo back)
        rlp::encode(&ping_hash),               // ping_hash
        rlp::encode(&expiration),              // expiration
    ];

    let packet_data = rlp::encode_list::<Vec<u8>, _>(&pong);
    let packet = build_packet(0x02, &packet_data)?;

    socket.send_to(&packet, sender).await?;
    Ok(())
}
```

**验证 Pong**:
```rust
async fn verify_pong(pong: Pong, expected_ping_hash: H256) -> Result<()> {
    // 1. 验证过期时间
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    if pong.expiration < now {
        return Err(Error::Expired);
    }

    // 2. 验证 ping_hash 匹配
    if pong.ping_hash != expected_ping_hash {
        return Err(Error::PingHashMismatch);
    }

    Ok(())
}
```

---

### 3. FindNode (0x03)

**用途**: 查找距离目标 ID 最近的节点

**消息格式**:
```
FindNode {
  target: NodeId,                // 查找目标 (64字节)
  expiration: unix_timestamp,    // 消息过期时间
}
```

**发送 FindNode**:
```rust
async fn send_find_node(
    socket: &UdpSocket,
    target_addr: SocketAddr,
    target_node_id: NodeId,
) -> Result<()> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() + 60;

    let find_node = vec![
        rlp::encode(&target_node_id),
        rlp::encode(&expiration),
    ];

    let packet_data = rlp::encode_list::<Vec<u8>, _>(&find_node);
    let packet = build_packet(0x03, &packet_data)?;

    socket.send_to(&packet, target_addr).await?;
    Ok(())
}
```

**处理 FindNode**:
```rust
async fn handle_find_node(
    socket: &UdpSocket,
    sender: SocketAddr,
    find_node: FindNode,
    routing_table: &RoutingTable,
) -> Result<()> {
    // 1. 验证过期时间
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    if find_node.expiration < now {
        return Err(Error::Expired);
    }

    // 2. 从路由表查找最近的 16 个节点
    let closest_nodes = routing_table.find_closest(find_node.target, 16);

    // 3. 发送 Neighbors 响应
    send_neighbors(socket, sender, closest_nodes).await?;

    Ok(())
}
```

---

### 4. Neighbors (0x04)

**用途**: 返回 FindNode 查询的结果

**消息格式**:
```
Neighbors {
  nodes: Vec<Node>,              // 节点列表 (最多 16 个)
  expiration: unix_timestamp,    // 消息过期时间
}

Node {
  ip: [u8; 4] or [u8; 16],      // IP 地址
  udp_port: u16,                 // UDP 端口
  tcp_port: u16,                 // TCP 端口
  node_id: [u8; 64],            // 节点 ID
}
```

**发送 Neighbors**:
```rust
async fn send_neighbors(
    socket: &UdpSocket,
    target: SocketAddr,
    nodes: Vec<Node>,
) -> Result<()> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() + 60;

    // 编码节点列表
    let encoded_nodes: Vec<Vec<u8>> = nodes
        .iter()
        .map(|node| {
            rlp::encode_list(&[
                rlp::encode(&node.endpoint.ip),
                rlp::encode(&node.endpoint.udp_port),
                rlp::encode(&node.endpoint.tcp_port),
                rlp::encode(&node.id),
            ])
        })
        .collect();

    let neighbors = vec![
        rlp::encode_list(&encoded_nodes),
        rlp::encode(&expiration),
    ];

    let packet_data = rlp::encode_list::<Vec<u8>, _>(&neighbors);
    let packet = build_packet(0x04, &packet_data)?;

    socket.send_to(&packet, target).await?;
    Ok(())
}
```

**处理 Neighbors**:
```rust
async fn handle_neighbors(
    neighbors: Neighbors,
    routing_table: &mut RoutingTable,
) -> Result<()> {
    // 1. 验证过期时间
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    if neighbors.expiration < now {
        return Err(Error::Expired);
    }

    // 2. 添加节点到路由表
    for node in neighbors.nodes {
        routing_table.add_node(node);
    }

    Ok(())
}
```

---

### 节点发现算法

**查找节点** (Kademlia Lookup):

```rust
async fn lookup_node(target: NodeId) -> Result<Vec<Node>> {
    const ALPHA: usize = 3;  // 并发查询数
    const K: usize = 16;     // 返回节点数

    let mut queried = HashSet::new();
    let mut closest = routing_table.find_closest(target, K);

    loop {
        // 选择未查询过的最近节点
        let to_query: Vec<_> = closest
            .iter()
            .filter(|n| !queried.contains(&n.id))
            .take(ALPHA)
            .cloned()
            .collect();

        if to_query.is_empty() {
            break;  // 所有节点已查询
        }

        // 并发查询
        let futures: Vec<_> = to_query
            .iter()
            .map(|node| find_node(node.endpoint, target))
            .collect();

        let results = join_all(futures).await;

        // 合并结果
        for result in results {
            if let Ok(nodes) = result {
                for node in nodes {
                    queried.insert(node.id);
                    if !closest.iter().any(|n| n.id == node.id) {
                        closest.push(node);
                    }
                }
            }
        }

        // 保留最近的 K 个节点
        closest.sort_by_key(|n| distance(&n.id, &target));
        closest.truncate(K);
    }

    Ok(closest)
}
```

**自举** (Bootstrap):

```rust
async fn bootstrap(bootnodes: Vec<SocketAddr>) -> Result<()> {
    // 1. Ping 所有 bootnode
    for bootnode in bootnodes {
        send_ping(bootnode).await?;
    }

    // 2. 查找自己的节点 ID (填充路由表)
    let local_id = self.local_node_id;
    lookup_node(local_id).await?;

    // 3. 刷新所有 bucket
    for i in 0..256 {
        let random_id_in_bucket = generate_random_id_for_bucket(i);
        lookup_node(random_id_in_bucket).await?;
    }

    Ok(())
}
```

---

## 🔍 Discovery v5 (共识层/通用)

### 标准来源
- **规范**: https://github.com/ethereum/devp2p/blob/master/discv5/discv5.md
- **传输层**: UDP
- **端口**: 9000 (Beacon Node 默认)

---

### 改进点

| 特性 | Discovery v4 | Discovery v5 |
|------|-------------|-------------|
| **节点记录** | 简单 (IP, Port, ID) | ENR (可扩展) |
| **加密** | 签名 | ECIES + AES-GCM |
| **主题发现** | ❌ | ✅ (已弃用部分功能) |
| **会话管理** | 无状态 | 有状态会话 |
| **请求-响应** | 简单 | 带请求 ID |
| **节点信息** | 静态 | 动态更新 (ENR seq) |

---

### ENR (Ethereum Node Record)

**格式**:
```
ENR = RLP([signature, seq, k1, v1, k2, v2, ...])

signature = sign(keccak256(rlp([seq, k1, v1, k2, v2, ...])), private_key)
seq = 序列号 (每次更新递增)
k, v = 键值对 (按键排序)
```

**标准字段**:
```
id: "v4"                                    // 标识方案
secp256k1: <compressed public key>          // 公钥 (33字节)
ip: <IPv4 address>                          // IPv4 地址 (4字节)
ip6: <IPv6 address>                         // IPv6 地址 (16字节)
tcp: <TCP port>                             // TCP 端口
udp: <UDP port>                             // UDP 端口
tcp6: <TCP IPv6 port>                       // TCP IPv6 端口
udp6: <UDP IPv6 port>                       // UDP IPv6 端口

// Beacon Chain 特定
eth2: [fork_digest, next_fork_version, next_fork_epoch]
attnets: <bitfield>                         // 证明子网 (64 bits)
syncnets: <bitfield>                        // 同步委员会子网 (4 bits)
```

**ENR 示例**:
```
enr:-IS4QHCYrYZbAKWCBRlAy5zzaDZXJBGkcnh4MHcBFZntXNFrdvJjX04jRzjzCBOonrkTfj499SZuOh8R33Ls8RRcy5wBgmlkgnY0gmlwhH8AAAGJc2VjcDI1NmsxoQPKY0yuDUmstAHYpMa2_oxVtw0RW_QAdpzBQA8yWM0xOIN1ZHCCdl8
```

**解码 ENR**:
```rust
use enr::{Enr, CombinedKey};

fn decode_enr(enr_str: &str) -> Result<Enr<CombinedKey>> {
    // ENR 使用 base64url 编码
    let enr: Enr<CombinedKey> = enr_str.parse()?;

    println!("Node ID: {}", enr.node_id());
    println!("IP: {:?}", enr.ip4());
    println!("UDP Port: {:?}", enr.udp4());
    println!("TCP Port: {:?}", enr.tcp4());
    println!("Seq: {}", enr.seq());

    Ok(enr)
}
```

**创建 ENR**:
```rust
use enr::{EnrBuilder, CombinedKey};
use k256::ecdsa::SigningKey;

fn create_enr() -> Result<Enr<CombinedKey>> {
    // 1. 生成密钥
    let key = SigningKey::random(&mut rand::thread_rng());
    let enr_key = CombinedKey::from(key);

    // 2. 构建 ENR
    let enr = EnrBuilder::new("v4")
        .ip4("127.0.0.1".parse()?)
        .tcp4(30303)
        .udp4(30303)
        .build(&enr_key)?;

    println!("ENR: {}", enr.to_base64());

    Ok(enr)
}
```

**更新 ENR**:
```rust
fn update_enr(enr: &mut Enr<CombinedKey>, key: &CombinedKey) -> Result<()> {
    // 修改字段会自动递增 seq 并重新签名
    enr.set_ip("192.168.1.1".parse()?, key)?;
    enr.set_tcp4(30304, key)?;

    Ok(())
}
```

---

### 消息格式

**通用消息结构**:
```
UDP Packet = masking-iv || header || message

masking-iv = random data (16 bytes)
header = masked_header(masking-iv, dest-node-id, static-header)
static-header = protocol-id || version || flag || nonce || authdata-size

protocol-id = "discv5" (6 bytes)
version = 0x0001 (2 bytes)
flag = message type (1 byte)
nonce = AES-GCM nonce (12 bytes)
authdata-size = size of authdata (2 bytes)

message = encrypted_message(nonce, message-data)
```

**会话密钥派生**:
```rust
fn derive_keys(
    local_node_id: &NodeId,
    remote_node_id: &NodeId,
    challenge_data: &[u8],
) -> (Vec<u8>, Vec<u8>) {
    // 1. ECDH 共享密钥
    let shared_secret = ecdh(local_private_key, remote_public_key);

    // 2. HKDF 派生
    let info = [local_node_id, remote_node_id, challenge_data].concat();
    let (initiator_key, recipient_key) = hkdf_expand(shared_secret, &info, 32);

    (initiator_key, recipient_key)
}
```

---

### 消息类型

#### 1. PING (0x01)

```
PING {
  request_id: u64,               // 请求 ID
  enr_seq: u64,                  // 本地 ENR 序列号
}
```

#### 2. PONG (0x02)

```
PONG {
  request_id: u64,               // 对应的 PING 请求 ID
  enr_seq: u64,                  // 本地 ENR 序列号
  ip: IpAddr,                    // 对方的 IP (回显)
  port: u16,                     // 对方的端口 (回显)
}
```

#### 3. FINDNODE (0x03)

```
FINDNODE {
  request_id: u64,               // 请求 ID
  distances: Vec<u16>,           // 请求的距离列表 (0-256)
}
```

**距离查询**:
```
distances = [256]              // 查找距离为 256 的节点 (随机)
distances = [0]                // 查找自己的 ENR
distances = [250, 251, 252]    // 查找多个距离的节点
```

#### 4. NODES (0x04)

```
NODES {
  request_id: u64,               // 对应的 FINDNODE 请求 ID
  total: u8,                     // 总响应数
  enrs: Vec<Enr>,                // ENR 列表
}
```

**响应分片**:
```
// 如果 ENR 太多，分多个 NODES 消息发送
NODES { request_id: 1, total: 3, enrs: [enr1, enr2, ...] }
NODES { request_id: 1, total: 3, enrs: [enr10, enr11, ...] }
NODES { request_id: 1, total: 3, enrs: [enr20, enr21, ...] }
```

#### 5. TALKREQ (0x05) / TALKRESP (0x06)

**用途**: 应用层自定义请求/响应

```
TALKREQ {
  request_id: u64,               // 请求 ID
  protocol: String,              // 协议标识
  request: Vec<u8>,              // 请求数据
}

TALKRESP {
  request_id: u64,               // 对应的 TALKREQ 请求 ID
  response: Vec<u8>,             // 响应数据
}
```

**示例**:
```rust
// 发送自定义请求
async fn send_talk_request(
    node: &Enr,
    protocol: &str,
    request: Vec<u8>,
) -> Result<Vec<u8>> {
    let request_id = generate_request_id();

    let talk_req = TalkRequest {
        request_id,
        protocol: protocol.to_string(),
        request,
    };

    discv5.send_talk_req(node, talk_req).await?;

    // 等待响应
    let response = discv5.await_talk_resp(request_id).await?;
    Ok(response)
}
```

---

### Discv5 实现示例

**完整示例**:
```rust
use discv5::{Discv5, Discv5Config, Discv5Event};
use enr::{Enr, CombinedKey};
use std::net::SocketAddr;

async fn run_discv5() -> Result<()> {
    // 1. 创建 ENR 密钥
    let enr_key = CombinedKey::generate_secp256k1();

    // 2. 创建 ENR
    let enr = {
        let mut builder = enr::EnrBuilder::new("v4");
        builder.ip4("0.0.0.0".parse()?);
        builder.udp4(9000);
        builder.build(&enr_key)?
    };

    // 3. 配置 Discv5
    let config = Discv5Config::default();

    // 4. 创建 Discv5 实例
    let mut discv5 = Discv5::new(enr, enr_key, config)?;

    // 5. 启动
    discv5.start("0.0.0.0:9000".parse()?).await?;

    // 6. 添加 bootnode
    let bootnode: Enr<CombinedKey> = "enr:-IS4...".parse()?;
    discv5.add_enr(bootnode)?;

    // 7. 查找节点
    let target = enr::NodeId::random();
    let nodes = discv5.find_node(target).await?;
    println!("Found {} nodes", nodes.len());

    // 8. 事件循环
    loop {
        match discv5.next_event().await {
            Discv5Event::NodeDiscovered(enr) => {
                println!("Discovered node: {}", enr.node_id());
            }
            Discv5Event::SessionEstablished(node_id, addr) => {
                println!("Session established: {} at {}", node_id, addr);
            }
            _ => {}
        }
    }
}
```

---

## 📊 性能优化

### 路由表维护

```rust
async fn refresh_buckets(routing_table: &mut RoutingTable) {
    // 定期刷新 bucket (每 1 小时)
    let mut interval = tokio::time::interval(Duration::from_secs(3600));

    loop {
        interval.tick().await;

        // 对每个 bucket 执行随机查询
        for i in 0..256 {
            // 生成该 bucket 范围内的随机 ID
            let random_id = generate_id_in_bucket(i);

            // 异步查询
            tokio::spawn(async move {
                let _ = lookup_node(random_id).await;
            });
        }
    }
}
```

### 并发查询

```rust
async fn concurrent_lookup(
    targets: Vec<NodeId>,
) -> Vec<Result<Vec<Node>>> {
    let futures: Vec<_> = targets
        .into_iter()
        .map(|target| lookup_node(target))
        .collect();

    futures::future::join_all(futures).await
}
```

### 缓存优化

```rust
use lru::LruCache;

struct DiscoveryCache {
    // 缓存最近查询的节点
    lookup_cache: LruCache<NodeId, Vec<Node>>,

    // 缓存 ENR
    enr_cache: LruCache<NodeId, Enr>,
}

impl DiscoveryCache {
    fn get_or_lookup(&mut self, target: NodeId) -> Vec<Node> {
        if let Some(nodes) = self.lookup_cache.get(&target) {
            return nodes.clone();
        }

        let nodes = lookup_node(target).await;
        self.lookup_cache.put(target, nodes.clone());
        nodes
    }
}
```

---

## 🔍 调试和监控

### 路由表统计

```rust
struct RoutingTableStats {
    total_nodes: usize,
    active_buckets: usize,
    oldest_node: Option<Instant>,
    newest_node: Option<Instant>,
}

impl RoutingTable {
    fn stats(&self) -> RoutingTableStats {
        let total_nodes = self.buckets.iter().map(|b| b.nodes.len()).sum();
        let active_buckets = self.buckets.iter().filter(|b| !b.nodes.is_empty()).count();

        let oldest_node = self
            .buckets
            .iter()
            .flat_map(|b| &b.nodes)
            .map(|n| n.last_seen)
            .min();

        let newest_node = self
            .buckets
            .iter()
            .flat_map(|b| &b.nodes)
            .map(|n| n.last_seen)
            .max();

        RoutingTableStats {
            total_nodes,
            active_buckets,
            oldest_node,
            newest_node,
        }
    }
}
```

### 网络诊断

```bash
# Discv5 节点查询
curl -X POST http://localhost:9000/debug/discovery \
  -d '{"method":"nodeInfo"}'

# 路由表转储
curl -X POST http://localhost:9000/debug/discovery \
  -d '{"method":"routingTable"}'

# 查找节点
curl -X POST http://localhost:9000/debug/discovery \
  -d '{"method":"findNode","params":{"target":"0x1234..."}}'
```

---

## 📚 参考资源

### 官方规范
- [Discovery v4](https://github.com/ethereum/devp2p/blob/master/discv4.md)
- [Discovery v5](https://github.com/ethereum/devp2p/blob/master/discv5/discv5.md)
- [ENR 规范](https://github.com/ethereum/devp2p/blob/master/enr.md)

### 参考实现
- [discv5 (Rust)](https://github.com/sigp/discv5) - Lighthouse 使用
- [go-ethereum/p2p/discover](https://github.com/ethereum/go-ethereum/tree/master/p2p/discover)

### 工具
- [enr](https://github.com/sigp/enr) - ENR 库
- [Kademlia 论文](https://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf)

---

**文档版本**: v1.0
**更新日期**: 2025-11-09
**适用于**: Ethereum Node Discovery Protocols (v4 & v5)
