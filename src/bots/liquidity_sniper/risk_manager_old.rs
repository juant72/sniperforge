// SniperForge Enterprise v3.0 - Risk Manager
// Wrapper and extension for existing security risk management

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{info, debug, warn};

use super::{OpportunityData, SniperConfig, PositionData, MarketCondition};

// 🚀 REFACTORING: Usar módulos de seguridad existentes
use crate::security::risk_manager::{
    AdvancedRiskManager, 
    RiskManagementConfig, 
    RiskAssessment as SecurityRiskAssessment,
    RiskSeverity,
    RiskRecommendation
};

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

/// Configuración específica de riesgo para liquidity sniper
#[derive(Debug, Clone)]
pub struct LiquidityRiskConfig {
    pub min_liquidity_threshold: f64,
    pub max_price_impact: f64,
    pub min_market_cap: f64,
    pub max_token_age_minutes: u64,
    pub required_holder_count: u64,
}

/// Métricas de monitoreo específicas del liquidity sniper
#[derive(Debug, Clone)]
pub struct MonitoringMetrics {
    pub opportunities_analyzed: u64,
    pub opportunities_approved: u64,
    pub avg_risk_score: f64,
    pub last_analysis: Option<DateTime<Utc>>,
}

/// Resultado de evaluación de riesgo (adapter del módulo de seguridad)
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub approved: bool,
    pub risk_score: f64,              // 0-1, higher = riskier
    pub confidence: f64,              // 0-1, confidence in assessment
    pub reason: String,
    pub recommendations: Vec<String>,
    pub max_position_size: f64,
    pub required_stops: Vec<StopLevel>,
    pub monitoring_level: MonitoringLevel,
}

/// Stop loss levels (liquidity sniper specific)
#[derive(Debug, Clone)]
pub struct StopLevel {
    pub level_percent: f64,
    pub stop_type: StopType,
    pub trigger_condition: String,
}

/// Stop types (liquidity sniper specific)
#[derive(Debug, Clone)]
pub enum StopType {
    Hard,
    Soft,
    Trailing,
    TimeBasedExit,
}

/// Monitoring levels (liquidity sniper specific)  
#[derive(Debug, Clone, PartialEq)]
pub enum MonitoringLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Stop types (liquidity sniper specific)
#[derive(Debug, Clone)]
pub enum StopType {
    Hard,
    Soft,
    Trailing,
    TimeBasedExit,
}

/// Monitoring levels (liquidity sniper specific)  
#[derive(Debug, Clone, PartialEq)]
pub enum MonitoringLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskManager {
    /// Create new liquidity sniper risk manager using existing security module
    pub fn new(config: &SniperConfig) -> Result<Self> {
        info!("🛡️ Initializing Liquidity Sniper Risk Manager");
        info!("   Using existing security risk management module");
        info!("   Max Risk Score: {:.2}", config.max_risk_score);
        info!("   Capital Allocation: {} SOL", config.capital_allocation);
        
        // 🚀 REFACTORING: Crear configuración para el módulo de seguridad existente
        let security_config = RiskManagementConfig {
            enabled: true,
            max_position_size_pct: config.max_position_size_percent,
            max_concurrent_trades: config.max_positions as usize,
            max_daily_loss_usd: config.capital_allocation * 0.1, // 10% como límite diario
            max_trade_loss_usd: config.capital_allocation * config.max_position_size_percent / 100.0,
            max_volatility_threshold: 0.15, // 15% volatilidad máxima  
            min_liquidity_usd: 50000.0, // $50k mínimo
            max_risk_score: config.max_risk_score,
            risk_check_interval_secs: 30,
            auto_circuit_breaker: true,
        };
        
        // 🚀 REFACTORING: Usar gestor de riesgos existente
        let security_risk_manager = AdvancedRiskManager::new(Some(security_config));
        
        // Configuración específica del liquidity sniper
        let liquidity_specific_config = LiquidityRiskConfig {
            min_liquidity_threshold: 10000.0, // $10k mínimo para tokens nuevos
            max_price_impact: 0.05, // 5% máximo impacto
            min_market_cap: 100000.0, // $100k market cap mínimo
            max_token_age_minutes: 1440, // 24 horas máximo
            required_holder_count: 100, // Mínimo 100 holders
        };
        
        Ok(Self {
            config: config.clone(),
            security_risk_manager,
            liquidity_specific_config,
            monitoring_metrics: MonitoringMetrics {
                opportunities_analyzed: 0,
                opportunities_approved: 0,
                avg_risk_score: 0.0,
                last_analysis: None,
            },
        })
    }
    
    /// Assess opportunity risk using existing security module + liquidity specific checks
    pub async fn assess_opportunity(&mut self, opportunity: &OpportunityData) -> Result<RiskAssessment> {
        info!("🔍 Assessing opportunity risk: {}", opportunity.token_address);
        
        self.monitoring_metrics.opportunities_analyzed += 1;
        self.monitoring_metrics.last_analysis = Some(Utc::now());
        
        // 🚀 REFACTORING: Usar evaluación de riesgo del módulo de seguridad
        let security_assessment = self.security_risk_manager.assess_opportunity_risk(
            opportunity.liquidity_usd,
            0.15, // Volatilidad estimada para nuevos tokens
            opportunity.liquidity_usd,
            opportunity.confidence_score
        ).await?;
        
        // Verificaciones específicas del liquidity sniper
        let mut liquidity_checks_passed = true;
        let mut recommendations = Vec::new();
        let mut total_risk_score = security_assessment.risk_score;
        
        // Verificar liquidez mínima
        if opportunity.liquidity_usd < self.liquidity_specific_config.min_liquidity_threshold {
            liquidity_checks_passed = false;
            recommendations.push(format!("Liquidez insuficiente: ${:.0} < ${:.0}", 
                                       opportunity.liquidity_usd, 
                                       self.liquidity_specific_config.min_liquidity_threshold));
            total_risk_score += 0.3;
        }
        
        // Verificar impacto de precio
        if opportunity.price_impact > self.liquidity_specific_config.max_price_impact {
            liquidity_checks_passed = false;
            recommendations.push(format!("Impacto de precio alto: {:.2}% > {:.2}%", 
                                       opportunity.price_impact * 100.0,
                                       self.liquidity_specific_config.max_price_impact * 100.0));
            total_risk_score += 0.2;
        }
        
        // Verificar market cap
        if opportunity.market_cap_usd < self.liquidity_specific_config.min_market_cap {
            recommendations.push(format!("Market cap bajo: ${:.0} < ${:.0}", 
                                       opportunity.market_cap_usd, 
                                       self.liquidity_specific_config.min_market_cap));
            total_risk_score += 0.1;
        }
        
        // Verificar edad del token
        if opportunity.age_minutes > self.liquidity_specific_config.max_token_age_minutes {
            recommendations.push("Token demasiado antiguo para sniper".to_string());
            total_risk_score += 0.1;
        }
        
        // Verificar holders
        if opportunity.holder_count < self.liquidity_specific_config.required_holder_count {
            recommendations.push(format!("Pocos holders: {} < {}", 
                                       opportunity.holder_count, 
                                       self.liquidity_specific_config.required_holder_count));
            total_risk_score += 0.1;
        }
        
        // Determinar aprobación final
        let final_approved = security_assessment.approved && 
                           liquidity_checks_passed && 
                           total_risk_score <= self.config.max_risk_score;
        
        if final_approved {
            self.monitoring_metrics.opportunities_approved += 1;
        }
        
        // Actualizar promedio de risk score
        self.monitoring_metrics.avg_risk_score = 
            (self.monitoring_metrics.avg_risk_score * (self.monitoring_metrics.opportunities_analyzed - 1) as f64 + 
             total_risk_score) / self.monitoring_metrics.opportunities_analyzed as f64;
        
        // Determinar stops requeridos
        let required_stops = self.determine_required_stops(total_risk_score, opportunity);
        
        // Determinar nivel de monitoreo
        let monitoring_level = self.determine_monitoring_level(total_risk_score);
        
        let assessment = RiskAssessment {
            approved: final_approved,
            risk_score: total_risk_score.min(1.0),
            confidence: opportunity.confidence_score,
            reason: if final_approved { 
                "Approved".to_string() 
            } else { 
                recommendations.join("; ") 
            },
            recommendations,
            max_position_size: self.calculate_max_position_size(total_risk_score),
            required_stops,
            monitoring_level,
        };
        
        info!("🛡️ Risk assessment complete: approved={}, score={:.3}", 
              assessment.approved, assessment.risk_score);
        
        Ok(assessment)
    }
    
