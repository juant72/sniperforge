//! Machine Learning Module - Advanced ML Features
//! 
//! This module provides state-of-the-art machine learning capabilities for the
//! SniperForge enterprise trading system, including sentiment analysis, predictive
//! analytics, risk assessment, and portfolio optimization.

pub mod advanced_ml_engine;

// Re-export main ML components
pub use advanced_ml_engine::{
    AdvancedMLEngine, MLConfig, SentimentAnalysis, MarketPrediction, 
    RiskAssessment, PortfolioOptimization, PatternMatch, MLAnalysisResult,
    SentimentTrend, TrendDirection, RiskCategory, PatternType, ModelMetrics
};

/// ML Engine factory for creating configured ML instances
pub struct MLEngineFactory;

impl MLEngineFactory {
    /// Create a new ML engine with default configuration
    pub fn create_default() -> AdvancedMLEngine {
        AdvancedMLEngine::default()
    }
    
    /// Create a new ML engine with custom configuration
    pub fn create_with_config(config: MLConfig) -> AdvancedMLEngine {
        AdvancedMLEngine::new(config)
    }
    
    /// Create a production-optimized ML engine
    pub fn create_production() -> AdvancedMLEngine {
        let config = MLConfig {
            sentiment_threshold: 0.75,
            prediction_horizon: 10, // 10 minutes for production
            risk_tolerance: 0.25,   // Lower risk tolerance for production
            portfolio_rebalance_frequency: 30, // 30 minutes
            pattern_confidence_threshold: 0.85, // Higher confidence for production
            model_update_interval: 180, // 3 minutes
            enable_real_time_learning: true,
        };
        
        AdvancedMLEngine::new(config)
    }
    
    /// Create a high-frequency trading ML engine
    pub fn create_hft() -> AdvancedMLEngine {
        let config = MLConfig {
            sentiment_threshold: 0.6,  // Lower threshold for more signals
            prediction_horizon: 5,     // 5 minutes for HFT
            risk_tolerance: 0.4,       // Higher risk tolerance for HFT
            portfolio_rebalance_frequency: 15, // 15 minutes
            pattern_confidence_threshold: 0.7, // Lower threshold for more opportunities
            model_update_interval: 60, // 1 minute updates
            enable_real_time_learning: true,
        };
        
        AdvancedMLEngine::new(config)
    }
}

/// ML utilities and helper functions
pub mod utils {
    
    /// Calculate Sharpe ratio for performance metrics
    pub fn calculate_sharpe_ratio(returns: &[f64], risk_free_rate: f64) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }
        
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let excess_return = mean_return - risk_free_rate;
        
        if returns.len() < 2 {
            return excess_return;
        }
        
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / (returns.len() - 1) as f64;
        
        let std_dev = variance.sqrt();
        
        if std_dev == 0.0 {
            0.0
        } else {
            excess_return / std_dev
        }
    }
    
    /// Calculate maximum drawdown
    pub fn calculate_max_drawdown(equity_curve: &[f64]) -> f64 {
        if equity_curve.len() < 2 {
            return 0.0;
        }
        
        let mut max_drawdown = 0.0;
        let mut peak = equity_curve[0];
        
        for &value in equity_curve.iter().skip(1) {
            if value > peak {
                peak = value;
            } else {
                let drawdown = (peak - value) / peak;
                if drawdown > max_drawdown {
                    max_drawdown = drawdown;
                }
            }
        }
        
        max_drawdown
    }
    
    /// Normalize values to 0-1 range
    pub fn normalize(values: &[f64]) -> Vec<f64> {
        if values.is_empty() {
            return Vec::new();
        }
        
        let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if max_val == min_val {
            return vec![0.5; values.len()];
        }
        
        values.iter()
            .map(|&v| (v - min_val) / (max_val - min_val))
            .collect()
    }
    
    /// Calculate correlation coefficient between two series
    pub fn correlation(x: &[f64], y: &[f64]) -> f64 {
        if x.len() != y.len() || x.len() < 2 {
            return 0.0;
        }
        
        let n = x.len() as f64;
        let mean_x = x.iter().sum::<f64>() / n;
        let mean_y = y.iter().sum::<f64>() / n;
        
        let numerator = x.iter().zip(y.iter())
            .map(|(&xi, &yi)| (xi - mean_x) * (yi - mean_y))
            .sum::<f64>();
        
        let sum_sq_x = x.iter().map(|&xi| (xi - mean_x).powi(2)).sum::<f64>();
        let sum_sq_y = y.iter().map(|&yi| (yi - mean_y).powi(2)).sum::<f64>();
        
        let denominator = (sum_sq_x * sum_sq_y).sqrt();
        
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }
}

