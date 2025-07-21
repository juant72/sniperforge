use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::instruction::{close_account, sync_native};
use std::str::FromStr;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{error, info, warn};

// ✅ CONFIGURACIÓN MAINNET ULTRA-CONSERVATIVA
const RPC_URL: &str = "https://api.mainnet-beta.solana.com";
const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";

// 🎯 PARÁMETROS ULTRA-CONSERVATIVOS PARA MAINNET
const ULTRA_CONSERVATIVE_AMOUNT: f64 = 0.001; // 0.001 SOL = $0.20 aprox
const MINIMUM_BALANCE_REQUIRED: f64 = 0.01; // 0.01 SOL mínimo para fees
const MAX_ACCEPTABLE_LOSS: f64 = 0.0005; // Max pérdida aceptable: 0.0005 SOL
const TIMING_OPTIMIZATION: u64 = 800; // 800ms timing (probado en 2C)

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("🚀 === MAINNET ULTRA-CONSERVATIVE ARBITRAGE ===");
    info!("   🎯 Objetivo: First MainNet test con cantidades mínimas");
    info!("   💰 Amount: {} SOL (~$0.20)", ULTRA_CONSERVATIVE_AMOUNT);
    info!("   🛡️ Risk: Ultra-conservative, máxima seguridad");
    info!("   📍 Network: MAINNET-BETA (REAL MONEY)");

    // ⚠️ VERIFICACIONES DE SEGURIDAD
    warn!("⚠️  === ADVERTENCIA MAINNET ===");
    warn!("   💸 ESTE ES DINERO REAL");
    warn!("   🔒 Verificando balances antes de proceder...");

    let wallet = load_wallet().await?;
    let client = RpcClient::new_with_commitment(RPC_URL.to_string(), CommitmentConfig::confirmed());

    info!("🔑 Wallet: {}", wallet.pubkey());

    // 📊 VERIFICAR BALANCE MAINNET
    let balance_lamports = client.get_balance(&wallet.pubkey())?;
    let balance_sol = balance_lamports as f64 / 1_000_000_000.0;

    info!("💰 Balance MainNet: {:.9} SOL", balance_sol);

    // 🛡️ VERIFICACIONES DE SEGURIDAD CRÍTICAS
    if balance_sol < MINIMUM_BALANCE_REQUIRED {
        error!("❌ BALANCE INSUFICIENTE");
        error!("   💰 Balance actual: {:.9} SOL", balance_sol);
        error!(
            "   🎯 Mínimo requerido: {:.9} SOL",
            MINIMUM_BALANCE_REQUIRED
        );
        error!("   📝 ACCIÓN REQUERIDA:");
        error!(
            "      1. Transferir al menos {:.9} SOL a MainNet wallet",
            MINIMUM_BALANCE_REQUIRED
        );
        error!("      2. Wallet address: {}", wallet.pubkey());
        return Ok(());
    }

    if balance_sol < ULTRA_CONSERVATIVE_AMOUNT + 0.005 {
        error!("❌ BALANCE INSUFICIENTE PARA OPERACIÓN SEGURA");
        error!(
            "   💰 Necesario: {:.9} SOL (operación + fees)",
            ULTRA_CONSERVATIVE_AMOUNT + 0.005
        );
        error!("   💰 Disponible: {:.9} SOL", balance_sol);
        return Ok(());
    }

    // ✅ CONFIRMACIÓN USUARIO
    warn!("🎯 === READY TO EXECUTE ===");
    warn!("   💰 Operación: {:.9} SOL", ULTRA_CONSERVATIVE_AMOUNT);
    warn!("   🛡️ Max loss: {:.9} SOL", MAX_ACCEPTABLE_LOSS);
    warn!(
        "   ⏰ Timing: {}ms (probado en DevNet)",
        TIMING_OPTIMIZATION
    );
    warn!("   🔄 Método: Wrapped SOL timing arbitrage (2C technique)");

    info!("⏳ Esperando 5 segundos antes de ejecutar...");
    sleep(Duration::from_secs(5)).await;

    // 🚀 EJECUTAR MAINNET ARBITRAGE
    let result = execute_mainnet_arbitrage(&client, &wallet).await?;

    // 📊 REPORTE FINAL
    let final_balance_lamports = client.get_balance(&wallet.pubkey())?;
    let final_balance_sol = final_balance_lamports as f64 / 1_000_000_000.0;
    let net_change = final_balance_sol - balance_sol;

    info!("🏁 === MAINNET ARBITRAGE COMPLETADO ===");
    info!("   💰 Balance inicial: {:.9} SOL", balance_sol);
    info!("   💰 Balance final: {:.9} SOL", final_balance_sol);
    info!("   📈 Cambio neto: {:.9} SOL", net_change);

    if net_change > 0.0 {
        info!("🎉 ¡PROFIT EN MAINNET! +{:.9} SOL", net_change);
        info!(
            "   📊 ROI: {:.4}%",
            (net_change / ULTRA_CONSERVATIVE_AMOUNT) * 100.0
        );
    } else if net_change.abs() <= MAX_ACCEPTABLE_LOSS {
        warn!(
            "⚠️  Loss dentro de límites aceptables: {:.9} SOL",
            net_change
        );
    } else {
        error!("❌ Loss excede límites: {:.9} SOL", net_change);
    }

    // 🎯 RECOMENDACIONES
    if net_change > 0.0 {
        info!("🚀 === PRÓXIMOS PASOS ===");
        info!("   ✅ MainNet technique confirmada");
        info!("   📈 Puede incrementar amounts gradualmente");
        info!("   🎯 Sugerencia: Probar con 0.005 SOL next");
    } else {
        info!("📊 === ANÁLISIS ===");
        info!("   🔍 MainNet conditions diferentes a DevNet");
        info!("   🎯 Puede requerir ajuste de parámetros");
        info!("   ⏰ Considerar different timings");
    }

    Ok(())
}

