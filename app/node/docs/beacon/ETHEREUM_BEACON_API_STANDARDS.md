# 以太坊 Beacon Chain API 标准规范

## 🎯 标准来源

| 标准 | 地址 | 说明 |
|------|------|------|
| **官方规范** | https://github.com/ethereum/beacon-APIs | OpenAPI 3.0 格式的完整规范 |
| **在线文档** | https://ethereum.github.io/beacon-APIs/ | 交互式 API 浏览器 |
| **共识层规范** | https://github.com/ethereum/consensus-specs | 信标链共识规范 |

**核心标准**:
- **OpenAPI 3.0**: RESTful API 规范格式
- **HTTP/REST**: 基于标准 HTTP 协议
- **JSON**: 唯一支持的数据格式
- **Server-Sent Events (SSE)**: 事件流订阅

---

## 📚 架构概述

### 核心组件

```
┌─────────────────────────────────────────────────────────┐
│                    共识层 (Consensus Layer)              │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────────┐         ┌───────────────────┐    │
│  │  Beacon Node (BN) │ ◄────► │ Validator Client   │    │
│  │  (信标节点)        │  API   │  (验证者客户端)     │    │
│  └──────────────────┘         └───────────────────┘    │
│         ▲                                                │
│         │ Beacon API (本文档所述)                       │
│         ▼                                                │
│  ┌──────────────────┐                                   │
│  │  外部应用/工具    │                                   │
│  │  (钱包、区块浏览器等)│                                │
│  └──────────────────┘                                   │
└─────────────────────────────────────────────────────────┘
```

### 角色说明

| 组件 | 职责 | 通信方式 |
|------|------|----------|
| **Beacon Node** | 维护信标链状态，与其他节点通信，处理共识 | P2P + REST API |
| **Validator Client** | 使用私钥执行验证者职责（提议区块、签署证明） | REST API 客户端 |
| **外部应用** | 查询链状态、监听事件、获取验证者信息 | REST API 客户端 |

**重要**:
- Beacon Node 与 Validator Client 应该**私密通信**（同一机器或 SSH 隧道）
- 某些端点暴露在公网会有 **DoS 风险**或**信息泄露**

---

## 🔥 API 命名空间分类

### 标准 API (所有客户端必须实现)

| 命名空间 | 端点数量 | 用途 | 实现优先级 |
|----------|---------|------|-----------|
| `/eth/v1/beacon/*` | ~50 | 信标链核心功能（区块、状态、池） | ⭐⭐⭐⭐⭐ 最高 |
| `/eth/v1/validator/*` | ~20 | 验证者操作（职责、提议、证明） | ⭐⭐⭐⭐⭐ 最高 |
| `/eth/v1/node/*` | ~7 | 节点信息和健康检查 | ⭐⭐⭐⭐ 高 |
| `/eth/v1/config/*` | ~5 | 链配置和规范参数 | ⭐⭐⭐⭐ 高 |

### 可选 API

| 命名空间 | 端点数量 | 用途 | 实现优先级 |
|----------|---------|------|-----------|
| `/eth/v1/debug/*` | ~10 | 调试和链状态转储 | ⭐⭐ 低 |
| `/eth/v1/events` | 1 (SSE) | 事件订阅（区块、证明、重组等） | ⭐⭐⭐ 中 |
| `/eth/v1/light_client/*` | ~5 | 轻客户端支持 | ⭐⭐ 低 |
| `/eth/v1/rewards/*` | ~5 | 奖励和惩罚查询 | ⭐⭐⭐ 中 |

---

## 📋 API 端点详细分类

### 1. Beacon API (`/eth/v1/beacon/*`) - 核心功能

#### 1.1 Genesis (创世)
```
GET /eth/v1/beacon/genesis
```
- **用途**: 获取链创世信息
- **返回**: genesis_time, genesis_validators_root, genesis_fork_version

#### 1.2 States (状态查询)

**基础状态查询**:
```
GET /eth/v1/beacon/states/{state_id}/root              - 状态根哈希
GET /eth/v1/beacon/states/{state_id}/fork              - 分叉信息
GET /eth/v1/beacon/states/{state_id}/finality_checkpoints - 最终性检查点
GET /eth/v2/beacon/states/{state_id}                   - 完整状态 (慎用)
```

