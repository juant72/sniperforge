// SniperForge Enterprise v3.0 - Risk Manager
// Wrapper and extension for existing security risk management

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{info, debug};

use super::{OpportunityData, SniperConfig, MarketCondition};

// 🚀 REFACTORING: Usar módulos de seguridad existentes
use crate::security::risk_manager::{
    AdvancedRiskManager, 
    RiskManagementConfig, 
    RiskAssessment as SecurityRiskAssessment,
    RiskSeverity
};

// Re-export common types that other modules use (removed duplicates)
pub use crate::security::risk_manager::RiskSeverity as StopLevel;

// Define MonitoringLevel as alias for RiskSeverity for compatibility
pub type MonitoringLevel = RiskSeverity;

// Create StopType enum for compatibility with position_manager
#[derive(Debug, Clone)]
pub enum StopType {
    Hard,
    Soft,
    Trailing,
    TimeBasedExit,
}

// Create a compatibility RiskAssessment that matches what position_manager expects
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub approved: bool,
    pub risk_score: f64,
    pub max_position_size: f64,
    pub monitoring_level: MonitoringLevel,
    pub required_stops: Vec<RequiredStop>,
    pub warnings: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

// Create the stop structure that position_manager expects
#[derive(Debug, Clone)]
pub struct RequiredStop {
    pub stop_type: StopType,
    pub level_percent: f64,
}

/// Liquidity Sniper specific risk manager - wrapper around security module
#[derive(Debug)]
pub struct RiskManager {
    config: SniperConfig,
    
    // 🚀 REFACTORING: Usar gestor de riesgos existente
    security_risk_manager: AdvancedRiskManager,
    
    // Solo funcionalidades específicas del liquidity sniper que no existen
    liquidity_specific_config: LiquidityRiskConfig,
    monitoring_metrics: MonitoringMetrics,
}

/// Configuración específica de riesgos de liquidez
#[derive(Debug, Clone)]
pub struct LiquidityRiskConfig {
    pub min_liquidity_threshold: f64,
    pub max_slippage_tolerance: f64,
    pub liquidity_depth_analysis: bool,
    pub real_time_liquidity_monitoring: bool,
    pub emergency_exit_triggers: Vec<EmergencyTrigger>,
}

