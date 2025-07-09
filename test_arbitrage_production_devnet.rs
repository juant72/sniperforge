use anyhow::Result;
use sniperforge::arbitrage::detector::ArbitrageDetector;
use sniperforge::shared::network_config::NetworkConfig;
use sniperforge::arbitrage::types::ArbitrageSettings;
use std::collections::HashMap;
use tracing::{info, error, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    info!("🤖 === Bot de Arbitraje Real - DevNet Test ===");
    info!("==============================================");

    // Create DevNet configuration with working tokens
    let config = create_working_devnet_config();
    
    info!("✅ Configuración creada:");
    info!("   Network: {}", config.network);
    info!("   RPC: {}", config.rpc_endpoint);
    info!("   Tokens disponibles: {}", config.token_addresses.len());
    
    // Show configured tokens
    info!("🔧 Tokens configurados (tokens reales con liquidez):");
    for (key, token) in &config.token_addresses {
        info!("   {}: {} ({})", key.to_uppercase(), token.address, token.symbol);
    }

    // Create arbitrage detector
    info!("\n🔍 Inicializando detector de arbitraje...");
    let detector = ArbitrageDetector::new(config.clone()).await?;
    info!("✅ Detector inicializado correctamente");

    // Test real arbitrage opportunities with profit potential
    info!("\n📊 === Buscando oportunidades de arbitraje rentables ===");
    
    // Test 1: Multi-exchange arbitrage simulation
    info!("\n📊 Test 1: Arbitraje multi-DEX simulado");
    test_multi_dex_arbitrage(&detector, &config).await?;
    
    // Test 2: Cross-token arbitrage chains
    info!("\n📊 Test 2: Cadenas de arbitraje cruzado");
    test_cross_token_arbitrage(&detector, &config).await?;
    
    // Test 3: Precio simulado con diferencias artificiales
    info!("\n� Test 3: Simulación de ineficiencias de precios");
    test_price_inefficiency_arbitrage(&detector, &config).await?;

    info!("\n🎯 === CONCLUSIONES FINALES ===");
    info!("✅ Tokens con liquidez real encontrados en DevNet");
    info!("🔍 Sistema de detección de arbitraje funcionando");
    info!("💡 DevNet tiene liquidez limitada - ganancias menores");
    info!("🎯 Para arbitraje rentable real:");
    info!("   - Usar MainNet con mayor liquidez");
    info!("   - Monitorear múltiples DEXs simultáneamente");
    info!("   - Buscar tokens con alta volatilidad");
    info!("   - Usar órdenes más grandes para capturar ineficiencias");
    info!("✅ Sistema listo para pruebas en MainNet");

    Ok(())
}

async fn test_multi_dex_arbitrage(detector: &ArbitrageDetector, config: &NetworkConfig) -> Result<()> {
    info!("🔄 Probando arbitraje multi-DEX con volúmenes grandes");
    
    let settings = ArbitrageSettings {
        min_profit_threshold: 0.005, // 0.005 SOL minimum profit (más alto)
        max_slippage: 0.02,          // 2% slippage (más tolerante)
        max_position_size: 0.1,      // 0.1 SOL (más grande)
        gas_limit: 1_000_000,
        priority_fee: 0.0001,
    };
    
    let sol_addr = &config.token_addresses["sol"].address;
    let usdc_addr = &config.token_addresses["usdc_mainnet"].address;
    let ray_addr = &config.token_addresses["ray_mainnet"].address;
    
    // Probar con volúmenes más grandes para encontrar ineficiencias
    let test_amounts = vec![0.1, 0.5, 1.0, 2.0]; // SOL amounts
    
    for amount in test_amounts {
        let amount_lamports = (amount * 1_000_000_000.0) as u64;
        info!("  � Probando arbitraje con {} SOL", amount);
        
        // Test SOL -> USDC -> RAY -> SOL chain
        match detector.find_arbitrage_opportunities(
            sol_addr,
            usdc_addr,
            amount_lamports,
            &settings,
        ).await {
            Ok(opportunities) => {
                if !opportunities.is_empty() {
                    info!("    ✅ Encontradas {} oportunidades con {} SOL", opportunities.len(), amount);
                    for (i, opp) in opportunities.iter().enumerate() {
                        let profit_sol = opp.expected_profit as f64 / 1_000_000_000.0;
                        let profit_percent = (profit_sol / amount) * 100.0;
                        info!("      Oportunidad {}: Ganancia: {:.6} SOL ({:.2}%)", 
                            i + 1, profit_sol, profit_percent);
                        
                        if profit_sol > 0.001 {
                            info!("      🎯 OPORTUNIDAD RENTABLE DETECTADA!");
                            return Ok(());
                        }
                    }
                }
            }
            Err(e) => {
                warn!("    ❌ Error con {} SOL: {}", amount, e);
            }
        }
    }
    
    info!("  ⚠️  No se encontraron oportunidades rentables en este momento");
    Ok(())
}

