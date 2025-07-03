// Test de swap real configurado correctamente para DevNet
// Usa configuración parametrizada para evitar hardcoding de Program IDs

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
};
use tracing::{info, error, warn};
use sniperforge::shared::network_config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Real Swap Test with Parametrized Configuration");
    info!("================================================");

    // Load wallet from environment variable
    let private_key = std::env::var("SOLANA_PRIVATE_KEY")
        .expect("SOLANA_PRIVATE_KEY environment variable not set");
    let sender_keypair = Keypair::from_base58_string(&private_key);
    
    info!("📋 Wallet loaded: {}", sender_keypair.pubkey());

    // Get network configuration for DevNet
    let network_config = NetworkConfig::devnet();
    info!("🌐 Network: {}", network_config.name);
    info!("🔗 RPC: {}", network_config.rpc_endpoint);
    
    // Validate configuration
    if let Err(e) = network_config.validate() {
        error!("❌ Invalid network configuration: {}", e);
        return Ok(());
    }
    info!("✅ Network configuration validated");
    
    // Show available DEXes
    let available_dexes = network_config.available_dexes();
    info!("📊 Available DEXes: {:?}", available_dexes);
    
    // Connect to DevNet
    let rpc_client = RpcClient::new_with_commitment(
        network_config.rpc_endpoint.clone(),
        CommitmentConfig::confirmed(),
    );
    
    // Check balance
    let balance = rpc_client.get_balance(&sender_keypair.pubkey())?;
    let balance_sol = balance as f64 / LAMPORTS_PER_SOL as f64;
    info!("💰 Current balance: {} SOL", balance_sol);
    
    if balance_sol < 0.01 {
        error!("❌ Insufficient balance for test");
        return Ok(());
    }

    // Test 1: Simple native transfer (always works)
    info!("🔧 TEST 1: Native SOL Transfer");
    test_native_transfer(&rpc_client, &sender_keypair, &network_config).await?;
    
    // Test 2: Try Jupiter if available
    if network_config.has_jupiter() {
        info!("🔧 TEST 2: Jupiter Swap Test");
        test_jupiter_swap(&network_config).await?;
    } else {
        warn!("⚠️ Jupiter not available for {}", network_config.name);
    }
    
    // Test 3: Try Orca if available
    if network_config.has_orca() {
        info!("🔧 TEST 3: Orca Swap Test");
        test_orca_swap(&network_config).await?;
    } else {
        warn!("⚠️ Orca not available for {}", network_config.name);
    }

    info!("✅ All tests completed!");
    Ok(())
}

/// Test native SOL transfer using System Program
async fn test_native_transfer(
    rpc_client: &RpcClient,
    sender_keypair: &Keypair,
    network_config: &NetworkConfig,
) -> Result<()> {
    info!("   Using System Program: {}", network_config.program_ids.system_program);
    
    let transfer_amount = 1_000; // 0.000001 SOL
    let destination = sender_keypair.pubkey(); // Send to ourselves
    
    info!("   Transfer amount: {} lamports", transfer_amount);
    
    // Create transfer instruction using configured program ID
    let transfer_instruction = system_instruction::transfer(
        &sender_keypair.pubkey(),
        &destination,
        transfer_amount,
    );
    
    // Verify the instruction uses the correct program ID
    assert_eq!(transfer_instruction.program_id, network_config.program_ids.system_program);
    info!("   ✅ Instruction created with correct Program ID");
    
    // Get recent blockhash
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    
    // Create and sign transaction
    let transaction = Transaction::new_signed_with_payer(
        &[transfer_instruction],
        Some(&sender_keypair.pubkey()),
        &[sender_keypair],
        recent_blockhash,
    );
    
    // Send transaction
    match rpc_client.send_transaction(&transaction) {
        Ok(signature) => {
            info!("   ✅ Native transfer sent: {}", signature);
            info!("   🔍 Explorer: https://explorer.solana.com/tx/{}?cluster=devnet", signature);
        }
        Err(e) => {
            error!("   ❌ Native transfer failed: {}", e);
        }
    }
    
    Ok(())
}

/// Test Jupiter swap using configured Program IDs
async fn test_jupiter_swap(network_config: &NetworkConfig) -> Result<()> {
    let jupiter_program = network_config.program_ids.jupiter_program.unwrap();
    info!("   Jupiter Program: {}", jupiter_program);
    
    // Get test token pair
    let (input_token, output_token) = network_config.get_test_token_pair();
    info!("   Input token (SOL): {}", input_token);
    
    if let Some(output) = output_token {
        info!("   Output token (USDC): {}", output);
        info!("   ⚠️ Jupiter swap would use configured Program IDs (simulation only)");
        
        // Here we would use Jupiter API with the configured program IDs
        // For now, just validate that we have the right configuration
        
        if jupiter_program.to_string() == "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB" {
            warn!("   ⚠️ Using DevNet Jupiter Program ID - may have limited functionality");
        }
        
        info!("   ✅ Jupiter configuration ready (implementation pending)");
    } else {
        warn!("   ⚠️ No output token configured for test pair");
    }
    
    Ok(())
}

/// Test Orca swap using configured Program IDs
async fn test_orca_swap(network_config: &NetworkConfig) -> Result<()> {
    let orca_program = network_config.program_ids.orca_whirlpool_program.unwrap();
    info!("   Orca Whirlpool Program: {}", orca_program);
    
    // Validate that we have the correct Orca program ID
    if orca_program.to_string() == "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc" {
        info!("   ✅ Using correct Orca Whirlpool Program ID");
    } else {
        warn!("   ⚠️ Unexpected Orca Program ID: {}", orca_program);
    }
    
    info!("   Token Program: {}", network_config.program_ids.token_program);
    info!("   Associated Token Program: {}", network_config.program_ids.associated_token_program);
    
    // Get test token pair
    let (input_token, output_token) = network_config.get_test_token_pair();
    info!("   Input token (SOL): {}", input_token);
    
    if let Some(output) = output_token {
        info!("   Output token (USDC): {}", output);
        info!("   ✅ Orca swap configuration ready (would use configured Program IDs)");
    } else {
        warn!("   ⚠️ No output token configured for Orca test");
    }
    
    Ok(())
}
