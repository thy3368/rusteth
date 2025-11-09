# RLPx 传输协议详解

## 📚 概述

RLPx 是以太坊节点间加密通信的传输层协议，基于 TCP 连接，提供：
- 🔐 端到端加密（ECIES + AES-256-CTR）
- 🔑 身份认证（secp256k1 ECDSA）
- 📦 消息完整性（HMAC-SHA256）
- 🔀 多路复用（Capability-based multiplexing）
- 🔄 向前兼容（EIP-8）

**标准来源**: https://github.com/ethereum/devp2p/blob/master/rlpx.md

---

## 🔐 加密握手 (Encrypted Handshake)

### 握手流程

```
发起方 (Initiator)                    接收方 (Recipient)
     │                                      │
     │                                      │
     │────── auth (ECIES 加密) ──────────────>│
     │  包含:                                │
     │  - 签名                               │
     │  - 公钥                               │
     │  - nonce                              │
     │  - 版本                               │
     │                                      │
     │<────── ack (ECIES 加密) ───────────────│
     │  包含:                                │
     │  - 公钥                               │
     │  - nonce                              │
     │  - 版本                               │
     │                                      │
     │═══════ 派生共享密钥 ════════════════════│
     │                                      │
     │────── Hello (RLPx 帧) ────────────────>│
     │                                      │
     │<────── Hello (RLPx 帧) ────────────────│
     │                                      │
     │════════ 加密通道建立 ═══════════════════│
```

---

### auth 消息 (发起方 → 接收方)

**消息结构** (EIP-8 格式):
```
auth = auth-size || enc-auth-body
auth-size = size of enc-auth-body, encoded as a big-endian 16-bit integer
enc-auth-body = ECIES_encrypt(recipient-public-key, auth-body)

auth-body = [
  signature,              // secp256k1 签名
  initiator-public-key,   // 发起方公钥 (64字节)
  initiator-nonce,        // 随机数 (32字节)
  version                 // RLPx 版本 (4)
]

signature = sign(initiator-private-key, keccak256(initiator-nonce || recipient-public-key))
```

**ECIES 加密过程**:
```rust
fn ecies_encrypt(recipient_pubkey: &PublicKey, plaintext: &[u8]) -> Vec<u8> {
    // 1. 生成临时密钥对
    let ephemeral_key = generate_keypair();

    // 2. ECDH 计算共享密钥
    let shared_secret = ecdh(ephemeral_key.secret, recipient_pubkey);

    // 3. 派生加密密钥
    let (enc_key, mac_key) = kdf(shared_secret);

    // 4. AES-128-CTR 加密
    let ciphertext = aes_128_ctr_encrypt(plaintext, enc_key);

    // 5. 计算 HMAC
    let mac = hmac_sha256(mac_key, ciphertext);

    // 6. 返回: ephemeral_pubkey || ciphertext || mac
    [ephemeral_key.public, ciphertext, mac].concat()
}
```

**字段说明**:
- `signature`: 证明发起方拥有私钥
- `initiator-public-key`: 用于后续密钥派生
- `initiator-nonce`: 防重放攻击
- `version`: 协议版本号 (当前为 5)

---

### ack 消息 (接收方 → 发起方)

**消息结构**:
```
ack = ack-size || enc-ack-body
ack-size = size of enc-ack-body, encoded as a big-endian 16-bit integer
enc-ack-body = ECIES_encrypt(initiator-public-key, ack-body)

ack-body = [
  recipient-public-key,   // 接收方公钥 (64字节)
  recipient-nonce,        // 随机数 (32字节)
  version                 // RLPx 版本 (4)
]
```

**解密和验证**:
```rust
fn handle_auth(auth_msg: &[u8], recipient_private_key: &SecretKey) -> Result<AuthData> {
    // 1. 解析大小
    let size = u16::from_be_bytes(&auth_msg[0..2]);

    // 2. ECIES 解密
    let auth_body = ecies_decrypt(&auth_msg[2..], recipient_private_key)?;

    // 3. RLP 解码
    let decoded: Vec<Vec<u8>> = rlp::decode_list(&auth_body)?;

    // 4. 验证签名
    let signature = Signature::from_slice(&decoded[0])?;
    let initiator_pubkey = PublicKey::from_slice(&decoded[1])?;
    let initiator_nonce = &decoded[2];

    let message = keccak256([initiator_nonce, recipient_pubkey.serialize()].concat());
    signature.verify(&message, &initiator_pubkey)?;

    Ok(AuthData {
        remote_public_key: initiator_pubkey,
        remote_nonce: initiator_nonce,
        version: decoded[3][0],
    })
}
```

