# BeaconApiServer 启动指南

BeaconApiServer 是一个遵循以太坊 Beacon API 标准的 RESTful 服务器实现。

## 快速开始

### 方式 1: Mock 服务器（用于开发/测试）

运行本地 Mock 服务器，返回预定义的测试数据：

```bash
# 启动 Mock 服务器
cargo run --example beacon_server_mock

# 服务器默认监听 http://127.0.0.1:8080
```

**测试端点**：

```bash
# 基础信息
curl http://127.0.0.1:8080/eth/v1/beacon/genesis | jq .
curl http://127.0.0.1:8080/eth/v1/node/version | jq .
curl http://127.0.0.1:8080/eth/v1/node/health
curl http://127.0.0.1:8080/eth/v1/node/syncing | jq .

# 配置查询
curl http://127.0.0.1:8080/eth/v1/config/spec | jq .
curl http://127.0.0.1:8080/eth/v1/config/fork_schedule | jq .

# 区块查询
curl http://127.0.0.1:8080/eth/v1/beacon/headers/head | jq .
curl http://127.0.0.1:8080/eth/v1/beacon/blocks/head/root | jq .

# 状态查询
curl http://127.0.0.1:8080/eth/v1/beacon/states/head/root | jq .
curl http://127.0.0.1:8080/eth/v1/beacon/states/head/fork | jq .
curl http://127.0.0.1:8080/eth/v1/beacon/states/head/finality_checkpoints | jq .
```

### 方式 2: 代理服务器（连接到真实 Beacon Node）

将本地端点代理到远程 Beacon Node：

```bash
# 设置环境变量
export BEACON_NODE_URL="https://ethereum-beacon-api.publicnode.com"
export SERVER_ADDR="127.0.0.1:8080"

# 启动代理服务器
cargo run --example beacon_server_proxy

# 测试（将自动转发到远程节点）
curl http://127.0.0.1:8080/eth/v1/beacon/genesis | jq .
```

**常用公开 Beacon Node 端点**：
- 主网: `https://ethereum-beacon-api.publicnode.com`
- 主网: `https://www.lightclientdata.org`
- Sepolia 测试网: `https://sepolia.beaconstate.info`
- Holesky 测试网: `https://holesky-beacon.stakely.io`

### 方式 3: 自定义实现

创建自己的 `BeaconApi` 实现：

```rust
use lib::domain::service::beacon_api::*;
use async_trait::async_trait;
use std::sync::Arc;

// 实现 BeaconApi trait
struct MyBeaconApi {
    // 你的数据源（数据库、缓存等）
}

#[async_trait]
impl BeaconApi for MyBeaconApi {
    async fn get_genesis(&self) -> Result<GenesisInfo, RepositoryError> {
        // 你的实现
        todo!()
    }

    // 实现其他方法...
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建你的实现
    let my_beacon_api = Arc::new(MyBeaconApi { /* ... */ });

    // 创建服务器
    let server = lib::domain::service::beacon_api_server::BeaconApiServer::new(my_beacon_api);

    // 构建路由
    let app = server.router();

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

## 配置选项

### 环境变量

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `SERVER_ADDR` | `127.0.0.1:8080` | 服务器监听地址 |
| `BEACON_NODE_URL` | `http://localhost:5052` | 远程 Beacon Node URL（仅代理模式） |

### 示例配置

```bash
# 自定义端口
export SERVER_ADDR="0.0.0.0:9000"

# 连接到本地 Lighthouse 节点
export BEACON_NODE_URL="http://localhost:5052"

# 连接到主网公开节点
export BEACON_NODE_URL="https://ethereum-beacon-api.publicnode.com"
```

## 支持的端点

BeaconApiServer 完整实现了以太坊 Beacon API 规范，支持以下端点组：

### 1. 基础信息 (Basic Information)
- `GET /eth/v1/beacon/genesis` - 获取创世信息
- `GET /eth/v1/node/version` - 获取节点版本
- `GET /eth/v1/node/health` - 健康检查
- `GET /eth/v1/node/syncing` - 同步状态
- `GET /eth/v1/node/identity` - 节点身份

### 2. 配置查询 (Configuration)
- `GET /eth/v1/config/spec` - 链规范参数
- `GET /eth/v1/config/fork_schedule` - 分叉时间表

### 3. 区块头查询 (Block Headers)
- `GET /eth/v1/beacon/headers` - 区块头列表
- `GET /eth/v1/beacon/headers/{block_id}` - 单个区块头

