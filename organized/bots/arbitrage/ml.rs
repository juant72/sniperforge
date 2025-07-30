// ================================================================================
// MACHINE LEARNING - Reconocimiento de Patrones para Arbitraje
// ================================================================================

use std::collections::{HashMap, VecDeque};
use std::time::{Instant, Duration};
use anyhow::Result;
use tracing::{info, debug, warn};
use serde::{Serialize, Deserialize};

use sniperforge_core::{CoreResult, CoreError};
use crate::config::MlConfig;
use crate::engine::ArbitrageOpportunity;

/// Sistema de reconocimiento de patrones para arbitraje
pub struct PatternRecognition {
    config: MlConfig,
    pattern_history: VecDeque<OpportunityPattern>,
    success_patterns: HashMap<String, PatternScore>,
    feature_weights: FeatureWeights,
    model_state: ModelState,
    statistics: MlStatistics,
    last_model_update: Instant,
}

/// Patrón de oportunidad para ML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityPattern {
    pub opportunity_id: String,
    pub features: OpportunityFeatures,
    pub outcome: OpportunityOutcome,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub execution_time: Option<Duration>,
}

/// Características de la oportunidad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityFeatures {
    pub profit_percentage: f64,
    pub expected_profit_sol: f64,
    pub price_impact: f64,
    pub execution_time_estimate: f64,
    pub token_pair_volatility: f64,
    pub dex_liquidity_score: f64,
    pub market_conditions: MarketConditions,
    pub time_features: TimeFeatures,
    pub technical_indicators: TechnicalIndicators,
}

/// Condiciones del mercado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConditions {
    pub overall_volatility: f64,
    pub trading_volume_24h: f64,
    pub market_trend: MarketTrend,
    pub gas_price_level: GasPriceLevel,
    pub network_congestion: f64,
}

/// Tendencia del mercado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketTrend {
    Bullish,
    Bearish,
    Sideways,
    Volatile,
}

/// Nivel de precio de gas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GasPriceLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Características temporales
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeFeatures {
    pub hour_of_day: u8,
    pub day_of_week: u8,
    pub is_weekend: bool,
    pub is_market_hours: bool,
    pub time_since_last_opportunity: f64, // seconds
}

/// Indicadores técnicos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalIndicators {
    pub rsi: f64,
    pub moving_average_convergence: f64,
    pub bollinger_position: f64,
    pub volume_weighted_average_price: f64,
    pub relative_strength: f64,
}

/// Resultado de la oportunidad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityOutcome {
    pub executed: bool,
    pub successful: bool,
    pub actual_profit_sol: f64,
    pub execution_time_actual: Option<Duration>,
    pub failure_reason: Option<String>,
    pub slippage_experienced: f64,
}

/// Score de patrón
#[derive(Debug, Clone)]
pub struct PatternScore {
    pub success_rate: f64,
    pub avg_profit: f64,
    pub pattern_count: u32,
    pub confidence: f64,
    pub last_updated: Instant,
}

/// Pesos de características
#[derive(Debug, Clone)]
pub struct FeatureWeights {
    pub profit_percentage: f64,
    pub price_impact: f64,
    pub liquidity_score: f64,
    pub volatility: f64,
    pub time_features: f64,
    pub technical_indicators: f64,
    pub market_conditions: f64,
}

/// Estado del modelo
#[derive(Debug, Clone)]
pub struct ModelState {
    pub trained_patterns: u32,
    pub accuracy: f64,
    pub last_accuracy_check: Instant,
    pub learning_iterations: u32,
    pub model_confidence: f64,
}

/// Estadísticas de ML
#[derive(Debug, Clone, Default)]
pub struct MlStatistics {
    pub patterns_analyzed: u64,
    pub predictions_made: u64,
    pub correct_predictions: u64,
    pub total_profit_predicted: f64,
    pub total_profit_actual: f64,
    pub avg_confidence_score: f64,
    pub model_updates: u32,
}