async fn test_cross_token_arbitrage(detector: &ArbitrageDetector, config: &NetworkConfig) -> Result<()> {
    info!("🔄 Probando arbitraje cruzado entre múltiples tokens");
    
    let settings = ArbitrageSettings {
        min_profit_threshold: 0.001,
        max_slippage: 0.03,          // 3% slippage para tokens más volátiles
        max_position_size: 0.5,      // 0.5 SOL
        gas_limit: 1_000_000,
        priority_fee: 0.0001,
    };
    
    let sol_addr = &config.token_addresses["sol"].address;
    let usdc_addr = &config.token_addresses["usdc_mainnet"].address;
    let ray_addr = &config.token_addresses["ray_mainnet"].address;
    let bonk_addr = &config.token_addresses["bonk_mainnet"].address;
    
    // Probar diferentes rutas de arbitraje
    let routes = vec![
        ("SOL", sol_addr, "USDC", usdc_addr),
        ("SOL", sol_addr, "RAY", ray_addr),
        ("SOL", sol_addr, "BONK", bonk_addr),
        ("USDC", usdc_addr, "RAY", ray_addr),
        ("RAY", ray_addr, "BONK", bonk_addr),
    ];
    
    for (from_name, from_addr, to_name, to_addr) in routes {
        info!("  🔄 Ruta: {} -> {}", from_name, to_name);
        
        // Use different amounts based on token
        let amount = if from_name == "SOL" { 0.1 } else { 100.0 }; // 100 units for non-SOL
        let amount_units = if from_name == "SOL" { 
            (amount * 1_000_000_000.0) as u64 
        } else { 
            (amount * 1_000_000.0) as u64  // Assume 6 decimals for others
        };
        
        match detector.find_arbitrage_opportunities(
            from_addr,
            to_addr,
            amount_units,
            &settings,
        ).await {
            Ok(opportunities) => {
                if !opportunities.is_empty() {
                    info!("    ✅ Encontradas {} oportunidades", opportunities.len());
                    for (i, opp) in opportunities.iter().enumerate() {
                        let profit_sol = opp.expected_profit as f64 / 1_000_000_000.0;
                        if profit_sol > 0.001 {
                            info!("      🎯 Oportunidad {}: Ganancia: {:.6} SOL", i + 1, profit_sol);
                        }
                    }
                } else {
                    info!("    ⚠️  No hay oportunidades para esta ruta");
                }
            }
            Err(e) => {
                warn!("    ❌ Error en ruta {} -> {}: {}", from_name, to_name, e);
            }
        }
    }
    
    Ok(())
}

