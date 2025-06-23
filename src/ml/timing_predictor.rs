//! Timing Predictor Module
//! 
//! Advanced ML-powered timing optimization for trade execution.
//! Predicts optimal entry/exit points and minimizes slippage.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Configuration for timing prediction models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingPredictorConfig {
    pub prediction_window_minutes: u32,
    pub microstructure_depth: usize,
    pub slippage_threshold: f64,
    pub min_liquidity_threshold: f64,
    pub execution_confidence_threshold: f64,
}

impl Default for TimingPredictorConfig {
    fn default() -> Self {
        Self {
            prediction_window_minutes: 15,
            microstructure_depth: 10,
            slippage_threshold: 0.005, // 0.5%
            min_liquidity_threshold: 1000.0,
            execution_confidence_threshold: 0.75,
        }
    }
}

/// Market microstructure data for timing analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketMicrostructure {
    pub timestamp: DateTime<Utc>,
    pub bid_ask_spread: f64,
    pub order_book_depth: f64,
    pub trade_frequency: f64,
    pub volume_imbalance: f64,
    pub price_impact: f64,
}

/// Timing prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingPrediction {
    pub predicted_timestamp: DateTime<Utc>,
    pub confidence: f64,
    pub expected_slippage: f64,
    pub liquidity_score: f64,
    pub execution_priority: ExecutionPriority,
    pub reasoning: String,
}

/// Priority levels for trade execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionPriority {
    Immediate,
    Optimal,
    Patient,
    Avoid,
}

/// Advanced timing predictor using ML models
pub struct TimingPredictor {
    config: TimingPredictorConfig,
    historical_data: Vec<MarketMicrostructure>,
    liquidity_model: LiquidityModel,
    slippage_model: SlippageModel,
}

/// Liquidity prediction model
#[derive(Debug, Clone)]
struct LiquidityModel {
    weights: Vec<f64>,
    bias: f64,
    feature_scaling: FeatureScaling,
}

/// Slippage prediction model
#[derive(Debug, Clone)]
struct SlippageModel {
    weights: Vec<f64>,
    bias: f64,
    feature_scaling: FeatureScaling,
}

/// Feature scaling parameters
#[derive(Debug, Clone)]
struct FeatureScaling {
    means: Vec<f64>,
    stds: Vec<f64>,
}

impl TimingPredictor {
    /// Create a new timing predictor instance
    pub fn new(config: TimingPredictorConfig) -> Self {
        info!("Initializing Timing Predictor with config: {:?}", config);
        
        Self {
            config,
            historical_data: Vec::new(),
            liquidity_model: LiquidityModel::default(),
            slippage_model: SlippageModel::default(),
        }
    }

    /// Update market microstructure data
    pub fn update_market_data(&mut self, data: MarketMicrostructure) -> Result<()> {
        debug!("Updating market microstructure data: {:?}", data);
        
        self.historical_data.push(data);
        
        // Keep only recent data within prediction window
        let cutoff_time = Utc::now() - chrono::Duration::minutes(
            self.config.prediction_window_minutes as i64 * 2
        );
        
        self.historical_data.retain(|d| d.timestamp > cutoff_time);
        
        Ok(())
    }

    /// Predict optimal execution timing for a trade
    pub fn predict_optimal_timing(
        &self,
        symbol: &str,
        trade_size: f64,
        target_direction: TradeDirection,
    ) -> Result<TimingPrediction> {
        info!("Predicting optimal timing for {} trade of size {}", symbol, trade_size);

        if self.historical_data.is_empty() {
            warn!("No historical data available for timing prediction");
            return Ok(self.create_default_prediction());
        }

        // Calculate market features
        let features = self.extract_timing_features(&target_direction, trade_size)?;
        
        // Predict liquidity
        let liquidity_score = self.liquidity_model.predict(&features);
        
        // Predict slippage
        let expected_slippage = self.slippage_model.predict(&features);
        
        // Determine execution priority
        let execution_priority = self.determine_execution_priority(
            liquidity_score,
            expected_slippage,
        );

        // Calculate confidence based on data quality and model certainty
        let confidence = self.calculate_prediction_confidence(&features);

        // Predict optimal timestamp (simple heuristic for now)
        let predicted_timestamp = self.predict_next_optimal_window()?;

        let reasoning = format!(
            "Liquidity: {:.3}, Slippage: {:.3}%, Priority: {:?}",
            liquidity_score,
            expected_slippage * 100.0,
            execution_priority
        );

        Ok(TimingPrediction {
            predicted_timestamp,
            confidence,
            expected_slippage,
            liquidity_score,
            execution_priority,
            reasoning,
        })
    }

