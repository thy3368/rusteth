# RustEth Beacon API 实现状态

## 📊 总览

| 类别 | 已实现 | 计划实现 | 未计划 | 总计 |
|------|--------|----------|--------|------|
| **beacon** 方法 | 0 | 50 | 0 | 50 |
| **validator** 方法 | 0 | 20 | 0 | 20 |
| **node** 方法 | 0 | 7 | 0 | 7 |
| **config** 方法 | 0 | 3 | 0 | 3 |
| **events** 方法 | 0 | 1 | 0 | 1 |
| **debug** 方法 | 0 | 0 | 10 | 10 |
| **rewards** 方法 | 0 | 3 | 0 | 3 |
| **light_client** 方法 | 0 | 0 | 5 | 5 |
| **总计** | **0** | **84** | **15** | **99** |

**实现率**: 0% (0/84 核心方法)

**说明**: RustEth 当前专注于执行层实现，Beacon API 属于共识层，暂未开始实现。

---

## 🎯 实现路线图

### 阶段 0: 架构设计 (Q2 2025)

**目标**: 设计 Beacon Node 架构

- [ ] 定义 Beacon Chain 数据结构
- [ ] 设计状态存储方案（LevelDB/RocksDB）
- [ ] 规划 P2P 网络架构（libp2p/discv5）
- [ ] 设计 REST API 框架（Axum）

**预期产出**: 架构设计文档和 PoC

---

### 阶段 1: 基础节点功能 (Q3 2025)

**目标**: 实现只读 Beacon API，支持基础查询

#### 优先级 1: 节点信息 (7个)

```
⏳ GET /eth/v1/node/version                - 客户端版本
⏳ GET /eth/v1/node/health                 - 健康检查
⏳ GET /eth/v1/node/syncing                - 同步状态
⏳ GET /eth/v1/node/identity               - 节点身份
⏳ GET /eth/v1/node/peers                  - 对等节点列表
⏳ GET /eth/v1/node/peers/{peer_id}        - 特定对等节点
⏳ GET /eth/v1/node/peer_count             - 对等节点数量
```

**实现要点**:
- HTTP 服务器基础设施（复用 Execution API 的 Axum 服务器）
- P2P 网络层集成（libp2p）
- 节点身份管理（ENR）

#### 优先级 2: 配置查询 (3个)

```
⏳ GET /eth/v1/config/fork_schedule        - 分叉时间表
⏳ GET /eth/v1/config/spec                 - 链规范参数
⏳ GET /eth/v1/config/deposit_contract     - 存款合约地址
```

**实现要点**:
- 加载链配置文件（mainnet/testnet）
- 硬编码关键参数（SLOTS_PER_EPOCH=32, SECONDS_PER_SLOT=12）

#### 优先级 3: Genesis 查询 (1个)

```
⏳ GET /eth/v1/beacon/genesis              - 创世信息
```

**实现要点**:
- 存储 genesis_time, genesis_validators_root
- 支持从检查点同步（checkpoint sync）

**阶段 1 目标**: 实现 11 个基础端点，支持节点运行和基本监控

---

### 阶段 2: 状态查询功能 (Q4 2025)

**目标**: 实现信标链状态查询，支持区块和验证者查询

#### 优先级 1: 区块头查询 (2个)

```
⏳ GET /eth/v1/beacon/headers              - 区块头列表
⏳ GET /eth/v1/beacon/headers/{block_id}   - 特定区块头
```

#### 优先级 2: 区块查询 (4个)

```
⏳ GET /eth/v2/beacon/blocks/{block_id}             - 获取区块
⏳ GET /eth/v1/beacon/blocks/{block_id}/root        - 区块根
⏳ GET /eth/v1/beacon/blocks/{block_id}/attestations - 区块证明
⏳ POST /eth/v1/beacon/blocks                        - 发布区块
```

**实现要点**:
- 区块存储（RocksDB/LevelDB）
- 支持多种 block_id 格式（head, finalized, slot, root）
- SSZ 编码/解码

#### 优先级 3: 状态根查询 (3个)

```
⏳ GET /eth/v1/beacon/states/{state_id}/root               - 状态根
⏳ GET /eth/v1/beacon/states/{state_id}/fork               - 分叉信息
⏳ GET /eth/v1/beacon/states/{state_id}/finality_checkpoints - 最终性检查点
```

#### 优先级 4: 验证者查询 (5个)

