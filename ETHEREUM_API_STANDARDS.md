# 以太坊执行客户端 API 标准

## 📚 官方标准与规范

### 核心规范来源

1. **Execution APIs 仓库** (主要标准)
   - 地址: https://github.com/ethereum/execution-apis
   - 许可: CC0-1.0 (公共领域)
   - 格式: OpenRPC 规范
   - 说明: 所有执行客户端必须实现的标准 API 集合

2. **官方文档**
   - 地址: https://ethereum.github.io/execution-apis/api-documentation/
   - 内容: JSON-RPC API 完整文档

3. **Ethereum.org 开发者文档**
   - 地址: https://ethereum.org/developers/docs/apis/json-rpc/
   - 内容: 面向开发者的 JSON-RPC 教程和参考

### 相关 EIP 标准

- **EIP-1474**: Remote procedure call specification (JSON-RPC 基础)
- **EIP-1767**: GraphQL interface to Ethereum node data
- **EIP-7769**: JSON-RPC API for ERC-4337 (2024年8月)

---

## 🔧 API 分类与命名空间

以太坊执行客户端 API 按功能分为以下命名空间：

### 1. `eth_` - 以太坊核心功能 (最重要)

这是执行客户端的主要 API 命名空间。

#### 1.1 状态读取方法 (State Methods)

| 方法 | 描述 | 参数 | 返回值 |
|------|------|------|--------|
| `eth_getBalance` | 获取账户余额 | address, block | Wei 余额 |
| `eth_getStorageAt` | 读取合约存储 | address, position, block | 存储值 |
| `eth_getTransactionCount` | 获取账户 nonce | address, block | 交易计数 |
| `eth_getCode` | 获取合约代码 | address, block | 字节码 |
| `eth_call` | 执行只读调用 | transaction, block | 返回数据 |
| `eth_estimateGas` | 估算 gas 消耗 | transaction | gas 数量 |

#### 1.2 区块方法 (Block Methods)

| 方法 | 描述 | 参数 | 返回值 |
|------|------|------|--------|
| `eth_blockNumber` | 获取最新区块号 | - | 区块号 |
| `eth_getBlockByHash` | 通过哈希获取区块 | hash, full | 区块对象 |
| `eth_getBlockByNumber` | 通过编号获取区块 | number, full | 区块对象 |
| `eth_getBlockTransactionCountByHash` | 获取区块交易数 | hash | 交易数量 |
| `eth_getBlockTransactionCountByNumber` | 获取区块交易数 | number | 交易数量 |
| `eth_getUncleCountByBlockHash` | 获取叔块数量 | hash | 叔块数量 |
| `eth_getUncleCountByBlockNumber` | 获取叔块数量 | number | 叔块数量 |

#### 1.3 交易方法 (Transaction Methods)

| 方法 | 描述 | 参数 | 返回值 |
|------|------|------|--------|
| `eth_sendTransaction` | 发送交易 | transaction | 交易哈希 |
| `eth_sendRawTransaction` | 发送已签名交易 | data | 交易哈希 |
| `eth_getTransactionByHash` | 获取交易详情 | hash | 交易对象 |
| `eth_getTransactionByBlockHashAndIndex` | 通过区块和索引获取交易 | hash, index | 交易对象 |
| `eth_getTransactionByBlockNumberAndIndex` | 通过区块号和索引获取交易 | number, index | 交易对象 |
| `eth_getTransactionReceipt` | 获取交易回执 | hash | 回执对象 |
| `eth_sign` | 签名数据 | address, data | 签名 |
| `eth_signTransaction` | 签名交易 | transaction | 已签名交易 |

#### 1.4 过滤器与日志方法 (Filter & Log Methods)

| 方法 | 描述 | 参数 | 返回值 |
|------|------|------|--------|
| `eth_newFilter` | 创建过滤器 | filter | 过滤器 ID |
| `eth_newBlockFilter` | 创建区块过滤器 | - | 过滤器 ID |
| `eth_newPendingTransactionFilter` | 创建待处理交易过滤器 | - | 过滤器 ID |
| `eth_uninstallFilter` | 删除过滤器 | filterId | 是否成功 |
| `eth_getFilterChanges` | 获取过滤器变化 | filterId | 日志数组 |
| `eth_getFilterLogs` | 获取过滤器所有日志 | filterId | 日志数组 |
| `eth_getLogs` | 查询日志 | filter | 日志数组 |

#### 1.5 链信息方法 (Chain Info Methods)

| 方法 | 描述 | 参数 | 返回值 |
|------|------|------|--------|
| `eth_chainId` | 获取链 ID | - | 链 ID |
| `eth_syncing` | 获取同步状态 | - | 同步信息/false |
| `eth_coinbase` | 获取 coinbase 地址 | - | 地址 |
| `eth_mining` | 是否正在挖矿 | - | 布尔值 |
| `eth_hashrate` | 获取哈希率 | - | 哈希率 |
| `eth_gasPrice` | 获取 gas 价格 | - | gas 价格 |
| `eth_accounts` | 获取账户列表 | - | 地址数组 |
| `eth_protocolVersion` | 获取协议版本 | - | 版本字符串 |

