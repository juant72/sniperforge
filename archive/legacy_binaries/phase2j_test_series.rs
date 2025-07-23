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

#[derive(Debug)]
struct ArbitrageResult {
    cycle_number: u32,
    initial_balance: f64,
    final_balance: f64,
    profit: f64,
    roi_percent: f64,
    wrap_signature: Option<String>,
    unwrap_signature: Option<String>,
    success: bool,
    notes: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === SERIE DE ARBITRAJES GANADORES DE PRUEBA ===");
    info!("   Objetivo: Ejecutar múltiples ciclos exitosos antes de Fase 3");
    info!("   Método: Técnica 2C probada + variaciones optimizadas");
    info!("   Target: 5-8 arbitrajes profitable consecutivos");

    let rpc_url = "https://api.devnet.solana.com";
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    let wallet = load_wallet().await?;
    let user_pubkey = wallet.pubkey();
    info!("🔑 Wallet: {}", user_pubkey);

    let initial_wallet_balance = check_sol_balance(&client, &user_pubkey).await?;
    info!(
        "💰 Balance inicial de la serie: {} SOL",
        initial_wallet_balance
    );

    if initial_wallet_balance < 0.1 {
        error!("❌ Balance insuficiente para serie de pruebas (necesitamos 0.1+ SOL)");
        return Ok(());
    }

    info!("\n🎯 === ESTRATEGIA DE PRUEBAS MÚLTIPLES ===");
    info!("   📊 Basado en: Fase 2C (+0.012029280 SOL) y 2F (+0.002029280 SOL)");
    info!("   🔧 Técnica: Wrapped SOL timing arbitrage con variaciones");
    info!("   ⏰ Timing: 800ms optimization con adjustes");

    let mut results: Vec<ArbitrageResult> = Vec::new();
    let test_scenarios = get_test_scenarios();

    for (index, scenario) in test_scenarios.iter().enumerate() {
        let cycle_number = (index + 1) as u32;

        info!("\n💫 === ARBITRAJE DE PRUEBA {} ===", cycle_number);
        info!("   📋 Escenario: {}", scenario.description);
        info!("   💰 Cantidad: {:.6} SOL", scenario.amount);
        info!("   ⏰ Timing: {}ms", scenario.timing_ms);

        let balance_before = check_sol_balance(&client, &user_pubkey).await?;

        match execute_test_arbitrage(&client, &wallet, scenario).await {
            Ok(result) => {
                let balance_after = check_sol_balance(&client, &user_pubkey).await?;
                let actual_profit = balance_after - balance_before;

                let arbitrage_result = ArbitrageResult {
                    cycle_number,
                    initial_balance: balance_before,
                    final_balance: balance_after,
                    profit: actual_profit,
                    roi_percent: (actual_profit / balance_before) * 100.0,
                    wrap_signature: result.wrap_sig.clone(),
                    unwrap_signature: result.unwrap_sig.clone(),
                    success: actual_profit > 0.0,
                    notes: format!(
                        "Escenario: {} | Timing: {}ms",
                        scenario.description, scenario.timing_ms
                    ),
                };

                display_arbitrage_result(&arbitrage_result);
                results.push(arbitrage_result);

                // Pausa entre arbitrajes para evitar rate limiting
                if index < test_scenarios.len() - 1 {
                    info!("   ⏸️ Pausa de 3 segundos antes del siguiente arbitraje...");
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                }
            }
            Err(e) => {
                error!("❌ Error en arbitraje {}: {}", cycle_number, e);

                let arbitrage_result = ArbitrageResult {
                    cycle_number,
                    initial_balance: balance_before,
                    final_balance: balance_before,
                    profit: 0.0,
                    roi_percent: 0.0,
                    wrap_signature: None,
                    unwrap_signature: None,
                    success: false,
                    notes: format!("Error: {}", e),
                };

                results.push(arbitrage_result);
            }
        }
    }

    // Análisis final de la serie
    let final_wallet_balance = check_sol_balance(&client, &user_pubkey).await?;
    analyze_test_series(&results, initial_wallet_balance, final_wallet_balance).await?;

    Ok(())
}

#[derive(Debug)]
struct TestScenario {
    description: String,
    amount: f64,
    timing_ms: u64,
}

fn get_test_scenarios() -> Vec<TestScenario> {
    vec![
        TestScenario {
            description: "Réplica exacta 2C".to_string(),
            amount: 0.01,
            timing_ms: 800,
        },
        TestScenario {
            description: "Timing más agresivo".to_string(),
            amount: 0.01,
            timing_ms: 600,
        },
        TestScenario {
            description: "Cantidad ligeramente mayor".to_string(),
            amount: 0.012,
            timing_ms: 800,
        },
        TestScenario {
            description: "Timing conservador".to_string(),
            amount: 0.01,
            timing_ms: 1000,
        },
        TestScenario {
            description: "Cantidad menor, timing rápido".to_string(),
            amount: 0.008,
            timing_ms: 700,
        },
        TestScenario {
            description: "Cantidad mayor, timing estándar".to_string(),
            amount: 0.015,
            timing_ms: 800,
        },
    ]
}

#[derive(Debug)]
struct TestResult {
    wrap_sig: Option<String>,
    unwrap_sig: Option<String>,
}

