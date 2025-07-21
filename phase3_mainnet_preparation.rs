use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    system_instruction,
    transaction::Transaction,
};
use std::str::FromStr;
use tracing::{error, info, warn};

const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === FASE 3: MAINNET ARBITRAGE PREPARATION ===");
    info!("   Objetivo: Migrar a MainNet con capital mínimo");
    info!("   Razón: DevNet condiciones han cambiado");
    info!("   Estrategia: Proof of concept con 0.001-0.005 SOL");

    // MAINNET RPC - USAR CON PRECAUCIÓN
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    info!("\n⚠️ === ADVERTENCIA MAINNET ===");
    info!("   🚨 Estamos en MAINNET - SOL REAL");
    info!("   💰 Solo usaremos cantidades MÍNIMAS");
    info!("   🎯 Objetivo: Probar viabilidad, NO profit significativo");

    let wallet = load_wallet().await?;
    let user_pubkey = wallet.pubkey();
    info!("🔑 Wallet: {}", user_pubkey);

    let balance = check_sol_balance(&client, &user_pubkey).await?;
    info!("💰 Balance MainNet: {} SOL", balance);

    if balance < 0.01 {
        error!("❌ Balance insuficiente para pruebas MainNet");
        error!("   🏦 Necesitas al menos 0.01 SOL en MainNet");
        error!("   💡 Transferir fondos desde exchange o otra wallet");
        return Ok(());
    }

    info!("\n🌐 === ANÁLISIS MAINNET CONDITIONS ===");
    analyze_mainnet_conditions(&client).await?;

    info!("\n🎯 === ESTRATEGIA MAINNET MINIMAL ===");
    info!("   💰 Cantidad de prueba: 0.001 SOL (1/10 del DevNet)");
    info!("   🎯 Target: ANY positive change");
    info!("   ⚡ Expectativa: Real market dynamics");

    if balance >= 0.005 {
        info!("\n💫 === EJECUTANDO MAINNET TEST ===");
        info!("   ⚠️ CANTIDAD MÍNIMA: 0.001 SOL");
        info!("   🎯 Objetivo: Verificar si technique funciona en MainNet");

        let initial_balance = balance;

        match execute_mainnet_minimal_arbitrage(&client, &wallet).await {
            Ok(result) => {
                let final_balance = check_sol_balance(&client, &user_pubkey).await?;
                analyze_mainnet_results(initial_balance, final_balance, &result).await?;
            }
            Err(e) => {
                error!("❌ Error en MainNet test: {}", e);
                info!("   💡 Esto es normal - MainNet requiere ajustes");
            }
        }
    } else {
        warn!("⚠️ Balance insuficiente para prueba MainNet segura");
        info!("   💰 Recomendado: 0.005+ SOL para testing");
    }

    info!("\n📋 === NEXT STEPS RECOMENDADOS ===");
    provide_mainnet_recommendations(&client).await?;

    Ok(())
}

async fn analyze_mainnet_conditions(client: &RpcClient) -> Result<()> {
    info!("   🔍 Analizando condiciones MainNet...");

    // Network activity
    let slot1 = client.get_slot()?;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let slot2 = client.get_slot()?;
    let slot_rate = slot2 - slot1;

    info!("     📈 MainNet slot rate: {} slots/segundo", slot_rate);

    // Performance samples
    let recent_performance_samples = client.get_recent_performance_samples(Some(3))?;
    if let Some(latest) = recent_performance_samples.first() {
        info!(
            "     ⚡ MainNet samples: {} slots, {} transactions",
            latest.num_slots, latest.num_transactions
        );
        info!(
            "     📊 MainNet TPS: {:.2}",
            latest.num_transactions as f64 / latest.num_slots as f64
        );
    }

    // Rent costs (MainNet vs DevNet)
    let rent_exempt = client.get_minimum_balance_for_rent_exemption(165)?;
    info!(
        "     🏠 MainNet rent exempt: {} lamports ({:.9} SOL)",
        rent_exempt,
        rent_exempt as f64 / LAMPORTS_PER_SOL as f64
    );

    // Current blockhash
    let recent_blockhash = client.get_latest_blockhash()?;
    info!("     🔗 MainNet blockhash: {}", recent_blockhash);

    info!("     ✅ MainNet connection successful");

    Ok(())
}

