use anyhow::Result;
use tracing::{info, error};
use sniperforge::Config;
use sniperforge::shared::rpc_pool::RpcConnectionPool;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Test all major RPC methods to verify 100% functionality
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🧪 Testing ALL RPC Methods - Comprehensive Verification");
    println!("=========================================================");

    // Test both networks
    for network in ["devnet", "mainnet"] {
        println!("\n🌐 Testing Network: {}", network);
        println!("{}", "=".repeat(50));
        
        let config_file = match network {
            "mainnet" => "config/mainnet.toml",
            "devnet" => "config/devnet.toml",
            _ => continue,
        };

        let config = Config::load(config_file)?;
        let rpc_pool = RpcConnectionPool::new(&config).await?;
        rpc_pool.start().await?;

        let mut test_count = 0;
        let mut passed_count = 0;

        // Test 1: Basic connectivity
        test_count += 1;
        print!("📡 Test 1: getSlot... ");
        match rpc_pool.get_current_slot().await {
            Ok(slot) => {
                println!("✅ Current slot: {}", slot);
                passed_count += 1;
            }
            Err(e) => {
                println!("❌ Failed: {}", e);
            }
        }

        // Test 2: Latest blockhash
        test_count += 1;
        print!("📡 Test 2: getLatestBlockhash... ");
        match rpc_pool.get_latest_blockhash().await {
            Ok(blockhash) => {
                println!("✅ Blockhash: {}", blockhash);
                passed_count += 1;
            }
            Err(e) => {
                println!("❌ Failed: {}", e);
            }
        }

        // Test 3: Account info (SOL native mint)
        test_count += 1;
        print!("📡 Test 3: getAccountInfo... ");
        let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
        match rpc_pool.get_account_info(&sol_mint).await {
            Ok(Some(account)) => {
                println!("✅ SOL mint account found ({} bytes)", account.data.len());
                passed_count += 1;
            }
            Ok(None) => {
                println!("⚠️ Account not found (unexpected)");
            }
            Err(e) => {
                println!("❌ Failed: {}", e);
            }
        }

        // Test 4: Balance check (random account)
        test_count += 1;
        print!("📡 Test 4: getBalance... ");
        let test_pubkey = Pubkey::from_str("11111111111111111111111111111111")?;
        match rpc_pool.get_balance(&test_pubkey).await {
            Ok(balance) => {
                println!("✅ Balance: {} lamports", balance);
                passed_count += 1;
            }
            Err(e) => {
                println!("❌ Failed: {}", e);
            }
        }

        // Summary for this network
        println!("\n📊 Network {} Summary:", network);
        println!("   Tests: {}/{} passed", passed_count, test_count);
        println!("   Success rate: {:.1}%", (passed_count as f64 / test_count as f64) * 100.0);
        
        if passed_count == test_count {
            println!("   🎉 ALL TESTS PASSED!");
        } else {
            println!("   ⚠️ Some tests failed");
        }

        rpc_pool.stop().await?;
    }

    println!("\n🎯 Comprehensive RPC Testing Complete!");
    Ok(())
}
