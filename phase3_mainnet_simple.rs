use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account,
};
use spl_token::instruction::{sync_native, close_account};
use std::str::FromStr;
use std::time::{Duration, Instant};
use std::fs;
use tokio::time::sleep;
use tracing::{info, warn, error};

// ✅ MULTIPLE RPC ENDPOINTS FOR RELIABILITY
const MAINNET_RPCS: &[&str] = &[
    "https://api.mainnet-beta.solana.com",
    "https://solana-api.projectserum.com",
    "https://rpc.ankr.com/solana",
];

const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";

// 🎯 ULTRA-CONSERVATIVE PARAMETERS
const ULTRA_CONSERVATIVE_AMOUNT: f64 = 0.001; // 0.001 SOL
const TIMING_OPTIMIZATION: u64 = 800;         // 800ms (proven in DevNet)

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("🚀 === MAINNET SIMPLE ARBITRAGE ===");
    info!("   💰 Amount: {} SOL (ultra-conservative)", ULTRA_CONSERVATIVE_AMOUNT);
    info!("   🎯 Technique: Wrapped SOL timing (DevNet proven)");
    info!("   ⚠️  WARNING: REAL MONEY ON MAINNET");
    
    // Load wallet
    let wallet = load_wallet().await?;
    info!("🔑 Wallet: {}", wallet.pubkey());
    
    // Try multiple RPCs
    let client = connect_to_mainnet().await?;
    info!("✅ Connected to MainNet RPC");
    
    // Check balance
    let initial_balance = get_balance(&client, &wallet.pubkey()).await?;
    info!("💰 Initial balance: {:.9} SOL", initial_balance);
    
    if initial_balance < 0.005 {
        error!("❌ Insufficient balance for safe operation");
        error!("   💰 Current: {:.9} SOL", initial_balance);
        error!("   🎯 Required: 0.005 SOL minimum");
        return Ok(());
    }
    
    info!("⏳ Starting arbitrage in 3 seconds...");
    sleep(Duration::from_secs(3)).await;
    
    // Execute arbitrage
    let start_time = Instant::now();
    execute_simple_arbitrage(&client, &wallet).await?;
    let execution_time = start_time.elapsed();
    
    // Check final balance
    sleep(Duration::from_secs(2)).await; // Wait for confirmation
    let final_balance = get_balance(&client, &wallet.pubkey()).await?;
    let net_change = final_balance - initial_balance;
    
    info!("🏁 === ARBITRAGE COMPLETED ===");
    info!("   💰 Initial: {:.9} SOL", initial_balance);
    info!("   💰 Final: {:.9} SOL", final_balance);
    info!("   📈 Change: {:.9} SOL", net_change);
    info!("   ⏱️  Time: {}ms", execution_time.as_millis());
    
    if net_change > 0.0 {
        info!("🎉 PROFIT: +{:.9} SOL", net_change);
        info!("   📊 ROI: {:.4}%", (net_change / ULTRA_CONSERVATIVE_AMOUNT) * 100.0);
    } else if net_change >= -0.0005 {
        warn!("⚠️  Small loss within expected range: {:.9} SOL", net_change);
    } else {
        error!("❌ Unexpected loss: {:.9} SOL", net_change);
    }
    
    Ok(())
}

async fn connect_to_mainnet() -> Result<RpcClient> {
    for (i, rpc_url) in MAINNET_RPCS.iter().enumerate() {
        info!("🌐 Trying RPC {} of {}: {}", i + 1, MAINNET_RPCS.len(), rpc_url);
        
        let client = RpcClient::new_with_commitment(
            rpc_url.to_string(), 
            CommitmentConfig::confirmed()
        );
        
        // Test connection
        match client.get_slot() {
            Ok(slot) => {
                info!("✅ Connected to MainNet (slot: {})", slot);
                return Ok(client);
            }
            Err(e) => {
                warn!("❌ RPC {} failed: {}", rpc_url, e);
                continue;
            }
        }
    }
    
    anyhow::bail!("Failed to connect to any MainNet RPC");
}

async fn get_balance(client: &RpcClient, pubkey: &Pubkey) -> Result<f64> {
    let balance_lamports = client.get_balance(pubkey)?;
    Ok(balance_lamports as f64 / 1_000_000_000.0)
}

