// ================================================================================
// HYBRID ARBITRAGE SYSTEM - JUPITER + DIRECT
// ================================================================================
// Sistema híbrido que elige automáticamente entre Jupiter y swaps directos
// basado en condiciones óptimas para maximizar profit
// ================================================================================

use anyhow::{Result, anyhow};
use tracing::{info, warn, debug};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

/// Configuración del sistema híbrido
#[derive(Debug, Clone)]
pub struct HybridArbitrageConfig {
    /// Umbral de profit para usar Jupiter (arriba = Jupiter, abajo = Directo)
    pub jupiter_profit_threshold: f64,  // 0.01 SOL
    
    /// Umbral de monto para usar Jupiter (arriba = Jupiter, abajo = Directo)  
    pub jupiter_amount_threshold: f64,  // 0.05 SOL
    
    /// Factor de ventaja mínima para cambiar de método (5% = 1.05)
    pub switch_advantage_factor: f64,   // 1.05
    
    /// Tiempo máximo para quote comparison (ms)
    pub comparison_timeout_ms: u64,     // 2000
}

impl Default for HybridArbitrageConfig {
    fn default() -> Self {
        Self {
            jupiter_profit_threshold: 0.01,    // 0.01 SOL
            jupiter_amount_threshold: 0.05,    // 0.05 SOL  
            switch_advantage_factor: 1.05,     // 5% ventaja mínima
            comparison_timeout_ms: 2000,       // 2 segundos max
        }
    }
}

/// Sistema híbrido de arbitraje
pub struct HybridArbitrageSystem {
    config: HybridArbitrageConfig,
    jupiter_client: crate::jupiter_real_client::JupiterRealClient,
    direct_client: DirectDEXClient,
    stats: HybridStats,
}

/// Estadísticas del sistema híbrido
#[derive(Debug, Clone, Default)]
pub struct HybridStats {
    pub jupiter_trades: u64,
    pub direct_trades: u64,
    pub jupiter_profit: f64,
    pub direct_profit: f64,
    pub fees_saved_vs_full_jupiter: f64,
    pub last_method_used: Option<String>,
}

/// Resultado de comparación de métodos
#[derive(Debug, Clone)]
pub struct MethodComparison {
    pub jupiter_quote: Option<HybridQuote>,
    pub direct_quote: Option<HybridQuote>,
    pub recommended_method: ArbitrageMethod,
    pub reason: String,
    pub expected_savings: f64,
}

/// Quote unificado para comparación
#[derive(Debug, Clone)]
pub struct HybridQuote {
    pub method: ArbitrageMethod,
    pub gross_profit: f64,
    pub total_fees: f64,
    pub net_profit: f64,
    pub execution_time_estimate: u64, // milliseconds
    pub confidence: f64, // 0.0 - 1.0
}

/// Métodos de arbitraje disponibles
#[derive(Debug, Clone, PartialEq)]
pub enum ArbitrageMethod {
    Jupiter { aggregated: bool },
    Direct { dex1: String, dex2: String },
    Hybrid { primary: Box<ArbitrageMethod>, fallback: Box<ArbitrageMethod> },
}

impl HybridArbitrageSystem {
    /// Crear nuevo sistema híbrido
    pub fn new(
        config: HybridArbitrageConfig,
        jupiter_client: crate::jupiter_real_client::JupiterRealClient,
        direct_client: DirectDEXClient,
    ) -> Self {
        Self {
            config,
            jupiter_client,
            direct_client,
            stats: HybridStats::default(),
        }
    }
    
    /// FUNCIÓN PRINCIPAL: Ejecutar arbitraje con método óptimo
    pub async fn execute_optimal_arbitrage(
        &mut self,
        opportunity: &ArbitrageOpportunity,
        wallet: &impl solana_sdk::signer::Signer,
    ) -> Result<HybridExecutionResult> {
        
        info!("🧠 SISTEMA HÍBRIDO: Analizando método óptimo");
        info!("   💰 Profit estimado: {:.6} SOL", opportunity.estimated_profit);
        info!("   💱 Monto: {:.6} SOL", opportunity.trade_amount);
        
        // 1. DECISIÓN RÁPIDA basada en heurísticas
        let quick_decision = self.make_quick_decision(opportunity);
        
        match quick_decision {
            QuickDecision::ObviousJupiter(reason) => {
                info!("⚡ DECISIÓN RÁPIDA: Jupiter ({})", reason);
                self.execute_jupiter_method(opportunity, wallet).await
            },
            QuickDecision::ObviousDirect(reason) => {
                info!("⚡ DECISIÓN RÁPIDA: Directo ({})", reason);
                self.execute_direct_method(opportunity, wallet).await
            },
            QuickDecision::NeedsComparison => {
                info!("🔄 COMPARACIÓN DETALLADA requerida");
                self.execute_with_comparison(opportunity, wallet).await
            },
        }
    }
    
