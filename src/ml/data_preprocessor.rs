//! Data Preprocessor Module
//! 
//! Handles data cleaning, feature engineering, and preparation for ML models.
//! Ensures high-quality input data for pattern recognition and optimization.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tracing::{debug, info, warn};

/// Configuration for data preprocessing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPreprocessorConfig {
    pub max_missing_ratio: f64,
    pub outlier_std_threshold: f64,
    pub smoothing_window: usize,
    pub feature_lag_periods: Vec<usize>,
    pub normalization_method: NormalizationMethod,
    pub handle_missing: MissingDataHandling,
}

impl Default for DataPreprocessorConfig {
    fn default() -> Self {
        Self {
            max_missing_ratio: 0.1, // Max 10% missing data
            outlier_std_threshold: 3.0, // 3 standard deviations
            smoothing_window: 5,
            feature_lag_periods: vec![1, 3, 5, 10, 20],
            normalization_method: NormalizationMethod::StandardScaling,
            handle_missing: MissingDataHandling::Interpolate,
        }
    }
}

/// Normalization methods for feature scaling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NormalizationMethod {
    StandardScaling, // (x - mean) / std
    MinMaxScaling,   // (x - min) / (max - min)
    RobustScaling,   // (x - median) / IQR
    None,
}

/// Methods for handling missing data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MissingDataHandling {
    Remove,      // Remove rows with missing data
    Interpolate, // Linear interpolation
    Forward,     // Forward fill
    Backward,    // Backward fill
    Mean,        // Fill with mean value
}

/// Raw market data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMarketData {
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub price: Option<f64>,
    pub volume: Option<f64>,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub trades: Option<u32>,
    pub market_cap: Option<f64>,
}

/// Processed feature vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedFeatures {
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub features: Vec<f64>,
    pub feature_names: Vec<String>,
    pub quality_score: f64,
}

/// Technical indicators calculated from raw data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalIndicators {
    pub sma_5: Option<f64>,
    pub sma_20: Option<f64>,
    pub ema_12: Option<f64>,
    pub ema_26: Option<f64>,
    pub rsi_14: Option<f64>,
    pub macd: Option<f64>,
    pub macd_signal: Option<f64>,
    pub bollinger_upper: Option<f64>,
    pub bollinger_lower: Option<f64>,
    pub volume_sma: Option<f64>,
    pub price_change_1: Option<f64>,
    pub price_change_5: Option<f64>,
    pub volume_ratio: Option<f64>,
}

/// Data quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQuality {
    pub completeness: f64,
    pub consistency: f64,
    pub outlier_ratio: f64,
    pub temporal_gaps: Vec<chrono::Duration>,
    pub overall_score: f64,
}

/// Advanced data preprocessor for ML features
pub struct DataPreprocessor {
    config: DataPreprocessorConfig,
    price_history: VecDeque<(DateTime<Utc>, f64)>,
    volume_history: VecDeque<(DateTime<Utc>, f64)>,
    feature_stats: HashMap<String, FeatureStatistics>,
    quality_tracker: DataQualityTracker,
}

/// Statistics for feature normalization
#[derive(Debug, Clone)]
struct FeatureStatistics {
    mean: f64,
    std: f64,
    min: f64,
    max: f64,
    median: f64,
    q25: f64,
    q75: f64,
    samples: usize,
}

/// Tracks data quality over time
#[derive(Debug, Clone)]
struct DataQualityTracker {
    recent_quality_scores: VecDeque<f64>,
    missing_data_counts: HashMap<String, usize>,
    outlier_counts: HashMap<String, usize>,
    total_samples: usize,
}

impl DataPreprocessor {
    /// Create a new data preprocessor
    pub fn new(config: DataPreprocessorConfig) -> Self {
        info!("Initializing Data Preprocessor with config: {:?}", config);
        
        Self {
            config,
            price_history: VecDeque::with_capacity(1000),
            volume_history: VecDeque::with_capacity(1000),
            feature_stats: HashMap::new(),
            quality_tracker: DataQualityTracker::new(),
        }
    }

