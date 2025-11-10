//! REVM 使用示例
//!
//! 本示例展示如何使用 revm (Rust Ethereum Virtual Machine) 执行：
//! 1. 简单的以太坊价值转账
//! 2. 智能合约部署
//! 3. 智能合约调用
//! 4. eth_call 模拟（只读调用）
//! 5. 预编译合约调用
//! 6. 批量交易执行
//!
//! REVM 是一个高性能的 EVM 实现，用于模拟以太坊交易执行。

use alloy_primitives::{address, Address, Bytes, U256};
use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{AccountInfo, Bytecode, ExecutionResult, Output, TransactTo},
    Database, Evm,
};

/// 示例 1: 简单的以太坊价值转账
fn example_value_transfer() {
    println!("\n=== 示例 1: 以太坊价值转账 ===");

    // 1. 创建空数据库作为状态存储
    let mut cache_db = CacheDB::new(EmptyDB::default());

    // 2. 设置发送方账户状态
    let sender = address!("0000000000000000000000000000000000000001");
    let sender_balance = U256::from(1_000_000_000_000_000_000u64); // 1 ETH

    cache_db.insert_account_info(
        sender,
        AccountInfo {
            balance: sender_balance,
            nonce: 0,
            code_hash: Default::default(),
            code: None,
        },
    );

    // 3. 设置接收方地址
    let receiver = address!("0000000000000000000000000000000000000002");

    // 4. 构建 EVM 实例并执行交易
    let mut evm = Evm::builder()
        .with_db(&mut cache_db)
        .modify_tx_env(|tx| {
            tx.caller = sender;
            tx.transact_to = TransactTo::Call(receiver);
            tx.value = U256::from(500_000_000_000_000_000u64); // 转账 0.5 ETH
            tx.gas_limit = 21_000; // 标准转账 gas limit
            tx.gas_price = U256::from(20_000_000_000u64); // 20 Gwei
        })
        .build();

    // 5. 执行交易
    let result = evm.transact_commit();

    match result {
        Ok(ref execution_result) => {
            println!("✅ 交易执行成功!");
            println!("   Gas 使用量: {:?}", execution_result.gas_used());
        }
        Err(e) => {
            eprintln!("❌ 交易执行失败: {:?}", e);
        }
    }

    // 6. 释放 EVM 后查询账户余额变化
    drop(evm);

    if let Ok(sender_after) = cache_db.load_account(sender) {
        println!("   发送方余额: {} Wei", sender_after.info.balance);
    }
    if let Ok(receiver_after) = cache_db.load_account(receiver) {
        println!("   接收方余额: {} Wei", receiver_after.info.balance);
    }
}

/// 示例 2: 部署智能合约
fn example_contract_deployment() {
    println!("\n=== 示例 2: 智能合约部署 ===");

    // 1. 简单合约的创建字节码（部署时执行的代码）
    // 这段代码将运行时字节码复制到内存并返回
    let runtime_code = Bytes::from(vec![
        0x60, 0x2a, // PUSH1 42
        0x60, 0x00, // PUSH1 0
        0x52, // MSTORE
        0x60, 0x20, // PUSH1 32
        0x60, 0x00, // PUSH1 0
        0xf3, // RETURN
    ]);

    // 构造完整的部署字节码
    let mut deploy_code = Vec::new();
    // PUSH runtime_code.len()
    deploy_code.push(0x60);
    deploy_code.push(runtime_code.len() as u8);
    // PUSH 0 (offset in code where runtime code starts)
    deploy_code.extend_from_slice(&[0x60, 0x0c]); // 12 bytes offset
    // PUSH 0 (dest memory offset)
    deploy_code.extend_from_slice(&[0x60, 0x00]);
    // CODECOPY
    deploy_code.push(0x39);
    // PUSH runtime_code.len()
    deploy_code.push(0x60);
    deploy_code.push(runtime_code.len() as u8);
    // PUSH 0
    deploy_code.extend_from_slice(&[0x60, 0x00]);
    // RETURN
    deploy_code.push(0xf3);
    // Append runtime code
    deploy_code.extend_from_slice(&runtime_code);

    let contract_bytecode = Bytes::from(deploy_code);

    let mut cache_db = CacheDB::new(EmptyDB::default());

    // 2. 设置部署者账户
    let deployer = address!("1000000000000000000000000000000000000001");
    cache_db.insert_account_info(
        deployer,
        AccountInfo {
            balance: U256::from(10_000_000_000_000_000_000u64), // 10 ETH
            nonce: 0,
            code_hash: Default::default(),
            code: None,
        },
    );

    // 3. 构建 EVM 并执行部署交易
    let mut evm = Evm::builder()
        .with_db(&mut cache_db)
        .modify_tx_env(|tx| {
            tx.caller = deployer;
            tx.transact_to = TransactTo::Create; // 部署合约
            tx.data = contract_bytecode.clone();
            tx.gas_limit = 1_000_000;
            tx.gas_price = U256::from(20_000_000_000u64);
        })
        .build();

    // 4. 执行部署
    let result = evm.transact_commit();

    match result {
        Ok(ref execution_result) => {
            if let ExecutionResult::Success { ref output, .. } = execution_result {
                if let Output::Create(_, Some(contract_address)) = output {
                    println!("✅ 合约部署成功!");
                    println!("   合约地址: {:?}", contract_address);
                    println!("   Gas 使用量: {:?}", execution_result.gas_used());
                } else {
                    println!("⚠️  合约部署返回了意外的输出格式");
                }
            }
        }
        Err(e) => {
            eprintln!("❌ 合约部署失败: {:?}", e);
        }
    }
}