impl PatternRecognition {
    /// Crear nuevo sistema de reconocimiento de patrones
    pub fn new(config: &MlConfig) -> CoreResult<Self> {
        info!("🧠 Inicializando PatternRecognition ML System");
        
        Ok(Self {
            config: config.clone(),
            pattern_history: VecDeque::with_capacity(config.pattern_history_size as usize),
            success_patterns: HashMap::new(),
            feature_weights: FeatureWeights::default(),
            model_state: ModelState::default(),
            statistics: MlStatistics::default(),
            last_model_update: Instant::now(),
        })
    }
    
    /// Puntuar oportunidad usando ML
    pub async fn score_opportunity(&mut self, opportunity: &ArbitrageOpportunity) -> CoreResult<f64> {
        if !self.config.enabled {
            return Ok(0.5); // Neutral score if ML disabled
        }
        
        // Extract features from opportunity
        let features = self.extract_features(opportunity).await?;
        
        // Calculate ML score based on patterns and weights
        let ml_score = self.calculate_ml_score(&features).await?;
        
        // Update statistics
        self.statistics.predictions_made += 1;
        self.statistics.patterns_analyzed += 1;
        
        // Check if model needs updating
        if self.should_update_model() {
            self.update_model().await?;
        }
        
        debug!("🧠 ML Score para oportunidad {}: {:.4}", opportunity.id, ml_score);
        
        Ok(ml_score)
    }
    
    /// Extraer características de la oportunidad
    async fn extract_features(&self, opportunity: &ArbitrageOpportunity) -> CoreResult<OpportunityFeatures> {
        let now = chrono::Utc::now();
        
        Ok(OpportunityFeatures {
            profit_percentage: opportunity.profit_percentage,
            expected_profit_sol: opportunity.expected_profit_sol,
            price_impact: opportunity.price_impact,
            execution_time_estimate: opportunity.execution_time_estimate_ms as f64,
            token_pair_volatility: self.calculate_token_pair_volatility(&opportunity.token_in.symbol, &opportunity.token_out.symbol).await?,
            dex_liquidity_score: self.calculate_dex_liquidity_score(&opportunity.dex_from.name).await?,
            market_conditions: self.get_current_market_conditions().await?,
            time_features: TimeFeatures {
                hour_of_day: now.hour() as u8,
                day_of_week: now.weekday().number_from_monday() as u8,
                is_weekend: now.weekday().number_from_monday() > 5,
                is_market_hours: self.is_market_hours(now),
                time_since_last_opportunity: self.get_time_since_last_opportunity(),
            },
            technical_indicators: self.calculate_technical_indicators(&opportunity.token_in.symbol).await?,
        })
    }
    
    /// Calcular score ML basado en características
    async fn calculate_ml_score(&self, features: &OpportunityFeatures) -> CoreResult<f64> {
        let mut score = 0.0;
        
        // Profit percentage component (higher is better)
        let profit_component = (features.profit_percentage * 100.0).min(10.0) / 10.0;
        score += profit_component * self.feature_weights.profit_percentage;
        
        // Price impact component (lower is better)
        let impact_component = (1.0 - features.price_impact.min(0.1) / 0.1);
        score += impact_component * self.feature_weights.price_impact;
        
        // Liquidity component (higher is better)
        score += features.dex_liquidity_score * self.feature_weights.liquidity_score;
        
        // Volatility component (moderate is better)
        let volatility_component = if features.token_pair_volatility > 0.5 {
            1.0 - (features.token_pair_volatility - 0.5) / 0.5
        } else {
            features.token_pair_volatility / 0.5
        };
        score += volatility_component * self.feature_weights.volatility;
        
        // Time features component
        let time_component = self.calculate_time_score(&features.time_features);
        score += time_component * self.feature_weights.time_features;
        
        // Technical indicators component
        let technical_component = self.calculate_technical_score(&features.technical_indicators);
        score += technical_component * self.feature_weights.technical_indicators;
        
        // Market conditions component
        let market_component = self.calculate_market_score(&features.market_conditions);
        score += market_component * self.feature_weights.market_conditions;
        
        // Pattern matching bonus
        let pattern_bonus = self.get_pattern_matching_bonus(features).await?;
        score += pattern_bonus * 0.1; // 10% weight for pattern matching
        
        // Normalize score to 0-1 range
        Ok(score.max(0.0).min(1.0))
    }
    
