use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Checking wallet balance...");
    
    // Connect to RPC
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url);
    
    // Our wallet address
    let wallet_address = "JDzF9LkpoQVac6c6ufHW1c6Gevd3vFB4RXSSjceL8Kq7";
    let pubkey = Pubkey::from_str(wallet_address)?;
    
    // Get balance
    let balance = client.get_balance(&pubkey)?;
    let sol_balance = balance as f64 / 1_000_000_000.0;
    
    println!("💰 Current balance: {} SOL ({} lamports)", sol_balance, balance);
    
    Ok(())
}
