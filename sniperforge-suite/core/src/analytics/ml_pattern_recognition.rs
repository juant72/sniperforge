// ================================================================================
// ML PATTERN RECOGNITION ENGINE (MIGRATED FROM WORKING BOT)
// ================================================================================
// Sistema de ML integrado para optimización automática de trading
// Análisis predictivo de oportunidades y adaptive parameter tuning
// ================================================================================

use std::collections::{HashMap, VecDeque};
use std::time::Instant;
use chrono::{DateTime, Utc};
use tracing::{info, debug};

// ================================================================================
// PATTERN RECOGNITION STRUCTURES (MIGRATED FROM WORKING BOT)
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

/// Motor de reconocimiento de patrones (migrado del bot que funciona)
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
        info!("🧠 Inicializando Pattern Recognition Engine");
        
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
    
    /// Agregar nueva oportunidad al historial para aprendizaje
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
    
    /// Actualizar condiciones de mercado
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
    
    /// Scoring ML de calidad de oportunidad (optimizado para profits reales)
    pub fn score_opportunity(&mut self, 
                            token_pair: &str, 
                            profit_percentage: f64,
                            liquidity_level: f64,
                            current_volatility: f64) -> OpportunityScore {
        
        self.predictions_made += 1;
        
        // Base profit score - optimizado para profits reales 0.1-0.5%
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
        
        // Risk score based on volatility and market conditions
        let risk_score = self.calculate_risk_score(current_volatility, liquidity_level) * 0.5; // Less conservative
        
        // Timing score based on current market conditions
        let timing_score = self.calculate_timing_score();
        
        // Composite score using learned weights
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
    
    /// Verificar y actualizar accuracy del modelo ML
    pub fn check_accuracy(&mut self) -> f64 {
        let now = Instant::now();
        
        // Solo verificar accuracy cada 10 minutos para performance
        if now.duration_since(self.last_accuracy_check).as_secs() < 600 {
            return self.get_current_accuracy();
        }
        
        self.last_accuracy_check = now;
        
        // Calcular accuracy basado en predicciones recientes vs resultados reales
        let accuracy = if self.predictions_made > 0 {
            (self.successful_predictions as f64 / self.predictions_made as f64) * 100.0
        } else {
            0.0
        };
        
        info!("🎯 ML Accuracy Check: {:.1}% ({}/{} predictions)", 
              accuracy, self.successful_predictions, self.predictions_made);
        
        // Auto-adjust learning parameters based on accuracy
        if accuracy < 70.0 {
            self.learning_rate = (self.learning_rate * 1.1).min(0.05); // Increase learning rate
            warn!("📉 Low accuracy detected, increasing learning rate to {:.4}", self.learning_rate);
        } else if accuracy > 90.0 {
            self.learning_rate = (self.learning_rate * 0.9).max(0.001); // Decrease learning rate
            info!("📈 High accuracy achieved, reducing learning rate to {:.4}", self.learning_rate);
        }
        
        accuracy
    }
    
    /// Reportar resultado de predicción para actualizar accuracy
    pub fn report_prediction_result(&mut self, was_successful: bool) {
        if was_successful {
            self.successful_predictions += 1;
        }
        
        // Trigger accuracy check if enough predictions accumulated
        if self.predictions_made % 50 == 0 {
            self.check_accuracy();
        }
    }
    
    /// Obtener accuracy actual sin recalcular
    pub fn get_current_accuracy(&self) -> f64 {
        if self.predictions_made > 0 {
            (self.successful_predictions as f64 / self.predictions_made as f64) * 100.0
        } else {
            0.0
        }
    }
    }
    
    /// Predicción de parámetros adaptativos
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
    
    /// Calcular probabilidad de ejecución basada en historial
    fn calculate_execution_probability(&self, token_pair: &str, profit_percentage: f64) -> f64 {
        if self.opportunity_history.is_empty() {
            return 0.7; // Default optimistic
        }
        
        // Filtrar oportunidades similares
        let similar_opportunities: Vec<_> = self.opportunity_history
            .iter()
            .filter(|opp| {
                opp.token_pair.contains(&token_pair.split('-').next().unwrap_or("")) &&
                (opp.profit_percentage - profit_percentage).abs() < 0.1
            })
            .collect();
        
        if similar_opportunities.is_empty() {
            return 0.6; // Default for new patterns
        }
        
        let success_rate = similar_opportunities.iter()
            .filter(|opp| opp.success)
            .count() as f64 / similar_opportunities.len() as f64;
        
        success_rate.max(0.1).min(0.95) // Clamp between 10% and 95%
    }
    
    /// Calcular score de riesgo basado en volatilidad y liquidez
    fn calculate_risk_score(&self, volatility: f64, liquidity: f64) -> f64 {
        let volatility_risk = (volatility * 10.0).min(1.0); // Normalize volatility
        let liquidity_risk = if liquidity > 100_000.0 {
            0.1 // Low risk for high liquidity
        } else if liquidity > 50_000.0 {
            0.3 // Medium risk
        } else {
            0.7 // High risk for low liquidity
        };
        
        (volatility_risk + liquidity_risk) / 2.0
    }
    
    /// Calcular score de timing basado en condiciones de mercado
    fn calculate_timing_score(&self) -> f64 {
        if self.market_conditions.is_empty() {
            return 0.7; // Default neutral timing
        }
        
        // Usar la condición de mercado más reciente
        let latest_condition = &self.market_conditions[self.market_conditions.len() - 1];
        
        match latest_condition.market_trend {
            MarketTrend::Bullish => 0.9,
            MarketTrend::Sideways => 0.7,
            MarketTrend::Bearish => 0.5,
            MarketTrend::HighVolatility => 0.6,
        }
    }
    
    /// Aprender y actualizar pesos de patrón
    fn update_pattern_weights(&mut self) {
        if self.opportunity_history.len() < self.min_samples {
            return;
        }
        
        // Análisis simple de correlación success vs features
        let recent_opportunities: Vec<_> = self.opportunity_history
            .iter()
            .rev()
            .take(self.pattern_window)
            .collect();
        
        let successful_count = recent_opportunities.iter()
            .filter(|opp| opp.success)
            .count() as f64;
        
        let total_count = recent_opportunities.len() as f64;
        let success_rate = successful_count / total_count;
        
        // Ajustar pesos basado en success rate
        if success_rate > 0.8 {
            // High success rate - increase confidence in current weights
            debug!("🎯 High success rate ({:.1}%) - maintaining current weights", success_rate * 100.0);
        } else if success_rate < 0.5 {
            // Low success rate - adjust weights
            self.profit_weight = (self.profit_weight + self.learning_rate).min(0.6);
            self.timing_weight = (self.timing_weight - self.learning_rate * 0.5).max(0.1);
            debug!("🔧 Low success rate ({:.1}%) - adjusting weights", success_rate * 100.0);
        }
        
        // Normalize weights
        let total_weight = self.profit_weight + self.timing_weight + self.volatility_weight + self.liquidity_weight;
        if total_weight > 1.1 || total_weight < 0.9 {
            let factor = 1.0 / total_weight;
            self.profit_weight *= factor;
            self.timing_weight *= factor;
            self.volatility_weight *= factor;
            self.liquidity_weight *= factor;
        }
    }
    
    /// Obtener estadísticas de performance del ML
    pub fn get_performance_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        let accuracy = if self.predictions_made > 0 {
            self.successful_predictions as f64 / self.predictions_made as f64
        } else {
            0.0
        };
        
        stats.insert("predictions_made".to_string(), self.predictions_made as f64);
        stats.insert("accuracy".to_string(), accuracy);
        stats.insert("data_points".to_string(), self.opportunity_history.len() as f64);
        stats.insert("profit_weight".to_string(), self.profit_weight);
        stats.insert("timing_weight".to_string(), self.timing_weight);
        
        stats
    }
    
    /// Marcar predicción como exitosa o fallida
    pub fn mark_prediction_result(&mut self, success: bool) {
        if success {
            self.successful_predictions += 1;
        }
        
        // Update weights periodically based on prediction results
        if self.predictions_made % 20 == 0 {
            self.update_pattern_weights();
        }
    }
}

impl Default for PatternRecognitionEngine {
    fn default() -> Self {
        Self::new()
    }
}
