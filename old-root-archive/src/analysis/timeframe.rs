use crate::strategies::{PricePoint, Timeframe, VolumePoint};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeframeAnalysis {
    pub timeframe: Timeframe,
    pub trend_direction: TrendDirection,
    pub trend_strength: f64,
    pub support_level: f64,
    pub resistance_level: f64,
    pub volume_profile: VolumeProfile,
    pub volatility: f64,
    pub momentum: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Bullish,
    Bearish,
    Sideways,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeProfile {
    pub average_volume: f64,
    pub volume_trend: VolumeTrend,
    pub volume_spikes: Vec<VolumeSpike>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeTrend {
    Increasing,
    Decreasing,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSpike {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub volume: f64,
    pub spike_ratio: f64, // How many times above average
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTimeframeSignal {
    pub overall_bias: TrendDirection,
    pub confidence: f64,
    pub timeframe_alignment: f64, // How well timeframes agree (0.0 to 1.0)
    pub entry_timing: EntryTiming,
    pub risk_level: RiskAssessment,
    pub timeframe_analyses: HashMap<Timeframe, TimeframeAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryTiming {
    Immediate, // All timeframes aligned, strong signal
    Cautious,  // Some divergence, wait for confirmation
    Wait,      // Conflicting signals, wait for clarity
    Avoid,     // High risk or unclear conditions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskAssessment {
    Low,
    Medium,
    High,
    VeryHigh,
}

pub struct MultiTimeframeAnalyzer {
    timeframes: Vec<Timeframe>,
    min_data_points: HashMap<Timeframe, usize>,
}

impl MultiTimeframeAnalyzer {
    pub fn new() -> Self {
        let mut min_data_points = HashMap::new();
        min_data_points.insert(Timeframe::OneMin, 60); // 1 hour of 1min data
        min_data_points.insert(Timeframe::FiveMin, 48); // 4 hours of 5min data
        min_data_points.insert(Timeframe::FifteenMin, 32); // 8 hours of 15min data
        min_data_points.insert(Timeframe::OneHour, 24); // 1 day of 1hour data
        min_data_points.insert(Timeframe::FourHour, 12); // 2 days of 4hour data
        min_data_points.insert(Timeframe::OneDay, 7); // 1 week of daily data

        Self {
            timeframes: vec![
                Timeframe::OneMin,
                Timeframe::FiveMin,
                Timeframe::FifteenMin,
                Timeframe::OneHour,
            ],
            min_data_points,
        }
    }

    pub fn with_timeframes(timeframes: Vec<Timeframe>) -> Self {
        let mut analyzer = Self::new();
        analyzer.timeframes = timeframes;
        analyzer
    }

    pub fn analyze(
        &self,
        price_history: &[PricePoint],
        volume_history: &[VolumePoint],
    ) -> Result<MultiTimeframeSignal> {
        let mut timeframe_analyses = HashMap::new();

        // Analyze each timeframe
        for &timeframe in &self.timeframes {
            if let Some(analysis) =
                self.analyze_timeframe(timeframe, price_history, volume_history)?
            {
                timeframe_analyses.insert(timeframe, analysis);
            }
        }

        if timeframe_analyses.is_empty() {
            return Err(anyhow::anyhow!(
                "Insufficient data for any timeframe analysis"
            ));
        }

        // Determine overall bias and confidence
        let (overall_bias, confidence) = self.calculate_overall_bias(&timeframe_analyses);

        // Calculate timeframe alignment
        let timeframe_alignment = self.calculate_timeframe_alignment(&timeframe_analyses);

        // Determine entry timing
        let entry_timing = self.determine_entry_timing(&timeframe_analyses, timeframe_alignment);

        // Assess risk level
        let risk_level = self.assess_risk_level(&timeframe_analyses, timeframe_alignment);

        Ok(MultiTimeframeSignal {
            overall_bias,
            confidence,
            timeframe_alignment,
            entry_timing,
            risk_level,
            timeframe_analyses,
        })
    }

    fn analyze_timeframe(
        &self,
        timeframe: Timeframe,
        price_history: &[PricePoint],
        volume_history: &[VolumePoint],
    ) -> Result<Option<TimeframeAnalysis>> {
        let min_points = self.min_data_points.get(&timeframe).unwrap_or(&20);

        if price_history.len() < *min_points || volume_history.len() < *min_points {
            return Ok(None);
        }

        // Aggregate data according to timeframe
        let (aggregated_prices, aggregated_volumes) =
            self.aggregate_data(timeframe, price_history, volume_history)?;

        if aggregated_prices.is_empty() {
            return Ok(None);
        }

        // Calculate trend analysis
        let (trend_direction, trend_strength) = self.analyze_trend(&aggregated_prices)?;

        // Calculate support and resistance
        let (support_level, resistance_level) =
            self.calculate_support_resistance(&aggregated_prices)?;

        // Analyze volume profile
        let volume_profile = self.analyze_volume_profile(&aggregated_volumes)?;

        // Calculate volatility
        let volatility = self.calculate_volatility(&aggregated_prices)?;

        // Calculate momentum
        let momentum = self.calculate_momentum(&aggregated_prices)?;

        Ok(Some(TimeframeAnalysis {
            timeframe,
            trend_direction,
            trend_strength,
            support_level,
            resistance_level,
            volume_profile,
            volatility,
            momentum,
            last_updated: chrono::Utc::now(),
        }))
    }

    fn aggregate_data(
        &self,
        timeframe: Timeframe,
        price_history: &[PricePoint],
        volume_history: &[VolumePoint],
    ) -> Result<(Vec<f64>, Vec<f64>)> {
        let interval_minutes = match timeframe {
            Timeframe::OneMin => 1,
            Timeframe::FiveMin => 5,
            Timeframe::FifteenMin => 15,
            Timeframe::OneHour => 60,
            Timeframe::FourHour => 240,
            Timeframe::OneDay => 1440,
        };

        // For simplification, we'll just sample the data at intervals
        // In a real implementation, you'd properly aggregate OHLCV data
        let mut aggregated_prices = Vec::new();
        let mut aggregated_volumes = Vec::new();

        let step_size = interval_minutes.max(1);

        for i in (0..price_history.len()).step_by(step_size) {
            aggregated_prices.push(price_history[i].close);
        }

        for i in (0..volume_history.len()).step_by(step_size) {
            aggregated_volumes.push(volume_history[i].volume);
        }

        Ok((aggregated_prices, aggregated_volumes))
    }

    fn analyze_trend(&self, prices: &[f64]) -> Result<(TrendDirection, f64)> {
        if prices.len() < 10 {
            return Ok((TrendDirection::Sideways, 0.0));
        }

        // Calculate multiple trend indicators
        let short_ma = self.calculate_ma(prices, 5);
        let long_ma = self.calculate_ma(prices, 20);
        let current_price = prices[prices.len() - 1];

        let mut trend_score = 0.0;

        // Moving average trend
        if let (Some(short), Some(long)) = (short_ma, long_ma) {
            if short > long {
                trend_score += 0.3; // Bullish
            } else {
                trend_score -= 0.3; // Bearish
            }

            // Price position relative to MAs
            if current_price > short && current_price > long {
                trend_score += 0.2;
            } else if current_price < short && current_price < long {
                trend_score -= 0.2;
            }
        }

        // Price momentum
        if prices.len() >= 5 {
            let recent_change =
                (current_price - prices[prices.len() - 5]) / prices[prices.len() - 5];
            trend_score += recent_change * 2.0; // Amplify momentum effect
        }

        // Linear regression trend
        if let Some(slope) = self.calculate_linear_regression_slope(prices) {
            trend_score += slope * 10.0; // Normalize slope contribution
        }

        // Determine trend direction and strength
        let trend_strength = trend_score.abs().min(1.0);
        let trend_direction = if trend_score > 0.1 {
            TrendDirection::Bullish
        } else if trend_score < -0.1 {
            TrendDirection::Bearish
        } else {
            TrendDirection::Sideways
        };

        Ok((trend_direction, trend_strength))
    }

    fn calculate_ma(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let sum: f64 = prices.iter().rev().take(period).sum();
        Some(sum / period as f64)
    }

    fn calculate_linear_regression_slope(&self, prices: &[f64]) -> Option<f64> {
        if prices.len() < 10 {
            return None;
        }

        let n = prices.len() as f64;
        let x_sum: f64 = (0..prices.len()).map(|i| i as f64).sum();
        let y_sum: f64 = prices.iter().sum();
        let xy_sum: f64 = prices
            .iter()
            .enumerate()
            .map(|(i, &price)| i as f64 * price)
            .sum();
        let x_squared_sum: f64 = (0..prices.len()).map(|i| (i as f64).powi(2)).sum();

        let denominator = n * x_squared_sum - x_sum.powi(2);
        if denominator.abs() < f64::EPSILON {
            return None;
        }

        let slope = (n * xy_sum - x_sum * y_sum) / denominator;
        Some(slope)
    }

    fn calculate_support_resistance(&self, prices: &[f64]) -> Result<(f64, f64)> {
        if prices.len() < 20 {
            let current = prices[prices.len() - 1];
            return Ok((current * 0.98, current * 1.02));
        }

        // Find local minima and maxima
        let mut local_mins = Vec::new();
        let mut local_maxs = Vec::new();
        let window = 3;

        for i in window..prices.len() - window {
            let mut is_min = true;
            let mut is_max = true;

            for j in 1..=window {
                if prices[i] > prices[i - j] || prices[i] > prices[i + j] {
                    is_min = false;
                }
                if prices[i] < prices[i - j] || prices[i] < prices[i + j] {
                    is_max = false;
                }
            }

            if is_min {
                local_mins.push(prices[i]);
            }
            if is_max {
                local_maxs.push(prices[i]);
            }
        }

        let current_price = prices[prices.len() - 1];

        // Find the most relevant support (highest below current) and resistance (lowest above current)
        let support = local_mins
            .iter()
            .filter(|&&price| price < current_price)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .copied()
            .unwrap_or(current_price * 0.95);

        let resistance = local_maxs
            .iter()
            .filter(|&&price| price > current_price)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .copied()
            .unwrap_or(current_price * 1.05);

        Ok((support, resistance))
    }

    fn analyze_volume_profile(&self, volumes: &[f64]) -> Result<VolumeProfile> {
        if volumes.is_empty() {
            return Ok(VolumeProfile {
                average_volume: 0.0,
                volume_trend: VolumeTrend::Stable,
                volume_spikes: Vec::new(),
            });
        }

        let average_volume = volumes.iter().sum::<f64>() / volumes.len() as f64;

        // Determine volume trend
        let volume_trend = if volumes.len() >= 10 {
            let recent_avg = volumes.iter().rev().take(5).sum::<f64>() / 5.0;
            let past_avg = volumes.iter().rev().skip(5).take(5).sum::<f64>() / 5.0;

            if recent_avg > past_avg * 1.1 {
                VolumeTrend::Increasing
            } else if recent_avg < past_avg * 0.9 {
                VolumeTrend::Decreasing
            } else {
                VolumeTrend::Stable
            }
        } else {
            VolumeTrend::Stable
        };

        // Find volume spikes (volumes significantly above average)
        let mut volume_spikes = Vec::new();
        for (i, &volume) in volumes.iter().enumerate() {
            if volume > average_volume * 2.0 {
                volume_spikes.push(VolumeSpike {
                    timestamp: chrono::Utc::now()
                        - chrono::Duration::minutes((volumes.len() - i) as i64),
                    volume,
                    spike_ratio: volume / average_volume,
                });
            }
        }

        Ok(VolumeProfile {
            average_volume,
            volume_trend,
            volume_spikes,
        })
    }

    fn calculate_volatility(&self, prices: &[f64]) -> Result<f64> {
        if prices.len() < 2 {
            return Ok(0.0);
        }

        let returns: Vec<f64> = prices
            .windows(2)
            .map(|window| (window[1] - window[0]) / window[0])
            .collect();

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|&r| (r - mean_return).powi(2))
            .sum::<f64>()
            / returns.len() as f64;

        Ok(variance.sqrt())
    }

    fn calculate_momentum(&self, prices: &[f64]) -> Result<f64> {
        if prices.len() < 10 {
            return Ok(0.0);
        }

        let current = prices[prices.len() - 1];
        let past = prices[prices.len() - 10];

        Ok((current - past) / past)
    }

    fn calculate_overall_bias(
        &self,
        analyses: &HashMap<Timeframe, TimeframeAnalysis>,
    ) -> (TrendDirection, f64) {
        if analyses.is_empty() {
            return (TrendDirection::Sideways, 0.0);
        }

        let mut bullish_score = 0.0;
        let mut bearish_score = 0.0;
        let mut total_weight = 0.0;

        for (timeframe, analysis) in analyses {
            // Longer timeframes get higher weight
            let weight = match timeframe {
                Timeframe::OneMin => 1.0,
                Timeframe::FiveMin => 2.0,
                Timeframe::FifteenMin => 3.0,
                Timeframe::OneHour => 4.0,
                Timeframe::FourHour => 5.0,
                Timeframe::OneDay => 6.0,
            };

            let trend_contribution = analysis.trend_strength * weight;

            match analysis.trend_direction {
                TrendDirection::Bullish => bullish_score += trend_contribution,
                TrendDirection::Bearish => bearish_score += trend_contribution,
                TrendDirection::Sideways => {} // No contribution
            }

            total_weight += weight;
        }

        if total_weight == 0.0 {
            return (TrendDirection::Sideways, 0.0);
        }

        let net_score = (bullish_score - bearish_score) / total_weight;
        let confidence = (bullish_score + bearish_score) / total_weight;

        let overall_bias = if net_score > 0.1 {
            TrendDirection::Bullish
        } else if net_score < -0.1 {
            TrendDirection::Bearish
        } else {
            TrendDirection::Sideways
        };

        (overall_bias, confidence.min(1.0))
    }

    fn calculate_timeframe_alignment(
        &self,
        analyses: &HashMap<Timeframe, TimeframeAnalysis>,
    ) -> f64 {
        if analyses.len() < 2 {
            return 1.0; // Perfect alignment if only one timeframe
        }

        let bullish_count = analyses
            .values()
            .filter(|a| matches!(a.trend_direction, TrendDirection::Bullish))
            .count();

        let bearish_count = analyses
            .values()
            .filter(|a| matches!(a.trend_direction, TrendDirection::Bearish))
            .count();

        let total_count = analyses.len();
        let max_aligned = bullish_count.max(bearish_count);

        max_aligned as f64 / total_count as f64
    }
    fn determine_entry_timing(
        &self,
        _analyses: &HashMap<Timeframe, TimeframeAnalysis>,
        alignment: f64,
    ) -> EntryTiming {
        if alignment >= 0.8 {
            EntryTiming::Immediate
        } else if alignment >= 0.6 {
            EntryTiming::Cautious
        } else if alignment >= 0.4 {
            EntryTiming::Wait
        } else {
            EntryTiming::Avoid
        }
    }

    fn assess_risk_level(
        &self,
        analyses: &HashMap<Timeframe, TimeframeAnalysis>,
        alignment: f64,
    ) -> RiskAssessment {
        let avg_volatility: f64 =
            analyses.values().map(|a| a.volatility).sum::<f64>() / analyses.len() as f64;

        let high_volatility = avg_volatility > 0.05; // 5% volatility threshold

        match (alignment, high_volatility) {
            (a, false) if a >= 0.8 => RiskAssessment::Low,
            (a, false) if a >= 0.6 => RiskAssessment::Medium,
            (a, true) if a >= 0.8 => RiskAssessment::Medium,
            (a, _) if a >= 0.4 => RiskAssessment::High,
            _ => RiskAssessment::VeryHigh,
        }
    }
}

impl Default for MultiTimeframeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
