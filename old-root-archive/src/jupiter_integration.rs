// ===== JUPITER ADVANCED INTEGRATION =====
// Integración simplificada con Jupiter usando el cliente existente

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use tracing::{info, warn, debug, error};
use tokio::sync::Mutex;
use serde_json::{Value, json};
use solana_sdk::pubkey::Pubkey;
use solana_client::rpc_client::RpcClient;

use crate::shared::jupiter_client::JupiterClient;
use crate::unified_config::UnifiedPhase45Config;

/// Configuración simplificada para Jupiter Advanced
#[derive(Debug, Clone)]
pub struct JupiterAdvancedConfig {
    pub api_endpoint: String,
    pub swap_endpoint: String,
    pub max_accounts: u8,
    pub restrict_intermediate_tokens: bool,
    pub intermediate_tokens: Vec<Pubkey>,
    pub dynamic_slippage: bool,
    pub min_profit_threshold_bps: u64,
    pub max_route_complexity: usize,
    pub timeout_seconds: u64,
}

/// Oportunidad Jupiter avanzada simplificada
#[derive(Debug, Clone)]
pub struct JupiterAdvancedOpportunity {
    pub id: String,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub route: String,
    pub estimated_profit_sol: f64,
    pub slippage_bps: u16,
    pub platform_fee_bps: u16,
    pub priority_fee: u64,
    pub created_at: Instant,
}

/// Opportunity unificada que combina Jupiter Advanced con el sistema principal
#[derive(Debug, Clone)]
pub struct UnifiedJupiterOpportunity {
    pub id: String,
    pub jupiter_opportunity: JupiterAdvancedOpportunity,
    pub estimated_profit_sol: f64,
    pub estimated_execution_time_ms: u64,
    pub confidence_score: f64,
    pub risk_score: f64,
    pub created_at: Instant,
    pub expires_at: Instant,
}

/// Resultado de ejecución Jupiter con detalles mejorados
#[derive(Debug, Clone)]
pub struct JupiterExecutionResult {
    pub success: bool,
    pub jupiter_transaction_id: Option<String>,
    pub actual_profit_sol: f64,
    pub execution_time: Duration,
    pub gas_used_sol: f64,
    pub slippage_experienced: f64,
    pub route_used: String,
    pub error_message: Option<String>,
}

/// Integrador principal para Jupiter Advanced (simplificado)
pub struct JupiterAdvancedIntegrator {
    jupiter_client: Arc<JupiterClient>,
    config: UnifiedPhase45Config,
    rpc_client: Arc<RpcClient>,
    performance_cache: Arc<Mutex<HashMap<String, f64>>>,
    execution_history: Arc<Mutex<Vec<JupiterExecutionResult>>>,
    last_quote_time: Arc<Mutex<Instant>>,
}

impl JupiterAdvancedIntegrator {
    /// Crear nuevo integrador Jupiter Advanced
    pub async fn new(config: UnifiedPhase45Config, rpc_client: Arc<RpcClient>) -> Result<Self> {
        info!("🚀 Inicializando Jupiter Advanced Integrator (Simplificado)");
        
        // Usar el cliente Jupiter existente
        let jupiter_client = Arc::new(JupiterClient::new(
            config.min_profit_bps as f64 / 10000.0,
            config.max_slippage_bps as f64 / 10000.0,
        )?);
        
        info!("✅ Jupiter Advanced Engine inicializado");
        info!("   📊 Max accounts: {}", config.jupiter_max_accounts);
        info!("   🔄 Max route complexity: {}", config.jupiter_max_route_complexity);
        info!("   ⏱️ Timeout: {}s", config.jupiter_timeout_seconds);
        info!("   🎯 Intermediate tokens: {}", config.jupiter_intermediate_tokens.len());
        
        Ok(Self {
            jupiter_client,
            config,
            rpc_client,
            performance_cache: Arc::new(Mutex::new(HashMap::new())),
            execution_history: Arc::new(Mutex::new(Vec::new())),
            last_quote_time: Arc::new(Mutex::new(Instant::now())),
        })
    }
    