```
⏳ GET  /eth/v1/beacon/states/{state_id}/validators             - 所有验证者
⏳ GET  /eth/v1/beacon/states/{state_id}/validators/{id}        - 单个验证者
⏳ POST /eth/v1/beacon/states/{state_id}/validators             - 批量查询
⏳ GET  /eth/v1/beacon/states/{state_id}/validator_balances     - 验证者余额
⏳ POST /eth/v1/beacon/states/{state_id}/validator_balances     - 批量余额查询
```

**实现要点**:
- 状态树存储（MPT 或类似结构）
- 验证者索引和公钥映射
- 批量查询优化

#### 优先级 5: 其他状态查询 (5个)

```
⏳ GET /eth/v1/beacon/states/{state_id}/committees        - 委员会信息
⏳ GET /eth/v1/beacon/states/{state_id}/sync_committees   - 同步委员会
⏳ GET /eth/v1/beacon/states/{state_id}/randao            - RANDAO
⏳ GET /eth/v2/beacon/states/{state_id}                   - 完整状态 (谨慎)
```

**阶段 2 目标**: 实现 19 个状态查询端点，支持完整的只读功能

---

### 阶段 3: 验证者客户端支持 (Q1 2026)

**目标**: 实现验证者 API，支持验证者操作

#### 优先级 1: 职责查询 (3个)

```
⏳ GET  /eth/v1/validator/duties/attester/{epoch}   - 证明者职责
⏳ GET  /eth/v1/validator/duties/proposer/{epoch}   - 提议者职责
⏳ POST /eth/v1/validator/duties/sync/{epoch}       - 同步委员会职责
```

**实现要点**:
- 职责计算算法（shuffle, committees）
- Epoch 边界处理

#### 优先级 2: 区块生产 (3个)

```
⏳ GET  /eth/v3/validator/blocks/{slot}                    - 获取待提议区块
⏳ GET  /eth/v1/validator/blinded_blocks/{slot}            - 获取盲区块 (MEV)
⏳ POST /eth/v1/validator/beacon_committee_subscriptions   - 订阅委员会
```

**实现要点**:
- 与执行层 Engine API 集成
- MEV-Boost 集成（可选）
- 聚合选择（aggregate selection）

#### 优先级 3: 证明操作 (3个)

```
⏳ GET  /eth/v1/validator/attestation_data          - 获取证明数据
⏳ GET  /eth/v1/validator/aggregate_attestation     - 获取聚合证明
⏳ POST /eth/v1/validator/aggregate_and_proofs      - 发布聚合证明
```

**实现要点**:
- 证明数据生成
- BLS 签名验证
- 聚合器职责

#### 优先级 4: 同步委员会 (3个)

```
⏳ POST /eth/v1/validator/sync_committee_subscriptions  - 订阅同步委员会
⏳ GET  /eth/v1/validator/sync_committee_contribution   - 获取贡献
⏳ POST /eth/v1/validator/contribution_and_proofs       - 发布贡献证明
```

#### 优先级 5: 验证者管理 (3个)

```
⏳ POST /eth/v1/validator/prepare_beacon_proposer   - 准备提议者
⏳ POST /eth/v1/validator/register_validator        - 注册验证者 (MEV)
⏳ GET  /eth/v1/validator/liveness/{epoch}          - 活跃性查询
```

**阶段 3 目标**: 实现 15 个验证者端点，支持完整的验证者功能

---

### 阶段 4: 交易池和事件 (Q2 2026)

**目标**: 实现交易池和事件订阅

#### 优先级 1: 证明池 (2个)

```
⏳ GET  /eth/v1/beacon/pool/attestations    - 获取待处理证明
⏳ POST /eth/v1/beacon/pool/attestations    - 提交证明
```

#### 优先级 2: 其他池操作 (8个)

```
⏳ GET  /eth/v1/beacon/pool/attester_slashings          - 证明者削减
⏳ POST /eth/v1/beacon/pool/attester_slashings
⏳ GET  /eth/v1/beacon/pool/proposer_slashings          - 提议者削减
⏳ POST /eth/v1/beacon/pool/proposer_slashings
⏳ GET  /eth/v1/beacon/pool/voluntary_exits             - 自愿退出
⏳ POST /eth/v1/beacon/pool/voluntary_exits
⏳ GET  /eth/v1/beacon/pool/bls_to_execution_changes    - BLS 地址变更
⏳ POST /eth/v1/beacon/pool/bls_to_execution_changes
```

**实现要点**:
- 内存池管理（类似执行层 txpool）
- 签名验证
- 防重放攻击

#### 优先级 3: 事件订阅 (1个 SSE)

```
⏳ GET /eth/v1/events?topics=<topics>       - SSE 事件流
```