    /// Calcular volatilidad del par de tokens
    async fn calculate_token_pair_volatility(&self, token_a: &str, token_b: &str) -> CoreResult<f64> {
        // Simplified volatility calculation
        // In reality, you'd analyze price history
        let base_volatility = match (token_a, token_b) {
            ("SOL", _) | (_, "SOL") => 0.3,
            ("USDC", _) | (_, "USDC") => 0.1,
            ("USDT", _) | (_, "USDT") => 0.1,
            _ => 0.5, // Higher volatility for other tokens
        };
        
        Ok(base_volatility)
    }
    
    /// Calcular score de liquidez del DEX
    async fn calculate_dex_liquidity_score(&self, dex_name: &str) -> CoreResult<f64> {
        // Simplified liquidity scoring
        let score = match dex_name {
            "Raydium" => 0.9,
            "Orca" => 0.8,
            "Serum" => 0.7,
            "Phoenix" => 0.6,
            _ => 0.5,
        };
        
        Ok(score)
    }
    
    /// Obtener condiciones actuales del mercado
    async fn get_current_market_conditions(&self) -> CoreResult<MarketConditions> {
        // Simplified market conditions
        // In reality, you'd fetch from market data APIs
        Ok(MarketConditions {
            overall_volatility: 0.3,
            trading_volume_24h: 1000000.0,
            market_trend: MarketTrend::Sideways,
            gas_price_level: GasPriceLevel::Medium,
            network_congestion: 0.5,
        })
    }
    
    /// Verificar si es horario de mercado
    fn is_market_hours(&self, time: chrono::DateTime<chrono::Utc>) -> bool {
        // Crypto markets are 24/7, but traditional market hours might affect some tokens
        let hour = time.hour();
        let weekday = time.weekday().number_from_monday();
        
        // Consider traditional market hours for some analysis
        weekday <= 5 && hour >= 9 && hour <= 17
    }
    
    /// Obtener tiempo desde última oportunidad
    fn get_time_since_last_opportunity(&self) -> f64 {
        if let Some(last_pattern) = self.pattern_history.back() {
            let duration = chrono::Utc::now() - last_pattern.timestamp;
            duration.num_seconds() as f64
        } else {
            3600.0 // Default to 1 hour
        }
    }
    
    /// Calcular indicadores técnicos
    async fn calculate_technical_indicators(&self, token: &str) -> CoreResult<TechnicalIndicators> {
        // Simplified technical indicators
        // In reality, you'd calculate from price history
        Ok(TechnicalIndicators {
            rsi: 50.0, // Neutral
            moving_average_convergence: 0.0,
            bollinger_position: 0.5,
            volume_weighted_average_price: 1.0,
            relative_strength: 1.0,
        })
    }
    
    /// Calcular score temporal
    fn calculate_time_score(&self, time_features: &TimeFeatures) -> f64 {
        let mut score = 0.5; // Base score
        
        // Higher activity during market hours
        if time_features.is_market_hours {
            score += 0.2;
        }
        
        // Lower activity on weekends for some tokens
        if time_features.is_weekend {
            score -= 0.1;
        }
        
        // Peak hours bonus (9-11 AM, 2-4 PM UTC)
        if (time_features.hour_of_day >= 9 && time_features.hour_of_day <= 11) ||
           (time_features.hour_of_day >= 14 && time_features.hour_of_day <= 16) {
            score += 0.1;
        }
        
        // Recent activity bonus
        if time_features.time_since_last_opportunity < 300.0 { // < 5 minutes
            score += 0.1;
        }
        
        score.max(0.0).min(1.0)
    }
    
    /// Calcular score técnico
    fn calculate_technical_score(&self, indicators: &TechnicalIndicators) -> f64 {
        let mut score = 0.5;
        
        // RSI component
        let rsi_score = if indicators.rsi > 30.0 && indicators.rsi < 70.0 {
            0.8 // Good range
        } else {
            0.3 // Overbought/oversold
        };
        score = score * 0.5 + rsi_score * 0.5;
        
        // Bollinger bands position
        let bb_score = if indicators.bollinger_position > 0.2 && indicators.bollinger_position < 0.8 {
            0.8
        } else {
            0.4
        };
        score = score * 0.7 + bb_score * 0.3;
        
        score
    }
    