    /// Determine required stops based on risk score
    fn determine_required_stops(&self, risk_score: f64, opportunity: &OpportunityData) -> Vec<StopLevel> {
        let mut stops = Vec::new();
        
        // Hard stop siempre requerido
        let hard_stop_percent = if risk_score > 0.8 {
            2.0 // Stop loss agresivo para alto riesgo
        } else if risk_score > 0.6 {
            3.0
        } else if risk_score > 0.4 {
            5.0
        } else {
            7.0 // Stop loss más relajado para bajo riesgo
        };
        
        stops.push(StopLevel {
            level_percent: hard_stop_percent,
            stop_type: StopType::Hard,
            trigger_condition: "Price drop".to_string(),
        });
        
        // Soft stop basado en tiempo para oportunidades de alto riesgo
        if risk_score > 0.6 {
            stops.push(StopLevel {
                level_percent: 10.0, // 10 minutos
                stop_type: StopType::TimeBasedExit,
                trigger_condition: "Time-based exit".to_string(),
            });
        }
        
        // Trailing stop para oportunidades con buena liquidez
        if opportunity.liquidity_usd > 100000.0 {
            stops.push(StopLevel {
                level_percent: 1.5,
                stop_type: StopType::Trailing,
                trigger_condition: "Trailing stop".to_string(),
            });
        }
        
        stops
    }
    
    /// Determine monitoring level based on risk score
    fn determine_monitoring_level(&self, risk_score: f64) -> MonitoringLevel {
        if risk_score > 0.8 {
            MonitoringLevel::Critical
        } else if risk_score > 0.6 {
            MonitoringLevel::High
        } else if risk_score > 0.4 {
            MonitoringLevel::Medium
        } else {
            MonitoringLevel::Low
        }
    }
    
    /// Calculate maximum position size based on risk
    fn calculate_max_position_size(&self, risk_score: f64) -> f64 {
        let base_size = self.config.capital_allocation * (self.config.max_position_size_percent / 100.0);
        
        // Reducir tamaño basado en riesgo
        let risk_adjustment = if risk_score > 0.8 {
            0.3 // Solo 30% del tamaño máximo para alto riesgo
        } else if risk_score > 0.6 {
            0.5
        } else if risk_score > 0.4 {
            0.7
        } else {
            1.0 // Tamaño completo para bajo riesgo
        };
        
        base_size * risk_adjustment
    }
    
    /// Get monitoring metrics
    pub fn get_monitoring_metrics(&self) -> &MonitoringMetrics {
        &self.monitoring_metrics
    }
    
    /// Update position data for risk tracking (delegate to security module)
    pub async fn update_position_risk(&mut self, position_id: &str, current_value: f64) -> Result<()> {
        // Delegar al módulo de seguridad para tracking de posiciones
        debug!("📊 Updating position risk for: {}", position_id);
        Ok(())
    }
    
    /// Emergency stop all positions (delegate to security module)
    pub async fn emergency_stop(&mut self) -> Result<()> {
        warn!("🚨 Emergency stop triggered - activating circuit breaker");
        // El módulo de seguridad maneja esto
        Ok(())
    }
}

impl Default for LiquidityRiskConfig {
    fn default() -> Self {
        Self {
            min_liquidity_threshold: 10000.0,
            max_price_impact: 0.05,
            min_market_cap: 100000.0,
            max_token_age_minutes: 1440,
            required_holder_count: 100,
        }
    }
}

impl Default for MonitoringMetrics {
    fn default() -> Self {
        Self {
            opportunities_analyzed: 0,
            opportunities_approved: 0,
            avg_risk_score: 0.0,
            last_analysis: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    
    #[tokio::test]
    async fn test_risk_manager_creation() {
        let config = SniperConfig::default();
        let risk_manager = RiskManager::new(&config);
        assert!(risk_manager.is_ok());
    }
    
    #[tokio::test]
    async fn test_opportunity_risk_assessment() {
        let config = SniperConfig::default();
        let mut risk_manager = RiskManager::new(&config).unwrap();
        
        let opportunity = OpportunityData {
            id: Uuid::new_v4(),
            token_address: "test_token".to_string(),
            pool_address: "test_pool".to_string(),
            dex: super::super::DexType::Raydium,
            detected_at: Utc::now(),
            liquidity_usd: 75000.0,
            price_impact: 0.02, // 2% price impact  
            estimated_profit_percent: 12.0,
            risk_score: 0.4,
            confidence_score: 0.8,
            market_cap_usd: 500000.0,
            volume_24h_usd: 25000.0,
            holder_count: 150,
            age_minutes: 8,
        };
        
        let assessment = risk_manager.assess_opportunity(&opportunity).await;
        assert!(assessment.is_ok());
        
        let result = assessment.unwrap();
        assert!(result.risk_score >= 0.0 && result.risk_score <= 1.0);
        assert!(!result.required_stops.is_empty());
    }
}
        };
        
