// Test de Jupiter corregido con configuración parametrizada
// Este debería resolver el problema original de Program IDs incorrectos

use anyhow::Result;
use tracing::{info, error, warn};
use solana_sdk::{signature::{Keypair, Signer}};
use sniperforge::shared::{
    jupiter_api::Jupiter,
    jupiter_config::JupiterConfig,
    network_config::NetworkConfig,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Jupiter Fixed Test with Network Configuration");
    info!("===============================================");

    // Load wallet from environment variable
    let private_key = std::env::var("SOLANA_PRIVATE_KEY")
        .expect("SOLANA_PRIVATE_KEY environment variable not set");
    let sender_keypair = Keypair::from_base58_string(&private_key);
    
    info!("📋 Wallet loaded: {}", sender_keypair.pubkey());

    // Create network configuration for DevNet
    let network_config = NetworkConfig::devnet();
    info!("🌐 Network: {}", network_config.name);
    info!("🔗 RPC: {}", network_config.rpc_endpoint);
    
    // Validate configuration
    if let Err(e) = network_config.validate() {
        error!("❌ Invalid network configuration: {}", e);
        return Ok(());
    }
    
    // Check if Jupiter is available
    if !network_config.has_jupiter() {
        error!("❌ Jupiter not available for {}", network_config.name);
        return Ok(());
    }
    
    info!("✅ Jupiter Program ID for {}: {:?}", 
          network_config.name, 
          network_config.program_ids.jupiter_program);

    // Create Jupiter configuration that matches the network
    let jupiter_config = JupiterConfig {
        base_url: "https://quote-api.jup.ag".to_string(),
        api_key: None,
        timeout_seconds: 30,
        max_retries: 3,
        rpc_endpoint: network_config.rpc_endpoint.clone(),
        network_name: network_config.name.clone(),
    };

    // Create Jupiter instance with explicit network configuration
    info!("🔧 Creating Jupiter instance with network configuration...");
    let jupiter = match Jupiter::new_with_network(&jupiter_config, network_config.clone()).await {
        Ok(jupiter) => {
            info!("✅ Jupiter created successfully");
            jupiter
        }
        Err(e) => {
            error!("❌ Failed to create Jupiter: {}", e);
            return Ok(());
        }
    };

    // Get test token pair from network configuration
    let (input_token, output_token) = network_config.get_test_token_pair();
    
    if output_token.is_none() {
        warn!("⚠️ No output token configured for {} - using SOL transfer test", network_config.name);
        return Ok(());
    }
    
    let output_token = output_token.unwrap();
    
    info!("🔄 Testing Jupiter swap:");
    info!("   Input: {} (SOL)", input_token);
    info!("   Output: {} (USDC)", output_token);
    info!("   Amount: 0.00001 SOL");

    // Test 1: Get a quote
    info!("📊 Step 1: Getting quote from Jupiter...");
    let quote = match jupiter.get_quote(
        &input_token.to_string(),
        &output_token.to_string(),
        0.00001, // 0.00001 SOL
        100, // 1% slippage
    ).await {
        Ok(quote) => {
            info!("✅ Quote received:");
            info!("   Input amount: {} SOL", quote.in_amount());
            info!("   Output amount: {} USDC", quote.out_amount());
            info!("   Price impact: {}%", quote.price_impact_pct());
            quote
        }
        Err(e) => {
            error!("❌ Failed to get quote: {}", e);
            return Ok(());
        }
    };

    // Test 2: Build transaction with correct Program IDs
    info!("🔧 Step 2: Building swap transaction with correct Program IDs...");
    match jupiter.build_swap_transaction(&quote, &sender_keypair.pubkey().to_string()).await {
        Ok(swap_result) => {
            info!("✅ Transaction built successfully:");
            info!("   Success: {}", swap_result.success);
            info!("   Expected output: {} tokens", swap_result.output_amount);
            info!("   Estimated fee: {} SOL", swap_result.fee_amount);
        }
        Err(e) => {
            error!("❌ Failed to build transaction: {}", e);
            return Ok(());
        }
    }

    // Test 3: Execute swap with wallet (REAL EXECUTION)
    info!("🚀 Step 3: Executing REAL swap on {} with corrected Program IDs...", network_config.name);
    warn!("⚠️  This will execute a REAL transaction on DevNet");
    
    match jupiter.execute_swap_with_wallet(
        &quote,
        &sender_keypair.pubkey().to_string(),
        Some(&sender_keypair),
    ).await {
        Ok(result) => {
            if result.success {
                info!("🎉 SWAP EXECUTED SUCCESSFULLY!");
                info!("🎯 Transaction signature: {}", result.transaction_signature);
                info!("💰 Output amount: {} tokens", result.output_amount);
                info!("📊 Actual slippage: {}%", result.actual_slippage);
                info!("💸 Fee paid: {} SOL", result.fee_amount);
                info!("🔍 Explorer: https://explorer.solana.com/tx/{}?cluster=devnet", 
                      result.transaction_signature);
                
                info!("✅ SUCCESS: Jupiter swap completed with correct Program IDs!");
            } else {
                warn!("⚠️ Swap not executed:");
                for log in &result.logs {
                    warn!("   {}", log);
                }
            }
        }
        Err(e) => {
            error!("❌ Swap execution failed: {}", e);
            error!("This may be due to:");
            error!("  - Insufficient DevNet liquidity for this token pair");
            error!("  - Network issues with DevNet");
            error!("  - Jupiter API limitations on DevNet");
        }
    }

    info!("✅ Jupiter test completed with network configuration!");
    Ok(())
}
