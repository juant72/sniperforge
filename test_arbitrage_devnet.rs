use anyhow::Result;
use sniperforge::arbitrage::detector::ArbitrageDetector;
use sniperforge::arbitrage::types::ArbitrageOpportunity;
use sniperforge::shared::config_loader::ConfigLoader;
use sniperforge::shared::network_config::NetworkConfig;
use std::env;
use std::time::Duration;
use tracing::{info, error, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    info!("� === Bot de Arbitraje - DevNet Test ===");
    info!("==========================================");

    // Load DevNet configuration
    let config = ConfigLoader::load_network_config("devnet").await?;
    
    info!("✅ Configuración cargada:");
    info!("   Network: {}", config.network);
    info!("   RPC: {}", config.rpc_endpoint);
    info!("   Tokens disponibles: {}", config.token_addresses.len());
    
    // Get tokens from configuration (no hardcode)
    let sol_token = config.token_addresses.get("sol")
        .ok_or_else(|| anyhow::anyhow!("SOL token not found in config"))?;
    let usdc_token = config.token_addresses.get("usdc")  
        .ok_or_else(|| anyhow::anyhow!("USDC token not found in config"))?;
    let ray_token = config.token_addresses.get("ray")
        .ok_or_else(|| anyhow::anyhow!("RAY token not found in config"))?;

    info!("🔧 Tokens configurados:");
    info!("   SOL: {} ({})", sol_token.address, sol_token.symbol);
    info!("   USDC: {} ({})", usdc_token.address, usdc_token.symbol);
    info!("   RAY: {} ({})", ray_token.address, ray_token.symbol);

    // Verify arbitrage configuration
    if let Some(arbitrage_settings) = &config.arbitrage_settings {
        info!("⚙️ Configuración de arbitraje:");
        info!("   Min profit threshold: {:.2}%", arbitrage_settings.min_profit_threshold * 100.0);
        info!("   Max slippage: {:.2}%", arbitrage_settings.max_slippage * 100.0);
        info!("   Detection interval: {}ms", arbitrage_settings.detection_interval_ms);
        info!("   Enabled: {}", arbitrage_settings.enabled);
    }

    // Create arbitrage detector
    info!("\n🔍 Inicializando detector de arbitraje...");
    let detector = ArbitrageDetector::new(config.clone()).await?;
    info!("✅ Detector inicializado correctamente");

    // Test arbitrage detection with real tokens
    info!("\n📊 === Iniciando detección de arbitraje ===");
    
    // Test 1: SOL -> USDC
    info!("\n� Test 1: SOL -> USDC");
    test_arbitrage_pair(
        &detector,
        &sol_token.address,
        &usdc_token.address,
        0.001, // 0.001 SOL
        &sol_token.symbol,
        &usdc_token.symbol,
    ).await?;
    
    // Test 2: SOL -> RAY
    info!("\n📊 Test 2: SOL -> RAY");
    test_arbitrage_pair(
        &detector,
        &sol_token.address,
        &ray_token.address,
        0.001, // 0.001 SOL
        &sol_token.symbol,
        &ray_token.symbol,
    ).await?;
    
    // Test 3: USDC -> RAY
    info!("\n📊 Test 3: USDC -> RAY");
    test_arbitrage_pair(
        &detector,
        &usdc_token.address,
        &ray_token.address,
        1.0, // 1 USDC
        &usdc_token.symbol,
        &ray_token.symbol,
    ).await?;

    info!("\n🎯 === Test de arbitraje completado ===");
    Ok(())
}

async fn test_arbitrage_pair(
    detector: &ArbitrageDetector,
    from_token: &str,
    to_token: &str,
    amount: f64,
    from_symbol: &str,
    to_symbol: &str,
) -> Result<()> {
    info!("   Testing {} {} -> {}", amount, from_symbol, to_symbol);
    info!("   From: {}", from_token);
    info!("   To: {}", to_token);
    
    match detector.detect_opportunities(from_token, to_token, amount).await {
        Ok(opportunities) => {
            if opportunities.is_empty() {
                warn!("   ❌ No arbitrage opportunities found");
            } else {
                info!("   ✅ {} opportunities found:", opportunities.len());
                for (i, opp) in opportunities.iter().enumerate() {
                    info!("     [{}] {} -> {}", i+1, opp.buy_dex, opp.sell_dex);
                    info!("         Profit: {:.6} SOL ({:.2}%)", 
                          opp.profit_amount, opp.profit_percentage * 100.0);
                    info!("         Buy Price: {:.6}", opp.buy_price);
                    info!("         Sell Price: {:.6}", opp.sell_price);
                    info!("         Spread: {:.4}%", (opp.sell_price - opp.buy_price) / opp.buy_price * 100.0);
                }
            }
        }
        Err(e) => {
            error!("   ❌ Error detecting opportunities: {}", e);
        }
    }
    
    Ok(())
}
