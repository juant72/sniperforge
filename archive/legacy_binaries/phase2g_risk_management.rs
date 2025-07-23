use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    message::Message,
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
struct RiskAnalysis {
    estimated_fees: u64,
    rent_exempt_cost: u64,
    total_cost: u64,
    minimum_balance_required: u64,
    profit_threshold: u64,
    risk_level: RiskLevel,
    recommendation: String,
}

#[derive(Debug)]
enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug)]
struct ArbitrageParams {
    amount_sol: f64,
    amount_lamports: u64,
    expected_profit_lamports: u64,
    max_acceptable_loss: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === FASE 2G: ARBITRAJE CON ANÁLISIS DE RIESGO ===");
    info!("   Objetivo: Calcular fees y riesgo antes de ejecutar");
    info!("   Estrategia: Risk management + profit validation");

    let rpc_url = "https://api.devnet.solana.com";
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    let wallet = load_wallet().await?;
    let user_pubkey = wallet.pubkey();
    info!("🔑 Wallet: {}", user_pubkey);

    let initial_balance = check_sol_balance(&client, &user_pubkey).await?;
    info!("💰 Balance inicial: {} SOL", initial_balance);

    if initial_balance < 0.03 {
        error!("❌ Balance insuficiente para análisis de riesgo");
        return Ok(());
    }

    info!("\n🎯 === ANÁLISIS DE RIESGO PREVIO ===");

    // Definir parámetros de arbitraje
    let params = ArbitrageParams {
        amount_sol: 0.015,
        amount_lamports: 15_000_000,
        expected_profit_lamports: 2_000_000, // Esperamos ~0.002 SOL profit
        max_acceptable_loss: 100_000,        // Máximo 0.0001 SOL pérdida
    };

    info!("   💰 Cantidad a invertir: {} SOL", params.amount_sol);
    info!(
        "   📈 Profit esperado: {} SOL",
        params.expected_profit_lamports as f64 / LAMPORTS_PER_SOL as f64
    );
    info!(
        "   🚨 Pérdida máxima aceptable: {} SOL",
        params.max_acceptable_loss as f64 / LAMPORTS_PER_SOL as f64
    );

    // PASO 1: Análisis completo de riesgo
    match perform_comprehensive_risk_analysis(&client, &wallet, &params).await {
        Ok(risk_analysis) => {
            display_risk_analysis(&risk_analysis);

            match risk_analysis.risk_level {
                RiskLevel::Low | RiskLevel::Medium => {
                    info!("\n✅ RIESGO ACEPTABLE - Procediendo con arbitraje");

                    match execute_risk_calculated_arbitrage(
                        &client,
                        &wallet,
                        &params,
                        &risk_analysis,
                    )
                    .await
                    {
                        Ok(actual_profit) => {
                            let final_balance = check_sol_balance(&client, &user_pubkey).await?;

                            info!("\n📊 === RESULTADO CON ANÁLISIS DE RIESGO ===");
                            info!("💰 Balance inicial: {} SOL", initial_balance);
                            info!("💰 Balance final: {} SOL", final_balance);
                            info!("📈 Profit real: {:.9} SOL", actual_profit);
                            info!(
                                "🎯 Profit esperado: {:.9} SOL",
                                params.expected_profit_lamports as f64 / LAMPORTS_PER_SOL as f64
                            );
                            info!(
                                "📊 Fees estimados: {:.9} SOL",
                                risk_analysis.estimated_fees as f64 / LAMPORTS_PER_SOL as f64
                            );
                            info!(
                                "📊 Fees reales: {:.9} SOL",
                                initial_balance - final_balance - actual_profit
                            );

                            if actual_profit > 0.0 {
                                info!("🎉 ¡ARBITRAJE EXITOSO CON GESTIÓN DE RIESGO!");
                                let accuracy = ((actual_profit * LAMPORTS_PER_SOL as f64)
                                    / params.expected_profit_lamports as f64)
                                    * 100.0;
                                info!("   📊 Precisión predicción: {:.1}%", accuracy);
                            } else {
                                info!(
                                    "⚠️ Pérdida dentro de rango aceptable: {:.9} SOL",
                                    actual_profit
                                );
                            }
                        }
                        Err(e) => error!("❌ Error en ejecución: {}", e),
                    }
                }
                RiskLevel::High | RiskLevel::Critical => {
                    warn!("🚨 RIESGO ALTO - OPERACIÓN CANCELADA");
                    info!("   {}", risk_analysis.recommendation);
                    info!("   💡 Sugerencia: Ajustar parámetros o esperar mejores condiciones");
                }
            }
        }
        Err(e) => error!("❌ Error en análisis de riesgo: {}", e),
    }

    Ok(())
}

