# Beacon API Server 使用示例

## 概述

`BeaconApiServer` 是一个基于 Axum 框架的高性能 RESTful 服务器实现,用于暴露符合以太坊标准的 Beacon API 端点。

## 架构设计

遵循 Clean Architecture 原则:

- **领域层**: `BeaconApi` trait 定义业务逻辑接口
- **接口层**: `BeaconApiServer` 将 HTTP 请求转换为领域用例调用
- **基础设施层**: 具体的 `BeaconApi` 实现 (如 `BeaconApiClient` 或本地实现)

## 基本用法

### 示例 1: 使用远程 Beacon Node 作为后端

```rust
use std::sync::Arc;
use lib::domain::service::beacon_api_client::BeaconApiClient;
use lib::domain::service::beacon_api_server::BeaconApiServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建 BeaconApiClient 连接到远程 Beacon Node
    let beacon_client = Arc::new(
        BeaconApiClient::new("http://localhost:5052")?
    );

    // 2. 创建 BeaconApiServer (代理模式)
    let server = BeaconApiServer::new(beacon_client);

    // 3. 构建 Axum Router
    let app = server.router();

    // 4. 启动 HTTP 服务器
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await?;

    println!("Beacon API Server listening on http://127.0.0.1:8080");

    axum::serve(listener, app).await?;

    Ok(())
}
```

### 示例 2: 使用自定义 BeaconApi 实现

```rust
use std::sync::Arc;
use async_trait::async_trait;
use lib::domain::service::beacon_api::*;
use lib::domain::service::beacon_api_server::BeaconApiServer;

// 自定义实现
struct MyBeaconNode {
    // 你的状态
}

#[async_trait]
impl BeaconApi for MyBeaconNode {
    async fn get_genesis(&self) -> Result<GenesisInfo, RepositoryError> {
        // 你的实现逻辑
        Ok(GenesisInfo {
            genesis_time: "1606824023".to_string(),
            genesis_validators_root: "0x4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95".to_string(),
            genesis_fork_version: "0x00000000".to_string(),
        })
    }

    async fn get_node_version(&self) -> Result<NodeVersion, RepositoryError> {
        Ok(NodeVersion {
            version: "RustEth/v1.0.0".to_string(),
        })
    }

    // ... 实现其他方法
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let my_node = Arc::new(MyBeaconNode { /* ... */ });
    let server = BeaconApiServer::new(my_node);
    let app = server.router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

### 示例 3: 添加中间件和 CORS

```rust
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use lib::domain::service::beacon_api_client::BeaconApiClient;
use lib::domain::service::beacon_api_server::BeaconApiServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    let beacon_client = Arc::new(BeaconApiClient::new("http://localhost:5052")?);
    let server = BeaconApiServer::new(beacon_client);

    // 添加 CORS 和日志中间件
    let app = server.router()
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        )
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;

    println!("Beacon API Server with CORS listening on http://127.0.0.1:8080");

    axum::serve(listener, app).await?;

    Ok(())
}
```

## 支持的端点

### 1. 基础信息查询

| 方法 | 端点 | 描述 |
|------|------|------|
| GET | `/eth/v1/beacon/genesis` | 获取创世信息 |
| GET | `/eth/v1/node/version` | 获取节点版本 |
| GET | `/eth/v1/node/health` | 获取节点健康状态 |
| GET | `/eth/v1/node/syncing` | 获取同步状态 |
| GET | `/eth/v1/node/identity` | 获取节点身份 |

### 2. 配置查询

| 方法 | 端点 | 描述 |
|------|------|------|
| GET | `/eth/v1/config/spec` | 获取链规范参数 |
| GET | `/eth/v1/config/fork_schedule` | 获取分叉时间表 |

### 3. 区块查询

| 方法 | 端点 | 描述 |
|------|------|------|
| GET | `/eth/v1/beacon/headers` | 获取区块头列表 |
| GET | `/eth/v1/beacon/headers/{block_id}` | 获取区块头 |
| GET | `/eth/v2/beacon/blocks/{block_id}` | 获取信标区块 |
| GET | `/eth/v1/beacon/blocks/{block_id}/root` | 获取区块根 |
| GET | `/eth/v1/beacon/blocks/{block_id}/attestations` | 获取区块证明 |
| POST | `/eth/v1/beacon/blocks` | 发布区块 |

### 4. 状态查询

| 方法 | 端点 | 描述 |
|------|------|------|
| GET | `/eth/v1/beacon/states/{state_id}/root` | 获取状态根 |
| GET | `/eth/v1/beacon/states/{state_id}/fork` | 获取分叉信息 |
| GET | `/eth/v1/beacon/states/{state_id}/finality_checkpoints` | 获取最终性检查点 |

### 5. 验证者查询

| 方法 | 端点 | 描述 |
|------|------|------|
| GET | `/eth/v1/beacon/states/{state_id}/validators` | 获取验证者列表 |
| POST | `/eth/v1/beacon/states/{state_id}/validators` | 批量查询验证者 |
| GET | `/eth/v1/beacon/states/{state_id}/validators/{validator_id}` | 获取单个验证者 |
| GET | `/eth/v1/beacon/states/{state_id}/validator_balances` | 获取验证者余额 |

### 6. 委员会查询

| 方法 | 端点 | 描述 |
|------|------|------|
| GET | `/eth/v1/beacon/states/{state_id}/committees` | 获取委员会信息 |
| GET | `/eth/v1/beacon/states/{state_id}/sync_committees` | 获取同步委员会 |

### 7. 交易池查询

| 方法 | 端点 | 描述 |
|------|------|------|
| GET | `/eth/v1/beacon/pool/attestations` | 获取待处理证明 |
| POST | `/eth/v1/beacon/pool/attestations` | 提交证明 |
| GET | `/eth/v1/beacon/pool/voluntary_exits` | 获取自愿退出 |
| POST | `/eth/v1/beacon/pool/voluntary_exits` | 提交自愿退出 |

## 请求示例

### 获取创世信息

```bash
curl http://127.0.0.1:8080/eth/v1/beacon/genesis
```

响应:
```json
{
  "data": {
    "genesis_time": "1606824023",
    "genesis_validators_root": "0x4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95",
    "genesis_fork_version": "0x00000000"
  }
}
```

### 获取区块头

```bash
curl http://127.0.0.1:8080/eth/v1/beacon/headers/head
```

### 获取验证者

```bash
# 按状态过滤
curl "http://127.0.0.1:8080/eth/v1/beacon/states/head/validators?status=active_ongoing"

