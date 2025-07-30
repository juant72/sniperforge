//! Advanced Risk Management System for SniperForge

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use tracing::{info, warn, error};

/// Configuración del gestor de riesgos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagementConfig {
    /// Habilitar gestión de riesgos
    pub enabled: bool,
    /// Máximo porcentaje de portfolio por trade
    pub max_position_size_pct: f64,
    /// Máximo número de trades simultáneos
    pub max_concurrent_trades: usize,
    /// Máxima pérdida diaria permitida (USD)
    pub max_daily_loss_usd: f64,
    /// Máxima pérdida por trade individual (USD)
    pub max_trade_loss_usd: f64,
    /// Volatilidad máxima permitida para trading
    pub max_volatility_threshold: f64,
    /// Liquidez mínima requerida (USD)
    pub min_liquidity_usd: f64,
    /// Score de riesgo máximo aceptable [0-1]
    pub max_risk_score: f64,
    /// Intervalo de verificación de riesgo (segundos)
    pub risk_check_interval_secs: u64,
    /// Habilitar circuit breaker automático
    pub auto_circuit_breaker: bool,
}

impl Default for RiskManagementConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_position_size_pct: 10.0,    // Máximo 10% del portfolio por trade
            max_concurrent_trades: 5,        // Máximo 5 trades simultáneos
            max_daily_loss_usd: 1000.0,     // Máximo $1000 pérdida por día
            max_trade_loss_usd: 200.0,      // Máximo $200 pérdida por trade
            max_volatility_threshold: 0.15, // 15% volatilidad máxima
            min_liquidity_usd: 50000.0,     // Mínimo $50k liquidez
            max_risk_score: 0.7,            // Score de riesgo máximo 70%
            risk_check_interval_secs: 30,   // Verificar cada 30 segundos
            auto_circuit_breaker: true,     // Circuit breaker automático
        }
    }
}

/// Resultado de evaluación de riesgo
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub approved: bool,
    pub risk_score: f64,
    pub risk_factors: Vec<RiskFactor>,
    pub recommendation: RiskRecommendation,
    pub max_trade_amount: f64,
    pub timestamp: DateTime<Utc>,
}

/// Factor de riesgo individual
#[derive(Debug, Clone)]
pub struct RiskFactor {
    pub factor_type: String,
    pub severity: RiskSeverity,
    pub description: String,
    pub impact_score: f64,
}

/// Severidad del riesgo
#[derive(Debug, Clone)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Recomendación de gestión de riesgo
#[derive(Debug, Clone)]
pub enum RiskRecommendation {
    Proceed,
    ReduceSize(f64),
    Delay(u64),
    Reject,
    CircuitBreak,
}

/// Estadísticas de riesgo
#[derive(Debug, Clone, Default)]
pub struct RiskStats {
    pub total_assessments: u64,
    pub approved_trades: u64,
    pub rejected_trades: u64,
    pub circuit_breaks_triggered: u64,
    pub daily_loss_usd: f64,
    pub current_open_positions: usize,
    pub last_assessment_time: Option<DateTime<Utc>>,
}

/// Gestor de riesgos avanzado
#[derive(Debug)]
pub struct AdvancedRiskManager {
    config: RiskManagementConfig,
    stats: RiskStats,
    daily_pnl_tracker: HashMap<String, f64>, // Fecha -> P&L
    active_positions: HashMap<String, TradePosition>,
    market_conditions: MarketConditions,
    circuit_breaker_active: bool,
    last_risk_check: Option<DateTime<Utc>>,
}

/// Posición activa de trading
#[derive(Debug, Clone)]
pub struct TradePosition {
    pub id: String,
    pub symbol: String,
    pub amount_usd: f64,
    pub entry_time: DateTime<Utc>,
    pub risk_score: f64,
    pub max_loss_usd: f64,
}

/// Condiciones del mercado
#[derive(Debug, Clone, Default)]
pub struct MarketConditions {
    pub overall_volatility: f64,
    pub liquidity_index: f64,
    pub market_sentiment: f64,
    pub last_update: Option<DateTime<Utc>>,
}

impl AdvancedRiskManager {
    /// Crear nueva instancia del gestor de riesgos
    pub fn new(config: Option<RiskManagementConfig>) -> Self {
        let config = config.unwrap_or_default();
        
        info!("🛡️ Inicializando Advanced Risk Manager");
        info!("🛡️ Max position size: {:.1}% | Max concurrent: {} | Daily loss limit: ${:.0}", 
              config.max_position_size_pct, config.max_concurrent_trades, config.max_daily_loss_usd);
        
        Self {
            config,
            stats: RiskStats::default(),
            daily_pnl_tracker: HashMap::new(),
            active_positions: HashMap::new(),
            market_conditions: MarketConditions::default(),
            circuit_breaker_active: false,
            last_risk_check: None,
        }
    }
    