**支持的事件**:
- `head` - 新链头
- `block` - 新区块
- `attestation` - 新证明
- `voluntary_exit` - 自愿退出
- `finalized_checkpoint` - 最终性检查点
- `chain_reorg` - 链重组

**实现要点**:
- Server-Sent Events (SSE) 协议
- 事件分发机制
- 客户端订阅管理

**阶段 4 目标**: 实现 11 个交易池和事件端点

---

### 阶段 5: 奖励查询 (Q3 2026)

**目标**: 实现奖励和惩罚查询

```
⏳ POST /eth/v1/beacon/rewards/attestations               - 证明奖励
⏳ GET  /eth/v1/beacon/rewards/blocks/{block_id}          - 区块奖励
⏳ POST /eth/v1/beacon/rewards/sync_committee/{block_id}  - 同步委员会奖励
```

**实现要点**:
- 奖励计算算法
- 惩罚计算（inactivity leak）
- 历史数据查询优化

**阶段 5 目标**: 实现 3 个奖励查询端点

---

### 阶段 6: 高级功能 (Q4 2026)

#### 轻客户端支持 (5个) - 可选

```
❓ GET /eth/v1/beacon/light_client/bootstrap/{block_root}     - 引导
❓ GET /eth/v1/beacon/light_client/updates                    - 更新
❓ GET /eth/v1/beacon/light_client/finality_update            - 最终性
❓ GET /eth/v1/beacon/light_client/optimistic_update          - 乐观更新
```

**说明**: 轻客户端协议支持（sync committee 证明）

#### 调试端点 (10个) - 不推荐生产使用

```
❌ GET /eth/v2/debug/beacon/states/{state_id}   - 完整状态转储
❌ GET /eth/v2/debug/beacon/heads                - 所有链头
❌ GET /eth/v1/debug/fork_choice                 - 分叉选择状态
```

**说明**: 调试端点有安全风险，不推荐在生产环境启用

---

## 📝 技术栈规划

### 核心依赖

```toml
[dependencies]
# 已有依赖
tokio = { version = "1.35", features = ["full"] }
axum = { version = "0.7", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Beacon 特定依赖（待添加）
# SSZ 编码
ssz = "0.5"
ssz_derive = "0.5"
tree_hash = "0.5"
tree_hash_derive = "0.5"

# BLS 签名
blst = "0.3"                              # BLS12-381 签名库

# 状态存储
rocksdb = "0.21"                          # 持久化存储

# P2P 网络（已有）
libp2p = { version = "0.54", features = ["tcp", "noise", "yamux", "gossipsub"] }
discv5 = "0.10.2"
enr = "0.12"

# 共识规范类型
ethereum-consensus = "0.1"                # 共识层类型定义
```

### 数据结构设计

```rust
// src/consensus/types/

/// 信标区块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconBlock {
    pub slot: Slot,
    pub proposer_index: ValidatorIndex,
    pub parent_root: Hash256,
    pub state_root: Hash256,
    pub body: BeaconBlockBody,
}

/// 信标状态
#[derive(Debug, Clone)]
pub struct BeaconState {
    pub genesis_time: u64,
    pub slot: Slot,
    pub fork: Fork,
    pub validators: Vec<Validator>,
    pub balances: Vec<Gwei>,
    pub finalized_checkpoint: Checkpoint,
    // ... 更多字段
}

/// 验证者
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub pubkey: BlsPublicKey,
    pub withdrawal_credentials: Hash256,
    pub effective_balance: Gwei,
    pub slashed: bool,
    pub activation_epoch: Epoch,
    pub exit_epoch: Epoch,
}
```

### 架构设计

```
app/beacon-node/
├── src/
│   ├── consensus/                # 共识层核心
│   │   ├── types/               # 信标链数据类型
│   │   ├── state/               # 状态管理
│   │   ├── fork_choice/         # 分叉选择
│   │   └── transition/          # 状态转换
│   │
│   ├── storage/                  # 存储层
│   │   ├── db/                  # 数据库接口
│   │   ├── state_store/         # 状态存储
│   │   └── block_store/         # 区块存储
│   │
│   ├── network/                  # 网络层
│   │   ├── p2p/                 # libp2p 集成
│   │   ├── discovery/           # discv5 集成
│   │   └── gossipsub/           # 消息传播
│   │
│   ├── api/                      # REST API 层
│   │   ├── beacon/              # /eth/v1/beacon/*
│   │   ├── validator/           # /eth/v1/validator/*
│   │   ├── node/                # /eth/v1/node/*
│   │   ├── config/              # /eth/v1/config/*
│   │   └── events/              # SSE 事件流
│   │
│   ├── validator/                # 验证者逻辑
│   │   ├── duties/              # 职责计算
│   │   ├── attestation/         # 证明生成
│   │   └── block_production/    # 区块生产
│   │
│   └── main.rs                   # 应用入口
```