---

### 密钥派生 (Key Derivation)

握手完成后，双方派生相同的加密密钥：

```rust
fn derive_secrets(
    initiator_nonce: &[u8],
    recipient_nonce: &[u8],
    shared_secret: &[u8],
) -> Secrets {
    let h_nonce = keccak256([recipient_nonce, initiator_nonce].concat());

    // 计算共享密钥
    let shared_secret_hash = keccak256(shared_secret);

    // 派生 MAC 密钥和加密密钥
    let aes_secret = keccak256([shared_secret_hash, h_nonce].concat());
    let mac_secret = keccak256([shared_secret_hash, aes_secret].concat());

    // 初始化 MAC 状态
    let egress_mac = keccak256([mac_secret, recipient_nonce].concat());
    let ingress_mac = keccak256([mac_secret, initiator_nonce].concat());

    Secrets {
        aes_secret,       // AES-256-CTR 密钥
        mac_secret,       // MAC 密钥
        egress_mac,       // 发送 MAC 状态
        ingress_mac,      // 接收 MAC 状态
    }
}
```

**密钥用途**:
- `aes_secret`: AES-256-CTR 帧加密
- `mac_secret`: HMAC 计算
- `egress_mac`: 发送方向的 MAC 累加器
- `ingress_mac`: 接收方向的 MAC 累加器

---

## 📦 帧传输 (Framing)

### 帧结构

```
frame = header || header-mac || frame-data || frame-mac

header = frame-size || header-data || padding
header-size = 16 bytes (固定)

header-data:
  - capability-id (1 byte)    // 子协议 ID
  - context-id (variable)     // 消息类型 ID
  - padding (to 16 bytes)

frame-size = 3 bytes big-endian integer
frame-data = RLP-encoded message payload
```

**完整帧格式**:
```
┌────────────────────────────────────────────────────────────┐
│ Frame Header (16 bytes, encrypted)                         │
├───────────────────┬────────────────────────────────────────┤
│ frame-size (3)    │ header-data (variable) │ padding       │
└───────────────────┴────────────────────────────────────────┘
┌────────────────────────────────────────────────────────────┐
│ Header MAC (16 bytes)                                      │
└────────────────────────────────────────────────────────────┘
┌────────────────────────────────────────────────────────────┐
│ Frame Data (variable, encrypted)                           │
├────────────────────────────────────────────────────────────┤
│ RLP-encoded message (padded to 16-byte alignment)          │
└────────────────────────────────────────────────────────────┘
┌────────────────────────────────────────────────────────────┐
│ Frame MAC (16 bytes)                                       │
└────────────────────────────────────────────────────────────┘
```

### 帧发送

```rust
fn send_frame(
    secrets: &mut Secrets,
    capability_id: u8,
    message_id: u8,
    payload: &[u8],
) -> Vec<u8> {
    // 1. 构造 header-data
    let header_data = [capability_id, message_id];

    // 2. RLP 编码 payload
    let frame_data = rlp::encode(payload);

    // 3. 计算帧大小（填充到 16 字节对齐）
    let frame_size = ((frame_data.len() + 15) / 16) * 16;
    let padded_data = pad_to_16_bytes(frame_data);

    // 4. 构造并加密 header
    let mut header = vec![0u8; 16];
    header[0..3].copy_from_slice(&(frame_size as u32).to_be_bytes()[1..4]);
    header[3..5].copy_from_slice(&header_data);

    let encrypted_header = aes_256_ctr_encrypt(&header, &secrets.aes_secret);

    // 5. 更新并计算 header-mac
    update_mac(&mut secrets.egress_mac, &encrypted_header);
    let header_mac = secrets.egress_mac[0..16].to_vec();

    // 6. 加密 frame-data
    let encrypted_data = aes_256_ctr_encrypt(&padded_data, &secrets.aes_secret);

    // 7. 更新并计算 frame-mac
    update_mac(&mut secrets.egress_mac, &encrypted_data);
    let frame_mac = secrets.egress_mac[0..16].to_vec();

    // 8. 组装完整帧
    [encrypted_header, header_mac, encrypted_data, frame_mac].concat()
}
```

