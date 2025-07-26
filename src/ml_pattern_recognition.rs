// ================================================================================
// ACCIÓN 8.1: PATTERN RECOGNITION ENGINE - MACHINE LEARNING INTEGRATION
// ================================================================================
// Sistema de ML integrado para optimización automática de trading
// Análisis predictivo de oportunidades y adaptive parameter tuning
// ================================================================================

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use tracing::{info, warn, debug};
use anyhow::Result;

// ================================================================================
// ACCIÓN 8.1: PATTERN RECOGNITION STRUCTURES
// ================================================================================

/// Datos históricos de oportunidades para análisis ML
#[derive(Debug, Clone)]
pub struct OpportunityPattern {
    pub timestamp: DateTime<Utc>,
    pub token_pair: String,
    pub profit_percentage: f64,
    pub execution_time_ms: u64,
    pub market_volatility: f64,
    pub liquidity_level: f64,
    pub success: bool,
    pub dex_source: String,
}

/// Métricas de mercado para análisis predictivo
#[derive(Debug, Clone)]
pub struct MarketCondition {
    pub timestamp: DateTime<Utc>,
    pub overall_volatility: f64,
    pub market_trend: MarketTrend,
    pub liquidity_index: f64,
    pub opportunity_frequency: f64,
    pub average_profit: f64,
}

#[derive(Debug, Clone)]
pub enum MarketTrend {
    Bullish,
    Bearish,
    Sideways,
    HighVolatility,
}

/// Scoring de calidad de oportunidades usando ML
#[derive(Debug, Clone)]
pub struct OpportunityScore {
    pub base_profit_score: f64,      // 0.0 - 1.0
    pub execution_probability: f64,   // 0.0 - 1.0
    pub risk_score: f64,             // 0.0 - 1.0 (lower is better)
    pub timing_score: f64,           // 0.0 - 1.0
    pub composite_score: f64,        // 0.0 - 1.0 (final score)
}

/// Predicción de parámetros adaptativos
#[derive(Debug, Clone)]
pub struct AdaptiveParameters {
    pub recommended_trade_size: f64,
    pub optimal_profit_threshold: f64,
    pub execution_urgency: f64,      // 0.0 - 1.0
    pub confidence_level: f64,       // 0.0 - 1.0
}

/// Motor de reconocimiento de patrones
pub struct PatternRecognitionEngine {
    // Historical data storage
    opportunity_history: VecDeque<OpportunityPattern>,
    market_conditions: VecDeque<MarketCondition>,
    
    // Learning parameters
    learning_rate: f64,
    pattern_window: usize,
    min_samples: usize,
    
    // Pattern weights (learned over time)
    profit_weight: f64,
    timing_weight: f64,
    volatility_weight: f64,
    liquidity_weight: f64,
    
    // Performance tracking
    predictions_made: u64,
    successful_predictions: u64,
    last_accuracy_check: Instant,
}

impl PatternRecognitionEngine {
    pub fn new() -> Self {
        info!("🧠 Inicializando Pattern Recognition Engine (ACCIÓN 8.1)");
        
        Self {
            opportunity_history: VecDeque::new(),
            market_conditions: VecDeque::new(),
            learning_rate: 0.01,
            pattern_window: 50,
            min_samples: 10,
            
            // Initial weights (will be adjusted via learning)
            profit_weight: 0.4,
            timing_weight: 0.25,
            volatility_weight: 0.2,
            liquidity_weight: 0.15,
            
            predictions_made: 0,
            successful_predictions: 0,
            last_accuracy_check: Instant::now(),
        }
    }
    
