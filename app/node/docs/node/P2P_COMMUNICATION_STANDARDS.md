# 以太坊节点间通信 (P2P) 标准规范

## 🎯 标准来源

| 标准 | 地址 | 说明 |
|------|------|------|
| **DevP2P** | https://github.com/ethereum/devp2p | P2P 网络协议规范 |
| **EIP-8** | https://eips.ethereum.org/EIPS/eip-8 | 向前兼容的网络协议变更 |
| **RLPx** | https://github.com/ethereum/devp2p/blob/master/rlpx.md | 加密传输协议 |
| **Node Discovery** | https://github.com/ethereum/devp2p/tree/master/discv4.md | 节点发现协议 v4 |
| **Discv5** | https://github.com/ethereum/devp2p/blob/master/discv5/discv5.md | 节点发现协议 v5 |

---

## 📚 协议栈概览

以太坊节点间通信采用分层架构：

```
┌─────────────────────────────────────────────────────────────┐
│              Application Layer (应用层)                      │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐  │
│  │  eth/68  │  snap/1  │  wit/0   │  les/4   │  其他...  │  │
│  │  (主链)  │ (快照)   │ (见证)   │ (轻客户端)│          │  │
│  └──────────┴──────────┴──────────┴──────────┴──────────┘  │
├─────────────────────────────────────────────────────────────┤
│              RLPx Layer (加密传输层)                         │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  • 握手 (ECIES 加密)                                 │   │
│  │  • 帧传输 (AES-256-CTR + MAC)                        │   │
│  │  • 多路复用 (Multiplexing)                           │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│         Discovery Layer (节点发现层)                         │
│  ┌──────────────────┬──────────────────┐                   │
│  │  Discv4 (UDP)    │  Discv5 (UDP)    │                   │
│  │  • Ping/Pong     │  • 主题发现      │                   │
│  │  • FindNode      │  • ENR 记录      │                   │
│  │  • Neighbors     │  • Kademlia DHT  │                   │
│  └──────────────────┴──────────────────┘                   │
├─────────────────────────────────────────────────────────────┤
│              Transport Layer (传输层)                        │
│             TCP (RLPx) + UDP (Discovery)                    │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔥 核心协议组件

### 1. DevP2P 协议族

DevP2P 是以太坊 P2P 网络的核心协议栈，包含以下组件：

| 组件 | 协议 | 传输层 | 用途 |
|------|------|--------|------|
| **RLPx** | 加密传输 | TCP | 节点间安全通信 |
| **Discovery v4** | 节点发现 | UDP | 发现对等节点 (执行层) |
| **Discovery v5** | 节点发现 | UDP | 发现对等节点 (共识层) |
| **eth/68** | 以太坊协议 | RLPx/TCP | 区块和交易同步 |
| **snap/1** | 快照协议 | RLPx/TCP | 快速状态同步 |
| **wit/0** | 见证协议 | RLPx/TCP | 无状态客户端支持 |
| **les/4** | 轻客户端 | RLPx/TCP | 轻量级以太坊服务 |

---

## 📋 协议详细分类

### 一、传输层协议

#### 1. RLPx 协议

**用途**: 加密的点对点传输层协议

**核心特性**:
- ✅ ECIES 加密握手（secp256k1）
- ✅ AES-256-CTR 流加密
- ✅ MAC 消息认证
- ✅ 多路复用支持
- ✅ 向前兼容（EIP-8）

**握手流程**:
```
节点 A                                    节点 B
  │                                         │
  │────────── auth (加密) ──────────────────>│
  │                                         │
  │<────────── ack (加密) ───────────────────│
  │                                         │
  │──────── Hello (RLPx) ──────────────────>│
  │                                         │
  │<─────── Hello (RLPx) ────────────────────│
  │                                         │
  │═══════════ 加密通道建立 ══════════════════│
```

**消息格式**:
```
frame = header || header-mac || frame-data || frame-mac

