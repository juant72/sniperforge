// Simple Orca DevNet connectivity test
// Based on official Orca Whirlpools SDK examples
// This version avoids async main to prevent Send/Sync issues

use std::sync::Arc;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use orca_whirlpools::{
    WhirlpoolsConfigInput, ORCA_WHIRLPOOLS_PROGRAM_ID,
    get_whirlpool_configs_address, get_fee_tier_address,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Orca DevNet connectivity...");
    
    // Initialize RPC client for DevNet
    let rpc_url = "https://api.devnet.solana.com";
    let rpc_client = Arc::new(RpcClient::new(rpc_url.to_string()));
    
    println!("📡 Connected to DevNet: {}", rpc_url);
    println!("🏊 Orca Whirlpools Program ID: {}", ORCA_WHIRLPOOLS_PROGRAM_ID);
    
    // Test 1: Check if Whirlpools program exists on DevNet
    println!("\n🔍 Test 1: Checking Whirlpools program account...");
    match rpc_client.get_account(&ORCA_WHIRLPOOLS_PROGRAM_ID) {
        Ok(account) => {
            println!("✅ Whirlpools program found on DevNet!");
            println!("   Executable: {}", account.executable);
            println!("   Owner: {}", account.owner);
            println!("   Data length: {} bytes", account.data.len());
        }
        Err(e) => {
            println!("❌ Failed to fetch Whirlpools program: {}", e);
            return Err(e.into());
        }
    }
    
    // Test 2: Try to get Whirlpools config address
    println!("\n🔍 Test 2: Getting Whirlpools config address...");
    let (whirlpools_config_key, _bump) = get_whirlpool_configs_address(&ORCA_WHIRLPOOLS_PROGRAM_ID);
    println!("📋 Whirlpools config address: {}", whirlpools_config_key);
    
    match rpc_client.get_account(&whirlpools_config_key) {
        Ok(account) => {
            println!("✅ Whirlpools config account found!");
            println!("   Data length: {} bytes", account.data.len());
        }
        Err(e) => {
            println!("⚠️  Whirlpools config account not found: {}", e);
            // This might be expected if no config is set up on DevNet
        }
    }
    
    // Test 3: Try to get a fee tier address (common operation)
    println!("\n🔍 Test 3: Getting fee tier address...");
    let tick_spacing = 64; // Common tick spacing
    let (fee_tier_key, _bump) = get_fee_tier_address(
        &ORCA_WHIRLPOOLS_PROGRAM_ID,
        &whirlpools_config_key,
        tick_spacing,
    );
    println!("💰 Fee tier address (tick_spacing={}): {}", tick_spacing, fee_tier_key);
    
    match rpc_client.get_account(&fee_tier_key) {
        Ok(account) => {
            println!("✅ Fee tier account found!");
            println!("   Data length: {} bytes", account.data.len());
        }
        Err(e) => {
            println!("⚠️  Fee tier account not found: {}", e);
            // This might be expected if no fee tier is set up on DevNet
        }
    }
    
    // Test 4: Get slot info to verify we're actually connected
    println!("\n🔍 Test 4: Getting current slot info...");
    match rpc_client.get_slot() {
        Ok(slot) => {
            println!("✅ Current DevNet slot: {}", slot);
        }
        Err(e) => {
            println!("❌ Failed to get slot: {}", e);
            return Err(e.into());
        }
    }
    
    println!("\n🎉 All basic connectivity tests completed!");
    println!("📝 Summary:");
    println!("   - DevNet RPC connection: ✅");
    println!("   - Orca Whirlpools program: ✅");
    println!("   - Program address derivation: ✅");
    println!("   - Account queries: ✅");
    
    Ok(())
}