impl Default for LiquidityRiskConfig {
    fn default() -> Self {
        Self {
            min_liquidity_threshold: 1000000.0,
            max_slippage_tolerance: 0.02,
            liquidity_depth_analysis: true,
            real_time_liquidity_monitoring: true,
            emergency_exit_triggers: vec![
                EmergencyTrigger::LiquidityDrop(0.5),
                EmergencyTrigger::SlippageSpike(0.05),
                EmergencyTrigger::VolumeAnomaly(10.0),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub enum EmergencyTrigger {
    LiquidityDrop(f64),     // Porcentaje de caída de liquidez
    SlippageSpike(f64),     // Aumento de slippage
    VolumeAnomaly(f64),     // Multiplicador de volumen anómalo
    PriceManipulation,      // Indicadores de manipulación
}

/// Métricas de monitoreo específicas del liquidity sniper
#[derive(Debug, Default)]
pub struct MonitoringMetrics {
    pub total_assessments: u64,
    pub approved_trades: u64,
    pub rejected_trades: u64,
    pub emergency_exits: u64,
    pub liquidity_warnings: u64,
    pub slippage_events: u64,
    pub last_assessment_time: Option<DateTime<Utc>>,
    pub avg_assessment_time_ms: f64,
}

impl RiskManager {
    /// Crear nuevo gestor de riesgos para liquidity sniper
    pub fn new(config: SniperConfig) -> Result<Self> {
        info!("🚀 Iniciando Risk Manager para Liquidity Sniper");
        
        // Crear configuración para el módulo de seguridad
        let security_config = RiskManagementConfig {
            enabled: true,
            max_position_size_pct: config.max_position_size_percent,
            max_concurrent_trades: config.max_positions as usize,
            max_daily_loss_usd: config.capital_allocation * 0.1, // 10% of capital as max daily loss
            max_trade_loss_usd: config.capital_allocation * config.max_position_size_percent * config.stop_loss_percent / 100.0,
            max_volatility_threshold: 0.15, // Default
            min_liquidity_usd: config.min_liquidity_usd,
            max_risk_score: config.max_risk_score,
            risk_check_interval_secs: 30,
            auto_circuit_breaker: true,
        };
        
        let security_risk_manager = AdvancedRiskManager::new(Some(security_config));
        
        Ok(Self {
            config,
            security_risk_manager,
            liquidity_specific_config: LiquidityRiskConfig::default(),
            monitoring_metrics: MonitoringMetrics::default(),
        })
    }

    /// Evaluar oportunidad usando módulo de seguridad + extensiones específicas
    pub async fn assess_opportunity(&mut self, opportunity: &OpportunityData) -> Result<RiskAssessment> {
        let start_time = std::time::Instant::now();
        self.monitoring_metrics.total_assessments += 1;
        
        debug!("🔍 Evaluando oportunidad de liquidez: {:?}", opportunity.token_address);
        
        // 1. Usar evaluación del módulo de seguridad existente
        let trade_amount = self.config.capital_allocation * self.config.max_position_size_percent / 100.0;
        let security_assessment = self.security_risk_manager.assess_opportunity_risk(
            trade_amount, // Calculate trade amount based on position size
            opportunity.risk_score, // Use risk_score as volatility proxy
            opportunity.liquidity_usd,
            opportunity.confidence_score
        ).await?;
        
        // 2. Evaluaciones específicas de liquidez
        let liquidity_assessment = self.assess_liquidity_specific_risks(opportunity).await?;
        
        // 3. Combinar evaluaciones y convertir a formato compatible
        let combined_assessment = self.convert_to_compatible_assessment(security_assessment, liquidity_assessment, trade_amount)?;
        
        // 4. Actualizar métricas
        self.update_metrics(&combined_assessment, start_time.elapsed().as_millis() as f64);
        
        debug!("✅ Evaluación completada: aprobada={}", combined_assessment.approved);
        Ok(combined_assessment)
    }

    /// Evaluaciones específicas de liquidez que no están en el módulo de seguridad
    async fn assess_liquidity_specific_risks(&self, opportunity: &OpportunityData) -> Result<LiquiditySpecificRisk> {
        let mut assessment = LiquiditySpecificRisk {
            liquidity_score: 1.0,
            slippage_risk: 0.0,
            depth_analysis: HashMap::new(),
            emergency_triggers: Vec::new(),
            warnings: Vec::new(),
        };

        // Análisis de liquidez mínima
        if opportunity.liquidity_usd < self.liquidity_specific_config.min_liquidity_threshold {
            assessment.liquidity_score *= 0.3;
            assessment.warnings.push("Liquidez por debajo del umbral mínimo".to_string());
        }

        // Análisis de slippage esperado
        let expected_slippage = self.calculate_expected_slippage(opportunity);
        if expected_slippage > self.liquidity_specific_config.max_slippage_tolerance {
            assessment.slippage_risk = expected_slippage;
            assessment.warnings.push(format!("Slippage esperado alto: {:.2}%", expected_slippage * 100.0));
        }

        // Verificar triggers de emergencia
        for trigger in &self.liquidity_specific_config.emergency_exit_triggers {
            if self.check_emergency_trigger(trigger, opportunity) {
                assessment.emergency_triggers.push(trigger.clone());
            }
        }

        Ok(assessment)
    }

    /// Combinar evaluación de seguridad con evaluación específica de liquidez
    fn convert_to_compatible_assessment(
        &self, 
        security: SecurityRiskAssessment, 
        liquidity: LiquiditySpecificRisk,
        trade_amount: f64
    ) -> Result<RiskAssessment> {
        // Determine monitoring level based on risk score
        let monitoring_level = if security.risk_score > 0.8 {
            RiskSeverity::Critical
        } else if security.risk_score > 0.6 {
            RiskSeverity::High
        } else if security.risk_score > 0.3 {
            RiskSeverity::Medium
        } else {
            RiskSeverity::Low
        };

        // Create required stops based on risk assessment
        let mut required_stops = Vec::new();
        
        // Add hard stop based on stop loss percentage
        required_stops.push(RequiredStop {
            stop_type: StopType::Hard,
            level_percent: self.config.stop_loss_percent,
        });

        // Add soft stop for high risk trades
        if security.risk_score > 0.7 {
            required_stops.push(RequiredStop {
                stop_type: StopType::Soft,
                level_percent: self.config.stop_loss_percent * 0.5, // 50% of hard stop
            });
        }

        // Convert warnings
        let mut warnings = Vec::new();
        for factor in &security.risk_factors {
            warnings.push(format!("{}: {}", factor.factor_type, factor.description));
        }
        warnings.extend(liquidity.warnings);

        let approved = security.approved && liquidity.emergency_triggers.is_empty();

        Ok(RiskAssessment {
            approved,
            risk_score: security.risk_score * liquidity.liquidity_score,
            max_position_size: trade_amount,
            monitoring_level,
            required_stops,
            warnings,
            timestamp: Utc::now(),
        })
    }

    fn calculate_expected_slippage(&self, opportunity: &OpportunityData) -> f64 {
        // Cálculo simplificado de slippage esperado usando price_impact
        opportunity.price_impact
    }

    fn check_emergency_trigger(&self, trigger: &EmergencyTrigger, opportunity: &OpportunityData) -> bool {
        match trigger {
            EmergencyTrigger::LiquidityDrop(threshold) => {
                // 🚀 USAR THRESHOLD: Comparar liquidez actual con threshold
                let current_liquidity = opportunity.liquidity_usd;
                let baseline_liquidity = 100000.0; // Baseline temporal
                let drop_percentage = ((baseline_liquidity - current_liquidity) / baseline_liquidity) * 100.0;
                
                let is_triggered = drop_percentage > *threshold;
                if is_triggered {
                    debug!("🚨 Liquidity drop trigger activated: {:.1}% drop (threshold: {:.1}%)", 
                           drop_percentage, threshold);
                }
                is_triggered
            },
            EmergencyTrigger::SlippageSpike(threshold) => {
                self.calculate_expected_slippage(opportunity) > *threshold
            },
            EmergencyTrigger::VolumeAnomaly(multiplier) => {
                // 🚀 USAR MULTIPLIER: Comparar volumen actual con baseline
                let current_volume = opportunity.volume_24h_usd;
                let baseline_volume = 50000.0; // Baseline temporal
                let volume_ratio = current_volume / baseline_volume;
                
                let is_anomaly = volume_ratio > *multiplier || volume_ratio < (1.0 / multiplier);
                if is_anomaly {
                    debug!("🚨 Volume anomaly trigger activated: {:.2}x ratio (multiplier: {:.2}x)", 
                           volume_ratio, multiplier);
                }
                is_anomaly
            },
            EmergencyTrigger::PriceManipulation => {
                // Algoritmos de detección de manipulación
                false // Implementación compleja
            }
        }
    }

    fn update_metrics(&mut self, assessment: &RiskAssessment, duration_ms: f64) {
        if assessment.approved {
            self.monitoring_metrics.approved_trades += 1;
        } else {
            self.monitoring_metrics.rejected_trades += 1;
        }

        self.monitoring_metrics.liquidity_warnings += assessment.warnings.len() as u64;
        self.monitoring_metrics.last_assessment_time = Some(Utc::now());
        
        // Actualizar tiempo promedio (EMA simple)
        if self.monitoring_metrics.avg_assessment_time_ms == 0.0 {
            self.monitoring_metrics.avg_assessment_time_ms = duration_ms;
        } else {
            self.monitoring_metrics.avg_assessment_time_ms = 
                0.9 * self.monitoring_metrics.avg_assessment_time_ms + 0.1 * duration_ms;
        }
    }

    /// Obtener métricas de monitoreo
    pub fn get_metrics(&self) -> &MonitoringMetrics {
        &self.monitoring_metrics
    }

    /// Configurar parámetros específicos de liquidez
    pub fn configure_liquidity_params(&mut self, config: LiquidityRiskConfig) {
        info!("🔧 Configurando parámetros específicos de liquidez");
        self.liquidity_specific_config = config;
    }

    /// Evaluar condiciones de mercado para liquidez
    pub async fn assess_market_conditions(&self, condition: &MarketCondition) -> Result<MarketLiquidityAssessment> {
        debug!("📊 Evaluando condiciones de mercado para liquidez");
        
        Ok(MarketLiquidityAssessment {
            overall_liquidity_health: self.calculate_market_liquidity_health(condition),
            recommended_action: self.get_market_recommendation(condition),
            risk_factors: self.identify_market_risk_factors(condition),
            timestamp: Utc::now(),
        })
    }

    fn calculate_market_liquidity_health(&self, condition: &MarketCondition) -> f64 {
        // Algoritmo simplificado de salud de liquidez del mercado
        match condition {
            MarketCondition::Bull => 0.8,
            MarketCondition::Bear => 0.4,
            MarketCondition::Sideways => 0.6,
            MarketCondition::Volatile => 0.3,
            MarketCondition::Unknown => 0.5,
        }
    }

    fn get_market_recommendation(&self, condition: &MarketCondition) -> MarketRecommendation {
        match condition {
            MarketCondition::Bull => MarketRecommendation::Increase,
            MarketCondition::Bear => MarketRecommendation::Decrease,
            MarketCondition::Sideways => MarketRecommendation::Hold,
            MarketCondition::Volatile => MarketRecommendation::Pause,
            MarketCondition::Unknown => MarketRecommendation::Hold,
        }
    }

    fn identify_market_risk_factors(&self, condition: &MarketCondition) -> Vec<String> {
        match condition {
            MarketCondition::Volatile => vec![
                "Alta volatilidad detectada".to_string(),
                "Liquidez fragmentada".to_string(),
                "Riesgo de slippage elevado".to_string(),
            ],
            MarketCondition::Bear => vec![
                "Tendencia bajista".to_string(),
                "Liquidez reducida".to_string(),
            ],
            _ => vec![],
        }
    }
}

/// Resultado específico de evaluación de riesgo de liquidez
#[derive(Debug, Clone)]
pub struct LiquidityRiskAssessment {
    pub approved: bool,
    pub risk_score: f64,
    pub recommended_position_size: f64,
    pub stop_loss_price: f64,
    pub take_profit_price: f64,
    pub warnings: Vec<String>,
    pub liquidity_specific: LiquiditySpecificRisk,
    pub timestamp: DateTime<Utc>,
}

/// Evaluación específica de riesgos de liquidez
#[derive(Debug, Clone)]
pub struct LiquiditySpecificRisk {
    pub liquidity_score: f64,
    pub slippage_risk: f64,
    pub depth_analysis: HashMap<String, f64>,
    pub emergency_triggers: Vec<EmergencyTrigger>,
    pub warnings: Vec<String>,
}

/// Evaluación de condiciones de mercado para liquidez
#[derive(Debug, Clone)]
pub struct MarketLiquidityAssessment {
    pub overall_liquidity_health: f64,
    pub recommended_action: MarketRecommendation,
    pub risk_factors: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum MarketRecommendation {
    Increase,
    Decrease,
    Hold,
    Pause,
}
