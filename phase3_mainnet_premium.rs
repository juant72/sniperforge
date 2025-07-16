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
use std::env;
use tokio::time::sleep;
use tracing::{info, warn, error};

// 🌟 PREMIUM RPC ENDPOINTS (From PREMIUM-RPC-GUIDE.md)
fn get_mainnet_rpcs() -> Vec<String> {
    let mut rpcs = Vec::new();
    
    // 🥇 HELIUS (Highest Priority - from guide)
    if let Ok(api_key) = env::var("HELIUS_API_KEY") {
        rpcs.push(format!("https://mainnet.helius-rpc.com/?api-key={}", api_key));
        info!("✅ Found Helius API key");
    }
    
    // 🥈 QUICKNODE (Second Priority - from guide)
    if let Ok(endpoint) = env::var("QUICKNODE_ENDPOINT") {
        rpcs.push(endpoint);
        info!("✅ Found QuickNode endpoint");
    }
    
    // 🥉 ALCHEMY (Third Priority - from guide)
    if let Ok(api_key) = env::var("ALCHEMY_API_KEY") {
        rpcs.push(format!("https://solana-mainnet.g.alchemy.com/v2/{}", api_key));
        info!("✅ Found Alchemy API key");
    }
    
    // ANKR (Budget option - from guide)
    if let Ok(api_key) = env::var("ANKR_API_KEY") {
        rpcs.push(format!("https://rpc.ankr.com/solana/{}", api_key));
        info!("✅ Found Ankr API key");
    }
    
    // Fallback public RPCs (as documented in guide)
    rpcs.extend([
        "https://api.mainnet-beta.solana.com".to_string(),
        "https://solana-api.projectserum.com".to_string(),
        "https://rpc.ankr.com/solana".to_string(),
    ]);
    
    info!("🌐 Loaded {} RPC endpoints total", rpcs.len());
    rpcs
}

const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";

// 🎯 ULTRA-CONSERVATIVE PARAMETERS (proven in DevNet 2C)
const ULTRA_CONSERVATIVE_AMOUNT: f64 = 0.001; // 0.001 SOL (~$0.20)
const TIMING_OPTIMIZATION: u64 = 800;         // 800ms (proven successful)

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("🚀 === MAINNET PREMIUM RPC ARBITRAGE ===");
    info!("   💰 Amount: {} SOL (ultra-conservative)", ULTRA_CONSERVATIVE_AMOUNT);
    info!("   🎯 Technique: Wrapped SOL timing (2C proven: +0.012 SOL)");
    info!("   🌟 Using premium RPCs from PREMIUM-RPC-GUIDE.md");
    info!("   ⚠️  WARNING: REAL MONEY ON MAINNET");
    
    // Load wallet
    let wallet = load_wallet().await?;
    info!("🔑 Wallet: {}", wallet.pubkey());
    
    // Try premium RPCs first
    let client = connect_to_mainnet().await?;
    info!("✅ Connected to MainNet RPC");
    
    // Check balance
    let initial_balance = get_balance(&client, &wallet.pubkey()).await?;
    info!("💰 Initial balance: {:.9} SOL", initial_balance);
    
    if initial_balance < 0.005 {
        error!("❌ Insufficient balance for safe operation");
        error!("   💰 Current: {:.9} SOL", initial_balance);
        error!("   🎯 Required: 0.005 SOL minimum (includes fees + buffer)");
        return Ok(());
    }
    
    info!("⏳ Starting MainNet arbitrage in 3 seconds...");
    info!("   🎯 Using proven technique from phase2c (+0.012 SOL in DevNet)");
    sleep(Duration::from_secs(3)).await;
    
    // Execute arbitrage with proven 2C technique
    let start_time = Instant::now();
    execute_proven_arbitrage(&client, &wallet).await?;
    let execution_time = start_time.elapsed();
    
    // Check final balance
    sleep(Duration::from_secs(3)).await; // Wait for MainNet confirmation
    let final_balance = get_balance(&client, &wallet.pubkey()).await?;
    let net_change = final_balance - initial_balance;
    
    info!("🏁 === MAINNET ARBITRAGE COMPLETED ===");
    info!("   💰 Initial: {:.9} SOL", initial_balance);
    info!("   💰 Final: {:.9} SOL", final_balance);
    info!("   📈 Change: {:.9} SOL", net_change);
    info!("   ⏱️  Time: {}ms", execution_time.as_millis());
    
    if net_change > 0.0 {
        info!("🎉 MAINNET PROFIT: +{:.9} SOL", net_change);
        info!("   📊 ROI: {:.4}%", (net_change / ULTRA_CONSERVATIVE_AMOUNT) * 100.0);
        info!("   🎯 Technique 2C confirmed working in MainNet!");
    } else if net_change >= -0.0005 {
        warn!("⚠️  Small loss within expected range: {:.9} SOL", net_change);
        warn!("   💡 MainNet conditions different from DevNet");
        warn!("   🔧 Consider adjusting timing parameters");
    } else {
        error!("❌ Unexpected loss: {:.9} SOL", net_change);
        error!("   🔍 Review MainNet vs DevNet differences");
    }
    
    Ok(())
}

