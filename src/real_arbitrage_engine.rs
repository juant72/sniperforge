// ================================================================================
// REAL ARBITRAGE ENGINE - MOTOR DE ARBITRAJE VERDADERO
// ================================================================================
// Sistema que ejecuta arbitraje REAL entre diferentes DEXs
// ================================================================================

use anyhow::{Result, anyhow};
use tracing::{info, warn, error};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

use crate::optimized_real_price_feeds::{RealPriceFeeds, RealArbitrageOpportunity, DEXPrice};
use crate::jupiter_real_client::JupiterRealClient;
use crate::wallet_manager::WalletManager;
use crate::optimal_trading_config::OptimalTradingConfig;

/// Motor de arbitraje real entre DEXs
pub struct RealArbitrageEngine {
    price_feeds: RealPriceFeeds,
    jupiter_client: JupiterRealClient,
    direct_dex_clients: HashMap<String, Box<dyn DirectDEXClient>>,
    wallet: WalletManager,
    config: RealArbitrageConfig,
}

/// Configuración para arbitraje real
#[derive(Debug, Clone)]
pub struct RealArbitrageConfig {
    pub max_trade_amount_sol: f64,
    pub min_profit_threshold_pct: f64,
    pub max_slippage_pct: f64,
    pub min_confidence_score: f64,
    pub validation_timeout_sec: u64,
}

impl Default for RealArbitrageConfig {
    fn default() -> Self {
        Self {
            max_trade_amount_sol: 0.001,   // 1 mSOL máximo inicialmente
            min_profit_threshold_pct: 0.3, // 0.3% profit mínimo
            max_slippage_pct: 0.5,         // 0.5% slippage máximo
            min_confidence_score: 0.7,     // 70% confianza mínima
            validation_timeout_sec: 30,    // 30 segundos validación
        }
    }
}

/// Resultado de ejecución de arbitraje real
#[derive(Debug, Clone)]
pub struct RealArbitrageResult {
    pub opportunity_id: String,
    pub success: bool,
    pub profit_sol: f64,
    pub fees_paid_sol: f64,
    pub net_profit_sol: f64,
    pub execution_time_ms: u64,
    pub buy_transaction: Option<String>,
    pub sell_transaction: Option<String>,
    pub error_message: Option<String>,
    pub actual_price_difference: f64,
}

/// Estrategia de ejecución para arbitraje
#[derive(Debug, Clone)]
pub enum ArbitrageStrategy {
    /// Comprar en DEX A, vender en DEX B usando Jupiter
    DirectJupiter { buy_dex: String, sell_via_jupiter: bool },
    /// Usar Jupiter para ambas operaciones con routing inteligente
    FullJupiter { prefer_direct_routes: bool },
    /// Flash loan para arbitraje sin capital inicial
    FlashLoan { loan_amount_sol: f64 },
}

impl RealArbitrageEngine {
    /// Crear nuevo motor de arbitraje real
    pub async fn new(
        jupiter_client: JupiterRealClient,
        wallet: WalletManager,
        config: RealArbitrageConfig,
    ) -> Result<Self> {
        let price_feeds = RealPriceFeeds::new(OptimalTradingConfig::default());
        let direct_dex_clients = HashMap::new(); // Se implementa después

        Ok(Self {
            price_feeds,
            jupiter_client,
            direct_dex_clients,
            wallet,
            config,
        })
    }

