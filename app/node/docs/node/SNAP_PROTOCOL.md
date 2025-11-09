# snap/1 协议详解

## 📚 概述

**snap** (Snapshot Protocol) 是以太坊的快照同步协议，允许节点快速下载和验证区块链状态，而无需执行所有历史交易。

- **协议版本**: snap/1
- **协议 ID**: "snap"
- **版本号**: 1
- **传输层**: RLPx/TCP
- **端口**: 30303 (与 eth 协议共用)

**标准来源**: https://github.com/ethereum/devp2p/blob/master/caps/snap.md

---

## 🎯 设计目标

### 传统同步 vs 快照同步

| 特性 | 完整同步 | 快速同步 | 快照同步 |
|------|---------|---------|---------|
| **下载区块** | ✅ 全部 | ✅ 全部 | ✅ 最近的 |
| **执行交易** | ✅ 全部 | ❌ 不执行 | ❌ 不执行 |
| **下载状态** | ❌ 自己生成 | ✅ 最新状态 | ✅ 最新状态 |
| **同步时间** | 数周 | 数小时 | **10-30分钟** |
| **带宽消耗** | 极高 | 高 | **中等** |
| **验证** | 完整 | 部分 | **Merkle 证明** |

### 快照同步优势

- ⚡ **速度**: 比完整同步快 100+ 倍
- 📦 **压缩**: Snappy 压缩减少 50-70% 带宽
- 🔍 **验证**: Merkle 证明保证数据完整性
- 🔄 **并行**: 支持并发下载多个范围
- 🎯 **精准**: 按需下载所需状态

---

## 📋 消息类型总览

| 消息 ID | 名称 | 用途 |
|---------|------|------|
| **0x00** | GetAccountRange | 请求账户范围 |
| **0x01** | AccountRange | 账户范围响应 |
| **0x02** | GetStorageRanges | 请求存储范围 |
| **0x03** | StorageRanges | 存储范围响应 |
| **0x04** | GetByteCodes | 请求字节码 |
| **0x05** | ByteCodes | 字节码响应 |
| **0x06** | GetTrieNodes | 请求 Trie 节点 |
| **0x07** | TrieNodes | Trie 节点响应 |

---

## 🌳 状态树结构

### 以太坊状态 Merkle Patricia Trie

```
                     State Root
                         │
         ┌───────────────┼───────────────┐
         │               │               │
    Account 1       Account 2       Account 3
    (hash: 0x00..)  (hash: 0x55..)  (hash: 0xaa..)
         │
    ┌────┴────┐
    │         │
  Nonce   Balance
    │         │
Storage Root  Code Hash
    │
Storage Trie
    │
 ┌──┴──┐
Slot1  Slot2
```

**账户结构**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct Account {
    nonce: u64,
    balance: U256,
    storage_root: H256,  // 存储树根
    code_hash: H256,     // 代码哈希
}
```

---

## 📥 0x00/0x01: GetAccountRange / AccountRange

### GetAccountRange (请求)

**用途**: 请求指定哈希范围内的账户

**消息格式**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct GetAccountRange {
    request_id: u64,        // 请求 ID
    root_hash: H256,        // 状态根哈希
    starting_hash: H256,    // 起始账户哈希
    limit_hash: H256,       // 限制账户哈希 (exclusive)
    response_bytes: u64,    // 响应大小限制
}
```

**请求示例**:
```rust
async fn request_account_range(
    conn: &mut RlpxConnection,
    state_root: H256,
    start: H256,
    limit: H256,
) -> Result<AccountRangeResponse> {
    let request = GetAccountRange {
        request_id: generate_request_id(),
        root_hash: state_root,
        starting_hash: start,
        limit_hash: limit,
        response_bytes: 500_000,  // ~500KB
    };

    let payload = rlp::encode(&request);
    conn.send_message(SNAP_CAPABILITY_ID, 0x00, &payload).await?;

    let response = wait_for_response(request.request_id).await?;
    Ok(response)
}
```

**范围查询策略**:
```rust
fn split_account_space(num_peers: usize) -> Vec<(H256, H256)> {
    // 将整个账户空间 [0x00..00, 0xff..ff] 分成 num_peers 个范围
    let chunk_size = U256::MAX / U256::from(num_peers);
    let mut ranges = Vec::new();

    for i in 0..num_peers {
        let start = chunk_size * U256::from(i);
        let end = if i == num_peers - 1 {
            U256::MAX
        } else {
            chunk_size * U256::from(i + 1)
        };

        ranges.push((
            H256::from(start),
            H256::from(end),
        ));
    }

    ranges
}

// 示例: 8 个并发请求
let ranges = split_account_space(8);
for (start, limit) in ranges {
    tokio::spawn(async move {
        request_account_range(state_root, start, limit).await
    });
}
```

