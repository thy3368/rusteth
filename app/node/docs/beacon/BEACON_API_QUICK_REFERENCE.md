# Beacon Chain API 快速参考

## 🎯 标准来源

| 标准 | 地址 | 说明 |
|------|------|------|
| **官方规范** | https://github.com/ethereum/beacon-APIs | OpenAPI 3.0 规范 |
| **在线文档** | https://ethereum.github.io/beacon-APIs/ | 交互式 API 浏览器 |
| **协议** | REST/HTTP + JSON | RESTful API |

**核心特点**:
- 🌐 **RESTful API**: 标准 HTTP 方法（GET, POST）
- 📄 **JSON**: 唯一数据格式
- 📡 **SSE**: Server-Sent Events 事件流
- 🔐 **本地访问**: 建议仅本地或 SSH 访问

---

## 📚 API 命名空间速览

### 必须实现的核心 API

| 命名空间 | 端点数 | 用途 | 优先级 |
|----------|--------|------|--------|
| `/eth/v1/beacon/*` | ~50 | 信标链核心（区块、状态、池） | ⭐⭐⭐⭐⭐ |
| `/eth/v1/validator/*` | ~20 | 验证者操作（职责、证明） | ⭐⭐⭐⭐⭐ |
| `/eth/v1/node/*` | ~7 | 节点信息和健康检查 | ⭐⭐⭐⭐ |
| `/eth/v1/config/*` | ~5 | 链配置查询 | ⭐⭐⭐⭐ |

### 可选 API

| 命名空间 | 端点数 | 用途 | 优先级 |
|----------|--------|------|--------|
| `/eth/v1/events` | 1 | SSE 事件订阅 | ⭐⭐⭐ |
| `/eth/v1/debug/*` | ~10 | 调试端点 | ⭐⭐ |
| `/eth/v1/rewards/*` | ~5 | 奖励查询 | ⭐⭐⭐ |
| `/eth/v1/light_client/*` | ~5 | 轻客户端 | ⭐⭐ |

---

## 🔥 最重要的 20 个端点

### 必须实现的核心端点 (20个)

#### 基础信息 (4个)
```
1. GET  /eth/v1/beacon/genesis                  - 创世信息 ⭐ 最重要
2. GET  /eth/v1/beacon/headers/head             - 当前链头 ⭐ 最重要
3. GET  /eth/v1/node/version                    - 节点版本
4. GET  /eth/v1/node/health                     - 健康检查 ⭐ 最重要
```

#### 状态查询 (5个)
```
5. GET  /eth/v1/beacon/states/{state_id}/root              - 状态根
6. GET  /eth/v1/beacon/states/{state_id}/validators        - 验证者列表 ⭐ 最重要
7. GET  /eth/v1/beacon/states/{state_id}/validators/{id}   - 单个验证者 ⭐ 最重要
8. GET  /eth/v1/beacon/states/{state_id}/validator_balances - 验证者余额
9. GET  /eth/v1/beacon/states/{state_id}/finality_checkpoints - 最终性检查点
```

#### 区块操作 (4个)
```
10. GET  /eth/v2/beacon/blocks/{block_id}        - 获取区块 ⭐ 最重要
11. GET  /eth/v1/beacon/blocks/{block_id}/root   - 区块根
12. POST /eth/v1/beacon/blocks                   - 发布区块 ⭐ 最重要
13. GET  /eth/v1/beacon/headers                  - 区块头列表
```

#### 验证者职责 (3个)
```
14. GET  /eth/v1/validator/duties/attester/{epoch}   - 证明者职责 ⭐ 最重要
15. GET  /eth/v1/validator/duties/proposer/{epoch}   - 提议者职责 ⭐ 最重要
16. GET  /eth/v1/validator/attestation_data          - 证明数据 ⭐ 最重要
```

#### 交易池 (2个)
```
17. GET  /eth/v1/beacon/pool/attestations        - 获取待处理证明
18. POST /eth/v1/beacon/pool/attestations        - 提交证明
```