async fn perform_comprehensive_risk_analysis(
    client: &RpcClient,
    wallet: &Keypair,
    params: &ArbitrageParams,
) -> Result<RiskAnalysis> {
    let user_pubkey = wallet.pubkey();

    info!("🔍 === CALCULANDO RIESGOS Y FEES ===");

    // 1. Calcular rent exempt cost
    let rent_exempt_cost = client.get_minimum_balance_for_rent_exemption(165)?;
    info!(
        "   💰 Rent exempt requerido: {} lamports ({:.9} SOL)",
        rent_exempt_cost,
        rent_exempt_cost as f64 / LAMPORTS_PER_SOL as f64
    );

    // 2. Simular transacciones para calcular fees
    let estimated_fees = estimate_transaction_fees(client, wallet, params).await?;
    info!(
        "   💸 Fees estimados: {} lamports ({:.9} SOL)",
        estimated_fees,
        estimated_fees as f64 / LAMPORTS_PER_SOL as f64
    );

    // 3. Calcular costos totales
    let total_cost = estimated_fees + rent_exempt_cost;
    info!(
        "   💼 Costo total: {} lamports ({:.9} SOL)",
        total_cost,
        total_cost as f64 / LAMPORTS_PER_SOL as f64
    );

    // 4. Calcular balance mínimo requerido
    let minimum_balance_required = params.amount_lamports + total_cost + 5_000_000; // 0.005 SOL buffer
    info!(
        "   🏦 Balance mínimo requerido: {} lamports ({:.9} SOL)",
        minimum_balance_required,
        minimum_balance_required as f64 / LAMPORTS_PER_SOL as f64
    );

    // 5. Verificar balance actual
    let current_balance = client.get_balance(&user_pubkey)?;
    info!(
        "   💳 Balance actual: {} lamports ({:.9} SOL)",
        current_balance,
        current_balance as f64 / LAMPORTS_PER_SOL as f64
    );

    // 6. Calcular profit threshold
    let profit_threshold = total_cost + params.max_acceptable_loss;
    info!(
        "   📊 Profit mínimo requerido: {} lamports ({:.9} SOL)",
        profit_threshold,
        profit_threshold as f64 / LAMPORTS_PER_SOL as f64
    );

    // 7. Análisis de riesgo
    let (risk_level, recommendation) = analyze_risk(
        current_balance,
        minimum_balance_required,
        params.expected_profit_lamports,
        profit_threshold,
        estimated_fees,
    );

    Ok(RiskAnalysis {
        estimated_fees,
        rent_exempt_cost,
        total_cost,
        minimum_balance_required,
        profit_threshold,
        risk_level,
        recommendation,
    })
}

async fn estimate_transaction_fees(
    client: &RpcClient,
    wallet: &Keypair,
    params: &ArbitrageParams,
) -> Result<u64> {
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;
    let wsol_account =
        spl_associated_token_account::get_associated_token_address(&user_pubkey, &wsol_mint);

    info!("   🧮 Simulando transacciones para estimar fees...");

    let mut total_estimated_fees = 0u64;

    // Simular transacción 1: Wrap
    let wrap_instructions = vec![
        system_instruction::transfer(
            &user_pubkey,
            &wsol_account,
            params.amount_lamports + 2_039_280,
        ),
        spl_token::instruction::sync_native(&spl_token::id(), &wsol_account)?,
    ];

    let wrap_fee = simulate_transaction_fee(client, &wrap_instructions, &user_pubkey).await?;
    total_estimated_fees += wrap_fee;
    info!("     📝 Fee estimado wrap: {} lamports", wrap_fee);

    // Simular transacción 2: Unwrap
    let unwrap_instructions = vec![spl_token::instruction::close_account(
        &spl_token::id(),
        &wsol_account,
        &user_pubkey,
        &user_pubkey,
        &[],
    )?];

    let unwrap_fee = simulate_transaction_fee(client, &unwrap_instructions, &user_pubkey).await?;
    total_estimated_fees += unwrap_fee;
    info!("     📝 Fee estimado unwrap: {} lamports", unwrap_fee);

    // Agregar buffer para variabilidad de fees
    let fee_buffer = (total_estimated_fees as f64 * 0.2) as u64; // 20% buffer
    total_estimated_fees += fee_buffer;
    info!("     🛡️ Buffer fees (20%): {} lamports", fee_buffer);

    Ok(total_estimated_fees)
}

async fn simulate_transaction_fee(
    client: &RpcClient,
    instructions: &[solana_sdk::instruction::Instruction],
    payer: &Pubkey,
) -> Result<u64> {
    let recent_blockhash = client.get_latest_blockhash()?;
    let message = Message::new(instructions, Some(payer));

    // Calcular fee basado en el message
    let fee = client.get_fee_for_message(&message)?;
    Ok(fee)
}