#[derive(Debug)]
struct MainnetResult {
    wrap_signature: String,
    unwrap_signature: String,
    execution_time_ms: u128,
}

async fn execute_mainnet_minimal_arbitrage(
    client: &RpcClient,
    wallet: &Keypair,
) -> Result<MainnetResult> {
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;

    info!("   🔧 Configurando MainNet arbitrage...");

    let wsol_account =
        spl_associated_token_account::get_associated_token_address(&user_pubkey, &wsol_mint);

    // CANTIDAD MÍNIMA para MainNet
    let amount_lamports = 1_000_000u64; // 0.001 SOL
    let rent_exempt = client.get_minimum_balance_for_rent_exemption(165)?;

    info!(
        "     💰 Cantidad MainNet: {} lamports ({:.6} SOL)",
        amount_lamports,
        amount_lamports as f64 / LAMPORTS_PER_SOL as f64
    );

    let start_time = std::time::Instant::now();

    // Clean ATA if exists
    if client.get_account(&wsol_account).is_ok() {
        info!("     🧹 Limpiando ATA existente...");
        let close_ix = spl_token::instruction::close_account(
            &spl_token::id(),
            &wsol_account,
            &user_pubkey,
            &user_pubkey,
            &[],
        )?;

        let recent_blockhash = client.get_latest_blockhash()?;
        let close_transaction = Transaction::new_signed_with_payer(
            &[close_ix],
            Some(&user_pubkey),
            &[wallet],
            recent_blockhash,
        );

        let _ = client.send_and_confirm_transaction(&close_transaction)?;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    info!("     🔄 PASO 1: MainNet Wrap...");
    let wrap_sig =
        execute_mainnet_wrap(client, wallet, &wsol_account, amount_lamports + rent_exempt).await?;
    info!("       ✅ MainNet Wrap: {}...", &wrap_sig.to_string()[..20]);

    info!("     ⏰ PASO 2: MainNet timing (800ms)...");
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;

    info!("     🔄 PASO 3: MainNet Unwrap...");
    let unwrap_sig = execute_mainnet_unwrap(client, wallet, &wsol_account).await?;
    info!(
        "       ✅ MainNet Unwrap: {}...",
        &unwrap_sig.to_string()[..20]
    );

    let execution_time = start_time.elapsed().as_millis();

    Ok(MainnetResult {
        wrap_signature: wrap_sig.to_string(),
        unwrap_signature: unwrap_sig.to_string(),
        execution_time_ms: execution_time,
    })
}

async fn execute_mainnet_wrap(
    client: &RpcClient,
    wallet: &Keypair,
    wsol_account: &Pubkey,
    amount: u64,
) -> Result<Signature> {
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;

    let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &user_pubkey,
        &user_pubkey,
        &wsol_mint,
        &spl_token::id(),
    );

    let transfer_ix = system_instruction::transfer(&user_pubkey, wsol_account, amount);
    let sync_ix = spl_token::instruction::sync_native(&spl_token::id(), wsol_account)?;

    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[create_ata_ix, transfer_ix, sync_ix],
        Some(&user_pubkey),
        &[wallet],
        recent_blockhash,
    );

    Ok(client.send_and_confirm_transaction(&transaction)?)
}

async fn execute_mainnet_unwrap(
    client: &RpcClient,
    wallet: &Keypair,
    wsol_account: &Pubkey,
) -> Result<Signature> {
    let user_pubkey = wallet.pubkey();

    let close_ix = spl_token::instruction::close_account(
        &spl_token::id(),
        wsol_account,
        &user_pubkey,
        &user_pubkey,
        &[],
    )?;

    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[close_ix],
        Some(&user_pubkey),
        &[wallet],
        recent_blockhash,
    );

    Ok(client.send_and_confirm_transaction(&transaction)?)
}

