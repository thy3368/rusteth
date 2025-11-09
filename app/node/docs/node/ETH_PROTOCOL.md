# eth/68 协议详解

## 📚 概述

**eth** 协议是以太坊执行层节点间同步区块和交易的核心应用层协议，运行在 RLPx 传输层之上。

- **当前版本**: eth/68
- **协议 ID**: "eth"
- **版本号**: 68
- **传输层**: RLPx/TCP
- **端口**: 30303 (默认)

**标准来源**: https://github.com/ethereum/devp2p/blob/master/caps/eth.md

---

## 🔄 版本演进

| 版本 | 发布时间 | 主要变更 |
|------|----------|----------|
| **eth/60** | 2015 | 初始版本 |
| **eth/61** | 2015 | 添加 `GetNodeData` |
| **eth/62** | 2015 | 添加 `GetBlockBodies` |
| **eth/63** | 2016 | 添加 `GetReceipts`, `GetNodeData` |
| **eth/64** | 2019 | 添加 `ForkId` 检查 (EIP-2124) |
| **eth/65** | 2020 | 添加 `NewPooledTransactionHashes`, `GetPooledTransactions` |
| **eth/66** | 2021 | 所有请求/响应消息添加 `request_id` |
| **eth/67** | 2021 | 移除 `GetNodeData`, `NodeData` |
| **eth/68** | 2023 | 优化交易广播，移除 legacy 交易哈希通知 |

---

## 📋 消息类型总览

### 状态交换 (握手)
```
0x00: Status     - 握手消息 (必须首先发送)
```

### 区块传播
```
0x01: NewBlockHashes     - 新区块哈希通知
0x07: NewBlock           - 完整新区块广播
```

### 交易传播
```
0x02: Transactions                  - 完整交易广播
0x08: NewPooledTransactionHashes   - 新交易哈希通知
0x09: GetPooledTransactions        - 请求池中交易
0x0a: PooledTransactions           - 池中交易响应
```

### 状态同步
```
0x03: GetBlockHeaders    - 请求区块头
0x04: BlockHeaders       - 区块头响应

0x05: GetBlockBodies     - 请求区块体
0x06: BlockBodies        - 区块体响应

0x0b: GetReceipts        - 请求收据
0x0c: Receipts           - 收据响应
```

---

## 🔐 0x00: Status (握手消息)

### 消息格式

```rust
#[derive(RlpEncodable, RlpDecodable)]
struct Status {
    protocol_version: u32,      // eth 协议版本 (68)
    network_id: u64,            // 网络 ID (1=mainnet, 5=goerli, 11155111=sepolia)
    total_difficulty: U256,     // 总难度 (PoW, PoS 后为 0)
    best_hash: H256,            // 最佳区块哈希
    genesis_hash: H256,         // 创世区块哈希
    fork_id: ForkId,            // 分叉 ID (EIP-2124)
}

#[derive(RlpEncodable, RlpDecodable)]
struct ForkId {
    hash: [u8; 4],              // 分叉哈希
    next: u64,                  // 下一个分叉区块号
}
```

### 发送 Status

```rust
async fn send_status(conn: &mut RlpxConnection) -> Result<()> {
    let status = Status {
        protocol_version: 68,
        network_id: 1,  // mainnet
        total_difficulty: U256::zero(),  // PoS
        best_hash: blockchain.best_block_hash(),
        genesis_hash: blockchain.genesis_hash(),
        fork_id: calculate_fork_id(),
    };

    let payload = rlp::encode(&status);
    conn.send_message(ETH_CAPABILITY_ID, 0x00, &payload).await?;

    Ok(())
}
```

### 验证 Status

```rust
async fn verify_status(remote_status: Status) -> Result<()> {
    // 1. 检查网络 ID
    if remote_status.network_id != local_status.network_id {
        return Err(Error::NetworkMismatch);
    }

    // 2. 检查创世哈希
    if remote_status.genesis_hash != local_status.genesis_hash {
        return Err(Error::GenesisMismatch);
    }

    // 3. 验证 ForkId (EIP-2124)
    if !is_fork_id_compatible(&remote_status.fork_id) {
        return Err(Error::ForkIdMismatch);
    }

    Ok(())
}
```

### ForkId 计算 (EIP-2124)