    /// ACCIÓN 8.1: Agregar nueva oportunidad al historial para aprendizaje
    pub fn record_opportunity(&mut self, opportunity: OpportunityPattern) {
        debug!("🧠 Recording opportunity pattern: {} - {:.4}% profit", 
               opportunity.token_pair, opportunity.profit_percentage);
        
        self.opportunity_history.push_back(opportunity);
        
        // Mantener ventana de datos manejable
        if self.opportunity_history.len() > 1000 {
            self.opportunity_history.pop_front();
        }
        
        // Aprender de los nuevos datos
        if self.opportunity_history.len() >= self.min_samples {
            self.update_pattern_weights();
        }
    }
    
    /// ACCIÓN 8.1: Actualizar condiciones de mercado
    pub fn update_market_conditions(&mut self, condition: MarketCondition) {
        debug!("📊 Updating market conditions: trend={:?}, volatility={:.3}", 
               condition.market_trend, condition.overall_volatility);
        
        self.market_conditions.push_back(condition);
        
        // Mantener últimas 24 horas de datos de mercado
        let cutoff = Utc::now() - chrono::Duration::hours(24);
        while let Some(front) = self.market_conditions.front() {
            if front.timestamp < cutoff {
                self.market_conditions.pop_front();
            } else {
                break;
            }
        }
    }
    
    /// ACCIÓN 8.1: Scoring ML de calidad de oportunidad (FIXED for real profits)
    pub fn score_opportunity(&mut self, 
                            token_pair: &str, 
                            profit_percentage: f64,
                            current_volatility: f64,
                            liquidity_level: f64) -> OpportunityScore {
        
        self.predictions_made += 1;
        
        // Base profit score - FIXED for real 0.1-0.5% profits
        let base_profit_score = if profit_percentage >= 0.5 {
            1.0  // Excellent profit (0.5%+)
        } else if profit_percentage >= 0.2 {
            0.8  // Very good profit (0.2-0.5%)
        } else if profit_percentage >= 0.1 {
            0.6  // Good profit (0.1-0.2%)
        } else if profit_percentage >= 0.05 {
            0.4  // Acceptable profit (0.05-0.1%)
        } else {
            0.2  // Low profit (<0.05%)
        };
        
        // Execution probability based on historical success rate
        let execution_probability = self.calculate_execution_probability(token_pair, profit_percentage);
        
        // Risk score based on volatility and market conditions (REDUCED risk weighting)
        let risk_score = self.calculate_risk_score(current_volatility, liquidity_level) * 0.5; // Less conservative
        
        // Timing score based on current market conditions
        let timing_score = self.calculate_timing_score();
        
        // Composite score using learned weights (BOOSTED for real trading)
        let composite_score = (
            base_profit_score * self.profit_weight +
            execution_probability * self.timing_weight +
            (1.0 - risk_score) * self.volatility_weight +
            timing_score * self.liquidity_weight
        ).min(1.0).max(0.0);
        
        let score = OpportunityScore {
            base_profit_score,
            execution_probability,
            risk_score,
            timing_score,
            composite_score,
        };
        
        debug!("🎯 ML Score for {}: composite={:.3}, profit={:.3}, exec={:.3}, risk={:.3}", 
               token_pair, composite_score, base_profit_score, execution_probability, risk_score);
        
        score
    }
    
    /// ACCIÓN 8.1: Predicción de parámetros adaptativos
    pub fn predict_adaptive_parameters(&self, 
                                     opportunity_score: &OpportunityScore,
                                     base_trade_size: f64) -> AdaptiveParameters {
        
        // Adaptive trade size based on confidence and market conditions
        let size_multiplier = if opportunity_score.composite_score > 0.8 {
            1.5 // Increase size for high-confidence opportunities
        } else if opportunity_score.composite_score > 0.6 {
            1.0 // Normal size for medium confidence
        } else {
            0.5 // Reduce size for low confidence
        };
        
        let recommended_trade_size = (base_trade_size * size_multiplier).min(0.01); // Max 0.01 SOL
        
        // Dynamic profit threshold
        let optimal_profit_threshold = if opportunity_score.execution_probability > 0.8 {
            0.002 // Lower threshold for high-probability opportunities
        } else {
            0.005 // Higher threshold for uncertain opportunities
        };
        
        // Execution urgency based on timing score
        let execution_urgency = opportunity_score.timing_score;
        
        // Overall confidence
        let confidence_level = (opportunity_score.composite_score + opportunity_score.execution_probability) / 2.0;
        
        AdaptiveParameters {
            recommended_trade_size,
            optimal_profit_threshold,
            execution_urgency,
            confidence_level,
        }
    }
    