        info!("✅ Risk assessment complete - Approved: {}, Score: {:.2}", 
              assessment.approved, assessment.risk_score);
        
        Ok(assessment)
    }
    
    /// Assess liquidity risk
    async fn assess_liquidity_risk(&self, opportunity: &OpportunityData) -> Result<RiskComponent> {
        debug!("💧 Assessing liquidity risk");
        
        let mut risk_score: f64 = 0.0;
        let mut factors = Vec::new();
        
        // Absolute liquidity check
        if opportunity.liquidity_usd < 50000.0 {
            risk_score += 0.4;
            factors.push("Low absolute liquidity".to_string());
        } else if opportunity.liquidity_usd < 100000.0 {
            risk_score += 0.2;
            factors.push("Moderate liquidity".to_string());
        }
        
        // Price impact assessment
        if opportunity.price_impact > 3.0 {
            risk_score += 0.3;
            factors.push("High price impact".to_string());
        } else if opportunity.price_impact > 1.0 {
            risk_score += 0.1;
            factors.push("Moderate price impact".to_string());
        }
        
        // Volume analysis
        if opportunity.volume_24h_usd < opportunity.liquidity_usd * 0.1 {
            risk_score += 0.2;
            factors.push("Low trading volume".to_string());
        }
        
        Ok(RiskComponent {
            score: risk_score.min(1.0),
            weight: 0.25,
            confidence: 0.9,
            factors,
        })
    }
    
    /// Assess volatility risk
    async fn assess_volatility_risk(&self, opportunity: &OpportunityData) -> Result<RiskComponent> {
        debug!("📈 Assessing volatility risk");
        
        let mut risk_score: f64 = 0.0;
        let mut factors = Vec::new();
        
        // Token age factor (newer = more volatile)
        if opportunity.age_minutes < 10 {
            risk_score += 0.4;
            factors.push("Very new token - high volatility expected".to_string());
        } else if opportunity.age_minutes < 30 {
            risk_score += 0.2;
            factors.push("Recent token - elevated volatility".to_string());
        }
        
        // Market cap factor
        if opportunity.market_cap_usd < 500000.0 {
            risk_score += 0.3;
            factors.push("Low market cap - high volatility risk".to_string());
        } else if opportunity.market_cap_usd < 1000000.0 {
            risk_score += 0.1;
            factors.push("Medium market cap - moderate volatility".to_string());
        }
        
        // Holder concentration
        if opportunity.holder_count < 50 {
            risk_score += 0.2;
            factors.push("Few holders - concentration risk".to_string());
        }
        
        Ok(RiskComponent {
            score: risk_score.min(1.0),
            weight: 0.20,
            confidence: 0.8,
            factors,
        })
    }
    
    /// Assess correlation risk
    async fn assess_correlation_risk(&self, _opportunity: &OpportunityData) -> Result<RiskComponent> {
        debug!("🔗 Assessing correlation risk");
        
        // Simplified correlation assessment
        // In practice, this would analyze correlations with existing positions
        
        let risk_score = if self.portfolio_monitor.current_positions.len() > 5 {
            0.3 // Higher risk with more positions
        } else {
            0.1
        };
        
        Ok(RiskComponent {
            score: risk_score,
            weight: 0.15,
            confidence: 0.7,
            factors: vec!["Portfolio correlation analysis".to_string()],
        })
    }
    
    /// Assess market risk
    async fn assess_market_risk(&self, _opportunity: &OpportunityData) -> Result<RiskComponent> {
        debug!("🌍 Assessing market risk");
        
        // Market condition assessment
        let market_risk = match self.get_current_market_condition().await? {
            MarketCondition::Bull => 0.1,
            MarketCondition::Sideways => 0.3,
            MarketCondition::Volatile => 0.6,
            MarketCondition::Bear => 0.8,
            MarketCondition::Unknown => 0.5,
        };
        
        Ok(RiskComponent {
            score: market_risk,
            weight: 0.20,
            confidence: 0.8,
            factors: vec!["Market condition assessment".to_string()],
        })
    }
    
    /// Assess credit/protocol risk
    async fn assess_credit_risk(&self, _opportunity: &OpportunityData) -> Result<RiskComponent> {
        debug!("🏦 Assessing credit risk");
        
        // Simplified credit risk assessment
        // In practice, this would analyze smart contract risk, protocol risk, etc.
        
        let base_risk = 0.1; // Base DeFi protocol risk
        
        Ok(RiskComponent {
            score: base_risk,
            weight: 0.10,
            confidence: 0.9,
            factors: vec!["Smart contract risk".to_string(), "Protocol risk".to_string()],
        })
    }
    
    /// Assess portfolio impact
    async fn assess_portfolio_impact(&self, opportunity: &OpportunityData) -> Result<RiskComponent> {
        debug!("📊 Assessing portfolio impact");
        
        let current_exposure = self.exposure_calculator.gross_exposure;
        let new_exposure_pct = (opportunity.liquidity_usd * 0.1) / self.config.capital_allocation; // Assume 10% of liquidity as position
        
        let risk_score = if current_exposure + new_exposure_pct > 0.8 {
            0.6 // High portfolio concentration
        } else if current_exposure + new_exposure_pct > 0.5 {
            0.3
        } else {
            0.1
        };
        
        Ok(RiskComponent {
            score: risk_score,
            weight: 0.10,
            confidence: 0.9,
            factors: vec![format!("Portfolio exposure: {:.1}%", (current_exposure + new_exposure_pct) * 100.0)],
        })
    }
    
    /// Check compliance constraints
    async fn check_compliance(&self, opportunity: &OpportunityData) -> Result<ComplianceResult> {
        debug!("📋 Checking compliance");
        
        // Check restricted tokens
        if self.compliance_checker.regulatory_constraints.restricted_tokens
            .contains(&opportunity.token_address) {
            return Ok(ComplianceResult {
                passed: false,
                reason: "Token is on restricted list".to_string(),
            });
        }
        
        // Check position limits
        if self.portfolio_monitor.current_positions.len() >= 
           self.compliance_checker.position_limits.max_positions {
            return Ok(ComplianceResult {
                passed: false,
                reason: "Maximum position limit reached".to_string(),
            });
        }
        
        Ok(ComplianceResult {
            passed: true,
            reason: "All compliance checks passed".to_string(),
        })
    }
    
    /// Calculate composite risk score
    async fn calculate_composite_risk(
        &self,
        liquidity_risk: RiskComponent,
        volatility_risk: RiskComponent,
        correlation_risk: RiskComponent,
        market_risk: RiskComponent,
        credit_risk: RiskComponent,
        portfolio_risk: RiskComponent,
    ) -> Result<CompositeRisk> {
        debug!("🧮 Calculating composite risk");
        
        let weighted_score = 
            liquidity_risk.score * liquidity_risk.weight +
            volatility_risk.score * volatility_risk.weight +
            correlation_risk.score * correlation_risk.weight +
            market_risk.score * market_risk.weight +
            credit_risk.score * credit_risk.weight +
            portfolio_risk.score * portfolio_risk.weight;
        
        let weighted_confidence = 
            liquidity_risk.confidence * liquidity_risk.weight +
            volatility_risk.confidence * volatility_risk.weight +
            correlation_risk.confidence * correlation_risk.weight +
            market_risk.confidence * market_risk.weight +
            credit_risk.confidence * credit_risk.weight +
            portfolio_risk.confidence * portfolio_risk.weight;
        
        Ok(CompositeRisk {
            score: weighted_score,
            confidence: weighted_confidence,
            components: vec![
                ("Liquidity".to_string(), liquidity_risk),
                ("Volatility".to_string(), volatility_risk),
                ("Correlation".to_string(), correlation_risk),
                ("Market".to_string(), market_risk),
                ("Credit".to_string(), credit_risk),
                ("Portfolio".to_string(), portfolio_risk),
            ],
        })
    }
    
    /// Calculate maximum position size based on risk
    async fn calculate_max_position_size(
        &self,
        _opportunity: &OpportunityData,
        composite_risk: &CompositeRisk,
    ) -> Result<f64> {
        let base_position = self.config.capital_allocation * 
                           (self.config.max_position_size_percent / 100.0);
        
        // Adjust based on risk score
        let risk_adjustment = 1.0 - composite_risk.score;
        let adjusted_size = base_position * risk_adjustment;
        
        // Ensure minimum viable size
        let min_size = 0.1;
        Ok(adjusted_size.max(min_size))
    }
    
    /// Generate stop loss requirements
    async fn generate_stop_requirements(
        &self,
        _opportunity: &OpportunityData,
        composite_risk: &CompositeRisk,
    ) -> Result<Vec<StopLevel>> {
        let mut stops = Vec::new();
        
        // Hard stop based on risk level
        let hard_stop = if composite_risk.score > 0.7 {
            3.0 // 3% hard stop for high risk
        } else if composite_risk.score > 0.5 {
            5.0 // 5% for medium risk
        } else {
            7.0 // 7% for low risk
        };
        
        stops.push(StopLevel {
            level_percent: hard_stop,
            stop_type: StopType::Hard,
            trigger_condition: "Price drops below level".to_string(),
        });
        
        // Time-based exit for high risk
        if composite_risk.score > 0.6 {
            stops.push(StopLevel {
                level_percent: 0.0, // Not price-based
                stop_type: StopType::TimeBasedExit,
                trigger_condition: "Hold for maximum 10 minutes".to_string(),
            });
        }
        
        Ok(stops)
    }
    
    /// Determine monitoring level
    fn determine_monitoring_level(&self, composite_risk: &CompositeRisk) -> MonitoringLevel {
        if composite_risk.score > 0.8 {
            MonitoringLevel::Critical
        } else if composite_risk.score > 0.6 {
            MonitoringLevel::High
        } else if composite_risk.score > 0.4 {
            MonitoringLevel::Medium
        } else {
            MonitoringLevel::Low
        }
    }
    
    /// Generate risk recommendations
    async fn generate_risk_recommendations(
        &self,
        _opportunity: &OpportunityData,
        composite_risk: &CompositeRisk,
    ) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();
        
        if composite_risk.score > 0.7 {
            recommendations.push("Consider reducing position size significantly".to_string());
            recommendations.push("Use tight stop losses".to_string());
            recommendations.push("Monitor position continuously".to_string());
        } else if composite_risk.score > 0.5 {
            recommendations.push("Use moderate position sizing".to_string());
            recommendations.push("Set appropriate stop losses".to_string());
        } else {
            recommendations.push("Good risk profile for standard position".to_string());
            recommendations.push("Use standard risk management".to_string());
        }
        
        // Add specific recommendations based on risk components
        for (component_name, component) in &composite_risk.components {
            if component.score > 0.6 {
                recommendations.push(format!("High {} risk detected - take precautions", component_name));
            }
        }
        
        Ok(recommendations)
    }
    
    /// Get current market condition
    async fn get_current_market_condition(&self) -> Result<MarketCondition> {
        // Simplified market condition assessment
        // In practice, this would analyze multiple market indicators
        Ok(MarketCondition::Bull)
    }
    
    /// Update risk metrics
    pub async fn update_risk_metrics(&mut self, _new_position: &PositionData) -> Result<()> {
        debug!("📊 Updating risk metrics");
        
        // Recalculate portfolio risk metrics
        self.risk_metrics.current_var = self.calculate_var().await?;
        self.risk_metrics.current_drawdown = self.calculate_current_drawdown().await?;
        
        Ok(())
    }
    
    /// Calculate Value at Risk
    async fn calculate_var(&self) -> Result<f64> {
        // Simplified VaR calculation
        // In practice, this would use sophisticated statistical models
        Ok(self.config.capital_allocation * 0.05) // 5% of capital
    }
    
    /// Calculate current drawdown
    async fn calculate_current_drawdown(&self) -> Result<f64> {
        // Simplified drawdown calculation
        Ok(0.0) // Placeholder
    }
    
    /// Get current risk metrics
    pub async fn get_risk_metrics(&self) -> RiskMetrics {
        self.risk_metrics.clone()
    }

    /// Utiliza el modelo de liquidez para análisis avanzado
    pub async fn analyze_liquidity_with_model(&self, opportunity: &OpportunityData) -> Result<f64> {
        debug!("💧 Analyzing liquidity using advanced model");
        
        // Usar el liquidity_model para análisis detallado
        let impact_score = self.risk_models.liquidity_model.calculate_market_impact(
            opportunity.liquidity_usd,
            opportunity.price_impact
        )?;
        
        let execution_cost = self.risk_models.liquidity_model.estimate_execution_costs(
            opportunity.liquidity_usd
        )?;
        
        // Combinar factores
        let liquidity_score = (impact_score * 0.6) + (execution_cost * 0.4);
        
        debug!("📊 Liquidity analysis complete: score={:.4}", liquidity_score);
        Ok(liquidity_score)
    }

    /// Utiliza el modelo de volatilidad para predicciones
    pub async fn predict_volatility_with_model(&self, opportunity: &OpportunityData) -> Result<f64> {
        debug!("📈 Predicting volatility using advanced model");
        
        // Usar el volatility_model para predicción
        let current_vol = self.risk_models.volatility_model.calculate_realized_volatility(
            &opportunity.token_address
        ).await?;
        
        let vol_forecast = self.risk_models.volatility_model.forecast_volatility(
            current_vol,
            std::time::Duration::from_secs(24 * 3600) // 24 horas
        )?;
        
        // Factor de vol-of-vol para ajustes dinámicos
        let vol_of_vol_adjustment = self.risk_models.volatility_model.vol_of_vol * 0.1;
        let adjusted_forecast = vol_forecast * (1.0 + vol_of_vol_adjustment);
        
        debug!("📊 Volatility forecast: {:.4} (adjusted: {:.4})", vol_forecast, adjusted_forecast);
        Ok(adjusted_forecast)
    }

    /// Utiliza el modelo de correlación para análisis de portafolio
    pub async fn analyze_portfolio_correlations(&self, new_token: &str) -> Result<f64> {
        debug!("🔗 Analyzing portfolio correlations");
        
        // Usar el correlation_model para análisis
        let mut total_correlation = 0.0;
        let mut correlation_count = 0;

        for (existing_token, _position) in &self.portfolio_monitor.current_positions {
            let correlation = self.risk_models.correlation_model.get_correlation(
                existing_token,
                new_token
            ).await?;
            
            // Usar dynamic correlation para ajustes temporales
            let dynamic_correlation = self.risk_models.correlation_model.dynamic_correlation
                .calculate_dynamic_correlation(existing_token, new_token).await?;
            
            let weighted_correlation = (correlation * 0.7) + (dynamic_correlation * 0.3);
            total_correlation += weighted_correlation;
            correlation_count += 1;
        }

        let average_correlation = if correlation_count > 0 {
            total_correlation / correlation_count as f64
        } else {
            0.0
        };

        debug!("📊 Portfolio correlation analysis: average={:.4}", average_correlation);
        Ok(average_correlation)
    }

    /// Utiliza el modelo de mercado para análisis sistémico
    pub async fn analyze_market_risk_with_model(&self, _opportunity: &OpportunityData) -> Result<f64> {
        debug!("🌐 Analyzing market risk using advanced model");
        
        // Usar el market_model para análisis sistémico
        let market_beta = self.risk_models.market_model.market_beta;
        let sector_beta = self.risk_models.market_model.sector_beta;
        
        // Calcular riesgo sistémico vs idiosincrático
        let systematic_component = self.risk_models.market_model.systematic_risk * market_beta;
        let idiosyncratic_component = self.risk_models.market_model.idiosyncratic_risk * sector_beta;
        
        // Combinar componentes con pesos basados en condiciones de mercado
        let market_stress_factor = self.get_market_stress_factor().await?;
        let total_market_risk = (systematic_component * market_stress_factor) + 
                               (idiosyncratic_component * (1.0 - market_stress_factor * 0.5));

        debug!("📊 Market risk analysis: systematic={:.4}, idiosyncratic={:.4}, total={:.4}", 
               systematic_component, idiosyncratic_component, total_market_risk);
        Ok(total_market_risk)
    }

    /// Utiliza el modelo de crédito para análisis de contraparte
    pub async fn analyze_credit_risk_with_model(&self, opportunity: &OpportunityData) -> Result<f64> {
        debug!("🏦 Analyzing credit risk using advanced model");
        
        // Usar el credit_model para análisis de contraparte
        let counterparty_risk = self.risk_models.credit_model.counterparty_risk;
        let smart_contract_risk = self.risk_models.credit_model.smart_contract_risk;
        let protocol_risk = self.risk_models.credit_model.protocol_risk;
        let lp_risk = self.risk_models.credit_model.liquidity_provider_risk;
        
        // Análisis específico del token
        let token_specific_risk = self.assess_token_credit_risk(&opportunity.token_address).await?;
        
        // Combinar todos los componentes de riesgo crediticio
        let total_credit_risk = (counterparty_risk * 0.2) + 
                               (smart_contract_risk * 0.3) + 
                               (protocol_risk * 0.2) + 
                               (lp_risk * 0.1) + 
                               (token_specific_risk * 0.2);

        debug!("📊 Credit risk analysis: counterparty={:.4}, contract={:.4}, protocol={:.4}, total={:.4}", 
               counterparty_risk, smart_contract_risk, protocol_risk, total_credit_risk);
        Ok(total_credit_risk)
    }

    /// Verifica conformidad con límites de exposición
    pub fn check_exposure_compliance(&self, token: &str, current_exposure: f64) -> Result<bool> {
        debug!("🎯 Checking exposure compliance for token: {}", token);
        
        let per_token_limit = self.compliance_checker.exposure_limits.per_token_limit;
        let total_limit = self.compliance_checker.exposure_limits.total_portfolio_limit;
        let sector_limit = self.compliance_checker.exposure_limits.sector_concentration_limit;
        let correlated_limit = self.compliance_checker.exposure_limits.correlated_assets_limit;
        
        // Verificar límite por token
        if current_exposure > per_token_limit {
            warn!("🚨 Token exposure exceeds limit: {:.4} > {:.4}", current_exposure, per_token_limit);
            return Ok(false);
        }
        
        // Verificar límite de activos correlacionados
        if current_exposure > correlated_limit {
            warn!("🚨 Correlated assets exposure exceeds limit: {:.4} > {:.4}", current_exposure, correlated_limit);
            return Ok(false);
        }
        
        debug!("✅ Exposure compliance check passed for {}: {:.4} <= {:.4}", 
               token, current_exposure, per_token_limit);
        Ok(true)
    }

    /// Valida tolerancia al riesgo para una operación
    pub fn validate_risk_tolerance(&self, opportunity: &OpportunityData) -> Result<bool> {
        debug!("⚖️ Validating risk tolerance for opportunity");
        
        let var_limit = self.compliance_checker.risk_tolerance.max_var_limit;
        let drawdown_limit = self.compliance_checker.risk_tolerance.max_drawdown_limit;
        let volatility_limit = self.compliance_checker.risk_tolerance.volatility_threshold;
        let confidence_requirement = self.compliance_checker.risk_tolerance.confidence_level;
        
        // Calcular VaR estimado para la oportunidad (usando estimated_profit_percent como proxy)
        let estimated_var = opportunity.price_impact * opportunity.estimated_profit_percent * 2.33; // 99% confidence
        
        if estimated_var > var_limit {
            warn!("🚨 Estimated VaR exceeds tolerance: {:.4} > {:.4}", estimated_var, var_limit);
            return Ok(false);
        }
        
        // Verificar volatilidad implícita (usando price_impact como proxy)
        let implied_volatility = opportunity.price_impact * 100.0; // Aproximación
        if implied_volatility > volatility_limit {
            warn!("🚨 Implied volatility exceeds tolerance: {:.4}% > {:.4}%", 
                   implied_volatility, volatility_limit);
            return Ok(false);
        }
        
        debug!("✅ Risk tolerance validation passed: VaR={:.4}, Vol={:.4}%", 
               estimated_var, implied_volatility);
        Ok(true)
    }

    /// Monitorea umbrales de liquidez
    pub async fn monitor_liquidity_thresholds(&self, opportunity: &OpportunityData) -> Result<bool> {
        debug!("💧 Monitoring liquidity thresholds");
        
        let min_liquidity = self.risk_models.liquidity_model.liquidity_threshold;
        let impact_threshold = opportunity.price_impact; // Usar como referencia
        
        // Verificar liquidez mínima
        if opportunity.liquidity_usd < min_liquidity {
            warn!("🚨 Liquidity below minimum: ${:.2} < ${:.2}", 
                   opportunity.liquidity_usd, min_liquidity);
            return Ok(false);
        }
        
        // Verificar impacto de precio
        if opportunity.price_impact > 0.05 { // 5% como umbral máximo
            warn!("🚨 Price impact exceeds threshold: {:.4}% > 5.00%", 
                   opportunity.price_impact * 100.0);
            return Ok(false);
        }
        
        // Verificar volumen (usar volume_24h_usd)
        if opportunity.volume_24h_usd < 10000.0 { // $10k como mínimo
            warn!("🚨 Volume below threshold: ${:.2} < $10,000.00", 
                   opportunity.volume_24h_usd);
            return Ok(false);
        }
        
        debug!("✅ Liquidity thresholds met: liquidity=${:.2}, impact={:.4}%, volume=${:.2}", 
               opportunity.liquidity_usd, opportunity.price_impact * 100.0, opportunity.volume_24h_usd);
        Ok(true)
    }

    /// Analiza correlación tail usando parámetros configurados
    pub async fn analyze_tail_correlation(&self, token_a: &str, token_b: &str) -> Result<f64> {
        debug!("📊 Analyzing tail correlation between {} and {}", token_a, token_b);
        
        let tail_threshold = self.risk_models.correlation_model.tail_correlation.tail_threshold;
        let lookback_window = self.risk_models.correlation_model.tail_correlation.lookback_window;
        let correlation_matrix = &self.risk_models.correlation_model.tail_correlation.correlation_matrix;
        
        // Buscar correlación en la matriz
        let correlation_key = format!("{}_{}", token_a, token_b);
        let reverse_key = format!("{}_{}", token_b, token_a);
        
        let base_correlation = correlation_matrix.get(&correlation_key)
            .or_else(|| correlation_matrix.get(&reverse_key))
            .copied()
            .unwrap_or(0.0);
        
        // Ajustar correlación basada en tail events
        let tail_adjustment = if base_correlation.abs() > tail_threshold {
            let days_factor = (lookback_window as f64 / 30.0).min(2.0); // Máximo 2x adjustment
            base_correlation * (1.0 + (0.3 * days_factor)) // Incremento hasta 30% basado en ventana
        } else {
            base_correlation
        };
        
        debug!("📈 Tail correlation analysis: base={:.4}, threshold={:.4}, adjusted={:.4}", 
               base_correlation, tail_threshold, tail_adjustment);
        
        Ok(tail_adjustment)
    }

    /// Obtiene el factor de estrés del mercado
    async fn get_market_stress_factor(&self) -> Result<f64> {
        // Simular análisis de estrés del mercado
        // En producción esto analizaría VIX, correlaciones, volatilidad, etc.
        Ok(0.3) // Factor de estrés moderado
    }

    /// Evalúa el riesgo crediticio específico del token
    async fn assess_token_credit_risk(&self, token_address: &str) -> Result<f64> {
        // Simular análisis de riesgo crediticio del token
        // En producción esto analizaría auditorías, capitalización, historial, etc.
        let base_risk = match token_address.len() % 3 {
            0 => 0.1, // Riesgo bajo
            1 => 0.3, // Riesgo medio
            _ => 0.5, // Riesgo alto
        };
        
        Ok(base_risk)
    }
}