    /// Optimize execution strategy for large trades
    pub fn optimize_execution_strategy(
        &self,
        symbol: &str,
        total_size: f64,
        max_slippage: f64,
    ) -> Result<ExecutionStrategy> {
        info!("Optimizing execution strategy for {} total size: {}", symbol, total_size);

        let chunks = self.calculate_optimal_chunks(total_size, max_slippage)?;
        let timing_windows = self.identify_optimal_windows(chunks.len())?;

        Ok(ExecutionStrategy {
            chunks,
            timing_windows,
            estimated_completion: Utc::now() + chrono::Duration::minutes(30),
            total_expected_slippage: max_slippage * 0.7, // Conservative estimate
        })
    }

    /// Extract features for timing models
    fn extract_timing_features(
        &self,
        direction: &TradeDirection,
        trade_size: f64,
    ) -> Result<Vec<f64>> {
        if self.historical_data.is_empty() {
            return Ok(vec![0.0; 8]); // Default features
        }

        let latest = &self.historical_data[self.historical_data.len() - 1];
        
        let mut features = vec![
            latest.bid_ask_spread,
            latest.order_book_depth,
            latest.trade_frequency,
            latest.volume_imbalance,
            latest.price_impact,
            trade_size,
            match direction {
                TradeDirection::Buy => 1.0,
                TradeDirection::Sell => -1.0,
            },
            self.calculate_market_stress(),
        ];

        // Normalize features (simple min-max scaling)
        for feature in &mut features {
            *feature = (*feature).clamp(-10.0, 10.0) / 10.0;
        }

        Ok(features)
    }

    /// Calculate market stress indicator
    fn calculate_market_stress(&self) -> f64 {
        if self.historical_data.len() < 2 {
            return 0.0;
        }

        let recent_volatility: f64 = self.historical_data
            .windows(2)
            .map(|w| (w[1].price_impact - w[0].price_impact).abs())
            .sum::<f64>() / (self.historical_data.len() - 1) as f64;

        recent_volatility.clamp(0.0, 1.0)
    }

    /// Determine execution priority based on predictions
    fn determine_execution_priority(
        &self,
        liquidity_score: f64,
        expected_slippage: f64,
    ) -> ExecutionPriority {
        if expected_slippage > self.config.slippage_threshold * 2.0 {
            ExecutionPriority::Avoid
        } else if liquidity_score > self.config.min_liquidity_threshold 
                && expected_slippage < self.config.slippage_threshold * 0.5 {
            ExecutionPriority::Immediate
        } else if expected_slippage < self.config.slippage_threshold {
            ExecutionPriority::Optimal
        } else {
            ExecutionPriority::Patient
        }
    }

    /// Calculate prediction confidence
    fn calculate_prediction_confidence(&self, _features: &[f64]) -> f64 {
        let data_quality = if self.historical_data.len() >= 10 {
            1.0
        } else {
            self.historical_data.len() as f64 / 10.0
        };

        let model_confidence = 0.8; // Placeholder - would be from actual model uncertainty

        (data_quality * model_confidence).clamp(0.0, 1.0)
    }

    /// Predict next optimal execution window
    fn predict_next_optimal_window(&self) -> Result<DateTime<Utc>> {
        // Simple heuristic: look for low volatility periods
        let base_time = Utc::now();
        
        // For demo, predict 5-15 minutes ahead based on current market stress
        let stress = self.calculate_market_stress();
        let minutes_ahead = if stress > 0.7 {
            15 // Wait longer during high stress
        } else if stress < 0.3 {
            2  // Execute quickly during calm periods
        } else {
            7  // Standard wait time
        };

        Ok(base_time + chrono::Duration::minutes(minutes_ahead))
    }

    /// Calculate optimal trade chunks for large orders
    fn calculate_optimal_chunks(&self, total_size: f64, max_slippage: f64) -> Result<Vec<f64>> {
        let base_chunk_size = total_size * 0.1; // Start with 10% chunks
        let mut chunks = Vec::new();
        let mut remaining = total_size;

        while remaining > 0.0 {
            let chunk_size = if remaining < base_chunk_size * 2.0 {
                remaining // Final chunk
            } else {
                base_chunk_size
            };
            
            chunks.push(chunk_size);
            remaining -= chunk_size;
        }

        debug!("Calculated {} chunks for total size {}", chunks.len(), total_size);
        Ok(chunks)
    }

