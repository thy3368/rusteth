# 以太坊 API 快速参考

## 🎯 标准来源

| 标准 | 地址 | 说明 |
|------|------|------|
| **官方规范** | https://github.com/ethereum/execution-apis | OpenRPC 格式的完整规范 |
| **文档** | https://ethereum.github.io/execution-apis/api-documentation/ | 在线文档 |
| **教程** | https://ethereum.org/developers/docs/apis/json-rpc/ | 开发者教程 |

**核心 EIP**:
- **EIP-1474**: JSON-RPC 基础规范
- **EIP-1767**: GraphQL 接口 (可选)

---

## 📚 API 命名空间

### 标准 API (所有客户端必须实现)

| 命名空间 | 方法数 | 用途 | 实现优先级 |
|----------|--------|------|------------|
| `eth_*` | ~39 | 以太坊核心功能 | ⭐⭐⭐⭐⭐ 最高 |
| `net_*` | 3 | 网络信息 | ⭐⭐⭐⭐ 高 |
| `web3_*` | 2 | Web3 工具 | ⭐⭐⭐⭐ 高 |
| `engine_*` | 9 | 共识层通信 (PoS) | ⭐⭐⭐ 中 |

### 可选 API (客户端特定)

| 命名空间 | 方法数 | 用途 | 实现优先级 |
|----------|--------|------|------------|
| `debug_*` | ~7 | 调试和跟踪 | ⭐⭐ 低 |
| `admin_*` | ~6 | 节点管理 (Geth) | ⭐ 可选 |
| `txpool_*` | ~3 | 交易池查询 (Geth) | ⭐ 可选 |

### 废弃 API (不应实现)

| 命名空间 | 原因 |
|----------|------|
| `miner_*` | PoS 后已废弃 |
| `personal_*` | 安全风险，应使用外部签名器 |

---

## 🔥 最重要的方法 (Top 20)

### 必须实现 (轻客户端最小集)

**状态读取** (6个):
1. `eth_getBalance` - 获取余额
2. `eth_getTransactionCount` - 获取 nonce
3. `eth_getCode` - 获取合约代码
4. `eth_getStorageAt` - 读取存储
5. `eth_call` - 只读调用
6. `eth_estimateGas` - 估算 gas

**区块查询** (3个):
7. `eth_blockNumber` - 当前区块号
8. `eth_getBlockByNumber` - 获取区块
9. `eth_getBlockByHash` - 获取区块

**交易** (5个):
10. `eth_sendRawTransaction` - 发送交易 ⭐ **最重要**
11. `eth_getTransactionByHash` - 查询交易
12. `eth_getTransactionReceipt` - 获取回执 ⭐ **最重要**
13. `eth_getTransactionByBlockNumberAndIndex` - 查询交易
14. `eth_getTransactionByBlockHashAndIndex` - 查询交易

**链信息** (3个):
15. `eth_chainId` - 链 ID
16. `eth_gasPrice` - gas 价格
17. `eth_syncing` - 同步状态

**网络** (2个):
18. `net_version` - 网络 ID
19. `web3_clientVersion` - 客户端版本

**日志** (1个):
20. `eth_getLogs` - 查询日志 ⭐ **最重要**

---

## 📋 方法分类速查

### 按功能分类

#### 1. 状态读取 (6个)
```
eth_getBalance          - 账户余额
eth_getStorageAt        - 合约存储
eth_getTransactionCount - 账户 nonce
eth_getCode             - 合约代码
eth_call                - 只读调用
eth_estimateGas         - gas 估算
```

#### 2. 区块查询 (7个)
```
eth_blockNumber                          - 当前区块号
eth_getBlockByHash                       - 获取区块 (hash)
eth_getBlockByNumber                     - 获取区块 (number)
eth_getBlockTransactionCountByHash       - 交易数 (hash)
eth_getBlockTransactionCountByNumber     - 交易数 (number)
eth_getUncleCountByBlockHash            - 叔块数 (hash)
eth_getUncleCountByBlockNumber          - 叔块数 (number)
```