header = frame-size || header-data || padding
header-data = [capability-id, context-id]
frame-data = RLP-encoded message data
```

**端口**: TCP 30303 (默认)

**参考**: https://github.com/ethereum/devp2p/blob/master/rlpx.md

---

### 二、节点发现协议

#### 1. Discovery v4 (执行层)

**用途**: 基于 Kademlia DHT 的节点发现

**消息类型**:
```
1. Ping (0x01)     - 探测节点存活
2. Pong (0x02)     - Ping 响应
3. FindNode (0x03) - 查找指定节点
4. Neighbors (0x04) - FindNode 响应
```

**Ping/Pong 流程**:
```
节点 A                          节点 B
  │                               │
  │─────── Ping ──────────────────>│
  │  (to, ping-hash, expiration)  │
  │                               │
  │<────── Pong ───────────────────│
  │  (to, ping-hash, expiration)  │
```

**FindNode 流程**:
```
节点 A                              节点 B
  │                                   │
  │─────── FindNode ──────────────────>│
  │  (target, expiration)             │
  │                                   │
  │<────── Neighbors ──────────────────│
  │  (nodes[], expiration)            │
```

**节点表示** (v4):
```
Node = (IP, UDP Port, TCP Port, Node ID)
Node ID = keccak256(public_key)  // 64字节
```

**端口**: UDP 30303 (默认)

**参考**: https://github.com/ethereum/devp2p/blob/master/discv4.md

---

#### 2. Discovery v5 (共识层/通用)

**用途**: 增强的节点发现协议，支持主题发现

**改进点**:
- ✅ ENR (Ethereum Node Records) - 可扩展节点记录
- ✅ 主题发现 (Topic Discovery)
- ✅ 更好的 NAT 穿透
- ✅ 支持多种网络协议

**ENR 格式**:
```
ENR = RLP([signature, seq, k1, v1, k2, v2, ...])

示例字段:
- id: "v4" (标识方案)
- secp256k1: <compressed public key>
- ip: <IPv4 address>
- tcp: <TCP port>
- udp: <UDP port>
- eth2: <fork digest + next fork version + next fork epoch>
```

**消息类型**:
```
1. PING (0x01)          - 探测节点
2. PONG (0x02)          - Ping 响应
3. FINDNODE (0x03)      - 查找节点
4. NODES (0x04)         - FindNode 响应
5. TALKREQ (0x05)       - 应用层请求
6. TALKRESP (0x06)      - TALKREQ 响应
7. REGTOPIC (0x07)      - 注册主题（已弃用）
8. TICKET (0x08)        - 主题票据（已弃用）
9. REGCONFIRMATION (0x09) - 注册确认（已弃用）
10. TOPICQUERY (0x0a)   - 主题查询（已弃用）
```

**ENR 示例**:
```
enr:-IS4QHCYrYZbAKWCBRlAy5zzaDZXJBGkcnh4MHcBFZntXNFrdvJjX04jRzjzCBOonrkTfj499SZuOh8R33Ls8RRcy5wBgmlkgnY0gmlwhH8AAAGJc2VjcDI1NmsxoQPKY0yuDUmstAHYpMa2_oxVtw0RW_QAdpzBQA8yWM0xOIN1ZHCCdl8
```

**端口**: UDP 9000 (Beacon Node 默认)

**参考**: https://github.com/ethereum/devp2p/blob/master/discv5/discv5.md

---

### 三、应用层协议 (RLPx 子协议)

#### 1. eth/68 协议 (最新主链协议)

**版本演进**:
```
eth/60 → eth/61 → eth/62 → eth/63 → eth/64 → eth/65 → eth/66 → eth/67 → eth/68
```

**eth/68 主要变更**:
- ✅ 移除 `GetNodeData` 消息
- ✅ 优化交易传播

**消息类型** (eth/68):

**状态消息** (0x00):
```
Status {
  protocol_version: uint32,
  network_id: uint64,
  total_difficulty: U256,
  best_hash: H256,
  genesis_hash: H256,
  fork_id: ForkId
}
```

**核心消息**:
```
0x00: Status              - 握手消息 (双向)
0x01: NewBlockHashes      - 新区块哈希通知
0x02: Transactions        - 交易广播
0x03: GetBlockHeaders     - 请求区块头
0x04: BlockHeaders        - 区块头响应
0x05: GetBlockBodies      - 请求区块体
0x06: BlockBodies         - 区块体响应
0x07: NewBlock            - 新区块通知
0x08: NewPooledTransactionHashes - 新交易哈希通知
0x09: GetPooledTransactions - 请求池中交易
0x0a: PooledTransactions  - 池中交易响应
0x0b: GetReceipts         - 请求收据
0x0c: Receipts            - 收据响应
```

**已移除** (eth/68):
```
❌ GetNodeData (0x0d)     - 已废弃
❌ NodeData (0x0e)        - 已废弃
```

**区块同步流程**:
```
节点 A                                    节点 B
  │                                         │
  │────── GetBlockHeaders ──────────────────>│
  │  (block_number/hash, max_headers,       │
  │   skip, reverse)                        │
  │                                         │
  │<─────── BlockHeaders ────────────────────│
  │  ([header1, header2, ...])              │
  │                                         │
  │────── GetBlockBodies ────────────────────>│
  │  ([hash1, hash2, ...])                  │
  │                                         │
  │<─────── BlockBodies ─────────────────────│
  │  ([body1, body2, ...])                  │
  │                                         │
  │────── GetReceipts ───────────────────────>│
  │  ([hash1, hash2, ...])                  │
  │                                         │
  │<─────── Receipts ────────────────────────│
  │  ([[receipt1, ...], [receipt2, ...]])   │