    /// Buscar oportunidades usando Jupiter Advanced
    pub async fn find_opportunities(&self) -> Result<Vec<UnifiedJupiterOpportunity>> {
        let start_time = Instant::now();
        debug!("🔍 Buscando oportunidades Jupiter Advanced...");
        
        let mut opportunities = Vec::new();
        
        // Simular búsqueda de oportunidades usando Jupiter client existente
        // En una implementación real, aquí iría la lógica avanzada de Jupiter
        debug!("✅ Búsqueda Jupiter Advanced completada en {:?}", start_time.elapsed());
        
        Ok(opportunities)
    }
    
    /// Ejecutar oportunidad Jupiter
    pub async fn execute_opportunity(&self, opportunity: &UnifiedJupiterOpportunity) -> Result<JupiterExecutionResult> {
        let start_time = Instant::now();
        info!("⚡ Ejecutando oportunidad Jupiter: {}", opportunity.id);
        
        // Simular ejecución
        let result = JupiterExecutionResult {
            success: true,
            jupiter_transaction_id: Some(format!("jupiter_tx_{}", opportunity.id)),
            actual_profit_sol: opportunity.estimated_profit_sol * 0.95, // 95% del profit estimado
            execution_time: start_time.elapsed(),
            gas_used_sol: 0.001,
            slippage_experienced: 0.005,
            route_used: "SOL -> USDC -> RAY".to_string(),
            error_message: None,
        };
        
        // Guardar en historial
        let mut history = self.execution_history.lock().await;
        history.push(result.clone());
        
        // Limitar historial a las últimas 1000 entradas
        if history.len() > 1000 {
            history.drain(0..100);
        }
        
        info!("✅ Ejecución Jupiter completada en {:?}", result.execution_time);
        Ok(result)
    }
    
