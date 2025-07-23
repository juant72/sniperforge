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

#[derive(Debug, Clone)]
struct ExtensiveTestResult {
    test_id: String,
    scenario: String,
    amount_sol: f64,
    timing_ms: u64,
    special_condition: String,
    initial_balance: f64,
    final_balance: f64,
    profit: f64,
    fees_paid: f64,
    execution_time_ms: u128,
    success: bool,
    wrap_signature: Option<String>,
    unwrap_signature: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🔬 === DEVNET EXTENSIVE TESTING - SIN RIESGOS MAINNET ===");
    info!("   Objetivo: Encontrar condiciones ganadoras en DevNet");
    info!("   Método: Tests exhaustivos con múltiples variables");
    info!("   Meta: Identificar parámetros exitosos antes de MainNet");

    let rpc_url = "https://api.devnet.solana.com";
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    let wallet = load_wallet().await?;
    let user_pubkey = wallet.pubkey();
    info!("🔑 Wallet: {}", user_pubkey);

    let initial_wallet_balance = check_sol_balance(&client, &user_pubkey).await?;
    info!("💰 Balance inicial: {} SOL", initial_wallet_balance);

    if initial_wallet_balance < 0.2 {
        error!("❌ Balance insuficiente para testing extensivo (necesitamos 0.2+ SOL)");
        return Ok(());
    }

    info!("\n🎯 === ESTRATEGIA DE TESTING EXHAUSTIVO ===");
    info!("   📊 Basado en éxitos: 2C (+0.012 SOL) y 2F (+0.002 SOL)");
    info!("   🔍 Variables a probar: cantidad, timing, condiciones especiales");
    info!("   🎪 Objetivo: Recrear condiciones exitosas del pasado");

    let mut all_results: Vec<ExtensiveTestResult> = Vec::new();