    /// FUNCIÓN PRINCIPAL: Buscar y ejecutar arbitraje real
    pub async fn find_and_execute_arbitrage(&mut self) -> Result<Vec<RealArbitrageResult>> {
        info!("🎯 Iniciando búsqueda de arbitraje REAL...");
        
        // 1. Buscar oportunidades reales
        let opportunities = self.price_feeds.find_real_arbitrage_opportunities().await?;
        
        if opportunities.is_empty() {
            info!("📊 No se encontraron oportunidades de arbitraje en este momento");
            return Ok(Vec::new());
        }

        info!("🔍 Encontradas {} oportunidades potenciales", opportunities.len());

        // 2. Filtrar por configuración
        let filtered_opportunities = self.filter_opportunities(opportunities).await?;
        
        if filtered_opportunities.is_empty() {
            info!("⚠️ Ninguna oportunidad pasó los filtros de seguridad");
            return Ok(Vec::new());
        }

        info!("✅ {} oportunidades pasaron filtros", filtered_opportunities.len());

        // 3. Ejecutar las mejores oportunidades
        let mut results = Vec::new();
        
        for (index, opportunity) in filtered_opportunities.into_iter().enumerate() {
            if index >= 3 { // Máximo 3 trades por ciclo
                break;
            }

            info!("⚡ Ejecutando oportunidad {}: {} ({})", 
                  index + 1, opportunity.token_symbol, opportunity.id);

            let opportunity_id = opportunity.id.clone(); // Clone para usar después

            match self.execute_single_arbitrage(opportunity).await {
                Ok(result) => {
                    if result.success {
                        info!("✅ ARBITRAJE EXITOSO: +{:.6} SOL neto", result.net_profit_sol);
                    } else {
                        warn!("❌ Arbitraje falló: {}", result.error_message.as_deref().unwrap_or("Unknown"));
                    }
                    results.push(result);
                }
                Err(e) => {
                    error!("💥 Error ejecutando arbitraje: {}", e);
                    results.push(RealArbitrageResult {
                        opportunity_id: opportunity_id,
                        success: false,
                        profit_sol: 0.0,
                        fees_paid_sol: 0.0,
                        net_profit_sol: 0.0,
                        execution_time_ms: 0,
                        buy_transaction: None,
                        sell_transaction: None,
                        error_message: Some(e.to_string()),
                        actual_price_difference: 0.0,
                    });
                }
            }
        }

        // 4. Resumen de resultados
        let successful_trades = results.iter().filter(|r| r.success).count();
        let total_profit: f64 = results.iter().map(|r| r.net_profit_sol).sum();

        info!("📊 RESUMEN ARBITRAJE:");
        info!("   ✅ Trades exitosos: {}/{}", successful_trades, results.len());
        info!("   💰 Profit total: {:.6} SOL", total_profit);

        Ok(results)
    }

    /// Filtrar oportunidades por configuración de seguridad
    async fn filter_opportunities(&self, opportunities: Vec<RealArbitrageOpportunity>) -> Result<Vec<RealArbitrageOpportunity>> {
        let mut filtered = Vec::new();

        for opportunity in opportunities {
            // Filtro 1: Confidence score mínimo
            if opportunity.confidence_score < self.config.min_confidence_score {
                continue;
            }

            // Filtro 2: Profit threshold mínimo
            if opportunity.price_difference_pct < self.config.min_profit_threshold_pct {
                continue;
            }

            // Filtro 3: Liquidez mínima
            if opportunity.min_liquidity_usd < 10000.0 { // $10K mínimo
                continue;
            }

            // Filtro 4: Validación en tiempo real
            if !self.price_feeds.validate_opportunity(&opportunity).await.unwrap_or(false) {
                continue;
            }

            filtered.push(opportunity);
        }

        Ok(filtered)
    }

    /// Ejecutar arbitraje individual
    async fn execute_single_arbitrage(&mut self, opportunity: RealArbitrageOpportunity) -> Result<RealArbitrageResult> {
        let _start_time = std::time::Instant::now();
        
        info!("🔄 Ejecutando arbitraje: {} -> {}", 
              opportunity.dex_a.dex_name, opportunity.dex_b.dex_name);
        info!("   💱 Token: {} ({:.4}% diferencia)", 
              opportunity.token_symbol, opportunity.price_difference_pct);

        // 1. Determinar estrategia óptima
        let strategy = self.determine_strategy(&opportunity).await?;
        
        // 2. Ejecutar según estrategia
        match strategy {
            ArbitrageStrategy::DirectJupiter { buy_dex, sell_via_jupiter } => {
                self.execute_direct_jupiter_arbitrage(&opportunity, &buy_dex, sell_via_jupiter).await
            }
            ArbitrageStrategy::FullJupiter { prefer_direct_routes } => {
                self.execute_full_jupiter_arbitrage(&opportunity, prefer_direct_routes).await
            }
            ArbitrageStrategy::FlashLoan { loan_amount_sol: _ } => {
                // TODO: Implementar flash loans
                Err(anyhow!("Flash loans not implemented yet"))
            }
        }
    }

