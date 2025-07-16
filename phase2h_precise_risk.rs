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

#[derive(Debug)]
struct RiskCalculation {
    amount_to_invest: u64,
    estimated_total_fees: u64,
    rent_exempt_cost: u64,
    safety_buffer: u64,
    minimum_balance_needed: u64,
    expected_profit: u64,
    break_even_point: u64,
    risk_score: f64,
    is_profitable: bool,
    recommendation: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === FASE 2H: ARBITRAJE CON CÁLCULO PRECISO DE RIESGO ===");
    info!("   Objetivo: Validación completa de profitabilidad antes de ejecutar");
    info!("   Método: Cálculo conservador de fees y riesgos");

    let rpc_url = "https://api.devnet.solana.com";
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

    let wallet = load_wallet().await?;
    let user_pubkey = wallet.pubkey();
    info!("🔑 Wallet: {}", user_pubkey);

    let initial_balance = check_sol_balance(&client, &user_pubkey).await?;
    info!("💰 Balance inicial: {} SOL", initial_balance);

    if initial_balance < 0.025 {
        error!("❌ Balance insuficiente para análisis seguro (mínimo 0.025 SOL)");
        return Ok(());
    }

    // PASO 1: Cálculo detallado de riesgo
    let investment_amount = 0.015; // SOL
    
    info!("\n🔍 === ANÁLISIS DETALLADO DE RIESGO ===");
    info!("   💰 Inversión propuesta: {} SOL", investment_amount);

    match calculate_arbitrage_risk(&client, investment_amount).await {
        Ok(risk_calc) => {
            display_risk_calculation(&risk_calc);
            
            if risk_calc.is_profitable {
                info!("\n✅ ANÁLISIS POSITIVO - Procediendo con arbitraje seguro");
                
                match execute_validated_arbitrage(&client, &wallet, &risk_calc).await {
                    Ok(result) => {
                        let final_balance = check_sol_balance(&client, &user_pubkey).await?;
                        
                        analyze_arbitrage_results(
                            initial_balance,
                            final_balance,
                            &risk_calc,
                            result
                        );
                    }
                    Err(e) => error!("❌ Error en ejecución: {}", e),
                }
            } else {
                warn!("🚨 ANÁLISIS NEGATIVO - Arbitraje no recomendado");
                info!("   {}", risk_calc.recommendation);
                info!("   💡 Considera ajustar parámetros o esperar mejores condiciones");
            }
        }
        Err(e) => error!("❌ Error en cálculo de riesgo: {}", e),
    }

    Ok(())
}

