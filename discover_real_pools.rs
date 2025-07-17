use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
};
use std::str::FromStr;
use tokio;

// Raydium AMM Program ID
const RAYDIUM_AMM_PROGRAM: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

// Orca Program IDs
const ORCA_SWAP_PROGRAM: &str = "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP";
const ORCA_WHIRLPOOL_PROGRAM: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";

// Known active pools (these are real pools that should exist)
const KNOWN_POOLS: &[(&str, &str)] = &[
    // Raydium SOL-USDC
    ("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2", "Raydium SOL-USDC"),
    ("7XawhbbxtsRcQA8KTkHT9f9nc6d69UwqCDh6U5EEbEmX", "Raydium SOL-USDT"),
    
    // Major Orca pools
    ("EGZ7tiLeH62TPV1gL8WwbXGzEPa9zmcpVnnkPKKnrE2U", "Orca SOL-USDC"),
    ("7qbRF6YsyGuLUVs6Y1q64bdVrfe4ZcUUz1JRdoVNUJnm", "Orca SOL-USDT"),
    
    // Try some other major pools
    ("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1", "Raydium RAY-SOL"),
    ("6UmmUiYoBjSrhakAobJw8BvkmJtDVxaeBtbt7rxWo1mg", "Raydium RAY-USDC"),
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Discovering real active pools...");
    
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    
    println!("📡 Connected to Solana RPC");
    
    for (pool_address, description) in KNOWN_POOLS {
        let pubkey = match Pubkey::from_str(pool_address) {
            Ok(pk) => pk,
            Err(_) => {
                println!("❌ Invalid pubkey: {}", pool_address);
                continue;
            }
        };
        
        print!("🔬 Checking {}: ", description);
        
        match client.get_account(&pubkey) {
            Ok(account) => {
                println!("✅ EXISTS - Size: {} bytes, Owner: {}", 
                    account.data.len(), 
                    account.owner);
                
                // Check if it's a known program
                let owner_str = account.owner.to_string();
                if owner_str == RAYDIUM_AMM_PROGRAM {
                    println!("   🔥 Raydium AMM Pool");
                } else if owner_str == ORCA_SWAP_PROGRAM {
                    println!("   🐋 Orca Swap Pool");
                } else if owner_str == ORCA_WHIRLPOOL_PROGRAM {
                    println!("   🌊 Orca Whirlpool Pool");
                } else {
                    println!("   ❓ Unknown program: {}", owner_str);
                }
            }
            Err(e) => {
                println!("❌ NOT FOUND: {}", e);
            }
        }
    }
    
    println!("\n🔍 Searching for Raydium pools...");
    
    // Search for accounts owned by Raydium AMM program
    let raydium_program = Pubkey::from_str(RAYDIUM_AMM_PROGRAM)?;
    
    match client.get_program_accounts(&raydium_program) {
        Ok(accounts) => {
            println!("✅ Found {} Raydium accounts", accounts.len());
            
            // Show first 10
            for (i, (pubkey, account)) in accounts.iter().take(10).enumerate() {
                println!("  {}. {} - {} bytes", i + 1, pubkey, account.data.len());
            }
        }
        Err(e) => {
            println!("❌ Failed to get Raydium accounts: {}", e);
        }
    }
    
    println!("\n🔍 Searching for Orca pools...");
    
    // Search for Orca accounts
    let orca_program = Pubkey::from_str(ORCA_SWAP_PROGRAM)?;
    
    match client.get_program_accounts(&orca_program) {
        Ok(accounts) => {
            println!("✅ Found {} Orca swap accounts", accounts.len());
            
            // Show first 10
            for (i, (pubkey, account)) in accounts.iter().take(10).enumerate() {
                println!("  {}. {} - {} bytes", i + 1, pubkey, account.data.len());
            }
        }
        Err(e) => {
            println!("❌ Failed to get Orca accounts: {}", e);
        }
    }
    
    Ok(())
}