#### 1.6 叔块方法 (Uncle Methods)

| 方法 | 描述 | 参数 | 返回值 |
|------|------|------|--------|
| `eth_getUncleByBlockHashAndIndex` | 通过区块哈希和索引获取叔块 | hash, index | 叔块对象 |
| `eth_getUncleByBlockNumberAndIndex` | 通过区块号和索引获取叔块 | number, index | 叔块对象 |

---

### 2. `net_` - 网络信息

| 方法 | 描述 | 参数 | 返回值 |
|------|------|------|--------|
| `net_version` | 获取网络 ID | - | 网络 ID 字符串 |
| `net_listening` | 是否正在监听 | - | 布尔值 |
| `net_peerCount` | 获取对等节点数量 | - | 节点数量 |

---

### 3. `web3_` - Web3 工具方法

| 方法 | 描述 | 参数 | 返回值 |
|------|------|------|--------|
| `web3_clientVersion` | 获取客户端版本 | - | 版本字符串 |
| `web3_sha3` | 计算 Keccak-256 哈希 | data | 哈希值 |

---

### 4. `engine_` - 引擎 API (共识层-执行层通信)

**内部 API** - 用于共识客户端与执行客户端之间的通信。

| 方法 | 描述 | 用途 |
|------|------|------|
| `engine_newPayloadV1/V2/V3` | 接收新的执行载荷 | 区块执行 |
| `engine_forkchoiceUpdatedV1/V2/V3` | 更新分叉选择 | 链头更新 |
| `engine_getPayloadV1/V2/V3` | 获取执行载荷 | 区块构建 |
| `engine_exchangeTransitionConfigurationV1` | 交换转换配置 | 合并前配置 |
| `engine_getPayloadBodiesByHashV1` | 通过哈希获取载荷体 | 同步 |
| `engine_getPayloadBodiesByRangeV1` | 通过范围获取载荷体 | 同步 |

**规范**: https://github.com/ethereum/execution-apis/blob/main/src/engine/

---

### 5. `debug_` - 调试 API (可选)

这些是非标准方法，主要用于开发和调试。

| 方法 | 描述 | Geth | Erigon |
|------|------|------|--------|
| `debug_traceTransaction` | 跟踪交易执行 | ✅ | ✅ |
| `debug_traceBlockByNumber` | 跟踪区块执行 | ✅ | ✅ |
| `debug_traceBlockByHash` | 跟踪区块执行 | ✅ | ✅ |
| `debug_traceCall` | 跟踪调用 | ✅ | ✅ |
| `debug_storageRangeAt` | 获取存储范围 | ✅ | ✅ |
| `debug_getModifiedAccountsByNumber` | 获取修改的账户 | ✅ | ❌ |
| `debug_getModifiedAccountsByHash` | 获取修改的账户 | ✅ | ❌ |

---

### 6. `admin_` - 管理 API (Geth 特定)

节点管理功能（非标准）。

| 方法 | 描述 |
|------|------|
| `admin_addPeer` | 添加对等节点 |
| `admin_removePeer` | 移除对等节点 |
| `admin_nodeInfo` | 获取节点信息 |
| `admin_peers` | 获取对等节点列表 |
| `admin_startRPC` | 启动 RPC 服务器 |
| `admin_stopRPC` | 停止 RPC 服务器 |

---

### 7. `txpool_` - 交易池 API (Geth 特定)

| 方法 | 描述 |
|------|------|
| `txpool_content` | 获取交易池内容 |
| `txpool_inspect` | 检查交易池 |
| `txpool_status` | 获取交易池状态 |

---

### 8. `miner_` - 挖矿 API (已废弃)

PoS 后不再使用，仅用于开发链。

| 方法 | 描述 |
|------|------|
| `miner_start` | 开始挖矿 |
| `miner_stop` | 停止挖矿 |
| `miner_setEtherbase` | 设置 coinbase |
| `miner_setGasPrice` | 设置 gas 价格 |

---

### 9. `personal_` - 账户管理 API (已废弃)

**安全警告**: 不推荐使用，应使用外部签名器。

| 方法 | 描述 |
|------|------|
| `personal_newAccount` | 创建账户 |
| `personal_unlockAccount` | 解锁账户 |
| `personal_lockAccount` | 锁定账户 |
| `personal_sendTransaction` | 发送交易 |
| `personal_sign` | 签名 |

---

## 📊 API 标准化程度

### 标准化 API (所有客户端必须实现)

| 命名空间 | 标准化程度 | 文档 |
|----------|------------|------|
| `eth_` | ✅ 完全标准化 | execution-apis/eth.yaml |
| `net_` | ✅ 完全标准化 | execution-apis/net.yaml |
| `web3_` | ✅ 完全标准化 | execution-apis/web3.yaml |
| `engine_` | ✅ 完全标准化 | execution-apis/engine/ |

### 可选/客户端特定 API