#### 配置和网络 (2个)
```
19. GET  /eth/v1/config/spec                     - 链规范参数
20. GET  /eth/v1/node/syncing                    - 同步状态
```

---

## 📋 按功能分类速查

### 1. 基础查询 (5个)
```
GET /eth/v1/beacon/genesis                 - 创世信息
GET /eth/v1/beacon/headers/head            - 当前链头
GET /eth/v1/node/version                   - 节点版本
GET /eth/v1/node/health                    - 健康检查
GET /eth/v1/node/syncing                   - 同步状态
```

### 2. 状态查询 (12个)
```
GET /eth/v1/beacon/states/{state_id}/root
GET /eth/v1/beacon/states/{state_id}/fork
GET /eth/v1/beacon/states/{state_id}/finality_checkpoints
GET /eth/v1/beacon/states/{state_id}/validators
GET /eth/v1/beacon/states/{state_id}/validators/{validator_id}
GET /eth/v1/beacon/states/{state_id}/validator_balances
GET /eth/v1/beacon/states/{state_id}/committees
GET /eth/v1/beacon/states/{state_id}/sync_committees
GET /eth/v1/beacon/states/{state_id}/randao
POST /eth/v1/beacon/states/{state_id}/validators           - 批量查询
POST /eth/v1/beacon/states/{state_id}/validator_balances   - 批量余额
GET /eth/v2/beacon/states/{state_id}                        - 完整状态
```

### 3. 区块操作 (6个)
```
GET  /eth/v1/beacon/headers                      - 区块头列表
GET  /eth/v1/beacon/headers/{block_id}           - 特定区块头
GET  /eth/v2/beacon/blocks/{block_id}            - 获取区块
GET  /eth/v1/beacon/blocks/{block_id}/root       - 区块根
GET  /eth/v1/beacon/blocks/{block_id}/attestations - 区块证明
POST /eth/v1/beacon/blocks                        - 发布区块
```

### 4. 验证者职责 (8个)
```
GET  /eth/v1/validator/duties/attester/{epoch}           - 证明者职责
GET  /eth/v1/validator/duties/proposer/{epoch}           - 提议者职责
POST /eth/v1/validator/duties/sync/{epoch}               - 同步委员会职责
GET  /eth/v3/validator/blocks/{slot}                     - 获取待提议区块
GET  /eth/v1/validator/attestation_data                  - 获取证明数据
GET  /eth/v1/validator/aggregate_attestation             - 获取聚合证明
POST /eth/v1/validator/aggregate_and_proofs              - 发布聚合证明
POST /eth/v1/validator/beacon_committee_subscriptions    - 订阅委员会
```

### 5. 交易池 (10个)
```
GET  /eth/v1/beacon/pool/attestations              - 获取证明
POST /eth/v1/beacon/pool/attestations              - 提交证明
GET  /eth/v1/beacon/pool/attester_slashings        - 获取证明者削减
POST /eth/v1/beacon/pool/attester_slashings        - 提交证明者削减
GET  /eth/v1/beacon/pool/proposer_slashings        - 获取提议者削减
POST /eth/v1/beacon/pool/proposer_slashings        - 提交提议者削减
GET  /eth/v1/beacon/pool/voluntary_exits           - 获取自愿退出
POST /eth/v1/beacon/pool/voluntary_exits           - 提交自愿退出
GET  /eth/v1/beacon/pool/bls_to_execution_changes  - BLS 地址变更
POST /eth/v1/beacon/pool/bls_to_execution_changes  - 提交 BLS 变更
```

### 6. 节点管理 (7个)
```
GET /eth/v1/node/identity       - 节点身份 (peer_id, enr)
GET /eth/v1/node/peers          - 对等节点列表
GET /eth/v1/node/peers/{id}     - 特定对等节点
GET /eth/v1/node/peer_count     - 对等节点数量
GET /eth/v1/node/version        - 客户端版本
GET /eth/v1/node/syncing        - 同步状态
GET /eth/v1/node/health         - 健康检查
```