    /// Aprender y actualizar pesos de patrón
    fn update_pattern_weights(&mut self) {
        if self.opportunity_history.len() < self.min_samples {
            return;
        }
        
        // Análisis simple de correlación
        let recent_opportunities: Vec<_> = self.opportunity_history
            .iter()
            .rev()
            .take(self.pattern_window)
            .collect();
        
        let successful_count = recent_opportunities.iter().filter(|o| o.success).count() as f64;
        let success_rate = successful_count / recent_opportunities.len() as f64;
        
        // Ajustar pesos basado en performance reciente
        if success_rate > 0.7 {
            // Good performance, increase confidence in current weights
            info!("🧠 ML Learning: High success rate ({:.1}%), maintaining strategy", success_rate * 100.0);
        } else if success_rate < 0.3 {
            // Poor performance, adjust weights
            warn!("🧠 ML Learning: Low success rate ({:.1}%), adjusting strategy", success_rate * 100.0);
            
            // Decrease profit weight, increase risk assessment
            self.profit_weight = (self.profit_weight - self.learning_rate).max(0.2);
            self.volatility_weight = (self.volatility_weight + self.learning_rate).min(0.4);
            
            info!("🧠 Adjusted weights: profit={:.3}, volatility={:.3}", 
                  self.profit_weight, self.volatility_weight);
        }
    }
    
    /// Calcular probabilidad de ejecución basada en historia
    fn calculate_execution_probability(&self, token_pair: &str, profit_percentage: f64) -> f64 {
        let similar_opportunities: Vec<_> = self.opportunity_history
            .iter()
            .filter(|o| o.token_pair == token_pair || 
                       (o.profit_percentage - profit_percentage).abs() < 0.01)
            .collect();
        
        if similar_opportunities.is_empty() {
            return 0.5; // Default probability
        }
        
        let successful = similar_opportunities.iter().filter(|o| o.success).count() as f64;
        successful / similar_opportunities.len() as f64
    }
    
    /// Calcular score de riesgo
    fn calculate_risk_score(&self, volatility: f64, liquidity: f64) -> f64 {
        // Higher volatility = higher risk
        let volatility_risk = (volatility / 10.0).min(1.0);
        
        // Lower liquidity = higher risk
        let liquidity_risk = (1.0 - liquidity.min(1.0)).max(0.0);
        
        // Combined risk score
        (volatility_risk * 0.6 + liquidity_risk * 0.4).min(1.0)
    }
    
    /// Calcular score de timing basado en condiciones actuales
    fn calculate_timing_score(&self) -> f64 {
        if let Some(latest_condition) = self.market_conditions.back() {
            match latest_condition.market_trend {
                MarketTrend::HighVolatility => 0.8, // Good for arbitrage
                MarketTrend::Sideways => 0.6,       // Decent for arbitrage
                MarketTrend::Bullish => 0.4,        // Less arbitrage opportunities
                MarketTrend::Bearish => 0.3,        // Risky conditions
            }
        } else {
            0.5 // Default when no market data
        }
    }
    
    /// Generar reporte de análisis ML
    pub fn generate_ml_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("🧠 MACHINE LEARNING ANALYSIS REPORT (ACCIÓN 8.1)\n");
        report.push_str("================================================\n\n");
        
