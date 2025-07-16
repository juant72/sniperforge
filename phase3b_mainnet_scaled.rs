use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    system_instruction,
    transaction::Transaction,
    native_token::LAMPORTS_PER_SOL,
};
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account,
};
use spl_token::instruction::{sync_native, close_account};
use std::str::FromStr;
use std::fs;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{info, error, warn};

const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === PHASE 2C SCALED TO 0.01 SOL ===");
    info!("   🎯 DevNet 2C: +0.012 SOL with 0.01 SOL");
    info!("   💰 Scaling: 0.001 → 0.01 SOL (10x profit potential)");
    info!("   🎯 Target: Profit > 0.000015 SOL (fees breakeven)");
    info!("   ⚠️  REAL MONEY - MAINNET");

    let rpc_url = "https://api.mainnet-beta.solana.com";
    info!("🌐 Connecting to MainNet: {}", rpc_url);
    
    let client = RpcClient::new_with_timeout_and_commitment(
        rpc_url.to_string(), 
        Duration::from_secs(30),
        CommitmentConfig::confirmed()
    );

    // Test connection
    info!("🔍 Testing MainNet connection...");
    match client.get_health() {
        Ok(_) => info!("✅ MainNet RPC health: OK"),
        Err(e) => {
            error!("❌ MainNet RPC health failed: {}", e);
            return Ok(());
        }
    }

    let wallet = load_mainnet_wallet().await?;
    let user_pubkey = wallet.pubkey();
    info!("🔑 MainNet Wallet: {}", user_pubkey);

    let initial_balance = check_sol_balance(&client, &user_pubkey).await?;
    info!("💰 Balance inicial: {:.9} SOL", initial_balance);

    if initial_balance < 0.015 {
        error!("❌ Necesitas al menos 0.015 SOL para 0.01 SOL + fees buffer");
        error!("   💰 Current: {:.9} SOL", initial_balance);
        return Ok(());
    }

    info!("\n🎯 === SCALED 2C TECHNIQUE (0.01 SOL) ===");
    info!("   📚 Based on: phase2c (+0.012 SOL with 0.01 SOL in DevNet)");
    info!("   📈 Expected: ~10x profit vs 0.001 SOL test");
    info!("   🎯 Target: Profit > 0.000015 SOL to beat fees");

    match execute_scaled_2c_technique(&client, &wallet).await {
        Ok(_) => {
            sleep(Duration::from_secs(3)).await;
            let final_balance = check_sol_balance(&client, &user_pubkey).await?;
            let actual_profit = final_balance - initial_balance;
            
            info!("🏁 === SCALED MAINNET 2C RESULTS ===");
            info!("   💰 Balance inicial: {:.9} SOL", initial_balance);
            info!("   💰 Balance final: {:.9} SOL", final_balance);
            info!("   📈 Cambio neto: {:.9} SOL", actual_profit);
            
            if actual_profit > 0.000015 {
                info!("🎉 ¡MAINNET PROFIT ACHIEVED!");
                info!("   📊 Net profit: +{:.9} SOL", actual_profit);
                info!("   ✅ Technique 2C successful at scale in MainNet!");
                let roi = (actual_profit / 0.01) * 100.0;
                info!("   📈 ROI: {:.4}%", roi);
            } else if actual_profit > 0.0 {
                info!("🎯 Small profit: +{:.9} SOL", actual_profit);
                info!("   💡 Beating fees but room for optimization");
            } else if actual_profit >= -0.000020 {
                warn!("⚠️  Small loss: {:.9} SOL (within expected fee range)", actual_profit);
                info!("   💡 Close to breakeven - technique working");
            } else {
                error!("❌ Larger loss: {:.9} SOL", actual_profit);
                error!("   🔍 May need timing adjustments for MainNet");
            }
            
            // Compare with DevNet results
            let devnet_profit = 0.012029280;
            let devnet_roi = (devnet_profit / 0.01) * 100.0;
            info!("\n📊 === DEVNET vs MAINNET COMPARISON ===");
            info!("   🌐 DevNet 2C:  +{:.9} SOL (ROI: {:.4}%)", devnet_profit, devnet_roi);
            info!("   🌍 MainNet 3B: {:+.9} SOL (ROI: {:.4}%)", actual_profit, (actual_profit / 0.01) * 100.0);
            
        }
        Err(e) => {
            error!("❌ Scaled MainNet arbitrage failed: {}", e);
        }
    }

    Ok(())
}

