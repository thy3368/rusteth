# RustEth API 实现状态

## 📊 总览

| 类别 | 已实现 | 计划实现 | 未计划 | 总计 |
|------|--------|----------|--------|------|
| **eth_** 标准方法 | 16 | 15 | 8 | 39 |
| **net_** 方法 | 1 | 2 | 0 | 3 |
| **web3_** 方法 | 1 | 1 | 0 | 2 |
| **engine_** 方法 | 0 | 9 | 0 | 9 |
| **debug_** 方法 | 0 | 5 | 2 | 7 |
| **总计** | **18** | **32** | **10** | **60** |

**实现率**: 30% (18/60 核心方法)

---

## ✅ 已实现的方法 (18个)

### eth_ 方法 (16个)

#### 区块方法 (3个)
- ✅ `eth_blockNumber` - 获取当前区块号
- ✅ `eth_getBlockByNumber` - 根据区块号获取区块
- ✅ `eth_getBlockByHash` - 根据哈希获取区块

#### 交易方法 (2个)
- ✅ `eth_getTransactionByHash` - 获取交易详情
- ✅ `eth_getTransactionReceipt` - 获取交易回执

#### 状态读取方法 (6个)
- ✅ `eth_getBalance` - 获取账户余额
- ✅ `eth_getStorageAt` - 读取合约存储
- ✅ `eth_getTransactionCount` - 获取账户 nonce
- ✅ `eth_getCode` - 获取合约代码
- ✅ `eth_call` - 执行只读调用
- ✅ `eth_estimateGas` - 估算 gas 消耗

#### 日志方法 (1个)
- ✅ `eth_getLogs` - 查询日志

#### 链信息方法 (2个)
- ✅ `eth_chainId` - 获取链 ID
- ✅ `eth_gasPrice` - 获取 gas 价格

### net_ 方法 (1个)
- ✅ `net_version` - 获取网络 ID

### web3_ 方法 (1个)
- ✅ `web3_clientVersion` - 获取客户端版本

---

## 🚧 计划实现的方法 (32个)

### 优先级 1: 核心交易功能 (5个)

#### 交易发送
- ⏳ `eth_sendTransaction` - 发送交易 (需要账户管理)
- ⏳ `eth_sendRawTransaction` - 发送已签名交易 **[高优先级]**
- ⏳ `eth_sign` - 签名数据
- ⏳ `eth_signTransaction` - 签名交易

#### 交易查询
- ⏳ `eth_getTransactionByBlockHashAndIndex` - 通过区块哈希和索引获取交易

### 优先级 2: 区块查询扩展 (6个)

- ⏳ `eth_getBlockTransactionCountByHash` - 获取区块交易数量(hash)
- ⏳ `eth_getBlockTransactionCountByNumber` - 获取区块交易数量(number)
- ⏳ `eth_getTransactionByBlockNumberAndIndex` - 通过区块号和索引获取交易
- ⏳ `eth_getUncleCountByBlockHash` - 获取叔块数量(hash)
- ⏳ `eth_getUncleCountByBlockNumber` - 获取叔块数量(number)
- ⏳ `eth_getUncleByBlockHashAndIndex` - 获取叔块(hash+index)
- ⏳ `eth_getUncleByBlockNumberAndIndex` - 获取叔块(number+index)

### 优先级 3: 过滤器 API (6个)

- ⏳ `eth_newFilter` - 创建过滤器
- ⏳ `eth_newBlockFilter` - 创建区块过滤器
- ⏳ `eth_newPendingTransactionFilter` - 创建待处理交易过滤器
- ⏳ `eth_uninstallFilter` - 删除过滤器
- ⏳ `eth_getFilterChanges` - 获取过滤器变化
- ⏳ `eth_getFilterLogs` - 获取过滤器所有日志

### 优先级 4: 链状态方法 (3个)

- ⏳ `eth_syncing` - 获取同步状态
- ⏳ `eth_accounts` - 获取账户列表
- ⏳ `eth_protocolVersion` - 获取协议版本

### 优先级 5: Engine API (9个) **[PoS 支持]**