```rust
fn calculate_fork_id() -> ForkId {
    // 1. 收集所有分叉区块号和时间戳
    let forks = vec![
        1150000,   // Homestead
        1920000,   // DAO Fork
        2463000,   // Tangerine Whistle
        2675000,   // Spurious Dragon
        4370000,   // Byzantium
        7280000,   // Constantinople
        9069000,   // Istanbul
        9200000,   // Muir Glacier
        12244000,  // Berlin
        12965000,  // London
        13773000,  // Arrow Glacier
        15050000,  // Gray Glacier
    ];

    // 2. 计算分叉哈希
    let mut hash = crc32(genesis_hash);
    for fork in &forks {
        hash = crc32_update(hash, fork.to_be_bytes());
    }

    // 3. 找到下一个分叉
    let current_block = blockchain.best_block_number();
    let next_fork = forks.iter()
        .find(|&&f| f > current_block)
        .cloned()
        .unwrap_or(0);

    ForkId {
        hash: hash.to_be_bytes(),
        next: next_fork,
    }
}
```

**ForkId 验证**:
```rust
fn is_fork_id_compatible(remote: &ForkId) -> bool {
    let local = calculate_fork_id();

    // 情况 1: 哈希匹配
    if remote.hash == local.hash {
        return true;
    }

    // 情况 2: 对方在我们过去的分叉上
    if is_past_fork(&remote.hash) {
        return remote.next == 0 || remote.next >= local.next;
    }

    // 情况 3: 对方在我们未来的分叉上
    if is_future_fork(&remote.hash) {
        return true;
    }

    false
}
```

---

## 📦 区块传播

### 0x01: NewBlockHashes

**用途**: 通知对等节点新区块的哈希（轻量级广播）

**消息格式**:
```rust
type NewBlockHashes = Vec<(H256, u64)>;  // [(block_hash, block_number), ...]
```

**发送示例**:
```rust
async fn announce_new_block_hashes(
    conn: &mut RlpxConnection,
    blocks: Vec<(H256, u64)>,
) -> Result<()> {
    let payload = rlp::encode(&blocks);
    conn.send_message(ETH_CAPABILITY_ID, 0x01, &payload).await?;
    Ok(())
}
```

**接收处理**:
```rust
async fn handle_new_block_hashes(hashes: Vec<(H256, u64)>) -> Result<()> {
    for (hash, number) in hashes {
        // 检查是否已有该区块
        if !blockchain.has_block(&hash) {
            // 请求完整区块
            request_block_by_hash(hash).await?;
        }
    }
    Ok(())
}
```

---

### 0x07: NewBlock

**用途**: 广播完整新区块（通常发送给部分对等节点）

**消息格式**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct NewBlock {
    block: Block,            // 完整区块
    total_difficulty: U256,  // 总难度
}

#[derive(RlpEncodable, RlpDecodable)]
struct Block {
    header: BlockHeader,
    transactions: Vec<Transaction>,
    uncles: Vec<BlockHeader>,
}
```

**Block Header**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct BlockHeader {
    parent_hash: H256,
    uncle_hash: H256,
    coinbase: Address,
    state_root: H256,
    transactions_root: H256,
    receipts_root: H256,
    logs_bloom: Bloom,
    difficulty: U256,
    number: u64,
    gas_limit: u64,
    gas_used: u64,
    timestamp: u64,
    extra_data: Vec<u8>,
    mix_hash: H256,
    nonce: u64,
    base_fee_per_gas: Option<U256>,  // EIP-1559
    withdrawals_root: Option<H256>,   // EIP-4895
    blob_gas_used: Option<u64>,       // EIP-4844
    excess_blob_gas: Option<u64>,     // EIP-4844
    parent_beacon_block_root: Option<H256>,  // EIP-4788
}
```

**广播策略**:
```rust
async fn broadcast_new_block(block: Block, td: U256) -> Result<()> {
    let peers = peer_manager.get_all_peers();

    // 策略: sqrt(N) 个节点收到完整区块，其他收到哈希
    let full_broadcast_count = (peers.len() as f64).sqrt() as usize;

    for (i, peer) in peers.iter().enumerate() {
        if i < full_broadcast_count {
            // 发送完整区块
            send_new_block(peer, &block, td).await?;
        } else {
            // 发送区块哈希
            send_new_block_hashes(peer, vec![(block.hash(), block.number)]).await?;
        }
    }

    Ok(())
}
```