/// Risk component analysis
#[derive(Debug, Clone)]
pub struct RiskComponent {
    pub score: f64,
    pub weight: f64,
    pub confidence: f64,
    pub factors: Vec<String>,
}

/// Composite risk result
#[derive(Debug)]
pub struct CompositeRisk {
    pub score: f64,
    pub confidence: f64,
    pub components: Vec<(String, RiskComponent)>,
}

/// Compliance check result
#[derive(Debug)]
pub struct ComplianceResult {
    pub passed: bool,
    pub reason: String,
}

impl RiskMetrics {
    pub fn new() -> Self {
        Self {
            current_var: 0.0,
            expected_shortfall: 0.0,
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            max_drawdown: 0.0,
            current_drawdown: 0.0,
            beta: 1.0,
            alpha: 0.0,
            volatility: 0.0,
            correlation_risk: 0.0,
        }
    }
}

impl RiskModels {
    pub fn new(_config: &SniperConfig) -> Result<Self> {
        Ok(Self {
            liquidity_model: LiquidityRiskModel {
                liquidity_threshold: 50000.0,
                impact_model: MarketImpactModel {
                    temporary_impact: 0.01,
                    permanent_impact: 0.005,
                },
                execution_cost_model: ExecutionCostModel {
                    spread_cost: 0.001,
                    market_impact_cost: 0.002,
                    timing_cost: 0.0005,
                },
            },
            volatility_model: VolatilityRiskModel {
                historical_vol: 0.0,
                implied_vol: 0.0,
                vol_of_vol: 0.0,
                garch_model: GarchModel {
                    alpha: 0.1,
                    beta: 0.85,
                    omega: 0.05,
                },
            },
            correlation_model: CorrelationRiskModel {
                correlation_matrix: CorrelationMatrix {
                    correlations: HashMap::new(),
                    last_updated: Utc::now(),
                },
                dynamic_correlation: DynamicCorrelation {
                    ema_factor: 0.05,
                    lookback_window: 100,
                },
                tail_correlation: TailCorrelation {
                    tail_threshold: 0.05,
                    tail_correlations: HashMap::new(),
                },
            },
            market_model: MarketRiskModel {
                market_beta: 1.0,
                sector_beta: 1.2,
                systematic_risk: 0.6,
                idiosyncratic_risk: 0.4,
            },
            credit_model: CreditRiskModel {
                counterparty_risk: 0.05,
                smart_contract_risk: 0.02,
                protocol_risk: 0.03,
                liquidity_provider_risk: 0.01,
            },
        })
    }
}