### 4. 区块查询 (Blocks)
- `GET /eth/v2/beacon/blocks/{block_id}` - 获取区块
- `GET /eth/v1/beacon/blocks/{block_id}/root` - 区块根哈希
- `GET /eth/v1/beacon/blocks/{block_id}/attestations` - 区块证明
- `POST /eth/v1/beacon/blocks` - 发布区块

### 5. 状态查询 (States)
- `GET /eth/v1/beacon/states/{state_id}/root` - 状态根
- `GET /eth/v1/beacon/states/{state_id}/fork` - 分叉信息
- `GET /eth/v1/beacon/states/{state_id}/finality_checkpoints` - 最终性检查点

### 6. 验证者查询 (Validators)
- `GET /eth/v1/beacon/states/{state_id}/validators` - 验证者列表
- `GET /eth/v1/beacon/states/{state_id}/validators/{validator_id}` - 单个验证者
- `GET /eth/v1/beacon/states/{state_id}/validator_balances` - 验证者余额
- `POST /eth/v1/beacon/states/{state_id}/validators` - 批量查询验证者

### 7. 委员会查询 (Committees)
- `GET /eth/v1/beacon/states/{state_id}/committees` - 委员会信息
- `GET /eth/v1/beacon/states/{state_id}/sync_committees` - 同步委员会

### 8. 交易池 (Pool)
- `GET /eth/v1/beacon/pool/attestations` - 待处理证明
- `POST /eth/v1/beacon/pool/attestations` - 提交证明
- `GET /eth/v1/beacon/pool/voluntary_exits` - 待处理退出
- `POST /eth/v1/beacon/pool/voluntary_exits` - 提交退出

## 架构设计

项目遵循 **Clean Architecture** 原则：

```
┌─────────────────────────────────────────┐
│   Interface Layer (接口层)              │
│   - beacon_api_server.rs               │
│   - beacon_api_client.rs               │
└─────────────────┬───────────────────────┘
                  │ 依赖
┌─────────────────▼───────────────────────┐
│   Domain Layer (领域层)                 │
│   - beacon_api.rs (trait)              │
│   - beacon_api_types.rs                │
└─────────────────────────────────────────┘
```

### 核心组件

1. **BeaconApi trait** (`beacon_api.rs`) - 领域接口
   - 定义所有 Beacon API 方法
   - 纯抽象，不依赖具体实现

2. **BeaconApiServer** (`beacon_api_server.rs`) - HTTP 服务器
   - 接收 HTTP 请求
   - 调用 BeaconApi trait 方法
   - 返回标准 Beacon API 响应

3. **BeaconApiClient** (`beacon_api_client.rs`) - HTTP 客户端
   - 实现 BeaconApi trait
   - 通过 HTTP 调用远程 Beacon Node

4. **共享类型** (`beacon_api_types.rs`)
   - ApiResponse<T> - 标准响应格式
   - ApiError - 错误响应
   - 查询参数和请求体类型

## 性能优化

服务器采用以下优化措施：

1. **缓存行对齐** - 关键数据结构使用 `#[repr(align(64))]`
2. **零拷贝** - 最小化内存分配和数据复制
3. **异步处理** - 基于 Tokio 异步运行时
4. **连接池** - HTTP 客户端复用连接

## 故障排查

### 服务器无法启动

```bash
# 检查端口是否被占用
lsof -i :8080

# 更换端口
export SERVER_ADDR="127.0.0.1:9000"
```

### 代理连接失败

```bash
# 检查远程节点是否可达
curl https://ethereum-beacon-api.publicnode.com/eth/v1/node/version

# 查看详细日志
RUST_LOG=debug cargo run --example beacon_server_proxy
```

### Mock 数据不符合预期

编辑 `examples/beacon_server_mock.rs` 中的 `MockBeaconApi` 实现，自定义返回数据。

## 开发和测试

### 运行测试

```bash
# 单元测试
cargo test

# 集成测试（需要网络）
cargo test --test beacon_api_integration_tests -- --ignored
```

### 启用详细日志

```bash
# 设置日志级别
export RUST_LOG=debug

# 或在代码中设置
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

## 标准参考

- [以太坊 Beacon API 规范](https://github.com/ethereum/beacon-APIs)
- [EIP-1474: JSON-RPC 规范](https://eips.ethereum.org/EIPS/eip-1474)
- [Clean Architecture 原则](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)

## 许可证

本项目遵循与主项目相同的许可证。
