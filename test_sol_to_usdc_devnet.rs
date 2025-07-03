// Test de swap real SOL -> USDC en DevNet
// Versión corregida usando token que realmente funciona

use dotenv::dotenv;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::Keypair,
    signer::Signer,
    commitment_config::CommitmentConfig,
};
use std::fs;
use std::error::Error;
use sniperforge::shared::jupiter_api::Jupiter;
use sniperforge::shared::network_config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    
    println!("🎯 TESTING REAL SWAP: SOL -> USDC on DevNet");
    println!("==================================================");
    
    // Configuración DevNet
    let config = NetworkConfig::devnet();
    let rpc_client = RpcClient::new_with_commitment(&config.rpc_endpoint, CommitmentConfig::confirmed());
    
    // Cargar wallet
    let wallet_path = "test-wallet-new.json";
    println!("📂 Loading wallet from: {}", wallet_path);
    
    let wallet_data = fs::read_to_string(wallet_path)?;
    let wallet_bytes: Vec<u8> = serde_json::from_str(&wallet_data)?;
    let wallet = Keypair::from_bytes(&wallet_bytes)?;
    
    println!("✅ Wallet loaded successfully");
    println!("🔑 Public key: {}", wallet.pubkey());
    
    // Verificar balance
    println!("\n💰 Checking wallet balance...");
    let balance = rpc_client.get_balance(&wallet.pubkey())?;
    let sol_balance = balance as f64 / 1_000_000_000.0;
    println!("   Balance: {} SOL", sol_balance);
    
    if sol_balance < 0.001 {
        println!("❌ Insufficient balance for swap test");
        return Ok(());
    }
    
    // Inicializar Jupiter API
    println!("\n🌍 Initializing Jupiter API...");
    let jupiter = Jupiter::new(&config).await?;
    println!("✅ Jupiter API connected");
    
    // Configurar swap: SOL -> USDC
    let sol_mint = config.token_addresses.sol;
    let usdc_mint = config.token_addresses.usdc.expect("USDC not configured for DevNet");
    let amount_lamports = 10_000; // 0.00001 SOL
    
    println!("\n📊 Getting quote from Jupiter...");
    println!("   Input: {} SOL", amount_lamports as f64 / 1_000_000_000.0);
    println!("   From: {} (SOL)", sol_mint);
    println!("   To: {} (USDC)", usdc_mint);
    
    // Obtener quote
    let quote = jupiter.get_quote(
        &sol_mint.to_string(),
        &usdc_mint.to_string(),
        amount_lamports,
        300, // 3% slippage
    ).await?;
    
    println!("✅ Quote received from Jupiter");
    println!("   Output: {} USDC tokens", quote.out_amount);
    println!("   Price Impact: {}%", quote.price_impact_pct.unwrap_or(0.0));
    println!("   Route: {} steps", quote.route_plan.len());
    
    // Construir y ejecutar transacción
    println!("\n🚀 EXECUTING REAL SWAP ON DEVNET...");
    
    match jupiter.execute_swap_with_wallet(&quote, &wallet, &rpc_client).await {
        Ok(signature) => {
            println!("✅ SWAP EXECUTED SUCCESSFULLY!");
            println!("📝 Transaction signature: {}", signature);
            println!("🔗 View on Solana Explorer:");
            println!("   https://explorer.solana.com/tx/{}?cluster=devnet", signature);
            
            // Verificar nuevo balance
            println!("\n💰 Checking new balance...");
            let new_balance = rpc_client.get_balance(&wallet.pubkey())?;
            let new_sol_balance = new_balance as f64 / 1_000_000_000.0;
            println!("   New balance: {} SOL", new_sol_balance);
            println!("   Difference: {} SOL", new_sol_balance - sol_balance);
        }
        Err(e) => {
            println!("❌ SWAP EXECUTION FAILED");
            println!("   Reason: {}", e);
            
            // Analizar el error
            let error_str = e.to_string();
            if error_str.contains("IncorrectProgramId") {
                println!("\n🔍 Analysis: Program ID mismatch detected");
                println!("   This usually means:");
                println!("   • Token account doesn't exist for this wallet");
                println!("   • Token mint is not properly initialized on DevNet");
                println!("   • Associated token account creation failed");
            } else if error_str.contains("InsufficientFunds") {
                println!("\n🔍 Analysis: Insufficient funds");
                println!("   Check wallet balance and transaction fees");
            } else {
                println!("\n🔍 Analysis: Other execution error");
                println!("   Check DevNet RPC status and transaction limits");
            }
        }
    }
    
    Ok(())
}
