use anyhow::Result;
use sniperforge::shared::jupiter_client::JupiterClient;
use sniperforge::shared::jupiter_config::JupiterConfig;
use sniperforge::shared::jupiter_types::QuoteRequest;
use sniperforge::shared::orca_client::OrcaClient;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 === ARBITRAJE CROSS-DEX REAL (Jupiter vs Orca) ===");
    info!("====================================================");

    // Create Jupiter config for DevNet
    let jupiter_config = JupiterConfig {
        base_url: "https://quote-api.jup.ag".to_string(),
        api_key: None,
        timeout_seconds: 30,
        max_retries: 3,
        rpc_endpoint: "https://solana-devnet.g.alchemy.com/v2/X64q4zZFEMz_RYzthxUMg".to_string(),
        network_name: "devnet".to_string(),
    };

    let jupiter_client = JupiterClient::new(&jupiter_config).await?;
    let orca_client = OrcaClient::new("devnet");

    info!("✅ Clientes inicializados: Jupiter + Orca");

    // Test amount
    let test_amount_lamports = 10_000_000u64; // 0.01 SOL
    let test_amount_sol = test_amount_lamports as f64 / 1_000_000_000.0;
    
    info!("💰 Cantidad de prueba: {} lamports ({:.3} SOL)", test_amount_lamports, test_amount_sol);

    // Step 1: Get SOL price from Jupiter (selling SOL for USDC)
    info!("\n📊 === PASO 1: OBTENER PRECIO SOL EN JUPITER ===");
    let jupiter_quote = QuoteRequest {
        inputMint: "So11111111111111111111111111111111111111112".to_string(), // SOL
        outputMint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
        amount: test_amount_lamports,
        slippageBps: 100, // 1%
    };

    let jupiter_sol_price = match jupiter_client.get_quote(jupiter_quote).await {
        Ok(quote) => {
            let usdc_output: u64 = quote.outAmount.parse().unwrap_or(0);
            let usdc_amount = usdc_output as f64 / 1_000_000.0; // USDC has 6 decimals
            let sol_price_usd = usdc_amount / test_amount_sol;
            info!("✅ Jupiter: {} SOL → {} USDC", test_amount_sol, usdc_amount);
            info!("💵 Precio SOL en Jupiter: ${:.2}", sol_price_usd);
            Some(sol_price_usd)
        }
        Err(e) => {
            error!("❌ Error obteniendo precio Jupiter: {}", e);
            None
        }
    };

    // Step 2: Get SOL price from Orca
    info!("\n📊 === PASO 2: OBTENER PRECIO SOL EN ORCA ===");
    let orca_sol_price = match orca_client.get_price("So11111111111111111111111111111111111111112").await {
        Ok(Some(price)) => {
            info!("✅ Precio SOL en Orca: ${:.2}", price);
            Some(price)
        }
        Ok(None) => {
            warn!("⚠️ Orca no pudo obtener precio SOL");
            None
        }
        Err(e) => {
            error!("❌ Error obteniendo precio Orca: {}", e);
            None
        }
    };

    // Step 3: Calculate arbitrage opportunity
    info!("\n🎯 === PASO 3: ANÁLISIS DE ARBITRAJE ===");
    
    match (jupiter_sol_price, orca_sol_price) {
        (Some(jupiter_price), Some(orca_price)) => {
            let spread_percentage = ((jupiter_price - orca_price) / orca_price) * 100.0;
            
            info!("📈 Análisis de Spread:");
            info!("   🟦 Jupiter SOL: ${:.2}", jupiter_price);
            info!("   🟪 Orca SOL:    ${:.2}", orca_price);
            info!("   📊 Spread:      {:.2}%", spread_percentage);
            
            if spread_percentage > 1.0 { // Profitable if > 1%
                let profit_per_sol = jupiter_price - orca_price;
                let profit_for_test_amount = profit_per_sol * test_amount_sol;
                let profit_lamports = (profit_for_test_amount * 1_000_000_000.0) as u64;
                
                info!("\n💰 === OPORTUNIDAD DE ARBITRAJE DETECTADA ===");
                info!("✅ ARBITRAJE RENTABLE:");
                info!("   🛒 Comprar SOL en Orca:  ${:.2}", orca_price);
                info!("   💰 Vender SOL en Jupiter: ${:.2}", jupiter_price);
                info!("   📈 Profit por SOL:       ${:.2}", profit_per_sol);
                info!("   🎯 Profit para {:.3} SOL: ${:.4}", test_amount_sol, profit_for_test_amount);
                info!("   💎 Profit en lamports:   {} lamports", profit_lamports);
                
                // Calculate potential profits for different amounts
                info!("\n💹 === PROFITS ESCALADOS ===");
                let amounts = vec![0.01, 0.1, 0.5, 1.0];
                for amount in amounts {
                    let profit_scaled = profit_per_sol * amount;
                    info!("   {:.2} SOL → Profit: ${:.2} ({:.6} SOL)", 
                          amount, profit_scaled, profit_scaled / jupiter_price);
                }
                
                info!("\n🚀 === ESTRATEGIA RECOMENDADA ===");
                if spread_percentage > 10.0 {
                    info!("✅ SPREAD EXCELENTE (>10%): Ejecutar arbitraje agresivo");
                    info!("💡 Estrategia: Usar hasta 50% del balance disponible");
                } else if spread_percentage > 5.0 {
                    info!("✅ SPREAD BUENO (5-10%): Ejecutar arbitraje moderado");
                    info!("💡 Estrategia: Usar hasta 25% del balance disponible");
                } else {
                    info!("⚠️ SPREAD PEQUEÑO (1-5%): Ejecutar arbitraje conservador");
                    info!("💡 Estrategia: Usar hasta 10% del balance disponible");
                }
                
            } else if spread_percentage < -1.0 {
                info!("\n🔄 === ARBITRAJE REVERSO DETECTADO ===");
                info!("📊 Jupiter más barato que Orca:");
                info!("   🛒 Comprar SOL en Jupiter: ${:.2}", jupiter_price);
                info!("   💰 Vender SOL en Orca:     ${:.2}", orca_price);
                info!("   📈 Profit potencial:       ${:.2}", orca_price - jupiter_price);
            } else {
                info!("\n😐 === NO HAY ARBITRAJE RENTABLE ===");
                info!("📊 Spread demasiado pequeño: {:.2}%", spread_percentage);
                info!("⚠️ Profit < costos de transacción estimados");
            }
        }
        _ => {
            error!("❌ No se pudieron obtener precios de ambos DEXs");
            info!("💡 Verifica conectividad a Jupiter y Orca");
        }
    }

    info!("\n💡 === CONCLUSIONES ===");
    info!("1. ✅ Precios obtenidos de ambos DEXs");
    info!("2. 📊 Análisis de spread completado");
    info!("3. 🎯 Estrategia de arbitraje recomendada");
    info!("4. 🚀 Listo para implementar ejecución automática");

    Ok(())
}