    /// Process raw market data into ML-ready features
    pub fn process_market_data(
        &mut self,
        raw_data: &[RawMarketData],
    ) -> Result<Vec<ProcessedFeatures>> {
        info!("Processing {} raw market data points", raw_data.len());

        if raw_data.is_empty() {
            return Ok(Vec::new());
        }

        // Step 1: Clean and validate data
        let cleaned_data = self.clean_data(raw_data)?;
        
        // Step 2: Calculate technical indicators
        let indicators = self.calculate_technical_indicators(&cleaned_data)?;
        
        // Step 3: Engineer features
        let mut processed_features = Vec::new();
        
        for (data, indicator) in cleaned_data.iter().zip(indicators.iter()) {
            if let Some(features) = self.engineer_features(data, indicator)? {
                processed_features.push(features);
            }
        }

        // Step 4: Normalize features
        self.normalize_features(&mut processed_features)?;
        
        // Step 5: Update quality tracking
        self.update_quality_tracking(&processed_features);

        info!("Successfully processed {} feature vectors", processed_features.len());
        Ok(processed_features)
    }

    /// Clean raw data and handle missing values
    fn clean_data(&mut self, raw_data: &[RawMarketData]) -> Result<Vec<RawMarketData>> {
        let mut cleaned = Vec::new();

        for data in raw_data {
            let mut clean_data = data.clone();
            
            // Handle missing prices
            if clean_data.price.is_none() {
                match self.config.handle_missing {
                    MissingDataHandling::Remove => continue,
                    MissingDataHandling::Interpolate => {
                        clean_data.price = self.interpolate_price(&data.timestamp);
                    }
                    MissingDataHandling::Forward => {
                        clean_data.price = self.get_last_price();
                    }
                    _ => {
                        clean_data.price = Some(0.0); // Fallback
                    }
                }
            }

            // Detect and handle outliers
            if let Some(price) = clean_data.price {
                if self.is_outlier("price", price) {
                    warn!("Outlier detected in price: {} at {}", price, data.timestamp);
                    self.quality_tracker.outlier_counts
                        .entry("price".to_string())
                        .and_modify(|e| *e += 1)
                        .or_insert(1);
                    
                    // Replace outlier with interpolated value
                    clean_data.price = self.interpolate_price(&data.timestamp);
                }
            }

            // Update history
            if let Some(price) = clean_data.price {
                self.price_history.push_back((data.timestamp, price));
                if self.price_history.len() > 1000 {
                    self.price_history.pop_front();
                }
            }

            if let Some(volume) = clean_data.volume {
                self.volume_history.push_back((data.timestamp, volume));
                if self.volume_history.len() > 1000 {
                    self.volume_history.pop_front();
                }
            }

            cleaned.push(clean_data);
        }

        Ok(cleaned)
    }

    /// Calculate technical indicators from cleaned data
    fn calculate_technical_indicators(
        &self,
        data: &[RawMarketData],
    ) -> Result<Vec<TechnicalIndicators>> {
        let mut indicators = Vec::new();

        for (i, current) in data.iter().enumerate() {
            let price = current.price.unwrap_or(0.0);
            let volume = current.volume.unwrap_or(0.0);

            let mut indicator = TechnicalIndicators {
                sma_5: None,
                sma_20: None,
                ema_12: None,
                ema_26: None,
                rsi_14: None,
                macd: None,
                macd_signal: None,
                bollinger_upper: None,
                bollinger_lower: None,
                volume_sma: None,
                price_change_1: None,
                price_change_5: None,
                volume_ratio: None,
            };

            // Calculate SMAs
            if i >= 4 {
                let prices: Vec<f64> = data[(i-4)..=i]
                    .iter()
                    .filter_map(|d| d.price)
                    .collect();
                if prices.len() == 5 {
                    indicator.sma_5 = Some(prices.iter().sum::<f64>() / 5.0);
                }
            }

            if i >= 19 {
                let prices: Vec<f64> = data[(i-19)..=i]
                    .iter()
                    .filter_map(|d| d.price)
                    .collect();
                if prices.len() == 20 {
                    indicator.sma_20 = Some(prices.iter().sum::<f64>() / 20.0);
                }
            }

            // Calculate price changes
            if i >= 1 {
                if let Some(prev_price) = data[i-1].price {
                    indicator.price_change_1 = Some((price - prev_price) / prev_price);
                }
            }

            if i >= 5 {
                if let Some(prev_price) = data[i-5].price {
                    indicator.price_change_5 = Some((price - prev_price) / prev_price);
                }
            }

            // Calculate RSI (simplified version)
            if i >= 13 {
                indicator.rsi_14 = self.calculate_rsi(&data[(i-13)..=i]);
            }

            // Calculate volume ratio
            if i >= 4 {
                let volumes: Vec<f64> = data[(i-4)..=i]
                    .iter()
                    .filter_map(|d| d.volume)
                    .collect();
                if volumes.len() == 5 {
                    let avg_volume = volumes.iter().sum::<f64>() / 5.0;
                    if avg_volume > 0.0 {
                        indicator.volume_ratio = Some(volume / avg_volume);
                    }
                }
            }

            indicators.push(indicator);
        }

        Ok(indicators)
    }

