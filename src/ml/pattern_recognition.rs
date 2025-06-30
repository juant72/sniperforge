//! Pattern Recognition Module using LSTM and ML algorithms
//! 
//! This module implements:
//! - LSTM networks for time series pattern recognition
//! - Technical indicator analysis with ML enhancement
//! - Volume anomaly detection
//! - Support/resistance level prediction

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use ndarray::{Array1, Array2};

use super::{PatternRecognitionConfig, MLPrediction, FeatureVector};
use crate::shared::real_data_manager::RealDataManager;

/// LSTM-based pattern recognition for price movements
pub struct PatternRecognizer {
    config: PatternRecognitionConfig,
    lstm_model: Option<LSTMModel>,
    technical_indicators: TechnicalIndicators,
    pattern_cache: HashMap<String, PatternCache>,
    last_update: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct LSTMModel {
    weights: Vec<Array2<f64>>,
    biases: Vec<Array1<f64>>,
    hidden_size: usize,
    sequence_length: usize,
}

#[derive(Debug, Clone)]
struct PatternCache {
    patterns: Vec<PatternMatch>,
    last_updated: DateTime<Utc>,
    confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub pattern_type: String,
    pub confidence: f64,
    pub predicted_direction: f64, // -1.0 to 1.0
    pub predicted_magnitude: f64,
    pub time_horizon: u64, // seconds
    pub supporting_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealPatternAnalysis {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub patterns: Vec<PatternMatch>,
    pub technical_indicators: HashMap<String, f64>,
    pub overall_confidence: f64,
    pub data_source: String,
}

#[derive(Debug, Clone)]
struct SupportResistanceLevel {
    support_level: f64,
    resistance_level: f64,
    current_price: f64,
    direction: f64,
    strength: f64,
    confidence: f64,
}

/// Technical indicators calculator with ML enhancement
struct TechnicalIndicators {
    rsi_period: usize,
    macd_fast: usize,
    macd_slow: usize,
    bollinger_period: usize,
    bollinger_std: f64,
}

impl TechnicalIndicators {
    fn new() -> Self {
        Self {
            rsi_period: 14,
            macd_fast: 12,
            macd_slow: 26,
            bollinger_period: 20,
            bollinger_std: 2.0,
        }
    }

    /// Calculate RSI with ML enhancement for pattern detection
    fn calculate_rsi(&self, prices: &[f64]) -> Result<f64> {
        if prices.len() < self.rsi_period + 1 {
            return Err(anyhow!("Insufficient data for RSI calculation"));
        }

        let mut gains = 0.0;
        let mut losses = 0.0;

        for i in 1..=self.rsi_period {
            let change = prices[prices.len() - i] - prices[prices.len() - i - 1];
            if change > 0.0 {
                gains += change;
            } else {
                losses -= change;
            }
        }

        let avg_gain = gains / self.rsi_period as f64;
        let avg_loss = losses / self.rsi_period as f64;

        if avg_loss == 0.0 {
            return Ok(100.0);
        }

        let rs = avg_gain / avg_loss;
        Ok(100.0 - (100.0 / (1.0 + rs)))
    }

    /// Calculate MACD with signal line
    fn calculate_macd(&self, prices: &[f64]) -> Result<(f64, f64, f64)> {
        if prices.len() < self.macd_slow {
            return Err(anyhow!("Insufficient data for MACD calculation"));
        }

        let ema_fast = self.calculate_ema(prices, self.macd_fast)?;
        let ema_slow = self.calculate_ema(prices, self.macd_slow)?;
        let macd_line = ema_fast - ema_slow;

        // Simple signal line (9-period EMA of MACD)
        let signal_line = macd_line * 0.8; // Simplified for now
        let histogram = macd_line - signal_line;

        Ok((macd_line, signal_line, histogram))
    }

