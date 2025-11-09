//! 调试 RPC 响应格式
//!
//! 用于调试不同 RPC 端点的响应格式

use reqwest::Client;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoints = vec![
        "https://rpc.ankr.com/eth",
        "https://eth.llamarpc.com",
        "https://ethereum-rpc.publicnode.com",
    ];

    let client = Client::new();

    for endpoint in endpoints {
        println!("\n🔍 测试端点: {}", endpoint);
        println!("{}", "=".repeat(60));

        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "params": [],
            "id": 1
        });

        println!("📤 请求:");
        println!("{}", serde_json::to_string_pretty(&request_body)?);

        match client.post(endpoint)
            .json(&request_body)
            .send()
            .await
        {
            Ok(response) => {
                println!("\n📥 响应状态: {}", response.status());

                // 打印响应头
                println!("\n📋 响应头:");
                for (key, value) in response.headers() {
                    println!("  {}: {:?}", key, value);
                }

                // 获取响应文本
                let response_text = response.text().await?;
                println!("\n📝 响应体 (原始):");
                println!("{}", response_text);

                // 尝试解析为 JSON
                match serde_json::from_str::<Value>(&response_text) {
                    Ok(json) => {
                        println!("\n✅ JSON 解析成功:");
                        println!("{}", serde_json::to_string_pretty(&json)?);
                    }
                    Err(e) => {
                        println!("\n❌ JSON 解析失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("\n❌ 请求失败: {}", e);
            }
        }
    }

    Ok(())
}