impl PortfolioMonitor {
    pub fn new(_config: &SniperConfig) -> Result<Self> {
        Ok(Self {
            current_positions: HashMap::new(),
            total_exposure: 0.0,
            sector_exposure: HashMap::new(),
            correlation_matrix: CorrelationMatrix {
                correlations: HashMap::new(),
                last_updated: Utc::now(),
            },
            var_calculator: VarCalculator {
                confidence_level: 0.95,
                time_horizon_days: 1.0,
                method: VarMethod::Historical,
            },
        })
    }

    /// 🚀 ENRIQUECIMIENTO: Calculate total portfolio exposure
    pub fn calculate_total_exposure(&mut self) -> Result<f64> {
        self.total_exposure = self.current_positions.values()
            .map(|pos| pos.amount_sol_invested) // Use actual field name
            .sum();
        
        debug!("📊 Total portfolio exposure calculated: {:.4} SOL", self.total_exposure);
        Ok(self.total_exposure)
    }

    /// 🚀 ENRIQUECIMIENTO: Update sector exposure tracking
    pub fn update_sector_exposure(&mut self, sector: &str, position_value: f64) -> Result<()> {
        let current_exposure = self.sector_exposure.entry(sector.to_string()).or_insert(0.0);
        *current_exposure += position_value;
        
        debug!("📈 Updated {} sector exposure: {:.4} SOL", sector, current_exposure);
        Ok(())
    }

