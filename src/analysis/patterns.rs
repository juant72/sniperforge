use crate::strategies::{PricePoint, VolumePoint};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAnalysis {
    pub detected_patterns: Vec<Pattern>,
    pub pattern_confidence: f64,
    pub recommended_action: PatternAction,
    pub price_targets: Vec<PriceTarget>,
    pub volume_confirmation: bool,
    pub pattern_maturity: PatternMaturity,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub confidence: f64,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub completion_time: Option<chrono::DateTime<chrono::Utc>>,
    pub key_levels: Vec<f64>,
    pub expected_target: f64,
    pub stop_loss_level: f64,
    pub volume_profile: PatternVolumeProfile,
    pub reliability_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    // Reversal Patterns
    HeadAndShoulders,
    InverseHeadAndShoulders,
    DoubleTop,
    DoubleBottom,
    TripleTop,
    TripleBottom,

    // Continuation Patterns
    Flag,
    Pennant,
    Triangle,
    Rectangle,
    Wedge,

    // Candlestick Patterns
    Doji,
    Hammer,
    ShootingStar,
    Engulfing,

    // Custom Patterns
    Breakout,
    Reversal,
    Consolidation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternAction {
    StrongBuy,
    Buy,
    Hold,
    Sell,
    StrongSell,
    Wait,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceTarget {
    pub target_price: f64,
    pub probability: f64,
    pub timeframe_estimate: chrono::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternMaturity {
    Forming,   // Pattern is still developing
    Mature,    // Pattern is complete and ready for action
    Completed, // Pattern has played out
    Failed,    // Pattern failed to complete or work as expected
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternVolumeProfile {
    pub volume_confirmation: bool,
    pub volume_trend: String,
    pub breakout_volume: Option<f64>,
    pub average_volume: f64,
}

pub struct PatternRecognizer {
    min_pattern_length: usize,
    confidence_threshold: f64,
    volume_confirmation_enabled: bool,
}

impl PatternRecognizer {
    pub fn new() -> Self {
        Self {
            min_pattern_length: 10,
            confidence_threshold: 0.6,
            volume_confirmation_enabled: true,
        }
    }

    pub fn with_settings(
        min_length: usize,
        confidence_threshold: f64,
        volume_confirmation: bool,
    ) -> Self {
        Self {
            min_pattern_length: min_length,
            confidence_threshold,
            volume_confirmation_enabled: volume_confirmation,
        }
    }

    pub fn analyze_patterns(
        &self,
        price_history: &[PricePoint],
        volume_history: &[VolumePoint],
    ) -> Result<PatternAnalysis> {
        if price_history.len() < self.min_pattern_length {
            return Err(anyhow::anyhow!("Insufficient data for pattern analysis"));
        }

        let mut detected_patterns = Vec::new();

        // Detect various pattern types
        detected_patterns.extend(self.detect_head_and_shoulders(price_history, volume_history)?);
        detected_patterns.extend(self.detect_double_patterns(price_history, volume_history)?);
        detected_patterns.extend(self.detect_triangle_patterns(price_history, volume_history)?);
        detected_patterns.extend(self.detect_flag_patterns(price_history, volume_history)?);
        detected_patterns.extend(self.detect_breakout_patterns(price_history, volume_history)?);
        detected_patterns.extend(self.detect_candlestick_patterns(price_history, volume_history)?);

        // Filter patterns by confidence
        detected_patterns.retain(|p| p.confidence >= self.confidence_threshold);

        // Calculate overall pattern confidence
        let pattern_confidence = if detected_patterns.is_empty() {
            0.0
        } else {
            detected_patterns.iter().map(|p| p.confidence).sum::<f64>()
                / detected_patterns.len() as f64
        };

        // Determine recommended action
        let recommended_action = self.determine_pattern_action(&detected_patterns);

        // Generate price targets
        let price_targets = self.generate_price_targets(&detected_patterns, price_history);

        // Check volume confirmation
        let volume_confirmation =
            self.check_volume_confirmation(&detected_patterns, volume_history);

        // Determine pattern maturity
        let pattern_maturity = self.assess_pattern_maturity(&detected_patterns);

        Ok(PatternAnalysis {
            detected_patterns,
            pattern_confidence,
            recommended_action,
            price_targets,
            volume_confirmation,
            pattern_maturity,
            last_updated: chrono::Utc::now(),
        })
    }

    fn detect_head_and_shoulders(
        &self,
        prices: &[PricePoint],
        volumes: &[VolumePoint],
    ) -> Result<Vec<Pattern>> {
        let mut patterns = Vec::new();

        if prices.len() < 20 {
            return Ok(patterns);
        }
        let highs: Vec<f64> = prices.iter().map(|p| p.high).collect();
        let _lows: Vec<f64> = prices.iter().map(|p| p.low).collect();

        // Look for head and shoulders pattern
        for i in 10..highs.len() - 10 {
            // Find potential left shoulder, head, and right shoulder
            let left_shoulder_idx = self.find_local_maximum(&highs[i - 10..i], 0, i - 10)?;
            let head_idx = self.find_local_maximum(&highs[i - 5..i + 5], 0, i - 5)?;
            let right_shoulder_idx = self.find_local_maximum(&highs[i..i + 10], 0, i)?;

            if let (Some(left), Some(head), Some(right)) =
                (left_shoulder_idx, head_idx, right_shoulder_idx)
            {
                let left_high = highs[left];
                let head_high = highs[head];
                let right_high = highs[right];

                // Check if it forms a valid head and shoulders
                if head_high > left_high
                    && head_high > right_high
                    && (left_high - right_high).abs() / left_high < 0.05
                {
                    // Shoulders at similar levels

                    let neckline = (left_high + right_high) / 2.0;
                    let target = neckline - (head_high - neckline);

                    let confidence =
                        self.calculate_hs_confidence(left_high, head_high, right_high, neckline);

                    if confidence >= self.confidence_threshold {
                        patterns.push(Pattern {
                            pattern_type: PatternType::HeadAndShoulders,
                            confidence,
                            start_time: prices[left].timestamp,
                            completion_time: Some(prices[right].timestamp),
                            key_levels: vec![left_high, head_high, right_high, neckline],
                            expected_target: target,
                            stop_loss_level: head_high,
                            volume_profile: self.analyze_pattern_volume(volumes, left, right),
                            reliability_score: confidence * 0.8, // H&S patterns are generally reliable
                        });
                    }
                }
            }
        }

        Ok(patterns)
    }

    fn detect_double_patterns(
        &self,
        prices: &[PricePoint],
        volumes: &[VolumePoint],
    ) -> Result<Vec<Pattern>> {
        let mut patterns = Vec::new();

        if prices.len() < 15 {
            return Ok(patterns);
        }

        let highs: Vec<f64> = prices.iter().map(|p| p.high).collect();
        let lows: Vec<f64> = prices.iter().map(|p| p.low).collect();

        // Detect double tops
        for i in 10..highs.len() - 5 {
            if let Some(first_peak_idx) = self.find_local_maximum(&highs[i - 10..i], 0, i - 10)? {
                if let Some(second_peak_idx) = self.find_local_maximum(&highs[i..i + 5], 0, i)? {
                    let first_peak = highs[first_peak_idx];
                    let second_peak = highs[second_peak_idx];

                    // Check if peaks are at similar levels (within 3%)
                    if (first_peak - second_peak).abs() / first_peak < 0.03 {
                        let valley_start = first_peak_idx;
                        let valley_end = second_peak_idx;
                        let valley_low = lows[valley_start..=valley_end]
                            .iter()
                            .fold(f64::INFINITY, |a, &b| a.min(b));

                        let target = valley_low - (first_peak - valley_low);
                        let confidence = self.calculate_double_pattern_confidence(
                            first_peak,
                            second_peak,
                            valley_low,
                        );

                        if confidence >= self.confidence_threshold {
                            patterns.push(Pattern {
                                pattern_type: PatternType::DoubleTop,
                                confidence,
                                start_time: prices[first_peak_idx].timestamp,
                                completion_time: Some(prices[second_peak_idx].timestamp),
                                key_levels: vec![first_peak, second_peak, valley_low],
                                expected_target: target,
                                stop_loss_level: first_peak.max(second_peak),
                                volume_profile: self.analyze_pattern_volume(
                                    volumes,
                                    first_peak_idx,
                                    second_peak_idx,
                                ),
                                reliability_score: confidence * 0.75,
                            });
                        }
                    }
                }
            }
        }

        // Detect double bottoms (similar logic but inverted)
        for i in 10..lows.len() - 5 {
            if let Some(first_trough_idx) = self.find_local_minimum(&lows[i - 10..i], 0, i - 10)? {
                if let Some(second_trough_idx) = self.find_local_minimum(&lows[i..i + 5], 0, i)? {
                    let first_trough = lows[first_trough_idx];
                    let second_trough = lows[second_trough_idx];

                    if (first_trough - second_trough).abs() / first_trough < 0.03 {
                        let peak_start = first_trough_idx;
                        let peak_end = second_trough_idx;
                        let peak_high = highs[peak_start..=peak_end]
                            .iter()
                            .fold(0.0f64, |a, &b| a.max(b));

                        let target = peak_high + (peak_high - first_trough);
                        let confidence = self.calculate_double_pattern_confidence(
                            first_trough,
                            second_trough,
                            peak_high,
                        );

                        if confidence >= self.confidence_threshold {
                            patterns.push(Pattern {
                                pattern_type: PatternType::DoubleBottom,
                                confidence,
                                start_time: prices[first_trough_idx].timestamp,
                                completion_time: Some(prices[second_trough_idx].timestamp),
                                key_levels: vec![first_trough, second_trough, peak_high],
                                expected_target: target,
                                stop_loss_level: first_trough.min(second_trough),
                                volume_profile: self.analyze_pattern_volume(
                                    volumes,
                                    first_trough_idx,
                                    second_trough_idx,
                                ),
                                reliability_score: confidence * 0.75,
                            });
                        }
                    }
                }
            }
        }

        Ok(patterns)
    }

    fn detect_triangle_patterns(
        &self,
        prices: &[PricePoint],
        volumes: &[VolumePoint],
    ) -> Result<Vec<Pattern>> {
        let mut patterns = Vec::new();

        if prices.len() < 20 {
            return Ok(patterns);
        }

        let highs: Vec<f64> = prices.iter().map(|p| p.high).collect();
        let lows: Vec<f64> = prices.iter().map(|p| p.low).collect();

        // Detect ascending triangle (horizontal resistance, rising support)
        for i in 15..prices.len() - 5 {
            let window = &highs[i - 15..i];
            let resistance_level = window.iter().fold(0.0f64, |a, &b| a.max(b));

            // Check if highs are consistently near resistance
            let resistance_touches = window
                .iter()
                .filter(|&&h| (h - resistance_level).abs() / resistance_level < 0.02)
                .count();

            if resistance_touches >= 2 {
                // Check if lows are rising
                let low_window = &lows[i - 15..i];
                let slope = self.calculate_trendline_slope(low_window);

                if slope > 0.0001 {
                    // Positive slope for rising support
                    let current_support = self.calculate_support_level(low_window);
                    let target = resistance_level + (resistance_level - current_support) * 0.5;

                    let confidence = self.calculate_triangle_confidence(resistance_touches, slope);

                    if confidence >= self.confidence_threshold {
                        patterns.push(Pattern {
                            pattern_type: PatternType::Triangle,
                            confidence,
                            start_time: prices[i - 15].timestamp,
                            completion_time: None, // Pattern is forming
                            key_levels: vec![resistance_level, current_support],
                            expected_target: target,
                            stop_loss_level: current_support * 0.98,
                            volume_profile: self.analyze_pattern_volume(volumes, i - 15, i),
                            reliability_score: confidence * 0.7,
                        });
                    }
                }
            }
        }

        Ok(patterns)
    }

    fn detect_flag_patterns(
        &self,
        prices: &[PricePoint],
        volumes: &[VolumePoint],
    ) -> Result<Vec<Pattern>> {
        let mut patterns = Vec::new();

        if prices.len() < 20 {
            return Ok(patterns);
        }

        // Look for flag patterns (sharp move followed by consolidation)
        for i in 10..prices.len() - 10 {
            let pre_flag = &prices[i - 10..i];
            let flag_period = &prices[i..i + 10];

            // Check for sharp price movement (flagpole)
            let price_move =
                (pre_flag[pre_flag.len() - 1].close - pre_flag[0].close) / pre_flag[0].close;

            if price_move.abs() > 0.05 {
                // 5% move for flagpole
                // Check for consolidation in flag period
                let flag_range = self.calculate_price_range(flag_period);
                let flag_avg_price =
                    flag_period.iter().map(|p| p.close).sum::<f64>() / flag_period.len() as f64;

                if flag_range / flag_avg_price < 0.03 {
                    // Tight consolidation
                    let is_bullish_flag = price_move > 0.0;
                    let target = if is_bullish_flag {
                        flag_avg_price + price_move.abs() * flag_avg_price
                    } else {
                        flag_avg_price - price_move.abs() * flag_avg_price
                    };

                    let confidence =
                        self.calculate_flag_confidence(price_move, flag_range, flag_avg_price);

                    if confidence >= self.confidence_threshold {
                        patterns.push(Pattern {
                            pattern_type: PatternType::Flag,
                            confidence,
                            start_time: pre_flag[0].timestamp,
                            completion_time: Some(flag_period[flag_period.len() - 1].timestamp),
                            key_levels: vec![
                                pre_flag[0].close,
                                pre_flag[pre_flag.len() - 1].close,
                                flag_avg_price,
                            ],
                            expected_target: target,
                            stop_loss_level: if is_bullish_flag {
                                flag_avg_price * 0.97
                            } else {
                                flag_avg_price * 1.03
                            },
                            volume_profile: self.analyze_pattern_volume(volumes, i - 10, i + 10),
                            reliability_score: confidence * 0.8,
                        });
                    }
                }
            }
        }

        Ok(patterns)
    }

    fn detect_breakout_patterns(
        &self,
        prices: &[PricePoint],
        volumes: &[VolumePoint],
    ) -> Result<Vec<Pattern>> {
        let mut patterns = Vec::new();

        if prices.len() < 20 {
            return Ok(patterns);
        }

        // Detect breakouts from consolidation
        for i in 15..prices.len() - 5 {
            let consolidation_period = &prices[i - 15..i];
            let breakout_period = &prices[i..i + 5];

            // Check for consolidation
            let consolidation_range = self.calculate_price_range(consolidation_period);
            let consolidation_avg = consolidation_period.iter().map(|p| p.close).sum::<f64>()
                / consolidation_period.len() as f64;

            if consolidation_range / consolidation_avg < 0.04 {
                // Tight consolidation
                // Check for breakout
                let current_price = breakout_period[breakout_period.len() - 1].close;
                let breakout_percentage = (current_price - consolidation_avg) / consolidation_avg;

                if breakout_percentage.abs() > 0.02 {
                    // 2% breakout
                    let is_upward_breakout = breakout_percentage > 0.0;
                    let target = if is_upward_breakout {
                        current_price + consolidation_range
                    } else {
                        current_price - consolidation_range
                    };

                    let volume_confirmation = if volumes.len() > i {
                        let breakout_volume =
                            volumes[i..i + 5].iter().map(|v| v.volume).sum::<f64>() / 5.0;
                        let avg_volume =
                            volumes[i - 15..i].iter().map(|v| v.volume).sum::<f64>() / 15.0;
                        breakout_volume > avg_volume * 1.5
                    } else {
                        false
                    };

                    let confidence = self.calculate_breakout_confidence(
                        breakout_percentage,
                        volume_confirmation,
                        consolidation_range,
                    );

                    if confidence >= self.confidence_threshold {
                        patterns.push(Pattern {
                            pattern_type: PatternType::Breakout,
                            confidence,
                            start_time: consolidation_period[0].timestamp,
                            completion_time: Some(
                                breakout_period[breakout_period.len() - 1].timestamp,
                            ),
                            key_levels: vec![consolidation_avg, current_price],
                            expected_target: target,
                            stop_loss_level: consolidation_avg,
                            volume_profile: self.analyze_pattern_volume(volumes, i - 15, i + 5),
                            reliability_score: confidence
                                * if volume_confirmation { 0.9 } else { 0.6 },
                        });
                    }
                }
            }
        }

        Ok(patterns)
    }

    fn detect_candlestick_patterns(
        &self,
        prices: &[PricePoint],
        volumes: &[VolumePoint],
    ) -> Result<Vec<Pattern>> {
        let mut patterns = Vec::new();

        if prices.len() < 3 {
            return Ok(patterns);
        }

        // Detect simple candlestick patterns
        for i in 1..prices.len() - 1 {
            let current = &prices[i];
            let body_size = (current.close - current.open).abs();
            let total_range = current.high - current.low;

            if total_range == 0.0 {
                continue;
            }

            let body_percentage = body_size / total_range;

            // Doji pattern (small body relative to range)
            if body_percentage < 0.1 {
                patterns.push(Pattern {
                    pattern_type: PatternType::Doji,
                    confidence: 0.7,
                    start_time: current.timestamp,
                    completion_time: Some(current.timestamp),
                    key_levels: vec![current.open, current.close, current.high, current.low],
                    expected_target: current.close, // Neutral pattern
                    stop_loss_level: current.low,
                    volume_profile: self.analyze_single_candle_volume(volumes, i),
                    reliability_score: 0.5, // Doji requires confirmation
                });
            }

            // Hammer pattern (small body at top, long lower shadow)
            let lower_shadow = current.open.min(current.close) - current.low;
            let upper_shadow = current.high - current.open.max(current.close);

            if body_percentage < 0.3 && lower_shadow > body_size * 2.0 && upper_shadow < body_size {
                patterns.push(Pattern {
                    pattern_type: PatternType::Hammer,
                    confidence: 0.75,
                    start_time: current.timestamp,
                    completion_time: Some(current.timestamp),
                    key_levels: vec![current.open, current.close, current.high, current.low],
                    expected_target: current.close * 1.05, // Bullish reversal target
                    stop_loss_level: current.low,
                    volume_profile: self.analyze_single_candle_volume(volumes, i),
                    reliability_score: 0.7,
                });
            }
        }

        Ok(patterns)
    }

    // Helper methods for calculations
    fn find_local_maximum(
        &self,
        data: &[f64],
        _start_offset: usize,
        global_offset: usize,
    ) -> Result<Option<usize>> {
        if data.len() < 3 {
            return Ok(None);
        }

        for i in 1..data.len() - 1 {
            if data[i] > data[i - 1] && data[i] > data[i + 1] {
                return Ok(Some(global_offset + i));
            }
        }
        Ok(None)
    }
    fn find_local_minimum(
        &self,
        data: &[f64],
        _start_offset: usize,
        global_offset: usize,
    ) -> Result<Option<usize>> {
        if data.len() < 3 {
            return Ok(None);
        }

        for i in 1..data.len() - 1 {
            if data[i] < data[i - 1] && data[i] < data[i + 1] {
                return Ok(Some(global_offset + i));
            }
        }
        Ok(None)
    }

    fn calculate_hs_confidence(&self, left: f64, head: f64, right: f64, neckline: f64) -> f64 {
        let shoulder_symmetry = 1.0 - (left - right).abs() / left;
        let head_prominence = (head - neckline) / neckline;
        (shoulder_symmetry + head_prominence.min(0.5)) / 1.5
    }

    fn calculate_double_pattern_confidence(&self, first: f64, second: f64, middle: f64) -> f64 {
        let peak_symmetry = 1.0 - (first - second).abs() / first;
        let depth = ((first + second) / 2.0 - middle) / middle;
        (peak_symmetry + depth.min(0.3)) / 1.3
    }

    fn calculate_triangle_confidence(&self, touches: usize, slope: f64) -> f64 {
        let touch_score = (touches as f64 / 5.0).min(1.0);
        let slope_score = slope.abs().min(0.001) / 0.001;
        (touch_score + slope_score) / 2.0
    }

    fn calculate_flag_confidence(&self, price_move: f64, flag_range: f64, flag_avg: f64) -> f64 {
        let move_strength = price_move.abs().min(0.2) / 0.2;
        let consolidation_tightness = 1.0 - (flag_range / flag_avg / 0.05).min(1.0);
        (move_strength + consolidation_tightness) / 2.0
    }

    fn calculate_breakout_confidence(
        &self,
        breakout_pct: f64,
        volume_conf: bool,
        consolidation_range: f64,
    ) -> f64 {
        let breakout_strength = breakout_pct.abs().min(0.1) / 0.1;
        let volume_bonus = if volume_conf { 0.3 } else { 0.0 };
        let range_factor = (consolidation_range * 10.0).min(0.2);
        (breakout_strength + volume_bonus + range_factor).min(1.0)
    }

    fn calculate_price_range(&self, prices: &[PricePoint]) -> f64 {
        let high = prices.iter().map(|p| p.high).fold(0.0f64, |a, b| a.max(b));
        let low = prices
            .iter()
            .map(|p| p.low)
            .fold(f64::INFINITY, |a, b| a.min(b));
        high - low
    }

    fn calculate_trendline_slope(&self, data: &[f64]) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }

        let n = data.len() as f64;
        let x_sum: f64 = (0..data.len()).map(|i| i as f64).sum();
        let y_sum: f64 = data.iter().sum();
        let xy_sum: f64 = data.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let x_squared_sum: f64 = (0..data.len()).map(|i| (i as f64).powi(2)).sum();

        let denominator = n * x_squared_sum - x_sum.powi(2);
        if denominator.abs() < f64::EPSILON {
            return 0.0;
        }

        (n * xy_sum - x_sum * y_sum) / denominator
    }

    fn calculate_support_level(&self, lows: &[f64]) -> f64 {
        lows.iter().fold(f64::INFINITY, |a, &b| a.min(b))
    }
    fn analyze_pattern_volume(
        &self,
        _volumes: &[VolumePoint],
        start_idx: usize,
        end_idx: usize,
    ) -> PatternVolumeProfile {
        if _volumes.is_empty() || start_idx >= _volumes.len() || end_idx >= _volumes.len() {
            return PatternVolumeProfile {
                volume_confirmation: false,
                volume_trend: "Unknown".to_string(),
                breakout_volume: None,
                average_volume: 0.0,
            };
        }
        let pattern_volumes: Vec<f64> = _volumes[start_idx..=end_idx]
            .iter()
            .map(|v| v.volume)
            .collect();
        let average_volume = pattern_volumes.iter().sum::<f64>() / pattern_volumes.len() as f64;

        let volume_trend = if pattern_volumes.len() >= 2 {
            let first_half: f64 = pattern_volumes.iter().take(pattern_volumes.len() / 2).sum();
            let second_half: f64 = pattern_volumes.iter().skip(pattern_volumes.len() / 2).sum();
            if second_half > first_half * 1.1 {
                "Increasing".to_string()
            } else if second_half < first_half * 0.9 {
                "Decreasing".to_string()
            } else {
                "Stable".to_string()
            }
        } else {
            "Unknown".to_string()
        };

        PatternVolumeProfile {
            volume_confirmation: average_volume > 0.0,
            volume_trend,
            breakout_volume: pattern_volumes.last().copied(),
            average_volume,
        }
    }

    fn analyze_single_candle_volume(
        &self,
        volumes: &[VolumePoint],
        idx: usize,
    ) -> PatternVolumeProfile {
        if volumes.is_empty() || idx >= volumes.len() {
            return PatternVolumeProfile {
                volume_confirmation: false,
                volume_trend: "Unknown".to_string(),
                breakout_volume: None,
                average_volume: 0.0,
            };
        }

        PatternVolumeProfile {
            volume_confirmation: true,
            volume_trend: "Single".to_string(),
            breakout_volume: Some(volumes[idx].volume),
            average_volume: volumes[idx].volume,
        }
    }

    fn determine_pattern_action(&self, patterns: &[Pattern]) -> PatternAction {
        if patterns.is_empty() {
            return PatternAction::Wait;
        }

        let mut bullish_score = 0.0;
        let mut bearish_score = 0.0;

        for pattern in patterns {
            let weight = pattern.confidence * pattern.reliability_score;

            match pattern.pattern_type {
                PatternType::InverseHeadAndShoulders
                | PatternType::DoubleBottom
                | PatternType::Hammer
                | PatternType::Flag => {
                    bullish_score += weight;
                }
                PatternType::HeadAndShoulders
                | PatternType::DoubleTop
                | PatternType::ShootingStar => {
                    bearish_score += weight;
                }
                PatternType::Breakout => {
                    // Determine breakout direction from key levels
                    if pattern.key_levels.len() >= 2
                        && pattern.key_levels[1] > pattern.key_levels[0]
                    {
                        bullish_score += weight;
                    } else {
                        bearish_score += weight;
                    }
                }
                _ => {} // Neutral patterns
            }
        }

        let net_score = bullish_score - bearish_score;
        let total_score = bullish_score + bearish_score;

        if total_score < 0.3 {
            PatternAction::Wait
        } else if net_score > 0.5 {
            PatternAction::StrongBuy
        } else if net_score > 0.2 {
            PatternAction::Buy
        } else if net_score < -0.5 {
            PatternAction::StrongSell
        } else if net_score < -0.2 {
            PatternAction::Sell
        } else {
            PatternAction::Hold
        }
    }

    fn generate_price_targets(
        &self,
        patterns: &[Pattern],
        prices: &[PricePoint],
    ) -> Vec<PriceTarget> {
        let mut targets = Vec::new();
        let current_price = prices[prices.len() - 1].close;

        for pattern in patterns {
            let distance_to_target =
                (pattern.expected_target - current_price).abs() / current_price;
            let probability = pattern.confidence
                * pattern.reliability_score
                * (1.0 - distance_to_target.min(0.5));

            let timeframe_estimate = match pattern.pattern_type {
                PatternType::Breakout => chrono::Duration::hours(1),
                PatternType::Flag | PatternType::Pennant => chrono::Duration::hours(6),
                PatternType::Triangle => chrono::Duration::days(1),
                PatternType::HeadAndShoulders
                | PatternType::DoubleTop
                | PatternType::DoubleBottom => chrono::Duration::days(3),
                _ => chrono::Duration::hours(12),
            };

            targets.push(PriceTarget {
                target_price: pattern.expected_target,
                probability,
                timeframe_estimate,
            });
        }

        // Sort by probability (highest first)
        targets.sort_by(|a, b| b.probability.partial_cmp(&a.probability).unwrap());
        targets.truncate(5); // Keep top 5 targets

        targets
    }
    fn check_volume_confirmation(&self, patterns: &[Pattern], _volumes: &[VolumePoint]) -> bool {
        if !self.volume_confirmation_enabled || patterns.is_empty() {
            return true; // Default to true if not checking volume
        }

        patterns
            .iter()
            .any(|p| p.volume_profile.volume_confirmation)
    }

    fn assess_pattern_maturity(&self, patterns: &[Pattern]) -> PatternMaturity {
        if patterns.is_empty() {
            return PatternMaturity::Forming;
        }

        let completed_patterns = patterns
            .iter()
            .filter(|p| p.completion_time.is_some())
            .count();
        let total_patterns = patterns.len();

        if completed_patterns == total_patterns {
            PatternMaturity::Mature
        } else if completed_patterns > 0 {
            PatternMaturity::Forming
        } else {
            PatternMaturity::Forming
        }
    }
}

impl Default for PatternRecognizer {
    fn default() -> Self {
        Self::new()
    }
}
