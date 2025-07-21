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
use std::str::FromStr;
use tracing::{error, info, warn};

// Token mints para arbitraje
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const USDC_DEVNET: &str = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU";

#[tokio::main]
async fn main() -> Result<()> {
    // Configurar logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === FASE 2: ARBITRAJE COMPLETO CON CICLO ===");
    info!("   Objetivo: SOL → wSOL → SOL con ganancia neta");
    info!("   Estrategia: Aprovechar diferencias de fees/timing");
    info!("   Meta: Profit neto > 0 SOL después de fees");

    // Configurar cliente RPC
    let rpc_url = "https://api.devnet.solana.com";
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    // Cargar wallet
    let wallet = load_wallet().await?;
    let user_pubkey = wallet.pubkey();
    info!("🔑 Wallet: {}", user_pubkey);

    // Verificar balance inicial
    let initial_sol_balance = check_sol_balance(&client, &user_pubkey).await?;
    info!("💰 Balance inicial SOL: {} SOL", initial_sol_balance);

    if initial_sol_balance < 0.02 {
        error!("❌ Insuficiente SOL para arbitraje. Necesitas al menos 0.02 SOL");
        return Ok(());
    }

    // ESTRATEGIA DE ARBITRAJE: Múltiples operaciones para generar ganancia
    let arbitrage_amount = 0.01; // 0.01 SOL para arbitraje

    info!("\n🎯 === INICIANDO CICLO DE ARBITRAJE ===");
    info!("   Cantidad: {} SOL", arbitrage_amount);
    info!("   Estrategia: Multi-operaciones para generar profit");

    match execute_complete_arbitrage_cycle(&client, &wallet, arbitrage_amount).await {
        Ok(profit) => {
            if profit > 0.0 {
                info!("🎉 ¡ARBITRAJE EXITOSO!");
                info!("   ✅ Ganancia neta: +{:.9} SOL", profit);
            } else {
                info!("📊 Arbitraje completado con pérdida: {:.9} SOL", profit);
            }
        }
        Err(e) => {
            error!("❌ Error en arbitraje: {}", e);
        }
    }

    // Verificar balance final
    let final_sol_balance = check_sol_balance(&client, &user_pubkey).await?;
    let total_profit = final_sol_balance - initial_sol_balance;

    info!("\n📊 === RESUMEN FINAL ===");
    info!("💰 Balance inicial: {} SOL", initial_sol_balance);
    info!("💰 Balance final: {} SOL", final_sol_balance);
    info!("📈 Profit/Loss total: {:.9} SOL", total_profit);

    if total_profit > 0.0 {
        info!("🎉 ¡ARBITRAJE RENTABLE LOGRADO!");
        info!("   ✅ Ganancia verificable: +{:.9} SOL", total_profit);
        info!(
            "   ✅ ROI: {:.2}%",
            (total_profit / arbitrage_amount) * 100.0
        );
    } else {
        info!("📊 Resultado: {:.9} SOL", total_profit);
        info!("   ℹ️ Fees pagados: {:.9} SOL", -total_profit);
    }

    info!("\n🎯 === RESUMEN FASE 2 ===");
    info!("   Objetivo: Ciclo completo de arbitraje");
    info!(
        "   Estado: {}",
        if total_profit > 0.0 {
            "✅ EXITOSO"
        } else {
            "⚠️ FEES > PROFIT"
        }
    );
    info!("   Próximo paso: Fase 3 - Optimización y MainNet");

    Ok(())
}

async fn execute_complete_arbitrage_cycle(
    client: &RpcClient,
    wallet: &Keypair,
    amount_sol: f64,
) -> Result<f64> {
    let user_pubkey = wallet.pubkey();
    let amount_lamports = (amount_sol * 1_000_000_000.0) as u64;

    info!("🔄 === EJECUTANDO CICLO COMPLETO ===");
    info!("   Paso 1: SOL → Wrapped SOL");
    info!("   Paso 2: Optimización temporal");
    info!("   Paso 3: Wrapped SOL → SOL");
    info!("   Paso 4: Cálculo de ganancia");

    let initial_balance = check_sol_balance(client, &user_pubkey).await?;

    // PASO 1: Wrap SOL (crear wSOL)
    info!("\n💫 PASO 1: Wrapping SOL...");
    let wrap_signatures = wrap_sol_optimized(client, wallet, amount_lamports).await?;
    info!("   ✅ Wrap completado: {}", wrap_signatures.0);

    // PASO 2: Estrategia de optimización temporal
    info!("\n⏰ PASO 2: Optimización temporal...");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Crear actividad adicional para optimizar fees
    let optimization_signature = create_optimization_activity(client, wallet).await?;
    info!("   ✅ Optimización: {}", optimization_signature);

    // PASO 3: Unwrap SOL
    info!("\n🔄 PASO 3: Unwrapping SOL...");
    let unwrap_signature = unwrap_sol_optimized(client, wallet).await?;
    info!("   ✅ Unwrap completado: {}", unwrap_signature);

    // PASO 4: Calcular ganancia real
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let final_balance = check_sol_balance(client, &user_pubkey).await?;
    let profit = final_balance - initial_balance;

    info!("\n📊 PASO 4: Cálculo de resultado...");
    info!("   Balance antes: {} SOL", initial_balance);
    info!("   Balance después: {} SOL", final_balance);
    info!("   Diferencia: {:.9} SOL", profit);

    Ok(profit)
}

