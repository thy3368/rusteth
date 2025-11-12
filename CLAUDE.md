# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.
沟通用中文

严格按照eth相关标准并验收
遇到问题不要绕过而是找到根本原因

尽量采用静态分发类型
尽量无状态不可变编程
应用服务定义都用trait

服务与状态数据分开，使用类似erlang的通信机制。erlang编程的最佳实践
领域的顺序：内存版->单机版->分布式版

## Project Overview

RustEth is a high-performance Ethereum JSON-RPC server implementation based on EIP-1474, built with Rust following Clean
Architecture principles. The project prioritizes low-latency performance with cache-aligned data structures and
optimized compilation flags.

## Build Commands

### Build the project

```bash
# Development build (from project root)
cargo build

# Release build (optimized with LTO)
cargo build --release

# Build only the node application
cd app/node && cargo build --release
```

### Run the server

```bash
# Development mode
cargo run

# Release mode (optimized)
cargo run --release

# Run from node directory
cd app/node && cargo run --release
```

The server starts on `http://127.0.0.1:8545` (standard Ethereum RPC port).

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests for specific module
cargo test --package node

# Run a single test
cargo test test_mock_repository
```

### Test the server

```bash
# Health check
curl http://127.0.0.1:8545/health

# Get current block number
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Get balance
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0x0000000000000000000000000000000000000000","latest"],"id":1}'
```

## Architecture

The codebase strictly follows Clean Architecture with clear separation of concerns:

### Layer Structure

```
app/node/src/
├── inbound/          # Interface Adapters Layer (入站层)
│   ├── server.rs    # HTTP server (Axum)
│   └── jsonrpc.rs   # Domain entities + Use case layer
├── infrastructure/   # Infrastructure Layer (基础设施层)
│   └── mock_repository.rs  # Repository implementation
└── main.rs          # Application entry + Dependency Injection
```

### Dependency Flow

**Critical Architecture Rule**: Dependencies flow inward only. Inner layers NEVER depend on outer layers.

```
HTTP Server (server.rs)
    ↓ depends on
Use Case Layer (EthJsonRpcHandler in jsonrpc.rs)
    ↓ depends on
Domain Layer (EthereumRepository trait in jsonrpc.rs)
    ↑ implemented by
Infrastructure (MockEthereumRepository in mock_repository.rs)
```

### Key Components

**Domain Layer** (`jsonrpc.rs`):

- **Entities**: `Block`, `Transaction`, `TransactionReceipt`, `Log` (cache-aligned with `#[repr(align(64))]`)
- **Repository Trait**: `EthereumRepository` - defines the port interface
- Pure business logic, no I/O operations, no framework dependencies

**Use Case Layer** (`jsonrpc.rs`):

- **Handler**: `EthJsonRpcHandler` - orchestrates EIP-1474 JSON-RPC methods
- Depends only on the `EthereumRepository` trait (abstraction, not concrete implementation)
- Implements all 16+ Ethereum JSON-RPC methods: `eth_blockNumber`, `eth_getBalance`, `eth_call`, etc.

**Infrastructure Layer** (`mock_repository.rs`):

- **Concrete Implementation**: `MockEthereumRepository` - in-memory implementation
- Future implementations: `PostgresEthereumRepository`, `RPCEthereumRepository`
- All external dependencies (databases, external APIs) belong here

**Interface Layer** (`server.rs`):

- **HTTP Server**: Axum-based server with CORS and tracing middleware
- Converts HTTP requests to domain requests
- Routes: `POST /` (RPC), `GET /health`

**Application Entry** (`main.rs`):

- Dependency injection setup
- Wires concrete implementations to abstractions
- Configures logging and starts the server

## Adding New Features

### Adding a New JSON-RPC Method

1. Add method signature to `EthereumRepository` trait in `jsonrpc.rs`:

```rust
async fn get_uncle_count(&self, block: BlockId) -> Result<U64, RepositoryError>;
```

2. Implement in `MockEthereumRepository` (or your repository):

```rust
async fn get_uncle_count(&self, _block: BlockId) -> Result<U64, RepositoryError> {
    Ok(U64::zero())
}
```

3. Add method handler in `EthJsonRpcHandler::execute_method`:

```rust
match method {
"eth_getUncleCount" => self.eth_get_uncle_count(params).await,
// ...
}
```

4. Implement the handler method:

```rust
async fn eth_get_uncle_count(&self, params: serde_json::Value)
                             -> Result<serde_json::Value, RpcMethodError>
{
    let params: (BlockId,) = serde_json::from_value(params)?;
    let count = self.repository.get_uncle_count(params.0).await?;
    Ok(serde_json::to_value(count)?)
}
```

### Implementing a Real Repository

Replace `MockEthereumRepository` in `main.rs` with a real implementation:

```rust
// In infrastructure/postgres_repository.rs
pub struct PostgresEthereumRepository {
    pool: PgPool,
}

#[async_trait]
impl EthereumRepository for PostgresEthereumRepository {
    async fn get_block_number(&self) -> Result<U64, RepositoryError> {
        // Query database
    }
}

// In main.rs
let repository = Arc::new(PostgresEthereumRepository::new(pool));
```

## Performance Optimizations

This project follows strict low-latency standards:

### Compilation Flags (Cargo.toml)

- `opt-level = 3` - Maximum optimization
- `lto = "fat"` - Link-time optimization across all crates
- `codegen-units = 1` - Single codegen unit for better optimization
- `panic = "abort"` - Smaller binary, faster panics
- `strip = true` - Remove debug symbols

### Cache Alignment

Critical data structures use `#[repr(align(64))]` for cache-line alignment:

- `Block` struct (line 91-114 in jsonrpc.rs)
- `Transaction` struct (line 117-135 in jsonrpc.rs)

### Zero-Copy Design

- Minimal allocations in hot paths
- `Arc` for shared ownership without cloning
- Direct async trait methods (no intermediate buffers)

### Async Runtime

- Tokio configured for low-latency workloads
- Full feature set enabled for flexibility

## Code Organization Principles

1. **Single Responsibility**: Each module has one clear responsibility
2. **Dependency Inversion**: Use cases depend on trait abstractions, not concrete implementations
3. **Interface Segregation**: Repository trait defines only needed operations
4. **Testability**: Pure domain logic can be tested without external dependencies

## Supported EIP-1474 Methods

Standard Methods:

- `eth_blockNumber`, `eth_getBlockByNumber`, `eth_getBlockByHash`
- `eth_getTransactionByHash`, `eth_getTransactionReceipt`
- `eth_getBalance`, `eth_getStorageAt`, `eth_getTransactionCount`, `eth_getCode`
- `eth_call`, `eth_estimateGas`, `eth_getLogs`
- `eth_chainId`, `eth_gasPrice`

Network Methods:

- `net_version`, `web3_clientVersion`

All method implementations are in `EthJsonRpcHandler::execute_method` (line 280-302 in jsonrpc.rs).

## References

- [EIP-1474: Remote procedure call specification](https://eips.ethereum.org/EIPS/eip-1474)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- Clean Architecture principles from global CLAUDE.md apply to this codebase

## 学习资料

https://ethereum.org/developers/docs/