---

### AccountRange (响应)

**消息格式**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct AccountRange {
    request_id: u64,
    accounts: Vec<(H256, Account)>,  // [(account_hash, account), ...]
    proof: Vec<Vec<u8>>,              // Merkle proof (RLP 编码的节点)
}
```

**Account 字段**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct Account {
    nonce: u64,
    balance: U256,
    storage_root: H256,
    code_hash: H256,
}
```

**Merkle Proof 验证**:
```rust
fn verify_account_range(
    response: &AccountRange,
    state_root: &H256,
    starting_hash: &H256,
    limit_hash: &H256,
) -> Result<()> {
    // 1. 验证账户连续性
    for i in 0..response.accounts.len() - 1 {
        let current = &response.accounts[i].0;
        let next = &response.accounts[i + 1].0;

        if current >= next {
            return Err(Error::InvalidAccountOrder);
        }
    }

    // 2. 验证范围
    if let Some(first) = response.accounts.first() {
        if &first.0 < starting_hash {
            return Err(Error::AccountOutOfRange);
        }
    }

    if let Some(last) = response.accounts.last() {
        if &last.0 >= limit_hash {
            return Err(Error::AccountOutOfRange);
        }
    }

    // 3. 验证 Merkle Proof
    verify_merkle_proof(
        &response.accounts,
        &response.proof,
        state_root,
    )?;

    Ok(())
}

fn verify_merkle_proof(
    accounts: &[(H256, Account)],
    proof: &[Vec<u8>],
    expected_root: &H256,
) -> Result<()> {
    // 从 proof 重建状态树的一部分
    let mut trie = PartialTrie::new();

    // 1. 添加 proof 节点
    for node in proof {
        trie.insert_node(node)?;
    }

    // 2. 添加账户数据
    for (hash, account) in accounts {
        let account_rlp = rlp::encode(account);
        trie.insert(hash, &account_rlp)?;
    }

    // 3. 计算根哈希
    let computed_root = trie.root()?;

    if &computed_root != expected_root {
        return Err(Error::InvalidProof);
    }

    Ok(())
}
```

**处理响应**:
```rust
async fn handle_account_range(response: AccountRange) -> Result<()> {
    // 1. 验证响应
    verify_account_range(&response, &state_root, &start, &limit)?;

    // 2. 存储账户
    for (hash, account) in response.accounts {
        db.insert_account(hash, account)?;

        // 3. 如果有存储，标记需要下载
        if account.storage_root != EMPTY_ROOT_HASH {
            pending_storage.insert(hash, account.storage_root);
        }

        // 4. 如果有代码，标记需要下载
        if account.code_hash != EMPTY_CODE_HASH {
            pending_codes.insert(account.code_hash);
        }
    }

    Ok(())
}
```

---

## 💾 0x02/0x03: GetStorageRanges / StorageRanges

### GetStorageRanges (请求)

**用途**: 请求多个账户的存储范围

**消息格式**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct GetStorageRanges {
    request_id: u64,
    root_hash: H256,              // 状态根哈希
    account_hashes: Vec<H256>,    // 账户哈希列表 (最多 256 个)
    starting_hash: H256,          // 存储起始哈希
    limit_hash: H256,             // 存储限制哈希
    response_bytes: u64,          // 响应大小限制
}
```

**请求策略**:
```rust
async fn request_storage_ranges(
    accounts_with_storage: Vec<(H256, H256)>,  // (account_hash, storage_root)
) -> Result<()> {
    const BATCH_SIZE: usize = 256;

    for chunk in accounts_with_storage.chunks(BATCH_SIZE) {
        let account_hashes: Vec<H256> = chunk.iter().map(|(h, _)| *h).collect();

        let request = GetStorageRanges {
            request_id: generate_request_id(),
            root_hash: state_root,
            account_hashes,
            starting_hash: H256::zero(),
            limit_hash: H256::from([0xff; 32]),
            response_bytes: 500_000,
        };

        send_request(request).await?;
    }

    Ok(())
}
```

---

### StorageRanges (响应)

**消息格式**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct StorageRanges {
    request_id: u64,
    slots: Vec<Vec<StorageSlot>>,  // 每个账户的存储槽列表
    proof: Vec<Vec<u8>>,           // Merkle proof
}

#[derive(RlpEncodable, RlpDecodable)]
struct StorageSlot {
    hash: H256,    // 存储槽哈希
    data: Vec<u8>, // 存储槽数据
}
```