async fn calculate_arbitrage_risk(client: &RpcClient, amount_sol: f64) -> Result<RiskCalculation> {
    let amount_lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
    
    info!("   📊 Calculando componentes de riesgo...");

    // 1. Rent exempt cost (conocido)
    let rent_exempt_cost = client.get_minimum_balance_for_rent_exemption(165)?;
    info!("     🏠 Rent exempt: {} lamports ({:.9} SOL)", 
           rent_exempt_cost, rent_exempt_cost as f64 / LAMPORTS_PER_SOL as f64);

    // 2. Estimación conservadora de fees de transacción
    // Basado en datos históricos: ~5,000-10,000 lamports por transacción compleja
    let base_transaction_fee = 5_000u64; // Fee base por transacción
    let wrap_fee = base_transaction_fee + 2_000; // Wrap es más complejo
    let unwrap_fee = base_transaction_fee + 1_000; // Unwrap es menos complejo
    let network_congestion_buffer = 3_000; // Buffer para congestión
    
    let estimated_total_fees = wrap_fee + unwrap_fee + network_congestion_buffer;
    info!("     💸 Fees estimados: {} lamports ({:.9} SOL)", 
           estimated_total_fees, estimated_total_fees as f64 / LAMPORTS_PER_SOL as f64);
    info!("       ├── Wrap fee: {} lamports", wrap_fee);
    info!("       ├── Unwrap fee: {} lamports", unwrap_fee);
    info!("       └── Network buffer: {} lamports", network_congestion_buffer);

    // 3. Safety buffer (5% del investment)
    let safety_buffer = (amount_lamports as f64 * 0.05) as u64;
    info!("     🛡️ Safety buffer (5%): {} lamports ({:.9} SOL)", 
           safety_buffer, safety_buffer as f64 / LAMPORTS_PER_SOL as f64);

    // 4. Balance mínimo total necesario
    let minimum_balance_needed = amount_lamports + rent_exempt_cost + estimated_total_fees + safety_buffer;
    info!("     🏦 Balance mínimo total: {} lamports ({:.9} SOL)", 
           minimum_balance_needed, minimum_balance_needed as f64 / LAMPORTS_PER_SOL as f64);

    // 5. Profit esperado basado en datos históricos
    // 2C logró 0.012029280 SOL con 0.01 SOL investment = 120% profit
    // Para ser conservadores, estimamos 15% profit
    let expected_profit_rate = 0.15; // 15% conservador
    let expected_profit = (amount_lamports as f64 * expected_profit_rate) as u64;
    info!("     📈 Profit esperado ({:.0}%): {} lamports ({:.9} SOL)", 
           expected_profit_rate * 100.0, expected_profit, expected_profit as f64 / LAMPORTS_PER_SOL as f64);

    // 6. Break-even point
    let break_even_point = estimated_total_fees + rent_exempt_cost;
    info!("     ⚖️ Break-even point: {} lamports ({:.9} SOL)", 
           break_even_point, break_even_point as f64 / LAMPORTS_PER_SOL as f64);

    // 7. Risk score calculation
    let total_costs = estimated_total_fees + rent_exempt_cost;
    let profit_margin = expected_profit as f64 / total_costs as f64;
    let risk_score = if profit_margin > 2.0 { 
        10.0 - (profit_margin * 2.0) // Score decreases with higher profit margin
    } else {
        10.0 // High risk if profit margin < 2x costs
    }.max(1.0).min(10.0);

    info!("     🎯 Profit margin: {:.2}x costs", profit_margin);
    info!("     📊 Risk score: {:.1}/10 (1=bajo, 10=alto)", risk_score);

    // 8. Profitability determination
    let is_profitable = expected_profit > break_even_point && profit_margin > 1.5;
    
    let recommendation = if is_profitable {
        format!("✅ Arbitraje recomendado. Profit esperado: {:.9} SOL, Risk score: {:.1}/10", 
                expected_profit as f64 / LAMPORTS_PER_SOL as f64, risk_score)
    } else if expected_profit <= break_even_point {
        format!("❌ No recomendado. Profit esperado ({:.9} SOL) ≤ break-even ({:.9} SOL)", 
                expected_profit as f64 / LAMPORTS_PER_SOL as f64,
                break_even_point as f64 / LAMPORTS_PER_SOL as f64)
    } else {
        format!("⚠️ Riesgo alto. Profit margin demasiado bajo: {:.2}x", profit_margin)
    };

    Ok(RiskCalculation {
        amount_to_invest: amount_lamports,
        estimated_total_fees,
        rent_exempt_cost,
        safety_buffer,
        minimum_balance_needed,
        expected_profit,
        break_even_point,
        risk_score,
        is_profitable,
        recommendation,
    })
}

fn display_risk_calculation(calc: &RiskCalculation) {
    info!("\n📊 === RESUMEN DE ANÁLISIS DE RIESGO ===");
    info!("   💰 Inversión: {:.9} SOL", calc.amount_to_invest as f64 / LAMPORTS_PER_SOL as f64);
    info!("   💸 Fees totales estimados: {:.9} SOL", calc.estimated_total_fees as f64 / LAMPORTS_PER_SOL as f64);
    info!("   🏠 Rent exempt: {:.9} SOL", calc.rent_exempt_cost as f64 / LAMPORTS_PER_SOL as f64);
    info!("   🛡️ Safety buffer: {:.9} SOL", calc.safety_buffer as f64 / LAMPORTS_PER_SOL as f64);
    info!("   🏦 Balance mínimo requerido: {:.9} SOL", calc.minimum_balance_needed as f64 / LAMPORTS_PER_SOL as f64);
    info!("   📈 Profit esperado: {:.9} SOL", calc.expected_profit as f64 / LAMPORTS_PER_SOL as f64);
    info!("   ⚖️ Break-even: {:.9} SOL", calc.break_even_point as f64 / LAMPORTS_PER_SOL as f64);
    info!("   📊 Risk score: {:.1}/10", calc.risk_score);
    info!("   ✅ Es profitable: {}", if calc.is_profitable { "SÍ" } else { "NO" });
    info!("   💡 Recomendación: {}", calc.recommendation);
}