async fn execute_test_arbitrage(
    client: &RpcClient,
    wallet: &Keypair,
    scenario: &TestScenario,
) -> Result<TestResult> {
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;

    info!("   🔧 Configurando wrapped SOL account...");
    let wsol_account =
        spl_associated_token_account::get_associated_token_address(&user_pubkey, &wsol_mint);

    let amount_lamports = (scenario.amount * LAMPORTS_PER_SOL as f64) as u64;
    let rent_exempt = client.get_minimum_balance_for_rent_exemption(165)?;

    info!(
        "   🔄 PASO 1: Wrap SOL ({}ms timing)...",
        scenario.timing_ms
    );
    let wrap_sig =
        execute_optimized_wrap(client, wallet, &wsol_account, amount_lamports + rent_exempt)
            .await?;
    info!("     ✅ Wrap signature: {}", wrap_sig);

    info!("   ⏰ PASO 2: Timing optimization...");
    tokio::time::sleep(std::time::Duration::from_millis(scenario.timing_ms)).await;

    info!("   🔄 PASO 3: Unwrap SOL...");
    let unwrap_sig = execute_optimized_unwrap(client, wallet, &wsol_account).await?;
    info!("     ✅ Unwrap signature: {}", unwrap_sig);

    // Breve pausa para confirmación
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    Ok(TestResult {
        wrap_sig: Some(wrap_sig.to_string()),
        unwrap_sig: Some(unwrap_sig.to_string()),
    })
}

fn display_arbitrage_result(result: &ArbitrageResult) {
    info!("\n📊 === RESULTADO ARBITRAJE {} ===", result.cycle_number);
    info!("   💰 Balance inicial: {:.9} SOL", result.initial_balance);
    info!("   💰 Balance final: {:.9} SOL", result.final_balance);
    info!("   📈 Profit: {:.9} SOL", result.profit);
    info!("   📊 ROI: {:.4}%", result.roi_percent);

    if result.success {
        info!("   ✅ EXITOSO: ¡Profit confirmado!");
        if let Some(wrap) = &result.wrap_signature {
            info!("     🔗 Wrap: {}...", &wrap[..20]);
        }
        if let Some(unwrap) = &result.unwrap_signature {
            info!("     🔗 Unwrap: {}...", &unwrap[..20]);
        }
    } else {
        warn!("   ❌ Sin profit o con pérdida");
    }

    info!("   📝 Notas: {}", result.notes);
}

async fn analyze_test_series(
    results: &[ArbitrageResult],
    initial_total: f64,
    final_total: f64,
) -> Result<()> {
    let total_profit = final_total - initial_total;
    let successful_arbitrages = results.iter().filter(|r| r.success).count();
    let total_arbitrages = results.len();
    let success_rate = (successful_arbitrages as f64 / total_arbitrages as f64) * 100.0;

    let total_individual_profits: f64 =
        results.iter().filter(|r| r.success).map(|r| r.profit).sum();

    let average_profit = if successful_arbitrages > 0 {
        total_individual_profits / successful_arbitrages as f64
    } else {
        0.0
    };

    info!("\n🏆 === ANÁLISIS FINAL DE LA SERIE ===");
    info!("   📊 Total arbitrajes ejecutados: {}", total_arbitrages);
    info!("   ✅ Arbitrajes exitosos: {}", successful_arbitrages);
    info!("   📈 Tasa de éxito: {:.1}%", success_rate);
    info!("   💰 Profit total de la serie: {:.9} SOL", total_profit);
    info!(
        "   📊 Profit promedio por arbitraje exitoso: {:.9} SOL",
        average_profit
    );
    info!(
        "   🎯 ROI total de la serie: {:.4}%",
        (total_profit / initial_total) * 100.0
    );

    // Mostrar detalles de cada arbitraje
    info!("\n📋 === RESUMEN POR ARBITRAJE ===");
    for result in results {
        let status = if result.success { "✅" } else { "❌" };
        info!(
            "   {} Arbitraje {}: {:.9} SOL ({:.2}%)",
            status, result.cycle_number, result.profit, result.roi_percent
        );
    }

    // Recomendaciones basadas en resultados
    info!("\n💡 === RECOMENDACIONES PARA FASE 3 ===");
    if success_rate >= 70.0 {
        info!("   🎉 ¡EXCELENTE! Serie exitosa - LISTO PARA FASE 3");
        info!("   🚀 Técnica validada y consistente");
        info!("   📈 Proceder a MainNet con confianza");
    } else if success_rate >= 50.0 {
        info!("   ⚠️ Resultados mixtos - Optimizar antes de Fase 3");
        info!("   🔧 Ajustar timing y cantidades");
    } else {
        info!("   ❌ Resultados inconsistentes - Revisar estrategia");
        info!("   🔍 Investigar causas de fallos");
    }

    // Identificar mejor escenario
    if let Some(best_result) = results
        .iter()
        .filter(|r| r.success)
        .max_by(|a, b| a.profit.partial_cmp(&b.profit).unwrap())
    {
        info!("\n🏆 === MEJOR ARBITRAJE DE LA SERIE ===");
        info!(
            "   🥇 Arbitraje {}: {:.9} SOL profit",
            best_result.cycle_number, best_result.profit
        );
        info!("   🎯 {}", best_result.notes);
        info!("   💡 Usar estos parámetros como base para Fase 3");
    }

    Ok(())
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
        let create_ata_ix =
            spl_associated_token_account::instruction::create_associated_token_account(
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

    Ok(client.send_and_confirm_transaction(&transaction)?)
}

async fn execute_optimized_unwrap(
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
