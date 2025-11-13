//! EIP-1559 交易、合约部署和调用的集成测试
//!
//! 本模块包含以下测试：
//! - EIP-1559 转账测试
//! - 合约部署测试
//! - 合约调用测试
//! - EIP-1559 费用历史测试

use node::infrastructure::json_rpc_trait::EthApiExecutor;

use node::infrastructure::mock_repository::MockEthereumRepository;
use node::service::ethereum_service_impl::EthereumServiceImpl;
use ethereum_types::{Address, U256, U64};
use node::inbound::json_rpc::EthJsonRpcHandler;
use node::domain::command_types::{BlockId, BlockTag, CallRequest, SendTransactionRequest};

#[cfg(test)]
mod eth_api_client_test;

/// 创建测试用的 Handler
fn create_test_handler() -> EthJsonRpcHandler<EthereumServiceImpl> {
    let repository = MockEthereumRepository::new();
    let service = EthereumServiceImpl::new(repository);
    EthJsonRpcHandler::new(service)
}

// ============================================================================
// EIP-1559 转账测试
// ============================================================================

#[tokio::test]
async fn test_eip1559_send_transaction_basic() {
    let handler = create_test_handler();

    // 创建 EIP-1559 转账请求
    let from_addr = Address::from_low_u64_be(1);
    let to_addr = Address::from_low_u64_be(2);

    let tx_request = SendTransactionRequest {
        from: from_addr,
        to: Some(to_addr),
        gas: Some(U256::from(21000)),
        gas_price: None, // EIP-1559 不使用 gasPrice
        value: Some(U256::from(1_000_000_000_000_000_000u64)), // 1 ETH
        data: None,
        nonce: Some(U256::zero()),
        // EIP-1559 特定字段
        max_fee_per_gas: Some(U256::from(30_000_000_000u64)), // 30 Gwei
        max_priority_fee_per_gas: Some(U256::from(2_000_000_000u64)), // 2 Gwei
    };

    let params = serde_json::json!([tx_request]);
    let result = handler.eth_send_transaction(params).await;

    assert!(result.is_ok(), "EIP-1559 转账应该成功");
    let tx_hash = result.unwrap();

    // 验证返回的交易哈希不为空
    assert!(tx_hash.is_string(), "交易哈希应该是字符串");
}

#[tokio::test]
async fn test_eip1559_send_transaction_with_data() {
    let handler = create_test_handler();

    let from_addr = Address::from_low_u64_be(100);
    let to_addr = Address::from_low_u64_be(200);

    // 包含数据的 EIP-1559 交易
    let tx_request = SendTransactionRequest {
        from: from_addr,
        to: Some(to_addr),
        gas: Some(U256::from(50000)),
        gas_price: None,
        value: Some(U256::from(500_000_000_000_000_000u64)), // 0.5 ETH
        data: Some(hex::decode("a9059cbb000000000000000000000000").unwrap()), // ERC20 transfer
        nonce: Some(U256::from(5)),
        max_fee_per_gas: Some(U256::from(50_000_000_000u64)),
        max_priority_fee_per_gas: Some(U256::from(3_000_000_000u64)),
    };

    let params = serde_json::json!([tx_request]);
    let result = handler.eth_send_transaction(params).await;

    assert!(result.is_ok(), "带数据的 EIP-1559 交易应该成功");
}

#[tokio::test]
async fn test_legacy_send_transaction() {
    let handler = create_test_handler();

    let from_addr = Address::from_low_u64_be(10);
    let to_addr = Address::from_low_u64_be(20);

    // Legacy 交易（使用 gasPrice）
    let tx_request = SendTransactionRequest {
        from: from_addr,
        to: Some(to_addr),
        gas: Some(U256::from(21000)),
        gas_price: Some(U256::from(20_000_000_000u64)), // 20 Gwei
        value: Some(U256::from(1_000_000_000_000_000_000u64)),
        data: None,
        nonce: Some(U256::zero()),
        max_fee_per_gas: None, // Legacy 不使用 EIP-1559 字段
        max_priority_fee_per_gas: None,
    };

    let params = serde_json::json!([tx_request]);
    let result = handler.eth_send_transaction(params).await;

    assert!(result.is_ok(), "Legacy 交易应该成功");
}