        report.push_str(&format!("📊 Data Summary:\n"));
        report.push_str(&format!("• Opportunity History: {} samples\n", self.opportunity_history.len()));
        report.push_str(&format!("• Market Conditions: {} data points\n", self.market_conditions.len()));
        report.push_str(&format!("• Total Predictions: {}\n", self.predictions_made));
        
        if self.predictions_made > 0 {
            let accuracy = (self.successful_predictions as f64 / self.predictions_made as f64) * 100.0;
            report.push_str(&format!("• Prediction Accuracy: {:.1}%\n", accuracy));
        }
        
        report.push_str(&format!("\n🎯 Current ML Weights:\n"));
        report.push_str(&format!("• Profit Weight: {:.3}\n", self.profit_weight));
        report.push_str(&format!("• Timing Weight: {:.3}\n", self.timing_weight));
        report.push_str(&format!("• Volatility Weight: {:.3}\n", self.volatility_weight));
        report.push_str(&format!("• Liquidity Weight: {:.3}\n", self.liquidity_weight));
        
        if let Some(latest_condition) = self.market_conditions.back() {
            report.push_str(&format!("\n📈 Current Market Analysis:\n"));
            report.push_str(&format!("• Market Trend: {:?}\n", latest_condition.market_trend));
            report.push_str(&format!("• Volatility: {:.3}\n", latest_condition.overall_volatility));
            report.push_str(&format!("• Liquidity Index: {:.3}\n", latest_condition.liquidity_index));
        }
        
        report
    }
    
    /// Validar performance de predicciones
    pub fn validate_prediction(&mut self, prediction_was_correct: bool) {
        if prediction_was_correct {
            self.successful_predictions += 1;
        }
        
        // Check accuracy periodically
        if self.last_accuracy_check.elapsed() > Duration::from_secs(300) { // Every 5 minutes
            if self.predictions_made > 0 {
                let accuracy = (self.successful_predictions as f64 / self.predictions_made as f64) * 100.0;
                info!("🧠 ML Prediction Accuracy: {:.1}% ({}/{})", 
                      accuracy, self.successful_predictions, self.predictions_made);
            }
            self.last_accuracy_check = Instant::now();
        }
    }
}

/// ACCIÓN 8.1: Crear condición de mercado desde datos de oportunidades
pub fn analyze_market_condition(recent_opportunities: &[OpportunityPattern]) -> MarketCondition {
    if recent_opportunities.is_empty() {
        return MarketCondition {
            timestamp: Utc::now(),
            overall_volatility: 0.5,
            market_trend: MarketTrend::Sideways,
            liquidity_index: 0.5,
            opportunity_frequency: 0.0,
            average_profit: 0.0,
        };
    }
    
    // Calculate metrics
    let avg_profit: f64 = recent_opportunities.iter()
        .map(|o| o.profit_percentage)
        .sum::<f64>() / recent_opportunities.len() as f64;
    
    let volatility: f64 = recent_opportunities.iter()
        .map(|o| o.market_volatility)
        .sum::<f64>() / recent_opportunities.len() as f64;
    
    let liquidity: f64 = recent_opportunities.iter()
        .map(|o| o.liquidity_level)
        .sum::<f64>() / recent_opportunities.len() as f64;
    
    // Determine trend
    let _profit_variance: f64 = recent_opportunities.iter()
        .map(|o| (o.profit_percentage - avg_profit).powi(2))
        .sum::<f64>() / recent_opportunities.len() as f64;
    
    let market_trend = if volatility > 1.5 {
        MarketTrend::HighVolatility
    } else if avg_profit > 99.85 {
        MarketTrend::Bullish
    } else if avg_profit < 99.80 {
        MarketTrend::Bearish
    } else {
        MarketTrend::Sideways
    };
    
    MarketCondition {
        timestamp: Utc::now(),
        overall_volatility: volatility,
        market_trend,
        liquidity_index: liquidity,
        opportunity_frequency: recent_opportunities.len() as f64,
        average_profit: avg_profit,
    }
}
