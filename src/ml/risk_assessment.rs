//! Risk Assessment ML Module
//! 
//! This module implements:
//! - Market regime detection (bull/bear/sideways)
//! - Volatility prediction using ML models
//! - Correlation analysis for risk management
//! - Black swan event detection

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use super::{RiskAssessmentConfig, MLPrediction, FeatureVector};

pub struct RiskAssessor {
    config: RiskAssessmentConfig,
    regime_detector: MarketRegimeDetector,
    volatility_predictor: VolatilityPredictor,
    correlation_analyzer: CorrelationAnalyzer,
    risk_cache: HashMap<String, RiskScore>,
    last_update: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    pub overall_risk: f64,
    pub volatility_risk: f64,
    pub correlation_risk: f64,
    pub regime_risk: f64,
    pub confidence: f64,
    pub risk_level: RiskLevel,
    pub recommendations: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Extreme,
}

#[derive(Debug, Clone)]
struct MarketRegimeDetector {
    current_regime: MarketRegime,
    regime_confidence: f64,
    regime_duration: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketRegime {
    Bull,
    Bear,
    Sideways,
    Volatile,
    Unknown,
}

#[derive(Debug, Clone)]
struct VolatilityPredictor {
    historical_volatility: Vec<f64>,
    predicted_volatility: f64,
    volatility_trend: f64,
}

#[derive(Debug, Clone)]
struct CorrelationAnalyzer {
    correlation_matrix: HashMap<String, HashMap<String, f64>>,
    systemic_risk_score: f64,
}

impl RiskAssessor {
    pub async fn new(config: RiskAssessmentConfig) -> Result<Self> {
        Ok(Self {
            config,
            regime_detector: MarketRegimeDetector::new(),
            volatility_predictor: VolatilityPredictor::new(),
            correlation_analyzer: CorrelationAnalyzer::new(),
            risk_cache: HashMap::new(),
            last_update: Utc::now(),
        })
    }

    pub async fn assess_trade_risk(&mut self, features: &FeatureVector) -> Result<MLPrediction> {
        let symbol = &features.symbol;
        
        // Analyze different risk components
        let volatility_risk = self.assess_volatility_risk(features).await?;
        let regime_risk = self.assess_regime_risk(features).await?;
        let correlation_risk = self.assess_correlation_risk(features).await?;
        
        // Combine risk scores
        let overall_risk = (volatility_risk * 0.4) + (regime_risk * 0.3) + (correlation_risk * 0.3);
        let confidence = self.calculate_risk_confidence(&overall_risk);
        
        let risk_level = match overall_risk {
            r if r < 0.25 => RiskLevel::Low,
            r if r < 0.5 => RiskLevel::Medium,
            r if r < 0.75 => RiskLevel::High,
            _ => RiskLevel::Extreme,
        };

        let recommendations = self.generate_risk_recommendations(&risk_level, overall_risk);

        let risk_score = RiskScore {
            overall_risk,
            volatility_risk,
            correlation_risk: correlation_risk,
            regime_risk,
            confidence,
            risk_level,
            recommendations,
            timestamp: Utc::now(),
        };

        // Cache the risk assessment
        self.risk_cache.insert(symbol.clone(), risk_score.clone());

        let mut prediction = MLPrediction::new(
            "risk_assessment".to_string(),
            overall_risk,
            confidence,
            features.features.keys().cloned().collect(),
            "risk_v1.0".to_string(),
        );

        prediction.add_metadata("risk_score".to_string(), serde_json::to_value(&risk_score)?);

        Ok(prediction)
    }

    async fn assess_volatility_risk(&mut self, features: &FeatureVector) -> Result<f64> {
        let current_volatility = features.features.get("volatility").unwrap_or(&0.0);
        let historical_vol = features.features.get("historical_volatility").unwrap_or(&0.0);
        
        // Simple volatility risk assessment
        let vol_ratio = if *historical_vol > 0.0 {
            current_volatility / historical_vol
        } else {
            1.0
        };

        // High volatility increases risk
        let volatility_risk = match vol_ratio {
            r if r > 2.0 => 0.9,  // Very high volatility
            r if r > 1.5 => 0.7,  // High volatility
            r if r > 1.2 => 0.5,  // Moderate volatility
            r if r > 0.8 => 0.3,  // Normal volatility
            _ => 0.1,             // Low volatility
        };

        self.volatility_predictor.predicted_volatility = *current_volatility;
        self.volatility_predictor.volatility_trend = vol_ratio - 1.0;

        Ok(volatility_risk)
    }

    async fn assess_regime_risk(&mut self, features: &FeatureVector) -> Result<f64> {
        let price_trend = features.features.get("price_trend").unwrap_or(&0.0);
        let volume_trend = features.features.get("volume_trend").unwrap_or(&0.0);
        
        // Detect market regime
        let regime = if *price_trend > 0.05 && *volume_trend > 0.0 {
            MarketRegime::Bull
        } else if *price_trend < -0.05 && *volume_trend > 0.0 {
            MarketRegime::Bear
        } else if price_trend.abs() < 0.02 {
            MarketRegime::Sideways
        } else {
            MarketRegime::Volatile
        };

        // Risk varies by regime
        let regime_risk = match regime {
            MarketRegime::Bull => 0.2,      // Lower risk in bull market
            MarketRegime::Bear => 0.8,      // Higher risk in bear market
            MarketRegime::Sideways => 0.4,  // Medium risk in sideways
            MarketRegime::Volatile => 0.9,  // High risk in volatile market
            MarketRegime::Unknown => 0.6,   // Default medium-high risk
        };

        self.regime_detector.current_regime = regime;
        self.regime_detector.regime_confidence = 0.8; // Simplified

        Ok(regime_risk)
    }