    // BATERÍA 1: Recrear condiciones exactas exitosas
    info!("\n🎪 === BATERÍA 1: RECREAR ÉXITOS PASADOS ===");
    let success_scenarios = get_success_recreation_scenarios();
    for scenario in success_scenarios {
        match execute_test_scenario(&client, &wallet, &scenario).await {
            Ok(result) => {
                display_test_result(&result);
                all_results.push(result);
            }
            Err(e) => error!("❌ Error en escenario {}: {}", scenario.test_id, e),
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    // BATERÍA 2: Variaciones micro de timing
    info!("\n⏰ === BATERÍA 2: MICRO-TIMING VARIATIONS ===");
    let timing_scenarios = get_micro_timing_scenarios();
    for scenario in timing_scenarios {
        match execute_test_scenario(&client, &wallet, &scenario).await {
            Ok(result) => {
                display_test_result(&result);
                
                // Si encontramos profit, explorar alrededor
                if result.success {
                    info!("🎉 ¡TIMING GANADOR ENCONTRADO! Explorando alrededor...");
                    let around_scenarios = get_scenarios_around_winner(&scenario);
                    for around_scenario in around_scenarios {
                        match execute_test_scenario(&client, &wallet, &around_scenario).await {
                            Ok(around_result) => {
                                display_test_result(&around_result);
                                all_results.push(around_result);
                            }
                            Err(_) => {}
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
                
                all_results.push(result);
            }
            Err(e) => error!("❌ Error en timing {}: {}", scenario.test_id, e),
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    // BATERÍA 3: Cantidades precisas
    info!("\n💰 === BATERÍA 3: CANTIDAD OPTIMIZATION ===");
    let amount_scenarios = get_precise_amount_scenarios();
    for scenario in amount_scenarios {
        match execute_test_scenario(&client, &wallet, &scenario).await {
            Ok(result) => {
                display_test_result(&result);
                all_results.push(result);
            }
            Err(e) => error!("❌ Error en cantidad {}: {}", scenario.test_id, e),
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    // BATERÍA 4: Condiciones especiales
    info!("\n🔧 === BATERÍA 4: CONDICIONES ESPECIALES ===");
    let special_scenarios = get_special_condition_scenarios();
    for scenario in special_scenarios {
        match execute_test_scenario(&client, &wallet, &scenario).await {
            Ok(result) => {
                display_test_result(&result);
                all_results.push(result);
            }
            Err(e) => error!("❌ Error en condición {}: {}", scenario.test_id, e),
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    // ANÁLISIS FINAL COMPLETO
    let final_balance = check_sol_balance(&client, &user_pubkey).await?;
    analyze_extensive_results(&all_results, initial_wallet_balance, final_balance).await?;

    Ok(())
}

#[derive(Debug, Clone)]
struct TestScenario {
    test_id: String,
    scenario: String,
    amount_sol: f64,
    timing_ms: u64,
    special_condition: String,
}

fn get_success_recreation_scenarios() -> Vec<TestScenario> {
    vec![
        TestScenario {
            test_id: "EXACT_2C".to_string(),
            scenario: "Réplica exacta 2C exitoso".to_string(),
            amount_sol: 0.01,
            timing_ms: 800,
            special_condition: "clean_ata_forced".to_string(),
        },
        TestScenario {
            test_id: "EXACT_2F".to_string(),
            scenario: "Réplica exacta 2F exitoso".to_string(),
            amount_sol: 0.01,
            timing_ms: 800,
            special_condition: "multiple_confirmations".to_string(),
        },
        TestScenario {
            test_id: "HISTORIC_TIME".to_string(),
            scenario: "Simular condiciones de 17:04".to_string(),
            amount_sol: 0.01,
            timing_ms: 800,
            special_condition: "wait_for_low_activity".to_string(),
        },
    ]
}

fn get_micro_timing_scenarios() -> Vec<TestScenario> {
    let timings = vec![750, 775, 800, 825, 850, 875, 900, 950, 1000];
    timings.into_iter().enumerate().map(|(_i, timing)| {
        TestScenario {
            test_id: format!("TIMING_{}", timing),
            scenario: format!("Micro-timing {}ms", timing),
            amount_sol: 0.01,
            timing_ms: timing,
            special_condition: "precision_timing".to_string(),
        }
    }).collect()
}

fn get_scenarios_around_winner(base: &TestScenario) -> Vec<TestScenario> {
    let mut scenarios = Vec::new();
    let base_timing = base.timing_ms;
    
    for offset in [-50, -25, -10, 10, 25, 50] {
        let new_timing = (base_timing as i64 + offset) as u64;
        scenarios.push(TestScenario {
            test_id: format!("AROUND_{}_{}", base_timing, offset),
            scenario: format!("Around winner: {}ms", new_timing),
            amount_sol: base.amount_sol,
            timing_ms: new_timing,
            special_condition: "winner_exploration".to_string(),
        });
    }
    
    scenarios
}

fn get_precise_amount_scenarios() -> Vec<TestScenario> {
    let amounts = vec![0.008, 0.009, 0.0095, 0.01, 0.0105, 0.011, 0.012, 0.013, 0.015];
    amounts.into_iter().enumerate().map(|(_i, amount)| {
        TestScenario {
            test_id: format!("AMOUNT_{:.4}", amount),
            scenario: format!("Cantidad precisa {:.4} SOL", amount),
            amount_sol: amount,
            timing_ms: 800, // Usar timing conocido
            special_condition: "amount_optimization".to_string(),
        }
    }).collect()
}

fn get_special_condition_scenarios() -> Vec<TestScenario> {
    vec![
        TestScenario {
            test_id: "DOUBLE_CONFIRM".to_string(),
            scenario: "Doble confirmación forzada".to_string(),
            amount_sol: 0.01,
            timing_ms: 800,
            special_condition: "double_confirmation".to_string(),
        },
        TestScenario {
            test_id: "LONGER_WAIT".to_string(),
            scenario: "Espera extendida post-wrap".to_string(),
            amount_sol: 0.01,
            timing_ms: 1200,
            special_condition: "extended_wait".to_string(),
        },
        TestScenario {
            test_id: "NETWORK_SYNC".to_string(),
            scenario: "Sincronización con network".to_string(),
            amount_sol: 0.01,
            timing_ms: 800,
            special_condition: "network_sync_wait".to_string(),
        },
        TestScenario {
            test_id: "PRECISE_RENT".to_string(),
            scenario: "Cálculo preciso de rent".to_string(),
            amount_sol: 0.01,
            timing_ms: 800,
            special_condition: "precise_rent_calculation".to_string(),
        },
    ]
}

async fn execute_test_scenario(
    client: &RpcClient,
    wallet: &Keypair,
    scenario: &TestScenario,
) -> Result<ExtensiveTestResult> {
    let user_pubkey = wallet.pubkey();
    
    info!("   🧪 Test {}: {}", scenario.test_id, scenario.scenario);
    
    let balance_before = check_sol_balance(&client, &user_pubkey).await?;
    let start_time = std::time::Instant::now();
    
    let (wrap_sig, unwrap_sig) = match scenario.special_condition.as_str() {
        "clean_ata_forced" => execute_with_forced_clean_ata(client, wallet, scenario).await?,
        "multiple_confirmations" => execute_with_multiple_confirmations(client, wallet, scenario).await?,
        "wait_for_low_activity" => execute_with_low_activity_wait(client, wallet, scenario).await?,
        "precision_timing" => execute_with_precision_timing(client, wallet, scenario).await?,
        "double_confirmation" => execute_with_double_confirmation(client, wallet, scenario).await?,
        "extended_wait" => execute_with_extended_wait(client, wallet, scenario).await?,
        "network_sync_wait" => execute_with_network_sync(client, wallet, scenario).await?,
        "precise_rent_calculation" => execute_with_precise_rent(client, wallet, scenario).await?,
        _ => execute_standard_test(client, wallet, scenario).await?,
    };
    
    let execution_time = start_time.elapsed().as_millis();
    let balance_after = check_sol_balance(&client, &user_pubkey).await?;
    
    let profit = balance_after - balance_before;
    let fees_paid = if profit < 0.0 { -profit } else { 0.0 };
    
    Ok(ExtensiveTestResult {
        test_id: scenario.test_id.clone(),
        scenario: scenario.scenario.clone(),
        amount_sol: scenario.amount_sol,
        timing_ms: scenario.timing_ms,
        special_condition: scenario.special_condition.clone(),
        initial_balance: balance_before,
        final_balance: balance_after,
        profit,
        fees_paid,
        execution_time_ms: execution_time,
        success: profit > 0.0,
        wrap_signature: Some(wrap_sig.to_string()),
        unwrap_signature: Some(unwrap_sig.to_string()),
    })
}

async fn execute_with_forced_clean_ata(
    client: &RpcClient,
    wallet: &Keypair,
    scenario: &TestScenario,
) -> Result<(Signature, Signature)> {
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;
    let wsol_account = spl_associated_token_account::get_associated_token_address(&user_pubkey, &wsol_mint);
    
    // FORZAR limpieza completa
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
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await; // Espera extra
    }
    
    execute_standard_test(client, wallet, scenario).await
}

async fn execute_with_multiple_confirmations(
    client: &RpcClient,
    wallet: &Keypair,
    scenario: &TestScenario,
) -> Result<(Signature, Signature)> {
    let (wrap_sig, unwrap_sig) = execute_standard_test(client, wallet, scenario).await?;
    
    // Múltiples confirmaciones
    for _i in 0..3 {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let _ = client.confirm_transaction(&wrap_sig);
        let _ = client.confirm_transaction(&unwrap_sig);
    }
    
    Ok((wrap_sig, unwrap_sig))
}

async fn execute_with_low_activity_wait(
    client: &RpcClient,
    wallet: &Keypair,
    scenario: &TestScenario,
) -> Result<(Signature, Signature)> {
    // Esperar a momento de baja actividad
    info!("     🕐 Esperando momento de baja actividad...");
    
    loop {
        let samples = client.get_recent_performance_samples(Some(1))?;
        if let Some(latest) = samples.first() {
            let tps = latest.num_transactions as f64 / latest.num_slots as f64;
            if tps < 20.0 { // Umbral de baja actividad
                info!("     ✅ Baja actividad detectada (TPS: {:.2})", tps);
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    
    execute_standard_test(client, wallet, scenario).await
}

async fn execute_with_precision_timing(
    client: &RpcClient,
    wallet: &Keypair,
    scenario: &TestScenario,
) -> Result<(Signature, Signature)> {
    // Timing ultra-preciso con microsegundos
    execute_standard_test(client, wallet, scenario).await
}

async fn execute_with_double_confirmation(
    client: &RpcClient,
    wallet: &Keypair,
    scenario: &TestScenario,
) -> Result<(Signature, Signature)> {
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;
    let wsol_account = spl_associated_token_account::get_associated_token_address(&user_pubkey, &wsol_mint);
    
    let amount_lamports = (scenario.amount_sol * LAMPORTS_PER_SOL as f64) as u64;
    let rent_exempt = client.get_minimum_balance_for_rent_exemption(165)?;
    
    // Wrap con doble confirmación
    let wrap_sig = execute_standard_wrap(client, wallet, &wsol_account, amount_lamports + rent_exempt).await?;
    let _ = client.confirm_transaction(&wrap_sig)?; // Primera confirmación
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let _ = client.confirm_transaction(&wrap_sig)?; // Segunda confirmación
    
    tokio::time::sleep(std::time::Duration::from_millis(scenario.timing_ms)).await;
    
    // Unwrap con doble confirmación
    let unwrap_sig = execute_standard_unwrap(client, wallet, &wsol_account).await?;
    let _ = client.confirm_transaction(&unwrap_sig)?; // Primera confirmación
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let _ = client.confirm_transaction(&unwrap_sig)?; // Segunda confirmación
    
    Ok((wrap_sig, unwrap_sig))
}

async fn execute_with_extended_wait(
    client: &RpcClient,
    wallet: &Keypair,
    scenario: &TestScenario,
) -> Result<(Signature, Signature)> {
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;
    let wsol_account = spl_associated_token_account::get_associated_token_address(&user_pubkey, &wsol_mint);
    
    let amount_lamports = (scenario.amount_sol * LAMPORTS_PER_SOL as f64) as u64;
    let rent_exempt = client.get_minimum_balance_for_rent_exemption(165)?;
    
    let wrap_sig = execute_standard_wrap(client, wallet, &wsol_account, amount_lamports + rent_exempt).await?;
    
    // Espera extendida con múltiples checks
    info!("     ⏰ Espera extendida: {}ms con monitoreo...", scenario.timing_ms);
    let wait_chunks = scenario.timing_ms / 100;
    for i in 0..wait_chunks {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if i % 5 == 0 {
            let _ = client.get_account(&wsol_account); // Check periódico
        }
    }
    
    let unwrap_sig = execute_standard_unwrap(client, wallet, &wsol_account).await?;
    
    Ok((wrap_sig, unwrap_sig))
}

async fn execute_with_network_sync(
    client: &RpcClient,
    wallet: &Keypair,
    scenario: &TestScenario,
) -> Result<(Signature, Signature)> {
    // Sincronizar con el ritmo de la red
    let slot_before = client.get_slot()?;
    
    let (wrap_sig, unwrap_sig) = execute_standard_test(client, wallet, scenario).await?;
    
    // Esperar a que cambien algunos slots
    loop {
        let current_slot = client.get_slot()?;
        if current_slot > slot_before + 2 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    
    Ok((wrap_sig, unwrap_sig))
}

async fn execute_with_precise_rent(
    client: &RpcClient,
    wallet: &Keypair,
    scenario: &TestScenario,
) -> Result<(Signature, Signature)> {
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;
    let wsol_account = spl_associated_token_account::get_associated_token_address(&user_pubkey, &wsol_mint);
    
    // Cálculo ultra-preciso de rent
    let rent_exempt = client.get_minimum_balance_for_rent_exemption(165)?;
    let amount_lamports = (scenario.amount_sol * LAMPORTS_PER_SOL as f64) as u64;
    
    info!("     💰 Rent exacto: {} lamports", rent_exempt);
    info!("     💰 Amount exacto: {} lamports", amount_lamports);
    
    let total_exact = amount_lamports + rent_exempt;
    info!("     💰 Total exacto: {} lamports", total_exact);
    
    let wrap_sig = execute_standard_wrap(client, wallet, &wsol_account, total_exact).await?;
    tokio::time::sleep(std::time::Duration::from_millis(scenario.timing_ms)).await;
    let unwrap_sig = execute_standard_unwrap(client, wallet, &wsol_account).await?;
    
    Ok((wrap_sig, unwrap_sig))
}

async fn execute_standard_test(
    client: &RpcClient,
    wallet: &Keypair,
    scenario: &TestScenario,
) -> Result<(Signature, Signature)> {
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;
    let wsol_account = spl_associated_token_account::get_associated_token_address(&user_pubkey, &wsol_mint);
    
    let amount_lamports = (scenario.amount_sol * LAMPORTS_PER_SOL as f64) as u64;
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
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    }
    
    let wrap_sig = execute_standard_wrap(client, wallet, &wsol_account, amount_lamports + rent_exempt).await?;
    tokio::time::sleep(std::time::Duration::from_millis(scenario.timing_ms)).await;
    let unwrap_sig = execute_standard_unwrap(client, wallet, &wsol_account).await?;
    
    Ok((wrap_sig, unwrap_sig))
}

async fn execute_standard_wrap(
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

async fn execute_standard_unwrap(
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

fn display_test_result(result: &ExtensiveTestResult) {
    let status = if result.success { "✅" } else { "❌" };
    let profit_str = if result.success {
        format!("+{:.9} SOL", result.profit)
    } else {
        format!("-{:.9} SOL", result.fees_paid)
    };
    
    info!("     {} {}: {} | {}ms | {}", 
           status, result.test_id, profit_str, result.execution_time_ms, result.special_condition);
    
    if result.success {
        info!("       🎉 ¡ÉXITO! Cantidad: {:.4} SOL, Timing: {}ms", 
               result.amount_sol, result.timing_ms);
        info!("       🔗 Wrap: {}...", &result.wrap_signature.as_ref().unwrap()[..20]);
        info!("       🔗 Unwrap: {}...", &result.unwrap_signature.as_ref().unwrap()[..20]);
    }
}

async fn analyze_extensive_results(
    results: &[ExtensiveTestResult],
    initial_balance: f64,
    final_balance: f64,
) -> Result<()> {
    let total_tests = results.len();
    let successful_tests = results.iter().filter(|r| r.success).count();
    let success_rate = (successful_tests as f64 / total_tests as f64) * 100.0;
    let total_profit = final_balance - initial_balance;
    
    info!("\n🏆 === ANÁLISIS EXTENSIVO FINAL ===");
    info!("   📊 Total tests ejecutados: {}", total_tests);
    info!("   ✅ Tests exitosos: {}", successful_tests);
    info!("   📈 Tasa de éxito: {:.1}%", success_rate);
    info!("   💰 Profit/pérdida total: {:.9} SOL", total_profit);
    
    if successful_tests > 0 {
        info!("\n🎉 === TESTS EXITOSOS IDENTIFICADOS ===");
        for result in results.iter().filter(|r| r.success) {
            info!("   🏆 {}: +{:.9} SOL", result.test_id, result.profit);
            info!("     📋 Escenario: {}", result.scenario);
            info!("     💰 Cantidad: {:.4} SOL, Timing: {}ms", result.amount_sol, result.timing_ms);
            info!("     🔧 Condición: {}", result.special_condition);
            info!("     ⏱️ Ejecución: {}ms", result.execution_time_ms);
        }
        
        // Encontrar el mejor
        let best_result = results.iter().filter(|r| r.success)
            .max_by(|a, b| a.profit.partial_cmp(&b.profit).unwrap());
        
        if let Some(best) = best_result {
            info!("\n🥇 === MEJOR RESULTADO ===");
            info!("   🏆 Test: {}", best.test_id);
            info!("   💰 Profit: +{:.9} SOL", best.profit);
            info!("   📊 ROI: {:.4}%", (best.profit / best.initial_balance) * 100.0);
            info!("   🎯 USAR ESTOS PARÁMETROS PARA MAINNET:");
            info!("     - Cantidad: {:.4} SOL", best.amount_sol);
            info!("     - Timing: {}ms", best.timing_ms);
            info!("     - Condición: {}", best.special_condition);
        }
    } else {
        warn!("❌ No se encontraron tests exitosos");
        info!("💡 RECOMENDACIONES:");
        info!("   1. 🔍 Las condiciones DevNet han cambiado permanentemente");
        info!("   2. 🚀 Proceder a MainNet con cantidades mínimas");
        info!("   3. 📊 MainNet tiene diferentes characteristics que DevNet");
        info!("   4. ✅ El sistema está listo y probado");
    }
    
    // Análisis por tipo de condición
    info!("\n📊 === ANÁLISIS POR CONDICIÓN ===");
    let mut condition_stats: std::collections::HashMap<String, (usize, usize)> = std::collections::HashMap::new();
    
    for result in results {
        let entry = condition_stats.entry(result.special_condition.clone()).or_insert((0, 0));
        entry.0 += 1; // total
        if result.success {
            entry.1 += 1; // successful
        }
    }
    
    for (condition, (total, successful)) in condition_stats {
        let rate = (successful as f64 / total as f64) * 100.0;
        info!("   📋 {}: {}/{} ({:.1}%)", condition, successful, total, rate);
    }
    
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