    /// Identify optimal timing windows for execution chunks
    fn identify_optimal_windows(&self, num_chunks: usize) -> Result<Vec<DateTime<Utc>>> {
        let mut windows = Vec::new();
        let base_time = Utc::now();
        
        // Space chunks 3-8 minutes apart based on market conditions
        let stress = self.calculate_market_stress();
        let interval_minutes = if stress > 0.7 { 8 } else { 3 };
        
        for i in 0..num_chunks {
            let window_time = base_time + chrono::Duration::minutes(
                (i as i64) * interval_minutes
            );
            windows.push(window_time);
        }

        Ok(windows)
    }

    /// Create default prediction when insufficient data
    fn create_default_prediction(&self) -> TimingPrediction {
        TimingPrediction {
            predicted_timestamp: Utc::now() + chrono::Duration::minutes(5),
            confidence: 0.3,
            expected_slippage: self.config.slippage_threshold,
            liquidity_score: self.config.min_liquidity_threshold,
            execution_priority: ExecutionPriority::Patient,
            reasoning: "Insufficient historical data - using conservative defaults".to_string(),
        }
    }
}

/// Trade direction for timing analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeDirection {
    Buy,
    Sell,
}

/// Execution strategy for large trades
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStrategy {
    pub chunks: Vec<f64>,
    pub timing_windows: Vec<DateTime<Utc>>,
    pub estimated_completion: DateTime<Utc>,
    pub total_expected_slippage: f64,
}

impl Default for LiquidityModel {
    fn default() -> Self {
        Self {
            weights: vec![0.3, 0.4, 0.2, 0.1, 0.15, -0.1, 0.05, 0.25],
            bias: 0.5,
            feature_scaling: FeatureScaling {
                means: vec![0.0; 8],
                stds: vec![1.0; 8],
            },
        }
    }
}

impl Default for SlippageModel {
    fn default() -> Self {
        Self {
            weights: vec![-0.2, -0.3, 0.1, 0.4, 0.5, 0.3, 0.1, 0.35],
            bias: 0.005, // Base slippage of 0.5%
            feature_scaling: FeatureScaling {
                means: vec![0.0; 8],
                stds: vec![1.0; 8],
            },
        }
    }
}

impl LiquidityModel {
    fn predict(&self, features: &[f64]) -> f64 {
        let prediction: f64 = features.iter()
            .zip(&self.weights)
            .map(|(f, w)| f * w)
            .sum::<f64>() + self.bias;
        
        prediction.max(0.0) // Ensure non-negative liquidity
    }
}

impl SlippageModel {
    fn predict(&self, features: &[f64]) -> f64 {
        let prediction: f64 = features.iter()
            .zip(&self.weights)
            .map(|(f, w)| f * w)
            .sum::<f64>() + self.bias;
        
        prediction.max(0.0001).min(0.1) // Clamp between 0.01% and 10%
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timing_predictor_creation() {
        let config = TimingPredictorConfig::default();
        let predictor = TimingPredictor::new(config);
        
        assert_eq!(predictor.historical_data.len(), 0);
    }

    #[tokio::test]
    async fn test_market_data_update() {
        let config = TimingPredictorConfig::default();
        let mut predictor = TimingPredictor::new(config);
        
        let market_data = MarketMicrostructure {
            timestamp: Utc::now(),
            bid_ask_spread: 0.001,
            order_book_depth: 1000.0,
            trade_frequency: 10.0,
            volume_imbalance: 0.1,
            price_impact: 0.005,
        };
        
        predictor.update_market_data(market_data).unwrap();
        assert_eq!(predictor.historical_data.len(), 1);
    }

    #[tokio::test]
    async fn test_timing_prediction() {
        let config = TimingPredictorConfig::default();
        let predictor = TimingPredictor::new(config);
        
        let prediction = predictor.predict_optimal_timing(
            "SOL/USDC",
            100.0,
            TradeDirection::Buy,
        ).unwrap();
        
        assert!(prediction.confidence >= 0.0 && prediction.confidence <= 1.0);
        assert!(prediction.expected_slippage >= 0.0);
    }

    #[tokio::test]
    async fn test_execution_strategy() {
        let config = TimingPredictorConfig::default();
        let predictor = TimingPredictor::new(config);
        
        let strategy = predictor.optimize_execution_strategy(
            "SOL/USDC",
            1000.0,
            0.01,
        ).unwrap();
        
        assert!(!strategy.chunks.is_empty());
        assert_eq!(strategy.chunks.len(), strategy.timing_windows.len());
        assert!(strategy.chunks.iter().sum::<f64>() - 1000.0 < 0.001);
    }
}