### 帧接收

```rust
fn receive_frame(
    secrets: &mut Secrets,
    stream: &mut TcpStream,
) -> Result<(u8, u8, Vec<u8>)> {
    // 1. 读取加密的 header (16 bytes)
    let mut encrypted_header = [0u8; 16];
    stream.read_exact(&mut encrypted_header)?;

    // 2. 读取 header-mac (16 bytes)
    let mut header_mac = [0u8; 16];
    stream.read_exact(&mut header_mac)?;

    // 3. 验证 header-mac
    update_mac(&mut secrets.ingress_mac, &encrypted_header);
    if secrets.ingress_mac[0..16] != header_mac {
        return Err(Error::InvalidHeaderMac);
    }

    // 4. 解密 header
    let header = aes_256_ctr_decrypt(&encrypted_header, &secrets.aes_secret);

    // 5. 解析 header
    let frame_size = u32::from_be_bytes([0, header[0], header[1], header[2]]) as usize;
    let capability_id = header[3];
    let message_id = header[4];

    // 6. 读取加密的 frame-data
    let mut encrypted_data = vec![0u8; frame_size];
    stream.read_exact(&mut encrypted_data)?;

    // 7. 读取 frame-mac
    let mut frame_mac = [0u8; 16];
    stream.read_exact(&mut frame_mac)?;

    // 8. 验证 frame-mac
    update_mac(&mut secrets.ingress_mac, &encrypted_data);
    if secrets.ingress_mac[0..16] != frame_mac {
        return Err(Error::InvalidFrameMac);
    }

    // 9. 解密 frame-data
    let frame_data = aes_256_ctr_decrypt(&encrypted_data, &secrets.aes_secret);

    // 10. RLP 解码
    let payload = rlp::decode(&frame_data)?;

    Ok((capability_id, message_id, payload))
}
```

### MAC 更新算法

```rust
fn update_mac(mac_state: &mut [u8; 32], data: &[u8]) {
    // 1. AES-256 加密 MAC 状态
    let encrypted = aes_256_ecb_encrypt(mac_state, mac_state);

    // 2. XOR 数据
    for i in 0..data.len().min(32) {
        encrypted[i] ^= data[i];
    }

    // 3. Keccak-256 更新
    *mac_state = keccak256([mac_state, &encrypted].concat());
}
```

---

## 🔀 多路复用 (Multiplexing)

### Capability 协商

**Hello 消息**:
```
Hello {
  protocol_version: 5,                    // RLPx 版本
  client_id: "RustEth/v0.1.0/linux",     // 客户端标识
  capabilities: [                         // 支持的协议
    ("eth", 68),
    ("snap", 1),
  ],
  listen_port: 30303,                     // 监听端口
  node_id: [0x12, 0x34, ...]             // 节点 ID (64字节)
}
```

**消息格式** (RLP 编码):
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct Hello {
    protocol_version: u8,
    client_id: String,
    capabilities: Vec<(String, u8)>,
    listen_port: u16,
    node_id: [u8; 64],
}
```

**发送 Hello**:
```rust
async fn send_hello(conn: &mut Connection) -> Result<()> {
    let hello = Hello {
        protocol_version: 5,
        client_id: "RustEth/v0.1.0/linux".to_string(),
        capabilities: vec![
            ("eth".to_string(), 68),
            ("snap".to_string(), 1),
        ],
        listen_port: 30303,
        node_id: conn.local_node_id,
    };

    let payload = rlp::encode(&hello);
    conn.send_frame(0, 0x00, &payload).await?;  // capability_id=0 (base protocol)
    Ok(())
}
```

### Capability ID 分配

```
Capability ID 0: Base Protocol (保留)
  - 0x00: Hello
  - 0x01: Disconnect
  - 0x02: Ping
  - 0x03: Pong