### 7. 配置查询 (3个)
```
GET /eth/v1/config/fork_schedule        - 分叉时间表
GET /eth/v1/config/spec                 - 链规范参数
GET /eth/v1/config/deposit_contract     - 存款合约地址
```

### 8. 事件订阅 (1个 SSE)
```
GET /eth/v1/events?topics=<topic1>,<topic2>,...
```

**支持的事件**:
- `head` - 新链头
- `block` - 新区块
- `attestation` - 新证明
- `voluntary_exit` - 自愿退出
- `finalized_checkpoint` - 最终性检查点
- `chain_reorg` - 链重组

### 9. 奖励查询 (3个)
```
POST /eth/v1/beacon/rewards/attestations               - 证明奖励
GET  /eth/v1/beacon/rewards/blocks/{block_id}          - 区块奖励
POST /eth/v1/beacon/rewards/sync_committee/{block_id}  - 同步委员会奖励
```

### 10. 调试端点 (3个) - 谨慎使用
```
GET /eth/v2/debug/beacon/states/{state_id}   - 完整状态转储 ⚠️
GET /eth/v2/debug/beacon/heads                - 所有链头
GET /eth/v1/debug/fork_choice                 - 分叉选择状态
```

---

## 💡 数据类型规范

### 标识符格式

| 类型 | 格式 | 示例 |
|------|------|------|
| **state_id** | `head`, `finalized`, `justified`, `genesis`, `<slot>`, `0x<root>` | `"head"`, `"1234567"` |
| **block_id** | `head`, `finalized`, `genesis`, `<slot>`, `0x<root>` | `"finalized"`, `"0x1234..."` |
| **validator_id** | `<pubkey>`, `<index>` | `"12345"`, `"0x1234..."` |
| **epoch** | 十进制数字 | `"12345"` |
| **slot** | 十进制数字 | `"1234567"` |

### 常用字段格式

| 字段 | 格式 | 说明 |
|------|------|------|
| **Root** | `0x` + 64字符 | 32字节哈希 |
| **公钥** | `0x` + 96字符 | 48字节 BLS 公钥 |
| **签名** | `0x` + 192字符 | 96字节 BLS 签名 |
| **Gwei** | 十进制字符串 | `"32000000000"` = 32 ETH |
| **时间戳** | Unix 秒 | `"1606824023"` |

### 验证者状态枚举

```
pending_initialized       - 已存款，等待激活
pending_queued           - 激活队列中
active_ongoing           - 正在验证 ✅
active_exiting           - 正在退出
active_slashed           - 被削减 ⚠️
exited_unslashed         - 已退出（正常）
exited_slashed           - 已退出（被削减）
withdrawal_possible      - 可提款
withdrawal_done          - 已提款
```

---

## 🛠️ HTTP 方法和状态码

### HTTP 方法

| 方法 | 用途 | 幂等性 |
|------|------|--------|
| `GET` | 查询数据 | ✅ 幂等 |
| `POST` | 提交数据、批量查询 | ❌ 非幂等 |

### 状态码

| 状态码 | 含义 | 场景 |
|--------|------|------|
| `200` | 成功 | 正常请求 |
| `202` | 已接受 | 异步处理（提交区块/证明） |
| `204` | 无内容 | 成功但无返回 |
| `206` | 部分内容 | 节点正在同步 |
| `400` | 请求错误 | 参数无效 |
| `404` | 未找到 | 资源不存在 |
| `500` | 服务器错误 | 内部错误 |
| `503` | 服务不可用 | 节点不健康 |

---

## 🚀 快速使用示例

### 1. 获取创世信息
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