**验证者查询**:
```
GET /eth/v1/beacon/states/{state_id}/validators                    - 所有验证者
GET /eth/v1/beacon/states/{state_id}/validators/{validator_id}    - 单个验证者
GET /eth/v1/beacon/states/{state_id}/validator_balances           - 验证者余额
POST /eth/v1/beacon/states/{state_id}/validators                  - 批量查询验证者
POST /eth/v1/beacon/states/{state_id}/validator_balances          - 批量查询余额
```

**委员会和同步**:
```
GET /eth/v1/beacon/states/{state_id}/committees              - 委员会信息
GET /eth/v1/beacon/states/{state_id}/sync_committees         - 同步委员会
```

**其他状态查询**:
```
GET /eth/v1/beacon/states/{state_id}/randao                       - RANDAO 随机数
GET /eth/v1/beacon/states/{state_id}/pending_consolidations      - 待处理合并
GET /eth/v1/beacon/states/{state_id}/pending_deposits            - 待处理存款
GET /eth/v1/beacon/states/{state_id}/pending_partial_withdrawals - 待处理部分提款
```

**state_id 支持的格式**:
- `head` - 当前头部状态
- `genesis` - 创世状态
- `finalized` - 最终确定状态
- `justified` - 最新合理状态
- `<slot>` - 特定 slot
- `0x<state_root>` - 特定状态根

#### 1.3 Headers (区块头)
```
GET /eth/v1/beacon/headers                 - 获取区块头列表
GET /eth/v1/beacon/headers/{block_id}      - 获取特定区块头
```

#### 1.4 Blocks (区块)
```
GET /eth/v2/beacon/blocks/{block_id}                       - 获取区块
GET /eth/v1/beacon/blocks/{block_id}/root                  - 区块根哈希
GET /eth/v1/beacon/blocks/{block_id}/attestations          - 区块中的证明
POST /eth/v1/beacon/blocks                                  - 发布区块
POST /eth/v2/beacon/blinded_blocks                          - 发布盲区块 (MEV)
```

**block_id 支持的格式**:
- `head` - 当前头部区块
- `genesis` - 创世区块
- `finalized` - 最终确定区块
- `<slot>` - 特定 slot
- `0x<block_root>` - 特定区块根

#### 1.5 Pool (交易池)
```
GET /eth/v1/beacon/pool/attestations               - 获取待处理证明
POST /eth/v1/beacon/pool/attestations              - 提交证明

GET /eth/v1/beacon/pool/attester_slashings         - 获取证明者削减
POST /eth/v1/beacon/pool/attester_slashings        - 提交证明者削减

GET /eth/v1/beacon/pool/proposer_slashings         - 获取提议者削减
POST /eth/v1/beacon/pool/proposer_slashings        - 提交提议者削减

GET /eth/v1/beacon/pool/voluntary_exits            - 获取自愿退出
POST /eth/v1/beacon/pool/voluntary_exits           - 提交自愿退出

GET /eth/v1/beacon/pool/bls_to_execution_changes   - 获取 BLS 到执行层地址变更
POST /eth/v1/beacon/pool/bls_to_execution_changes  - 提交 BLS 到执行层地址变更
```

#### 1.6 Rewards (奖励)
```
POST /eth/v1/beacon/rewards/attestations           - 查询证明奖励
GET /eth/v1/beacon/rewards/blocks/{block_id}       - 查询区块奖励
POST /eth/v1/beacon/rewards/sync_committee/{block_id} - 查询同步委员会奖励
```

#### 1.7 Light Client (轻客户端)
```
GET /eth/v1/beacon/light_client/bootstrap/{block_root}        - 轻客户端引导
GET /eth/v1/beacon/light_client/updates                       - 轻客户端更新
GET /eth/v1/beacon/light_client/finality_update               - 最终性更新
GET /eth/v1/beacon/light_client/optimistic_update             - 乐观更新
```

---

### 2. Validator API (`/eth/v1/validator/*`) - 验证者操作

#### 2.1 职责查询
```
GET /eth/v1/validator/duties/attester/{epoch}          - 获取证明者职责
GET /eth/v1/validator/duties/proposer/{epoch}          - 获取提议者职责
POST /eth/v1/validator/duties/sync/{epoch}             - 获取同步委员会职责
```

#### 2.2 区块生产
```
GET /eth/v3/validator/blocks/{slot}                    - 获取待提议区块
GET /eth/v1/validator/blinded_blocks/{slot}            - 获取盲区块 (MEV)
POST /eth/v1/validator/beacon_committee_subscriptions  - 订阅委员会
```

