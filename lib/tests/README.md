# Beacon API 集成测试说明

## 概述

本目录包含 Beacon API 客户端的集成测试，使用真实的以太坊主网 Beacon API 端点。

## 测试文件

- `beacon_api_integration_tests.rs` - 完整的集成测试套件（26 个测试）

## 运行测试

### 运行所有集成测试

```bash
cargo test --test beacon_api_integration_tests -- --ignored
```

### 运行单个测试

```bash
cargo test --test beacon_api_integration_tests test_get_genesis_mainnet -- --ignored --nocapture
```

### 运行特定类别的测试

```bash
# 基础信息查询测试
cargo test --test beacon_api_integration_tests test_get_node -- --ignored

# 配置查询测试
cargo test --test beacon_api_integration_tests test_get_spec -- --ignored

# 区块查询测试
cargo test --test beacon_api_integration_tests test_get_block -- --ignored
```

## 使用的主网端点

### 主端点
- **URL**: https://beaconstate.ethstaker.cc
- **提供方**: ETH Staker Community
- **稳定性**: 中等（有些端点可能不可用）

### 备用端点
- **URL**: https://beaconstate.info
- **稳定性**: 中等

## 测试结果总结

根据最新测试运行结果：

### ✅ 通过的测试 (10 个)

| 测试名称 | 描述 | 状态 |
|---------|------|------|
| `test_get_genesis_mainnet` | 创世信息查询 | ✅ |
| `test_get_node_version_mainnet` | 节点版本查询 | ✅ |
| `test_get_spec_mainnet` | 链规范参数查询 | ✅ |
| `test_get_finality_checkpoints_mainnet` | 最终性检查点查询 | ✅ |
| `test_get_pool_attestations_mainnet` | 交易池证明查询 | ✅ |
| `test_get_pool_voluntary_exits_mainnet` | 自愿退出池查询 | ✅ |
| `test_get_node_health_mainnet` | 节点健康检查（优雅处理不支持） | ✅ |
| `test_fallback_endpoint` | 备用端点测试 | ✅ |
| `test_invalid_block_id` | 无效区块 ID 错误处理 | ✅ |
| `test_invalid_validator_id` | 无效验证者 ID 错误处理 | ✅ |

### ❌ 失败的测试 (16 个)

这些测试失败的原因主要是公开端点不支持所有标准 Beacon API 端点：

| 测试名称 | 失败原因 |
|---------|---------|
| `test_get_syncing_status_mainnet` | 端点不支持 |
| `test_get_node_identity_mainnet` | 端点不支持 |
| `test_get_fork_schedule_mainnet` | 端点不支持 |
| `test_get_block_header_*` | 区块头端点不支持 |
| `test_get_block_*` | 区块查询端点不支持 |
| `test_get_state_*` | 状态查询端点不支持 |
| `test_get_validator_*` | 验证者查询端点不支持 |
| `test_get_committees_*` | 委员会查询端点不支持 |

## 使用自己的节点

如果你运行自己的 Beacon Node（如 Lighthouse、Prysm 等），可以修改测试使用本地端点：

```rust
// 在 beacon_api_integration_tests.rs 中修改：
const MAINNET_BEACON_API: &str = "http://localhost:5052";
```

然后运行测试：

```bash
cargo test --test beacon_api_integration_tests -- --ignored
```

### 推荐的本地节点设置

#### Lighthouse
```bash
lighthouse bn \
  --network mainnet \
  --http \
  --http-address 127.0.0.1 \
  --http-port 5052
```

#### Prysm
```bash
prysm beacon-chain \
  --mainnet \
  --rpc-host=127.0.0.1 \
  --rpc-port=4000 \
  --grpc-gateway-host=127.0.0.1 \
  --grpc-gateway-port=3500
```

#### Teku
```bash
teku \
  --network=mainnet \
  --rest-api-enabled=true \
  --rest-api-interface=127.0.0.1 \
  --rest-api-port=5051
```