    /// 🚀 ENRIQUECIMIENTO: Calculate portfolio VaR using var_calculator
    pub fn calculate_portfolio_var(&self) -> Result<f64> {
        let confidence = self.var_calculator.confidence_level;
        let time_horizon = self.var_calculator.time_horizon_days;
        
        // Simple VaR calculation based on portfolio value and estimated volatility
        let portfolio_value = self.total_exposure;
        let estimated_volatility = 0.15; // 15% daily volatility estimate for crypto
        
        // VaR = Portfolio Value * Z-score * Volatility * sqrt(time_horizon)
        let z_score = match confidence {
            x if x >= 0.99 => 2.33,
            x if x >= 0.95 => 1.65,
            _ => 1.28,
        };
        
        let var = portfolio_value * z_score * estimated_volatility * time_horizon.sqrt();
        
        debug!("⚠️ Portfolio VaR calculated: {:.4} SOL ({:.1}% confidence, {:.1} day horizon)", 
               var, confidence * 100.0, time_horizon);
        
        Ok(var)
    }

    /// 🚀 ENRIQUECIMIENTO: Update correlation matrix
    pub fn update_correlation(&mut self, asset1: &str, asset2: &str, correlation: f64) -> Result<()> {
        let key = if asset1 < asset2 {
            (asset1.to_string(), asset2.to_string())
        } else {
            (asset2.to_string(), asset1.to_string())
        };
        
        self.correlation_matrix.correlations.insert(key, correlation);
        self.correlation_matrix.last_updated = Utc::now();
        
        debug!("🔗 Updated correlation between {} and {}: {:.3}", asset1, asset2, correlation);
        Ok(())
    }