---

## 💸 交易传播

### 0x02: Transactions

**用途**: 广播完整交易

**消息格式**:
```rust
type Transactions = Vec<Transaction>;

#[derive(RlpEncodable, RlpDecodable)]
enum Transaction {
    Legacy(LegacyTransaction),        // Type 0
    Eip2930(Eip2930Transaction),      // Type 1
    Eip1559(Eip1559Transaction),      // Type 2
    Eip4844(Eip4844Transaction),      // Type 3 (Blob)
}

#[derive(RlpEncodable, RlpDecodable)]
struct Eip1559Transaction {
    chain_id: u64,
    nonce: u64,
    max_priority_fee_per_gas: U256,
    max_fee_per_gas: U256,
    gas_limit: u64,
    to: Option<Address>,
    value: U256,
    data: Vec<u8>,
    access_list: Vec<AccessListItem>,
    signature_y_parity: bool,
    signature_r: U256,
    signature_s: U256,
}
```

**广播交易**:
```rust
async fn broadcast_transactions(txs: Vec<Transaction>) -> Result<()> {
    let peers = peer_manager.get_all_peers();

    for peer in peers {
        // 过滤对方已知的交易
        let unknown_txs: Vec<_> = txs
            .iter()
            .filter(|tx| !peer.knows_transaction(tx.hash()))
            .cloned()
            .collect();

        if !unknown_txs.is_empty() {
            send_transactions(peer, unknown_txs).await?;

            // 标记为已发送
            for tx in &unknown_txs {
                peer.mark_transaction_sent(tx.hash());
            }
        }
    }

    Ok(())
}
```

---

### 0x08: NewPooledTransactionHashes (eth/68)

**用途**: 通知新交易的哈希（节省带宽）

**eth/68 改进**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct NewPooledTransactionHashes {
    types: Vec<u8>,        // 交易类型列表
    sizes: Vec<u32>,       // 交易大小列表
    hashes: Vec<H256>,     // 交易哈希列表
}
```

**发送示例**:
```rust
async fn announce_new_transactions(txs: Vec<Transaction>) -> Result<()> {
    let types: Vec<u8> = txs.iter().map(|tx| tx.tx_type()).collect();
    let sizes: Vec<u32> = txs.iter().map(|tx| tx.rlp_size()).collect();
    let hashes: Vec<H256> = txs.iter().map(|tx| tx.hash()).collect();

    let announcement = NewPooledTransactionHashes {
        types,
        sizes,
        hashes,
    };

    let payload = rlp::encode(&announcement);

    for peer in peer_manager.get_all_peers() {
        conn.send_message(ETH_CAPABILITY_ID, 0x08, &payload).await?;
    }

    Ok(())
}
```

---

### 0x09: GetPooledTransactions

**用途**: 请求池中的交易

**消息格式** (eth/66+):
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct GetPooledTransactions {
    request_id: u64,
    hashes: Vec<H256>,
}
```

**发送请求**:
```rust
async fn request_pooled_transactions(
    conn: &mut RlpxConnection,
    hashes: Vec<H256>,
) -> Result<Vec<Transaction>> {
    let request_id = generate_request_id();

    let request = GetPooledTransactions {
        request_id,
        hashes: hashes.clone(),
    };

    let payload = rlp::encode(&request);
    conn.send_message(ETH_CAPABILITY_ID, 0x09, &payload).await?;

    // 等待响应
    let response = wait_for_response(request_id).await?;
    Ok(response)
}
```

---

### 0x0a: PooledTransactions

**用途**: 返回请求的交易

**消息格式**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct PooledTransactions {
    request_id: u64,
    transactions: Vec<Transaction>,
}
```

**处理请求**:
```rust
async fn handle_get_pooled_transactions(
    conn: &mut RlpxConnection,
    request: GetPooledTransactions,
) -> Result<()> {
    let transactions: Vec<Transaction> = request
        .hashes
        .iter()
        .filter_map(|hash| txpool.get_transaction(hash))
        .collect();

    let response = PooledTransactions {
        request_id: request.request_id,
        transactions,
    };

    let payload = rlp::encode(&response);
    conn.send_message(ETH_CAPABILITY_ID, 0x0a, &payload).await?;

    Ok(())
}
```

---

## 📥 状态同步

### 0x03: GetBlockHeaders

**用途**: 请求区块头

**消息格式**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct GetBlockHeaders {
    request_id: u64,
    block: BlockId,        // 起始区块
    max_headers: u64,      // 最多返回数量
    skip: u64,             // 跳过间隔
    reverse: bool,         // 反向查询
}

enum BlockId {
    Number(u64),
    Hash(H256),
}
```