    /// Evaluar riesgo de una oportunidad de trading
    pub async fn assess_opportunity_risk(&mut self, 
                                       trade_amount_usd: f64,
                                       volatility: f64,
                                       liquidity_usd: f64,
                                       ml_confidence: f64) -> Result<RiskAssessment> {
        if !self.config.enabled {
            return Ok(RiskAssessment {
                approved: true,
                risk_score: 0.0,
                risk_factors: vec![],
                recommendation: RiskRecommendation::Proceed,
                max_trade_amount: trade_amount_usd,
                timestamp: Utc::now(),
            });
        }
        
        self.stats.total_assessments += 1;
        self.last_risk_check = Some(Utc::now());
        
        let mut risk_factors = Vec::new();
        let mut total_risk_score = 0.0;
        
        // 1. Verificar circuit breaker
        if self.circuit_breaker_active {
            risk_factors.push(RiskFactor {
                factor_type: "Circuit Breaker".to_string(),
                severity: RiskSeverity::Critical,
                description: "Sistema en modo circuit breaker".to_string(),
                impact_score: 1.0,
            });
            total_risk_score = 1.0;
        } else {
            // 2. Evaluar tamaño de posición
            let position_risk = self.assess_position_size_risk(trade_amount_usd);
            if position_risk.impact_score > 0.0 {
                risk_factors.push(position_risk);
                total_risk_score += 0.3;
            }
            
            // 3. Evaluar número de trades concurrentes
            let concurrent_risk = self.assess_concurrent_trades_risk();
            if concurrent_risk.impact_score > 0.0 {
                risk_factors.push(concurrent_risk);
                total_risk_score += 0.2;
            }
            
            // 4. Evaluar pérdidas diarias
            let daily_loss_risk = self.assess_daily_loss_risk();
            if daily_loss_risk.impact_score > 0.0 {
                risk_factors.push(daily_loss_risk);
                total_risk_score += 0.2;
            }
            
            // 5. Evaluar volatilidad del mercado
            let volatility_risk = self.assess_volatility_risk(volatility);
            if volatility_risk.impact_score > 0.0 {
                risk_factors.push(volatility_risk);
                total_risk_score += 0.15;
            }
            
            // 6. Evaluar liquidez
            let liquidity_risk = self.assess_liquidity_risk(liquidity_usd);
            if liquidity_risk.impact_score > 0.0 {
                risk_factors.push(liquidity_risk);
                total_risk_score += 0.1;
            }
            
            // 7. Evaluar confianza ML
            let ml_risk = self.assess_ml_confidence_risk(ml_confidence);
            if ml_risk.impact_score > 0.0 {
                risk_factors.push(ml_risk);
                total_risk_score += 0.05;
            }
        }
        
        // Normalizar risk score
        total_risk_score = total_risk_score.min(1.0);
        
        // Determinar recomendación
        let recommendation = self.determine_recommendation(total_risk_score, trade_amount_usd);
        let approved = matches!(recommendation, RiskRecommendation::Proceed | RiskRecommendation::ReduceSize(_));
        
        if approved {
            self.stats.approved_trades += 1;
        } else {
            self.stats.rejected_trades += 1;
        }
        
        let assessment = RiskAssessment {
            approved,
            risk_score: total_risk_score,
            risk_factors,
            recommendation,
            max_trade_amount: trade_amount_usd,
            timestamp: Utc::now(),
        };
        
        info!("🛡️ Risk Assessment: {} | Score: {:.3} | Factors: {}", 
              if approved { "✅ APPROVED" } else { "❌ REJECTED" },
              total_risk_score, assessment.risk_factors.len());
        
        Ok(assessment)
    }
    
    /// Evaluar riesgo del tamaño de posición
    fn assess_position_size_risk(&self, trade_amount_usd: f64) -> RiskFactor {
        // Simular portfolio total (en producción vendría de wallet real)
        let portfolio_value_usd = 100000.0; // $100k portfolio simulado
        let position_pct = (trade_amount_usd / portfolio_value_usd) * 100.0;
        
        if position_pct > self.config.max_position_size_pct {
            RiskFactor {
                factor_type: "Position Size".to_string(),
                severity: RiskSeverity::High,
                description: format!("Posición {:.1}% excede límite {:.1}%", 
                                   position_pct, self.config.max_position_size_pct),
                impact_score: 0.8,
            }
        } else if position_pct > self.config.max_position_size_pct * 0.8 {
            RiskFactor {
                factor_type: "Position Size".to_string(),
                severity: RiskSeverity::Medium,
                description: format!("Posición {:.1}% cerca del límite", position_pct),
                impact_score: 0.4,
            }
        } else {
            RiskFactor {
                factor_type: "Position Size".to_string(),
                severity: RiskSeverity::Low,
                description: "Tamaño de posición aceptable".to_string(),
                impact_score: 0.0,
            }
        }
    }
    