#[tokio::test]
async fn test_send_raw_transaction() {
    let handler = create_test_handler();

    // 模拟已签名的原始交易数据（EIP-1559 类型 2）
    let raw_tx = "02f876018203e882520894b5409d8a3d6c5e9d1bbd635f2bb1c28f2c1e2c8a8080c001a0c6b5b3f8d9e7a2c1f4d6b8e3a5c7d9f1e2a4c6b8d0e2f4a6c8e0f2a4c6b8d0a0e2f4a6c8d0e2f4a6c8d0e2f4a6c8d0e2f4a6c8d0e2f4a6c8d0e2f4a6c8d0";
    let params = serde_json::json!([format!("0x{}", raw_tx)]);

    let result = handler.eth_send_raw_transaction(params).await;

    assert!(result.is_ok(), "发送原始交易应该成功");
    let tx_hash = result.unwrap();
    assert!(tx_hash.is_string(), "交易哈希应该是字符串");
}

// ============================================================================
// 合约部署测试
// ============================================================================

#[tokio::test]
async fn test_contract_deployment_eip1559() {
    let handler = create_test_handler();

    let from_addr = Address::from_low_u64_be(1000);

    // 合约字节码（示例：简单的存储合约）
    let contract_bytecode = hex::decode(
        "608060405234801561001057600080fd5b50610150806100206000396000f3fe"
    ).unwrap();

    // EIP-1559 合约部署交易（to 为 None）
    let deploy_request = SendTransactionRequest {
        from: from_addr,
        to: None, // 合约部署时 to 为 None
        gas: Some(U256::from(200000)),
        gas_price: None,
        value: None, // 不发送 ETH
        data: Some(contract_bytecode),
        nonce: Some(U256::zero()),
        max_fee_per_gas: Some(U256::from(40_000_000_000u64)),
        max_priority_fee_per_gas: Some(U256::from(2_500_000_000u64)),
    };

    let params = serde_json::json!([deploy_request]);
    let result = handler.eth_send_transaction(params).await;

    assert!(result.is_ok(), "EIP-1559 合约部署应该成功");
}

#[tokio::test]
async fn test_contract_deployment_with_constructor_args() {
    let handler = create_test_handler();

    let from_addr = Address::from_low_u64_be(1001);

    // 合约字节码 + 构造函数参数编码
    let contract_bytecode_with_args = hex::decode(
        "608060405234801561001057600080fd5b50610150806100206000396000f3fe00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002"
    ).unwrap();

    let deploy_request = SendTransactionRequest {
        from: from_addr,
        to: None,
        gas: Some(U256::from(300000)),
        gas_price: None,
        value: None,
        data: Some(contract_bytecode_with_args),
        nonce: Some(U256::from(1)),
        max_fee_per_gas: Some(U256::from(35_000_000_000u64)),
        max_priority_fee_per_gas: Some(U256::from(2_000_000_000u64)),
    };

    let params = serde_json::json!([deploy_request]);
    let result = handler.eth_send_transaction(params).await;

    assert!(result.is_ok(), "带构造函数参数的合约部署应该成功");
}

#[tokio::test]
async fn test_contract_deployment_with_value() {
    let handler = create_test_handler();

    let from_addr = Address::from_low_u64_be(1002);

    // Payable 构造函数的合约部署
    let contract_bytecode = hex::decode("608060405234801561001057600080fd5b50").unwrap();

    let deploy_request = SendTransactionRequest {
        from: from_addr,
        to: None,
        gas: Some(U256::from(250000)),
        gas_price: None,
        value: Some(U256::from(100_000_000_000_000_000u64)), // 0.1 ETH
        data: Some(contract_bytecode),
        nonce: Some(U256::from(2)),
        max_fee_per_gas: Some(U256::from(45_000_000_000u64)),
        max_priority_fee_per_gas: Some(U256::from(3_000_000_000u64)),
    };

    let params = serde_json::json!([deploy_request]);
    let result = handler.eth_send_transaction(params).await;

    assert!(result.is_ok(), "发送 ETH 的合约部署应该成功");
}