    /// Tomar decisión rápida basada en heurísticas
    fn make_quick_decision(&self, opportunity: &ArbitrageOpportunity) -> QuickDecision {
        let profit = opportunity.estimated_profit;
        let amount = opportunity.trade_amount;
        
        // HEURÍSTICA 1: Trades muy grandes siempre Jupiter (liquidez)
        if amount > 1.0 {
            return QuickDecision::ObviousJupiter("trade muy grande, necesita liquidez agregada".to_string());
        }
        
        // HEURÍSTICA 2: Trades muy pequeños siempre directo (fees)
        if profit < 0.003 && amount < 0.02 {
            return QuickDecision::ObviousDirect("trade pequeño, ahorrar platform fees".to_string());
        }
        
        // HEURÍSTICA 3: Profit muy alto = Jupiter (mejor fill)
        if profit > 0.05 {
            return QuickDecision::ObviousJupiter("profit alto, necesita mejor fill rate".to_string());
        }
        
        // HEURÍSTICA 4: Entre umbrales = comparar
        QuickDecision::NeedsComparison
    }
    
    /// Ejecutar con comparación detallada
    async fn execute_with_comparison(
        &mut self,
        opportunity: &ArbitrageOpportunity,
        wallet: &impl solana_sdk::signer::Signer,
    ) -> Result<HybridExecutionResult> {
        
        info!("🔍 Comparando métodos Jupiter vs Directo...");
        
        // Obtener quotes de ambos métodos en paralelo
        let comparison = self.compare_methods(opportunity).await?;
        
        info!("📊 COMPARACIÓN COMPLETADA:");
        info!("   🚀 Jupiter net: {:.6} SOL", 
              comparison.jupiter_quote.as_ref().map(|q| q.net_profit).unwrap_or(0.0));
        info!("   🔥 Directo net: {:.6} SOL", 
              comparison.direct_quote.as_ref().map(|q| q.net_profit).unwrap_or(0.0));
        info!("   🎯 Recomendado: {:?}", comparison.recommended_method);
        info!("   💡 Razón: {}", comparison.reason);
        
        // Ejecutar método recomendado
        match comparison.recommended_method {
            ArbitrageMethod::Jupiter { .. } => {
                self.execute_jupiter_method(opportunity, wallet).await
            },
            ArbitrageMethod::Direct { .. } => {
                self.execute_direct_method(opportunity, wallet).await
            },
            _ => {
                warn!("Método híbrido complejo no implementado, usando Jupiter");
                self.execute_jupiter_method(opportunity, wallet).await
            }
        }
    }
    
    /// Comparar ambos métodos y recomendar el mejor
    async fn compare_methods(&self, opportunity: &ArbitrageOpportunity) -> Result<MethodComparison> {
        let start_time = std::time::Instant::now();
        
        // Ejecutar ambos quotes en paralelo con timeout
        let (jupiter_result, direct_result) = tokio::join!(
            self.get_jupiter_quote(opportunity),
            self.get_direct_quote(opportunity)
        );
        
        let comparison_time = start_time.elapsed();
        debug!("Comparison tiempo: {:?}", comparison_time);
        
        let jupiter_quote = jupiter_result.ok();
        let direct_quote = direct_result.ok();
        
        // Determinar mejor método
        let (recommended_method, reason, savings) = match (jupiter_quote.as_ref(), direct_quote.as_ref()) {
            (Some(jupiter), Some(direct)) => {
                let jupiter_net = jupiter.net_profit;
                let direct_net = direct.net_profit;
                
                if jupiter_net > direct_net * self.config.switch_advantage_factor {
                    (
                        ArbitrageMethod::Jupiter { aggregated: true },
                        format!("Jupiter {:.1}% mejor después de fees", ((jupiter_net / direct_net) - 1.0) * 100.0),
                        0.0
                    )
                } else {
                    let savings = jupiter.total_fees - direct.total_fees;
                    (
                        ArbitrageMethod::Direct { 
                            dex1: "Raydium".to_string(), 
                            dex2: "Orca".to_string() 
                        },
                        format!("Directo ahorra {:.6} SOL en fees", savings),
                        savings
                    )
                }
            },
            (Some(_), None) => {
                (
                    ArbitrageMethod::Jupiter { aggregated: true },
                    "Solo Jupiter disponible".to_string(),
                    0.0
                )
            },
            (None, Some(_)) => {
                (
                    ArbitrageMethod::Direct { 
                        dex1: "Raydium".to_string(), 
                        dex2: "Orca".to_string() 
                    },
                    "Solo método directo disponible".to_string(),
                    0.0
                )
            },
            (None, None) => {
                return Err(anyhow!("Ambos métodos fallaron en quote"));
            }
        };
        
        Ok(MethodComparison {
            jupiter_quote,
            direct_quote,
            recommended_method,
            reason,
            expected_savings: savings,
        })
    }
    