    /// Calcular score de mercado
    fn calculate_market_score(&self, conditions: &MarketConditions) -> f64 {
        let mut score = 0.5;
        
        // Volatility component
        if conditions.overall_volatility > 0.2 && conditions.overall_volatility < 0.6 {
            score += 0.2; // Good volatility for arbitrage
        }
        
        // Gas price component
        score += match conditions.gas_price_level {
            GasPriceLevel::Low => 0.3,
            GasPriceLevel::Medium => 0.1,
            GasPriceLevel::High => -0.1,
            GasPriceLevel::VeryHigh => -0.3,
        };
        
        // Network congestion
        score -= conditions.network_congestion * 0.2;
        
        score.max(0.0).min(1.0)
    }
    
    /// Obtener bonus de coincidencia de patrones
    async fn get_pattern_matching_bonus(&self, features: &OpportunityFeatures) -> CoreResult<f64> {
        let mut best_match_score = 0.0;
        
        // Find most similar patterns in history
        for pattern in &self.pattern_history {
            let similarity = self.calculate_pattern_similarity(features, &pattern.features);
            if similarity > 0.7 && pattern.outcome.successful {
                best_match_score = best_match_score.max(similarity * pattern.outcome.actual_profit_sol / 0.01);
            }
        }
        
        Ok(best_match_score.min(1.0))
    }
    
    /// Calcular similitud entre patrones
    fn calculate_pattern_similarity(&self, features1: &OpportunityFeatures, features2: &OpportunityFeatures) -> f64 {
        let profit_sim = 1.0 - (features1.profit_percentage - features2.profit_percentage).abs() / 0.1;
        let impact_sim = 1.0 - (features1.price_impact - features2.price_impact).abs() / 0.1;
        let liquidity_sim = 1.0 - (features1.dex_liquidity_score - features2.dex_liquidity_score).abs();
        
        (profit_sim + impact_sim + liquidity_sim) / 3.0
    }
    
    /// Reportar resultado de oportunidad
    pub fn report_opportunity_outcome(&mut self, opportunity_id: &str, outcome: OpportunityOutcome) -> CoreResult<()> {
        // Create pattern from outcome
        if let Some(features) = self.get_features_for_opportunity(opportunity_id) {
            let pattern = OpportunityPattern {
                opportunity_id: opportunity_id.to_string(),
                features,
                outcome: outcome.clone(),
                timestamp: chrono::Utc::now(),
                execution_time: outcome.execution_time_actual,
            };
            
            // Add to history
            self.pattern_history.push_back(pattern);
            
            // Maintain history size limit
            if self.pattern_history.len() > self.config.pattern_history_size as usize {
                self.pattern_history.pop_front();
            }
            
            // Update statistics
            if outcome.executed {
                if outcome.successful {
                    self.statistics.correct_predictions += 1;
                    self.statistics.total_profit_actual += outcome.actual_profit_sol;
                }
            }
            
            info!("📊 ML outcome reportado para {}: success={}", opportunity_id, outcome.successful);
        }
        
        Ok(())
    }
    
    /// Obtener características para oportunidad específica
    fn get_features_for_opportunity(&self, opportunity_id: &str) -> Option<OpportunityFeatures> {
        // In a real implementation, you'd store features temporarily
        // For now, return None (simplified)
        None
    }
    
    /// Verificar si el modelo necesita actualización
    fn should_update_model(&self) -> bool {
        let time_since_update = self.last_model_update.elapsed();
        let update_interval = Duration::from_secs(self.config.model_update_interval_minutes as u64 * 60);
        
        time_since_update > update_interval || self.pattern_history.len() > 100
    }
    
    /// Actualizar modelo ML
    async fn update_model(&mut self) -> CoreResult<()> {
        info!("🔄 Actualizando modelo ML");
        
        // Update feature weights based on recent outcomes
        self.update_feature_weights().await?;
        
        // Update pattern scores
        self.update_pattern_scores().await?;
        
        // Calculate new accuracy
        self.calculate_model_accuracy();
        
        // Update model state
        self.model_state.learning_iterations += 1;
        self.model_state.last_accuracy_check = Instant::now();
        self.last_model_update = Instant::now();
        
        self.statistics.model_updates += 1;
        
        info!("✅ Modelo ML actualizado. Accuracy: {:.2}%", self.model_state.accuracy * 100.0);
        
        Ok(())
    }
    