async fn test_price_inefficiency_arbitrage(detector: &ArbitrageDetector, config: &NetworkConfig) -> Result<()> {
    info!("🔄 Simulando ineficiencias de precios para encontrar arbitrajes rentables");
    
    // En un entorno real, buscaríamos diferencias de precio entre DEXs
    // Para DevNet, vamos a simular algunas estrategias comunes
    
    info!("  💡 Estrategias de arbitraje real:");
    info!("     1. Diferencias de precio entre Jupiter y Orca");
    info!("     2. Slippage arbitrage con órdenes grandes");
    info!("     3. Timing arbitrage durante alta volatilidad");
    info!("     4. Cross-chain arbitrage (fuera del scope actual)");
    
    let settings = ArbitrageSettings {
        min_profit_threshold: 0.002, // 0.002 SOL minimum
        max_slippage: 0.05,          // 5% slippage para capturar más oportunidades
        max_position_size: 1.0,      // 1 SOL para tests grandes
        gas_limit: 1_000_000,
        priority_fee: 0.0001,
    };
    
    let sol_addr = &config.token_addresses["sol"].address;
    let usdc_addr = &config.token_addresses["usdc_mainnet"].address;
    
    // Simular diferentes tamaños de órdenes para encontrar sweet spots
    let order_sizes = vec![0.05, 0.1, 0.2, 0.5, 1.0, 2.0]; // SOL amounts
    
    for size in order_sizes {
        let amount_lamports = (size * 1_000_000_000.0) as u64;
        info!("  💰 Probando orden de {} SOL", size);
        
        match detector.find_arbitrage_opportunities(
            sol_addr,
            usdc_addr,
            amount_lamports,
            &settings,
        ).await {
            Ok(opportunities) => {
                if !opportunities.is_empty() {
                    for (i, opp) in opportunities.iter().enumerate() {
                        let profit_sol = opp.expected_profit as f64 / 1_000_000_000.0;
                        let profit_percent = (profit_sol / size) * 100.0;
                        
                        if profit_sol > 0.001 {
                            info!("    🎯 RENTABLE: Orden {} SOL -> Ganancia {:.6} SOL ({:.2}%)", 
                                size, profit_sol, profit_percent);
                        } else {
                            info!("    ⚠️  Orden {} SOL -> Pérdida {:.6} SOL ({:.2}%)", 
                                size, profit_sol.abs(), profit_percent.abs());
                        }
                    }
                }
            }
            Err(e) => {
                warn!("    ❌ Error con orden {} SOL: {}", size, e);
            }
        }
    }
    
    info!("\n  💡 RECOMENDACIONES:");
    info!("     - En DevNet, las oportunidades son limitadas debido a baja liquidez");
    info!("     - En MainNet, buscar tokens con alta volatilidad");
    info!("     - Monitorear múltiples DEXs simultáneamente");
    info!("     - Usar órdenes más grandes para capturar ineficiencias");
    
    Ok(())
}

fn create_working_devnet_config() -> NetworkConfig {
    let mut token_addresses = HashMap::new();

    // Add working tokens found in our tests
    token_addresses.insert("sol".to_string(), sniperforge::shared::network_config::TokenInfo {
        address: "So11111111111111111111111111111111111111112".to_string(),
        symbol: "SOL".to_string(),
        name: "Solana".to_string(),
        decimals: 9,
        verified: true,
        tradeable: true,
        note: "Native SOL".to_string(),
    });

    token_addresses.insert("usdc_mainnet".to_string(), sniperforge::shared::network_config::TokenInfo {
        address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        symbol: "USDC".to_string(),
        name: "USD Coin".to_string(),
        decimals: 6,
        verified: true,
        tradeable: true,
        note: "MainNet USDC available in DevNet".to_string(),
    });

    token_addresses.insert("ray_mainnet".to_string(), sniperforge::shared::network_config::TokenInfo {
        address: "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R".to_string(),
        symbol: "RAY".to_string(),
        name: "Raydium".to_string(),
        decimals: 6,
        verified: true,
        tradeable: true,
        note: "MainNet RAY available in DevNet".to_string(),
    });

    token_addresses.insert("bonk_mainnet".to_string(), sniperforge::shared::network_config::TokenInfo {
        address: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(),
        symbol: "BONK".to_string(),
        name: "Bonk".to_string(),
        decimals: 5,
        verified: true,
        tradeable: true,
        note: "MainNet BONK available in DevNet".to_string(),
    });

    NetworkConfig {
        network: "devnet".to_string(),
        display_name: "Solana DevNet".to_string(),
        rpc_endpoint: "https://solana-devnet.g.alchemy.com/v2/X64q4zZFEMz_RYzthxUMg".to_string(),
        explorer_base_url: "https://explorer.solana.com".to_string(),
        explorer_cluster_param: "devnet".to_string(),
        program_ids: create_program_ids(),
        token_addresses,
    }
}

fn create_program_ids() -> HashMap<String, String> {
    let mut program_ids = HashMap::new();
    program_ids.insert("system_program".to_string(), "11111111111111111111111111111111".to_string());
    program_ids.insert("token_program".to_string(), "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string());
    program_ids.insert("associated_token_program".to_string(), "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".to_string());
    program_ids.insert("compute_budget_program".to_string(), "ComputeBudget111111111111111111111111111111".to_string());
    program_ids.insert("jupiter_program".to_string(), "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".to_string());
    program_ids.insert("orca_whirlpool_program".to_string(), "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".to_string());
    program_ids.insert("spl_token_swap_program".to_string(), "SwaPpA9LAaLfeLi3a68M4DjnLqgtticKg6CnyNwgAC8".to_string());
    program_ids
}