    /// Obtener quote de Jupiter con análisis de fees
    async fn get_jupiter_quote(&self, opportunity: &ArbitrageOpportunity) -> Result<HybridQuote> {
        debug!("📡 Obteniendo Jupiter quote...");
        
        // Simular quote de Jupiter (en implementación real usar jupiter_client)
        let gross_profit = opportunity.estimated_profit;
        let platform_fee = opportunity.trade_amount * 0.0025; // 0.25%
        let dex_fees = opportunity.trade_amount * 0.003; // 0.30%
        let gas_fees = 0.00003; // ~30k lamports
        let total_fees = platform_fee + dex_fees + gas_fees;
        let net_profit = gross_profit - total_fees;
        
        Ok(HybridQuote {
            method: ArbitrageMethod::Jupiter { aggregated: true },
            gross_profit,
            total_fees,
            net_profit,
            execution_time_estimate: 3000, // 3 segundos típico
            confidence: 0.95, // Alto fill rate
        })
    }
    
    /// Obtener quote directo con análisis de fees
    async fn get_direct_quote(&self, opportunity: &ArbitrageOpportunity) -> Result<HybridQuote> {
        debug!("🔥 Obteniendo quote directo...");
        
        // Simular quote directo (en implementación real usar direct_client)
        let gross_profit = opportunity.estimated_profit * 0.95; // 5% menos por liquidez limitada
        let platform_fee = 0.0; // CERO platform fee
        let dex_fees = opportunity.trade_amount * 0.0025; // 0.25% Raydium
        let gas_fees = 0.00002; // Menos gas, directo
        let total_fees = platform_fee + dex_fees + gas_fees;
        let net_profit = gross_profit - total_fees;
        
        Ok(HybridQuote {
            method: ArbitrageMethod::Direct { 
                dex1: "Raydium".to_string(), 
                dex2: "Orca".to_string() 
            },
            gross_profit,
            total_fees,
            net_profit,
            execution_time_estimate: 1500, // 1.5 segundos más rápido
            confidence: 0.80, // Menor fill rate
        })
    }
    
    /// Ejecutar usando método Jupiter
    async fn execute_jupiter_method(
        &mut self,
        opportunity: &ArbitrageOpportunity,
        wallet: &impl solana_sdk::signer::Signer,
    ) -> Result<HybridExecutionResult> {
        
        info!("🚀 EJECUTANDO CON JUPITER");
        
        // TODO: Usar jupiter_client real
        let result = self.simulate_jupiter_execution(opportunity).await?;
        
        // Actualizar estadísticas
        self.stats.jupiter_trades += 1;
        self.stats.jupiter_profit += result.actual_profit;
        self.stats.last_method_used = Some("Jupiter".to_string());
        
        info!("✅ Jupiter ejecutado: +{:.6} SOL", result.actual_profit);
        
        Ok(result)
    }
    
    /// Ejecutar usando método directo
    async fn execute_direct_method(
        &mut self,
        opportunity: &ArbitrageOpportunity,
        wallet: &impl solana_sdk::signer::Signer,
    ) -> Result<HybridExecutionResult> {
        
        info!("🔥 EJECUTANDO MÉTODO DIRECTO");
        
        // TODO: Usar direct_client real
        let result = self.simulate_direct_execution(opportunity).await?;
        
        // Actualizar estadísticas
        self.stats.direct_trades += 1;
        self.stats.direct_profit += result.actual_profit;
        self.stats.last_method_used = Some("Direct".to_string());
        
        // Calcular fees ahorrados vs Jupiter
        let jupiter_platform_fee = opportunity.trade_amount * 0.0025;
        self.stats.fees_saved_vs_full_jupiter += jupiter_platform_fee;
        
        info!("✅ Directo ejecutado: +{:.6} SOL (ahorró {:.6} SOL en fees)", 
              result.actual_profit, jupiter_platform_fee);
        
        Ok(result)
    }
    