/// ML performance monitoring
pub mod monitoring {
    use std::collections::VecDeque;
    use chrono::{DateTime, Utc};
    
    /// Performance monitor for ML models
    pub struct MLPerformanceMonitor {
        predictions: VecDeque<PredictionRecord>,
        max_history: usize,
    }
    
    #[derive(Debug, Clone)]
    pub struct PredictionRecord {
        pub predicted_value: f64,
        pub actual_value: Option<f64>,
        pub confidence: f64,
        pub timestamp: DateTime<Utc>,
    }
    
    impl MLPerformanceMonitor {
        pub fn new(max_history: usize) -> Self {
            Self {
                predictions: VecDeque::new(),
                max_history,
            }
        }
        
        /// Add a new prediction
        pub fn add_prediction(&mut self, predicted: f64, confidence: f64) {
            let record = PredictionRecord {
                predicted_value: predicted,
                actual_value: None,
                confidence,
                timestamp: Utc::now(),
            };
            
            self.predictions.push_back(record);
            
            if self.predictions.len() > self.max_history {
                self.predictions.pop_front();
            }
        }
        
        /// Update prediction with actual value
        pub fn update_actual(&mut self, index: usize, actual: f64) {
            if let Some(record) = self.predictions.get_mut(index) {
                record.actual_value = Some(actual);
            }
        }
        
        /// Calculate model accuracy
        pub fn calculate_accuracy(&self, tolerance: f64) -> f64 {
            let completed_predictions: Vec<_> = self.predictions.iter()
                .filter(|p| p.actual_value.is_some())
                .collect();
            
            if completed_predictions.is_empty() {
                return 0.0;
            }
            
            let accurate_predictions = completed_predictions.iter()
                .filter(|p| {
                    let actual = p.actual_value.unwrap();
                    (p.predicted_value - actual).abs() <= tolerance
                })
                .count();
            
            accurate_predictions as f64 / completed_predictions.len() as f64
        }
        
        /// Calculate mean absolute error
        pub fn calculate_mae(&self) -> f64 {
            let completed_predictions: Vec<_> = self.predictions.iter()
                .filter(|p| p.actual_value.is_some())
                .collect();
            
            if completed_predictions.is_empty() {
                return 0.0;
            }
            
            let total_error: f64 = completed_predictions.iter()
                .map(|p| (p.predicted_value - p.actual_value.unwrap()).abs())
                .sum();
            
            total_error / completed_predictions.len() as f64
        }
        
        /// Calculate root mean square error
        pub fn calculate_rmse(&self) -> f64 {
            let completed_predictions: Vec<_> = self.predictions.iter()
                .filter(|p| p.actual_value.is_some())
                .collect();
            
            if completed_predictions.is_empty() {
                return 0.0;
            }
            
            let mse: f64 = completed_predictions.iter()
                .map(|p| (p.predicted_value - p.actual_value.unwrap()).powi(2))
                .sum::<f64>() / completed_predictions.len() as f64;
            
            mse.sqrt()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TradingOpportunity, MarketData, OpportunityType};
    use std::collections::HashMap;
    use chrono::Utc;
    use std::time::Duration;
    