#### 2.3 证明操作
```
GET /eth/v1/validator/attestation_data                 - 获取证明数据
GET /eth/v1/validator/aggregate_attestation            - 获取聚合证明
POST /eth/v1/validator/aggregate_and_proofs            - 发布聚合证明
```

#### 2.4 同步委员会
```
POST /eth/v1/validator/sync_committee_subscriptions    - 订阅同步委员会
GET /eth/v1/validator/sync_committee_contribution      - 获取同步委员会贡献
POST /eth/v1/validator/contribution_and_proofs         - 发布贡献和证明
```

#### 2.5 验证者管理
```
POST /eth/v1/validator/prepare_beacon_proposer         - 准备成为提议者
POST /eth/v1/validator/register_validator              - 注册验证者 (MEV)
GET /eth/v1/validator/liveness/{epoch}                 - 查询验证者活跃性
```

---

### 3. Node API (`/eth/v1/node/*`) - 节点信息

```
GET /eth/v1/node/identity                - 节点身份信息 (peer_id, enr, p2p 地址)
GET /eth/v1/node/peers                   - 获取对等节点列表
GET /eth/v1/node/peers/{peer_id}         - 获取特定对等节点信息
GET /eth/v1/node/peer_count              - 获取对等节点数量
GET /eth/v1/node/version                 - 获取客户端版本
GET /eth/v1/node/syncing                 - 获取同步状态
GET /eth/v1/node/health                  - 健康检查端点
```

**用途**:
- 健康检查: `/health` 返回 200 (正常), 206 (同步中), 503 (错误)
- 监控: 对等节点数、同步状态
- 调试: 节点版本、网络身份

---

### 4. Config API (`/eth/v1/config/*`) - 配置查询

```
GET /eth/v1/config/fork_schedule          - 分叉时间表
GET /eth/v1/config/spec                   - 链规范参数
GET /eth/v1/config/deposit_contract       - 存款合约地址
```

**返回数据示例**:
- `SLOTS_PER_EPOCH`: 每个 epoch 的 slot 数 (32)
- `SECONDS_PER_SLOT`: 每个 slot 的秒数 (12)
- `MIN_GENESIS_TIME`: 创世最小时间
- `DEPOSIT_CONTRACT_ADDRESS`: 存款合约地址

---

### 5. Debug API (`/eth/v1/debug/*`) - 调试功能

```
GET /eth/v2/debug/beacon/states/{state_id}      - 获取完整状态 (巨大！)
GET /eth/v2/debug/beacon/heads                  - 获取所有链头
GET /eth/v1/debug/fork_choice                   - 获取分叉选择状态
```

**警告**:
- ⚠️ **不要在生产环境公开暴露**
- ⚠️ **状态转储可能达到数 GB**
- ⚠️ **容易被 DoS 攻击**

---

### 6. Events API (`/eth/v1/events`) - 事件订阅

```
GET /eth/v1/events?topics=<topic1>,<topic2>,...
```

**协议**: Server-Sent Events (SSE)

**支持的事件类型**:
- `head` - 新的链头
- `block` - 新区块
- `attestation` - 新证明
- `voluntary_exit` - 自愿退出
- `finalized_checkpoint` - 最终性检查点
- `chain_reorg` - 链重组
- `contribution_and_proof` - 同步委员会贡献
- `bls_to_execution_change` - BLS 地址变更
- `payload_attributes` - 执行层载荷属性

**示例**:
```bash
curl -N -H "Accept: text/event-stream" \
  http://localhost:5052/eth/v1/events?topics=head,block
```

---

## 💡 数据类型和编码规范

### 数字和字符串格式

| 类型 | 格式 | 示例 |
|------|------|------|
| **Slot** | 十进制字符串 | `"1234567"` |
| **Epoch** | 十进制字符串 | `"12345"` |
| **Root** | 十六进制 (0x + 64字符) | `"0x1234...abcd"` |
| **公钥** | 十六进制 (0x + 96字符) | `"0x1234...abcd"` (48字节) |
| **签名** | 十六进制 (0x + 192字符) | `"0x1234...abcd"` (96字节) |
| **Gwei** | 十进制字符串 | `"32000000000"` (32 ETH) |

### 时间和版本