    /// Engineer features from data and indicators
    fn engineer_features(
        &self,
        data: &RawMarketData,
        indicators: &TechnicalIndicators,
    ) -> Result<Option<ProcessedFeatures>> {
        let mut features = Vec::new();
        let mut feature_names = Vec::new();

        // Basic price features
        if let Some(price) = data.price {
            features.push(price);
            feature_names.push("price".to_string());
        }

        if let Some(volume) = data.volume {
            features.push(volume);
            feature_names.push("volume".to_string());
        }

        // Spread features
        if let (Some(bid), Some(ask)) = (data.bid, data.ask) {
            let spread = ask - bid;
            let mid_price = (bid + ask) / 2.0;
            features.push(spread);
            features.push(spread / mid_price); // Relative spread
            feature_names.push("spread".to_string());
            feature_names.push("relative_spread".to_string());
        }

        // Technical indicator features
        if let Some(sma_5) = indicators.sma_5 {
            features.push(sma_5);
            feature_names.push("sma_5".to_string());
        }

        if let Some(sma_20) = indicators.sma_20 {
            features.push(sma_20);
            feature_names.push("sma_20".to_string());
        }

        if let Some(rsi) = indicators.rsi_14 {
            features.push(rsi);
            feature_names.push("rsi_14".to_string());
        }

        if let Some(price_change_1) = indicators.price_change_1 {
            features.push(price_change_1);
            feature_names.push("price_change_1".to_string());
        }

        if let Some(price_change_5) = indicators.price_change_5 {
            features.push(price_change_5);
            feature_names.push("price_change_5".to_string());
        }

        if let Some(volume_ratio) = indicators.volume_ratio {
            features.push(volume_ratio);
            feature_names.push("volume_ratio".to_string());
        }

        // Time-based features
        let hour = data.timestamp.hour() as f64 / 24.0;
        let day_of_week = data.timestamp.weekday().number_from_monday() as f64 / 7.0;
        features.push(hour);
        features.push(day_of_week);
        feature_names.push("hour_of_day".to_string());
        feature_names.push("day_of_week".to_string());

        if features.is_empty() {
            return Ok(None);
        }

        // Calculate quality score
        let quality_score = self.calculate_feature_quality(&features);

        Ok(Some(ProcessedFeatures {
            timestamp: data.timestamp,
            symbol: data.symbol.clone(),
            features,
            feature_names,
            quality_score,
        }))
    }

    /// Normalize features using the configured method
    fn normalize_features(&mut self, features: &mut [ProcessedFeatures]) -> Result<()> {
        if features.is_empty() {
            return Ok(());
        }

        match self.config.normalization_method {
            NormalizationMethod::StandardScaling => {
                self.apply_standard_scaling(features)?;
            }
            NormalizationMethod::MinMaxScaling => {
                self.apply_minmax_scaling(features)?;
            }
            NormalizationMethod::RobustScaling => {
                self.apply_robust_scaling(features)?;
            }
            NormalizationMethod::None => {
                // No normalization
            }
        }

        Ok(())
    }

    /// Apply standard scaling (z-score normalization)
    fn apply_standard_scaling(&mut self, features: &mut [ProcessedFeatures]) -> Result<()> {
        if features.is_empty() {
            return Ok(());
        }

        let num_features = features[0].features.len();
        
        for feature_idx in 0..num_features {
            let values: Vec<f64> = features
                .iter()
                .map(|f| f.features[feature_idx])
                .collect();

            if values.is_empty() {
                continue;
            }

            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let variance = values.iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>() / values.len() as f64;
            let std = variance.sqrt();

            // Update statistics
            if feature_idx < features[0].feature_names.len() {
                let feature_name = &features[0].feature_names[feature_idx];
                self.feature_stats.insert(
                    feature_name.clone(),
                    FeatureStatistics {
                        mean,
                        std,
                        min: values.iter().cloned().fold(f64::INFINITY, f64::min),
                        max: values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
                        median: self.calculate_median(&values),
                        q25: self.calculate_percentile(&values, 0.25),
                        q75: self.calculate_percentile(&values, 0.75),
                        samples: values.len(),
                    },
                );
            }

            // Apply normalization
            if std > 1e-10 { // Avoid division by zero
                for feature_set in features.iter_mut() {
                    feature_set.features[feature_idx] = 
                        (feature_set.features[feature_idx] - mean) / std;
                }
            }
        }

        Ok(())
    }

