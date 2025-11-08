# RustEth Node - Ethereum JSON-RPC Server

A high-performance Ethereum JSON-RPC server implementation based on EIP-1474, built with Rust following Clean Architecture principles.

## Features

- **EIP-1474 Compliant**: Full implementation of Ethereum JSON-RPC 2.0 specification
- **Clean Architecture**: Separation of concerns with clear domain/use-case/infrastructure layers
- **Low Latency**: Optimized for performance with cache-aligned data structures
- **Type Safe**: Leverages Rust's type system for correctness
- **Async First**: Built on Tokio for efficient async I/O

## Supported RPC Methods

### Standard Methods (EIP-1474)

- `eth_blockNumber` - Returns the current block number
- `eth_getBlockByNumber` - Returns block by number
- `eth_getBlockByHash` - Returns block by hash
- `eth_getTransactionByHash` - Returns transaction by hash
- `eth_getTransactionReceipt` - Returns transaction receipt
- `eth_getBalance` - Returns account balance
- `eth_getStorageAt` - Returns storage value at position
- `eth_getTransactionCount` - Returns account nonce
- `eth_getCode` - Returns contract code
- `eth_call` - Executes a call without creating a transaction
- `eth_estimateGas` - Estimates gas for a transaction
- `eth_getLogs` - Returns logs matching filter
- `eth_chainId` - Returns chain ID
- `eth_gasPrice` - Returns current gas price

### Network Methods

- `net_version` - Returns network ID
- `web3_clientVersion` - Returns client version

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Interfaces Layer                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   HTTP/REST │  │   WebSocket │  │    gRPC     │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
└─────────┼─────────────────┼─────────────────┼───────────────┘
          │                 │                 │
┌─────────┼─────────────────┼─────────────────┼───────────────┐
│         │      Use Cases Layer (Business Logic)              │
│  ┌──────▼──────────────────────────────────────────┐        │
│  │        EthJsonRpcHandler                         │        │
│  │  - eth_blockNumber()                             │        │
│  │  - eth_getBalance()                              │        │
│  │  - eth_call()                                    │        │
│  └──────────────────────┬───────────────────────────┘        │
└─────────────────────────┼──────────────────────────────────┘
                          │
┌─────────────────────────┼──────────────────────────────────┐
│         Domain Layer    │  (Entities & Repository Ports)    │
│  ┌──────────────────────▼───────────────────────┐          │
│  │   EthereumRepository (trait)                 │          │
│  │   - Block, Transaction, Receipt entities    │          │
│  └──────────────────────────────────────────────┘          │
└─────────────────────────┬──────────────────────────────────┘
                          │
┌─────────────────────────┼──────────────────────────────────┐
│    Infrastructure Layer │  (Concrete Implementations)       │
│  ┌──────────────────────▼───────────────────────┐          │
│  │   MockEthereumRepository                     │          │
│  │   PostgresEthereumRepository (future)        │          │
│  │   RPCEthereumRepository (proxy, future)      │          │
│  └──────────────────────────────────────────────┘          │
└───────────────────────────────────────────────────────────┘
```

## Building and Running

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run --release
```

The server will start on `http://127.0.0.1:8545`

## Usage Examples

### Check Block Number

```bash
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{
    "jsonrpc": "2.0",
    "method": "eth_blockNumber",
    "params": [],
    "id": 1
  }'
```

### Get Block by Number

```bash
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{
    "jsonrpc": "2.0",
    "method": "eth_getBlockByNumber",
    "params": ["latest", false],
    "id": 1
  }'
```

### Get Balance

```bash
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{
    "jsonrpc": "2.0",
    "method": "eth_getBalance",
    "params": ["0x0000000000000000000000000000000000000000", "latest"],
    "id": 1
  }'
```

### Estimate Gas

```bash
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{
    "jsonrpc": "2.0",
    "method": "eth_estimateGas",
    "params": [{
      "from": "0x0000000000000000000000000000000000000000",
      "to": "0x0000000000000000000000000000000000000001",
      "value": "0x1"
    }],
    "id": 1
  }'
```

### Call Contract

```bash
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  --data '{
    "jsonrpc": "2.0",
    "method": "eth_call",
    "params": [{
      "to": "0x0000000000000000000000000000000000000000",
      "data": "0x70a08231"
    }, "latest"],
    "id": 1
  }'
```

### Health Check

```bash
curl http://127.0.0.1:8545/health
```

## Testing

Run tests:

```bash
cargo test
```

Run tests with output:

```bash
cargo test -- --nocapture
```

## Performance Optimizations

Following the low-latency development standards:

1. **Cache-Aligned Structures**: Critical data structures use `#[repr(align(64))]`
2. **Release Profile**: Optimized with LTO, single codegen unit, and native CPU features
3. **Zero-Copy**: Minimal allocations in hot paths
4. **Async Runtime**: Tokio configured for low-latency workloads

### Compilation Flags

The release profile uses:
- `opt-level = 3` - Maximum optimization
- `lto = "fat"` - Link-time optimization
- `codegen-units = 1` - Single codegen unit for better optimization
- `panic = "abort"` - Smaller binary, faster panics

## Extending the Implementation

### Adding a New RPC Method

1. Add the method to `EthJsonRpcHandler::execute_method()`:

```rust
match method {
    "eth_newMethod" => self.eth_new_method(params).await,
    // ...
}
```

2. Implement the method:

```rust
async fn eth_new_method(&self, params: serde_json::Value)
    -> Result<serde_json::Value, RpcMethodError>
{
    // Implementation
}
```

### Implementing a Real Repository

Replace `MockEthereumRepository` with a real implementation:

```rust
pub struct PostgresEthereumRepository {
    pool: PgPool,
}

#[async_trait]
impl EthereumRepository for PostgresEthereumRepository {
    async fn get_block_number(&self) -> Result<U64, RepositoryError> {
        // Query database
    }
    // ... implement other methods
}
```

## References

- [EIP-1474: Remote procedure call specification](https://eips.ethereum.org/EIPS/eip-1474)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [Ethereum JSON-RPC API](https://ethereum.org/en/developers/docs/apis/json-rpc/)

## License

MIT