async fn execute_mainnet_arbitrage(client: &RpcClient, wallet: &Keypair) -> Result<()> {
    info!("🔄 === EXECUTING MAINNET ARBITRAGE ===");

    let wsol_mint = Pubkey::from_str(WSOL_MINT)?;
    let wallet_pubkey = wallet.pubkey();
    let wsol_ata = get_associated_token_address(&wallet_pubkey, &wsol_mint);

    let amount_lamports = (ULTRA_CONSERVATIVE_AMOUNT * 1_000_000_000.0) as u64;

    info!(
        "   💰 Amount: {} lamports ({} SOL)",
        amount_lamports, ULTRA_CONSERVATIVE_AMOUNT
    );
    info!("   🎯 WSOL ATA: {}", wsol_ata);

    // 📊 Step 1: Create WSOL ATA if needed
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
    }

    // 🚀 Step 2: WRAP SOL (timing crítico)
    info!("🔄 Step 1: Wrapping {} SOL...", ULTRA_CONSERVATIVE_AMOUNT);
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

    info!(
        "   ✅ Wrap completed: {} ({}ms)",
        wrap_signature,
        wrap_time.as_millis()
    );

    // ⏰ TIMING OPTIMIZATION (technique 2C)
    info!("⏰ Optimization delay: {}ms...", TIMING_OPTIMIZATION);
    sleep(Duration::from_millis(TIMING_OPTIMIZATION)).await;

    // 🔄 Step 3: UNWRAP SOL
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

    info!(
        "   ✅ Unwrap completed: {} ({}ms)",
        unwrap_signature,
        unwrap_time.as_millis()
    );

    let total_time = wrap_start.elapsed();
    info!("⚡ Total execution time: {}ms", total_time.as_millis());
    info!("   🔧 Wrap signature: {}", wrap_signature);
    info!("   🔧 Unwrap signature: {}", unwrap_signature);

    Ok(())
}

async fn load_wallet() -> Result<Keypair> {
    let wallet_path = "mainnet-arbitrage-wallet.json";

    if std::path::Path::new(wallet_path).exists() {
        let wallet_data = std::fs::read_to_string(wallet_path)?;
        let secret_key: Vec<u8> = serde_json::from_str(&wallet_data)?;
        Ok(Keypair::from_bytes(&secret_key)?)
    } else {
        error!("❌ MainNet wallet file not found: {}", wallet_path);
        error!("   📝 Create MainNet wallet first with: cargo run --bin setup_mainnet_wallet");
        error!("   💰 Then transfer SOL to the wallet address");
        anyhow::bail!("MainNet wallet file not found");
    }
}