    /// Actualizar pesos de características
    async fn update_feature_weights(&mut self) -> CoreResult<()> {
        // Analyze which features correlate best with successful outcomes
        let mut feature_performance = HashMap::new();
        
        for pattern in &self.pattern_history {
            if pattern.outcome.executed {
                let success_weight = if pattern.outcome.successful { 1.0 } else { -0.5 };
                
                // Update feature correlations
                *feature_performance.entry("profit").or_insert(0.0) += 
                    pattern.features.profit_percentage * success_weight;
                *feature_performance.entry("impact").or_insert(0.0) += 
                    (1.0 - pattern.features.price_impact) * success_weight;
                *feature_performance.entry("liquidity").or_insert(0.0) += 
                    pattern.features.dex_liquidity_score * success_weight;
            }
        }
        
        // Adjust weights based on performance (simplified)
        let learning_rate = self.config.learning_rate;
        if let Some(&profit_perf) = feature_performance.get("profit") {
            self.feature_weights.profit_percentage = 
                (self.feature_weights.profit_percentage * (1.0 - learning_rate) + 
                 profit_perf.abs() * learning_rate).max(0.1).min(1.0);
        }
        
        Ok(())
    }
    
    /// Actualizar scores de patrones
    async fn update_pattern_scores(&mut self) -> CoreResult<()> {
        // Update success patterns based on recent outcomes
        for pattern in &self.pattern_history {
            if pattern.outcome.executed {
                let pattern_key = self.create_pattern_key(&pattern.features);
                
                let score = self.success_patterns.entry(pattern_key).or_insert(PatternScore {
                    success_rate: 0.5,
                    avg_profit: 0.0,
                    pattern_count: 0,
                    confidence: 0.5,
                    last_updated: Instant::now(),
                });
                
                // Update pattern score
                score.pattern_count += 1;
                let success = if pattern.outcome.successful { 1.0 } else { 0.0 };
                score.success_rate = (score.success_rate * (score.pattern_count - 1) as f64 + success) / score.pattern_count as f64;
                score.avg_profit = (score.avg_profit * (score.pattern_count - 1) as f64 + pattern.outcome.actual_profit_sol) / score.pattern_count as f64;
                score.confidence = (score.success_rate * score.pattern_count as f64 / (score.pattern_count as f64 + 10.0)).min(1.0);
                score.last_updated = Instant::now();
            }
        }
        
        Ok(())
    }
    
    /// Crear clave de patrón
    fn create_pattern_key(&self, features: &OpportunityFeatures) -> String {
        format!("profit_{:.1}_impact_{:.2}_liquidity_{:.1}", 
                features.profit_percentage * 100.0,
                features.price_impact * 100.0,
                features.dex_liquidity_score * 10.0)
    }
    
    /// Calcular accuracy del modelo
    fn calculate_model_accuracy(&mut self) {
        let recent_patterns: Vec<_> = self.pattern_history.iter()
            .filter(|p| p.outcome.executed)
            .take(100) // Last 100 executed patterns
            .collect();
        
        if !recent_patterns.is_empty() {
            let correct = recent_patterns.iter()
                .filter(|p| p.outcome.successful)
                .count();
            
            self.model_state.accuracy = correct as f64 / recent_patterns.len() as f64;
            self.model_state.trained_patterns = recent_patterns.len() as u32;
        }
    }
    
    /// Obtener estadísticas
    pub fn get_statistics(&self) -> &MlStatistics {
        &self.statistics
    }
    
    /// Obtener estado del modelo
    pub fn get_model_state(&self) -> &ModelState {
        &self.model_state
    }
}

impl Default for FeatureWeights {
    fn default() -> Self {
        Self {
            profit_percentage: 0.3,
            price_impact: 0.2,
            liquidity_score: 0.15,
            volatility: 0.1,
            time_features: 0.1,
            technical_indicators: 0.1,
            market_conditions: 0.05,
        }
    }
}

impl Default for ModelState {
    fn default() -> Self {
        Self {
            trained_patterns: 0,
            accuracy: 0.5,
            last_accuracy_check: Instant::now(),
            learning_iterations: 0,
            model_confidence: 0.5,
        }
    }
}