```

**交易传播流程**:
```
新交易产生
    │
    ├──> NewPooledTransactionHashes (广播哈希)
    │     ↓
    │    对等节点收到哈希
    │     ↓
    │    GetPooledTransactions (请求完整交易)
    │     ↓
    └──> PooledTransactions (返回交易)
```

**参考**: https://github.com/ethereum/devp2p/blob/master/caps/eth.md

---

#### 2. snap/1 协议 (快照同步)

**用途**: 快速同步以太坊状态（账户、存储、字节码）

**优势**:
- ⚡ 比完整同步快 10-100 倍
- 📦 压缩传输（减少带宽）
- 🔄 支持并行下载

**消息类型**:
```
0x00: GetAccountRange     - 请求账户范围
0x01: AccountRange        - 账户范围响应
0x02: GetStorageRanges    - 请求存储范围
0x03: StorageRanges       - 存储范围响应
0x04: GetByteCodes        - 请求字节码
0x05: ByteCodes           - 字节码响应
0x06: GetTrieNodes        - 请求 Trie 节点
0x07: TrieNodes           - Trie 节点响应
```

**同步流程**:
```
1. 获取最新状态根 (state_root)
2. 并行请求账户范围:
   GetAccountRange(state_root, start_hash, limit)
   ↓
   AccountRange(accounts[], proof[])

3. 对每个合约，请求存储:
   GetStorageRanges(state_root, account_hashes[], start_hash, limit)
   ↓
   StorageRanges(storage_slots[], proof[])

4. 请求合约字节码:
   GetByteCodes(code_hashes[])
   ↓
   ByteCodes(codes[])

5. 填补缺失的 Trie 节点:
   GetTrieNodes(state_root, paths[][])
   ↓
   TrieNodes(nodes[])
```

**数据格式**:
```
AccountRange {
  accounts: [(hash, account), ...],
  proof: [node1, node2, ...]  // Merkle proof
}