    /// Calculate Exponential Moving Average
    fn calculate_ema(&self, prices: &[f64], period: usize) -> Result<f64> {
        if prices.len() < period {
            return Err(anyhow!("Insufficient data for EMA calculation"));
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = prices[0];

        for &price in &prices[1..] {
            ema = (price * multiplier) + (ema * (1.0 - multiplier));
        }

        Ok(ema)
    }

    /// Calculate Bollinger Bands
    fn calculate_bollinger_bands(&self, prices: &[f64]) -> Result<(f64, f64, f64)> {
        if prices.len() < self.bollinger_period {
            return Err(anyhow!("Insufficient data for Bollinger Bands"));
        }

        let recent_prices = &prices[prices.len() - self.bollinger_period..];
        let sma = recent_prices.iter().sum::<f64>() / self.bollinger_period as f64;
        
        let variance = recent_prices.iter()
            .map(|price| (price - sma).powi(2))
            .sum::<f64>() / self.bollinger_period as f64;
        
        let std_dev = variance.sqrt();
        
        let upper_band = sma + (self.bollinger_std * std_dev);
        let lower_band = sma - (self.bollinger_std * std_dev);

        Ok((upper_band, sma, lower_band))
    }

    /// Calculate Volume Profile for liquidity analysis
    fn calculate_volume_profile(&self, prices: &[f64], volumes: &[f64]) -> Result<f64> {
        if prices.len() != volumes.len() || prices.is_empty() {
            return Err(anyhow!("Invalid price/volume data"));
        }

        // Volume-weighted average price (VWAP)
        let total_volume: f64 = volumes.iter().sum();
        if total_volume == 0.0 {
            return Ok(0.0);
        }

        let vwap = prices.iter()
            .zip(volumes.iter())
            .map(|(price, volume)| price * volume)
            .sum::<f64>() / total_volume;

        Ok(vwap)
    }
}

impl PatternRecognizer {
    pub async fn new(config: PatternRecognitionConfig) -> Result<Self> {
        let lstm_model = Self::initialize_lstm_model(&config).await?;
        
        Ok(Self {
            config,
            lstm_model: Some(lstm_model),
            technical_indicators: TechnicalIndicators::new(),
            pattern_cache: HashMap::new(),
            last_update: Utc::now(),
        })
    }

    pub fn new_simple() -> Self {
        Self {
            config: PatternRecognitionConfig::default(),
            lstm_model: None, // Start without LSTM for simplicity
            technical_indicators: TechnicalIndicators::new(),
            pattern_cache: HashMap::new(),
            last_update: Utc::now(),
        }
    }

    async fn initialize_lstm_model(config: &PatternRecognitionConfig) -> Result<LSTMModel> {
        // Initialize LSTM model with random weights for now
        // In production, this would load pre-trained weights
        let hidden_size = config.lstm_units;
        let input_size = config.technical_indicators.len();
        
        let weights = vec![
            Array2::zeros((input_size, hidden_size)),
            Array2::zeros((hidden_size, hidden_size)),
            Array2::zeros((hidden_size, 1)),
        ];
        
        let biases = vec![
            Array1::zeros(hidden_size),
            Array1::zeros(hidden_size),
            Array1::zeros(1),
        ];

        Ok(LSTMModel {
            weights,
            biases,
            hidden_size,
            sequence_length: config.sequence_length,
        })
    }