**请求示例**:
```rust
// 请求从区块 1000 开始的 100 个区块头
let request = GetBlockHeaders {
    request_id: 1,
    block: BlockId::Number(1000),
    max_headers: 100,
    skip: 0,
    reverse: false,
};

// 请求每隔 2 个区块的区块头（用于快速同步）
let request = GetBlockHeaders {
    request_id: 2,
    block: BlockId::Number(1000),
    max_headers: 50,
    skip: 2,  // 返回 1000, 1003, 1006, ...
    reverse: false,
};

// 反向查询（从最新往旧查）
let request = GetBlockHeaders {
    request_id: 3,
    block: BlockId::Hash(latest_hash),
    max_headers: 192,
    skip: 0,
    reverse: true,
};
```

---

### 0x04: BlockHeaders

**用途**: 返回区块头

**消息格式**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct BlockHeaders {
    request_id: u64,
    headers: Vec<BlockHeader>,
}
```

**处理请求**:
```rust
async fn handle_get_block_headers(
    conn: &mut RlpxConnection,
    request: GetBlockHeaders,
) -> Result<()> {
    // 1. 找到起始区块
    let start_block = match request.block {
        BlockId::Number(n) => blockchain.block_by_number(n)?,
        BlockId::Hash(h) => blockchain.block_by_hash(&h)?,
    };

    // 2. 收集区块头
    let mut headers = Vec::new();
    let mut current = start_block.number;

    for _ in 0..request.max_headers {
        if let Some(header) = blockchain.header_by_number(current) {
            headers.push(header);

            // 计算下一个区块号
            if request.reverse {
                if current == 0 {
                    break;
                }
                current = current.saturating_sub(request.skip + 1);
            } else {
                current += request.skip + 1;
            }
        } else {
            break;
        }
    }

    // 3. 发送响应
    let response = BlockHeaders {
        request_id: request.request_id,
        headers,
    };

    let payload = rlp::encode(&response);
    conn.send_message(ETH_CAPABILITY_ID, 0x04, &payload).await?;

    Ok(())
}
```

---

### 0x05: GetBlockBodies

**用途**: 请求区块体（交易和叔块）

**消息格式**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct GetBlockBodies {
    request_id: u64,
    hashes: Vec<H256>,
}
```

---

### 0x06: BlockBodies

**用途**: 返回区块体

**消息格式**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct BlockBodies {
    request_id: u64,
    bodies: Vec<BlockBody>,
}

#[derive(RlpEncodable, RlpDecodable)]
struct BlockBody {
    transactions: Vec<Transaction>,
    uncles: Vec<BlockHeader>,
    withdrawals: Option<Vec<Withdrawal>>,  // EIP-4895
}
```

---

### 0x0b: GetReceipts

**用途**: 请求交易收据

**消息格式**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct GetReceipts {
    request_id: u64,
    hashes: Vec<H256>,  // 区块哈希列表
}
```

---

### 0x0c: Receipts

**用途**: 返回交易收据

**消息格式**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct Receipts {
    request_id: u64,
    receipts: Vec<Vec<Receipt>>,  // 每个区块的收据列表
}

#[derive(RlpEncodable, RlpDecodable)]
struct Receipt {
    tx_type: u8,
    post_state_or_status: Vec<u8>,  // Legacy: post_state, EIP-658: status
    cumulative_gas_used: u64,
    logs_bloom: Bloom,
    logs: Vec<Log>,
}
```

---

## 🔄 完整同步流程

### 快速同步 (Fast Sync)

```rust
async fn fast_sync() -> Result<()> {
    // 1. 获取对等节点的最佳区块
    let best_peer = find_best_peer().await?;
    let target_block = best_peer.best_block_number;

    // 2. 下载区块头（从新到旧）
    let headers = download_headers(target_block, 0).await?;

    // 3. 验证区块头链
    verify_header_chain(&headers)?;

    // 4. 下载区块体
    let bodies = download_bodies(&headers).await?;

    // 5. 下载收据
    let receipts = download_receipts(&headers).await?;

    // 6. 下载状态（使用 snap 协议）
    let state = download_state(&headers.last().unwrap()).await?;

    // 7. 验证状态
    verify_state(&state, &headers.last().unwrap())?;

    Ok(())
}