Account {
  nonce: uint64,
  balance: U256,
  storage_root: H256,
  code_hash: H256
}
```

**参考**: https://github.com/ethereum/devp2p/blob/master/caps/snap.md

---

#### 3. wit/0 协议 (见证协议)

**用途**: 支持无状态客户端

**核心概念**:
- 📦 Witness: 区块执行所需的最小状态证明
- 🔍 允许客户端不存储完整状态
- ⚡ 减少存储需求

**消息类型**:
```
0x00: GetBlockWitness    - 请求区块见证
0x01: BlockWitness       - 区块见证响应
```

**Witness 内容**:
```
Witness {
  block_hash: H256,
  state_nodes: [node1, node2, ...],  // 状态树节点
  code: [code1, code2, ...]           // 合约代码
}
```

**参考**: https://github.com/ethereum/devp2p/issues/222

---

#### 4. les/4 协议 (轻客户端)

**用途**: 轻量级以太坊服务，适用于资源受限设备

**特点**:
- 📱 不存储完整状态
- 🔍 按需请求数据
- 💰 使用"费用模型"防止 DoS

**消息类型**:
```
0x00: Status                  - 握手
0x01: Announce                - 区块通知
0x02: GetBlockHeaders         - 请求区块头
0x03: BlockHeaders            - 区块头响应
0x04: GetBlockBodies          - 请求区块体
0x05: BlockBodies             - 区块体响应
0x06: GetReceipts             - 请求收据
0x07: Receipts                - 收据响应
0x08: GetProofs               - 请求状态证明
0x09: Proofs                  - 状态证明响应
0x0a: GetContractCodes        - 请求合约代码
0x0b: ContractCodes           - 合约代码响应
0x0c: GetHelperTrieProofs     - 请求辅助 Trie 证明
0x0d: HelperTrieProofs        - 辅助 Trie 证明响应
0x0e: SendTx                  - 发送交易
0x0f: GetTxStatus             - 查询交易状态
0x10: TxStatus                - 交易状态响应
```

**费用模型**:
- 客户端消耗"信用额度"请求数据
- 服务器根据负载调整费用
- 客户端需要定期"充值"

**参考**: https://github.com/ethereum/devp2p/blob/master/caps/les.md

---

## 🔐 安全机制

### 1. RLPx 加密

**握手加密** (ECIES):
- secp256k1 椭圆曲线
- ECDH 密钥交换
- AES-256-CTR 加密
- HMAC-SHA256 消息认证

**帧加密**:
```
帧加密密钥 = 从握手派生
MAC 密钥 = 从握手派生

每个帧:
  加密数据 = AES-256-CTR(frame-data, key)
  MAC = HMAC-SHA256(header || frame-data, mac-key)
```

### 2. 节点认证

**节点 ID 验证**:
```
Node ID = keccak256(secp256k1_public_key)

每个消息签名:
  signature = ECDSA(message_hash, private_key)

验证:
  recovered_pubkey = ecrecover(signature, message_hash)
  recovered_node_id = keccak256(recovered_pubkey)
  assert recovered_node_id == claimed_node_id
```

### 3. 防 DDoS 机制

**速率限制**:
- 消息类型限流
- 带宽限制
- 连接数限制

**信誉系统**:
- 跟踪对等节点行为
- 惩罚不良行为
- 优先服务良好节点

**资源限制**:
- 最大消息大小
- 请求批量限制
- 响应超时

---

## 📊 网络拓扑

### 节点类型

| 类型 | 说明 | 连接数 | 数据量 |
|------|------|--------|--------|
| **Full Node** | 存储完整区块链 | 25-50 | 完整 |
| **Archive Node** | 存储历史状态 | 25-50 | 完整+历史 |
| **Light Node** | 轻客户端 | 5-10 | 按需 |
| **Bootnode** | 引导节点 | 高 | 少 |

### 连接策略

**最大连接数**:
- 入站: 50 (可配置)
- 出站: 25 (主动连接)

**连接选择**:
1. 优先连接低延迟节点
2. 地理分布均匀
3. 客户端多样性
4. 避免同一子网过多连接

**连接保持**:
- Ping 心跳 (15秒间隔)
- 无响应断开 (60秒超时)
- 定期重连发现的新节点

---

## 🛠️ 消息编码

### RLP 编码

所有 DevP2P 消息使用 RLP (Recursive Length Prefix) 编码:

```
RLP 编码规则:
- 字节串:
  - [0x00, 0x7f]: 自身
  - [0x80, 0xb7]: 0x80 + length || data
  - [0xb8, 0xbf]: 0xb7 + length_of_length || length || data