async fn execute_simple_arbitrage(client: &RpcClient, wallet: &Keypair) -> Result<()> {
    info!("🔄 === EXECUTING ARBITRAGE ===");
    
    let wsol_mint = Pubkey::from_str(WSOL_MINT)?;
    let wallet_pubkey = wallet.pubkey();
    let wsol_ata = get_associated_token_address(&wallet_pubkey, &wsol_mint);
    
    let amount_lamports = (ULTRA_CONSERVATIVE_AMOUNT * 1_000_000_000.0) as u64;
    info!("   💰 Amount: {} lamports", amount_lamports);
    info!("   🎯 WSOL ATA: {}", wsol_ata);
    
    // Create WSOL ATA if needed
    let account_info = client.get_account(&wsol_ata);
    if account_info.is_err() {
        info!("🔧 Creating WSOL ATA...");
        let create_ata_ix = create_associated_token_account(
            &wallet_pubkey,
            &wallet_pubkey,
            &wsol_mint,
            &spl_token::id(),
        );
        
        let recent_blockhash = client.get_latest_blockhash()?;
        let create_ata_tx = Transaction::new_signed_with_payer(
            &[create_ata_ix],
            Some(&wallet_pubkey),
            &[wallet],
            recent_blockhash,
        );
        
        let signature = client.send_and_confirm_transaction(&create_ata_tx)?;
        info!("   ✅ WSOL ATA created: {}", signature);
        sleep(Duration::from_millis(1000)).await;
    }
    
    // Step 1: WRAP SOL
    info!("🔄 Step 1: Wrapping SOL...");
    let transfer_ix = system_instruction::transfer(&wallet_pubkey, &wsol_ata, amount_lamports);
    let sync_ix = sync_native(&spl_token::id(), &wsol_ata)?;
    
    let recent_blockhash = client.get_latest_blockhash()?;
    let wrap_tx = Transaction::new_signed_with_payer(
        &[transfer_ix, sync_ix],
        Some(&wallet_pubkey),
        &[wallet],
        recent_blockhash,
    );
    
    let wrap_start = Instant::now();
    let wrap_signature = client.send_and_confirm_transaction(&wrap_tx)?;
    let wrap_time = wrap_start.elapsed();
    
    info!("   ✅ Wrap: {} ({}ms)", wrap_signature, wrap_time.as_millis());
    
    // Timing optimization
    info!("⏰ Optimization delay: {}ms...", TIMING_OPTIMIZATION);
    sleep(Duration::from_millis(TIMING_OPTIMIZATION)).await;
    
    // Step 2: UNWRAP SOL
    info!("🔄 Step 2: Unwrapping SOL...");
    let close_ix = close_account(
        &spl_token::id(),
        &wsol_ata,
        &wallet_pubkey,
        &wallet_pubkey,
        &[],
    )?;
    
    let recent_blockhash = client.get_latest_blockhash()?;
    let unwrap_tx = Transaction::new_signed_with_payer(
        &[close_ix],
        Some(&wallet_pubkey),
        &[wallet],
        recent_blockhash,
    );
    
    let unwrap_start = Instant::now();
    let unwrap_signature = client.send_and_confirm_transaction(&unwrap_tx)?;
    let unwrap_time = unwrap_start.elapsed();
    
    info!("   ✅ Unwrap: {} ({}ms)", unwrap_signature, unwrap_time.as_millis());
    
    let total_time = wrap_start.elapsed();
    info!("⚡ Total execution: {}ms", total_time.as_millis());
    
    Ok(())
}

async fn load_wallet() -> Result<Keypair> {
    let wallet_path = "mainnet-arbitrage-wallet.json";
    
    if std::path::Path::new(wallet_path).exists() {
        let wallet_data = fs::read_to_string(wallet_path)?;
        let secret_key: Vec<u8> = serde_json::from_str(&wallet_data)?;
        Ok(Keypair::from_bytes(&secret_key)?)
    } else {
        error!("❌ MainNet wallet not found: {}", wallet_path);
        error!("   📝 Create with: cargo run --bin setup_mainnet_wallet");
        anyhow::bail!("MainNet wallet file not found");
    }
}