    /// Evaluar riesgo de trades concurrentes
    fn assess_concurrent_trades_risk(&self) -> RiskFactor {
        let concurrent_count = self.active_positions.len();
        
        if concurrent_count >= self.config.max_concurrent_trades {
            RiskFactor {
                factor_type: "Concurrent Trades".to_string(),
                severity: RiskSeverity::High,
                description: format!("Máximo {} trades concurrentes alcanzado", 
                                   self.config.max_concurrent_trades),
                impact_score: 0.7,
            }
        } else if concurrent_count >= (self.config.max_concurrent_trades as f64 * 0.8) as usize {
            RiskFactor {
                factor_type: "Concurrent Trades".to_string(),
                severity: RiskSeverity::Medium,
                description: format!("{} trades activos de {} máximo", 
                                   concurrent_count, self.config.max_concurrent_trades),
                impact_score: 0.3,
            }
        } else {
            RiskFactor {
                factor_type: "Concurrent Trades".to_string(),
                severity: RiskSeverity::Low,
                description: "Número de trades concurrentes aceptable".to_string(),
                impact_score: 0.0,
            }
        }
    }
    
    /// Evaluar riesgo de pérdidas diarias
    fn assess_daily_loss_risk(&self) -> RiskFactor {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let daily_pnl = self.daily_pnl_tracker.get(&today).unwrap_or(&0.0);
        
        if *daily_pnl < -self.config.max_daily_loss_usd {
            RiskFactor {
                factor_type: "Daily Loss".to_string(),
                severity: RiskSeverity::Critical,
                description: format!("Pérdida diaria ${:.0} excede límite ${:.0}", 
                                   daily_pnl.abs(), self.config.max_daily_loss_usd),
                impact_score: 1.0,
            }
        } else if *daily_pnl < -self.config.max_daily_loss_usd * 0.8 {
            RiskFactor {
                factor_type: "Daily Loss".to_string(),
                severity: RiskSeverity::High,
                description: format!("Pérdida diaria ${:.0} cerca del límite", daily_pnl.abs()),
                impact_score: 0.6,
            }
        } else {
            RiskFactor {
                factor_type: "Daily Loss".to_string(),
                severity: RiskSeverity::Low,
                description: "Pérdidas diarias dentro del límite".to_string(),
                impact_score: 0.0,
            }
        }
    }
    
    /// Evaluar riesgo de volatilidad
    fn assess_volatility_risk(&self, volatility: f64) -> RiskFactor {
        if volatility > self.config.max_volatility_threshold {
            RiskFactor {
                factor_type: "Market Volatility".to_string(),
                severity: RiskSeverity::High,
                description: format!("Volatilidad {:.1}% excede límite {:.1}%", 
                                   volatility * 100.0, self.config.max_volatility_threshold * 100.0),
                impact_score: 0.5,
            }
        } else if volatility > self.config.max_volatility_threshold * 0.8 {
            RiskFactor {
                factor_type: "Market Volatility".to_string(),
                severity: RiskSeverity::Medium,
                description: format!("Volatilidad {:.1}% elevada", volatility * 100.0),
                impact_score: 0.2,
            }
        } else {
            RiskFactor {
                factor_type: "Market Volatility".to_string(),
                severity: RiskSeverity::Low,
                description: "Volatilidad dentro de parámetros normales".to_string(),
                impact_score: 0.0,
            }
        }
    }
    
    /// Evaluar riesgo de liquidez
    fn assess_liquidity_risk(&self, liquidity_usd: f64) -> RiskFactor {
        if liquidity_usd < self.config.min_liquidity_usd {
            RiskFactor {
                factor_type: "Market Liquidity".to_string(),
                severity: RiskSeverity::High,
                description: format!("Liquidez ${:.0}k insuficiente (mín ${:.0}k)", 
                                   liquidity_usd / 1000.0, self.config.min_liquidity_usd / 1000.0),
                impact_score: 0.4,
            }
        } else if liquidity_usd < self.config.min_liquidity_usd * 1.5 {
            RiskFactor {
                factor_type: "Market Liquidity".to_string(),
                severity: RiskSeverity::Medium,
                description: format!("Liquidez ${:.0}k limitada", liquidity_usd / 1000.0),
                impact_score: 0.2,
            }
        } else {
            RiskFactor {
                factor_type: "Market Liquidity".to_string(),
                severity: RiskSeverity::Low,
                description: "Liquidez suficiente".to_string(),
                impact_score: 0.0,
            }
        }
    }
    