| 字段 | 格式 | 说明 |
|------|------|------|
| `genesis_time` | Unix 时间戳 (秒) | `"1606824023"` |
| `version` | 4字节十六进制 | `"0x00000000"` (Phase 0) |

### 验证者状态

| 状态 | 说明 |
|------|------|
| `pending_initialized` | 已存款，等待激活 |
| `pending_queued` | 在激活队列中 |
| `active_ongoing` | 正在验证 |
| `active_exiting` | 正在退出 |
| `active_slashed` | 被削减 |
| `exited_unslashed` | 已退出（未被削减） |
| `exited_slashed` | 已退出（被削减） |
| `withdrawal_possible` | 可以提款 |
| `withdrawal_done` | 已提款 |

---

## 🛠️ HTTP 状态码和错误处理

### 标准状态码

| 状态码 | 含义 | 使用场景 |
|--------|------|----------|
| `200` | 成功 | 正常请求 |
| `202` | 已接受 | 异步处理（如提交区块） |
| `204` | 无内容 | 成功但无返回数据 |
| `400` | 请求错误 | 参数无效 |
| `404` | 未找到 | 资源不存在 |
| `500` | 服务器错误 | 内部错误 |
| `503` | 服务不可用 | 节点未同步或不健康 |
| `206` | 部分内容 | 节点正在同步 |

### 错误响应格式

```json
{
  "code": 404,
  "message": "State not found",
  "stacktraces": []
}
```

---

## 🚀 快速开始示例

### 1. 获取链创世信息
```bash
curl http://localhost:5052/eth/v1/beacon/genesis
```

**响应**:
```json
{
  "data": {
    "genesis_time": "1606824023",
    "genesis_validators_root": "0x4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95",
    "genesis_fork_version": "0x00000000"
  }
}
```

### 2. 获取当前区块号
```bash
curl http://localhost:5052/eth/v1/beacon/headers/head
```

**响应**:
```json
{
  "data": {
    "root": "0x1234...",
    "canonical": true,
    "header": {
      "message": {
        "slot": "1234567",
        "proposer_index": "12345",
        "parent_root": "0xabcd...",
        "state_root": "0xef01...",
        "body_root": "0x2345..."
      },
      "signature": "0x5678..."
    }
  }
}
```

### 3. 查询验证者信息
```bash
curl http://localhost:5052/eth/v1/beacon/states/head/validators/12345
```

**响应**:
```json
{
  "data": {
    "index": "12345",
    "balance": "32000000000",
    "status": "active_ongoing",
    "validator": {
      "pubkey": "0x1234...",
      "withdrawal_credentials": "0x00abcd...",
      "effective_balance": "32000000000",
      "slashed": false,
      "activation_eligibility_epoch": "100",
      "activation_epoch": "101",
      "exit_epoch": "18446744073709551615",
      "withdrawable_epoch": "18446744073709551615"
    }
  }
}
```

### 4. 订阅链头事件 (SSE)
```bash
curl -N -H "Accept: text/event-stream" \
  http://localhost:5052/eth/v1/events?topics=head
```

**响应流**:
```
event: head
data: {"slot":"1234567","block":"0x1234...","state":"0xabcd...","epoch_transition":false}

event: head
data: {"slot":"1234568","block":"0x5678...","state":"0xef01...","epoch_transition":false}
```

### 5. 获取验证者证明职责
```bash
curl -X POST http://localhost:5052/eth/v1/validator/duties/attester/12345 \
  -H "Content-Type: application/json" \
  -d '["0"]'
```

---

## 📊 客户端实现对比

| 客户端 | 语言 | Beacon API 实现 | 特色功能 |
|--------|------|----------------|----------|
| **Lighthouse** | Rust | ✅ 完整 | 高性能、Slasher |
| **Prysm** | Go | ✅ 完整 | MEV-Boost 集成 |
| **Teku** | Java | ✅ 完整 | 企业级、Slashing 保护 |
| **Nimbus** | Nim | ✅ 完整 | 低资源占用 |
| **Lodestar** | TypeScript | ✅ 完整 | 易于开发和测试 |

---

## 🔐 安全建议

### 1. 访问控制
- ✅ **仅本地访问**: 绑定 `127.0.0.1` 而非 `0.0.0.0`
- ✅ **防火墙**: 使用防火墙限制访问
- ✅ **SSH 隧道**: 远程访问使用 SSH 端口转发
- ❌ **不要公开**: 避免将 API 直接暴露到互联网