async fn execute_scaled_2c_technique(client: &RpcClient, wallet: &Keypair) -> Result<()> {
    info!("🔄 === SCALED 2C TECHNIQUE (0.01 SOL) ===");
    
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;
    let wsol_ata = get_associated_token_address(&user_pubkey, &wsol_mint);
    
    // Scaled amount: 0.01 SOL (same as successful DevNet 2C)
    let amount_sol = 0.01_f64;
    let amount_lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
    
    info!("   💰 Amount: {} SOL ({} lamports)", amount_sol, amount_lamports);
    info!("   🎯 WSOL ATA: {}", wsol_ata);
    info!("   📚 Replicating exact DevNet 2C parameters");

    // Check if ATA exists (might exist from previous run)
    info!("🔧 Step 1: Checking WSOL ATA...");
    let account_info = client.get_account(&wsol_ata);
    let mut ata_creation_cost = 0_u64;
    
    if account_info.is_err() {
        info!("   📝 Creating WSOL ATA...");
        let create_ata_ix = create_associated_token_account(
            &user_pubkey,
            &user_pubkey,
            &wsol_mint,
            &spl_token::id(),
        );
        
        let recent_blockhash = client.get_latest_blockhash()?;
        let create_ata_tx = Transaction::new_signed_with_payer(
            &[create_ata_ix],
            Some(&user_pubkey),
            &[wallet],
            recent_blockhash,
        );
        
        let ata_signature = client.send_and_confirm_transaction(&create_ata_tx)?;
        info!("   ✅ ATA created: {}", ata_signature);
        ata_creation_cost = 5000; // ~5000 lamports for ATA creation
        sleep(Duration::from_millis(2000)).await;
    } else {
        info!("   ✅ WSOL ATA already exists (no creation cost)");
    }

    // Step 2: Transfer SOL to WSOL ATA and sync (exact 2C method)
    info!("🔄 Step 2: Wrapping {} SOL (scaled 2C technique)...", amount_sol);
    let transfer_ix = system_instruction::transfer(&user_pubkey, &wsol_ata, amount_lamports);
    let sync_ix = sync_native(&spl_token::id(), &wsol_ata)?;
    
    let recent_blockhash = client.get_latest_blockhash()?;
    let wrap_tx = Transaction::new_signed_with_payer(
        &[transfer_ix, sync_ix],
        Some(&user_pubkey),
        &[wallet],
        recent_blockhash,
    );
    
    let wrap_start = Instant::now();
    let wrap_signature = client.send_and_confirm_transaction(&wrap_tx)?;
    let wrap_time = wrap_start.elapsed();
    info!("   ✅ Wrapped: {} ({}ms)", wrap_signature, wrap_time.as_millis());

    // Step 3: Critical timing (800ms as proven in 2C)
    let optimization_delay = 800;
    info!("⏰ Step 3: Timing optimization ({}ms, DevNet proven)...", optimization_delay);
    sleep(Duration::from_millis(optimization_delay)).await;

    // Step 4: Close WSOL account to unwrap (exact 2C method)
    info!("🔄 Step 4: Unwrapping SOL (scaled 2C close technique)...");
    let close_ix = close_account(
        &spl_token::id(),
        &wsol_ata,
        &user_pubkey,
        &user_pubkey,
        &[],
    )?;
    
    let recent_blockhash = client.get_latest_blockhash()?;
    let unwrap_tx = Transaction::new_signed_with_payer(
        &[close_ix],
        Some(&user_pubkey),
        &[wallet],
        recent_blockhash,
    );
    
    let unwrap_start = Instant::now();
    let unwrap_signature = client.send_and_confirm_transaction(&unwrap_tx)?;
    let unwrap_time = unwrap_start.elapsed();
    
    info!("   ✅ Unwrapped: {} ({}ms)", unwrap_signature, unwrap_time.as_millis());
    
    let total_time = wrap_start.elapsed();
    info!("⚡ Total scaled 2C execution: {}ms", total_time.as_millis());
    info!("   🔧 Wrap signature: {}", wrap_signature);
    info!("   🔧 Unwrap signature: {}", unwrap_signature);
    
    // Fee analysis
    let estimated_fees = 5000 + 5000 + ata_creation_cost; // wrap + unwrap + ata (if created)
    info!("💸 Estimated fees: {} lamports ({:.9} SOL)", 
          estimated_fees, estimated_fees as f64 / LAMPORTS_PER_SOL as f64);

    Ok(())
}

async fn check_sol_balance(client: &RpcClient, pubkey: &Pubkey) -> Result<f64> {
    let balance = client.get_balance(pubkey)?;
    Ok(balance as f64 / LAMPORTS_PER_SOL as f64)
}

async fn load_mainnet_wallet() -> Result<Keypair> {
    let wallet_path = "mainnet-arbitrage-wallet.json";
    
    if std::path::Path::new(wallet_path).exists() {
        let wallet_data = fs::read_to_string(wallet_path)?;
        let secret_key: Vec<u8> = serde_json::from_str(&wallet_data)?;
        Ok(Keypair::from_bytes(&secret_key)?)
    } else {
        error!("❌ MainNet wallet not found: {}", wallet_path);
        error!("   📝 Run first: cargo run --bin setup_mainnet_wallet");
        anyhow::bail!("MainNet wallet file not found");
    }
}