Capability ID 1-255: 协商的子协议
  按字母顺序分配:
    ID 1: eth/68
    ID 2: snap/1
    ID 3: wit/0
    ...
```

**示例**:
```
双方协商:
  Node A: [("eth", 68), ("snap", 1), ("wit", 0)]
  Node B: [("eth", 68), ("snap", 1)]

共同协议:
  ID 1: eth/68
  ID 2: snap/1
  (wit/0 不可用)
```

---

## 🛡️ 基础协议消息

### 0x00: Hello

已在上文说明。

---

### 0x01: Disconnect

**用途**: 通知对等节点断开连接

**消息格式**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct Disconnect {
    reason: u8,
}
```

**断开原因代码**:
```
0x00: Disconnect requested
0x01: TCP sub-system error
0x02: Breach of protocol (bad RLP, invalid message)
0x03: Useless peer
0x04: Too many peers
0x05: Already connected
0x06: Incompatible P2P protocol version
0x07: Null node identity received
0x08: Client quitting
0x09: Unexpected identity (same as ours)
0x0a: Identity associated with different connection
0x0b: Timeout on receiving a message
0x10: Subprotocol-specific reasons
```

**发送示例**:
```rust
async fn disconnect(conn: &mut Connection, reason: u8) -> Result<()> {
    let disconnect = Disconnect { reason };
    let payload = rlp::encode(&disconnect);

    conn.send_frame(0, 0x01, &payload).await?;
    conn.close().await?;

    Ok(())
}
```

---

### 0x02: Ping / 0x03: Pong

**用途**: 保持连接活跃，测试延迟

**Ping 消息**:
```rust
// Ping 无 payload
let payload = vec![];
conn.send_frame(0, 0x02, &payload).await?;
```

**Pong 消息**:
```rust
// Pong 无 payload
let payload = vec![];
conn.send_frame(0, 0x03, &payload).await?;
```

**心跳机制**:
```rust
async fn heartbeat_loop(conn: Arc<Mutex<Connection>>) {
    let mut interval = tokio::time::interval(Duration::from_secs(15));

    loop {
        interval.tick().await;

        let mut conn = conn.lock().await;
        if let Err(e) = conn.send_frame(0, 0x02, &[]).await {
            eprintln!("Ping failed: {}", e);
            break;
        }

        // 等待 Pong (带超时)
        match timeout(Duration::from_secs(5), conn.recv_pong()).await {
            Ok(Ok(_)) => {
                // Pong 收到
            }
            _ => {
                eprintln!("Pong timeout, disconnecting");
                let _ = conn.disconnect(0x0b).await;
                break;
            }
        }
    }
}
```

---

## ⚡ 性能优化

### 零拷贝发送

```rust
use bytes::{Bytes, BytesMut};

struct ZeroCopyConnection {
    stream: TcpStream,
    send_buffer: BytesMut,
    secrets: Secrets,
}

impl ZeroCopyConnection {
    async fn send_frame_zero_copy(
        &mut self,
        capability_id: u8,
        message_id: u8,
        payload: Bytes,  // 零拷贝
    ) -> Result<()> {
        // 直接在发送缓冲区构造帧
        self.send_buffer.clear();
        self.send_buffer.reserve(32 + payload.len() + 32);

        // 构造 header
        let header = build_header(capability_id, message_id, payload.len());
        let encrypted_header = encrypt(&header, &self.secrets);
        self.send_buffer.extend_from_slice(&encrypted_header);

        // 计算 header-mac
        let header_mac = compute_mac(&encrypted_header, &mut self.secrets.egress_mac);
        self.send_buffer.extend_from_slice(&header_mac);

        // 加密 payload (直接加密到缓冲区)
        encrypt_in_place(&mut self.send_buffer, &payload, &self.secrets);

        // 计算 frame-mac
        let frame_mac = compute_mac(&payload, &mut self.secrets.egress_mac);
        self.send_buffer.extend_from_slice(&frame_mac);

        // 发送
        self.stream.write_all(&self.send_buffer).await?;

        Ok(())
    }
}
```

### 批量发送