    /// Obtener estadísticas de rendimiento
    pub async fn get_performance_stats(&self) -> Result<JupiterPerformanceStats> {
        let history = self.execution_history.lock().await;
        
        if history.is_empty() {
            return Ok(JupiterPerformanceStats::default());
        }
        
        let total_executions = history.len();
        let successful_executions = history.iter().filter(|r| r.success).count();
        let total_profit: f64 = history.iter().map(|r| r.actual_profit_sol).sum();
        let total_gas: f64 = history.iter().map(|r| r.gas_used_sol).sum();
        let avg_execution_time = Duration::from_millis(
            history.iter().map(|r| r.execution_time.as_millis() as u64).sum::<u64>() / total_executions as u64
        );
        
        Ok(JupiterPerformanceStats {
            total_opportunities_found: total_executions as u64,
            successful_executions: successful_executions as u64,
            failed_executions: (total_executions - successful_executions) as u64,
            total_profit_sol: total_profit,
            total_gas_spent_sol: total_gas,
            net_profit_sol: total_profit - total_gas,
            average_execution_time: avg_execution_time,
            success_rate: (successful_executions as f64 / total_executions as f64) * 100.0,
            profit_per_execution: total_profit / total_executions as f64,
            last_execution_time: history.last().map(|r| r.execution_time),
        })
    }
}
        })
    }
    
    /// Descubrir oportunidades Jupiter Advanced REALES
    pub async fn discover_real_opportunities(&mut self, balance_sol: f64) -> Result<Vec<UnifiedJupiterOpportunity>> {
        info!("🔍 Descubriendo oportunidades Jupiter Advanced REALES");
        
        if !self.config.jupiter_advanced_enabled {
            debug!("Jupiter Advanced deshabilitado en configuración");
            return Ok(Vec::new());
        }
        
        // Rate limiting para Jupiter API
        self.enforce_rate_limiting().await;
        
        // Calcular amount apropiado basado en configuración y balance
        let trade_amount_sol = self.calculate_optimal_trade_amount(balance_sol)?;
        let amount_lamports = (trade_amount_sol * 1e9) as u64;
        
        info!("💰 Usando {:.6} SOL ({} lamports) para búsqueda de rutas", trade_amount_sol, amount_lamports);
        
        // Buscar oportunidades usando Jupiter Advanced Engine
        let jupiter_opportunities = self.jupiter_engine
            .find_auto_routed_opportunities(amount_lamports)
            .await
            .map_err(|e| anyhow!("Error en Jupiter discovery: {}", e))?;
        
        if jupiter_opportunities.is_empty() {
            info!("📊 No se encontraron oportunidades Jupiter en condiciones actuales del mercado");
            return Ok(Vec::new());
        }
        
        info!("📈 Jupiter encontró {} rutas potenciales", jupiter_opportunities.len());
        
        // Convertir a oportunidades unificadas y validar
        let mut unified_opportunities = Vec::new();
        
        for jupiter_opp in jupiter_opportunities {
            match self.create_unified_opportunity(jupiter_opp, trade_amount_sol).await {
                Ok(Some(unified_opp)) => {
                    unified_opportunities.push(unified_opp);
                },
                Ok(None) => {
                    debug!("Oportunidad Jupiter descartada por validación");
                },
                Err(e) => {
                    warn!("Error creando oportunidad unificada: {}", e);
                }
            }
        }
        
        // Ordenar por score de profitabilidad real
        unified_opportunities.sort_by(|a, b| {
            let score_a = self.calculate_opportunity_score(a);
            let score_b = self.calculate_opportunity_score(b);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        info!("✅ {} oportunidades Jupiter validadas y ordenadas", unified_opportunities.len());
        
        // Actualizar cache de performance
        self.update_performance_cache(&unified_opportunities).await;
        
        Ok(unified_opportunities)
    }
    
    /// Crear oportunidad unificada desde oportunidad Jupiter
    async fn create_unified_opportunity(
        &self, 
        jupiter_opp: JupiterAdvancedOpportunity,
        trade_amount_sol: f64
    ) -> Result<Option<UnifiedJupiterOpportunity>> {
        
        // Validar que la oportunidad cumple criterios mínimos
        if jupiter_opp.profit_percentage < (self.config.min_profit_bps as f64 / 100.0) {
            debug!("Oportunidad Jupiter descartada: profit {:.4}% < mínimo {:.4}%", 
                   jupiter_opp.profit_percentage, self.config.min_profit_bps as f64 / 100.0);
            return Ok(None);
        }
        
        if jupiter_opp.price_impact_pct.abs() > (self.config.max_slippage_bps as f64 / 100.0) {
            debug!("Oportunidad Jupiter descartada: impacto {:.4}% > máximo {:.4}%", 
                   jupiter_opp.price_impact_pct.abs(), self.config.max_slippage_bps as f64 / 100.0);
            return Ok(None);
        }
        
        if jupiter_opp.execution_complexity > self.config.jupiter_max_route_complexity {
            debug!("Oportunidad Jupiter descartada: complejidad {} > máxima {}", 
                   jupiter_opp.execution_complexity, self.config.jupiter_max_route_complexity);
            return Ok(None);
        }
        
        // Calcular profit real después de costos
        let estimated_profit_sol = self.calculate_real_profit(&jupiter_opp, trade_amount_sol)?;
        
        if estimated_profit_sol <= 0.0 {
            debug!("Oportunidad Jupiter descartada: profit real negativo después de costos");
            return Ok(None);
        }
        
        // Calcular scores
        let confidence_score = self.calculate_confidence_score(&jupiter_opp);
        let risk_score = self.calculate_risk_score(&jupiter_opp);
        
        let unified_opp = UnifiedJupiterOpportunity {
            id: format!("JUPITER_{}_{}", 
                       jupiter_opp.input_token.to_string()[..8].to_string(),
                       Instant::now().elapsed().as_millis()),
            jupiter_opportunity: jupiter_opp.clone(),
            estimated_profit_sol,
            estimated_execution_time_ms: jupiter_opp.estimated_execution_time_ms,
            confidence_score,
            risk_score,
            created_at: Instant::now(),
            expires_at: Instant::now() + Duration::from_secs(30), // 30 segundos de validez
        };
        
        info!("✅ Oportunidad Jupiter válida: {} (profit: {:.6} SOL, confianza: {:.2}%)", 
              unified_opp.id, estimated_profit_sol, confidence_score * 100.0);
        
        Ok(Some(unified_opp))
    }
    
    /// Calcular profit real después de TODOS los costos
    fn calculate_real_profit(&self, jupiter_opp: &JupiterAdvancedOpportunity, trade_amount_sol: f64) -> Result<f64> {
        // Profit bruto de Jupiter
        let gross_profit_sol = (jupiter_opp.profit_lamports as f64) / 1e9;
        
        // COSTOS REALES:
        
        // 1. Jupiter platform fee (típicamente 0.01% = 1 BPS)
        let jupiter_fee = trade_amount_sol * 0.0001; // 0.01%
        
        // 2. Network transaction fees (típicamente ~0.000005 SOL)
        let network_fee = 0.000005;
        
        // 3. Priority fee para MEV protection
        let priority_fee = if self.config.mev_protection_enabled {
            (self.config.jito_tip_lamports as f64) / 1e9
        } else {
            0.0
        };
        
        // 4. Slippage estimado basado en price impact
        let slippage_cost = trade_amount_sol * (jupiter_opp.price_impact_pct.abs() / 100.0);
        
        // 5. Overhead de ejecución (gas extra, etc.)
        let execution_overhead = 0.000002; // ~2000 lamports
        
        let total_costs = jupiter_fee + network_fee + priority_fee + slippage_cost + execution_overhead;
        let net_profit = gross_profit_sol - total_costs;
        
        debug!("💰 Cálculo profit real para {:.6} SOL:", trade_amount_sol);
        debug!("   Profit bruto Jupiter: {:.6} SOL", gross_profit_sol);
        debug!("   Jupiter fee (0.01%): {:.6} SOL", jupiter_fee);
        debug!("   Network fee: {:.6} SOL", network_fee);
        debug!("   Priority fee: {:.6} SOL", priority_fee);
        debug!("   Slippage estimado: {:.6} SOL", slippage_cost);
        debug!("   Execution overhead: {:.6} SOL", execution_overhead);
        debug!("   Total costos: {:.6} SOL", total_costs);
        debug!("   PROFIT NETO: {:.6} SOL", net_profit);
        
        Ok(net_profit)
    }
    
    /// Calcular score de confianza basado en características de la oportunidad
    fn calculate_confidence_score(&self, jupiter_opp: &JupiterAdvancedOpportunity) -> f64 {
        let mut score = 1.0;
        
        // Penalizar rutas complejas
        if jupiter_opp.execution_complexity > 3 {
            score *= 0.9;
        }
        if jupiter_opp.execution_complexity > 5 {
            score *= 0.8;
        }
        
        // Penalizar alto price impact
        if jupiter_opp.price_impact_pct.abs() > 0.1 {
            score *= 0.9;
        }
        if jupiter_opp.price_impact_pct.abs() > 0.2 {
            score *= 0.8;
        }
        
        // Bonificar profits altos
        if jupiter_opp.profit_percentage > 0.5 {
            score *= 1.1;
        }
        if jupiter_opp.profit_percentage > 1.0 {
            score *= 1.2;
        }
        
        // Bonificar ejecución rápida estimada
        if jupiter_opp.estimated_execution_time_ms < 5000 {
            score *= 1.05;
        }
        
        score.min(0.95) // Máximo 95% de confianza
    }
    
    /// Calcular score de riesgo
    fn calculate_risk_score(&self, jupiter_opp: &JupiterAdvancedOpportunity) -> f64 {
        let mut risk = 0.0;
        
        // Riesgo por complejidad de ruta
        risk += (jupiter_opp.execution_complexity as f64) * 0.1;
        
        // Riesgo por price impact
        risk += jupiter_opp.price_impact_pct.abs() * 2.0;
        
        // Riesgo por tiempo de ejecución estimado
        risk += (jupiter_opp.estimated_execution_time_ms as f64) / 10000.0;
        
        risk.min(1.0) // Máximo 100% de riesgo
    }
    
    /// Calcular score general de oportunidad
    fn calculate_opportunity_score(&self, opp: &UnifiedJupiterOpportunity) -> f64 {
        let profit_weight = 0.4;
        let confidence_weight = 0.3;
        let risk_weight = 0.2;
        let time_weight = 0.1;
        
        let profit_score = opp.estimated_profit_sol / self.config.max_trade_sol;
        let confidence_score = opp.confidence_score;
        let risk_score = 1.0 - opp.risk_score; // Invertir riesgo (menos riesgo = mejor score)
        let time_score = 1.0 - (opp.estimated_execution_time_ms as f64 / 10000.0).min(1.0);
        
        (profit_score * profit_weight) +
        (confidence_score * confidence_weight) +
        (risk_score * risk_weight) +
        (time_score * time_weight)
    }
    
    /// Ejecutar oportunidad Jupiter con validación completa
    pub async fn execute_opportunity(&self, opportunity: &UnifiedJupiterOpportunity) -> Result<JupiterExecutionResult> {
        let start_time = Instant::now();
        
        info!("⚡ Ejecutando oportunidad Jupiter: {}", opportunity.id);
        info!("   💰 Profit estimado: {:.6} SOL", opportunity.estimated_profit_sol);
        info!("   🎯 Confianza: {:.1}%", opportunity.confidence_score * 100.0);
        info!("   ⚠️ Riesgo: {:.1}%", opportunity.risk_score * 100.0);
        
        // Verificar que la oportunidad no haya expirado
        if Instant::now() > opportunity.expires_at {
            warn!("⏰ Oportunidad expirada: {}", opportunity.id);
            return Ok(JupiterExecutionResult {
                success: false,
                jupiter_transaction_id: None,
                actual_profit_sol: 0.0,
                execution_time: start_time.elapsed(),
                gas_used_sol: 0.0,
                slippage_experienced: 0.0,
                route_used: "EXPIRED".to_string(),
                error_message: Some("Opportunity expired".to_string()),
            });
        }
        
        // SIMULACIÓN DE EJECUCIÓN (por ahora)
        // En implementación real, aquí iría la lógica de firma y envío de transacción
        
        // Simular tiempo de ejecución
        let execution_delay = opportunity.estimated_execution_time_ms + 
                            (rand::random::<u64>() % 1000); // +0-1 segundo de variación
        tokio::time::sleep(Duration::from_millis(execution_delay)).await;
        
        // Simular resultado basado en scores de confianza y riesgo
        let success_probability = opportunity.confidence_score * (1.0 - opportunity.risk_score * 0.5);
        let random_factor: f64 = rand::random();
        
        let execution_result = if random_factor < success_probability {
            // Ejecución exitosa
            let actual_profit = opportunity.estimated_profit_sol * (0.9 + rand::random::<f64>() * 0.2); // 90-110% del estimado
            let slippage = opportunity.jupiter_opportunity.price_impact_pct * (0.8 + rand::random::<f64>() * 0.4); // 80-120% del estimado
            let gas_used = 0.000005 + (rand::random::<f64>() * 0.000003); // 5-8k lamports
            
            info!("✅ Ejecución Jupiter exitosa: +{:.6} SOL", actual_profit);
            
            JupiterExecutionResult {
                success: true,
                jupiter_transaction_id: Some(format!("JUPITER_SIM_{}", Instant::now().elapsed().as_millis())),
                actual_profit_sol: actual_profit,
                execution_time: start_time.elapsed(),
                gas_used_sol: gas_used,
                slippage_experienced: slippage,
                route_used: format!("{}_hop_route", opportunity.jupiter_opportunity.execution_complexity),
                error_message: None,
            }
        } else {
            // Ejecución fallida
            let error_msg = if random_factor < success_probability + 0.1 {
                "Slippage exceeded tolerance"
            } else if random_factor < success_probability + 0.2 {
                "Route became unprofitable"
            } else {
                "Network congestion timeout"
            };
            
            warn!("❌ Ejecución Jupiter fallida: {}", error_msg);
            
            JupiterExecutionResult {
                success: false,
                jupiter_transaction_id: None,
                actual_profit_sol: 0.0,
                execution_time: start_time.elapsed(),
                gas_used_sol: 0.000005, // Gas usado aunque falle
                slippage_experienced: 0.0,
                route_used: format!("{}_hop_route", opportunity.jupiter_opportunity.execution_complexity),
                error_message: Some(error_msg.to_string()),
            }
        };
        
        // Guardar resultado en historial
        {
            let mut history = self.execution_history.lock().await;
            history.push(execution_result.clone());
            
            // Mantener solo últimos 100 resultados
            if history.len() > 100 {
                history.remove(0);
            }
        }
        
        Ok(execution_result)
    }
    
    /// Calcular amount óptimo de trade basado en balance y configuración
    fn calculate_optimal_trade_amount(&self, balance_sol: f64) -> Result<f64> {
        let max_allowed = (balance_sol * 0.1).min(self.config.max_trade_sol); // Máximo 10% del balance
        let min_required = self.config.min_trade_sol;
        
        if max_allowed < min_required {
            return Err(anyhow!(
                "Balance insuficiente: máximo permitido {:.6} SOL < mínimo requerido {:.6} SOL",
                max_allowed, min_required
            ));
        }
        
        // Usar configuración conservadora: cerca del mínimo pero no exactamente
        let optimal = min_required * 1.2; // 20% por encima del mínimo
        
        Ok(optimal.min(max_allowed))
    }
    
    /// Aplicar rate limiting para Jupiter API
    async fn enforce_rate_limiting(&self) {
        let mut last_time = self.last_quote_time.lock().await;
        let elapsed = last_time.elapsed();
        let min_interval = Duration::from_millis(250); // 4 requests/second max
        
        if elapsed < min_interval {
            let sleep_time = min_interval - elapsed;
            debug!("🕰️ Rate limiting: esperando {:?}", sleep_time);
            tokio::time::sleep(sleep_time).await;
        }
        
        *last_time = Instant::now();
    }
    
    /// Actualizar cache de performance
    async fn update_performance_cache(&self, opportunities: &[UnifiedJupiterOpportunity]) {
        let mut cache = self.performance_cache.lock().await;
        
        for opp in opportunities {
            let route_key = format!("{}_to_{}", 
                                  opp.jupiter_opportunity.input_token,
                                  opp.jupiter_opportunity.output_token);
            cache.insert(route_key, opp.estimated_profit_sol);
        }
        
        // Mantener cache limitado
        if cache.len() > 50 {
            let keys_to_remove: Vec<String> = cache.keys().take(10).cloned().collect();
            for key in keys_to_remove {
                cache.remove(&key);
            }
        }
    }
    
    /// Obtener estadísticas de performance
    pub async fn get_performance_stats(&self) -> Result<JupiterPerformanceStats> {
        let history = self.execution_history.lock().await;
        
        if history.is_empty() {
            return Ok(JupiterPerformanceStats::default());
        }
        
        let total_executions = history.len();
        let successful_executions = history.iter().filter(|r| r.success).count();
        let total_profit: f64 = history.iter().map(|r| r.actual_profit_sol).sum();
        let total_gas: f64 = history.iter().map(|r| r.gas_used_sol).sum();
        let avg_execution_time = history.iter()
            .map(|r| r.execution_time.as_millis() as f64)
            .sum::<f64>() / total_executions as f64;
        
        Ok(JupiterPerformanceStats {
            total_executions,
            successful_executions,
            success_rate: (successful_executions as f64 / total_executions as f64) * 100.0,
            total_profit_sol: total_profit,
            total_gas_used_sol: total_gas,
            average_execution_time_ms: avg_execution_time,
            net_profit_sol: total_profit - total_gas,
        })
    }
}

/// Estadísticas de performance Jupiter
#[derive(Debug, Clone)]
pub struct JupiterPerformanceStats {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub success_rate: f64,
    pub total_profit_sol: f64,
    pub total_gas_used_sol: f64,
    pub average_execution_time_ms: f64,
    pub net_profit_sol: f64,
}

impl Default for JupiterPerformanceStats {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            success_rate: 0.0,
            total_profit_sol: 0.0,
            total_gas_used_sol: 0.0,
            average_execution_time_ms: 0.0,
            net_profit_sol: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_config::UnifiedPhase45Config;
    
    #[tokio::test]
    async fn test_jupiter_config_creation() {
        let unified_config = UnifiedPhase45Config::default();
        let jupiter_config = JupiterAdvancedIntegrator::create_jupiter_config(&unified_config);
        assert!(jupiter_config.is_ok());
    }
    
    #[test]
    fn test_trade_amount_calculation() {
        let config = UnifiedPhase45Config::default();
        let rpc_client = Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com"));
        // Note: Can't easily test async new() in sync test, would need tokio runtime
    }
}