    /// Determinar estrategia óptima para la oportunidad
    async fn determine_strategy(&self, opportunity: &RealArbitrageOpportunity) -> Result<ArbitrageStrategy> {
        // Por ahora, usar Jupiter para todo (más simple y confiable)
        // En el futuro, implementar DEX directo para ahorrar fees

        if opportunity.price_difference_pct > 2.0 {
            // Diferencia alta: usar Jupiter routing completo
            Ok(ArbitrageStrategy::FullJupiter { prefer_direct_routes: true })
        } else {
            // Diferencia baja: usar estrategia híbrida
            Ok(ArbitrageStrategy::DirectJupiter { 
                buy_dex: opportunity.dex_a.dex_name.clone(),
                sell_via_jupiter: true 
            })
        }
    }

    /// Ejecutar arbitraje usando Jupiter completo
    async fn execute_full_jupiter_arbitrage(
        &mut self, 
        opportunity: &RealArbitrageOpportunity,
        _prefer_direct_routes: bool,
    ) -> Result<RealArbitrageResult> {
        
        let start_time = std::time::Instant::now();
        
        info!("🚀 Ejecutando arbitraje Full Jupiter");
        
        // Determinar montos y direcciones
        let trade_amount_sol = self.config.max_trade_amount_sol.min(opportunity.trade_amount_sol);
        let trade_amount_lamports = (trade_amount_sol * 1_000_000_000.0) as u64;
        
        // Mints
        let sol_mint = "So11111111111111111111111111111111111111112";
        let token_mint = &opportunity.token_mint;
        
        // PASO 1: SOL → TOKEN (comprar donde está más barato)
        info!("   🔄 Paso 1: SOL → {} (comprar barato)", opportunity.token_symbol);
        
        let quote1_result = self.jupiter_client.get_quote(
            sol_mint,
            token_mint,
            trade_amount_lamports
        ).await;

        let quote1 = match quote1_result {
            Ok(q) => q,
            Err(e) => {
                return Ok(RealArbitrageResult {
                    opportunity_id: opportunity.id.clone(),
                    success: false,
                    profit_sol: 0.0,
                    fees_paid_sol: 0.0,
                    net_profit_sol: 0.0,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    buy_transaction: None,
                    sell_transaction: None,
                    error_message: Some(format!("Quote 1 error: {}", e)),
                    actual_price_difference: 0.0,
                });
            }
        };

        let buy_signature_result = self.jupiter_client.execute_swap(
            quote1.clone(),
            self.wallet.keypair()
        ).await;

        let buy_signature = match buy_signature_result {
            Ok(sig) => sig,
            Err(e) => {
                return Ok(RealArbitrageResult {
                    opportunity_id: opportunity.id.clone(),
                    success: false,
                    profit_sol: 0.0,
                    fees_paid_sol: 0.0,
                    net_profit_sol: 0.0,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    buy_transaction: None,
                    sell_transaction: None,
                    error_message: Some(format!("Buy transaction error: {}", e)),
                    actual_price_difference: 0.0,
                });
            }
        };

        info!("   ✅ Compra ejecutada: {}", buy_signature);

        // Obtener tokens recibidos
        let tokens_received: u64 = quote1.out_amount.parse()
            .map_err(|e| anyhow!("Invalid token amount: {}", e))?;

        // PASO 2: TOKEN → SOL (vender donde está más caro)
        info!("   🔄 Paso 2: {} → SOL (vender caro)", opportunity.token_symbol);
        
        // Esperar confirmación de la primera transacción (importante)
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

        let quote2_result = self.jupiter_client.get_quote(
            token_mint,
            sol_mint,
            tokens_received
        ).await;

        let quote2 = match quote2_result {
            Ok(q) => q,
            Err(e) => {
                return Ok(RealArbitrageResult {
                    opportunity_id: opportunity.id.clone(),
                    success: false,
                    profit_sol: 0.0,
                    fees_paid_sol: 0.0,
                    net_profit_sol: 0.0,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    buy_transaction: Some(buy_signature.to_string()),
                    sell_transaction: None,
                    error_message: Some(format!("Quote 2 error: {}", e)),
                    actual_price_difference: 0.0,
                });
            }
        };

        let sell_signature_result = self.jupiter_client.execute_swap(
            quote2.clone(),
            self.wallet.keypair()
        ).await;

        let sell_signature = match sell_signature_result {
            Ok(sig) => sig,
            Err(e) => {
                return Ok(RealArbitrageResult {
                    opportunity_id: opportunity.id.clone(),
                    success: false,
                    profit_sol: 0.0,
                    fees_paid_sol: 0.0,
                    net_profit_sol: 0.0,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    buy_transaction: Some(buy_signature.to_string()),
                    sell_transaction: None,
                    error_message: Some(format!("Sell transaction error: {}", e)),
                    actual_price_difference: 0.0,
                });
            }
        };

        info!("   ✅ Venta ejecutada: {}", sell_signature);

        // CÁLCULO DE RESULTADOS
        let final_sol_amount: u64 = quote2.out_amount.parse()
            .map_err(|e| anyhow!("Invalid final SOL amount: {}", e))?;

        let gross_profit_lamports = final_sol_amount.saturating_sub(trade_amount_lamports);
        let gross_profit_sol = gross_profit_lamports as f64 / 1_000_000_000.0;

        // Estimar fees (simplificado)
        let estimated_fees_sol = trade_amount_sol * 0.01; // 1% fees estimados
        let net_profit_sol = gross_profit_sol - estimated_fees_sol;

        Ok(RealArbitrageResult {
            opportunity_id: opportunity.id.clone(),
            success: net_profit_sol > 0.0,
            profit_sol: gross_profit_sol,
            fees_paid_sol: estimated_fees_sol,
            net_profit_sol,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            buy_transaction: Some(buy_signature.to_string()),
            sell_transaction: Some(sell_signature.to_string()),
            error_message: None,
            actual_price_difference: opportunity.price_difference_pct,
        })
    }