#[tokio::test]
async fn test_estimate_gas_for_contract_deployment() {
    let handler = create_test_handler();

    let from_addr = Address::from_low_u64_be(1003);
    let contract_bytecode = hex::decode("608060405234801561001057600080fd5b50").unwrap();

    let call_request = CallRequest {
        from: Some(from_addr),
        to: None, // 合约部署
        gas: None,
        gas_price: None,
        value: None,
        data: Some(contract_bytecode),
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };

    let params = serde_json::json!([call_request]);
    let result = handler.eth_estimate_gas(params).await;

    assert!(result.is_ok(), "估算合约部署 gas 应该成功");

    if let Ok(gas_estimate) = result {
        let gas: U256 = serde_json::from_value(gas_estimate).unwrap();
        assert!(gas > U256::zero(), "Gas 估算应该大于 0");
    }
}

// ============================================================================
// 合约调用测试
// ============================================================================

#[tokio::test]
async fn test_contract_call_read_only() {
    let handler = create_test_handler();

    let contract_addr = Address::from_low_u64_be(5000);

    // 调用 ERC20 balanceOf(address) 方法
    // balanceOf 函数签名: 0x70a08231
    let balance_of_data = hex::decode(
        "70a082310000000000000000000000001234567890123456789012345678901234567890"
    ).unwrap();

    let call_request = CallRequest {
        from: None,
        to: Some(contract_addr),
        gas: None,
        gas_price: None,
        value: None,
        data: Some(balance_of_data),
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };

    let params = serde_json::json!([call_request, "latest"]);
    let result = handler.eth_call(params).await;

    assert!(result.is_ok(), "只读合约调用应该成功");
}

#[tokio::test]
async fn test_contract_call_with_value() {
    let handler = create_test_handler();

    let from_addr = Address::from_low_u64_be(2000);
    let contract_addr = Address::from_low_u64_be(5001);

    // 调用 payable 函数
    let function_data = hex::decode("a9059cbb").unwrap();

    let call_request = CallRequest {
        from: Some(from_addr),
        to: Some(contract_addr),
        gas: Some(U256::from(100000)),
        gas_price: None,
        value: Some(U256::from(500_000_000_000_000_000u64)), // 0.5 ETH
        data: Some(function_data),
        max_fee_per_gas: Some(U256::from(30_000_000_000u64)),
        max_priority_fee_per_gas: Some(U256::from(2_000_000_000u64)),
    };

    let params = serde_json::json!([call_request, "latest"]);
    let result = handler.eth_call(params).await;

    assert!(result.is_ok(), "带 value 的合约调用应该成功");
}

#[tokio::test]
async fn test_contract_transaction_eip1559() {
    let handler = create_test_handler();

    let from_addr = Address::from_low_u64_be(2001);
    let contract_addr = Address::from_low_u64_be(5002);

    // ERC20 transfer(address, uint256)
    // 函数签名: 0xa9059cbb
    let transfer_data = hex::decode(
        "a9059cbb000000000000000000000000abcdef1234567890abcdef1234567890abcdef1200000000000000000000000000000000000000000000000000000000000003e8"
    ).unwrap();

    let tx_request = SendTransactionRequest {
        from: from_addr,
        to: Some(contract_addr),
        gas: Some(U256::from(65000)),
        gas_price: None,
        value: None,
        data: Some(transfer_data),
        nonce: Some(U256::from(10)),
        max_fee_per_gas: Some(U256::from(35_000_000_000u64)),
        max_priority_fee_per_gas: Some(U256::from(2_500_000_000u64)),
    };

    let params = serde_json::json!([tx_request]);
    let result = handler.eth_send_transaction(params).await;

    assert!(result.is_ok(), "EIP-1559 合约交易应该成功");
}

#[tokio::test]
async fn test_estimate_gas_for_contract_call() {
    let handler = create_test_handler();

    let from_addr = Address::from_low_u64_be(2002);
    let contract_addr = Address::from_low_u64_be(5003);

    // 复杂合约调用
    let function_data = hex::decode(
        "a9059cbb000000000000000000000000deadbeefdeadbeefdeadbeefdeadbeefdeadbeef0000000000000000000000000000000000000000000000000000000000001000"
    ).unwrap();

    let call_request = CallRequest {
        from: Some(from_addr),
        to: Some(contract_addr),
        gas: None,
        gas_price: None,
        value: None,
        data: Some(function_data),
        max_fee_per_gas: Some(U256::from(40_000_000_000u64)),
        max_priority_fee_per_gas: Some(U256::from(3_000_000_000u64)),
    };

    let params = serde_json::json!([call_request]);
    let result = handler.eth_estimate_gas(params).await;

    assert!(result.is_ok(), "估算合约调用 gas 应该成功");

    if let Ok(gas_estimate) = result {
        let gas: U256 = serde_json::from_value(gas_estimate).unwrap();
        assert!(gas > U256::zero(), "Gas 估算应该大于 0");
    }
}