/// 示例 3: 调用智能合约
fn example_contract_call() {
    println!("\n=== 示例 3: 智能合约调用 ===");

    let mut cache_db = CacheDB::new(EmptyDB::default());

    // 1. 预先部署合约（简化示例，直接设置合约账户）
    let contract_address = address!("2000000000000000000000000000000000000001");

    // 简单的合约代码：返回固定值 42
    let simple_storage_code = Bytes::from(vec![
        0x60, 0x2a, // PUSH1 42
        0x60, 0x00, // PUSH1 0
        0x52, // MSTORE
        0x60, 0x20, // PUSH1 32
        0x60, 0x00, // PUSH1 0
        0xf3, // RETURN
    ]);

    cache_db.insert_account_info(
        contract_address,
        AccountInfo {
            balance: U256::ZERO,
            nonce: 1,
            code_hash: Default::default(),
            code: Some(Bytecode::new_raw(simple_storage_code)),
        },
    );

    // 2. 设置调用者账户
    let caller = address!("3000000000000000000000000000000000000001");
    cache_db.insert_account_info(
        caller,
        AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u64),
            nonce: 0,
            code_hash: Default::default(),
            code: None,
        },
    );

    // 3. 执行合约调用
    let mut evm = Evm::builder()
        .with_db(&mut cache_db)
        .modify_tx_env(|tx| {
            tx.caller = caller;
            tx.transact_to = TransactTo::Call(contract_address);
            tx.data = Bytes::new();
            tx.gas_limit = 100_000;
            tx.gas_price = U256::from(20_000_000_000u64);
        })
        .build();

    let result = evm.transact_commit();

    match result {
        Ok(ref execution_result) => {
            println!("✅ 合约调用成功!");
            println!("   Gas 使用量: {:?}", execution_result.gas_used());

            if let ExecutionResult::Success { ref output, .. } = execution_result {
                if let Output::Call(ref return_data) = output {
                    println!("   返回数据长度: {} bytes", return_data.len());
                    if !return_data.is_empty() {
                        if return_data.len() == 32 {
                            let value = U256::from_be_slice(&return_data);
                            println!("   返回值: {}", value);
                        }
                        println!(
                            "   返回数据(hex): {}",
                            alloy_primitives::hex::encode(&return_data)
                        );
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ 合约调用失败: {:?}", e);
        }
    }
}

/// 示例 4: eth_call 模拟（只读调用，不修改状态）
fn example_eth_call_simulation() {
    println!("\n=== 示例 4: eth_call 模拟（只读调用）===");

    let mut cache_db = CacheDB::new(EmptyDB::default());

    // 1. 设置目标合约
    let contract = address!("4000000000000000000000000000000000000001");

    // 模拟一个返回固定值的合约：返回 66 (0x42)
    let view_function_code = Bytes::from(vec![
        0x60, 0x42, // PUSH1 66 (0x42)
        0x60, 0x00, // PUSH1 0
        0x52, // MSTORE
        0x60, 0x20, // PUSH1 32
        0x60, 0x00, // PUSH1 0
        0xf3, // RETURN
    ]);

    cache_db.insert_account_info(
        contract,
        AccountInfo {
            balance: U256::ZERO,
            nonce: 1,
            code_hash: Default::default(),
            code: Some(Bytecode::new_raw(view_function_code)),
        },
    );

    // 2. 设置调用者（eth_call 不需要真实账户余额）
    let caller = address!("0000000000000000000000000000000000000000");

    // 3. 执行只读调用（使用 transact 而非 transact_commit）
    let mut evm = Evm::builder()
        .with_db(&mut cache_db)
        .modify_tx_env(|tx| {
            tx.caller = caller;
            tx.transact_to = TransactTo::Call(contract);
            tx.data = Bytes::new();
            tx.gas_limit = 100_000;
        })
        .build();

    let result = evm.transact();

    match result {
        Ok(result_and_state) => {
            println!("✅ eth_call 模拟成功!");

            if let ExecutionResult::Success {
                ref output,
                gas_used,
                ..
            } = result_and_state.result
            {
                println!("   模拟 Gas 使用量: {}", gas_used);

                if let Output::Call(ref return_data) = output {
                    if return_data.len() == 32 {
                        let value = U256::from_be_slice(&return_data);
                        println!("   返回值: {}", value);
                    } else {
                        println!(
                            "   返回数据: {}",
                            alloy_primitives::hex::encode(&return_data)
                        );
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ eth_call 模拟失败: {:?}", e);
        }
    }
}

/// 示例 5: 预编译合约调用 (ecrecover)
fn example_precompile_call() {
    println!("\n=== 示例 5: 预编译合约调用 (ecrecover) ===");

    let mut cache_db = CacheDB::new(EmptyDB::default());

    // ecrecover 预编译合约地址: 0x01
    let ecrecover_address = address!("0000000000000000000000000000000000000001");

    let caller = address!("5000000000000000000000000000000000000001");
    cache_db.insert_account_info(
        caller,
        AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u64),
            nonce: 0,
            code_hash: Default::default(),
            code: None,
        },
    );

    // ecrecover 输入格式: hash(32) + v(32) + r(32) + s(32)
    let mut call_data = Vec::new();
    call_data.extend_from_slice(&[0u8; 32]); // hash
    call_data.extend_from_slice(&[0u8; 31]);
    call_data.push(27); // v = 27
    call_data.extend_from_slice(&[0u8; 32]); // r
    call_data.extend_from_slice(&[0u8; 32]); // s

    let mut evm = Evm::builder()
        .with_db(&mut cache_db)
        .modify_tx_env(|tx| {
            tx.caller = caller;
            tx.transact_to = TransactTo::Call(ecrecover_address);
            tx.data = Bytes::from(call_data);
            tx.gas_limit = 100_000;
        })
        .build();

    let result = evm.transact();

    match result {
        Ok(result_and_state) => {
            println!("✅ 预编译合约调用完成!");

            if let ExecutionResult::Success {
                ref output,
                gas_used,
                ..
            } = result_and_state.result
            {
                println!("   Gas 使用量: {}", gas_used);

                if let Output::Call(ref return_data) = output {
                    if return_data.len() >= 20 {
                        let addr_bytes = if return_data.len() == 32 {
                            &return_data[12..]
                        } else {
                            &return_data[..]
                        };
                        println!(
                            "   恢复的地址: {}",
                            alloy_primitives::hex::encode(addr_bytes)
                        );
                    } else {
                        println!("   返回数据长度: {} bytes", return_data.len());
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ 预编译合约调用失败: {:?}", e);
        }
    }
}

/// 示例 6: 批量交易执行
fn example_batch_transactions() {
    println!("\n=== 示例 6: 批量交易执行 ===");

    let mut cache_db = CacheDB::new(EmptyDB::default());

    // 设置初始账户
    let accounts: Vec<Address> = (0..10)
        .map(|i| Address::from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, i as u8]))
        .collect();

    for account in &accounts {
        cache_db.insert_account_info(
            *account,
            AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64), // 1 ETH each
                nonce: 0,
                code_hash: Default::default(),
                code: None,
            },
        );
    }

    // 批量执行转账
    let mut total_gas = 0u64;
    let start = std::time::Instant::now();

    for i in 0..accounts.len() - 1 {
        let sender = accounts[i];
        let receiver = accounts[i + 1];

        let mut evm = Evm::builder()
            .with_db(&mut cache_db)
            .modify_tx_env(|tx| {
                tx.caller = sender;
                tx.transact_to = TransactTo::Call(receiver);
                tx.value = U256::from(100_000_000_000_000_000u64); // 0.1 ETH
                tx.gas_limit = 21_000;
                tx.gas_price = U256::from(20_000_000_000u64);
            })
            .build();

        if let Ok(ref result) = evm.transact_commit() {
            total_gas += result.gas_used();
        }
    }

    let duration = start.elapsed();

    println!("✅ 批量交易执行完成!");
    println!("   交易数量: {}", accounts.len() - 1);
    println!("   总 Gas 消耗: {}", total_gas);
    println!("   执行时间: {:?}", duration);
    println!(
        "   平均每笔交易: {:?}",
        duration / (accounts.len() as u32 - 1)
    );
}

fn main() {
    println!("╔════════════════════════════════════════════════════╗");
    println!("║     REVM (Rust Ethereum Virtual Machine) 示例      ║");
    println!("║        高性能 EVM 执行引擎演示                      ║");
    println!("╚════════════════════════════════════════════════════╝");

    // 执行所有示例
    example_value_transfer();
    example_contract_deployment();
    example_contract_call();
    example_eth_call_simulation();
    example_precompile_call();
    example_batch_transactions();

    println!("\n╔════════════════════════════════════════════════════╗");
    println!("║                 所有示例执行完成                    ║");
    println!("╚════════════════════════════════════════════════════╝");
    println!("\n📖 REVM 核心概念总结:");
    println!("   1. CacheDB: 内存数据库，用于状态存储");
    println!("   2. TransactTo::Create: 合约部署");
    println!("   3. TransactTo::Call: 合约调用");
    println!("   4. transact_commit(): 提交状态变更");
    println!("   5. transact(): 只读模拟（eth_call）");
    println!("   6. 预编译合约: 地址 0x01-0x09");
    println!("\n🚀 性能特性:");
    println!("   - 零拷贝设计");
    println!("   - 缓存行对齐的数据结构");
    println!("   - LTO 链接时优化");
    println!("   - 高效的批量交易执行");
}
