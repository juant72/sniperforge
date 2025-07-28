// ================================================================================
// ARBITRAGE REAL TESTING - SISTEMA DE ARBITRAJE VERDADERO
// ================================================================================
// Prueba del nuevo sistema que detecta y ejecuta arbitraje REAL
// ================================================================================

use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, error, warn};
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

use sniperforge::{
    real_price_feeds::RealPriceFeeds,
    real_arbitrage_engine::{RealArbitrageEngine, RealArbitrageConfig},
    jupiter_real_client::{JupiterRealClient, JupiterRealConfig},
    wallet_manager::WalletManager,
    unified_config::UnifiedPhase45Config,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializar logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🎯 === SISTEMA DE ARBITRAJE REAL ===");
    info!("🔍 Detectando oportunidades REALES entre DEXs");
    info!("💰 Ejecutando arbitraje verdadero con profit real");
    info!("================================================");

    // Verificar modo de ejecución
    let force_real = std::env::var("FORCE_REAL_TRANSACTIONS").unwrap_or("false".to_string()) == "true";
    if force_real {
        info!("🔥 MODO TRADING REAL ACTIVADO - Transacciones reales");
    } else {
        info!("🧪 MODO ANÁLISIS - Solo detección de oportunidades");
    }

    // 1. PRUEBA DE PRICE FEEDS REALES
    info!("\n🔍 === FASE 1: TESTING PRICE FEEDS REALES ===");
    
    let price_feeds = RealPriceFeeds::new();
    
    match price_feeds.find_real_arbitrage_opportunities().await {
        Ok(opportunities) => {
            info!("✅ Encontradas {} oportunidades de arbitraje REAL", opportunities.len());
            
            for (i, opp) in opportunities.iter().enumerate().take(5) {
                info!("🎯 Oportunidad {}: {} ({:.2}% profit, {:.1}% confianza)", 
                      i + 1, 
                      opp.token_symbol,
                      opp.price_difference_pct,
                      opp.confidence_score * 100.0);
                info!("   💱 {} ({:.4}$) → {} ({:.4}$)",
                      opp.dex_a.dex_name, opp.dex_a.price_usd,
                      opp.dex_b.dex_name, opp.dex_b.price_usd);
                info!("   💰 Profit estimado: {:.6} SOL", opp.estimated_profit_sol);
                info!("   💧 Liquidez mínima: ${:.0}K", opp.min_liquidity_usd / 1000.0);
                println!();
            }

            if opportunities.is_empty() {
                warn!("⚠️ No se encontraron oportunidades de arbitraje en este momento");
                warn!("   Esto es normal - el arbitraje real es menos frecuente");
                info!("   💡 El sistema seguirá monitoreando...");
            }
        }
        Err(e) => {
            error!("❌ Error obteniendo oportunidades: {}", e);
            return Err(e);
        }
    }

    // 2. INICIALIZACIÓN DEL MOTOR DE ARBITRAJE REAL (solo si hay transacciones reales)
    if force_real {
        info!("\n🚀 === FASE 2: INICIALIZANDO MOTOR DE ARBITRAJE REAL ===");
        
        // Configuración RPC
        let config = UnifiedPhase45Config::safe_trading();
        let rpc_client = RpcClient::new_with_commitment(
            config.rpc_endpoint.clone(),
            CommitmentConfig::confirmed(),
        );

        // Configuración Jupiter
        let jupiter_config = JupiterRealConfig {
            slippage_bps: 50, // 0.5% slippage ultra-conservador
            compute_unit_price_micro_lamports: Some(1000),
            priority_fee_lamports: Some(5000),
            ..Default::default()
        };

        match JupiterRealClient::new(config.rpc_endpoint.clone(), Some(jupiter_config)) {
            Ok(jupiter_client) => {
                info!("✅ Jupiter client inicializado");

                // Cargar wallet
                match WalletManager::from_file("./keypair.json") {
                    Ok(wallet) => {
                        info!("✅ Wallet cargada");

                        // Verificar balance
                        if let Err(e) = wallet.check_balance(&rpc_client, 0.001).await {
                            error!("❌ Balance insuficiente: {}", e);
                            return Err(e);
                        }

                        // Configuración ultra-conservadora
                        let arbitrage_config = RealArbitrageConfig {
                            max_trade_amount_sol: 0.0005, // 0.5 mSOL máximo
                            min_profit_threshold_pct: 0.2, // 0.2% mínimo
                            max_slippage_pct: 0.3,         // 0.3% slippage máximo
                            min_confidence_score: 0.8,     // 80% confianza mínima
                            validation_timeout_sec: 15,    // 15 segundos
                        };

                        match RealArbitrageEngine::new(jupiter_client, wallet, arbitrage_config).await {
                            Ok(mut arbitrage_engine) => {
                                info!("✅ Motor de arbitraje real inicializado");
                                info!("🎯 Configuración ultra-conservadora:");
                                info!("   💰 Max trade: 0.0005 SOL");
                                info!("   📈 Min profit: 0.2%");
                                info!("   🎯 Min confianza: 80%");

                                // LOOP PRINCIPAL DE ARBITRAJE REAL
                                info!("\n🔄 === INICIANDO LOOP DE ARBITRAJE REAL ===");
                                
                                for cycle in 1..=10 { // Máximo 10 ciclos de prueba
                                    info!("🔄 Ciclo {}/10: Buscando arbitraje...", cycle);
                                    
                                    match arbitrage_engine.find_and_execute_arbitrage().await {
                                        Ok(results) => {
                                            if results.is_empty() {
                                                info!("📊 No hay oportunidades rentables en este ciclo");
                                            } else {
                                                for result in results {
                                                    if result.success {
                                                        info!("🎉 ARBITRAJE EXITOSO!");
                                                        info!("   💰 Profit neto: {:.6} SOL", result.net_profit_sol);
                                                        info!("   📝 Buy TX: {}", result.buy_transaction.as_deref().unwrap_or("N/A"));
                                                        info!("   📝 Sell TX: {}", result.sell_transaction.as_deref().unwrap_or("N/A"));
                                                        info!("   ⏱️ Tiempo: {}ms", result.execution_time_ms);
                                                    } else {
                                                        warn!("❌ Arbitraje falló: {}", result.error_message.as_deref().unwrap_or("Unknown"));
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("💥 Error en ciclo de arbitraje: {}", e);
                                        }
                                    }

                                    // Estadísticas del motor
                                    let stats = arbitrage_engine.get_performance_stats();
                                    info!("📊 Stats: {} oportunidades, {} exitosos, {:.6} SOL total",
                                          stats.total_opportunities_found,
                                          stats.successful_arbitrages,
                                          stats.total_profit_sol);

                                    // Esperar entre ciclos
                                    info!("⏳ Esperando 30 segundos hasta próximo ciclo...");
                                    sleep(Duration::from_secs(30)).await;
                                }

                                info!("✅ Prueba de arbitraje real completada");
                            }
                            Err(e) => {
                                error!("❌ Error inicializando motor: {}", e);
                                return Err(e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ Error cargando wallet: {}", e);
                        error!("   💡 Asegúrate que existe ./keypair.json");
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                error!("❌ Error inicializando Jupiter: {}", e);
                return Err(e);
            }
        }
    } else {
        info!("\n💡 === MODO ANÁLISIS COMPLETADO ===");
        info!("Para ejecutar arbitraje real:");
        info!("   $env:FORCE_REAL_TRANSACTIONS=\"true\"; cargo run --bin arbitrage_real_test");
    }

    Ok(())
}