**存储槽哈希**:
```rust
fn storage_slot_hash(slot: U256) -> H256 {
    // 存储槽键的 keccak256 哈希
    keccak256(slot.to_be_bytes())
}

// 示例
let slot_0_hash = storage_slot_hash(U256::zero());
let slot_1_hash = storage_slot_hash(U256::one());
```

**处理响应**:
```rust
async fn handle_storage_ranges(
    response: StorageRanges,
    account_hashes: &[H256],
) -> Result<()> {
    // 1. 验证槽列表数量匹配
    if response.slots.len() != account_hashes.len() {
        return Err(Error::InvalidResponse);
    }

    // 2. 处理每个账户的存储
    for (i, account_hash) in account_hashes.iter().enumerate() {
        let slots = &response.slots[i];

        for slot in slots {
            // 存储到数据库
            db.insert_storage(account_hash, &slot.hash, &slot.data)?;
        }
    }

    // 3. 验证 Merkle Proof
    verify_storage_proof(&response, account_hashes)?;

    Ok(())
}
```

---

## 📜 0x04/0x05: GetByteCodes / ByteCodes

### GetByteCodes (请求)

**用途**: 请求合约字节码

**消息格式**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct GetByteCodes {
    request_id: u64,
    hashes: Vec<H256>,      // 代码哈希列表
    response_bytes: u64,    // 响应大小限制
}
```

**批量请求**:
```rust
async fn request_bytecodes(code_hashes: Vec<H256>) -> Result<()> {
    const BATCH_SIZE: usize = 256;

    for chunk in code_hashes.chunks(BATCH_SIZE) {
        let request = GetByteCodes {
            request_id: generate_request_id(),
            hashes: chunk.to_vec(),
            response_bytes: 2_000_000,  // 2MB
        };

        send_request(request).await?;
    }

    Ok(())
}
```

---

### ByteCodes (响应)

**消息格式**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct ByteCodes {
    request_id: u64,
    codes: Vec<Vec<u8>>,  // 字节码列表（顺序对应请求）
}
```

**处理响应**:
```rust
async fn handle_bytecodes(
    response: ByteCodes,
    requested_hashes: &[H256],
) -> Result<()> {
    if response.codes.len() != requested_hashes.len() {
        return Err(Error::InvalidResponse);
    }

    for (i, code) in response.codes.iter().enumerate() {
        let expected_hash = &requested_hashes[i];

        // 验证代码哈希
        let computed_hash = keccak256(code);
        if &computed_hash != expected_hash {
            return Err(Error::InvalidCodeHash);
        }

        // 存储字节码
        db.insert_code(expected_hash, code)?;
    }

    Ok(())
}
```

---

## 🌲 0x06/0x07: GetTrieNodes / TrieNodes

### GetTrieNodes (请求)

**用途**: 请求缺失的 Trie 节点（修复空洞）

**消息格式**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct GetTrieNodes {
    request_id: u64,
    root_hash: H256,               // 状态根哈希
    paths: Vec<Vec<Vec<u8>>>,      // Trie 路径列表
    response_bytes: u64,
}
```

**Trie 路径**:
```
路径是从根到目标节点的所有分支选择

示例:
  账户哈希: 0x1234...
  路径: [[0x1], [0x2], [0x3], [0x4], ...]

  每个元素是在该层选择的分支编号
```

**请求示例**:
```rust
async fn request_missing_trie_nodes(
    state_root: H256,
    missing_paths: Vec<Vec<Vec<u8>>>,
) -> Result<()> {
    let request = GetTrieNodes {
        request_id: generate_request_id(),
        root_hash: state_root,
        paths: missing_paths,
        response_bytes: 500_000,
    };

    send_request(request).await?;
    Ok(())
}
```

---

### TrieNodes (响应)

**消息格式**:
```rust
#[derive(RlpEncodable, RlpDecodable)]
struct TrieNodes {
    request_id: u64,
    nodes: Vec<Vec<u8>>,  // RLP 编码的 Trie 节点
}
```

**处理响应**:
```rust
async fn handle_trie_nodes(response: TrieNodes) -> Result<()> {
    for node in response.nodes {
        // 1. 计算节点哈希
        let node_hash = keccak256(&node);

        // 2. 解析节点
        let trie_node = parse_trie_node(&node)?;

        // 3. 存储节点
        db.insert_trie_node(&node_hash, &node)?;
    }

    Ok(())
}