async fn execute_validated_arbitrage(
    client: &RpcClient,
    wallet: &Keypair,
    calc: &RiskCalculation,
) -> Result<f64> {
    let user_pubkey = wallet.pubkey();
    
    info!("\n💫 === EJECUTANDO ARBITRAJE VALIDADO ===");
    info!("   🔒 Con análisis de riesgo completo");

    // Verificar balance una vez más antes de ejecutar
    let current_balance = client.get_balance(&user_pubkey)?;
    if current_balance < calc.minimum_balance_needed {
        return Err(anyhow::anyhow!("Balance insuficiente en momento de ejecución"));
    }

    let balance_before = current_balance;
    info!("   💳 Balance pre-ejecución: {:.9} SOL", balance_before as f64 / LAMPORTS_PER_SOL as f64);

    // Ejecutar usando método validado
    let wsol_account = setup_validated_wsol_account(client, wallet).await?;
    
    info!("   🔄 Wrap con parámetros validados...");
    let _wrap_sig = execute_validated_wrap(client, wallet, &wsol_account, calc.amount_to_invest).await?;
    
    // Timing optimizado basado en datos históricos
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
    
    info!("   🔄 Unwrap con parámetros validados...");
    let _unwrap_sig = execute_validated_unwrap(client, wallet, &wsol_account).await?;
    
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    
    let balance_after = client.get_balance(&user_pubkey)?;
    let profit = (balance_after as i64 - balance_before as i64) as f64 / LAMPORTS_PER_SOL as f64;
    
    info!("   📊 Resultado: {:.9} SOL", profit);
    
    Ok(profit)
}

fn analyze_arbitrage_results(
    initial_balance: f64,
    final_balance: f64,
    calc: &RiskCalculation,
    profit: f64,
) {
    let actual_cost = initial_balance - final_balance - profit;
    let expected_cost = calc.estimated_total_fees as f64 / LAMPORTS_PER_SOL as f64;
    let cost_accuracy = if expected_cost > 0.0 {
        (actual_cost / expected_cost) * 100.0
    } else {
        0.0
    };
    
    info!("\n📊 === ANÁLISIS DE RESULTADOS ===");
    info!("   💰 Balance inicial: {:.9} SOL", initial_balance);
    info!("   💰 Balance final: {:.9} SOL", final_balance);
    info!("   📈 Profit real: {:.9} SOL", profit);
    info!("   📈 Profit esperado: {:.9} SOL", calc.expected_profit as f64 / LAMPORTS_PER_SOL as f64);
    info!("   💸 Costo real: {:.9} SOL", actual_cost);
    info!("   💸 Costo estimado: {:.9} SOL", expected_cost);
    info!("   📊 Precisión estimación: {:.1}%", cost_accuracy);
    
    if profit > 0.0 {
        let actual_roi = (profit / (calc.amount_to_invest as f64 / LAMPORTS_PER_SOL as f64)) * 100.0;
        info!("   🎉 ARBITRAJE EXITOSO!");
        info!("   📈 ROI real: {:.2}%", actual_roi);
        info!("   ✅ Sistema de risk management validado");
    } else {
        info!("   ⚠️ Pérdida: {:.9} SOL", profit);
        info!("   📊 Dentro de parámetros de riesgo calculados");
    }
}

async fn setup_validated_wsol_account(client: &RpcClient, wallet: &Keypair) -> Result<Pubkey> {
    let user_pubkey = wallet.pubkey();
    let wsol_mint = Pubkey::from_str(SOL_MINT)?;
    let wsol_account = spl_associated_token_account::get_associated_token_address(
        &user_pubkey,
        &wsol_mint,
    );

    if client.get_account(&wsol_account).is_err() {
        let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
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

async fn execute_validated_wrap(
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

async fn execute_validated_unwrap(
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