### 2. 获取当前链头
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
        "proposer_index": "12345"
      }
    }
  }
}
```

### 3. 查询验证者信息
```bash
# 单个验证者
curl http://localhost:5052/eth/v1/beacon/states/head/validators/12345

# 批量查询（POST）
curl -X POST http://localhost:5052/eth/v1/beacon/states/head/validators \
  -H "Content-Type: application/json" \
  -d '{"ids":["12345","12346","12347"],"statuses":["active_ongoing"]}'
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
      "effective_balance": "32000000000",
      "slashed": false
    }
  }
}
```

### 4. 获取验证者职责
```bash
# 证明者职责
curl -X POST http://localhost:5052/eth/v1/validator/duties/attester/12345 \
  -H "Content-Type: application/json" \
  -d '["0","1","2"]'

# 提议者职责
curl http://localhost:5052/eth/v1/validator/duties/proposer/12345
```

**响应**:
```json
{
  "data": [
    {
      "pubkey": "0x1234...",
      "validator_index": "0",
      "committee_index": "1",
      "committee_length": "128",
      "committees_at_slot": "64",
      "validator_committee_index": "15",
      "slot": "123456"
    }
  ]
}
```

### 5. 订阅事件 (SSE)
```bash
# 订阅链头事件
curl -N -H "Accept: text/event-stream" \
  http://localhost:5052/eth/v1/events?topics=head

# 订阅多个事件
curl -N -H "Accept: text/event-stream" \
  http://localhost:5052/eth/v1/events?topics=head,block,attestation
```

**响应流**:
```
event: head
data: {"slot":"1234567","block":"0x1234...","state":"0xabcd..."}