#[tokio::test]
async fn test_get_contract_code() {
    let handler = create_test_handler();

    let contract_addr = Address::from_low_u64_be(5004);
    let params = serde_json::json!([contract_addr, "latest"]);

    let result = handler.eth_get_code(params).await;

    assert!(result.is_ok(), "获取合约代码应该成功");
}

// ============================================================================
// EIP-1559 费用相关测试
// ============================================================================

#[tokio::test]
async fn test_fee_history_basic() {
    let handler = create_test_handler();

    // 请求最近 5 个区块的费用历史
    let block_count = U64::from(5);
    let newest_block = BlockId::Tag(BlockTag::Latest);
    let reward_percentiles = None::<Vec<f64>>;

    let params = serde_json::json!([block_count, newest_block, reward_percentiles]);
    let result = handler.eth_fee_history(params).await;

    assert!(result.is_ok(), "获取费用历史应该成功");

    if let Ok(fee_history) = result {
        let history: serde_json::Value = fee_history;

        // 验证响应结构
        assert!(history.get("oldestBlock").is_some(), "应该包含 oldestBlock");
        assert!(history.get("baseFeePerGas").is_some(), "应该包含 baseFeePerGas");
        assert!(history.get("gasUsedRatio").is_some(), "应该包含 gasUsedRatio");

        // 验证数组长度
        let base_fees = history["baseFeePerGas"].as_array().unwrap();
        assert_eq!(base_fees.len(), 6, "baseFeePerGas 应该有 6 个元素（包含下一个区块）");

        let gas_ratios = history["gasUsedRatio"].as_array().unwrap();
        assert_eq!(gas_ratios.len(), 5, "gasUsedRatio 应该有 5 个元素");
    }
}

#[tokio::test]
async fn test_fee_history_with_reward_percentiles() {
    let handler = create_test_handler();

    // 请求费用历史，包含奖励百分位数
    let block_count = U64::from(3);
    let newest_block = BlockId::Tag(BlockTag::Latest);
    let reward_percentiles = Some(vec![25.0, 50.0, 75.0]);

    let params = serde_json::json!([block_count, newest_block, reward_percentiles]);
    let result = handler.eth_fee_history(params).await;

    assert!(result.is_ok(), "带奖励百分位数的费用历史应该成功");

    if let Ok(fee_history) = result {
        let history: serde_json::Value = fee_history;

        // 验证包含奖励数据
        assert!(history.get("reward").is_some(), "应该包含 reward 字段");

        let rewards = history["reward"].as_array().unwrap();
        assert_eq!(rewards.len(), 3, "reward 应该有 3 个区块的数据");

        // 每个区块应该有 3 个百分位数
        for reward in rewards {
            let percentiles = reward.as_array().unwrap();
            assert_eq!(percentiles.len(), 3, "每个区块应该有 3 个百分位数");
        }
    }
}

#[tokio::test]
async fn test_fee_history_specific_block() {
    let handler = create_test_handler();

    // 请求特定区块的费用历史
    let block_count = U64::from(10);
    let newest_block = BlockId::Number(U64::from(1000));
    let reward_percentiles = None::<Vec<f64>>;

    let params = serde_json::json!([block_count, newest_block, reward_percentiles]);
    let result = handler.eth_fee_history(params).await;

    assert!(result.is_ok(), "指定区块的费用历史应该成功");
}

#[tokio::test]
async fn test_max_priority_fee_per_gas() {
    let handler = create_test_handler();

    let result = handler.eth_max_priority_fee_per_gas().await;

    assert!(result.is_ok(), "获取最大优先费用应该成功");

    if let Ok(fee) = result {
        let priority_fee: U256 = serde_json::from_value(fee).unwrap();
        assert!(priority_fee > U256::zero(), "优先费用应该大于 0");
        assert!(
            priority_fee <= U256::from(10_000_000_000u64),
            "优先费用应该在合理范围内（<= 10 Gwei）"
        );
    }
}