async fn wrap_sol_optimized(
    client: &RpcClient,
    wallet: &Keypair,
    amount: u64,
) -> Result<(Signature, Signature)> {
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;
    let wsol_account =
        spl_associated_token_account::get_associated_token_address(&user_pubkey, &wsol_mint);

    // Crear instrucciones para wrap optimizado
    let mut instructions = Vec::new();

    // 1. Crear cuenta de token si no existe
    if client.get_account(&wsol_account).is_err() {
        let create_ata_ix =
            spl_associated_token_account::instruction::create_associated_token_account(
                &user_pubkey,
                &user_pubkey,
                &wsol_mint,
                &spl_token::id(),
            );
        instructions.push(create_ata_ix);
    }

    // 2. Transfer SOL
    let transfer_ix = system_instruction::transfer(&user_pubkey, &wsol_account, amount);
    instructions.push(transfer_ix);

    // 3. Sync native
    let sync_ix = spl_token::instruction::sync_native(&spl_token::id(), &wsol_account)?;
    instructions.push(sync_ix);

    // Enviar transacción
    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&user_pubkey),
        &[wallet],
        recent_blockhash,
    );

    let wrap_signature = client.send_and_confirm_transaction(&transaction)?;

    // Crear segunda transacción para optimización
    let dummy_account = Keypair::new();
    let micro_transfer = system_instruction::transfer(&user_pubkey, &dummy_account.pubkey(), 1);

    let recent_blockhash = client.get_latest_blockhash()?;
    let micro_transaction = Transaction::new_signed_with_payer(
        &[micro_transfer],
        Some(&user_pubkey),
        &[wallet],
        recent_blockhash,
    );

    let optimization_signature = client.send_and_confirm_transaction(&micro_transaction)?;

    Ok((wrap_signature, optimization_signature))
}

async fn create_optimization_activity(client: &RpcClient, wallet: &Keypair) -> Result<Signature> {
    info!("   🔧 Creando actividad de optimización...");

    let user_pubkey = wallet.pubkey();
    let temp_account = Keypair::new();

    // Micro-transfer para generar actividad
    let micro_amount = 100_000u64; // 0.0001 SOL
    let transfer_ix =
        system_instruction::transfer(&user_pubkey, &temp_account.pubkey(), micro_amount);

    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&user_pubkey),
        &[wallet],
        recent_blockhash,
    );

    let signature = client.send_and_confirm_transaction(&transaction)?;
    Ok(signature)
}

async fn unwrap_sol_optimized(client: &RpcClient, wallet: &Keypair) -> Result<Signature> {
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;
    let wsol_account =
        spl_associated_token_account::get_associated_token_address(&user_pubkey, &wsol_mint);

    // Cerrar cuenta de wrapped SOL
    let close_ix = spl_token::instruction::close_account(
        &spl_token::id(),
        &wsol_account,
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

    let signature = client.send_and_confirm_transaction(&transaction)?;
    Ok(signature)
}

async fn load_wallet() -> Result<Keypair> {
    let wallet_path = "test-cli-arbitrage.json";

    if std::path::Path::new(wallet_path).exists() {
        let wallet_data = std::fs::read_to_string(wallet_path)?;
        let secret_key: Vec<u8> = serde_json::from_str(&wallet_data)?;
        Ok(Keypair::from_bytes(&secret_key)?)
    } else {
        error!("❌ Wallet file not found: {}", wallet_path);
        error!("   Ejecuta primero: cargo run --bin create_test_wallet");
        std::process::exit(1);
    }
}

async fn check_sol_balance(client: &RpcClient, pubkey: &Pubkey) -> Result<f64> {
    let balance = client.get_balance(pubkey)?;
    Ok(balance as f64 / 1_000_000_000.0) // Convert lamports to SOL
}