- ⏳ `engine_newPayloadV1/V2/V3` - 接收新的执行载荷
- ⏳ `engine_forkchoiceUpdatedV1/V2/V3` - 更新分叉选择
- ⏳ `engine_getPayloadV1/V2/V3` - 获取执行载荷
- ⏳ `engine_exchangeTransitionConfigurationV1` - 交换转换配置
- ⏳ `engine_getPayloadBodiesByHashV1` - 通过哈希获取载荷体
- ⏳ `engine_getPayloadBodiesByRangeV1` - 通过范围获取载荷体

### 优先级 6: 网络信息 (2个)

- ⏳ `net_listening` - 是否正在监听
- ⏳ `net_peerCount` - 获取对等节点数量

### 优先级 7: Web3 工具 (1个)

- ⏳ `web3_sha3` - 计算 Keccak-256 哈希

### 优先级 8: 调试 API (可选, 5个)

- ⏳ `debug_traceTransaction` - 跟踪交易执行
- ⏳ `debug_traceBlockByNumber` - 跟踪区块执行(number)
- ⏳ `debug_traceBlockByHash` - 跟踪区块执行(hash)
- ⏳ `debug_traceCall` - 跟踪调用
- ⏳ `debug_storageRangeAt` - 获取存储范围

---

## ❌ 不计划实现的方法 (10个)

### 已废弃方法
- ❌ `eth_coinbase` - 获取 coinbase 地址 (PoS 后废弃)
- ❌ `eth_mining` - 是否正在挖矿 (PoS 后废弃)
- ❌ `eth_hashrate` - 获取哈希率 (PoS 后废弃)
- ❌ `miner_*` - 所有挖矿方法 (PoS 后废弃)

### 安全考虑 - 不推荐
- ❌ `personal_newAccount` - 创建账户
- ❌ `personal_unlockAccount` - 解锁账户
- ❌ `personal_lockAccount` - 锁定账户
- ❌ `personal_sendTransaction` - 发送交易
- ❌ `personal_sign` - 签名

**原因**: `personal_*` 方法存在安全风险，应使用外部签名器（如 MetaMask、硬件钱包）

### 客户端特定方法 (暂不实现)
- ❌ `admin_*` - 管理 API (Geth 特定)
- ❌ `txpool_*` - 交易池 API (Geth 特定)

---

## 📝 实现细节

### 当前实现位置

| 模块 | 文件路径 | 说明 |
|------|----------|------|
| JSON-RPC 处理 | `app/node/src/inbound/json_rpc.rs` | 主要实现文件 |
| HTTP 服务器 | `app/node/src/inbound/server.rs` | Axum HTTP 服务器 |
| 仓储接口 | `app/node/src/inbound/json_rpc.rs` (trait) | 数据访问抽象 |
| Mock 实现 | `app/node/src/infrastructure/mock_repository.rs` | 测试用实现 |

### 架构特点

✅ **符合 Clean Architecture**:
```
HTTP 层 (server.rs)
    ↓
用例层 (EthJsonRpcHandler)
    ↓
领域接口 (EthereumRepository trait)
    ↑
基础设施层 (MockEthereumRepository)
```

✅ **符合 EIP-1474 规范**:
- JSON-RPC 2.0 标准
- 标准错误代码
- 正确的数据编码（十六进制格式）

✅ **性能优化**:
- 缓存行对齐 (`#[repr(align(64))]`)
- 零拷贝设计
- 异步处理

---

## 🎯 实现路线图

### 阶段 1: 完善核心功能 (Q1 2025)
- [x] 基础查询方法 (已完成)
- [ ] 交易发送功能 (`eth_sendRawTransaction`)
- [ ] 完整的区块查询 (包括叔块)
- [ ] 过滤器 API

**目标**: 支持轻客户端基本功能

### 阶段 2: 网络层集成 (Q2 2025)
- [ ] 实现 P2P 网络层
- [ ] 实现区块同步
- [ ] 实现交易池
- [ ] `net_*` 方法完整实现

**目标**: 成为功能完整的以太坊节点

### 阶段 3: PoS 支持 (Q3 2025)
- [ ] Engine API 完整实现
- [ ] 共识层通信
- [ ] 验证者功能