    /// Evaluar riesgo de confianza ML
    fn assess_ml_confidence_risk(&self, confidence: f64) -> RiskFactor {
        if confidence < 0.6 {
            RiskFactor {
                factor_type: "ML Confidence".to_string(),
                severity: RiskSeverity::Medium,
                description: format!("Confianza ML {:.1}% baja", confidence * 100.0),
                impact_score: 0.3,
            }
        } else if confidence < 0.8 {
            RiskFactor {
                factor_type: "ML Confidence".to_string(),
                severity: RiskSeverity::Low,
                description: format!("Confianza ML {:.1}% moderada", confidence * 100.0),
                impact_score: 0.1,
            }
        } else {
            RiskFactor {
                factor_type: "ML Confidence".to_string(),
                severity: RiskSeverity::Low,
                description: "Confianza ML alta".to_string(),
                impact_score: 0.0,
            }
        }
    }
    
    /// Determinar recomendación basada en risk score
    fn determine_recommendation(&self, risk_score: f64, trade_amount_usd: f64) -> RiskRecommendation {
        if self.circuit_breaker_active {
            RiskRecommendation::CircuitBreak
        } else if risk_score > self.config.max_risk_score {
            RiskRecommendation::Reject
        } else if risk_score > self.config.max_risk_score * 0.8 {
            // Reducir tamaño del trade
            let reduction_factor = 1.0 - (risk_score - self.config.max_risk_score * 0.8) * 2.0;
            let reduced_amount = trade_amount_usd * reduction_factor.max(0.5);
            RiskRecommendation::ReduceSize(reduced_amount)
        } else if risk_score > self.config.max_risk_score * 0.6 {
            // Delay para re-evaluación
            RiskRecommendation::Delay(30) // 30 segundos
        } else {
            RiskRecommendation::Proceed
        }
    }
    
    /// Activar circuit breaker
    pub fn activate_circuit_breaker(&mut self, reason: &str) {
        if !self.circuit_breaker_active {
            self.circuit_breaker_active = true;
            self.stats.circuit_breaks_triggered += 1;
            error!("🚨 CIRCUIT BREAKER ACTIVADO: {}", reason);
            warn!("🚨 Todos los trades serán rechazados hasta reactivación manual");
        }
    }
    
    /// Desactivar circuit breaker
    pub fn deactivate_circuit_breaker(&mut self) {
        if self.circuit_breaker_active {
            self.circuit_breaker_active = false;
            info!("✅ Circuit breaker desactivado - trading resumed");
        }
    }
    
    /// Obtener estadísticas de riesgo
    pub fn get_risk_stats(&self) -> &RiskStats {
        &self.stats
    }
    
    /// Verificar si circuit breaker está activo
    pub fn is_circuit_breaker_active(&self) -> bool {
        self.circuit_breaker_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_risk_assessment_low_risk() {
        let mut risk_manager = AdvancedRiskManager::new(None);
        
        let assessment = risk_manager.assess_opportunity_risk(
            1000.0,  // $1k trade
            0.05,    // 5% volatility (low)
            100000.0, // $100k liquidity (good)
            0.9      // 90% ML confidence (high)
        ).await.unwrap();
        
        assert!(assessment.approved);
        assert!(assessment.risk_score < 0.3);
        assert!(matches!(assessment.recommendation, RiskRecommendation::Proceed));
    }
    
    #[tokio::test]
    async fn test_risk_assessment_high_risk() {
        let mut config = RiskManagementConfig::default();
        config.max_volatility_threshold = 0.1; // 10% max volatility
        
        let mut risk_manager = AdvancedRiskManager::new(Some(config));
        
        let assessment = risk_manager.assess_opportunity_risk(
            50000.0, // $50k trade (large)
            0.25,    // 25% volatility (very high)
            10000.0, // $10k liquidity (low)
            0.4      // 40% ML confidence (low)
        ).await.unwrap();
        
        assert!(!assessment.approved);
        assert!(assessment.risk_score > 0.7);
    }
    
    #[tokio::test] 
    async fn test_circuit_breaker() {
        let mut risk_manager = AdvancedRiskManager::new(None);
        
        // Activar circuit breaker
        risk_manager.activate_circuit_breaker("Test emergency stop");
        assert!(risk_manager.is_circuit_breaker_active());
        
        // Cualquier assessment debería ser rechazado
        let assessment = risk_manager.assess_opportunity_risk(
            100.0, 0.01, 1000000.0, 0.99
        ).await.unwrap();
        
        assert!(!assessment.approved);
        assert!(matches!(assessment.recommendation, RiskRecommendation::CircuitBreak));
        
        // Desactivar circuit breaker
        risk_manager.deactivate_circuit_breaker();
        assert!(!risk_manager.is_circuit_breaker_active());
    }
}