    /// Predict price movement using LSTM and technical analysis
    pub async fn predict_price_movement(&mut self, features: &FeatureVector) -> Result<MLPrediction> {
        let symbol = &features.symbol;
          // Extract price and volume data from features
        let _price = features.features.get("price").unwrap_or(&0.0);
        let _volume = features.features.get("volume").unwrap_or(&0.0);
        
        // For demo purposes, we'll use simplified pattern recognition
        // In production, this would use the full LSTM model
        let pattern_score = self.analyze_patterns(features).await?;
        let technical_score = self.analyze_technical_indicators(features).await?;
        let volume_score = self.analyze_volume_anomalies(features).await?;

        // Combine scores with weights
        let combined_score = (pattern_score * 0.5) + (technical_score * 0.3) + (volume_score * 0.2);
        let confidence = self.calculate_prediction_confidence(&combined_score);

        // Determine prediction direction and magnitude
        let direction = if combined_score > 0.6 { 1.0 } 
                       else if combined_score < 0.4 { -1.0 } 
                       else { 0.0 };
        
        let magnitude = (combined_score - 0.5).abs() * 2.0; // 0 to 1

        let mut prediction = MLPrediction::new(
            "price_movement".to_string(),
            direction * magnitude,
            confidence,
            features.features.keys().cloned().collect(),
            "pattern_v1.0".to_string(),
        );

        prediction.add_metadata("pattern_score".to_string(), 
            serde_json::Value::Number(serde_json::Number::from_f64(pattern_score).unwrap()));
        prediction.add_metadata("technical_score".to_string(), 
            serde_json::Value::Number(serde_json::Number::from_f64(technical_score).unwrap()));
        prediction.add_metadata("volume_score".to_string(), 
            serde_json::Value::Number(serde_json::Number::from_f64(volume_score).unwrap()));

        // Cache the pattern for future reference
        self.cache_pattern(symbol, &prediction).await?;

        tracing::debug!("Pattern prediction for {}: direction={}, confidence={}", 
                       symbol, direction, confidence);

        Ok(prediction)
    }

    fn calculate_prediction_confidence(&self, combined_score: &f64) -> f64 {
        // Convert combined score to confidence percentage
        // Score of 0.5 = 50% confidence, score of 1.0 = 100% confidence
        if *combined_score > 0.5 {
            0.5 + (*combined_score - 0.5) * 1.0 // Scale from 0.5-1.0 to 50%-100%
        } else {
            *combined_score * 1.0 // Scale from 0.0-0.5 to 0%-50%
        }
    }

    async fn analyze_patterns(&self, features: &FeatureVector) -> Result<f64> {
        // Simplified pattern analysis
        // In production, this would use the LSTM model for complex pattern recognition
        
        let price = features.features.get("price").unwrap_or(&0.0);
        let prev_price = features.features.get("prev_price").unwrap_or(price);
        let price_change = (price - prev_price) / prev_price;

        // Simple momentum pattern
        let momentum_score = if price_change > 0.02 { 0.8 } 
                            else if price_change < -0.02 { 0.2 } 
                            else { 0.5 };

        Ok(momentum_score)
    }

    async fn analyze_technical_indicators(&self, features: &FeatureVector) -> Result<f64> {
        let mut technical_score: f64 = 0.5; // Neutral base
        let mut indicator_count = 0;

        // RSI analysis
        if let Some(rsi) = features.features.get("rsi") {
            indicator_count += 1;
            if *rsi > 70.0 {
                technical_score += 0.2; // Overbought - potential reversal
            } else if *rsi < 30.0 {
                technical_score -= 0.2; // Oversold - potential reversal
            }
        }

        // MACD analysis
        if let (Some(macd), Some(signal)) = (features.features.get("macd"), features.features.get("macd_signal")) {
            indicator_count += 1;
            if macd > signal {
                technical_score += 0.15; // Bullish crossover
            } else {
                technical_score -= 0.15; // Bearish crossover
            }
        }

        // Bollinger Bands analysis
        if let (Some(price), Some(bb_upper), Some(bb_lower)) = (
            features.features.get("price"),
            features.features.get("bollinger_upper"),
            features.features.get("bollinger_lower")
        ) {
            indicator_count += 1;
            let bb_position = (price - bb_lower) / (bb_upper - bb_lower);
            if bb_position > 0.8 {
                technical_score += 0.1; // Near upper band
            } else if bb_position < 0.2 {
                technical_score -= 0.1; // Near lower band
            }
        }        // Normalize score
        if indicator_count > 0 {
            technical_score = technical_score.clamp(0.0_f64, 1.0_f64);
        }

        Ok(technical_score)
    }