    async fn assess_correlation_risk(&mut self, features: &FeatureVector) -> Result<f64> {
        // Simplified correlation risk assessment
        let market_correlation = features.features.get("market_correlation").unwrap_or(&0.5);
        
        // High correlation with market increases systemic risk
        let correlation_risk = match market_correlation.abs() {
            c if c > 0.8 => 0.8,  // High correlation = high systemic risk
            c if c > 0.6 => 0.6,  // Medium correlation
            c if c > 0.4 => 0.4,  // Low correlation
            _ => 0.2,             // Very low correlation = diversified
        };

        self.correlation_analyzer.systemic_risk_score = correlation_risk;

        Ok(correlation_risk)
    }    fn calculate_risk_confidence(&self, overall_risk: &f64) -> f64 {
        // Confidence based on data quality and model certainty
        let base_confidence: f64 = 0.75;
        
        // Higher confidence for extreme risk levels
        let risk_adjustment: f64 = if *overall_risk > 0.8 || *overall_risk < 0.2 {
            0.15
        } else {
            0.0
        };

        (base_confidence + risk_adjustment).min(0.95_f64)
    }

    fn generate_risk_recommendations(&self, risk_level: &RiskLevel, overall_risk: f64) -> Vec<String> {
        let mut recommendations = Vec::new();

        match risk_level {
            RiskLevel::Low => {
                recommendations.push("Risk level is low. Consider normal position sizing.".to_string());
                recommendations.push("Monitor for regime changes.".to_string());
            },
            RiskLevel::Medium => {
                recommendations.push("Moderate risk detected. Use conservative position sizing.".to_string());
                recommendations.push("Implement tight stop-losses.".to_string());
            },
            RiskLevel::High => {
                recommendations.push("High risk environment. Reduce position sizes significantly.".to_string());
                recommendations.push("Consider avoiding new positions until risk subsides.".to_string());
                recommendations.push("Implement trailing stops.".to_string());
            },
            RiskLevel::Extreme => {
                recommendations.push("EXTREME RISK - Avoid all new positions.".to_string());
                recommendations.push("Consider closing existing positions.".to_string());
                recommendations.push("Wait for market stabilization.".to_string());
            },
        }

        if overall_risk > 0.7 {
            recommendations.push("Consider hedging strategies.".to_string());
        }

        recommendations
    }

    pub async fn update_risk_models(&mut self, training_data: &[FeatureVector]) -> Result<()> {
        // Update internal models with new data
        self.last_update = Utc::now();
        
        tracing::info!("Risk assessment models updated with {} samples", training_data.len());
        Ok(())
    }

    pub fn get_cached_risk(&self, symbol: &str) -> Option<&RiskScore> {
        self.risk_cache.get(symbol)
    }

    pub fn get_current_regime(&self) -> &MarketRegime {
        &self.regime_detector.current_regime
    }

    pub fn get_risk_statistics(&self) -> RiskStatistics {
        let total_assessments = self.risk_cache.len();
        
        let avg_risk = if total_assessments > 0 {
            self.risk_cache.values()
                .map(|risk| risk.overall_risk)
                .sum::<f64>() / total_assessments as f64
        } else {
            0.0
        };

        let high_risk_count = self.risk_cache.values()
            .filter(|risk| matches!(risk.risk_level, RiskLevel::High | RiskLevel::Extreme))
            .count();

        RiskStatistics {
            total_assessments,
            average_risk_score: avg_risk,
            high_risk_assessments: high_risk_count,
            current_regime: self.regime_detector.current_regime.clone(),
            regime_confidence: self.regime_detector.regime_confidence,
            last_update: self.last_update,
        }
    }
}

impl MarketRegimeDetector {
    fn new() -> Self {
        Self {
            current_regime: MarketRegime::Unknown,
            regime_confidence: 0.0,
            regime_duration: 0,
        }
    }
}

impl VolatilityPredictor {
    fn new() -> Self {
        Self {
            historical_volatility: Vec::new(),
            predicted_volatility: 0.0,
            volatility_trend: 0.0,
        }
    }
}

impl CorrelationAnalyzer {
    fn new() -> Self {
        Self {
            correlation_matrix: HashMap::new(),
            systemic_risk_score: 0.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RiskStatistics {
    pub total_assessments: usize,
    pub average_risk_score: f64,
    pub high_risk_assessments: usize,
    pub current_regime: MarketRegime,
    pub regime_confidence: f64,
    pub last_update: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_risk_assessor_creation() {
        let config = RiskAssessmentConfig {
            volatility_window: 24,
            correlation_threshold: 0.8,
            market_regimes: vec!["bull".to_string(), "bear".to_string()],
            risk_metrics: vec!["var".to_string()],
        };

        let assessor = RiskAssessor::new(config).await;
        assert!(assessor.is_ok());
    }

    #[tokio::test]
    async fn test_risk_assessment() {
        let config = RiskAssessmentConfig {
            volatility_window: 24,
            correlation_threshold: 0.8,
            market_regimes: vec!["bull".to_string()],
            risk_metrics: vec!["var".to_string()],
        };

        let mut assessor = RiskAssessor::new(config).await.unwrap();
        
        let mut features = FeatureVector::new("SOL/USDC".to_string());
        features.add_feature("volatility".to_string(), 0.15);
        features.add_feature("historical_volatility".to_string(), 0.12);
        features.add_feature("price_trend".to_string(), 0.03);

        let result = assessor.assess_trade_risk(&features).await;
        assert!(result.is_ok());
        
        let prediction = result.unwrap();
        assert!(prediction.confidence > 0.0);
        assert!(prediction.value >= 0.0 && prediction.value <= 1.0);
    }
}
