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

    info!("🔍 === DIAGNÓSTICO: ¿POR QUÉ NO HAY PROFIT? ===");
    info!("   Objetivo: Identificar qué cambió desde los éxitos 2C/2F");
    info!("   Método: Análisis detallado de condiciones actuales");

    let rpc_url = "https://api.devnet.solana.com";
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    let wallet = load_wallet().await?;
    let user_pubkey = wallet.pubkey();
    info!("🔑 Wallet: {}", user_pubkey);

    let balance = check_sol_balance(&client, &user_pubkey).await?;
    info!("💰 Balance actual: {} SOL", balance);

    // DIAGNÓSTICO 1: Verificar condiciones de red
    info!("\n🌐 === DIAGNÓSTICO 1: CONDICIONES DE RED ===");
    analyze_network_conditions(&client).await?;

    // DIAGNÓSTICO 2: Estado de ATA
    info!("\n🏠 === DIAGNÓSTICO 2: ESTADO DE ATA ===");
    analyze_ata_state(&client, &user_pubkey).await?;

    // DIAGNÓSTICO 3: Timing analysis con múltiples ventanas
    info!("\n⏰ === DIAGNÓSTICO 3: ANÁLISIS DE TIMING ===");
    analyze_timing_windows(&client, &wallet).await?;

    // DIAGNÓSTICO 4: Comparar con condiciones exitosas pasadas
    info!("\n📊 === DIAGNÓSTICO 4: COMPARACIÓN HISTÓRICA ===");
    compare_with_successful_conditions(&client).await?;

    // DIAGNÓSTICO 5: Probar con cantidad significativamente mayor
    info!("\n💰 === DIAGNÓSTICO 5: TEST CON CANTIDAD MAYOR ===");
    test_larger_amounts(&client, &wallet).await?;

    Ok(())
}

async fn analyze_network_conditions(client: &RpcClient) -> Result<()> {
    info!("   🔍 Analizando condiciones actuales de DevNet...");

    // Slot rate y timing
    let slot1 = client.get_slot()?;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let slot2 = client.get_slot()?;
    let slot_rate = slot2 - slot1;

    info!("     📈 Slot rate: {} slots/segundo", slot_rate);

    // Network congestion indicators
    let recent_performance_samples = client.get_recent_performance_samples(Some(5))?;
    if let Some(latest) = recent_performance_samples.first() {
        info!(
            "     ⚡ Samples: {} slots, {} transactions",
            latest.num_slots, latest.num_transactions
        );
        info!(
            "     📊 TPS promedio: {:.2}",
            latest.num_transactions as f64 / latest.num_slots as f64
        );
    }

    // Rent exemption costs
    let rent_exempt = client.get_minimum_balance_for_rent_exemption(165)?;
    info!(
        "     🏠 Rent exempt actual: {} lamports ({:.9} SOL)",
        rent_exempt,
        rent_exempt as f64 / LAMPORTS_PER_SOL as f64
    );

    // Fee calculator
    let recent_blockhash = client.get_latest_blockhash()?;
    info!("     🔗 Blockhash reciente: {}", recent_blockhash);

    Ok(())
}

async fn analyze_ata_state(client: &RpcClient, user_pubkey: &Pubkey) -> Result<()> {
    info!("   🔍 Analizando estado actual de ATA...");

    let wsol_mint = Pubkey::from_str(SOL_MINT)?;
    let wsol_account =
        spl_associated_token_account::get_associated_token_address(user_pubkey, &wsol_mint);

    info!("     🏠 ATA Address: {}", wsol_account);

    match client.get_account(&wsol_account) {
        Ok(account) => {
            info!("     ✅ ATA EXISTS");
            info!("     💰 Lamports: {}", account.lamports);
            info!("     👤 Owner: {}", account.owner);
            info!("     📊 Data length: {}", account.data.len());

            // Verificar si tiene balance de tokens
            match client.get_token_account_balance(&wsol_account) {
                Ok(token_balance) => {
                    info!(
                        "     🪙 Token balance: {} ({})",
                        token_balance.ui_amount_string, token_balance.amount
                    );
                }
                Err(_) => {
                    info!("     ⚠️ No token balance data");
                }
            }
        }
        Err(_) => {
            info!("     ❌ ATA NO EXISTS - Esto puede ser la clave");
            info!("     💡 Los éxitos pasados pueden haber sido con ATA limpio");
        }
    }

    Ok(())
}

