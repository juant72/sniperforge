use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The wallet address from Solscan
    let wallet_address = "9pMAkWBFY8EWW4DisQDbeLBi5xTcFwh62X3E8guK26zD";
    let pubkey = Pubkey::from_str(wallet_address)?;
    
    // Connect to Mainnet
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
    
    println!("🔍 Checking wallet balance for: {}", wallet_address);
    
    match rpc_client.get_balance(&pubkey) {
        Ok(balance_lamports) => {
            let balance_sol = balance_lamports as f64 / 1_000_000_000.0;
            println!("💰 Current balance: {} SOL ({} lamports)", balance_sol, balance_lamports);
            
            if balance_sol == 0.0 {
                println!("🚨 CONFIRMED: Wallet has ZERO balance!");
            } else if balance_sol < 0.01 {
                println!("⚠️  WARNING: Very low balance!");
            } else {
                println!("✅ Wallet has funds remaining");
            }
        }
        Err(e) => {
            println!("❌ Error checking balance: {}", e);
        }
    }
    
    // Also check recent transaction signatures
    println!("\n🔍 Fetching recent transaction signatures...");
    match rpc_client.get_signatures_for_address(&pubkey) {
        Ok(signatures) => {
            println!("📋 Found {} recent transactions:", signatures.len());
            for (i, sig_info) in signatures.iter().take(10).enumerate() {
                println!("{}. {} (slot: {})", 
                    i + 1, 
                    sig_info.signature,
                    sig_info.slot.unwrap_or(0)
                );
                if let Some(err) = &sig_info.err {
                    println!("   ❌ Error: {:?}", err);
                }
            }
        }
        Err(e) => {
            println!("❌ Error fetching transactions: {}", e);
        }
    }
    
    Ok(())
}
