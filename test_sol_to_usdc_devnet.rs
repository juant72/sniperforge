// Test de swap real SOL -> USDC en DevNet
// Versión corregida usando token que realmente funciona

use dotenv::dotenv;
use sniperforge::shared::jupiter_api::Jupiter;
use sniperforge::shared::jupiter_config::JupiterConfig;
use sniperforge::shared::network_config::NetworkConfig;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, signature::Keypair, signer::Signer};
use std::error::Error;
use std::{env, fs};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    println!("🎯 TESTING REAL SWAP: SOL -> USDC on DevNet");
    println!("==================================================");

    // Configuración DevNet
    let network_config = NetworkConfig::devnet();
    let jupiter_config = JupiterConfig::devnet();
    let rpc_client =
        RpcClient::new_with_commitment(&network_config.rpc_endpoint, CommitmentConfig::confirmed());

    // Cargar wallet
    println!("📂 Loading wallet from environment...");

    let private_key =
        env::var("SOLANA_PRIVATE_KEY").expect("SOLANA_PRIVATE_KEY not found in environment");

    let private_key_bytes = bs58::decode(private_key)
        .into_vec()
        .expect("Failed to decode private key");

    let wallet =
        Keypair::from_bytes(&private_key_bytes).expect("Failed to create wallet from private key");

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
    let jupiter = Jupiter::new_with_network(&jupiter_config, network_config.clone()).await?;
    println!("✅ Jupiter API connected");

    // Configurar swap: SOL -> USDC
    let sol_mint = network_config.token_addresses.sol;
    let usdc_mint = network_config
        .token_addresses
        .usdc
        .expect("USDC not configured for DevNet");
    let amount_sol = 0.00001; // 0.00001 SOL

    println!("\n📊 Getting quote from Jupiter...");
    println!("   Input: {} SOL", amount_sol);
    println!("   From: {} (SOL)", sol_mint);
    println!("   To: {} (USDC)", usdc_mint);

    // Obtener quote
    let quote = jupiter
        .get_quote(
            &sol_mint.to_string(),
            &usdc_mint.to_string(),
            amount_sol,
            300, // 3% slippage
        )
        .await?;

    println!("✅ Quote received from Jupiter");
    println!("   Output: {} USDC tokens", quote.out_amount());
    println!("   Price Impact: {}%", quote.price_impact_pct());
    println!("   Route: {} steps", quote.routePlan.len());

    // Construir y ejecutar transacción
    println!("\n🚀 EXECUTING REAL SWAP ON DEVNET...");

    match jupiter
        .execute_swap_with_wallet(&quote, &wallet.pubkey().to_string(), Some(&wallet))
        .await
    {
        Ok(result) => {
            if result.success {
                println!("✅ SWAP EXECUTED SUCCESSFULLY!");
                println!("📝 Transaction signature: {}", result.transaction_signature);
                println!("🔗 View on Solana Explorer:");
                println!(
                    "   https://explorer.solana.com/tx/{}?cluster=devnet",
                    result.transaction_signature
                );

                // Verificar nuevo balance
                println!("\n💰 Checking new balance...");
                let new_balance = rpc_client.get_balance(&wallet.pubkey())?;
                let new_sol_balance = new_balance as f64 / 1_000_000_000.0;
                println!("   New balance: {} SOL", new_sol_balance);
                println!("   Difference: {} SOL", new_sol_balance - sol_balance);
            } else {
                println!("❌ SWAP EXECUTION FAILED");
                println!("   Transaction signature: {}", result.transaction_signature);
                println!("   Logs:");
                for log in result.logs {
                    println!("     {}", log);
                }
            }
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
