//! Machine Learning Module for SniperForge
//! 
//! This module provides advanced AI/ML capabilities including:
//! - Pattern recognition for market prediction
//! - Strategy optimization using genetic algorithms
//! - Risk assessment using ML models
//! - Predictive timing for optimal trade execution
//! 
//! Phase 6B: Expected 20-30% improvement in risk-adjusted returns

pub mod pattern_recognition;
pub mod strategy_optimizer;
pub mod risk_assessment;
pub mod timing_predictor;
pub mod data_preprocessor;
pub mod model_manager;

pub use pattern_recognition::PatternRecognizer;
pub use strategy_optimizer::StrategyOptimizer;
pub use risk_assessment::RiskAssessor;
pub use timing_predictor::{TimingPredictor, TimingPrediction, ExecutionStrategy, TradeDirection};
pub use data_preprocessor::{DataPreprocessor, ProcessedFeatures, RawMarketData, DataQuality};
pub use model_manager::{ModelManager, ModelMetadata, ModelType, PerformanceMetrics};

use anyhow::Result;
use crate::strategies::MarketData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// ML Configuration for the entire system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLConfig {
    pub pattern_recognition: PatternRecognitionConfig,
    pub strategy_optimizer: StrategyOptimizerConfig,
    pub risk_assessment: RiskAssessmentConfig,
    pub timing_predictor: TimingPredictorConfig,
    pub data_retention_days: u32,
    pub model_update_interval_hours: u32,
    pub confidence_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRecognitionConfig {
    pub lstm_units: usize,
    pub sequence_length: usize,
    pub prediction_horizon: usize,
    pub technical_indicators: Vec<String>,
    pub min_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyOptimizerConfig {
    pub population_size: usize,
    pub generations: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub fitness_metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessmentConfig {
    pub volatility_window: usize,
    pub correlation_threshold: f64,
    pub market_regimes: Vec<String>,
    pub risk_metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingPredictorConfig {
    pub microstructure_features: Vec<String>,
    pub liquidity_threshold: f64,
    pub slippage_threshold: f64,
    pub execution_horizon_seconds: u64,
}

impl Default for MLConfig {
    fn default() -> Self {
        Self {
            pattern_recognition: PatternRecognitionConfig {
                lstm_units: 128,
                sequence_length: 60,
                prediction_horizon: 5,
                technical_indicators: vec![
                    "rsi".to_string(),
                    "macd".to_string(),
                    "bollinger".to_string(),
                    "volume_profile".to_string(),
                ],
                min_confidence: 0.7,
            },
            strategy_optimizer: StrategyOptimizerConfig {
                population_size: 50,
                generations: 100,
                mutation_rate: 0.1,
                crossover_rate: 0.8,
                fitness_metrics: vec![
                    "sharpe_ratio".to_string(),
                    "max_drawdown".to_string(),
                    "win_rate".to_string(),
                ],
            },
            risk_assessment: RiskAssessmentConfig {
                volatility_window: 24,
                correlation_threshold: 0.8,
                market_regimes: vec![
                    "bull".to_string(),
                    "bear".to_string(),
                    "sideways".to_string(),
                ],
                risk_metrics: vec![
                    "var".to_string(),
                    "expected_shortfall".to_string(),
                    "volatility".to_string(),
                ],
            },
            timing_predictor: TimingPredictorConfig {
                microstructure_features: vec![
                    "bid_ask_spread".to_string(),
                    "order_book_depth".to_string(),
                    "trade_intensity".to_string(),
                ],
                liquidity_threshold: 10000.0,
                slippage_threshold: 0.005,
                execution_horizon_seconds: 300,
            },
            data_retention_days: 90,
            model_update_interval_hours: 24,
            confidence_threshold: 0.75,
        }
    }
}

/// ML Prediction result with confidence scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLPrediction {
    pub prediction_type: String,
    pub value: f64,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
    pub features_used: Vec<String>,
    pub model_version: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl MLPrediction {
    pub fn new(
        prediction_type: String,
        value: f64,
        confidence: f64,
        features_used: Vec<String>,
        model_version: String,
    ) -> Self {
        Self {
            prediction_type,
            value,
            confidence,
            timestamp: Utc::now(),
            features_used,
            model_version,
            metadata: HashMap::new(),
        }
    }

    pub fn is_confident(&self, threshold: f64) -> bool {
        self.confidence >= threshold
    }

    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }
}

/// ML Feature vector for training and prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub features: HashMap<String, f64>,
    pub target: Option<f64>,
}

impl FeatureVector {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            timestamp: Utc::now(),
            features: HashMap::new(),
            target: None,
        }
    }

    pub fn add_feature(&mut self, name: String, value: f64) {
        self.features.insert(name, value);
    }

    pub fn set_target(&mut self, target: f64) {
        self.target = Some(target);
    }

    pub fn get_feature_vector(&self) -> Vec<f64> {
        let mut features: Vec<_> = self.features.iter().collect();
        features.sort_by_key(|&(key, _)| key);
        features.into_iter().map(|(_, &value)| value).collect()
    }
}

/// ML System performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLMetrics {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub sharpe_improvement: f64,
    pub predictions_made: u64,
    pub confident_predictions: u64,
    pub last_updated: DateTime<Utc>,
}

impl MLMetrics {
    pub fn new() -> Self {
        Self {
            accuracy: 0.0,
            precision: 0.0,
            recall: 0.0,
            f1_score: 0.0,
            sharpe_improvement: 0.0,
            predictions_made: 0,
            confident_predictions: 0,
            last_updated: Utc::now(),
        }
    }

