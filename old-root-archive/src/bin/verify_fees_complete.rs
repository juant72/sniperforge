use anyhow::Result;
use tracing::{info, warn, error};
use sniperforge::arbitrage_settings::ArbitrageSettings;

/// Verificación exhaustiva de fees para trading real
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("🧮 === VERIFICACIÓN EXHAUSTIVA DE FEES ===");
    info!("   🎯 Analizando oportunidades detectadas en simulación");
    info!("   💰 Validando rentabilidad después de TODOS los fees");
    
    // Cargar configuración
    let settings = ArbitrageSettings::load_default()?;
    
    // === CASOS DE PRUEBA BASADOS EN OPORTUNIDADES REALES DETECTADAS ===
    
    // 1. Caso RAY - La mejor oportunidad detectada (1.36% profit)
    info!("\n🔍 === ANÁLISIS CASO 1: RAY (Mejor oportunidad) ===");
    verify_opportunity_fees(
        "RAY",
        0.01,    // Trade amount configurado en JSON
        1.36,    // 1.36% profit detectado
        "Raydium",
        "Jupiter",
    ).await?;
    
    // 2. Caso WIF - Oportunidad media-alta (0.49% profit)
    info!("\n🔍 === ANÁLISIS CASO 2: WIF (Oportunidad media-alta) ===");
    verify_opportunity_fees(
        "WIF",
        0.01,
        0.49,    // 0.49% profit detectado
        "DexScreener",
        "Jupiter",
    ).await?;
    
    // 3. Caso PYTH - Oportunidad media (0.28% profit)
    info!("\n🔍 === ANÁLISIS CASO 3: PYTH (Oportunidad media) ===");
    verify_opportunity_fees(
        "PYTH",
        0.01,
        0.28,    // 0.28% profit detectado
        "DexScreener",
        "Jupiter",
    ).await?;
    
    // 4. Caso BONK - Oportunidad baja (0.16% profit)
    info!("\n🔍 === ANÁLISIS CASO 4: BONK (Oportunidad baja) ===");
    verify_opportunity_fees(
        "BONK",
        0.01,
        0.16,    // 0.16% profit detectado
        "DexScreener",
        "Raydium",
    ).await?;
    
    // 5. Caso extremo - Trade más grande
    info!("\n🔍 === ANÁLISIS CASO 5: RAY con trade más grande ===");
    verify_opportunity_fees(
        "RAY",
        0.05,    // Trade 5x más grande
        1.36,    // Mismo profit %
        "Raydium",
        "Jupiter",
    ).await?;
    
    info!("\n✅ === VERIFICACIÓN COMPLETA TERMINADA ===");
    
    Ok(())
}