    /// Ejecutar arbitraje directo + Jupiter
    async fn execute_direct_jupiter_arbitrage(
        &mut self,
        opportunity: &RealArbitrageOpportunity,
        _buy_dex: &str,
        _sell_via_jupiter: bool,
    ) -> Result<RealArbitrageResult> {
        // TODO: Implementar DEX directo
        // Por ahora, delegar a Jupiter completo
        self.execute_full_jupiter_arbitrage(opportunity, true).await
    }

    /// Obtener estadísticas del motor
    pub fn get_performance_stats(&self) -> ArbitrageEngineStats {
        ArbitrageEngineStats {
            total_opportunities_found: 0,
            successful_arbitrages: 0,
            total_profit_sol: 0.0,
            average_profit_per_trade: 0.0,
            success_rate_pct: 0.0,
            // TODO: Implementar tracking real
        }
    }
}

/// Estadísticas del motor de arbitraje
#[derive(Debug, Clone)]
pub struct ArbitrageEngineStats {
    pub total_opportunities_found: u64,
    pub successful_arbitrages: u64,
    pub total_profit_sol: f64,
    pub average_profit_per_trade: f64,
    pub success_rate_pct: f64,
}

/// Trait para clientes DEX directos
pub trait DirectDEXClient: Send + Sync {
    fn get_name(&self) -> &str;
    
    // TODO: Definir interfaz para DEX directo
    // async fn get_price(&self, token_mint: &str) -> Result<f64>;
    // async fn execute_swap(&self, from_mint: &str, to_mint: &str, amount: u64) -> Result<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_arbitrage_engine_creation() {
        // Test básico de creación
        // Requiere configuración de wallet y Jupiter client
    }
}