    pub fn update_accuracy(&mut self, correct: u64, total: u64) {
        if total > 0 {
            self.accuracy = correct as f64 / total as f64;
        }
        self.last_updated = Utc::now();
    }

    pub fn confidence_rate(&self) -> f64 {
        if self.predictions_made > 0 {
            self.confident_predictions as f64 / self.predictions_made as f64
        } else {
            0.0
        }
    }
}

/// Main ML Engine that coordinates all ML modules
pub struct MLEngine {
    pub config: MLConfig,
    pub pattern_recognizer: PatternRecognizer,
    pub strategy_optimizer: StrategyOptimizer,
    pub risk_assessor: RiskAssessor,
    pub timing_predictor: TimingPredictor,
    pub data_preprocessor: DataPreprocessor,
    pub model_manager: ModelManager,
    pub metrics: MLMetrics,
}

impl MLEngine {
    pub async fn new(config: MLConfig) -> Result<Self> {
        let pattern_recognizer = PatternRecognizer::new(config.pattern_recognition.clone()).await?;
        let strategy_optimizer = StrategyOptimizer::new(config.strategy_optimizer.clone()).await?;
        let risk_assessor = RiskAssessor::new(config.risk_assessment.clone()).await?;
        let timing_predictor = TimingPredictor::new(config.timing_predictor.clone()).await?;
        let data_preprocessor = DataPreprocessor::new().await?;
        let model_manager = ModelManager::new().await?;

        Ok(Self {
            config,
            pattern_recognizer,
            strategy_optimizer,
            risk_assessor,
            timing_predictor,
            data_preprocessor,
            model_manager,
            metrics: MLMetrics::new(),
        })    }

    /// Generate comprehensive ML prediction for a trading opportunity
    pub async fn predict_trading_opportunity(
        &mut self,
        symbol: &str,
        market_data: &MarketData,
    ) -> Result<MLPrediction> {
        // Preprocess data
        let features = self.data_preprocessor
            .extract_features(symbol, market_data).await?;

        // Get pattern recognition prediction
        let pattern_prediction = self.pattern_recognizer
            .predict_price_movement(&features).await?;

        // Get risk assessment
        let risk_score = self.risk_assessor
            .assess_trade_risk(&features).await?;

        // Get timing recommendation
        let timing_score = self.timing_predictor
            .predict_optimal_timing(&features).await?;

        // Combine predictions with weighted average
        let combined_confidence = (
            pattern_prediction.confidence * 0.4 +
            risk_score.confidence * 0.3 +
            timing_score.confidence * 0.3
        );

        let combined_value = (
            pattern_prediction.value * 0.5 +
            timing_score.value * 0.5
        ) * (1.0 - risk_score.value); // Risk adjustment

        let mut prediction = MLPrediction::new(
            "combined_trading_opportunity".to_string(),
            combined_value,
            combined_confidence,
            features.features.keys().cloned().collect(),
            "v1.0".to_string(),
        );

        prediction.add_metadata("pattern_prediction".to_string(), 
            serde_json::to_value(&pattern_prediction)?);
        prediction.add_metadata("risk_score".to_string(), 
            serde_json::to_value(&risk_score)?);
        prediction.add_metadata("timing_score".to_string(), 
            serde_json::to_value(&timing_score)?);

        // Update metrics
        self.metrics.predictions_made += 1;
        if combined_confidence >= self.config.confidence_threshold {
            self.metrics.confident_predictions += 1;
        }

        Ok(prediction)
    }

    /// Optimize trading strategy using genetic algorithms
    pub async fn optimize_strategy(
        &mut self,
        historical_data: &[MarketData],
        strategy_params: &HashMap<String, f64>,
    ) -> Result<HashMap<String, f64>> {
        self.strategy_optimizer
            .optimize_parameters(historical_data, strategy_params).await
    }

    /// Update ML models with new market data
    pub async fn update_models(&mut self, training_data: &[FeatureVector]) -> Result<()> {
        self.pattern_recognizer.retrain(training_data).await?;
        self.risk_assessor.update_risk_models(training_data).await?;
        self.timing_predictor.update_timing_models(training_data).await?;
        
        tracing::info!("ML models updated with {} training samples", training_data.len());
        Ok(())
    }

    /// Get current ML system performance metrics
    pub fn get_metrics(&self) -> &MLMetrics {
        &self.metrics
    }

    /// Check if ML system is ready for production use
    pub fn is_ready(&self) -> bool {
        self.metrics.confidence_rate() >= 0.6 && 
        self.metrics.predictions_made >= 100
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_config_default() {
        let config = MLConfig::default();
        assert!(config.confidence_threshold > 0.0);
        assert!(!config.pattern_recognition.technical_indicators.is_empty());
    }

    #[test]
    fn test_feature_vector() {
        let mut features = FeatureVector::new("SOL/USDC".to_string());
        features.add_feature("price".to_string(), 100.0);
        features.add_feature("volume".to_string(), 1000.0);
        
        let vector = features.get_feature_vector();
        assert_eq!(vector.len(), 2);
    }

    #[test]
    fn test_ml_prediction() {
        let prediction = MLPrediction::new(
            "price_movement".to_string(),
            0.8,
            0.9,
            vec!["rsi".to_string(), "volume".to_string()],
            "v1.0".to_string(),
        );
        
        assert!(prediction.is_confident(0.8));
        assert!(!prediction.is_confident(0.95));
    }
}
