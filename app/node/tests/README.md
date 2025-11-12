# 集成测试目录

本目录包含了 RustEth 节点的集成测试。

## 目录结构

```
tests/
├── README.md                           # 本文件
└── eip1559_integration_tests.rs       # EIP-1559 相关的集成测试
```

## 测试分类

### EIP-1559 集成测试 (`eip1559_integration_tests.rs`)

包含以下测试类别：

#### 1. EIP-1559 转账测试（4个测试）
- `test_eip1559_send_transaction_basic` - 基础EIP-1559转账
- `test_eip1559_send_transaction_with_data` - 带数据的EIP-1559转账
- `test_legacy_send_transaction` - Legacy交易（向后兼容）
- `test_send_raw_transaction` - 发送原始交易

#### 2. 合约部署测试（5个测试）
- `test_contract_deployment_eip1559` - EIP-1559合约部署
- `test_contract_deployment_with_constructor_args` - 带构造函数参数的合约部署
- `test_contract_deployment_with_value` - 发送ETH的合约部署（payable构造函数）
- `test_estimate_gas_for_contract_deployment` - 估算合约部署gas
- `test_get_contract_code` - 获取合约代码

#### 3. 合约调用测试（5个测试）
- `test_contract_call_read_only` - 只读合约调用（view/pure函数）
- `test_contract_call_with_value` - 带value的合约调用（payable函数）
- `test_contract_transaction_eip1559` - EIP-1559合约交易
- `test_estimate_gas_for_contract_call` - 估算合约调用gas
- `test_get_contract_code` - 获取合约代码验证

#### 4. EIP-1559 费用测试（4个测试）
- `test_fee_history_basic` - 基础费用历史查询
- `test_fee_history_with_reward_percentiles` - 带奖励百分位数的费用历史
- `test_fee_history_specific_block` - 指定区块的费用历史
- `test_max_priority_fee_per_gas` - 获取建议的最大优先费用
- `test_gas_price_legacy` - 获取Legacy gas价格

#### 5. 综合集成测试（2个测试）
- `test_complete_eip1559_transaction_lifecycle` - 完整的EIP-1559交易生命周期
- `test_complete_contract_deployment_lifecycle` - 完整的合约部署生命周期

## 运行测试

### 运行所有集成测试
```bash
cargo test --tests
```

### 运行特定的测试文件
```bash
cargo test --test eip1559_integration_tests
```

### 运行特定的测试用例
```bash
cargo test --test eip1559_integration_tests test_eip1559_send_transaction_basic
```

### 显示测试输出
```bash
cargo test --test eip1559_integration_tests -- --nocapture
```

### 显示详细信息
```bash
cargo test --test eip1559_integration_tests -- --show-output
```

## 测试覆盖

当前测试覆盖了以下EIP-1559相关的JSON-RPC方法：

- ✅ `eth_sendTransaction` - 发送交易（支持EIP-1559和Legacy）
- ✅ `eth_sendRawTransaction` - 发送原始交易
- ✅ `eth_feeHistory` - 获取费用历史
- ✅ `eth_maxPriorityFeePerGas` - 获取建议的优先费用
- ✅ `eth_estimateGas` - 估算gas消耗
- ✅ `eth_call` - 执行调用（不创建交易）
- ✅ `eth_getCode` - 获取合约代码
- ✅ `eth_getTransactionCount` - 获取账户nonce
- ✅ `eth_gasPrice` - 获取gas价格

## 添加新测试

在添加新的集成测试时，请遵循以下规范：

1. **文件命名**: 使用 `<feature>_integration_tests.rs` 格式
2. **测试命名**: 使用 `test_<功能描述>` 格式，使用下划线分隔
3. **测试注释**: 使用中文注释说明测试意图
4. **断言**: 使用有意义的断言消息
5. **分组**: 使用注释分隔不同的测试类别

### 示例

```rust
// ============================================================================
// 新功能测试
// ============================================================================

#[tokio::test]
async fn test_new_feature() {
    let handler = create_test_handler();

    // 准备测试数据
    let param = create_test_param();

    // 执行测试
    let result = handler.new_method(param).await;

    // 验证结果
    assert!(result.is_ok(), "新功能应该成功");
}
```

## 持续集成

这些测试会在以下情况自动运行：

- 本地执行 `cargo test`
- CI/CD pipeline 中的测试阶段
- Pull Request 验证

## 相关文档

- [EIP-1559: Fee market change for ETH 1.0 chain](https://eips.ethereum.org/EIPS/eip-1559)
- [EIP-1474: Remote procedure call specification](https://eips.ethereum.org/EIPS/eip-1474)
- [项目架构文档](../CLAUDE.md)