async fn download_headers(from: u64, to: u64) -> Result<Vec<BlockHeader>> {
    let mut headers = Vec::new();
    let mut current = from;

    while current > to {
        // 批量请求（每次最多 192 个）
        let batch_size = std::cmp::min(192, current - to);

        let request = GetBlockHeaders {
            request_id: generate_request_id(),
            block: BlockId::Number(current),
            max_headers: batch_size,
            skip: 0,
            reverse: true,
        };

        let response = send_and_wait(request).await?;
        headers.extend(response.headers);

        current -= batch_size;
    }

    Ok(headers)
}
```

---

## ⚡ 性能优化

### 并发下载

```rust
use futures::stream::{StreamExt, FuturesUnordered};

async fn parallel_download_bodies(
    hashes: Vec<H256>,
) -> Result<Vec<BlockBody>> {
    const BATCH_SIZE: usize = 128;
    const CONCURRENT_REQUESTS: usize = 8;

    let chunks: Vec<_> = hashes.chunks(BATCH_SIZE).collect();
    let mut futures = FuturesUnordered::new();

    for chunk in chunks {
        futures.push(download_body_batch(chunk.to_vec()));

        // 限制并发数
        while futures.len() >= CONCURRENT_REQUESTS {
            futures.next().await;
        }
    }

    // 等待剩余请求
    let results: Vec<_> = futures.collect().await;
    let bodies = results.into_iter().flatten().collect();

    Ok(bodies)
}
```

### 请求管道化

```rust
struct RequestPipeline {
    pending_requests: HashMap<u64, oneshot::Sender<Response>>,
    next_request_id: AtomicU64,
}

impl RequestPipeline {
    async fn request<Req, Resp>(
        &self,
        request: Req,
    ) -> Result<Resp> {
        let request_id = self.next_request_id.fetch_add(1, Ordering::SeqCst);

        let (tx, rx) = oneshot::channel();
        self.pending_requests.lock().insert(request_id, tx);

        // 发送请求（不等待）
        self.send_request(request_id, request).await?;

        // 异步等待响应
        let response = rx.await?;
        Ok(response)
    }

    async fn handle_response(&self, request_id: u64, response: Response) {
        if let Some(tx) = self.pending_requests.lock().remove(&request_id) {
            let _ = tx.send(response);
        }
    }
}
```

---

## 📊 监控指标

```rust
struct EthProtocolMetrics {
    // 消息计数
    messages_sent: HashMap<u8, AtomicU64>,
    messages_received: HashMap<u8, AtomicU64>,

    // 同步进度
    sync_head: AtomicU64,
    sync_target: AtomicU64,

    // 性能
    block_download_rate: AtomicU64,  // blocks/sec
    tx_propagation_latency: AtomicU64,  // ms
}

impl EthProtocolMetrics {
    fn record_message_sent(&self, msg_id: u8) {
        self.messages_sent[&msg_id].fetch_add(1, Ordering::Relaxed);
    }

    fn sync_progress(&self) -> f64 {
        let head = self.sync_head.load(Ordering::Relaxed);
        let target = self.sync_target.load(Ordering::Relaxed);

        if target == 0 {
            0.0
        } else {
            (head as f64 / target as f64) * 100.0
        }
    }
}
```

---

## 📚 参考资源

### 官方规范
- [eth/68 规范](https://github.com/ethereum/devp2p/blob/master/caps/eth.md)
- [EIP-2124: Fork identifier for chain compatibility checks](https://eips.ethereum.org/EIPS/eip-2124)

### 参考实现
- [Geth eth protocol](https://github.com/ethereum/go-ethereum/tree/master/eth/protocols/eth)
- [Reth eth protocol](https://github.com/paradigmxyz/reth/tree/main/crates/net/eth-wire)

---

**文档版本**: v1.0
**更新日期**: 2025-11-09
**适用于**: Ethereum eth/68 Protocol