event: block
data: {"slot":"1234567","block":"0x1234...","execution_optimistic":false}
```

### 6. 健康检查
```bash
curl -I http://localhost:5052/eth/v1/node/health
```

**响应**:
```
HTTP/1.1 200 OK        # 节点正常
HTTP/1.1 206 Partial   # 节点同步中
HTTP/1.1 503 Error     # 节点不健康
```

### 7. 获取链配置
```bash
curl http://localhost:5052/eth/v1/config/spec
```

**响应**:
```json
{
  "data": {
    "SLOTS_PER_EPOCH": "32",
    "SECONDS_PER_SLOT": "12",
    "MIN_GENESIS_TIME": "1606824000",
    "DEPOSIT_CONTRACT_ADDRESS": "0x00000000219ab540356cbb839cbe05303d7705fa"
  }
}
```

---

## 📊 客户端对比

| 客户端 | 语言 | Beacon API | 默认端口 | 特色 |
|--------|------|-----------|---------|------|
| **Lighthouse** | Rust | ✅ 完整 | 5052 | 高性能 |
| **Prysm** | Go | ✅ 完整 | 3500 | MEV 集成 |
| **Teku** | Java | ✅ 完整 | 5051 | 企业级 |
| **Nimbus** | Nim | ✅ 完整 | 5052 | 低资源 |
| **Lodestar** | TypeScript | ✅ 完整 | 9596 | 易开发 |

---

## 🔐 安全最佳实践

### ✅ 应该做的

1. **本地访问**: 绑定 `127.0.0.1`
2. **SSH 隧道**: 远程访问使用 SSH
3. **防火墙**: 限制端口访问
4. **Rate Limiting**: 实施请求限制
5. **监控**: 记录和监控 API 访问

### ❌ 不应该做的

1. ❌ **公开暴露**: 不要绑定 `0.0.0.0` 并开放防火墙
2. ❌ **启用 Debug**: 生产环境不启用 `/debug/*`
3. ❌ **无限制**: 不限制请求速率
4. ❌ **无认证**: 公网访问无认证

### ⚠️ 高风险端点

```
⚠️ GET /eth/v2/debug/beacon/states/{state_id}  - 数据量巨大，易 DoS
⚠️ GET /eth/v1/debug/fork_choice                - 暴露内部状态
⚠️ POST /eth/v1/beacon/blocks                    - 写入操作，需保护
```

---

## 🎯 与执行层 API 的对比

| 特性 | Beacon API (共识层) | JSON-RPC (执行层) |
|------|-------------------|------------------|
| **协议** | REST/HTTP | JSON-RPC 2.0 |
| **格式** | JSON | JSON |
| **端口** | 5052 (Lighthouse) | 8545 |
| **用途** | 信标链状态、验证者操作 | 交易、合约、账户 |
| **事件** | SSE 流 | WebSocket 订阅 |
| **标准** | OpenAPI 3.0 | EIP-1474 |

### 典型使用场景

**Beacon API**:
- ✅ 查询验证者状态
- ✅ 获取证明职责
- ✅ 监听新区块
- ✅ 查询最终性

**JSON-RPC API**:
- ✅ 发送交易
- ✅ 调用智能合约
- ✅ 查询账户余额
- ✅ 获取交易收据

**Engine API** (内部):
- ✅ 共识层 ↔ 执行层通信
- ✅ 区块提议和验证

---

## 💡 开发技巧

### 1. 常用查询组合

**启动时初始化**:
```bash
# 获取链配置
curl http://localhost:5052/eth/v1/config/spec

# 获取创世信息
curl http://localhost:5052/eth/v1/beacon/genesis

# 检查同步状态
curl http://localhost:5052/eth/v1/node/syncing
```

**验证者监控**:
```bash
# 查询验证者状态
curl http://localhost:5052/eth/v1/beacon/states/head/validators/{id}

# 获取职责
curl -X POST http://localhost:5052/eth/v1/validator/duties/attester/{epoch} \
  -d '["validator_index"]'

# 订阅事件
curl -N http://localhost:5052/eth/v1/events?topics=head,attestation
```

### 2. 批量操作优化

使用 POST 端点批量查询，避免多次请求:

```bash
# ❌ 低效：多次单独查询
for id in 1 2 3 4 5; do
  curl http://localhost:5052/eth/v1/beacon/states/head/validators/$id
done

# ✅ 高效：批量查询
curl -X POST http://localhost:5052/eth/v1/beacon/states/head/validators \
  -H "Content-Type: application/json" \
  -d '{"ids":["1","2","3","4","5"]}'
```

### 3. 错误处理

```bash
# 检查 HTTP 状态码
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:5052/eth/v1/node/health)

if [ "$HTTP_CODE" -eq 200 ]; then
  echo "节点正常"
elif [ "$HTTP_CODE" -eq 206 ]; then
  echo "节点同步中"
else
  echo "节点异常"
fi
```

---

## 📚 学习路径

### 入门 (1天)
1. 理解信标链基础概念
2. 安装并运行信标节点
3. 测试基础 API (genesis, health, version)
4. 查询验证者信息

### 进阶 (1周)
1. 理解验证者职责
2. 使用职责 API
3. 监听事件流
4. 实现基础监控

### 高级 (1月)
1. 实现完整验证者客户端
2. 集成奖励查询
3. 实现轻客户端协议
4. 性能优化和缓存

---

## 🔗 参考资源

### 官方文档
- [Beacon APIs 仓库](https://github.com/ethereum/beacon-APIs)
- [在线 API 浏览器](https://ethereum.github.io/beacon-APIs/)
- [共识层规范](https://github.com/ethereum/consensus-specs)

### 工具
- [Swagger UI](https://swagger.io/tools/swagger-ui/) - API 测试
- [Postman](https://www.postman.com/) - API 集合
- [curl](https://curl.se/) - 命令行测试

### 客户端文档
- [Lighthouse Book](https://lighthouse-book.sigmaprime.io/)
- [Prysm Docs](https://docs.prylabs.network/)
- [Teku Docs](https://docs.teku.consensys.net/)

---

**快速查阅版本**: v1.0
**更新日期**: 2025-11-09
**适用于**: Ethereum Consensus Layer (Beacon Chain)