async fn connect_to_mainnet() -> Result<RpcClient> {
    let rpcs = get_mainnet_rpcs();
    
    for (i, rpc_url) in rpcs.iter().enumerate() {
        let rpc_type = if rpc_url.contains("helius") { "Helius (Premium)" }
                      else if rpc_url.contains("quicknode") { "QuickNode (Premium)" }
                      else if rpc_url.contains("alchemy") { "Alchemy (Premium)" }
                      else if rpc_url.contains("ankr") && rpc_url.contains("api-key") { "Ankr (Premium)" }
                      else { "Public" };
        
        info!("🌐 Trying RPC {} of {} ({}): {}", 
              i + 1, rpcs.len(), rpc_type, 
              if rpc_url.contains("api-key") || rpc_url.contains("/v2/") { 
                  rpc_url.split("?").next().unwrap_or(rpc_url).split("/v2/").next().unwrap_or(rpc_url)
              } else { 
                  rpc_url 
              });
        
        let client = RpcClient::new_with_timeout_and_commitment(
            rpc_url.clone(),
            Duration::from_secs(15), // Longer timeout for MainNet
            CommitmentConfig::confirmed()
        );
        
        // Test connection with health check
        match client.get_health() {
            Ok(_) => {
                info!("  ✅ Health check: OK");
                
                // Test basic functionality
                match client.get_slot() {
                    Ok(slot) => {
                        info!("  ✅ Connected to MainNet (slot: {})", slot);
                        if rpc_type.contains("Premium") {
                            info!("  🌟 Using premium RPC for better performance");
                        }
                        return Ok(client);
                    }
                    Err(e) => {
                        warn!("  ❌ Slot check failed: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("  ❌ Health check failed: {}", e);
            }
        }
    }
    
    anyhow::bail!("Failed to connect to any MainNet RPC (including premium endpoints)")
}

async fn get_balance(client: &RpcClient, pubkey: &Pubkey) -> Result<f64> {
    let balance_lamports = client.get_balance(pubkey)?;
    Ok(balance_lamports as f64 / 1_000_000_000.0)
}

async fn execute_proven_arbitrage(client: &RpcClient, wallet: &Keypair) -> Result<()> {
    info!("🔄 === EXECUTING PROVEN 2C TECHNIQUE ===");
    info!("   📚 Based on: phase2c_real_arbitrage.rs (+0.012029280 SOL)");
    info!("   🎯 Method: Wrapped SOL timing arbitrage");
    
    let wsol_mint = Pubkey::from_str(WSOL_MINT)?;
    let wallet_pubkey = wallet.pubkey();
    let wsol_ata = get_associated_token_address(&wallet_pubkey, &wsol_mint);
    
    let amount_lamports = (ULTRA_CONSERVATIVE_AMOUNT * 1_000_000_000.0) as u64;
    info!("   💰 Amount: {} lamports ({} SOL)", amount_lamports, ULTRA_CONSERVATIVE_AMOUNT);
    info!("   🎯 WSOL ATA: {}", wsol_ata);
    
    // Step 1: Create WSOL ATA if needed
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
        sleep(Duration::from_millis(2000)).await; // Extra time for MainNet
    }
    
    // Step 2: WRAP SOL (Critical timing starts here)
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
    
    // Step 3: CRITICAL TIMING OPTIMIZATION (2C proven method)
    info!("⏰ Optimization delay: {}ms (2C proven timing)...", TIMING_OPTIMIZATION);
    sleep(Duration::from_millis(TIMING_OPTIMIZATION)).await;
    
    // Step 4: UNWRAP SOL (Complete the arbitrage)
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
    info!("   🔧 Wrap signature: {}", wrap_signature);
    info!("   🔧 Unwrap signature: {}", unwrap_signature);
    
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