    /// Simulación de ejecución Jupiter
    async fn simulate_jupiter_execution(&self, opportunity: &ArbitrageOpportunity) -> Result<HybridExecutionResult> {
        tokio::time::sleep(tokio::time::Duration::from_millis(2500)).await;
        
        let quote = self.get_jupiter_quote(opportunity).await?;
        
        Ok(HybridExecutionResult {
            success: true,
            method_used: ArbitrageMethod::Jupiter { aggregated: true },
            actual_profit: quote.net_profit,
            fees_paid: quote.total_fees,
            execution_time: tokio::time::Duration::from_millis(2500),
            transaction_signatures: vec!["JUPITER_SIM_123".to_string()],
            fees_saved_vs_alternative: 0.0,
        })
    }
    
    /// Simulación de ejecución directa
    async fn simulate_direct_execution(&self, opportunity: &ArbitrageOpportunity) -> Result<HybridExecutionResult> {
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
        
        let quote = self.get_direct_quote(opportunity).await?;
        let jupiter_quote = self.get_jupiter_quote(opportunity).await?;
        let fees_saved = jupiter_quote.total_fees - quote.total_fees;
        
        Ok(HybridExecutionResult {
            success: true,
            method_used: ArbitrageMethod::Direct { 
                dex1: "Raydium".to_string(), 
                dex2: "Orca".to_string() 
            },
            actual_profit: quote.net_profit,
            fees_paid: quote.total_fees,
            execution_time: tokio::time::Duration::from_millis(1500),
            transaction_signatures: vec!["DIRECT_SIM_456".to_string()],
            fees_saved_vs_alternative: fees_saved,
        })
    }
    
    /// Obtener estadísticas del sistema híbrido
    pub fn get_stats(&self) -> &HybridStats {
        &self.stats
    }
    
    /// Generar reporte de performance
    pub fn generate_performance_report(&self) -> String {
        let total_trades = self.stats.jupiter_trades + self.stats.direct_trades;
        let total_profit = self.stats.jupiter_profit + self.stats.direct_profit;
        
        if total_trades == 0 {
            return "No hay trades para reportar".to_string();
        }
        
        let jupiter_percentage = (self.stats.jupiter_trades as f64 / total_trades as f64) * 100.0;
        let direct_percentage = (self.stats.direct_trades as f64 / total_trades as f64) * 100.0;
        
        format!(
            "🎯 REPORTE SISTEMA HÍBRIDO:\n\
             📊 Total trades: {}\n\
             🚀 Jupiter: {} ({:.1}%) - Profit: {:.6} SOL\n\
             🔥 Directo: {} ({:.1}%) - Profit: {:.6} SOL\n\
             💰 Profit total: {:.6} SOL\n\
             💸 Fees ahorrados: {:.6} SOL\n\
             📈 Último método: {:?}",
            total_trades,
            self.stats.jupiter_trades, jupiter_percentage, self.stats.jupiter_profit,
            self.stats.direct_trades, direct_percentage, self.stats.direct_profit,
            total_profit,
            self.stats.fees_saved_vs_full_jupiter,
            self.stats.last_method_used
        )
    }
}

/// Decisión rápida
#[derive(Debug)]
enum QuickDecision {
    ObviousJupiter(String),
    ObviousDirect(String),
    NeedsComparison,
}

/// Resultado de ejecución híbrida
#[derive(Debug, Clone)]
pub struct HybridExecutionResult {
    pub success: bool,
    pub method_used: ArbitrageMethod,
    pub actual_profit: f64,
    pub fees_paid: f64,
    pub execution_time: tokio::time::Duration,
    pub transaction_signatures: Vec<String>,
    pub fees_saved_vs_alternative: f64,
}

/// Oportunidad de arbitraje (estructura simplificada)
#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub id: String,
    pub estimated_profit: f64,
    pub trade_amount: f64,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub confidence: f64,
}

// Placeholder para DirectDEXClient (implementar según especificación anterior)
pub struct DirectDEXClient {
    // TODO: Implementar cliente directo
}

impl DirectDEXClient {
    pub fn new(_rpc_endpoint: String) -> Self {
        Self {}
    }
}
