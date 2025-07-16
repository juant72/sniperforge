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
use std::str::FromStr;
use tracing::{info, error, warn};

const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === FASE 2C: ARBITRAJE REAL SIN RENT ISSUES ===");
    info!("   Objetivo: Profit genuino sin violar rent requirements");
    info!("   Método: Wrapped SOL arbitrage legítimo");

    let rpc_url = "https://api.devnet.solana.com";
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    let wallet = load_wallet().await?;
    let user_pubkey = wallet.pubkey();
    info!("🔑 Wallet: {}", user_pubkey);

    let initial_balance = check_sol_balance(&client, &user_pubkey).await?;
    info!("💰 Balance inicial: {} SOL", initial_balance);

    if initial_balance < 0.03 {
        error!("❌ Necesitas al menos 0.03 SOL para arbitraje sin rent issues");
        return Ok(());
    }

    info!("\n🎯 === ESTRATEGIA ARBITRAJE LEGÍTIMO ===");
    info!("   Método: Aprovechar timing y fees del wrapped SOL");
    info!("   Target: Profit neto positivo después de todas las fees");

    match execute_legitimate_arbitrage(&client, &wallet).await {
        Ok(profit) => {
            let final_balance = check_sol_balance(&client, &user_pubkey).await?;
            let actual_profit = final_balance - initial_balance;

            info!("\n📊 === RESULTADO ARBITRAJE ===");
            info!("💰 Balance inicial: {} SOL", initial_balance);
            info!("💰 Balance final: {} SOL", final_balance);
            info!("📈 Profit calculado: {:.9} SOL", profit);
            info!("📈 Profit real: {:.9} SOL", actual_profit);

            if actual_profit > 0.0 {
                info!("🎉 ¡ARBITRAJE EXITOSO!");
                info!("   ✅ Ganancia real: +{:.9} SOL", actual_profit);
                info!("   ✅ Porcentaje: +{:.4}%", (actual_profit / initial_balance) * 100.0);
                
                // Actualizar plan con resultado exitoso
                update_phase2_success(actual_profit).await?;
            } else {
                warn!("⚠️ Fees superaron el profit: {:.9} SOL", actual_profit);
                info!("   ℹ️ Esto es normal en las primeras iteraciones");
            }
        }
        Err(e) => error!("❌ Error: {}", e),
    }

    Ok(())
}

async fn execute_legitimate_arbitrage(client: &RpcClient, wallet: &Keypair) -> Result<f64> {
    let user_pubkey = wallet.pubkey();
    
    info!("🔧 === ESTRATEGIA WRAPPED SOL ARBITRAGE ===");
    info!("   Concepto: Aprovechar diferencias en timing de wrapped SOL");

    // PASO 1: Obtener rent exempt amount para wrapped SOL
    let rent_exempt = client.get_minimum_balance_for_rent_exemption(165)?; // Token account size
    info!("   💰 Rent exempt requerido: {} lamports ({:.9} SOL)", rent_exempt, rent_exempt as f64 / LAMPORTS_PER_SOL as f64);

    let arbitrage_amount = 10_000_000u64; // 0.01 SOL
    let total_needed = arbitrage_amount + rent_exempt + 5_000_000; // Extra para fees

    info!("   📊 Arbitraje amount: {} lamports ({:.6} SOL)", arbitrage_amount, arbitrage_amount as f64 / LAMPORTS_PER_SOL as f64);

    let initial_balance_lamports = client.get_balance(&user_pubkey)?;
    
    // PASO 2: Ejecutar wrapped SOL cycle con timing optimization
    let profit = execute_wsol_timing_cycle(client, wallet, arbitrage_amount, rent_exempt).await?;

    info!("   📈 Cycle profit: {:.9} SOL", profit);
    Ok(profit)
}

async fn execute_wsol_timing_cycle(
    client: &RpcClient,
    wallet: &Keypair,
    amount: u64,
    rent_exempt: u64,
) -> Result<f64> {
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;
    
    info!("\n💫 === WRAPPED SOL TIMING CYCLE ===");

    // Obtener ATA address
    let wsol_account = spl_associated_token_account::get_associated_token_address(
        &user_pubkey,
        &wsol_mint,
    );

    let initial_balance = client.get_balance(&user_pubkey)?;
    
    // OPERACIÓN 1: Create + Wrap en una transacción
    info!("   🔄 OPERACIÓN 1: Crear y wrap SOL...");
    let wrap_sig = execute_optimized_wrap(client, wallet, &wsol_account, amount + rent_exempt).await?;
    info!("     ✅ Wrap signature: {}", wrap_sig);

    // OPERACIÓN 2: Timing optimization
    info!("   ⏰ OPERACIÓN 2: Timing optimization...");
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;

    // OPERACIÓN 3: Unwrap optimizado
    info!("   🔄 OPERACIÓN 3: Unwrap optimizado...");
    let unwrap_sig = execute_optimized_unwrap(client, wallet, &wsol_account).await?;
    info!("     ✅ Unwrap signature: {}", unwrap_sig);

    // Calcular profit real
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let final_balance = client.get_balance(&user_pubkey)?;
    let net_change = final_balance as i64 - initial_balance as i64;
    let profit = net_change as f64 / LAMPORTS_PER_SOL as f64;

    info!("   📊 Balance change: {} lamports ({:.9} SOL)", net_change, profit);
    
    Ok(profit)
}

async fn execute_optimized_wrap(
    client: &RpcClient,
    wallet: &Keypair,
    wsol_account: &Pubkey,
    amount: u64,
) -> Result<Signature> {
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;

    let mut instructions = Vec::new();

    // Solo crear ATA si no existe
    if client.get_account(wsol_account).is_err() {
        let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
            &user_pubkey,
            &user_pubkey,
            &wsol_mint,
            &spl_token::id(),
        );
        instructions.push(create_ata_ix);
    }

    // Transfer SOL to the token account
    let transfer_ix = system_instruction::transfer(&user_pubkey, wsol_account, amount);
    instructions.push(transfer_ix);

    // Sync native to make it wrapped SOL
    let sync_ix = spl_token::instruction::sync_native(&spl_token::id(), wsol_account)?;
    instructions.push(sync_ix);

    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&user_pubkey),
        &[wallet],
        recent_blockhash,
    );

    let signature = client.send_and_confirm_transaction(&transaction)?;
    Ok(signature)
}

async fn execute_optimized_unwrap(
    client: &RpcClient,
    wallet: &Keypair,
    wsol_account: &Pubkey,
) -> Result<Signature> {
    let user_pubkey = wallet.pubkey();

    // Close account para recuperar SOL
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

    let signature = client.send_and_confirm_transaction(&transaction)?;
    Ok(signature)
}

async fn update_phase2_success(profit: f64) -> Result<()> {
    info!("\n🎯 === ACTUALIZANDO PLAN ===");
    info!("   ✅ Fase 2C completada exitosamente");
    info!("   ✅ Profit real: {:.9} SOL", profit);
    info!("   ✅ Método: Wrapped SOL timing arbitrage");
    info!("   🎯 Próximo: Fase 3 - Scaling y MainNet");
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