async fn analyze_timing_windows(client: &RpcClient, wallet: &Keypair) -> Result<()> {
    info!("   🔍 Probando múltiples ventanas de timing...");

    let timing_windows = vec![400, 600, 800, 1000, 1200, 1500];
    let user_pubkey = wallet.pubkey();

    for (index, timing_ms) in timing_windows.iter().enumerate() {
        info!("     ⏰ Probando timing: {}ms", timing_ms);

        let balance_before = client.get_balance(&user_pubkey)?;

        // Mini arbitraje con timing específico
        match execute_timing_test(client, wallet, *timing_ms).await {
            Ok(_) => {
                let balance_after = client.get_balance(&user_pubkey)?;
                let change = balance_after as i64 - balance_before as i64;
                let change_sol = change as f64 / LAMPORTS_PER_SOL as f64;

                info!("       📊 Resultado {}ms: {:.9} SOL", timing_ms, change_sol);

                if change_sol > 0.0 {
                    info!("       🎉 ¡TIMING GANADOR ENCONTRADO!: {}ms", timing_ms);
                    return Ok(());
                }
            }
            Err(e) => {
                error!("       ❌ Error con timing {}ms: {}", timing_ms, e);
            }
        }

        // Pausa entre tests
        if index < timing_windows.len() - 1 {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    Ok(())
}

async fn execute_timing_test(client: &RpcClient, wallet: &Keypair, timing_ms: u64) -> Result<()> {
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;

    let wsol_account =
        spl_associated_token_account::get_associated_token_address(&user_pubkey, &wsol_mint);

    let amount = 8_000_000u64; // 0.008 SOL para test rápido
    let rent_exempt = client.get_minimum_balance_for_rent_exemption(165)?;

    // Clean ATA if exists
    if client.get_account(&wsol_account).is_ok() {
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
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    // Wrap
    let wrap_sig = execute_clean_wrap(client, wallet, &wsol_account, amount + rent_exempt).await?;

    // Timing test
    tokio::time::sleep(std::time::Duration::from_millis(timing_ms)).await;

    // Unwrap
    let _unwrap_sig = execute_clean_unwrap(client, wallet, &wsol_account).await?;

    Ok(())
}

async fn compare_with_successful_conditions(client: &RpcClient) -> Result<()> {
    info!("   🔍 Comparando con condiciones exitosas pasadas...");

    info!("     📊 ÉXITOS PASADOS:");
    info!("       🏆 Fase 2C: +0.012029280 SOL (16 julio ~17:04)");
    info!("       🏆 Fase 2F: +0.002029280 SOL (16 julio ~17:30)");

    // Calcular tiempo transcurrido
    info!("     ⏰ TIEMPO TRANSCURRIDO: ~1.5 horas");
    info!("     🌐 POSIBLES CAMBIOS EN DEVNET:");
    info!("       - Network congestion diferente");
    info!("       - Validator performance changes");
    info!("       - Timing characteristics modifications");

    // Verificar si hay cambios en fees básicos
    let transfer_fee = 5000u64; // Fee típico
    info!("     💸 Fee estimate actual: {} lamports", transfer_fee);

    let rent_exempt = client.get_minimum_balance_for_rent_exemption(165)?;
    info!("     🏠 Rent exempt: {} lamports", rent_exempt);

    // Total costs vs éxitos pasados
    let total_cost = transfer_fee * 2 + rent_exempt; // 2 transacciones + rent
    info!(
        "     💰 Costo total estimado: {} lamports ({:.9} SOL)",
        total_cost,
        total_cost as f64 / LAMPORTS_PER_SOL as f64
    );

    info!("     🎯 PARA PROFIT NECESITAMOS:");
    info!("       - Timing que genere >0.000020000 SOL benefit");
    info!("       - O cantidad mayor que amplifica el benefit");

    Ok(())
}

async fn test_larger_amounts(client: &RpcClient, wallet: &Keypair) -> Result<()> {
    info!("   🔍 Probando con cantidades significativamente mayores...");

    let user_pubkey = wallet.pubkey();
    let balance = client.get_balance(&user_pubkey)?;
    let balance_sol = balance as f64 / LAMPORTS_PER_SOL as f64;

    info!("     💰 Balance disponible: {:.6} SOL", balance_sol);

    // Probar con cantidades que pueden amplificar cualquier benefit
    let test_amounts = vec![0.025, 0.05, 0.1]; // Cantidades mayores

    for amount_sol in test_amounts {
        if balance_sol < amount_sol + 0.01 {
            // Necesitamos margen para fees
            warn!(
                "     ⚠️ Balance insuficiente para probar {:.3} SOL",
                amount_sol
            );
            continue;
        }

        info!("     💫 Probando con {:.3} SOL...", amount_sol);

        let balance_before = client.get_balance(&user_pubkey)?;

        match execute_large_amount_test(client, wallet, amount_sol).await {
            Ok(_) => {
                let balance_after = client.get_balance(&user_pubkey)?;
                let change = balance_after as i64 - balance_before as i64;
                let change_sol = change as f64 / LAMPORTS_PER_SOL as f64;

                info!(
                    "       📊 Resultado {:.3} SOL: {:.9} SOL",
                    amount_sol, change_sol
                );

                if change_sol > 0.0 {
                    info!(
                        "       🎉 ¡CANTIDAD GANADORA!: {:.3} SOL genera profit",
                        amount_sol
                    );
                    info!("       🎯 Usar esta cantidad para futuros arbitrajes");
                    break; // Encontramos una cantidad que funciona
                }
            }
            Err(e) => {
                error!("       ❌ Error con {:.3} SOL: {}", amount_sol, e);
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }

    Ok(())
}

async fn execute_large_amount_test(
    client: &RpcClient,
    wallet: &Keypair,
    amount_sol: f64,
) -> Result<()> {
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;

    let wsol_account =
        spl_associated_token_account::get_associated_token_address(&user_pubkey, &wsol_mint);

    let amount_lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
    let rent_exempt = client.get_minimum_balance_for_rent_exemption(165)?;

    // Clean state
    if client.get_account(&wsol_account).is_ok() {
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
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    }

    // Wrap with larger amount
    let _wrap_sig =
        execute_clean_wrap(client, wallet, &wsol_account, amount_lamports + rent_exempt).await?;

    // Optimal timing from previous successes
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;

    // Unwrap
    let _unwrap_sig = execute_clean_unwrap(client, wallet, &wsol_account).await?;

    Ok(())
}

async fn execute_clean_wrap(
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

async fn execute_clean_unwrap(
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