fn analyze_risk(
    current_balance: u64,
    minimum_required: u64,
    expected_profit: u64,
    profit_threshold: u64,
    estimated_fees: u64,
) -> (RiskLevel, String) {
    let balance_ratio = current_balance as f64 / minimum_required as f64;
    let profit_ratio = expected_profit as f64 / profit_threshold as f64;

    if current_balance < minimum_required {
        return (
            RiskLevel::Critical,
            "Balance insuficiente para la operación. Necesitas más SOL.".to_string(),
        );
    }

    if expected_profit < profit_threshold {
        return (
            RiskLevel::High,
            "Profit esperado menor que fees + pérdida aceptable. Alto riesgo de pérdida neta."
                .to_string(),
        );
    }

    if balance_ratio < 1.5 {
        return (
            RiskLevel::Medium,
            "Balance ajustado. Proceder con precaución.".to_string(),
        );
    }

    if profit_ratio < 2.0 {
        return (
            RiskLevel::Medium,
            "Margen de profit moderado. Riesgo controlado.".to_string(),
        );
    }

    (
        RiskLevel::Low,
        "Condiciones óptimas. Riesgo bajo, proceder con confianza.".to_string(),
    )
}

fn display_risk_analysis(analysis: &RiskAnalysis) {
    info!("\n📊 === ANÁLISIS DE RIESGO COMPLETO ===");
    info!(
        "   💸 Fees estimados: {:.9} SOL",
        analysis.estimated_fees as f64 / LAMPORTS_PER_SOL as f64
    );
    info!(
        "   🏠 Rent exempt: {:.9} SOL",
        analysis.rent_exempt_cost as f64 / LAMPORTS_PER_SOL as f64
    );
    info!(
        "   💼 Costo total: {:.9} SOL",
        analysis.total_cost as f64 / LAMPORTS_PER_SOL as f64
    );
    info!(
        "   🏦 Balance mínimo: {:.9} SOL",
        analysis.minimum_balance_required as f64 / LAMPORTS_PER_SOL as f64
    );
    info!(
        "   📊 Profit mínimo: {:.9} SOL",
        analysis.profit_threshold as f64 / LAMPORTS_PER_SOL as f64
    );
    info!("   🚨 Nivel de riesgo: {:?}", analysis.risk_level);
    info!("   💡 Recomendación: {}", analysis.recommendation);
}

async fn execute_risk_calculated_arbitrage(
    client: &RpcClient,
    wallet: &Keypair,
    params: &ArbitrageParams,
    _risk_analysis: &RiskAnalysis,
) -> Result<f64> {
    let user_pubkey = wallet.pubkey();

    info!("\n💫 === EJECUTANDO ARBITRAJE CON RIESGO CALCULADO ===");

    let balance_before = client.get_balance(&user_pubkey)?;

    // Usar el método exitoso de 2C con parámetros validados
    let wsol_account = setup_wsol_account_safe(client, wallet).await?;

    info!("   🔄 Ejecutando wrap con riesgo validado...");
    let _wrap_sig =
        execute_safe_wrap(client, wallet, &wsol_account, params.amount_lamports).await?;

    tokio::time::sleep(std::time::Duration::from_millis(800)).await;

    info!("   🔄 Ejecutando unwrap con riesgo validado...");
    let _unwrap_sig = execute_safe_unwrap(client, wallet, &wsol_account).await?;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let balance_after = client.get_balance(&user_pubkey)?;

    let profit = (balance_after as i64 - balance_before as i64) as f64 / LAMPORTS_PER_SOL as f64;

    info!("   📊 Resultado con gestión de riesgo: {:.9} SOL", profit);

    Ok(profit)
}

async fn setup_wsol_account_safe(client: &RpcClient, wallet: &Keypair) -> Result<Pubkey> {
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;
    let wsol_account =
        spl_associated_token_account::get_associated_token_address(&user_pubkey, &wsol_mint);

    if client.get_account(&wsol_account).is_err() {
        let create_ata_ix =
            spl_associated_token_account::instruction::create_associated_token_account(
                &user_pubkey,
                &user_pubkey,
                &wsol_mint,
                &spl_token::id(),
            );

        let recent_blockhash = client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[create_ata_ix],
            Some(&user_pubkey),
            &[wallet],
            recent_blockhash,
        );

        let _signature = client.send_and_confirm_transaction(&transaction)?;
    }

    Ok(wsol_account)
}

async fn execute_safe_wrap(
    client: &RpcClient,
    wallet: &Keypair,
    wsol_account: &Pubkey,
    amount: u64,
) -> Result<Signature> {
    let user_pubkey = wallet.pubkey();
    let rent_exempt = client.get_minimum_balance_for_rent_exemption(165)?;

    let instructions = vec![
        system_instruction::transfer(&user_pubkey, wsol_account, amount + rent_exempt),
        spl_token::instruction::sync_native(&spl_token::id(), wsol_account)?,
    ];

    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&user_pubkey),
        &[wallet],
        recent_blockhash,
    );

    Ok(client.send_and_confirm_transaction(&transaction)?)
}

async fn execute_safe_unwrap(
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