#[tokio::test]
async fn test_gas_price_legacy() {
    let handler = create_test_handler();

    let result = handler.eth_gas_price().await;

    assert!(result.is_ok(), "获取 gas 价格应该成功");

    if let Ok(price) = result {
        let gas_price: U256 = serde_json::from_value(price).unwrap();
        assert!(gas_price > U256::zero(), "Gas 价格应该大于 0");
    }
}

// ============================================================================
// 综合集成测试
// ============================================================================

#[tokio::test]
async fn test_complete_eip1559_transaction_lifecycle() {
    let handler = create_test_handler();

    let from_addr = Address::from_low_u64_be(3000);
    let to_addr = Address::from_low_u64_be(4000);

    // 1. 获取账户 nonce
    let nonce_params = serde_json::json!([from_addr, "latest"]);
    let nonce_result = handler.eth_get_transaction_count(nonce_params).await;
    assert!(nonce_result.is_ok(), "获取 nonce 应该成功");

    // 2. 获取建议的优先费用
    let priority_fee_result = handler.eth_max_priority_fee_per_gas().await;
    assert!(priority_fee_result.is_ok(), "获取优先费用应该成功");
    let priority_fee: U256 = serde_json::from_value(priority_fee_result.unwrap()).unwrap();

    // 3. 估算 gas
    let estimate_request = CallRequest {
        from: Some(from_addr),
        to: Some(to_addr),
        gas: None,
        gas_price: None,
        value: Some(U256::from(1_000_000_000_000_000_000u64)),
        data: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };
    let estimate_params = serde_json::json!([estimate_request]);
    let gas_result = handler.eth_estimate_gas(estimate_params).await;
    assert!(gas_result.is_ok(), "估算 gas 应该成功");

    // 4. 发送 EIP-1559 交易
    let tx_request = SendTransactionRequest {
        from: from_addr,
        to: Some(to_addr),
        gas: Some(U256::from(21000)),
        gas_price: None,
        value: Some(U256::from(1_000_000_000_000_000_000u64)),
        data: None,
        nonce: Some(U256::zero()),
        max_fee_per_gas: Some(U256::from(50_000_000_000u64)),
        max_priority_fee_per_gas: Some(priority_fee),
    };
    let tx_params = serde_json::json!([tx_request]);
    let tx_result = handler.eth_send_transaction(tx_params).await;
    assert!(tx_result.is_ok(), "发送交易应该成功");

    // 5. 获取费用历史
    let fee_history_params = serde_json::json!([U64::from(1), "latest", None::<Vec<f64>>]);
    let fee_result = handler.eth_fee_history(fee_history_params).await;
    assert!(fee_result.is_ok(), "获取费用历史应该成功");
}

#[tokio::test]
async fn test_complete_contract_deployment_lifecycle() {
    let handler = create_test_handler();

    let from_addr = Address::from_low_u64_be(3001);
    let contract_bytecode = hex::decode("608060405234801561001057600080fd5b50").unwrap();

    // 1. 估算部署 gas
    let estimate_request = CallRequest {
        from: Some(from_addr),
        to: None, // 部署合约
        gas: None,
        gas_price: None,
        value: None,
        data: Some(contract_bytecode.clone()),
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };
    let estimate_params = serde_json::json!([estimate_request]);
    let gas_result = handler.eth_estimate_gas(estimate_params).await;
    assert!(gas_result.is_ok(), "估算部署 gas 应该成功");

    // 2. 获取建议费用
    let priority_fee_result = handler.eth_max_priority_fee_per_gas().await;
    assert!(priority_fee_result.is_ok());
    let priority_fee: U256 = serde_json::from_value(priority_fee_result.unwrap()).unwrap();

    // 3. 部署合约
    let deploy_request = SendTransactionRequest {
        from: from_addr,
        to: None,
        gas: Some(U256::from(200000)),
        gas_price: None,
        value: None,
        data: Some(contract_bytecode),
        nonce: Some(U256::zero()),
        max_fee_per_gas: Some(U256::from(40_000_000_000u64)),
        max_priority_fee_per_gas: Some(priority_fee),
    };
    let deploy_params = serde_json::json!([deploy_request]);
    let deploy_result = handler.eth_send_transaction(deploy_params).await;
    assert!(deploy_result.is_ok(), "部署合约应该成功");
}