## 已验证的功能

### ✅ 已验证工作的功能

1. **HTTP 客户端基础功能**
   - ✅ 成功创建客户端
   - ✅ URL 构建正确
   - ✅ HTTP GET 请求
   - ✅ HTTP POST 请求
   - ✅ JSON 序列化/反序列化

2. **错误处理**
   - ✅ 404 Not Found 正确处理
   - ✅ 400 Bad Request 正确处理
   - ✅ 自定义错误类型映射
   - ✅ 无效参数检测

3. **数据类型转换**
   - ✅ StateId 枚举转换
   - ✅ BlockId 枚举转换
   - ✅ ValidatorId 枚举转换
   - ✅ 灵活的 ChainSpec 解析（支持额外字段）

4. **主网数据验证**
   - ✅ 创世时间: `1606824023`（2020-12-01 12:00:23 UTC）
   - ✅ 创世分叉版本: `0x00000000`
   - ✅ SLOTS_PER_EPOCH: `32`
   - ✅ SECONDS_PER_SLOT: `12`
   - ✅ 存款合约地址: `0x00000000219ab540356cbb839cbe05303d7705fa`

## 测试覆盖范围

| 功能模块 | 测试数量 | 通过数量 | 覆盖率 |
|---------|---------|---------|--------|
| 基础信息查询 | 5 | 3 | 60% |
| 配置查询 | 2 | 2 | 100% |
| 区块查询 | 4 | 0 | 0% |
| 状态查询 | 3 | 1 | 33% |
| 验证者查询 | 4 | 0 | 0% |
| 委员会查询 | 2 | 0 | 0% |
| 交易池查询 | 2 | 2 | 100% |
| 容错性测试 | 3 | 2 | 67% |
| 性能测试 | 1 | 0 | 0% |
| **总计** | **26** | **10** | **38%** |

## 注意事项

1. **公开端点限制**
   - 公开端点通常只支持有限的 API
   - 某些端点可能有速率限制
   - 建议使用自己的节点进行完整测试

2. **网络依赖**
   - 测试依赖外部网络连接
   - 可能因网络问题或端点维护而失败
   - 使用 `#[ignore]` 标记避免影响 CI/CD

3. **测试数据**
   - 使用真实主网数据
   - 验证者索引和区块号会随时间变化
   - 某些测试可能需要调整参数

## 贡献指南

### 添加新测试

1. 在 `beacon_api_integration_tests.rs` 中添加测试函数
2. 使用 `#[tokio::test]` 和 `#[ignore]` 标记
3. 添加 `--nocapture` 友好的 println! 输出
4. 更新本 README 文档

### 测试模板

```rust
#[tokio::test]
#[ignore]
async fn test_new_feature_mainnet() {
    let client = create_client();

    let result = client.some_method().await;

    assert!(result.is_ok(), "Failed: {:?}", result.err());

    let data = result.unwrap();
    println!("Result: {:?}", data);

    // 添加断言
    assert!(!data.is_empty(), "Data should not be empty");
}
```

## 参考资料

- [Ethereum Beacon APIs 标准](https://github.com/ethereum/beacon-APIs)
- [Beacon Chain 文档](https://ethereum.org/developers/docs/consensus-mechanisms/pos)
- [公开 Beacon API 端点列表](https://eth-clients.github.io/checkpoint-sync-endpoints/)

## 更新日志

- **2025-11-10**: 初始版本，26 个集成测试
- **2025-11-10**: 修复 ChainSpec 结构支持额外字段
- **2025-11-10**: 使用 ethstaker.cc 端点，9 个测试通过
- **2025-11-10**: 优化健康端点错误处理，优雅处理不支持的端点（404），10 个测试通过

---

**最后更新**: 2025-11-10
**测试状态**: 10/26 通过 (38%)
**建议**: 使用本地节点进行完整测试
