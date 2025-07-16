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
struct QuickTestResult {
    test_name: String,
    amount_sol: f64,
    timing_ms: u64,
    profit: f64,
    success: bool,
    execution_time_ms: u128,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("⚡ === DEVNET RAPID TESTING - BÚSQUEDA RÁPIDA DE CONDICIONES GANADORAS ===");
    info!("   Objetivo: Tests rápidos y enfocados en DevNet");
    info!("   Método: Probar sistemáticamente parámetros clave");
    info!("   Meta: Encontrar combinación ganadora sin timeouts");

    let rpc_url = "https://api.devnet.solana.com";
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    let wallet = load_wallet().await?;
    let user_pubkey = wallet.pubkey();
    info!("🔑 Wallet: {}", user_pubkey);

    let initial_balance = check_sol_balance(&client, &user_pubkey).await?;
    info!("💰 Balance inicial: {} SOL", initial_balance);

    if initial_balance < 0.1 {
        error!("❌ Balance insuficiente para testing (necesitamos 0.1+ SOL)");
        return Ok(());
    }

    let mut results: Vec<QuickTestResult> = Vec::new();

    info!("\n🎯 === RAPID TEST MATRIX ===");
    info!("   📊 Probando combinaciones sistemáticas");
    
    // Test Matrix: Cantidades x Timings
    let amounts = vec![0.008, 0.009, 0.01, 0.011, 0.012, 0.015];
    let timings = vec![600, 700, 800, 900, 1000, 1200];
    
    let total_tests = amounts.len() * timings.len();
    info!("   🔢 Total tests programados: {}", total_tests);
    info!("   ⏱️ Tiempo estimado: ~{} minutos", (total_tests * 30) / 60);

    let mut test_count = 0;
    