    fn create_test_opportunity() -> TradingOpportunity {
        TradingOpportunity {
            opportunity_type: OpportunityType::Arbitrage,
            token_pair: "SOL/USDC".to_string(),
            profit_percentage: 2.5,
            volume_24h: 1_000_000.0,
            liquidity: 100_000.0,
            source_exchange: "Jupiter".to_string(),
            target_exchange: "Raydium".to_string(),
            entry_price: 150.0,
            exit_price: 153.75,
            risk_score: 0.3,
            confidence: 0.85,
            timestamp: Utc::now(),
            execution_window: Duration::from_secs(30),
            metadata: HashMap::new(),
        }
    }
    
    fn create_test_market_data() -> MarketData {
        let mut prices = HashMap::new();
        prices.insert("SOL".to_string(), 150.0);
        prices.insert("USDC".to_string(), 1.0);
        
        let mut volumes = HashMap::new();
        volumes.insert("SOL".to_string(), 1_000_000.0);
        volumes.insert("USDC".to_string(), 500_000.0);
        
        let mut liquidity = HashMap::new();
        liquidity.insert("SOL".to_string(), 100_000.0);
        liquidity.insert("USDC".to_string(), 500_000.0);
        
        MarketData {
            prices,
            volumes,
            liquidity,
            current_price: 150.0,
            volume_24h: 1_000_000.0,
            last_updated: Some(std::time::Instant::now()),
            bid_ask_spread: 0.001,
        }
    }
    
    #[tokio::test]
    async fn test_ml_engine_creation() {
        let _engine = MLEngineFactory::create_default();
        // Engine should be created successfully
        assert!(true); // Basic existence test
    }
    
    #[tokio::test]
    async fn test_ml_analysis() {
        let engine = MLEngineFactory::create_default();
        let opportunity = create_test_opportunity();
        let market_data = create_test_market_data();
        
        let result = engine.analyze_opportunity(&opportunity, &market_data).await;
        assert!(result.is_ok());
        
        let analysis = result.unwrap();
        assert!(analysis.ml_score >= 0.0 && analysis.ml_score <= 1.0);
        assert!(analysis.confidence >= 0.0 && analysis.confidence <= 1.0);
    }
    
    #[test]
    fn test_sharpe_ratio_calculation() {
        let returns = vec![0.1, 0.15, 0.08, 0.12, 0.05];
        let risk_free_rate = 0.02;
        let sharpe = utils::calculate_sharpe_ratio(&returns, risk_free_rate);
        assert!(sharpe > 0.0);
    }
    
    #[test]
    fn test_max_drawdown_calculation() {
        let equity_curve = vec![100.0, 110.0, 105.0, 120.0, 90.0, 95.0];
        let max_dd = utils::calculate_max_drawdown(&equity_curve);
        assert!(max_dd > 0.0 && max_dd < 1.0);
    }
    
    #[test]
    fn test_correlation_calculation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let corr = utils::correlation(&x, &y);
        assert!((corr - 1.0).abs() < 0.001); // Perfect positive correlation
    }
    
    #[test]
    fn test_normalization() {
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let normalized = utils::normalize(&values);
        assert_eq!(normalized[0], 0.0);
        assert_eq!(normalized[4], 1.0);
        assert!(normalized.iter().all(|&v| v >= 0.0 && v <= 1.0));
    }
    
    #[test]
    fn test_performance_monitor() {
        let mut monitor = monitoring::MLPerformanceMonitor::new(100);
        
        // Add some predictions
        monitor.add_prediction(10.0, 0.8);
        monitor.add_prediction(15.0, 0.9);
        monitor.add_prediction(12.0, 0.7);
        
        // Update with actual values
        monitor.update_actual(0, 9.5);  // Close to prediction
        monitor.update_actual(1, 16.0); // Close to prediction
        monitor.update_actual(2, 20.0); // Far from prediction
        
        let accuracy = monitor.calculate_accuracy(1.0); // 1.0 tolerance
        let mae = monitor.calculate_mae();
        let rmse = monitor.calculate_rmse();
        
        assert!(accuracy > 0.0);
        assert!(mae > 0.0);
        assert!(rmse > 0.0);
    }
}