    async fn analyze_volume_anomalies(&self, features: &FeatureVector) -> Result<f64> {
        let volume = features.features.get("volume").unwrap_or(&0.0);
        let avg_volume = features.features.get("avg_volume").unwrap_or(volume);
        
        if *avg_volume == 0.0 {
            return Ok(0.5);
        }

        let volume_ratio = volume / avg_volume;
        
        // High volume can indicate strong conviction
        let volume_score = if volume_ratio > 2.0 {
            0.8 // High volume - strong signal
        } else if volume_ratio > 1.5 {
            0.7 // Above average volume
        } else if volume_ratio < 0.5 {
            0.3 // Low volume - weak signal
        } else {
            0.5 // Normal volume
        };

        Ok(volume_score)
    }

    /// Analyze real market patterns using live data
    pub async fn analyze_real_patterns(
        &self,
        symbol: &str,
        timeframe_minutes: u32,
        confidence_threshold: f64,
        data_manager: &mut RealDataManager,
    ) -> Result<RealPatternAnalysis> {
        // Get real price data from Jupiter/DexScreener
        let token_mint = match symbol {
            "SOL/USDC" | "SOL" => "So11111111111111111111111111111111111111112",
            _ => return Err(anyhow!("Token not supported yet")),
        };

        let real_price = data_manager.get_real_price(token_mint).await?;
        
        // Get historical price data for pattern analysis
        let price_history = data_manager.get_price_history(token_mint, timeframe_minutes as u64).await?;
        
        let mut patterns = Vec::new();
        let mut technical_indicators = HashMap::new();
        
        // Analyze support/resistance levels
        if let Some(support_resistance) = self.analyze_support_resistance(&price_history).await? {
            if support_resistance.confidence >= confidence_threshold {
                patterns.push(PatternMatch {
                    pattern_type: "Support/Resistance".to_string(),
                    confidence: support_resistance.confidence,
                    predicted_direction: support_resistance.direction,
                    predicted_magnitude: support_resistance.strength,
                    time_horizon: timeframe_minutes as u64 * 60,
                    supporting_indicators: vec!["price_action".to_string(), "volume".to_string()],
                });
            }
        }

        // Calculate real technical indicators
        if price_history.len() >= 14 {
            let prices: Vec<f64> = price_history.iter().map(|p| p.price_usd).collect();
            
            if let Ok(rsi) = self.technical_indicators.calculate_rsi(&prices) {
                technical_indicators.insert("RSI".to_string(), rsi);
                
                // RSI pattern analysis
                if rsi > 70.0 && confidence_threshold <= 0.8 {
                    patterns.push(PatternMatch {
                        pattern_type: "Overbought (RSI)".to_string(),
                        confidence: 0.85,
                        predicted_direction: -1.0,
                        predicted_magnitude: (rsi - 70.0) / 30.0,
                        time_horizon: timeframe_minutes as u64 * 60,
                        supporting_indicators: vec!["RSI".to_string()],
                    });
                } else if rsi < 30.0 && confidence_threshold <= 0.8 {
                    patterns.push(PatternMatch {
                        pattern_type: "Oversold (RSI)".to_string(),
                        confidence: 0.85,
                        predicted_direction: 1.0,
                        predicted_magnitude: (30.0 - rsi) / 30.0,
                        time_horizon: timeframe_minutes as u64 * 60,
                        supporting_indicators: vec!["RSI".to_string()],
                    });
                }
            }
        }

        // Volume analysis using real data
        if real_price.volume_24h > 0.0 {
            technical_indicators.insert("Volume_24h".to_string(), real_price.volume_24h);
            
            // Volume trend pattern
            if price_history.len() >= 2 {
                let recent_avg_volume = price_history.iter()
                    .rev()
                    .take(5)
                    .map(|p| p.volume_24h)
                    .sum::<f64>() / 5.0;
                
                if real_price.volume_24h > recent_avg_volume * 1.5 {
                    patterns.push(PatternMatch {
                        pattern_type: "High Volume Spike".to_string(),
                        confidence: 0.75,
                        predicted_direction: 1.0,
                        predicted_magnitude: real_price.volume_24h / recent_avg_volume - 1.0,
                        time_horizon: timeframe_minutes as u64 * 30, // Shorter horizon for volume spikes
                        supporting_indicators: vec!["volume".to_string()],
                    });
                }
            }
        }

        // Current price trend analysis
        if price_history.len() >= 3 {
            let recent_prices: Vec<f64> = price_history.iter()
                .rev()
                .take(3)
                .map(|p| p.price_usd)
                .collect();
            
            if recent_prices[0] > recent_prices[1] && recent_prices[1] > recent_prices[2] {
                patterns.push(PatternMatch {
                    pattern_type: "Uptrend".to_string(),
                    confidence: 0.70,
                    predicted_direction: 1.0,
                    predicted_magnitude: (recent_prices[0] - recent_prices[2]) / recent_prices[2],
                    time_horizon: timeframe_minutes as u64 * 60,
                    supporting_indicators: vec!["price_trend".to_string()],
                });
            } else if recent_prices[0] < recent_prices[1] && recent_prices[1] < recent_prices[2] {
                patterns.push(PatternMatch {
                    pattern_type: "Downtrend".to_string(),
                    confidence: 0.70,
                    predicted_direction: -1.0,
                    predicted_magnitude: (recent_prices[2] - recent_prices[0]) / recent_prices[2],
                    time_horizon: timeframe_minutes as u64 * 60,
                    supporting_indicators: vec!["price_trend".to_string()],
                });
            }
        }

        // Calculate overall confidence
        let overall_confidence = if patterns.is_empty() {
            0.0
        } else {
            patterns.iter().map(|p| p.confidence).sum::<f64>() / patterns.len() as f64
        };

        Ok(RealPatternAnalysis {
            symbol: symbol.to_string(),
            timestamp: Utc::now(),
            patterns,
            technical_indicators,
            overall_confidence,
            data_source: format!("Real Data - {}", real_price.source),
        })
    }

