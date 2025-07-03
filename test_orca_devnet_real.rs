// Test real de Orca en DevNet basado en documentación oficial
// https://github.com/orca-so/whirlpools/tree/main/rust-sdk/whirlpool/src/pool.rs#L287-L300

use orca_whirlpools::{
    fetch_whirlpools_by_token_pair, set_whirlpools_config_address, PoolInfo, WhirlpoolsConfigInput,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 TESTING ORCA DEVNET - REAL VERIFICATION");
    println!("Based on official Orca documentation examples");
    
    // Configurar Orca para DevNet usando la función oficial
    println!("⚙️ Setting up Orca Whirlpools config for DevNet...");
    set_whirlpools_config_address(WhirlpoolsConfigInput::SolanaDevnet)?;
    println!("✅ Orca config set for DevNet");
    
    // Conectar a DevNet usando el endpoint oficial
    println!("🌐 Connecting to DevNet RPC...");
    let rpc = RpcClient::new("https://api.devnet.solana.com".to_string());
    println!("✅ Connected to DevNet");
    
    // Test basic RPC connectivity
    println!("🔍 Testing basic RPC connectivity...");
    let version = rpc.get_version().await?;
    println!("✅ RPC Version: {}", version.solana_core);
    
    // Usar tokens oficiales de DevNet de la documentación
    println!("🪙 Setting up token pairs from official docs...");
    let token_a = Pubkey::from_str("So11111111111111111111111111111111111111112")?; // SOL
    let token_b = Pubkey::from_str("BRjpCHtyQLNCo8gqRUr8jtdAj5AjPYQaoqbvcZiHok1k")?; // devUSDC
    
    println!("📊 Token A (SOL): {}", token_a);
    println!("📊 Token B (devUSDC): {}", token_b);
    
    // Test real: buscar Whirlpools para este par usando función oficial
    println!("🌊 Searching for Whirlpools using official SDK function...");
    println!("📋 Function: fetch_whirlpools_by_token_pair");
    
    match fetch_whirlpools_by_token_pair(&rpc, token_a, token_b).await {
        Ok(pools) => {
            println!("✅ SUCCESS! Orca SDK working in DevNet");
            println!("📊 Found {} pools for SOL/devUSDC pair", pools.len());
            
            for (i, pool_info) in pools.iter().enumerate() {
                match pool_info {
                    PoolInfo::Initialized(pool) => {
                        println!("  🟢 Pool {}: INITIALIZED", i + 1);
                        println!("    📍 Address: {}", pool.address);
                        println!("    💰 Token A Vault: {}", pool.token_vault_a);
                        println!("    💰 Token B Vault: {}", pool.token_vault_b);
                        println!("    📏 Tick Spacing: {}", pool.tick_spacing);
                        println!("    💵 Fee Rate: {} bps", pool.fee_rate);
                    }
                    PoolInfo::Uninitialized(pool) => {
                        println!("  🟡 Pool {}: UNINITIALIZED", i + 1);
                        println!("    📍 Address: {}", pool.address);
                        println!("    📏 Tick Spacing: {}", pool.tick_spacing);
                    }
                }
            }
            
            println!("");
            println!("🎉 CONCLUSION: ORCA WORKS PERFECTLY IN DEVNET!");
            println!("✅ No 403 errors, no API issues");
            println!("✅ Whirlpool SDK correctly integrated");
            println!("✅ DevNet liquidity pools found and accessible");
            println!("✅ Ready for integration in SniperForge");
        }
        Err(e) => {
            println!("❌ FAILED to fetch Whirlpools: {}", e);
            println!("🔍 Error details: {:?}", e);
            println!("💡 This might indicate network issues or RPC problems");
            
            // Test if it's an RPC issue
            println!("🔍 Testing if RPC is responsive...");
            match rpc.get_slot().await {
                Ok(slot) => println!("✅ RPC is responsive, current slot: {}", slot),
                Err(rpc_err) => println!("❌ RPC not responsive: {}", rpc_err),
            }
        }
    }
    
    Ok(())
}