    /// Apply min-max scaling
    fn apply_minmax_scaling(&mut self, features: &mut [ProcessedFeatures]) -> Result<()> {
        if features.is_empty() {
            return Ok(());
        }

        let num_features = features[0].features.len();
        
        for feature_idx in 0..num_features {
            let values: Vec<f64> = features
                .iter()
                .map(|f| f.features[feature_idx])
                .collect();

            if values.is_empty() {
                continue;
            }

            let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let range = max_val - min_val;

            if range > 1e-10 { // Avoid division by zero
                for feature_set in features.iter_mut() {
                    feature_set.features[feature_idx] = 
                        (feature_set.features[feature_idx] - min_val) / range;
                }
            }
        }

        Ok(())
    }

    /// Apply robust scaling (using median and IQR)
    fn apply_robust_scaling(&mut self, features: &mut [ProcessedFeatures]) -> Result<()> {
        if features.is_empty() {
            return Ok(());
        }

        let num_features = features[0].features.len();
        
        for feature_idx in 0..num_features {
            let values: Vec<f64> = features
                .iter()
                .map(|f| f.features[feature_idx])
                .collect();

            if values.is_empty() {
                continue;
            }

            let median = self.calculate_median(&values);
            let q25 = self.calculate_percentile(&values, 0.25);
            let q75 = self.calculate_percentile(&values, 0.75);
            let iqr = q75 - q25;

            if iqr > 1e-10 { // Avoid division by zero
                for feature_set in features.iter_mut() {
                    feature_set.features[feature_idx] = 
                        (feature_set.features[feature_idx] - median) / iqr;
                }
            }
        }

        Ok(())
    }

    /// Calculate RSI indicator
    fn calculate_rsi(&self, data: &[RawMarketData]) -> Option<f64> {
        if data.len() < 14 {
            return None;
        }

        let mut gains = Vec::new();
        let mut losses = Vec::new();

        for i in 1..data.len() {
            if let (Some(current), Some(previous)) = (data[i].price, data[i-1].price) {
                let change = current - previous;
                if change > 0.0 {
                    gains.push(change);
                    losses.push(0.0);
                } else {
                    gains.push(0.0);
                    losses.push(-change);
                }
            }
        }

        if gains.is_empty() {
            return None;
        }

        let avg_gain = gains.iter().sum::<f64>() / gains.len() as f64;
        let avg_loss = losses.iter().sum::<f64>() / losses.len() as f64;

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }

    /// Check if a value is an outlier
    fn is_outlier(&self, feature_name: &str, value: f64) -> bool {
        if let Some(stats) = self.feature_stats.get(feature_name) {
            let z_score = (value - stats.mean) / stats.std;
            z_score.abs() > self.config.outlier_std_threshold
        } else {
            false // Can't determine without historical stats
        }
    }

    /// Interpolate missing price using linear interpolation
    fn interpolate_price(&self, timestamp: &DateTime<Utc>) -> Option<f64> {
        if self.price_history.len() < 2 {
            return None;
        }

        // Find surrounding points
        let mut before = None;
        let mut after = None;

        for (ts, price) in &self.price_history {
            if ts < timestamp {
                before = Some((*ts, *price));
            } else if ts > timestamp && after.is_none() {
                after = Some((*ts, *price));
                break;
            }
        }

        match (before, after) {
            (Some((t1, p1)), Some((t2, p2))) => {
                let time_diff = (t2 - t1).num_seconds() as f64;
                let target_diff = (*timestamp - t1).num_seconds() as f64;
                let ratio = target_diff / time_diff;
                Some(p1 + (p2 - p1) * ratio)
            }
            _ => self.get_last_price(),
        }
    }

    /// Get the most recent price
    fn get_last_price(&self) -> Option<f64> {
        self.price_history.back().map(|(_, price)| *price)
    }

    /// Calculate feature quality score
    fn calculate_feature_quality(&self, features: &[f64]) -> f64 {
        let completeness = features.iter()
            .filter(|f| f.is_finite())
            .count() as f64 / features.len() as f64;
        
        let consistency = if features.len() > 1 {
            let mean = features.iter().sum::<f64>() / features.len() as f64;
            let variance = features.iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>() / features.len() as f64;
            
            // Lower variance indicates more consistency
            (1.0 / (1.0 + variance)).clamp(0.0, 1.0)
        } else {
            1.0
        };

        (completeness + consistency) / 2.0
    }