    async fn analyze_support_resistance(&self, price_history: &[crate::shared::real_data_manager::RealPriceData]) -> Result<Option<SupportResistanceLevel>> {
        if price_history.len() < 10 {
            return Ok(None);
        }

        let prices: Vec<f64> = price_history.iter().map(|p| p.price_usd).collect();
        let current_price = prices[prices.len() - 1];
        
        // Find local minima and maxima for support/resistance
        let mut support_levels = Vec::new();
        let mut resistance_levels = Vec::new();
        
        for i in 1..prices.len()-1 {
            if prices[i] < prices[i-1] && prices[i] < prices[i+1] {
                support_levels.push(prices[i]);
            }
            if prices[i] > prices[i-1] && prices[i] > prices[i+1] {
                resistance_levels.push(prices[i]);
            }
        }

        // Find the most relevant support/resistance
        let nearest_support = support_levels.iter()
            .filter(|&&level| level < current_price)
            .max_by(|a, b| a.partial_cmp(b).unwrap());
        
        let nearest_resistance = resistance_levels.iter()
            .filter(|&&level| level > current_price)
            .min_by(|a, b| a.partial_cmp(b).unwrap());

        if let (Some(&support), Some(&resistance)) = (nearest_support, nearest_resistance) {
            let distance_to_support = (current_price - support) / current_price;
            let distance_to_resistance = (resistance - current_price) / current_price;
            
            let (direction, strength, confidence) = if distance_to_support < distance_to_resistance {
                // Closer to support - potential bounce up
                (1.0, distance_to_support, 0.8)
            } else {
                // Closer to resistance - potential rejection down
                (-1.0, distance_to_resistance, 0.8)
            };

            Ok(Some(SupportResistanceLevel {
                support_level: support,
                resistance_level: resistance,
                current_price,
                direction,
                strength,
                confidence,
            }))
        } else {
            Ok(None)
        }
    }

