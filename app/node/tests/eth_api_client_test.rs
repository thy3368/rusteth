//! EthApiClient 集成测试
//!
//! 注意: 这些测试需要真实的以太坊 RPC 端点才能运行。
//! 默认使用公共的以太坊测试网络。

#[cfg(test)]
mod tests {
    use node::infrastructure::eth_api_client::EthApiClient;
    use node::infrastructure::json_rpc_trait::EthApiExecutor;

    /// 测试客户端创建
    #[tokio::test]
    async fn test_client_creation() {
        // 使用 LlamaRPC 公共端点(无需 API 密钥)
        let rpc_url = "https://eth.llamarpc.com".to_string();
        let client = EthApiClient::new(rpc_url);

        assert!(client.is_ok(), "客户端创建应该成功");
    }

    /// 测试 eth_blockNumber
    ///
    /// 注意: 此测试需要网络访问,默认禁用
    #[tokio::test]
    #[ignore]
    async fn test_eth_block_number() {
        // 使用 LlamaRPC 公共节点 - 无需API密钥
        let rpc_url = "https://eth.llamarpc.com".to_string();
        let client = EthApiClient::new(rpc_url).expect("客户端创建失败");

        let result = client.eth_block_number().await;

        // 打印详细错误信息以便调试
        if let Err(ref e) = result {
            eprintln!("❌ 错误详情: {:?}", e);
        }

        assert!(result.is_ok(), "eth_blockNumber 应该成功: {:?}", result);

        if let Ok(block_number) = result {
            println!("✅ 当前区块号: {}", block_number);
            // 区块号应该是一个十六进制字符串
            assert!(block_number.is_string());
        }
    }

    /// 测试 eth_chainId
    ///
    /// 注意: 此测试需要网络访问,默认禁用
    #[tokio::test]
    #[ignore]
    async fn test_eth_chain_id() {
        let rpc_url = "https://eth.llamarpc.com".to_string();
        let client = EthApiClient::new(rpc_url).expect("客户端创建失败");

        let result = client.eth_chain_id().await;

        assert!(result.is_ok(), "eth_chainId 应该成功: {:?}", result);

        if let Ok(chain_id) = result {
            println!("✅ 链 ID: {}", chain_id);
            // 以太坊主网的链 ID 是 0x1
            assert_eq!(chain_id, serde_json::json!("0x1"));
        }
    }

    /// 测试 eth_getBalance
    ///
    /// 注意: 此测试需要网络访问,默认禁用
    #[tokio::test]
    #[ignore]
    async fn test_eth_get_balance() {
        let rpc_url = "https://eth.llamarpc.com".to_string();
        let client = EthApiClient::new(rpc_url).expect("客户端创建失败");

        // Vitalik Buterin 的公开地址
        let address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
        let params = serde_json::json!([address, "latest"]);

        let result = client.eth_get_balance(params).await;

        assert!(result.is_ok(), "eth_getBalance 应该成功: {:?}", result);

        if let Ok(balance) = result {
            println!("✅ 地址: {}", address);
            println!("   余额 (十六进制): {}", balance);

            // 转换为可读格式
            if let Some(hex_str) = balance.as_str() {
                if let Ok(wei) = u128::from_str_radix(hex_str.trim_start_matches("0x"), 16) {
                    // 转换为 ETH (1 ETH = 10^18 Wei)
                    let eth = wei as f64 / 1_000_000_000_000_000_000.0;

                    println!("   余额 (Wei):      {:>20}", format_number(wei));
                    println!("   余额 (ETH):      {:>20.6} ETH", eth);

                    // 估算 USD 价值 (假设 ETH = $3,000)
                    let eth_price = 3000.0;
                    let usd_value = eth * eth_price;
                    println!("   估值 (USD):      {:>20} (假设 ETH ≈ ${})",
                        format!("${:.2}", usd_value),
                        format_number(eth_price as u128)
                    );
                }
            }
        }
    }

    /// 格式化数字，添加千位分隔符
    fn format_number<T: std::fmt::Display>(n: T) -> String {
        let s = n.to_string();
        let bytes: Vec<_> = s.bytes().rev().collect();
        let chunks: Vec<_> = bytes
            .chunks(3)
            .map(|chunk| chunk.iter().rev().map(|&b| b as char).collect::<String>())
            .collect();
        chunks.iter().rev().map(|s| s.as_str()).collect::<Vec<_>>().join(",")
    }

    /// 测试错误处理: 无效的 RPC URL
    #[tokio::test]
    async fn test_invalid_rpc_url() {
        let rpc_url = "http://invalid-url-that-does-not-exist-12345.com".to_string();
        let client = EthApiClient::new(rpc_url).expect("客户端创建应该成功");

        // 请求应该失败
        let result = client.eth_block_number().await;
        assert!(result.is_err(), "无效的 RPC URL 应该返回错误");
        println!("✅ 正确处理了无效 URL 错误");
    }

    /// 测试多个并发请求(性能测试)
    ///
    /// 注意: 此测试需要网络访问,默认禁用
    #[tokio::test]
    #[ignore]
    async fn test_concurrent_requests() {
        use tokio::time::Instant;

        let rpc_url = "https://eth.llamarpc.com".to_string();
        let _client = EthApiClient::new(rpc_url).expect("客户端创建失败");

        let start = Instant::now();

        // 发送 10 个并发请求
        let mut handles = vec![];
        for i in 0..10 {
            let client_clone = EthApiClient::new("https://eth.llamarpc.com".to_string())
                .expect("客户端创建失败");

            let handle = tokio::spawn(async move {
                (i, client_clone.eth_block_number().await)
            });

            handles.push(handle);
        }

        // 等待所有请求完成
        let mut success_count = 0;
        for handle in handles {
            let (id, result) = handle.await.expect("任务应该成功完成");
            if result.is_ok() {
                success_count += 1;
                println!("✅ 请求 {} 成功", id);
            } else {
                println!("❌ 请求 {} 失败: {:?}", id, result);
            }
        }

        let duration = start.elapsed();
        println!("📊 并发测试统计:");
        println!("   成功请求: {}/10", success_count);
        println!("   总耗时: {:?}", duration);
        println!("   平均延迟: {:?}", duration / 10);

        // 至少 8/10 请求应该成功(允许一些失败)
        assert!(success_count >= 8, "至少 80% 的请求应该成功");
    }
}