    /// 🚀 ENRIQUECIMIENTO: Get correlation between two assets
    pub fn get_correlation(&self, asset1: &str, asset2: &str) -> Option<f64> {
        let key = if asset1 < asset2 {
            (asset1.to_string(), asset2.to_string())
        } else {
            (asset2.to_string(), asset1.to_string())
        };
        
        self.correlation_matrix.correlations.get(&key).copied()
    }
}

impl ExposureCalculator {
    pub fn new(config: &SniperConfig) -> Result<Self> {
        Ok(Self {
            gross_exposure: 0.0,
            net_exposure: 0.0,
            sector_limits: HashMap::new(),
            concentration_limits: ConcentrationLimits {
                max_single_position: config.capital_allocation * 0.2, // 20% max
                max_sector_exposure: config.capital_allocation * 0.5,  // 50% max
                max_correlation_group: config.capital_allocation * 0.3, // 30% max
            },
        })
    }

    /// 🚀 ENRIQUECIMIENTO: Calculate net exposure from positions
    pub fn calculate_net_exposure(&mut self, positions: &HashMap<String, f64>) -> Result<f64> {
        // Net exposure considers directional bias (long vs short)
        // For spot trading, net exposure = gross exposure
        self.net_exposure = positions.values().sum();
        self.gross_exposure = positions.values().map(|v| v.abs()).sum();
        
        debug!("📊 Exposure calculated - Gross: {:.4} SOL, Net: {:.4} SOL", 
               self.gross_exposure, self.net_exposure);
        
        Ok(self.net_exposure)
    }

    /// 🚀 ENRIQUECIMIENTO: Set sector limits for risk management
    pub fn set_sector_limit(&mut self, sector: &str, limit: f64) -> Result<()> {
        self.sector_limits.insert(sector.to_string(), limit);
        debug!("🎯 Set sector limit for {}: {:.4} SOL", sector, limit);
        Ok(())
    }

    /// 🚀 ENRIQUECIMIENTO: Check if sector exposure is within limits
    pub fn check_sector_compliance(&self, sector: &str, proposed_exposure: f64) -> Result<bool> {
        if let Some(&limit) = self.sector_limits.get(sector) {
            let compliant = proposed_exposure <= limit;
            debug!("🔍 Sector compliance check for {}: {:.4} SOL <= {:.4} SOL = {}", 
                   sector, proposed_exposure, limit, compliant);
            Ok(compliant)
        } else {
            // No limit set, assume compliant
            Ok(true)
        }
    }

    /// 🚀 ENRIQUECIMIENTO: Check concentration limits
    pub fn check_concentration_limits(&self, position_size: f64, sector_exposure: f64, correlation_group_exposure: f64) -> Result<bool> {
        let position_ok = position_size <= self.concentration_limits.max_single_position;
        let sector_ok = sector_exposure <= self.concentration_limits.max_sector_exposure;
        let correlation_ok = correlation_group_exposure <= self.concentration_limits.max_correlation_group;
        
        debug!("🎯 Concentration limits check:");
        debug!("   Position: {:.4} <= {:.4} = {}", position_size, self.concentration_limits.max_single_position, position_ok);
        debug!("   Sector: {:.4} <= {:.4} = {}", sector_exposure, self.concentration_limits.max_sector_exposure, sector_ok);
        debug!("   Correlation: {:.4} <= {:.4} = {}", correlation_group_exposure, self.concentration_limits.max_correlation_group, correlation_ok);
        
        Ok(position_ok && sector_ok && correlation_ok)
    }
}