| 命名空间 | 实现情况 | 说明 |
|----------|----------|------|
| `debug_` | 部分标准 | 各客户端实现不同 |
| `admin_` | Geth 特定 | 仅 Geth 实现 |
| `txpool_` | Geth 特定 | 仅 Geth 实现 |
| `miner_` | 已废弃 | PoS 后不再使用 |
| `personal_` | 已废弃 | 安全风险，不推荐 |

---

## 🏛️ API 设计原则

### JSON-RPC 2.0 规范

所有方法遵循 JSON-RPC 2.0 规范：

```json
// 请求
{
  "jsonrpc": "2.0",
  "method": "eth_blockNumber",
  "params": [],
  "id": 1
}

// 响应
{
  "jsonrpc": "2.0",
  "result": "0x4b7",
  "id": 1
}

// 错误
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32000,
    "message": "insufficient funds"
  },
  "id": 1
}
```

### 数据编码规范

1. **数量** (Quantities): 十六进制，前缀 `0x`，无前导零
   - 正确: `0x41` (65)
   - 错误: `0x041`, `41`

2. **数据** (Data): 十六进制，前缀 `0x`，偶数个字符
   - 正确: `0x41`
   - 错误: `0x041`, `0x4`, `41`

3. **地址**: 20 字节，十六进制，前缀 `0x`
   - 示例: `0x407d73d8a49eeb85d32cf465507dd71d507100c1`

4. **哈希**: 32 字节，十六进制，前缀 `0x`
   - 示例: `0xfe88c94d860f01a17f961bf4bdfb6e0c6cd10d3fda5cc861e805ca1240c58553`

### 区块标识符

| 标识符 | 描述 |
|--------|------|
| `"latest"` | 最新区块 |
| `"earliest"` | 创世区块 |
| `"pending"` | 待处理区块 |
| `"safe"` | 安全区块头 (PoS) |
| `"finalized"` | 最终确定区块 (PoS) |
| `"0x<number>"` | 特定区块号 |

---

## 📖 参考实现

### 主要执行客户端

| 客户端 | 语言 | 仓库 | 标准实现 |
|--------|------|------|----------|
| **Geth** | Go | ethereum/go-ethereum | ✅ 完整 + 扩展 |
| **Erigon** | Go | ledgerwatch/erigon | ✅ 完整 + 扩展 |
| **Besu** | Java | hyperledger/besu | ✅ 完整 |
| **Nethermind** | C# | NethermindEth/nethermind | ✅ 完整 |
| **Reth** | Rust | paradigmxyz/reth | ✅ 完整 |

---

## 🔍 测试与验证

### Hive 测试框架

官方使用 Hive 测试框架验证客户端 API 实现：
- 仓库: https://github.com/ethereum/hive
- 测试套件: JSON-RPC 一致性测试

### Speccheck 工具

验证测试用例与规范的一致性：
```bash
npm install -g @open-rpc/speccheck
speccheck -s openrpc.json -t tests/
```

---

## 📝 实现建议

### 必须实现的核心方法

**最小可用集合** (轻客户端):
```
eth_blockNumber
eth_chainId
eth_call
eth_estimateGas
eth_getBalance
eth_getBlockByNumber
eth_getCode
eth_getTransactionByHash
eth_getTransactionCount
eth_getTransactionReceipt
eth_sendRawTransaction
net_version
web3_clientVersion
```

**完整节点**:
- 所有 `eth_*` 标准方法
- 所有 `net_*` 方法
- 所有 `web3_*` 方法
- 所有 `engine_*` 方法 (如果支持 PoS)

### 实现优先级

1. **第一阶段**: 状态读取 + 区块查询
2. **第二阶段**: 交易发送 + 回执查询
3. **第三阶段**: 过滤器 + 日志查询
4. **第四阶段**: Engine API (PoS 支持)

---

## 🔗 相关资源

### 官方文档
- Execution APIs: https://github.com/ethereum/execution-apis
- API 文档: https://ethereum.github.io/execution-apis/api-documentation/
- Ethereum.org: https://ethereum.org/developers/docs/apis/json-rpc/

### 规范文件
- OpenRPC 规范: https://spec.open-rpc.org/
- JSON-RPC 2.0: https://www.jsonrpc.org/specification
- JSON Schema: https://json-schema.org/

### 工具
- OpenRPC Inspector: https://inspector.open-rpc.org/
- Postman Collection: 可从各客户端文档获取
- Hive 测试: https://github.com/ethereum/hive

---

## 📌 总结

### 关键要点

1. **标准来源**: `ethereum/execution-apis` GitHub 仓库
2. **核心格式**: OpenRPC + JSON-RPC 2.0
3. **必需命名空间**: `eth_`, `net_`, `web3_`
4. **内部 API**: `engine_` (共识层通信)
5. **可选扩展**: `debug_`, `admin_`, `txpool_`

### 本项目实现状态

当前项目 (`rusteth`) 已实现:
- ✅ JSON-RPC 2.0 框架
- ✅ 基础 `eth_*` 方法
- ✅ 基础 `net_*` 方法
- ✅ 基础 `web3_*` 方法
- ⏳ Engine API (待实现)

参考文件: `app/node/src/inbound/json_rpc.rs`

---

**最后更新**: 2025-11-09
**规范版本**: Execution APIs (最新)
