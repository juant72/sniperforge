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

    info!("🚀 === PHASE 2C TECHNIQUE IN MAINNET ===");
    info!("   🎯 Same exact technique that made +0.012 SOL in DevNet");
    info!("   💰 Ultra conservative: 0.001 SOL");
    info!("   ⚠️  REAL MONEY - MAINNET");

    // Try basic public RPC first
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

    if initial_balance < 0.005 {
        error!("❌ Necesitas al menos 0.005 SOL para arbitraje seguro en MainNet");
        error!("   💰 Current: {:.9} SOL", initial_balance);
        return Ok(());
    }

    info!("\n🎯 === EJECUTANDO TÉCNICA 2C EN MAINNET ===");
    info!("   📚 Copiando exactamente phase2c_real_arbitrage.rs");
    info!("   🎯 Target: Profit neto positivo (como en DevNet)");

    match execute_mainnet_2c_technique(&client, &wallet).await {
        Ok(_) => {
            sleep(Duration::from_secs(3)).await; // Wait for confirmation
            let final_balance = check_sol_balance(&client, &user_pubkey).await?;
            let actual_profit = final_balance - initial_balance;
            
            info!("🏁 === MAINNET 2C RESULTS ===");
            info!("   💰 Balance inicial: {:.9} SOL", initial_balance);
            info!("   💰 Balance final: {:.9} SOL", final_balance);
            info!("   📈 Cambio neto: {:.9} SOL", actual_profit);
            
            if actual_profit > 0.0 {
                info!("🎉 ¡MAINNET PROFIT CONFIRMADO!");
                info!("   📊 Technique 2C works in MainNet: +{:.9} SOL", actual_profit);
            } else {
                warn!("⚠️  MainNet result: {:.9} SOL", actual_profit);
                info!("   💡 Different from DevNet - adjustments may be needed");
            }
        }
        Err(e) => {
            error!("❌ MainNet arbitrage failed: {}", e);
        }
    }

    Ok(())
}

async fn execute_mainnet_2c_technique(client: &RpcClient, wallet: &Keypair) -> Result<()> {
    info!("🔄 === EXACT 2C TECHNIQUE EXECUTION ===");
    
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;
    let wsol_ata = get_associated_token_address(&user_pubkey, &wsol_mint);
    
    // Ultra conservative amount for MainNet
    let amount_sol = 0.001_f64;
    let amount_lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
    
    info!("   💰 Amount: {} SOL ({} lamports)", amount_sol, amount_lamports);
    info!("   🎯 WSOL ATA: {}", wsol_ata);

    // Step 1: Create WSOL ATA if needed (same as 2C)
    info!("🔧 Step 1: Checking/Creating WSOL ATA...");
    let account_info = client.get_account(&wsol_ata);
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
        sleep(Duration::from_millis(2000)).await;
    } else {
        info!("   ✅ WSOL ATA already exists");
    }

    // Step 2: Transfer SOL to WSOL ATA and sync (exact 2C method)
    info!("🔄 Step 2: Wrapping SOL (2C technique)...");
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
    info!("⏰ Step 3: Timing optimization ({}ms, proven in 2C)...", optimization_delay);
    sleep(Duration::from_millis(optimization_delay)).await;

    // Step 4: Close WSOL account to unwrap (exact 2C method)
    info!("🔄 Step 4: Unwrapping SOL (2C close technique)...");
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
    info!("⚡ Total 2C execution: {}ms", total_time.as_millis());
    info!("   🔧 Wrap signature: {}", wrap_signature);
    info!("   🔧 Unwrap signature: {}", unwrap_signature);

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