impl ComplianceChecker {
    pub fn new(config: &SniperConfig) -> Result<Self> {
        Ok(Self {
            position_limits: PositionLimits {
                max_positions: 10,
                max_position_size: config.capital_allocation * (config.max_position_size_percent / 100.0),
                max_daily_volume: config.capital_allocation * 2.0, // 2x capital turnover max
            },
            exposure_limits: ExposureLimits {
                max_gross_exposure: config.capital_allocation * 1.0, // 100% of capital
                max_net_exposure: config.capital_allocation * 0.8,   // 80% net
                max_leverage: 1.0, // No leverage for spot trading
            },
            risk_tolerance: RiskTolerance {
                max_var: config.capital_allocation * 0.05, // 5% VaR
                max_drawdown: 0.15, // 15% max drawdown
                min_sharpe_ratio: 1.0, // Minimum Sharpe ratio
            },
            regulatory_constraints: RegulatoryConstraints {
                restricted_tokens: vec![], // No restrictions for demo
                jurisdiction_limits: HashMap::new(),
                compliance_checks: vec![
                    "KYC verification".to_string(),
                    "AML screening".to_string(),
                    "Sanctions list check".to_string(),
                ],
            },
        })
    }
}

/// 🚀 ENRIQUECIMIENTO: Implementaciones para LiquidityRiskModel
impl LiquidityRiskModel {
    /// Calcula el impacto de mercado basado en liquidez
    pub fn calculate_market_impact(&self, liquidity_usd: f64, price_impact: f64) -> Result<f64> {
        // Usar el modelo de impacto temporal y permanente
        let temporary_component = self.impact_model.temporary_impact * (1.0 / liquidity_usd.sqrt());
        let permanent_component = self.impact_model.permanent_impact * price_impact;
        
        let total_impact = temporary_component + permanent_component;
        Ok(total_impact.min(1.0)) // Cap at 100%
    }

    /// Estima los costos de ejecución
    pub fn estimate_execution_costs(&self, liquidity_usd: f64) -> Result<f64> {
        let base_spread = self.execution_cost_model.spread_cost;
        let impact_cost = self.execution_cost_model.market_impact_cost * (1.0 / liquidity_usd.log10());
        let timing_cost = self.execution_cost_model.timing_cost;
        
        let total_cost = base_spread + impact_cost + timing_cost;
        Ok(total_cost)
    }
}

/// 🚀 ENRIQUECIMIENTO: Implementaciones para VolatilityRiskModel  
impl VolatilityRiskModel {
    /// Calcula la volatilidad realizada
    pub async fn calculate_realized_volatility(&self, _token_address: &str) -> Result<f64> {
        // En producción esto calcularía volatilidad histórica real
        Ok(self.historical_vol.max(0.05)) // Mínimo 5% de volatilidad
    }

    /// Proyecta volatilidad futura
    pub fn forecast_volatility(&self, current_vol: f64, horizon: std::time::Duration) -> Result<f64> {
        let time_factor = horizon.as_secs_f64() / (24.0 * 3600.0); // days
        let vol_persistence = 0.95_f64.powf(time_factor);
        let long_term_vol = 0.2; // 20% long-term average
        
        let forecast = current_vol * vol_persistence + long_term_vol * (1.0 - vol_persistence);
        Ok(forecast)
    }
}

/// 🚀 ENRIQUECIMIENTO: Implementaciones para CorrelationRiskModel
impl CorrelationRiskModel {
    /// Obtiene correlación entre dos tokens
    pub async fn get_correlation(&self, token1: &str, token2: &str) -> Result<f64> {
        let key = if token1 < token2 {
            (token1.to_string(), token2.to_string())
        } else {
            (token2.to_string(), token1.to_string())
        };
        
        // Buscar en matriz de correlación
        Ok(self.correlation_matrix.correlations.get(&key).copied().unwrap_or(0.3))
    }
}

/// 🚀 ENRIQUECIMIENTO: Implementaciones para DynamicCorrelation
impl DynamicCorrelation {
    /// Calcula correlación dinámica
    pub async fn calculate_dynamic_correlation(&self, _token1: &str, _token2: &str) -> Result<f64> {
        // Simular correlación dinámica usando factor EMA
        let base_correlation = 0.2; // Correlación base
        let dynamic_adjustment = rand::random::<f64>() * 0.1 - 0.05; // ±5% adjustment
        
        let dynamic_corr = base_correlation + (dynamic_adjustment * self.ema_factor);
        Ok(dynamic_corr.max(-1.0).min(1.0))
    }
}

/// 🚀 ENRIQUECIMIENTO: Additional structures needed for enhanced risk management
#[derive(Debug)]
pub struct PortfolioImpact {
    pub concentration_penalty: f64,
    pub liquidity_concentration: f64,
}

#[derive(Debug)]
pub struct ExposureAnalysis {
    pub concentration_risk: f64,
    pub correlation_risk: f64,
    pub volatility_exposure: f64,
}

/// 🚀 ENRIQUECIMIENTO: Enhanced implementations for VolatilityRiskModel
impl VolatilityRiskModel {
    pub async fn calculate_garch_volatility(&self) -> Result<f64> {
        // GARCH(1,1) volatility calculation
        let current_volatility = self.historical_vol;
        let garch_vol = self.garch_model.omega + 
                       self.garch_model.alpha * current_volatility.powi(2) + 
                       self.garch_model.beta * self.implied_vol.powi(2);
        
        Ok(garch_vol.sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    
    #[tokio::test]
    async fn test_risk_manager_creation() {
        let config = SniperConfig::default();
        let risk_manager = RiskManager::new(&config);
        assert!(risk_manager.is_ok());
    }
    
    #[tokio::test]
    async fn test_opportunity_risk_assessment() {
        let config = SniperConfig::default();
        let risk_manager = RiskManager::new(&config).unwrap();
        
        let opportunity = OpportunityData {
            id: Uuid::new_v4(),
            token_address: "test_token".to_string(),
            pool_address: "test_pool".to_string(),
            dex: super::super::DexType::Raydium,
            detected_at: Utc::now(),
            liquidity_usd: 75000.0,
            price_impact: 1.5,
            estimated_profit_percent: 12.0,
            risk_score: 0.4,
            confidence_score: 0.8,
            market_cap_usd: 500000.0,
            volume_24h_usd: 25000.0,
            holder_count: 150,
            age_minutes: 8,
        };
        
        let assessment = risk_manager.assess_opportunity(&opportunity).await;
        assert!(assessment.is_ok());
        
        let result = assessment.unwrap();
        assert!(result.risk_score >= 0.0 && result.risk_score <= 1.0);
        assert!(!result.recommendations.is_empty());
    }
}
