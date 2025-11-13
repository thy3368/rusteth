//! EthApiClient 使用示例
//!
//! 演示如何使用 EthApiClient 连接到远端以太坊 RPC 节点并调用各种方法。
//!
//! ## 运行示例
//!
//! ```bash
//! cargo run --example eth_api_client_usage
//! ```
//!
//! ## 环境变量
//!
//! 可以通过环境变量配置 RPC 端点:
//! ```bash
//! ETH_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY cargo run --example eth_api_client_usage
//! ```

use node::infrastructure::eth_api_client::EthApiClient;
use node::infrastructure::json_rpc_trait::EthJsonRpc;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 从环境变量或使用默认的公共端点(LlamaRPC - 无需API密钥)
    let rpc_url = env::var("ETH_RPC_URL")
        .unwrap_or_else(|_| "https://eth.llamarpc.com".to_string());

    println!("🔗 连接到以太坊节点: {}", rpc_url);
    println!();

    // 创建客户端
    let client = EthApiClient::new(rpc_url)?;

    // ========================================================================
    // 示例 1: 获取当前区块号
    // ========================================================================
    println!("📊 示例 1: 获取当前区块号");
    match client.eth_block_number().await {
        Ok(block_number) => {
            println!("   ✅ 当前区块号: {}", block_number);
        }
        Err(e) => {
            eprintln!("   ❌ 错误: {}", e);
        }
    }
    println!();

    // ========================================================================
    // 示例 2: 获取链 ID
    // ========================================================================
    println!("🔗 示例 2: 获取链 ID");
    match client.eth_chain_id().await {
        Ok(chain_id) => {
            println!("   ✅ 链 ID: {}", chain_id);
        }
        Err(e) => {
            eprintln!("   ❌ 错误: {}", e);
        }
    }
    println!();

    // ========================================================================
    // 示例 3: 获取 Gas 价格
    // ========================================================================
    println!("⛽ 示例 3: 获取当前 Gas 价格");
    match client.eth_gas_price().await {
        Ok(gas_price) => {
            println!("   ✅ Gas 价格: {}", gas_price);

            // 转换为 Gwei
            if let Some(hex_str) = gas_price.as_str() {
                if let Ok(wei) = u64::from_str_radix(hex_str.trim_start_matches("0x"), 16) {
                    let gwei = wei as f64 / 1_000_000_000.0;
                    println!("   ✅ Gas 价格: {:.2} Gwei", gwei);
                }
            }
        }
        Err(e) => {
            eprintln!("   ❌ 错误: {}", e);
        }
    }
    println!();

    // ========================================================================
    // 示例 4: 获取账户余额
    // ========================================================================
    println!("💰 示例 4: 获取账户余额");
    // Vitalik Buterin 的公开地址
    let address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
    let params = serde_json::json!([address, "latest"]);

    match client.eth_get_balance(params).await {
        Ok(balance) => {
            println!("   ✅ 地址: {}", address);
            println!("   ✅ 余额 (Wei): {}", balance);

            // 转换为 ETH
            if let Some(hex_str) = balance.as_str() {
                if let Ok(wei) = u128::from_str_radix(hex_str.trim_start_matches("0x"), 16) {
                    let eth = wei as f64 / 1_000_000_000_000_000_000.0;
                    println!("   ✅ 余额: {:.4} ETH", eth);
                }
            }
        }
        Err(e) => {
            eprintln!("   ❌ 错误: {}", e);
        }
    }
    println!();

    // ========================================================================
    // 示例 5: 获取区块信息
    // ========================================================================
    println!("🧱 示例 5: 获取最新区块信息");
    let params = serde_json::json!(["latest", false]); // false = 不包含完整交易

    match client.eth_get_block_by_number(params).await {
        Ok(block) => {
            println!("   ✅ 区块信息:");
            if let Some(obj) = block.as_object() {
                if let Some(number) = obj.get("number") {
                    println!("      区块号: {}", number);
                }
                if let Some(hash) = obj.get("hash") {
                    println!("      区块哈希: {}", hash);
                }
                if let Some(timestamp) = obj.get("timestamp") {
                    println!("      时间戳: {}", timestamp);
                }
                if let Some(tx_count) = obj.get("transactions") {
                    if let Some(txs) = tx_count.as_array() {
                        println!("      交易数量: {}", txs.len());
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("   ❌ 错误: {}", e);
        }
    }
    println!();

    // ========================================================================
    // 示例 6: 获取网络版本
    // ========================================================================
    println!("🌐 示例 6: 获取网络版本");
    match client.net_version().await {
        Ok(version) => {
            println!("   ✅ 网络版本: {}", version);
        }
        Err(e) => {
            eprintln!("   ❌ 错误: {}", e);
        }
    }
    println!();

    // ========================================================================
    // 示例 7: 获取客户端版本
    // ========================================================================
    println!("📦 示例 7: 获取客户端版本");
    match client.web3_client_version().await {
        Ok(version) => {
            println!("   ✅ 客户端版本: {}", version);
        }
        Err(e) => {
            eprintln!("   ❌ 错误: {}", e);
        }
    }
    println!();

    // ========================================================================
    // 性能测试: 并发请求
    // ========================================================================
    println!("⚡ 性能测试: 10 个并发 eth_blockNumber 请求");
    let start = tokio::time::Instant::now();

    let mut handles = vec![];
    for i in 0..10 {
        let client = EthApiClient::new("https://eth.llamarpc.com".to_string())?;
        let handle = tokio::spawn(async move {
            (i, client.eth_block_number().await)
        });
        handles.push(handle);
    }

    let mut success_count = 0;
    for handle in handles {
        if let Ok((id, result)) = handle.await {
            if result.is_ok() {
                success_count += 1;
                println!("   ✅ 请求 {} 完成", id);
            } else {
                println!("   ❌ 请求 {} 失败: {:?}", id, result.unwrap_err());
            }
        }
    }

    let duration = start.elapsed();
    println!();
    println!("   📈 统计:");
    println!("      成功请求: {}/10", success_count);
    println!("      总耗时: {:?}", duration);
    println!("      平均延迟: {:?}", duration / 10);
    println!();

    Ok(())
}