/// Verificar fees para una oportunidad específica
async fn verify_opportunity_fees(
    token: &str,
    trade_amount_sol: f64,
    profit_percent: f64,
    dex_buy: &str,
    dex_sell: &str,
) -> Result<()> {
    info!("🪙 Token: {} | 💰 Amount: {:.6} SOL | 📈 Profit: {:.2}%", 
          token, trade_amount_sol, profit_percent);
    info!("📊 Route: Buy {} → Sell {}", dex_buy, dex_sell);
    
    // Calcular expected output
    let expected_output_sol = trade_amount_sol * (1.0 + profit_percent / 100.0);
    let gross_profit_sol = expected_output_sol - trade_amount_sol;
    
    info!("   📈 Expected output: {:.6} SOL", expected_output_sol);
    info!("   💰 Gross profit: {:.6} SOL", gross_profit_sol);
    
    // === CÁLCULO DETALLADO DE FEES ===
    
    // 1. Fees de red Solana (2 transacciones: buy + sell)
    let base_tx_fee = 0.000005 * 2.0; // 5K lamports × 2 TX
    let priority_fee = 0.0001 * 2.0;  // 100K lamports × 2 TX (high priority)
    let compute_fee = 0.0002 * 2.0;   // 200K lamports × 2 TX (complex operations)
    let total_network_fees = base_tx_fee + priority_fee + compute_fee;
    
    info!("   🔧 FEES DE RED:");
    info!("      📋 Base TX fees: {:.6} SOL", base_tx_fee);
    info!("      ⚡ Priority fees: {:.6} SOL", priority_fee);
    info!("      💻 Compute fees: {:.6} SOL", compute_fee);
    info!("      🔧 Total red: {:.6} SOL", total_network_fees);
    
    // 2. Fees de Jupiter (plataforma)
    let jupiter_fee_bps = 4; // 0.04% Jupiter real fee
    let jupiter_fee_sol = trade_amount_sol * (jupiter_fee_bps as f64 / 10000.0);
    
    info!("   🎯 FEES DE JUPITER:");
    info!("      🏛️ Platform fee: {} bps ({:.3}%)", jupiter_fee_bps, jupiter_fee_bps as f64 / 100.0);
    info!("      💰 Jupiter fee: {:.6} SOL", jupiter_fee_sol);
    
    // 3. Slippage (específico por token)
    let slippage_bps = match token {
        "RAY" => 15,   // 0.15% (líquido)
        "WIF" => 30,   // 0.30% (menos líquido)
        "PYTH" => 20,  // 0.20% (medio)
        "BONK" => 25,  // 0.25% (volátil)
        _ => 20,       // 0.20% default
    };
    let slippage_cost = trade_amount_sol * (slippage_bps as f64 / 10000.0);
    
    info!("   📉 SLIPPAGE:");
    info!("      📊 Expected slippage: {} bps ({:.2}%)", slippage_bps, slippage_bps as f64 / 100.0);
    info!("      💸 Slippage cost: {:.6} SOL", slippage_cost);
    
    // 4. Price Impact (crece con el tamaño del trade)
    let price_impact_bps = match token {
        "RAY" => if trade_amount_sol > 0.02 { 10 } else { 5 },   // 0.05-0.10%
        "WIF" => if trade_amount_sol > 0.02 { 20 } else { 10 },  // 0.10-0.20%
        "PYTH" => if trade_amount_sol > 0.02 { 15 } else { 8 },  // 0.08-0.15%
        "BONK" => if trade_amount_sol > 0.02 { 25 } else { 15 }, // 0.15-0.25%
        _ => 10,
    };
    let price_impact_cost = trade_amount_sol * (price_impact_bps as f64 / 10000.0);
    
    info!("   📈 PRICE IMPACT:");
    info!("      📊 Price impact: {} bps ({:.2}%)", price_impact_bps, price_impact_bps as f64 / 100.0);
    info!("      💸 Impact cost: {:.6} SOL", price_impact_cost);
    
    // 5. DEX Trading Fees (cada DEX cobra fee)
    let dex_fee_bps = 25; // 0.25% típico para Raydium/Orca/etc
    let dex_fee_cost = trade_amount_sol * (dex_fee_bps as f64 / 10000.0) * 2.0; // 2 DEXs
    
    info!("   🏪 DEX FEES:");
    info!("      💱 DEX trading fee: {} bps × 2 DEXs", dex_fee_bps);
    info!("      💰 DEX fees cost: {:.6} SOL", dex_fee_cost);
    
    // 6. MEV Protection (Jito)
    let jito_tip = 0.00001;  // 10K lamports tip
    let bundle_fee = 0.000005; // 5K lamports bundle submission
    let mev_protection_cost = jito_tip + bundle_fee;
    
    info!("   🛡️ MEV PROTECTION:");
    info!("      💰 Jito tip: {:.6} SOL", jito_tip);
    info!("      📦 Bundle fee: {:.6} SOL", bundle_fee);
    info!("      🛡️ Total MEV: {:.6} SOL", mev_protection_cost);
    
    // === CÁLCULO TOTAL ===
    let total_fees = total_network_fees + jupiter_fee_sol + slippage_cost + 
                    price_impact_cost + dex_fee_cost + mev_protection_cost;
    
    // Margen de seguridad conservador (20% extra)
    let safety_margin = total_fees * 0.2;
    let total_fees_with_safety = total_fees + safety_margin;
    
    // Profit neto final
    let net_profit = gross_profit_sol - total_fees_with_safety;
    let net_profit_percent = (net_profit / trade_amount_sol) * 100.0;
    
    info!("   💰 === RESUMEN FINANCIERO ===");
    info!("      📈 Gross profit: {:.6} SOL ({:.2}%)", gross_profit_sol, profit_percent);
    info!("      💸 Total fees: {:.6} SOL", total_fees);
    info!("      🛡️ Safety margin: {:.6} SOL", safety_margin);
    info!("      💸 Total fees + safety: {:.6} SOL", total_fees_with_safety);
    info!("      🎯 NET PROFIT: {:.6} SOL ({:.2}%)", net_profit, net_profit_percent);
    info!("      📊 Fees ratio: {:.1}% of trade", (total_fees_with_safety / trade_amount_sol) * 100.0);
    
    // === DECISIÓN FINAL ===
    if net_profit > 0.0 {
        if net_profit_percent > 0.3 {
            info!("      ✅ RECOMENDACIÓN: EJECUTAR - Rentable con {:.2}% ROI", net_profit_percent);
        } else {
            warn!("      ⚠️ RECOMENDACIÓN: EVALUAR - Profit muy bajo ({:.2}% ROI)", net_profit_percent);
        }
    } else {
        error!("      ❌ RECOMENDACIÓN: NO EJECUTAR - Pérdida de {:.6} SOL", net_profit.abs());
    }
    
    // === BREAKDOWN DE FEES ===
    info!("   📊 === BREAKDOWN DE FEES ===");
    info!("      🔧 Network fees: {:.1}%", (total_network_fees / total_fees_with_safety) * 100.0);
    info!("      🎯 Jupiter fees: {:.1}%", (jupiter_fee_sol / total_fees_with_safety) * 100.0);
    info!("      📉 Slippage: {:.1}%", (slippage_cost / total_fees_with_safety) * 100.0);
    info!("      📈 Price impact: {:.1}%", (price_impact_cost / total_fees_with_safety) * 100.0);
    info!("      🏪 DEX fees: {:.1}%", (dex_fee_cost / total_fees_with_safety) * 100.0);
    info!("      🛡️ MEV protection: {:.1}%", (mev_protection_cost / total_fees_with_safety) * 100.0);
    info!("      🛡️ Safety margin: {:.1}%", (safety_margin / total_fees_with_safety) * 100.0);
    
    Ok(())
}