### 2. 端点限制
- ⚠️ **禁用 Debug API**: 生产环境不启用 `/debug/*`
- ⚠️ **限制状态查询**: 避免频繁查询完整状态
- ⚠️ **Rate Limiting**: 实施请求速率限制

### 3. 验证者安全
- 🔒 **私钥隔离**: 验证者客户端与信标节点分离
- 🔒 **Slashing 保护**: 使用 Slashing 保护数据库
- 🔒 **备份**: 定期备份验证者密钥

---

## 🎯 与执行层 API 的关系

### API 分工

| 层级 | API 类型 | 协议 | 端口 | 用途 |
|------|---------|------|------|------|
| **共识层** | Beacon API | REST/HTTP | 5052 | 信标链状态、验证者操作 |
| **执行层** | JSON-RPC | JSON-RPC 2.0 | 8545 | 交易、智能合约、账户状态 |
| **内部通信** | Engine API | JSON-RPC 2.0 | 8551 | 共识层 ↔ 执行层 |

### 典型工作流

```
用户 → Beacon API → Beacon Node (共识层)
                          ↓
                    Engine API (内部)
                          ↓
                    Execution Client (执行层)
                          ↓
                    以太坊网络
```

**示例场景**:
1. **质押存款**: 用户通过执行层 API 调用存款合约
2. **验证者激活**: 信标节点通过 Beacon API 查询验证者状态
3. **区块提议**: 验证者客户端通过 Beacon API 获取待提议区块
4. **交易查询**: 用户通过执行层 JSON-RPC API 查询交易

---

## 📚 学习资源

### 官方文档
- [Beacon APIs GitHub](https://github.com/ethereum/beacon-APIs)
- [共识层规范](https://github.com/ethereum/consensus-specs)
- [以太坊官网](https://ethereum.org/developers)

### 工具
- [在线 API 浏览器](https://ethereum.github.io/beacon-APIs/)
- [Swagger UI](https://swagger.io/tools/swagger-ui/)
- [Postman Collection](https://www.postman.com/)

### 参考实现
- [Lighthouse (Rust)](https://github.com/sigp/lighthouse)
- [Prysm (Go)](https://github.com/prysmaticlabs/prysm)
- [Teku (Java)](https://github.com/ConsenSys/teku)
- [Nimbus (Nim)](https://github.com/status-im/nimbus-eth2)
- [Lodestar (TypeScript)](https://github.com/ChainSafe/lodestar)

---

## 💡 开发建议

### 1. 实现优先级

**阶段 1: 基础查询** (只读)
```
✅ /eth/v1/beacon/genesis
✅ /eth/v1/beacon/headers/head
✅ /eth/v1/beacon/states/{state_id}/validators
✅ /eth/v1/node/version
✅ /eth/v1/node/health
```

**阶段 2: 验证者支持** (读写)
```
✅ /eth/v1/validator/duties/attester/{epoch}
✅ /eth/v1/validator/attestation_data
✅ /eth/v1/beacon/pool/attestations
```

**阶段 3: 高级功能**
```
✅ /eth/v1/events (SSE)
✅ /eth/v1/validator/aggregate_and_proofs
✅ /eth/v1/beacon/light_client/*
```

### 2. 测试建议
- 使用测试网（Goerli、Sepolia）
- 运行本地 devnet (Kurtosis, Ethereum-package)
- 自动化 API 兼容性测试

### 3. 性能优化
- 缓存常用查询（如配置、创世信息）
- 使用流式响应处理大数据集
- 实施请求批处理

---

## 📝 版本历史

**当前版本**: v4.0.0 (2025-10-14)

**主要变更**:
- v4.0.0: 添加 Electra 分叉支持
- v3.0.0: 添加 Deneb 分叉支持
- v2.5.0: 添加轻客户端端点
- v2.0.0: 添加 Bellatrix (合并) 支持
- v1.0.0: 初始稳定版本

---

## 🔗 相关标准

- [EIP-3675: The Merge (PoS 升级)](https://eips.ethereum.org/EIPS/eip-3675)
- [EIP-4881: 存款快照 Merkle 树](https://eips.ethereum.org/EIPS/eip-4881)
- [EIP-4844: Proto-Danksharding](https://eips.ethereum.org/EIPS/eip-4844)

---

**文档版本**: v1.0
**更新日期**: 2025-11-09
**适用于**: Ethereum Consensus Layer Clients (Beacon Nodes)