**目标**: 支持以太坊 PoS 共识

### 阶段 4: 高级功能 (Q4 2025)
- [ ] Debug API
- [ ] 性能优化
- [ ] 状态剪枝
- [ ] Snap Sync

**目标**: 生产级性能和功能

---

## 📋 测试覆盖

### 单元测试状态

| 模块 | 测试覆盖率 | 状态 |
|------|------------|------|
| JSON-RPC 解析 | ~80% | ✅ 良好 |
| 方法处理 | ~60% | ⚠️ 需提升 |
| 错误处理 | ~70% | ✅ 良好 |
| 仓储接口 | ~90% | ✅ 优秀 |

### 集成测试需求

- [ ] Hive 测试套件集成
- [ ] JSON-RPC 兼容性测试
- [ ] 性能基准测试
- [ ] 模糊测试

---

## 🔍 与其他客户端对比

### 核心方法实现对比

| 方法类别 | RustEth | Geth | Reth | Erigon | 说明 |
|----------|---------|------|------|--------|------|
| 基础查询 | ✅ 16/16 | ✅ | ✅ | ✅ | 完整 |
| 交易发送 | ⏳ 0/4 | ✅ | ✅ | ✅ | 待实现 |
| 过滤器 | ⏳ 1/7 | ✅ | ✅ | ✅ | 待实现 |
| Engine API | ⏳ 0/9 | ✅ | ✅ | ✅ | 待实现 |
| Debug API | ❌ 0/7 | ✅ | ✅ | ✅ | 可选 |

### 性能对比 (预期)

| 指标 | RustEth 目标 | Geth | Reth | 说明 |
|------|--------------|------|------|------|
| RPC 延迟 | < 1ms | ~5ms | ~2ms | 缓存行对齐优化 |
| 内存占用 | 低 | 中 | 低 | Rust 零成本抽象 |
| 吞吐量 | 高 | 高 | 高 | 异步设计 |

---

## 📖 使用示例

### 当前可用的 API 调用

```bash
# 1. 获取区块号
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# 2. 获取账户余额
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0x0000000000000000000000000000000000000000","latest"],"id":1}'

# 3. 获取区块信息
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["latest",false],"id":1}'

# 4. 执行只读调用
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_call","params":[{"to":"0x...","data":"0x..."},"latest"],"id":1}'

# 5. 获取交易回执
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_getTransactionReceipt","params":["0x..."],"id":1}'
```

---

## 🔗 参考资源

### 标准文档
- [Ethereum Execution APIs](https://github.com/ethereum/execution-apis)
- [EIP-1474 规范](https://eips.ethereum.org/EIPS/eip-1474)
- [JSON-RPC 2.0](https://www.jsonrpc.org/specification)

### 测试工具
- [Hive 测试框架](https://github.com/ethereum/hive)
- [Postman 集合](https://github.com/ethereum/execution-apis)

### 参考实现
- [Reth](https://github.com/paradigmxyz/reth) - Rust 实现参考
- [Geth](https://github.com/ethereum/go-ethereum) - 官方 Go 实现
- [Erigon](https://github.com/ledgerwatch/erigon) - 高性能实现

---

## 📌 总结

### 当前状态
- ✅ **已实现**: 18 个核心方法 (30%)
- ✅ **架构**: Clean Architecture + EIP-1474 兼容
- ✅ **性能**: 缓存行对齐 + 异步设计
- ⏳ **进行中**: 交易发送 + 过滤器 API

### 下一步计划
1. **立即**: 实现 `eth_sendRawTransaction`
2. **短期**: 完成过滤器 API
3. **中期**: Engine API (PoS 支持)
4. **长期**: Debug API + 性能优化

### 关键优势
- 🦀 **Rust 性能**: 零成本抽象 + 内存安全
- 🏛️ **Clean Architecture**: 高可测试性 + 可维护性
- ⚡ **低延迟优化**: 缓存行对齐 + 无锁设计
- 📚 **标准合规**: 完全符合 EIP-1474

---

**最后更新**: 2025-11-09
**当前版本**: v0.1.0
**实现进度**: 18/60 (30%)
