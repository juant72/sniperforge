use anyhow::Result;
use dotenv::dotenv;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    native_token::LAMPORTS_PER_SOL,
};
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
    
    // Cargar wallet desde variable de entorno
    let wallet_keypair = load_wallet_from_env()?;
    let wallet_pubkey = wallet_keypair.pubkey();
    
    println!("📍 Wallet address: {}", wallet_pubkey);
    
    // Verificar balance de SOL
    let sol_balance = client.get_balance(&wallet_pubkey)?;
    let sol_ui = sol_balance as f64 / LAMPORTS_PER_SOL as f64;
    println!("💰 SOL Balance: {:.9} SOL", sol_ui);
    
    // Verificar balances de tokens SPL usando RPC calls simplificados
    let tokens = config["tokens"].as_object().unwrap();
    
    for (symbol, token_info) in tokens {
        if symbol == "SOL" {
            continue;
        }
        
        let mint_str = token_info["mint"].as_str().unwrap();
        let mint_pubkey = Pubkey::from_str(mint_str)?;
        let decimals = token_info["decimals"].as_u64().unwrap() as u8;
        
        // Usar get_token_accounts_by_owner para obtener cuentas de tokens
        match client.get_token_accounts_by_owner(
            &wallet_pubkey,
            solana_client::rpc_request::TokenAccountsFilter::Mint(mint_pubkey),
        ) {
            Ok(token_accounts) => {
                if !token_accounts.is_empty() {
                    for account in token_accounts {
                        // Usar get_token_account_balance para obtener el balance directamente
                        if let Ok(balance_info) = client.get_token_account_balance(&account.pubkey) {
                            let balance = balance_info.ui_amount.unwrap_or(0.0);
                            println!("💰 {} Balance: {:.6} {}", symbol, balance, symbol);
                        }
                    }
                } else {
                    println!("💰 {} Balance: 0 {} (no account)", symbol, symbol);
                }
            }
            Err(_) => {
                println!("💰 {} Balance: 0 {} (error accessing account)", symbol, symbol);
            }
        }
    }
    
    println!("\n✅ Verificación de balance completada");
    
    if sol_ui < 0.05 {
        println!("⚠️ ADVERTENCIA: Balance SOL bajo. Necesitas al menos 0.05 SOL para arbitraje.");
        println!("💡 Ejecuta: cargo run --bin request_devnet_airdrop");
    } else {
        println!("✅ Balance SOL suficiente para arbitraje");
    }
    
    Ok(())
}

fn load_wallet_from_env() -> Result<Keypair> {
    if let Ok(private_key) = env::var("SOLANA_PRIVATE_KEY") {
        if private_key.starts_with('[') && private_key.ends_with(']') {
            let bytes_str = private_key.trim_start_matches('[').trim_end_matches(']');
            let bytes: Vec<u8> = bytes_str
                .split(',')
                .map(|s| s.trim().parse::<u8>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| anyhow::anyhow!("Invalid private key format: {}", e))?;
            
            if bytes.len() != 64 {
                return Err(anyhow::anyhow!("Private key must be 64 bytes long"));
            }
            
            Ok(Keypair::from_bytes(&bytes)?)
        } else {
            let bytes = bs58::decode(private_key)
                .into_vec()
                .map_err(|e| anyhow::anyhow!("Invalid base58 private key: {}", e))?;
            Ok(Keypair::from_bytes(&bytes)?)
        }
    } else {
        Err(anyhow::anyhow!("SOLANA_PRIVATE_KEY environment variable not found"))
    }
}