fn parse_trie_node(data: &[u8]) -> Result<TrieNode> {
    let decoded: Vec<Vec<u8>> = rlp::decode_list(data)?;

    match decoded.len() {
        2 => {
            // Leaf or Extension node
            let key = decoded[0].clone();
            let value = decoded[1].clone();

            if is_leaf(&key) {
                Ok(TrieNode::Leaf { key, value })
            } else {
                Ok(TrieNode::Extension { key, value })
            }
        }
        17 => {
            // Branch node
            let mut children = [H256::zero(); 16];
            for i in 0..16 {
                if !decoded[i].is_empty() {
                    children[i] = H256::from_slice(&decoded[i]);
                }
            }
            let value = decoded[16].clone();
            Ok(TrieNode::Branch { children, value })
        }
        _ => Err(Error::InvalidTrieNode),
    }
}
```

---

## 🔄 完整快照同步流程

```rust
async fn snapshot_sync() -> Result<()> {
    // 1. 获取目标状态根
    let target_block = get_best_peer_block().await?;
    let state_root = target_block.state_root;

    println!("Starting snapshot sync at block {}", target_block.number);

    // 2. 并行下载账户
    let accounts = download_accounts_parallel(state_root).await?;
    println!("Downloaded {} accounts", accounts.len());

    // 3. 收集需要下载的存储和代码
    let mut storage_tasks = Vec::new();
    let mut code_hashes = Vec::new();

    for (hash, account) in &accounts {
        if account.storage_root != EMPTY_ROOT_HASH {
            storage_tasks.push((*hash, account.storage_root));
        }
        if account.code_hash != EMPTY_CODE_HASH {
            code_hashes.push(account.code_hash);
        }
    }

    // 4. 并行下载存储
    download_storage_parallel(storage_tasks).await?;
    println!("Downloaded storage");

    // 5. 批量下载字节码
    download_bytecodes(code_hashes).await?;
    println!("Downloaded bytecodes");

    // 6. 修复缺失的 Trie 节点
    heal_trie(state_root).await?;
    println!("Healed trie");

    // 7. 验证完整性
    verify_state_integrity(state_root)?;
    println!("Snapshot sync complete!");

    Ok(())
}

async fn download_accounts_parallel(state_root: H256) -> Result<Vec<(H256, Account)>> {
    const NUM_WORKERS: usize = 8;

    let ranges = split_account_space(NUM_WORKERS);
    let mut tasks = Vec::new();

    for (start, limit) in ranges {
        tasks.push(tokio::spawn(async move {
            download_account_range(state_root, start, limit).await
        }));
    }

    let results = futures::future::try_join_all(tasks).await?;
    let accounts: Vec<_> = results.into_iter().flatten().collect();

    Ok(accounts)
}

async fn download_account_range(
    state_root: H256,
    mut start: H256,
    limit: H256,
) -> Result<Vec<(H256, Account)>> {
    let mut accounts = Vec::new();

    loop {
        let response = request_account_range(state_root, start, limit).await?;

        if response.accounts.is_empty() {
            break;  // 范围完成
        }

        accounts.extend(response.accounts.clone());

        // 更新起始点
        start = response.accounts.last().unwrap().0;
        start = next_hash(&start);

        if start >= limit {
            break;
        }
    }

    Ok(accounts)
}

async fn heal_trie(state_root: H256) -> Result<()> {
    loop {
        // 1. 查找缺失的节点
        let missing_paths = find_missing_trie_nodes(state_root)?;

        if missing_paths.is_empty() {
            break;  // Trie 完整
        }

        println!("Found {} missing trie nodes", missing_paths.len());

        // 2. 请求缺失的节点
        request_missing_trie_nodes(state_root, missing_paths).await?;
    }

    Ok(())
}

fn find_missing_trie_nodes(state_root: H256) -> Result<Vec<Vec<Vec<u8>>>> {
    let mut missing = Vec::new();
    let mut queue = vec![(state_root, Vec::new())];

    while let Some((node_hash, path)) = queue.pop() {
        // 检查节点是否存在
        if let Some(node_data) = db.get_trie_node(&node_hash)? {
            // 节点存在，遍历子节点
            let node = parse_trie_node(&node_data)?;

            match node {
                TrieNode::Branch { children, .. } => {
                    for (i, child_hash) in children.iter().enumerate() {
                        if child_hash != &H256::zero() {
                            let mut child_path = path.clone();
                            child_path.push(vec![i as u8]);
                            queue.push((*child_hash, child_path));
                        }
                    }
                }
                TrieNode::Extension { value, .. } => {
                    let child_hash = H256::from_slice(&value);
                    queue.push((child_hash, path.clone()));
                }
                _ => {}
            }
        } else {
            // 节点缺失
            missing.push(path);
        }
    }

    Ok(missing)
}
```

---

## ⚡ 性能优化

### Snappy 压缩

所有 snap 协议消息都使用 Snappy 压缩:

```rust
use snap::raw::{Encoder, Decoder};