- 列表:
  - [0xc0, 0xf7]: 0xc0 + length || items
  - [0xf8, 0xff]: 0xf7 + length_of_length || length || items
```

**示例**:
```
数字 15:       0x0f
字符串 "cat":  0x83 0x63 0x61 0x74
列表 []:       0xc0
列表 ["cat", "dog"]: 0xc8 0x83 0x63 0x61 0x74 0x83 0x64 0x6f 0x67
```

### Snappy 压缩

snap/1 协议使用 Snappy 压缩减少带宽:

```
压缩消息 = snappy_compress(rlp_encode(message))
解压消息 = rlp_decode(snappy_decompress(data))
```

---

## 🚀 实现示例

### Rust 实现框架

```rust
use libp2p::{
    identity,
    PeerId,
    swarm::{Swarm, SwarmBuilder},
    Transport,
};

// 1. 创建节点身份
let local_key = identity::Keypair::generate_secp256k1();
let local_peer_id = PeerId::from(local_key.public());

// 2. 配置传输层
let transport = libp2p::tcp::TokioTcpTransport::new(tcp_config)
    .upgrade(upgrade::Version::V1)
    .authenticate(secio::SecioConfig::new(local_key.clone()))
    .multiplex(yamux::YamuxConfig::default())
    .boxed();

// 3. 配置 RLPx 行为
let behaviour = RLPxBehaviour::new(
    eth_protocol_config,
    snap_protocol_config,
);

// 4. 创建 Swarm
let mut swarm = SwarmBuilder::new(transport, behaviour, local_peer_id)
    .executor(Box::new(|fut| {
        tokio::spawn(fut);
    }))
    .build();

// 5. 监听地址
swarm.listen_on("/ip4/0.0.0.0/tcp/30303".parse()?)?;

// 6. 连接 bootnode
let bootnode: Multiaddr = "/ip4/1.2.3.4/tcp/30303".parse()?;
swarm.dial(bootnode)?;

// 7. 事件循环
loop {
    match swarm.next().await {
        Some(event) => handle_event(event),
        None => break,
    }
}
```

### 消息处理示例

```rust
// eth/68 Status 消息
#[derive(RlpEncodable, RlpDecodable)]
struct Status {
    protocol_version: u32,
    network_id: u64,
    total_difficulty: U256,
    best_hash: H256,
    genesis_hash: H256,
    fork_id: ForkId,
}

// 发送 Status
async fn send_status(peer: PeerId, status: Status) -> Result<()> {
    let message = Message::Status(status);
    let rlp_data = rlp::encode(&message);

    network.send_message(peer, ETH_PROTOCOL_ID, 0x00, rlp_data).await?;
    Ok(())
}

