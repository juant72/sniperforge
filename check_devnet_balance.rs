use anyhow::Result;
use dotenv::dotenv;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::Keypair,
};
use spl_token::state::Account as TokenAccount;
use std::str::FromStr;
use std::env;
use std::fs;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    
    println!("🔍 Verificando balance de wallet en DevNet...");
    
    // Cargar configuración
    let config_content = fs::read_to_string("config/devnet-automated.json")?;
    let config: Value = serde_json::from_str(&config_content)?;
    
    let rpc_url = config["cluster_url"].as_str().unwrap();
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    
    // Cargar wallet
    let wallet_path = env::var("SOLANA_WALLET_PATH")
        .unwrap_or_else(|_| "wallet/devnet-wallet.json".to_string());
    
    let wallet_bytes = fs::read(&wallet_path)?;
    let keypair = Keypair::from_bytes(&wallet_bytes)?;
    let wallet_pubkey = keypair.pubkey();
    
    println!("📍 Wallet address: {}", wallet_pubkey);
    
    // Verificar balance de SOL
    let sol_balance = client.get_balance(&wallet_pubkey)?;
    println!("💰 SOL Balance: {} SOL", sol_balance as f64 / 1_000_000_000.0);
    
    // Verificar balances de tokens SPL
    let tokens = config["tokens"].as_object().unwrap();
    
    for (symbol, token_info) in tokens {
        if symbol == "SOL" {
            continue;
        }
        
        let mint_str = token_info["mint"].as_str().unwrap();
        let mint_pubkey = Pubkey::from_str(mint_str)?;
        let decimals = token_info["decimals"].as_u64().unwrap() as u8;
        
        // Buscar cuenta de token asociada
        let token_accounts = client.get_token_accounts_by_owner(
            &wallet_pubkey,
            solana_client::rpc_request::TokenAccountsFilter::Mint(mint_pubkey),
        )?;
        
        if !token_accounts.is_empty() {
            for account in token_accounts {
                let account_data = account.account.data.decode().unwrap();
                let token_account = TokenAccount::unpack(&account_data)?;
                let balance = token_account.amount as f64 / 10_f64.powi(decimals as i32);
                println!("💰 {} Balance: {} {}", symbol, balance, symbol);
            }
        } else {
            println!("💰 {} Balance: 0 {} (no account)", symbol, symbol);
        }
    }
    
    println!("\n✅ Verificación de balance completada");
    Ok(())
}