```rust
async fn send_batch(
    conn: &mut Connection,
    messages: Vec<(u8, u8, Vec<u8>)>,  // (cap_id, msg_id, payload)
) -> Result<()> {
    let mut buffer = BytesMut::with_capacity(65536);

    for (cap_id, msg_id, payload) in messages {
        let frame = conn.build_frame(cap_id, msg_id, &payload)?;
        buffer.extend_from_slice(&frame);
    }

    conn.stream.write_all(&buffer).await?;
    Ok(())
}
```

### 并发接收

```rust
async fn concurrent_receiver(
    conn: Arc<Mutex<Connection>>,
    handlers: HashMap<(u8, u8), Box<dyn MessageHandler>>,
) {
    let (tx, mut rx) = mpsc::channel(100);

    // 接收任务
    tokio::spawn(async move {
        loop {
            let mut conn = conn.lock().await;
            match conn.recv_frame().await {
                Ok((cap_id, msg_id, payload)) => {
                    let _ = tx.send((cap_id, msg_id, payload)).await;
                }
                Err(e) => {
                    eprintln!("Receive error: {}", e);
                    break;
                }
            }
        }
    });

    // 处理任务
    while let Some((cap_id, msg_id, payload)) = rx.recv().await {
        if let Some(handler) = handlers.get(&(cap_id, msg_id)) {
            tokio::spawn(async move {
                handler.handle(payload).await;
            });
        }
    }
}
```

---

## 🔍 调试和监控

### 连接状态跟踪

```rust
#[derive(Debug)]
struct ConnectionMetrics {
    connected_at: Instant,
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
    messages_sent: AtomicU64,
    messages_received: AtomicU64,
    last_ping: AtomicU64,  // timestamp
    last_pong: AtomicU64,
    errors: AtomicU64,
}

impl ConnectionMetrics {
    fn record_send(&self, bytes: usize) {
        self.bytes_sent.fetch_add(bytes as u64, Ordering::Relaxed);
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
    }

    fn record_receive(&self, bytes: usize) {
        self.bytes_received.fetch_add(bytes as u64, Ordering::Relaxed);
        self.messages_received.fetch_add(1, Ordering::Relaxed);
    }

    fn latency(&self) -> Duration {
        let ping = self.last_ping.load(Ordering::Relaxed);
        let pong = self.last_pong.load(Ordering::Relaxed);
        Duration::from_millis(pong.saturating_sub(ping))
    }
}
```

### 日志记录

```rust
use tracing::{info, warn, error, debug};

async fn handle_message(
    cap_id: u8,
    msg_id: u8,
    payload: &[u8],
) -> Result<()> {
    debug!(
        cap_id = cap_id,
        msg_id = msg_id,
        payload_size = payload.len(),
        "Received message"
    );

    match (cap_id, msg_id) {
        (0, 0x00) => {
            let hello: Hello = rlp::decode(payload)?;
            info!(
                peer_client = %hello.client_id,
                capabilities = ?hello.capabilities,
                "Received Hello"
            );
        }
        (0, 0x01) => {
            let disconnect: Disconnect = rlp::decode(payload)?;
            warn!(reason = disconnect.reason, "Peer disconnected");
        }
        _ => {
            debug!("Unknown message type");
        }
    }

    Ok(())
}
```

---

## 📚 参考资源

### 官方规范
- [RLPx 规范](https://github.com/ethereum/devp2p/blob/master/rlpx.md)
- [EIP-8: devp2p Forward Compatibility](https://eips.ethereum.org/EIPS/eip-8)

### 参考实现
- [Geth RLPx](https://github.com/ethereum/go-ethereum/tree/master/p2p/rlpx)
- [Reth RLPx](https://github.com/paradigmxyz/reth/tree/main/crates/net/rlpx)
- [Parity RLPx](https://github.com/paritytech/parity-ethereum/tree/master/util/network-devp2p)

### 密码学库
- [secp256k1](https://github.com/rust-bitcoin/rust-secp256k1) - ECDSA 签名
- [aes](https://github.com/RustCrypto/block-ciphers/tree/master/aes) - AES 加密
- [sha3](https://github.com/RustCrypto/hashes/tree/master/sha3) - Keccak-256

---

**文档版本**: v1.0
**更新日期**: 2025-11-09
**适用于**: Ethereum DevP2P RLPx Transport Protocol