// 接收并处理 GetBlockHeaders
async fn handle_get_block_headers(
    peer: PeerId,
    request: GetBlockHeaders,
) -> Result<()> {
    // 查询本地区块头
    let headers = blockchain.get_headers(
        request.start_block,
        request.max_headers,
        request.skip,
        request.reverse,
    )?;

    // 编码并发送响应
    let message = Message::BlockHeaders(headers);
    let rlp_data = rlp::encode(&message);

    network.send_message(peer, ETH_PROTOCOL_ID, 0x04, rlp_data).await?;
    Ok(())
}
```

---

## 📊 性能指标

### 网络性能目标

| 指标 | 目标值 | 说明 |
|------|--------|------|
| **区块传播** | < 500ms | 95% 节点收到新区块 |
| **交易传播** | < 2s | 全网传播 |
| **握手延迟** | < 100ms | RLPx 握手完成 |
| **节点发现** | < 5s | 发现足够对等节点 |
| **带宽** | 1-10 MB/s | 全节点平均带宽 |

### 资源消耗

| 资源 | 全节点 | 轻节点 | Archive 节点 |
|------|--------|--------|-------------|
| **CPU** | 2-4核 | 1核 | 4-8核 |
| **内存** | 8-16 GB | 512 MB | 32+ GB |
| **存储** | 1-2 TB | 1 GB | 10+ TB |
| **带宽** | 5 MB/s | 100 KB/s | 10 MB/s |

---

## 🔍 监控和调试

### 关键指标

**连接指标**:
```
- peer_count: 当前对等节点数
- inbound_connections: 入站连接数
- outbound_connections: 出站连接数
- peer_churn: 节点变化率
```

**消息指标**:
```
- messages_sent: 发送消息数
- messages_received: 接收消息数
- bytes_sent: 发送字节数
- bytes_received: 接收字节数
- message_latency: 消息延迟
```

**同步指标**:
```
- sync_progress: 同步进度 (%)
- blocks_imported: 已导入区块数
- blocks_per_second: 同步速度
- state_sync_progress: 状态同步进度
```

### 调试工具

**网络诊断**:
```bash
# 查看节点连接
curl -X POST http://localhost:8545 \
  -d '{"jsonrpc":"2.0","method":"admin_peers","params":[],"id":1}'

# 查看节点信息
curl -X POST http://localhost:8545 \
  -d '{"jsonrpc":"2.0","method":"admin_nodeInfo","params":[],"id":1}'

# 添加对等节点
curl -X POST http://localhost:8545 \
  -d '{"jsonrpc":"2.0","method":"admin_addPeer","params":["enode://..."],"id":1}'
```

**Wireshark 抓包**:
```
# 捕获 RLPx 流量
tcpdump -i any -w rlpx.pcap 'tcp port 30303'

# 捕获 Discovery 流量
tcpdump -i any -w discovery.pcap 'udp port 30303'
```

---

## 📚 参考资源

### 官方规范
- [DevP2P 规范](https://github.com/ethereum/devp2p)
- [RLPx 协议](https://github.com/ethereum/devp2p/blob/master/rlpx.md)
- [eth/68 协议](https://github.com/ethereum/devp2p/blob/master/caps/eth.md)
- [snap/1 协议](https://github.com/ethereum/devp2p/blob/master/caps/snap.md)
- [Discovery v4](https://github.com/ethereum/devp2p/blob/master/discv4.md)
- [Discovery v5](https://github.com/ethereum/devp2p/blob/master/discv5/discv5.md)

### 参考实现
- [Geth (Go)](https://github.com/ethereum/go-ethereum/tree/master/p2p)
- [Reth (Rust)](https://github.com/paradigmxyz/reth/tree/main/crates/net)
- [Nethermind (C#)](https://github.com/NethermindEth/nethermind)

### EIP 提案
- [EIP-8: devp2p Forward Compatibility](https://eips.ethereum.org/EIPS/eip-8)
- [EIP-2124: Fork identifier for chain compatibility checks](https://eips.ethereum.org/EIPS/eip-2124)

### 工具和库
- [libp2p](https://libp2p.io/) - 模块化 P2P 网络栈
- [rlp](https://github.com/paritytech/parity-common/tree/master/rlp) - RLP 编码库
- [secp256k1](https://github.com/rust-bitcoin/rust-secp256k1) - 椭圆曲线密码学

---

**文档版本**: v1.0
**更新日期**: 2025-11-09
**适用于**: Ethereum Execution Layer P2P Networking