# 按 ID 过滤
curl "http://127.0.0.1:8080/eth/v1/beacon/states/head/validators?id=1&id=2&id=3"
```

### 查询节点健康状态

```bash
curl -i http://127.0.0.1:8080/eth/v1/node/health
```

响应状态码:
- `200 OK`: 节点健康
- `206 Partial Content`: 节点正在同步
- `503 Service Unavailable`: 节点不健康

## 参数格式

### StateId

- `head`: 当前头部状态
- `genesis`: 创世状态
- `finalized`: 最终确定状态
- `justified`: 最新合理状态
- `<slot>`: 特定 slot 编号 (如 `12345`)
- `0x<root>`: 特定状态根哈希

### BlockId

- `head`: 当前头部区块
- `genesis`: 创世区块
- `finalized`: 最终确定区块
- `<slot>`: 特定 slot 编号 (如 `12345`)
- `0x<root>`: 特定区块根哈希

### ValidatorId

- `<index>`: 验证者索引 (如 `12345`)
- `0x<pubkey>`: BLS 公钥 (48字节十六进制)

## 错误处理

所有错误都遵循标准格式:

```json
{
  "code": 404,
  "message": "Resource not found: /eth/v1/beacon/blocks/99999999",
  "stacktraces": []
}
```

HTTP 状态码:
- `200 OK`: 成功
- `400 Bad Request`: 参数错误
- `404 Not Found`: 资源未找到
- `206 Partial Content`: 节点正在同步
- `503 Service Unavailable`: 节点不健康
- `500 Internal Server Error`: 内部错误

## 性能优化

### 缓存行对齐

关键数据结构使用 `#[repr(align(64))]` 进行缓存行对齐:

```rust
#[repr(align(64))]
pub struct BeaconApiServer<T: BeaconApi> {
    beacon_api: Arc<T>,
}
```

### 零拷贝设计

- 使用 `Arc` 共享 API 实现,避免克隆
- 直接传递引用,最小化内存分配
- 异步处理,非阻塞 I/O

### 编译优化

在 `Cargo.toml` 中启用:

```toml
[profile.release]
opt-level = 3           # 最高优化级别
lto = "fat"             # 链接时优化
codegen-units = 1       # 单个代码生成单元
panic = "abort"         # 更快的 panic 处理
strip = true            # 移除调试符号
```

## 测试

运行测试:

```bash
cd lib
cargo test --lib beacon_api_server::tests
```

测试覆盖:
- 参数解析 (StateId, BlockId, ValidatorId)
- 错误响应格式化
- 路由配置

## 参考

- [以太坊 Beacon API 标准](https://github.com/ethereum/beacon-APIs)
- [Axum 文档](https://docs.rs/axum/)
- [Clean Architecture 原则](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