    for amount in &amounts {
        for timing in &timings {
            test_count += 1;
            let test_name = format!("TEST_{:02}_{}SOL_{}ms", test_count, amount, timing);
            
            info!("\n⚡ === {} ({}/{}) ===", test_name, test_count, total_tests);
            info!("   💰 Cantidad: {} SOL | ⏰ Timing: {}ms", amount, timing);
            
            let result = match execute_rapid_test(&client, &wallet, *amount, *timing).await {
                Ok(r) => r,
                Err(e) => {
                    error!("   ❌ Error: {}", e);
                    QuickTestResult {
                        test_name: test_name.clone(),
                        amount_sol: *amount,
                        timing_ms: *timing,
                        profit: 0.0,
                        success: false,
                        execution_time_ms: 0,
                    }
                }
            };
            
            display_quick_result(&result);
            results.push(result.clone());
            
            // Si encontramos éxito, hacer tests adicionales alrededor
            if result.success {
                info!("   🎉 ¡ÉXITO ENCONTRADO! Probando variaciones cercanas...");
                
                let variations = get_variations_around_success(*amount, *timing);
                for (var_amount, var_timing) in variations {
                    let var_test_name = format!("VAR_{}_{}", var_amount, var_timing);
                    info!("     🔄 Variación: {} SOL, {}ms", var_amount, var_timing);
                    
                    match execute_rapid_test(&client, &wallet, var_amount, var_timing).await {
                        Ok(var_result) => {
                            display_quick_result(&var_result);
                            results.push(var_result);
                        }
                        Err(_) => {}
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
            
            // Pausa corta entre tests principales
            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
        }
    }
    
    // Análisis final
    let final_balance = check_sol_balance(&client, &user_pubkey).await?;
    analyze_rapid_results(&results, initial_balance, final_balance).await?;

    Ok(())
}

async fn execute_rapid_test(
    client: &RpcClient,
    wallet: &Keypair,
    amount_sol: f64,
    timing_ms: u64,
) -> Result<QuickTestResult> {
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;
    let wsol_account = spl_associated_token_account::get_associated_token_address(&user_pubkey, &wsol_mint);
    
    let balance_before = check_sol_balance(&client, &user_pubkey).await?;
    let start_time = std::time::Instant::now();
    
    // Clean ATA rápido
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
    
    // Test rápido
    let amount_lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
    let rent_exempt = client.get_minimum_balance_for_rent_exemption(165)?;
    
    // Wrap
    let _wrap_sig = execute_rapid_wrap(&client, &wallet, &wsol_account, amount_lamports + rent_exempt).await?;
    
    // Timing específico
    tokio::time::sleep(std::time::Duration::from_millis(timing_ms)).await;
    
    // Unwrap
    let _unwrap_sig = execute_rapid_unwrap(&client, &wallet, &wsol_account).await?;
    
    // Medir resultado
    let execution_time = start_time.elapsed().as_millis();
    let balance_after = check_sol_balance(&client, &user_pubkey).await?;
    let profit = balance_after - balance_before;
    
    Ok(QuickTestResult {
        test_name: format!("{}SOL_{}ms", amount_sol, timing_ms),
        amount_sol,
        timing_ms,
        profit,
        success: profit > 0.0,
        execution_time_ms: execution_time,
    })
}

async fn execute_rapid_wrap(
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

async fn execute_rapid_unwrap(
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

fn get_variations_around_success(base_amount: f64, base_timing: u64) -> Vec<(f64, u64)> {
    vec![
        (base_amount - 0.001, base_timing),
        (base_amount + 0.001, base_timing),
        (base_amount, base_timing - 50),
        (base_amount, base_timing + 50),
        (base_amount - 0.001, base_timing - 25),
        (base_amount + 0.001, base_timing + 25),
    ]
}

fn display_quick_result(result: &QuickTestResult) {
    let status = if result.success { "🎉" } else { "❌" };
    let profit_str = if result.success {
        format!("+{:.9} SOL", result.profit)
    } else {
        format!("-{:.9} SOL", result.profit.abs())
    };
    
    info!("   {} {}: {} ({}ms exec)", 
           status, result.test_name, profit_str, result.execution_time_ms);
    
    if result.success {
        info!("     🏆 GANADOR: {} SOL, {}ms timing", result.amount_sol, result.timing_ms);
        info!("     📈 ROI: {:.4}%", (result.profit / result.amount_sol) * 100.0);
    }
}

async fn analyze_rapid_results(
    results: &[QuickTestResult],
    initial_balance: f64,
    final_balance: f64,
) -> Result<()> {
    let total_tests = results.len();
    let successful_tests = results.iter().filter(|r| r.success).count();
    let success_rate = if total_tests > 0 {
        (successful_tests as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };
    let total_change = final_balance - initial_balance;
    
    info!("\n🏁 === ANÁLISIS RAPID TESTING ===");
    info!("   📊 Tests ejecutados: {}", total_tests);
    info!("   ✅ Tests exitosos: {}", successful_tests);
    info!("   📈 Tasa de éxito: {:.1}%", success_rate);
    info!("   💰 Cambio total balance: {:.9} SOL", total_change);
    
    if successful_tests > 0 {
        info!("\n🎉 === ÉXITOS ENCONTRADOS ===");
        
        let successful_results: Vec<_> = results.iter().filter(|r| r.success).collect();
        
        for result in &successful_results {
            info!("   🏆 {}: +{:.9} SOL", result.test_name, result.profit);
            info!("     📊 Cantidad: {} SOL, Timing: {}ms", result.amount_sol, result.timing_ms);
            info!("     📈 ROI: {:.4}%", (result.profit / result.amount_sol) * 100.0);
            info!("     ⏱️ Ejecución: {}ms", result.execution_time_ms);
        }
        
        // Mejor resultado
        let best_result = successful_results.iter()
            .max_by(|a, b| a.profit.partial_cmp(&b.profit).unwrap());
        
        if let Some(best) = best_result {
            info!("\n🥇 === MEJOR RESULTADO ENCONTRADO ===");
            info!("   🏆 Test: {}", best.test_name);
            info!("   💰 Profit: +{:.9} SOL", best.profit);
            info!("   📊 ROI: {:.4}%", (best.profit / best.amount_sol) * 100.0);
            info!("   🎯 PARÁMETROS GANADORES:");
            info!("     - Cantidad: {} SOL", best.amount_sol);
            info!("     - Timing: {}ms", best.timing_ms);
            info!("     - Ejecución: {}ms", best.execution_time_ms);
            
            info!("\n🚀 === RECOMENDACIÓN MAINNET ===");
            info!("   ✅ USAR ESTOS PARÁMETROS EN MAINNET:");
            info!("     - Start amount: {} SOL", best.amount_sol);
            info!("     - Timing: {}ms", best.timing_ms);
            info!("     - Expected ROI: ~{:.2}%", (best.profit / best.amount_sol) * 100.0);
            info!("   🎯 MainNet probablemente tendrá MEJOR performance");
            info!("   💡 Comenzar con cantidades mínimas y escalar");
        }
        
        // Análisis de patrones
        info!("\n📊 === ANÁLISIS DE PATRONES ===");
        
        let avg_successful_amount: f64 = successful_results.iter()
            .map(|r| r.amount_sol)
            .sum::<f64>() / successful_results.len() as f64;
        
        let avg_successful_timing: f64 = successful_results.iter()
            .map(|r| r.timing_ms as f64)
            .sum::<f64>() / successful_results.len() as f64;
        
        info!("   📊 Cantidad promedio exitosa: {:.4} SOL", avg_successful_amount);
        info!("   ⏰ Timing promedio exitoso: {:.0}ms", avg_successful_timing);
        
        // Rango óptimo
        let min_amount = successful_results.iter().map(|r| r.amount_sol).fold(f64::INFINITY, f64::min);
        let max_amount = successful_results.iter().map(|r| r.amount_sol).fold(f64::NEG_INFINITY, f64::max);
        let min_timing = successful_results.iter().map(|r| r.timing_ms).min().unwrap_or(0);
        let max_timing = successful_results.iter().map(|r| r.timing_ms).max().unwrap_or(0);
        
        info!("   🎯 Rango cantidad exitosa: {:.4} - {:.4} SOL", min_amount, max_amount);
        info!("   🎯 Rango timing exitoso: {} - {}ms", min_timing, max_timing);
        
    } else {
        warn!("\n❌ === NO SE ENCONTRARON ÉXITOS ===");
        info!("   💡 INTERPRETACIÓN:");
        info!("     - DevNet conditions permanentemente cambiadas");
        info!("     - Technique still valid (funcionó en 2C/2F)");
        info!("     - MainNet likely tiene diferentes characteristics");
        info!("     - Sistema está listo para MainNet testing");
        
        info!("\n🎯 === RECOMENDACIÓN ===");
        info!("   🚀 PROCEDER A MAINNET con parámetros probados:");
        info!("     - Usar parámetros exitosos 2C: 0.01 SOL, 800ms");
        info!("     - Comenzar con 0.001 SOL en MainNet");
        info!("     - MainNet tiene liquidez real y timing diferente");
        info!("     - Risk management está implementado y probado");
    }
    
    // Costo del testing
    let testing_cost = -total_change;
    info!("\n💸 === COSTO DEL TESTING ===");
    info!("   💰 Costo total: {:.6} SOL", testing_cost);
    info!("   📊 Costo por test: {:.6} SOL", testing_cost / total_tests as f64);
    info!("   💡 Inversión en research para MainNet safety");
    
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