---

## 🔍 与执行层 API 的集成

### Engine API 集成（关键）

RustEth 的 Beacon Node 需要与执行层客户端通过 Engine API 通信:

```
Beacon Node (本项目)
      ↓
Engine API (JSON-RPC)
      ↓
Execution Client (本项目的 Execution Layer)
```

**必须实现的 Engine API 端点** (已在执行层 API 规划中):
- `engine_newPayloadV3` - 接收新的执行载荷
- `engine_forkchoiceUpdatedV3` - 更新分叉选择
- `engine_getPayloadV3` - 获取执行载荷

### 数据流示例

**区块提议流程**:
```
1. Validator Client → GET /eth/v3/validator/blocks/{slot}
2. Beacon Node → Engine API: engine_forkchoiceUpdatedV3
3. Execution Client → 构建执行载荷
4. Execution Client → Engine API 响应: payload_id
5. Beacon Node → Engine API: engine_getPayloadV3(payload_id)
6. Execution Client → 返回完整载荷
7. Beacon Node → 构建信标区块
8. Validator Client ← 返回待签名区块
```

---

## 📊 性能目标

### 延迟目标

| 操作 | 目标延迟 | 说明 |
|------|---------|------|
| **区块查询** | < 10ms | 热缓存 |
| **状态查询** | < 50ms | 单个验证者 |
| **批量验证者查询** | < 200ms | 1000 个验证者 |
| **职责计算** | < 100ms | 单个 epoch |
| **区块生产** | < 500ms | 包括 Engine API 调用 |

### 资源目标

| 资源 | 目标 | 说明 |
|------|------|------|
| **内存** | < 8GB | 主网全节点 |
| **存储** | < 200GB | 归档模式 |
| **CPU** | 2-4 核 | 验证节点 |
| **网络** | 10 Mbps | P2P + API |

---

## 🧪 测试策略

### 单元测试
- SSZ 编码/解码
- 状态转换函数
- BLS 签名验证
- 职责计算算法

### 集成测试
- API 端点测试
- 数据库持久化
- P2P 网络通信
- Engine API 集成

### 规范测试
- [ethereum/consensus-spec-tests](https://github.com/ethereum/consensus-spec-tests)
- 状态转换测试向量
- 分叉选择测试

### 压力测试
- 大量验证者查询
- 事件订阅负载
- P2P 网络负载

---

## 📚 参考资源

### 官方规范
- [Beacon APIs](https://github.com/ethereum/beacon-APIs)
- [Consensus Specs](https://github.com/ethereum/consensus-specs)
- [Consensus Spec Tests](https://github.com/ethereum/consensus-spec-tests)

### 参考实现
- [Lighthouse (Rust)](https://github.com/sigp/lighthouse) - 最佳 Rust 参考
- [Prysm (Go)](https://github.com/prysmaticlabs/prysm)
- [Teku (Java)](https://github.com/ConsenSys/teku)

### 学习资源
- [Ethereum.org Consensus Layer](https://ethereum.org/developers/docs/consensus-mechanisms/pos)
- [Ben Edgington's Book](https://eth2book.info/)
- [Upgrading Ethereum](https://eth2book.info/capella/part2/)

---

## 🎯 当前状态总结

### ✅ 已完成
- 执行层 JSON-RPC API (18/60 方法)
- P2P 网络层基础（discv5 节点发现）
- Axum HTTP 服务器框架

### ⏳ 进行中
- 执行层剩余 API 方法
- 完善 P2P 网络层

### 📋 待启动
- Beacon Chain 核心逻辑
- 状态存储层
- SSZ 编码集成
- BLS 签名集成
- Beacon REST API

### 🚀 下一步行动
1. **Q2 2025**: 完成执行层 API (eth_sendRawTransaction 等)
2. **Q2 2025**: 开始 Beacon Node 架构设计
3. **Q3 2025**: 实现 Beacon Node 基础功能
4. **Q4 2025**: 实现状态查询 API
5. **Q1 2026**: 实现验证者 API

---

**最后更新**: 2025-11-09
**当前版本**: v0.1.0 (Beacon 模块未开始)
**实现进度**: 0/84 Beacon API (0%)
**总体进度**: 18/144 所有 API (12.5%)