fn compress_message(data: &[u8]) -> Vec<u8> {
    let mut encoder = Encoder::new();
    encoder.compress_vec(data).unwrap()
}

fn decompress_message(compressed: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = Decoder::new();
    decoder.decompress_vec(compressed)
        .map_err(|e| Error::DecompressionFailed(e))
}

// 在发送前压缩
let payload = rlp::encode(&message);
let compressed = compress_message(&payload);
conn.send_message(SNAP_CAPABILITY_ID, msg_id, &compressed).await?;

// 接收后解压
let compressed_data = conn.recv_message().await?;
let payload = decompress_message(&compressed_data)?;
let message = rlp::decode(&payload)?;
```

**压缩效果**:
- 账户数据: ~60% 压缩率
- 存储数据: ~70% 压缩率
- Trie 节点: ~50% 压缩率

---

### 内存优化

```rust
use parking_lot::RwLock;
use lru::LruCache;

struct SnapSyncState {
    // LRU 缓存最近请求的数据
    account_cache: RwLock<LruCache<H256, Account>>,
    storage_cache: RwLock<LruCache<(H256, H256), Vec<u8>>>,

    // 待处理队列（限制内存）
    pending_accounts: RwLock<Vec<H256>>,
    pending_storage: RwLock<Vec<(H256, H256)>>,
    pending_codes: RwLock<Vec<H256>>,
}

impl SnapSyncState {
    fn new() -> Self {
        Self {
            account_cache: RwLock::new(LruCache::new(10000)),
            storage_cache: RwLock::new(LruCache::new(100000)),
            pending_accounts: RwLock::new(Vec::new()),
            pending_storage: RwLock::new(Vec::new()),
            pending_codes: RwLock::new(Vec::new()),
        }
    }

    fn add_account(&self, hash: H256, account: Account) {
        self.account_cache.write().put(hash, account);
    }

    fn get_account(&self, hash: &H256) -> Option<Account> {
        self.account_cache.write().get(hash).cloned()
    }
}
```

---

## 📊 监控指标

```rust
struct SnapSyncMetrics {
    // 进度
    total_accounts: AtomicU64,
    synced_accounts: AtomicU64,
    synced_storage_slots: AtomicU64,
    synced_bytecodes: AtomicU64,

    // 速度
    accounts_per_second: AtomicU64,
    bytes_downloaded: AtomicU64,

    // 网络
    active_requests: AtomicU64,
    failed_requests: AtomicU64,
}

impl SnapSyncMetrics {
    fn progress_percentage(&self) -> f64 {
        let total = self.total_accounts.load(Ordering::Relaxed);
        let synced = self.synced_accounts.load(Ordering::Relaxed);

        if total == 0 {
            0.0
        } else {
            (synced as f64 / total as f64) * 100.0
        }
    }

    fn log_status(&self) {
        println!(
            "Snap sync: {:.2}% ({}/{}) accounts, {} KB/s",
            self.progress_percentage(),
            self.synced_accounts.load(Ordering::Relaxed),
            self.total_accounts.load(Ordering::Relaxed),
            self.bytes_downloaded.load(Ordering::Relaxed) / 1024,
        );
    }
}
```

---

## 🔍 故障排查

### 常见问题

**1. Merkle Proof 验证失败**:
```
原因: 状态根已更改（新区块产生）
解决: 使用 finalized 或 safe 区块的状态根
```

**2. 存储下载不完整**:
```
原因: 响应大小限制导致部分存储未返回
解决: 重复请求直到获取完整存储
```

**3. Trie 修复循环**:
```
原因: 状态持续变化
解决: 使用固定的历史状态根
```

---

## 📚 参考资源

### 官方规范
- [snap/1 规范](https://github.com/ethereum/devp2p/blob/master/caps/snap.md)
- [Merkle Patricia Trie](https://ethereum.org/en/developers/docs/data-structures-and-encoding/patricia-merkle-trie/)

### 参考实现
- [Geth snap sync](https://github.com/ethereum/go-ethereum/tree/master/eth/protocols/snap)
- [Reth snap sync](https://github.com/paradigmxyz/reth)

### 工具
- [snap](https://github.com/google/snappy) - Snappy 压缩库

---

**文档版本**: v1.0
**更新日期**: 2025-11-09
**适用于**: Ethereum snap/1 Snapshot Sync Protocol