#### 3. 交易操作 (9个)
```
eth_sendTransaction                      - 发送交易
eth_sendRawTransaction                   - 发送已签名交易 ⭐
eth_getTransactionByHash                 - 查询交易 (hash)
eth_getTransactionByBlockHashAndIndex    - 查询交易 (block hash + index)
eth_getTransactionByBlockNumberAndIndex  - 查询交易 (block number + index)
eth_getTransactionReceipt                - 获取回执 ⭐
eth_sign                                 - 签名数据
eth_signTransaction                      - 签名交易
```

#### 4. 过滤器与日志 (7个)
```
eth_newFilter                     - 创建过滤器
eth_newBlockFilter                - 创建区块过滤器
eth_newPendingTransactionFilter   - 创建待处理交易过滤器
eth_uninstallFilter               - 删除过滤器
eth_getFilterChanges              - 获取过滤器变化
eth_getFilterLogs                 - 获取过滤器所有日志
eth_getLogs                       - 查询日志 ⭐
```

#### 5. 链信息 (5个)
```
eth_chainId           - 链 ID
eth_syncing           - 同步状态
eth_gasPrice          - gas 价格
eth_protocolVersion   - 协议版本
eth_accounts          - 账户列表
```

#### 6. 叔块 (2个)
```
eth_getUncleByBlockHashAndIndex    - 获取叔块 (hash + index)
eth_getUncleByBlockNumberAndIndex  - 获取叔块 (number + index)
```

---

## 💡 数据类型规范

### 编码规则

| 类型 | 格式 | 示例 | 错误示例 |
|------|------|------|----------|
| **数量** | 十六进制，`0x` 前缀，无前导零 | `0x41` (65) | `0x041`, `41` |
| **数据** | 十六进制，`0x` 前缀，偶数位 | `0x41` | `0x4`, `41` |
| **地址** | 20 字节，十六进制 | `0x407d73d8a49eeb85d32cf465507dd71d507100c1` | 无 `0x` |
| **哈希** | 32 字节，十六进制 | `0xfe88c94d860f01a17f961bf4bdfb6e0c6cd10d3fda5cc861e805ca1240c58553` | 短于 32 字节 |

### 区块标识符

| 标识符 | 说明 | 示例 |
|--------|------|------|
| `"latest"` | 最新区块 | 大多数查询默认值 |
| `"earliest"` | 创世区块 | 区块号 0 |
| `"pending"` | 待处理区块 | 可能不被所有客户端支持 |
| `"safe"` | 安全区块 | PoS 特有 |
| `"finalized"` | 最终确定区块 | PoS 特有 |
| `"0x1234"` | 特定区块号 | 十六进制格式 |

---

## 🛠️ JSON-RPC 请求/响应示例

### 标准请求格式
```json
{
  "jsonrpc": "2.0",
  "method": "eth_blockNumber",
  "params": [],
  "id": 1
}
```

### 成功响应
```json
{
  "jsonrpc": "2.0",
  "result": "0x4b7",
  "id": 1
}
```

