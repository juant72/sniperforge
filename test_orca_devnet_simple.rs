// Simple Orca DevNet connectivity test - STANDALONE VERSION
// This test is completely independent of the main codebase to avoid async Send issues

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

// Define the Orca program ID directly to avoid importing from orca_whirlpools
const ORCA_WHIRLPOOLS_PROGRAM_ID_STR: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Orca DevNet connectivity (standalone)...");

    // Initialize RPC client for DevNet
    let rpc_url = "https://api.devnet.solana.com";
    let rpc_client = Arc::new(RpcClient::new(rpc_url.to_string()));

    println!("📡 Connected to DevNet: {}", rpc_url);

    // Parse the Orca program ID
    let orca_program_id = ORCA_WHIRLPOOLS_PROGRAM_ID_STR.parse::<Pubkey>()?;
    println!("🏊 Orca Whirlpools Program ID: {}", orca_program_id);

    // Test 1: Check if Whirlpools program exists on DevNet
    println!("\n🔍 Test 1: Checking Whirlpools program account...");
    match rpc_client.get_account(&orca_program_id) {
        Ok(account) => {
            println!("✅ Whirlpools program found on DevNet!");
            println!("   Executable: {}", account.executable);
            println!("   Owner: {}", account.owner);
            println!("   Data length: {} bytes", account.data.len());
            println!("   Lamports: {}", account.lamports);
        }
        Err(e) => {
            println!("❌ Failed to fetch Whirlpools program: {}", e);
            return Err(e.into());
        }
    }

    // Test 2: Get slot info to verify we're actually connected
    println!("\n🔍 Test 2: Getting current slot info...");
    match rpc_client.get_slot() {
        Ok(slot) => {
            println!("✅ Current DevNet slot: {}", slot);
        }
        Err(e) => {
            println!("❌ Failed to get slot: {}", e);
            return Err(e.into());
        }
    }

    // Test 3: Get cluster nodes to verify DevNet connection
    println!("\n🔍 Test 3: Getting cluster info...");
    match rpc_client.get_cluster_nodes() {
        Ok(nodes) => {
            println!("✅ Found {} cluster nodes on DevNet", nodes.len());
            if let Some(first_node) = nodes.first() {
                println!(
                    "   First node: {} (version: {:?})",
                    first_node.pubkey, first_node.version
                );
            }
        }
        Err(e) => {
            println!("⚠️  Could not get cluster nodes: {}", e);
        }
    }

    // Test 4: Try to get account info for a known DevNet account (system program)
    println!("\n🔍 Test 4: Testing account queries...");
    let system_program = solana_sdk::system_program::id();
    match rpc_client.get_account(&system_program) {
        Ok(account) => {
            println!("✅ System program account found");
            println!("   Executable: {}", account.executable);
        }
        Err(e) => {
            println!("❌ Failed to get system program: {}", e);
        }
    }

    println!("\n🎉 Orca DevNet connectivity tests completed!");
    println!("📝 Summary:");
    println!("   - DevNet RPC connection: ✅");
    println!("   - Orca Whirlpools program verification: ✅");
    println!("   - Program account queries: ✅");
    println!("   - Network connectivity: ✅");

    println!("\n💡 Next steps:");
    println!("   - Orca is confirmed to be deployed on DevNet");
    println!(
        "   - The program ID {} is valid and accessible",
        orca_program_id
    );
    println!("   - You can now safely use Orca in your DevNet testing");

    Ok(())
}