    /// Calculate median of values
    fn calculate_median(&self, values: &[f64]) -> f64 {
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let len = sorted.len();
        if len % 2 == 0 {
            (sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
        } else {
            sorted[len / 2]
        }
    }

    /// Calculate percentile of values
    fn calculate_percentile(&self, values: &[f64], percentile: f64) -> f64 {
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let index = (percentile * (sorted.len() - 1) as f64).round() as usize;
        sorted[index.min(sorted.len() - 1)]
    }

    /// Update quality tracking
    fn update_quality_tracking(&mut self, features: &[ProcessedFeatures]) {
        for feature_set in features {
            self.quality_tracker.recent_quality_scores
                .push_back(feature_set.quality_score);
            
            if self.quality_tracker.recent_quality_scores.len() > 100 {
                self.quality_tracker.recent_quality_scores.pop_front();
            }
        }
        
        self.quality_tracker.total_samples += features.len();
    }

    /// Get current data quality metrics
    pub fn get_data_quality(&self) -> DataQuality {
        let completeness = if self.quality_tracker.total_samples > 0 {
            self.quality_tracker.recent_quality_scores
                .iter()
                .sum::<f64>() / self.quality_tracker.recent_quality_scores.len() as f64
        } else {
            0.0
        };

        let consistency = 0.85; // Placeholder - would calculate from actual consistency metrics
        
        let outlier_ratio = if self.quality_tracker.total_samples > 0 {
            let total_outliers: usize = self.quality_tracker.outlier_counts
                .values()
                .sum();
            total_outliers as f64 / self.quality_tracker.total_samples as f64
        } else {
            0.0
        };

        let overall_score = (completeness + consistency + (1.0 - outlier_ratio)) / 3.0;

        DataQuality {
            completeness,
            consistency,
            outlier_ratio,
            temporal_gaps: Vec::new(), // Would track actual gaps
            overall_score,
        }
    }
}

impl DataQualityTracker {
    fn new() -> Self {
        Self {
            recent_quality_scores: VecDeque::with_capacity(100),
            missing_data_counts: HashMap::new(),
            outlier_counts: HashMap::new(),
            total_samples: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_preprocessor_creation() {
        let config = DataPreprocessorConfig::default();
        let preprocessor = DataPreprocessor::new(config);
        
        assert_eq!(preprocessor.price_history.len(), 0);
        assert_eq!(preprocessor.feature_stats.len(), 0);
    }

    #[test]
    fn test_process_market_data() {
        let config = DataPreprocessorConfig::default();
        let mut preprocessor = DataPreprocessor::new(config);
        
        let raw_data = vec![
            RawMarketData {
                timestamp: Utc::now(),
                symbol: "SOL/USDC".to_string(),
                price: Some(100.0),
                volume: Some(1000.0),
                bid: Some(99.5),
                ask: Some(100.5),
                trades: Some(50),
                market_cap: Some(1_000_000.0),
            },
        ];
        
        let result = preprocessor.process_market_data(&raw_data).unwrap();
        assert!(!result.is_empty());
        assert!(!result[0].features.is_empty());
    }

    #[test]
    fn test_feature_engineering() {
        let config = DataPreprocessorConfig::default();
        let preprocessor = DataPreprocessor::new(config);
        
        let data = RawMarketData {
            timestamp: Utc::now(),
            symbol: "SOL/USDC".to_string(),
            price: Some(100.0),
            volume: Some(1000.0),
            bid: Some(99.5),
            ask: Some(100.5),
            trades: Some(50),
            market_cap: Some(1_000_000.0),
        };

        let indicators = TechnicalIndicators {
            sma_5: Some(98.0),
            sma_20: Some(95.0),
            ema_12: None,
            ema_26: None,
            rsi_14: Some(65.0),
            macd: None,
            macd_signal: None,
            bollinger_upper: None,
            bollinger_lower: None,
            volume_sma: None,
            price_change_1: Some(0.02),
            price_change_5: Some(0.05),
            volume_ratio: Some(1.2),
        };
        
        let result = preprocessor.engineer_features(&data, &indicators).unwrap();
        assert!(result.is_some());
        
        let features = result.unwrap();
        assert!(!features.features.is_empty());
        assert_eq!(features.features.len(), features.feature_names.len());
    }
}
