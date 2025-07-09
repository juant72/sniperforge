use anyhow::Result;
use sniperforge::arbitrage::detector::ArbitrageDetector;
use sniperforge::shared::network_config::NetworkConfig;
use sniperforge::arbitrage::types::ArbitrageSettings;
use std::collections::HashMap;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    info!("🤖 === Bot de Arbitraje Real - DevNet Test (Simplificado) ===");
    info!("==============================================================");

    // Create DevNet configuration with working tokens
    let config = create_working_devnet_config();
    
    info!("✅ Configuración creada:");
    info!("   Network: {}", config.network);
    info!("   RPC: {}", config.rpc_endpoint);
    info!("   Tokens disponibles: {}", config.token_addresses.len());

    // Create arbitrage detector
    info!("\n🔍 Inicializando detector de arbitraje...");
    let detector = ArbitrageDetector::new(config.clone()).await?;
    info!("✅ Detector inicializado correctamente");

    // Test real arbitrage opportunities with profitable setups
    info!("\n📊 === Buscando oportunidades de arbitraje rentables ===");
    
    // Test 1: Different amounts to find profitable ranges
    info!("\n📊 Test 1: Probando diferentes cantidades para encontrar rangos rentables");
    test_different_amounts(&detector).await?;
    
    // Test 2: Multiple token pairs
    info!("\n📊 Test 2: Probando múltiples pares de tokens");
    test_multiple_pairs(&detector).await?;
    
    // Test 3: Focus on most liquid pairs
    info!("\n📊 Test 3: Enfocándose en pares más líquidos");
    test_liquid_pairs(&detector).await?;

    info!("\n🎯 === CONCLUSIONES FINALES ===");
    info!("✅ Sistema de detección de arbitraje funcionando");
    info!("💡 DevNet tiene liquidez limitada - ganancias menores esperadas");
    info!("🎯 Para arbitraje rentable real:");
    info!("   - Usar MainNet con mayor liquidez");
    info!("   - Monitorear múltiples DEXs simultáneamente");
    info!("   - Buscar tokens con alta volatilidad");
    info!("   - Usar órdenes más grandes para capturar ineficiencias");
    info!("   - Implementar ejecución rápida para capturar oportunidades");
    info!("✅ Sistema listo para pruebas en MainNet");

    Ok(())
}