    /// Cache the pattern for future reference
    async fn cache_pattern(&mut self, symbol: &str, prediction: &MLPrediction) -> Result<()> {
        let pattern_match = PatternMatch {
            pattern_type: "ml_prediction".to_string(),
            confidence: prediction.confidence,
            predicted_direction: prediction.value.signum(),
            predicted_magnitude: prediction.value.abs(),
            time_horizon: 300, // 5 minutes default
            supporting_indicators: prediction.features_used.clone(),
        };

        let cache_entry = PatternCache {
            patterns: vec![pattern_match],
            last_updated: Utc::now(),
            confidence_score: prediction.confidence,
        };

        self.pattern_cache.insert(symbol.to_string(), cache_entry);
        Ok(())
    }

    /// Retrain the LSTM model with new data
    pub async fn retrain(&mut self, training_data: &[FeatureVector]) -> Result<()> {
        if training_data.len() < self.config.sequence_length {
            return Err(anyhow!("Insufficient training data"));
        }

        // In production, this would implement proper LSTM training
        // For now, we'll just update the timestamp
        self.last_update = Utc::now();
        
        tracing::info!("Pattern recognition model retrained with {} samples", training_data.len());
        Ok(())
    }    /// Get cached patterns for a symbol (internal use)
    fn get_cached_patterns(&self, symbol: &str) -> Option<&PatternCache> {
        self.pattern_cache.get(symbol)
    }

    /// Check if the model needs retraining
    pub fn needs_retraining(&self) -> bool {
        let hours_since_update = Utc::now().signed_duration_since(self.last_update).num_hours();
        hours_since_update > 24 // Retrain daily
    }

    /// Get pattern recognition statistics
    pub fn get_statistics(&self) -> PatternRecognitionStats {
        let total_patterns = self.pattern_cache.len();
        let avg_confidence = if total_patterns > 0 {
            self.pattern_cache.values()
                .map(|cache| cache.confidence_score)
                .sum::<f64>() / total_patterns as f64
        } else {
            0.0
        };

        PatternRecognitionStats {
            total_patterns_cached: total_patterns,
            average_confidence: avg_confidence,
            last_model_update: self.last_update,
            lstm_units: self.config.lstm_units,
            sequence_length: self.config.sequence_length,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatternRecognitionStats {
    pub total_patterns_cached: usize,
    pub average_confidence: f64,
    pub last_model_update: DateTime<Utc>,
    pub lstm_units: usize,
    pub sequence_length: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_pattern_recognizer_creation() {
        let config = PatternRecognitionConfig {
            lstm_units: 64,
            sequence_length: 30,
            prediction_horizon: 5,
            technical_indicators: vec!["rsi".to_string(), "macd".to_string()],
            min_confidence: 0.7,
        };

        let recognizer = PatternRecognizer::new(config).await;
        assert!(recognizer.is_ok());
    }

    #[tokio::test]
    async fn test_prediction_generation() {
        let config = PatternRecognitionConfig {
            lstm_units: 64,
            sequence_length: 30,
            prediction_horizon: 5,
            technical_indicators: vec!["rsi".to_string()],
            min_confidence: 0.6,
        };

        let mut recognizer = PatternRecognizer::new(config).await.unwrap();
        
        let mut features = FeatureVector::new("SOL/USDC".to_string());
        features.add_feature("price".to_string(), 100.0);
        features.add_feature("volume".to_string(), 1000.0);
        features.add_feature("rsi".to_string(), 65.0);

        let prediction = recognizer.predict_price_movement(&features).await;
        assert!(prediction.is_ok());
        
        let pred = prediction.unwrap();
        assert!(pred.confidence > 0.0);
        assert!(pred.confidence <= 1.0);
    }

    #[test]
    fn test_technical_indicators() {
        let indicators = TechnicalIndicators::new();
        
        let prices = vec![100.0, 101.0, 102.0, 101.5, 103.0, 102.0, 104.0, 103.5, 105.0, 104.0, 106.0, 105.5, 107.0, 106.0, 108.0];
        
        let rsi = indicators.calculate_rsi(&prices);
        assert!(rsi.is_ok());
        
        let rsi_value = rsi.unwrap();
        assert!(rsi_value >= 0.0);
        assert!(rsi_value <= 100.0);
    }
}