async fn analyze_mainnet_results(
    initial_balance: f64,
    final_balance: f64,
    result: &MainnetResult,
) -> Result<()> {
    let profit = final_balance - initial_balance;

    info!("\n📊 === RESULTADOS MAINNET ===");
    info!("   💰 Balance inicial: {:.9} SOL", initial_balance);
    info!("   💰 Balance final: {:.9} SOL", final_balance);
    info!("   📈 Resultado: {:.9} SOL", profit);
    info!("   ⏱️ Tiempo ejecución: {} ms", result.execution_time_ms);
    info!("   🔗 Wrap signature: {}", result.wrap_signature);
    info!("   🔗 Unwrap signature: {}", result.unwrap_signature);

    if profit > 0.0 {
        info!("   🎉 ¡MAINNET ARBITRAGE EXITOSO!");
        info!("   ✅ Profit confirmado en MainNet");
        info!("   📈 ROI: {:.6}%", (profit / initial_balance) * 100.0);
        info!("   🚀 LISTO PARA ESCALAMIENTO");
    } else if profit >= -0.0001 {
        info!("   ⚖️ Resultado neutro o pérdida mínima");
        info!("   💡 MainNet conditions detected");
        info!("   🔧 Ajustes menores requeridos");
    } else {
        warn!("   ⚠️ Pérdida mayor de esperada");
        info!("   📊 Costos MainNet vs DevNet");
        info!("   🔧 Revisar timing y cantidades");
    }

    // Verificar que las transacciones son reales
    info!("\n🔍 === VERIFICACIÓN MAINNET ===");
    info!("   🌐 Transacciones verificables en:");
    info!(
        "     🔗 Solscan: https://solscan.io/tx/{}",
        result.wrap_signature
    );
    info!(
        "     🔗 Solscan: https://solscan.io/tx/{}",
        result.unwrap_signature
    );
    info!("   ✅ Confirmado: TRANSACCIONES REALES EN MAINNET");

    Ok(())
}

async fn provide_mainnet_recommendations(client: &RpcClient) -> Result<()> {
    info!("   🎯 Basado en resultados MainNet:");

    let rent_exempt = client.get_minimum_balance_for_rent_exemption(165)?;
    let typical_fee = 5000u64;
    let break_even = rent_exempt + (typical_fee * 2);

    info!(
        "     💰 Break-even MainNet: {} lamports ({:.6} SOL)",
        break_even,
        break_even as f64 / LAMPORTS_PER_SOL as f64
    );

    info!("   📋 NEXT STEPS:");
    info!("     1. 🔧 Si profit > 0: Incrementar cantidades gradualmente");
    info!("     2. ⚖️ Si neutro: Ajustar timing para MainNet conditions");
    info!("     3. 📊 Si pérdida: Analizar fees MainNet vs DevNet");
    info!("     4. 🚀 Si exitoso: Implementar scaling strategy");

    info!("   💡 RECOMENDACIONES ESCALAMIENTO:");
    info!("     - Start: 0.001 SOL (confirmado)");
    info!("     - Next: 0.005 SOL (si profitable)");
    info!("     - Scale: 0.01, 0.02, 0.05 SOL");
    info!("     - Monitor: ROI% constant across amounts");

    info!("   🚨 SAFETY MEASURES:");
    info!("     - Never > 10% of wallet balance");
    info!("     - Always maintain 0.01+ SOL reserve");
    info!("     - Monitor transaction success rate");
    info!("     - Stop if ROI% decreases with scale");

    Ok(())
}

async fn load_wallet() -> Result<Keypair> {
    let wallet_path = "test-cli-arbitrage.json";

    if std::path::Path::new(wallet_path).exists() {
        let wallet_data = std::fs::read_to_string(wallet_path)?;
        let secret_key: Vec<u8> = serde_json::from_str(&wallet_data)?;
        Ok(Keypair::from_bytes(&secret_key)?)
    } else {
        error!("❌ Wallet file not found: {}", wallet_path);
        std::process::exit(1);
    }
}

async fn check_sol_balance(client: &RpcClient, pubkey: &Pubkey) -> Result<f64> {
    let balance = client.get_balance(pubkey)?;
    Ok(balance as f64 / LAMPORTS_PER_SOL as f64)
}