### 错误响应
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32601,
    "message": "Method not found"
  },
  "id": 1
}
```

### 标准错误代码

| 代码 | 说明 | 含义 |
|------|------|------|
| -32700 | Parse error | JSON 解析错误 |
| -32600 | Invalid Request | 无效的请求对象 |
| -32601 | Method not found | 方法不存在 |
| -32602 | Invalid params | 无效的参数 |
| -32603 | Internal error | 内部错误 |
| -32000 | Server error | 服务器端错误 |

---

## 🎯 RustEth 当前实现

### ✅ 已实现 (18个)

**eth_** (16个):
```
✅ eth_blockNumber
✅ eth_getBlockByNumber
✅ eth_getBlockByHash
✅ eth_getTransactionByHash
✅ eth_getTransactionReceipt
✅ eth_getBalance
✅ eth_getStorageAt
✅ eth_getTransactionCount
✅ eth_getCode
✅ eth_call
✅ eth_estimateGas
✅ eth_getLogs
✅ eth_chainId
✅ eth_gasPrice
```

**net_** (1个):
```
✅ net_version
```

**web3_** (1个):
```
✅ web3_clientVersion
```

### ⏳ 下一步实现

**优先级 1** (立即):
```
⏳ eth_sendRawTransaction  - 发送交易
⏳ eth_newFilter           - 创建过滤器
⏳ eth_getFilterChanges    - 获取过滤器变化
```

**优先级 2** (短期):
```
⏳ eth_syncing             - 同步状态
⏳ net_peerCount           - 对等节点数
⏳ engine_newPayloadV3     - Engine API
```

---

## 📊 客户端对比

| 客户端 | 语言 | eth_ 实现 | engine_ 实现 | debug_ 实现 |
|--------|------|-----------|--------------|-------------|
| **Geth** | Go | 39/39 ✅ | 9/9 ✅ | 7/7 ✅ |
| **Reth** | Rust | 39/39 ✅ | 9/9 ✅ | 7/7 ✅ |
| **Erigon** | Go | 39/39 ✅ | 9/9 ✅ | 5/7 ⚠️ |
| **Besu** | Java | 39/39 ✅ | 9/9 ✅ | 6/7 ⚠️ |
| **RustEth** | Rust | 16/39 ⏳ | 0/9 ⏳ | 0/7 ⏳ |

---

## 🚀 快速开始

### 测试已实现的 API

```bash
# 启动服务器
cargo run

# 测试 API
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

### 常用查询

```bash
# 1. 获取账户余额
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{
    "jsonrpc":"2.0",
    "method":"eth_getBalance",
    "params":["0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb","latest"],
    "id":1
  }'

# 2. 调用合约只读方法
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{
    "jsonrpc":"2.0",
    "method":"eth_call",
    "params":[{
      "to":"0x6B175474E89094C44Da98b954EedeAC495271d0F",
      "data":"0x70a08231000000000000000000000000742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
    },"latest"],
    "id":1
  }'

# 3. 获取交易回执
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{
    "jsonrpc":"2.0",
    "method":"eth_getTransactionReceipt",
    "params":["0x..."],
    "id":1
  }'
```

---

## 📚 学习资源

### 官方文档
- [Execution APIs GitHub](https://github.com/ethereum/execution-apis)
- [Ethereum.org JSON-RPC](https://ethereum.org/developers/docs/apis/json-rpc/)
- [EIP-1474](https://eips.ethereum.org/EIPS/eip-1474)

### 工具
- [Postman Collection](https://www.postman.com/ethereum-org)
- [OpenRPC Inspector](https://inspector.open-rpc.org/)
- [Hive 测试框架](https://github.com/ethereum/hive)

### 参考实现
- [Reth (Rust)](https://github.com/paradigmxyz/reth)
- [Geth (Go)](https://github.com/ethereum/go-ethereum)

---

## 💡 Tips

### 开发建议

1. **先实现读取方法**，再实现写入方法
2. **优先实现高频 API**：`eth_getBalance`, `eth_call`, `eth_getLogs`
3. **Engine API 是 PoS 必需**，但可以后期添加
4. **Debug API 是可选的**，主要用于开发和调试
5. **Personal API 已废弃**，使用外部签名器代替

### 测试建议

1. 使用 **Hive** 进行标准合规测试
2. 使用 **Postman** 进行手动测试
3. 编写**单元测试**覆盖所有错误情况
4. 进行**性能基准测试**

---

**快速查阅版本**: v1.0
**更新日期**: 2025-11-09
**适用于**: Ethereum Execution Layer Clients