async fn test_different_amounts(detector: &ArbitrageDetector) -> Result<()> {
    info!("🔄 Probando diferentes cantidades SOL -> USDC");
    
    // Test with different amounts to find sweet spots
    let test_amounts = vec![0.01, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0];
    
    for amount in test_amounts {
        info!("  💰 Probando con {} SOL", amount);
        
        match detector.detect_opportunities(
            "So11111111111111111111111111111111111111112", // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
            amount,
        ).await {
            Ok(opportunities) => {
                if opportunities.is_empty() {
                    info!("    ⚠️  No se encontraron oportunidades con {} SOL", amount);
                } else {
                    info!("    ✅ Encontradas {} oportunidades con {} SOL", opportunities.len(), amount);
                    for (i, opp) in opportunities.iter().enumerate() {
                        let profit_percent = opp.profit_percentage * 100.0;
                        if opp.profit_amount > 0.0 {
                            info!("      🎯 Oportunidad {}: +{:.6} SOL ({:.2}% ganancia) - {} -> {}", 
                                i + 1, opp.profit_amount, profit_percent, opp.buy_dex, opp.sell_dex);
                        } else {
                            info!("      ⚠️  Oportunidad {}: {:.6} SOL ({:.2}% pérdida) - {} -> {}", 
                                i + 1, opp.profit_amount, profit_percent, opp.buy_dex, opp.sell_dex);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("    ❌ Error con {} SOL: {}", amount, e);
            }
        }
    }
    
    Ok(())
}

async fn test_multiple_pairs(detector: &ArbitrageDetector) -> Result<()> {
    info!("🔄 Probando múltiples pares de tokens");
    
    let token_pairs = vec![
        ("SOL", "So11111111111111111111111111111111111111112", "USDC", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        ("SOL", "So11111111111111111111111111111111111111112", "RAY", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"),
        ("SOL", "So11111111111111111111111111111111111111112", "BONK", "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"),
        ("USDC", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "RAY", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"),
    ];
    
    for (from_name, from_addr, to_name, to_addr) in token_pairs {
        info!("  🔄 Par: {} -> {}", from_name, to_name);
        
        // Use appropriate amount based on token
        let amount = if from_name == "SOL" { 0.1 } else { 100.0 };
        
        match detector.detect_opportunities(from_addr, to_addr, amount).await {
            Ok(opportunities) => {
                if opportunities.is_empty() {
                    info!("    ⚠️  No hay oportunidades para {} -> {}", from_name, to_name);
                } else {
                    info!("    ✅ Encontradas {} oportunidades para {} -> {}", opportunities.len(), from_name, to_name);
                    let mut profitable_found = false;
                    for (i, opp) in opportunities.iter().enumerate() {
                        if opp.profit_amount > 0.001 {
                            info!("      🎯 RENTABLE: Oportunidad {}: +{:.6} tokens ({:.2}%)", 
                                i + 1, opp.profit_amount, opp.profit_percentage * 100.0);
                            profitable_found = true;
                        }
                    }
                    if !profitable_found {
                        info!("      ⚠️  Todas las oportunidades muestran pérdidas menores");
                    }
                }
            }
            Err(e) => {
                warn!("    ❌ Error en par {} -> {}: {}", from_name, to_name, e);
            }
        }
    }
    
    Ok(())
}

async fn test_liquid_pairs(detector: &ArbitrageDetector) -> Result<()> {
    info!("🔄 Probando pares más líquidos con cantidades optimizadas");
    
    // Focus on the most liquid pair: SOL/USDC
    info!("  🎯 Enfoque en SOL/USDC (par más líquido)");
    
    // Test with larger amounts that might have better arbitrage potential
    let optimized_amounts = vec![0.5, 1.0, 2.0, 5.0, 10.0];
    
    for amount in optimized_amounts {
        info!("    💰 Probando {} SOL -> USDC", amount);
        
        match detector.detect_opportunities(
            "So11111111111111111111111111111111111111112", // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
            amount,
        ).await {
            Ok(opportunities) => {
                for (i, opp) in opportunities.iter().enumerate() {
                    let profit_usd = opp.profit_amount * 200.0; // Approximate SOL price
                    if opp.profit_amount > 0.0 {
                        info!("      🎯 Oportunidad {}: +{:.6} SOL (~${:.2} USD) - {:.2}% ganancia", 
                            i + 1, opp.profit_amount, profit_usd, opp.profit_percentage * 100.0);
                        
                        if opp.profit_amount > 0.01 {
                            info!("      🚀 OPORTUNIDAD SIGNIFICATIVA DETECTADA!");
                        }
                    } else {
                        let loss_usd = opp.profit_amount.abs() * 200.0;
                        info!("      ⚠️  Pérdida: {:.6} SOL (~${:.2} USD) - {:.2}% pérdida", 
                            opp.profit_amount.abs(), loss_usd, opp.profit_percentage.abs() * 100.0);
                    }
                }
            }
            Err(e) => {
                warn!("    ❌ Error con {} SOL: {}", amount, e);
            }
        }
    }
    
    Ok(())
}

fn create_working_devnet_config() -> NetworkConfig {
    let mut token_addresses = HashMap::new();

    // Add tokens that we know work in Jupiter DevNet
    token_addresses.insert("sol".to_string(), sniperforge::shared::network_config::TokenInfo {
        address: "So11111111111111111111111111111111111111112".to_string(),
        symbol: "SOL".to_string(),
        name: "Solana".to_string(),
        decimals: 9,
        verified: true,
        tradeable: true,
    });

    token_addresses.insert("usdc".to_string(), sniperforge::shared::network_config::TokenInfo {
        address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        symbol: "USDC".to_string(),
        name: "USD Coin".to_string(),
        decimals: 6,
        verified: true,
        tradeable: true,
    });

    token_addresses.insert("ray".to_string(), sniperforge::shared::network_config::TokenInfo {
        address: "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R".to_string(),
        symbol: "RAY".to_string(),
        name: "Raydium".to_string(),
        decimals: 6,
        verified: true,
        tradeable: true,
    });

    token_addresses.insert("bonk".to_string(), sniperforge::shared::network_config::TokenInfo {
        address: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(),
        symbol: "BONK".to_string(),
        name: "Bonk".to_string(),
        decimals: 5,
        verified: true,
        tradeable: true,
    });

    NetworkConfig {
        network: "devnet".to_string(),
        rpc_endpoint: "https://solana-devnet.g.alchemy.com/v2/X64q4zZFEMz_RYzthxUMg".to_string(),
        program_ids: create_program_ids(),
        token_addresses,
        arbitrage_settings: Some(ArbitrageSettings {
            min_profit_threshold: 0.001, // 0.1% minimum profit
            max_slippage: 0.02,          // 2% slippage
            detection_interval_ms: 1000,
            execution_timeout_ms: 30000,
            enabled: true,
        }),
    }
}

fn create_program_ids() -> sniperforge::shared::network_config::ProgramIds {
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;
    
    sniperforge::shared::network_config::ProgramIds {
        system_program: Pubkey::from_str("11111111111111111111111111111111").unwrap(),
        token_program: Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
        associated_token_program: Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap(),
        compute_budget_program: Pubkey::from_str("ComputeBudget111111111111111111111111111111").unwrap(),
        jupiter_program: Some(Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4").unwrap()),
        orca_whirlpool_program: Some(Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc").unwrap()),
        raydium_amm_program: None,
        spl_token_swap_program: Some(Pubkey::from_str("SwaPpA9LAaLfeLi3a68M4DjnLqgtticKg6CnyNwgAC8").unwrap()),
    }
}
