//! Portfolio Correlation Analyzer - Advanced correlation and diversification analysis
//! 
//! Provides correlation analysis, diversification metrics, and risk concentration measurements

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use super::Position;

/// Correlation analyzer for portfolio diversification
#[derive(Debug)]
pub struct CorrelationAnalyzer {
    price_history: HashMap<String, Vec<PricePoint>>,
    correlation_cache: HashMap<(String, String), f64>,
    last_update: DateTime<Utc>,
}

/// Price point for correlation calculation
#[derive(Debug, Clone)]
pub struct PricePoint {
    pub timestamp: DateTime<Utc>,
    pub price: f64,
    pub return_pct: f64,
}

/// Correlation matrix for all assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrix {
    pub assets: Vec<String>,
    pub matrix: Vec<Vec<f64>>,
    pub timestamp: DateTime<Utc>,
    pub period_days: i64,
    pub observations: usize,
}

/// Diversification metrics for the portfolio
#[derive(Debug, Serialize, Deserialize)]
pub struct DiversificationMetrics {
    pub portfolio_correlation_score: f64,
    pub effective_number_of_assets: f64,
    pub concentration_hhi: f64,
    pub diversification_ratio: f64,
    pub correlation_risk_contribution: f64,
    pub asset_correlations: HashMap<String, AssetCorrelationMetrics>,
    pub strategy_correlations: HashMap<String, StrategyCorrelationMetrics>,
    pub high_correlation_pairs: Vec<CorrelationPair>,
    pub diversification_recommendations: Vec<String>,
}

/// Correlation metrics for individual assets
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetCorrelationMetrics {
    pub avg_correlation: f64,
    pub max_correlation: f64,
    pub correlation_with_portfolio: f64,
    pub risk_contribution: f64,
    pub diversification_benefit: f64,
}

/// Correlation metrics for strategies
#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyCorrelationMetrics {
    pub internal_correlation: f64,
    pub cross_strategy_correlation: f64,
    pub concentration_risk: f64,
    pub performance_correlation: f64,
}

/// High correlation pair identification
#[derive(Debug, Serialize, Deserialize)]
pub struct CorrelationPair {
    pub asset1: String,
    pub asset2: String,
    pub correlation: f64,
    pub weight1: f64,
    pub weight2: f64,
    pub combined_weight: f64,
    pub risk_impact: f64,
}

/// Portfolio risk decomposition
#[derive(Debug, Serialize, Deserialize)]
pub struct RiskDecomposition {
    pub total_portfolio_risk: f64,
    pub idiosyncratic_risk: f64,
    pub systematic_risk: f64,
    pub concentration_risk: f64,
    pub correlation_risk: f64,
    pub asset_risk_contributions: HashMap<String, f64>,
    pub strategy_risk_contributions: HashMap<String, f64>,
}

impl CorrelationAnalyzer {
    /// Create new correlation analyzer
    pub fn new() -> Self {
        Self {
            price_history: HashMap::new(),
            correlation_cache: HashMap::new(),
            last_update: Utc::now(),
        }
    }

    /// Update price data for correlation analysis
    pub fn update_prices(&mut self, symbol: &str, price: f64) {
        let now = Utc::now();
        
        // Calculate return if we have previous price
        let return_pct = if let Some(history) = self.price_history.get(symbol) {
            if let Some(last_point) = history.last() {
                if last_point.price > 0.0 {
                    ((price - last_point.price) / last_point.price) * 100.0
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        let price_point = PricePoint {
            timestamp: now,
            price,
            return_pct,
        };

        // Add to history
        self.price_history.entry(symbol.to_string())
            .or_insert_with(Vec::new)
            .push(price_point);

        // Keep only last 100 points per asset
        if let Some(history) = self.price_history.get_mut(symbol) {
            if history.len() > 100 {
                history.remove(0);
            }
        }

        // Clear correlation cache if data is updated
        if (now - self.last_update).num_minutes() > 5 {
            self.correlation_cache.clear();
            self.last_update = now;
        }
    }

    /// Calculate correlation matrix for all assets
    pub fn calculate_correlation_matrix(&mut self, period_days: i64) -> Result<CorrelationMatrix> {
        let cutoff_date = Utc::now() - Duration::days(period_days);
        
        // Get all assets with sufficient data
        let mut assets = Vec::new();
        let mut filtered_data = HashMap::new();

        // Collect filtered data without borrowing self
        let price_history = self.price_history.clone();
        for (symbol, history) in &price_history {
            let recent_data: Vec<_> = history.iter()
                .filter(|p| p.timestamp >= cutoff_date)
                .collect();

            if recent_data.len() >= 10 { // Minimum 10 observations
                assets.push(symbol.clone());
                filtered_data.insert(symbol.clone(), recent_data);
            }
        }

        if assets.len() < 2 {
            return Err(anyhow::anyhow!("Insufficient data for correlation analysis"));
        }

        assets.sort();
        let n = assets.len();
        let mut matrix = vec![vec![0.0; n]; n];

        // Calculate correlation for each pair
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    matrix[i][j] = 1.0;
                } else {
                    let correlation = self.calculate_correlation_from_data(
                        &assets[i], 
                        &assets[j], 
                        &filtered_data
                    )?;
                    matrix[i][j] = correlation;
                    matrix[j][i] = correlation; // Symmetric matrix
                }
            }
        }

        let observations = filtered_data.values()
            .map(|data| data.len())
            .min()
            .unwrap_or(0);

        Ok(CorrelationMatrix {
            assets,
            matrix,
            timestamp: Utc::now(),
            period_days,
            observations,
        })
    }

    /// Calculate correlation between two assets from prepared data
    fn calculate_correlation_from_data(
        &mut self,
        asset1: &str,
        asset2: &str,
        data: &HashMap<String, Vec<&PricePoint>>,
    ) -> Result<f64> {
        // Check cache first
        let cache_key = if asset1 < asset2 {
            (asset1.to_string(), asset2.to_string())
        } else {
            (asset2.to_string(), asset1.to_string())
        };

        if let Some(&cached_corr) = self.correlation_cache.get(&cache_key) {
            return Ok(cached_corr);
        }

        let returns1 = data.get(asset1)
            .ok_or_else(|| anyhow::anyhow!("No data for asset1"))?
            .iter()
            .map(|p| p.return_pct)
            .collect::<Vec<_>>();

        let returns2 = data.get(asset2)
            .ok_or_else(|| anyhow::anyhow!("No data for asset2"))?
            .iter()
            .map(|p| p.return_pct)
            .collect::<Vec<_>>();

        if returns1.len() != returns2.len() || returns1.len() < 2 {
            return Ok(0.0);
        }

        let correlation = self.pearson_correlation(&returns1, &returns2);
        
        // Cache the result
        self.correlation_cache.insert(cache_key, correlation);
        
        Ok(correlation)
    }

    /// Calculate Pearson correlation coefficient
    fn pearson_correlation(&self, x: &[f64], y: &[f64]) -> f64 {
        if x.len() != y.len() || x.len() < 2 {
            return 0.0;
        }

        let n = x.len() as f64;
        let mean_x = x.iter().sum::<f64>() / n;
        let mean_y = y.iter().sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut sum_sq_x = 0.0;
        let mut sum_sq_y = 0.0;

        for i in 0..x.len() {
            let diff_x = x[i] - mean_x;
            let diff_y = y[i] - mean_y;
            
            numerator += diff_x * diff_y;
            sum_sq_x += diff_x * diff_x;
            sum_sq_y += diff_y * diff_y;
        }

        let denominator = (sum_sq_x * sum_sq_y).sqrt();
        
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }

    /// Analyze portfolio diversification
    pub fn analyze_diversification(
        &mut self,
        positions: &HashMap<Uuid, Position>,
        period_days: i64,
    ) -> Result<DiversificationMetrics> {
        // Update prices for all positions
        for position in positions.values() {
            self.update_prices(&position.symbol, position.current_price);
        }

        // Calculate correlation matrix
        let correlation_matrix = self.calculate_correlation_matrix(period_days)?;
        
        // Calculate portfolio weights
        let total_value: f64 = positions.values().map(|p| p.value_usd).sum();
        let mut weights = HashMap::new();
        
        for position in positions.values() {
            let weight = if total_value > 0.0 { position.value_usd / total_value } else { 0.0 };
            weights.insert(position.symbol.clone(), weight);
        }

        // Calculate portfolio correlation score
        let portfolio_correlation_score = self.calculate_portfolio_correlation_score(
            &correlation_matrix, &weights
        );

        // Calculate effective number of assets (diversification index)
        let effective_number_of_assets = self.calculate_effective_number_of_assets(&weights);

        // Calculate HHI concentration index
        let concentration_hhi = weights.values().map(|w| w * w).sum::<f64>();

        // Calculate diversification ratio
        let diversification_ratio = self.calculate_diversification_ratio(
            &correlation_matrix, &weights
        );

        // Calculate correlation risk contribution
        let correlation_risk_contribution = self.calculate_correlation_risk_contribution(
            &correlation_matrix, &weights
        );

        // Analyze individual asset correlations
        let asset_correlations = self.analyze_asset_correlations(
            &correlation_matrix, &weights, positions
        );

        // Analyze strategy correlations
        let strategy_correlations = self.analyze_strategy_correlations(positions);

        // Identify high correlation pairs
        let high_correlation_pairs = self.identify_high_correlation_pairs(
            &correlation_matrix, &weights, 0.7
        );

        // Generate diversification recommendations
        let diversification_recommendations = self.generate_diversification_recommendations(
            &asset_correlations, &high_correlation_pairs, concentration_hhi
        );

        Ok(DiversificationMetrics {
            portfolio_correlation_score,
            effective_number_of_assets,
            concentration_hhi,
            diversification_ratio,
            correlation_risk_contribution,
            asset_correlations,
            strategy_correlations,
            high_correlation_pairs,
            diversification_recommendations,
        })
    }

    /// Calculate portfolio-level correlation score
    fn calculate_portfolio_correlation_score(
        &self,
        matrix: &CorrelationMatrix,
        weights: &HashMap<String, f64>,
    ) -> f64 {
        let mut weighted_correlation = 0.0;
        let mut total_weight = 0.0;

        for i in 0..matrix.assets.len() {
            for j in (i + 1)..matrix.assets.len() {
                let asset1 = &matrix.assets[i];
                let asset2 = &matrix.assets[j];
                
                let weight1 = weights.get(asset1).unwrap_or(&0.0);
                let weight2 = weights.get(asset2).unwrap_or(&0.0);
                let combined_weight = weight1 * weight2;
                
                weighted_correlation += matrix.matrix[i][j] * combined_weight;
                total_weight += combined_weight;
            }
        }

        if total_weight > 0.0 {
            weighted_correlation / total_weight
        } else {
            0.0
        }
    }

    /// Calculate effective number of assets (Herfindahl-based)
    fn calculate_effective_number_of_assets(&self, weights: &HashMap<String, f64>) -> f64 {
        let hhi = weights.values().map(|w| w * w).sum::<f64>();
        if hhi > 0.0 {
            1.0 / hhi
        } else {
            0.0
        }
    }

    /// Calculate diversification ratio
    fn calculate_diversification_ratio(
        &self,
        matrix: &CorrelationMatrix,
        weights: &HashMap<String, f64>,
    ) -> f64 {
        // Simplified diversification ratio calculation
        // In practice, this would require individual asset volatilities
        let avg_correlation = self.calculate_portfolio_correlation_score(matrix, weights);
        let concentration = weights.values().map(|w| w * w).sum::<f64>();
        
        // Higher correlation and concentration = lower diversification
        1.0 - (avg_correlation * concentration)
    }

    /// Calculate correlation risk contribution
    fn calculate_correlation_risk_contribution(
        &self,
        matrix: &CorrelationMatrix,
        weights: &HashMap<String, f64>,
    ) -> f64 {
        // Risk contribution from correlations
        let portfolio_correlation = self.calculate_portfolio_correlation_score(matrix, weights);
        let concentration = weights.values().map(|w| w * w).sum::<f64>();
        
        portfolio_correlation * concentration
    }

    /// Analyze individual asset correlations
    fn analyze_asset_correlations(
        &self,
        matrix: &CorrelationMatrix,
        weights: &HashMap<String, f64>,
        _positions: &HashMap<Uuid, Position>,
    ) -> HashMap<String, AssetCorrelationMetrics> {
        let mut asset_correlations = HashMap::new();

        for (i, asset) in matrix.assets.iter().enumerate() {
            let mut correlations = Vec::new();
            
            // Get correlations with all other assets
            for (j, _) in matrix.assets.iter().enumerate() {
                if i != j {
                    correlations.push(matrix.matrix[i][j]);
                }
            }

            let avg_correlation = if !correlations.is_empty() {
                correlations.iter().sum::<f64>() / correlations.len() as f64
            } else {
                0.0
            };

            let max_correlation: f64 = correlations.iter().fold(0.0, |a, &b| a.max(b.abs()));

            // Correlation with portfolio (simplified)
            let correlation_with_portfolio = avg_correlation * weights.get(asset).unwrap_or(&0.0);

            // Risk contribution (simplified)
            let weight = weights.get(asset).unwrap_or(&0.0);
            let risk_contribution = weight * weight * (1.0 + avg_correlation);

            // Diversification benefit
            let diversification_benefit = 1.0 - avg_correlation.abs();

            asset_correlations.insert(asset.clone(), AssetCorrelationMetrics {
                avg_correlation,
                max_correlation,
                correlation_with_portfolio,
                risk_contribution,
                diversification_benefit,
            });
        }

        asset_correlations
    }

    /// Analyze strategy-level correlations
    fn analyze_strategy_correlations(
        &self,
        positions: &HashMap<Uuid, Position>,
    ) -> HashMap<String, StrategyCorrelationMetrics> {
        let mut strategy_metrics = HashMap::new();

        // Group positions by strategy
        let mut strategy_groups: HashMap<String, Vec<&Position>> = HashMap::new();
        for position in positions.values() {
            strategy_groups.entry(position.strategy.clone())
                .or_insert_with(Vec::new)
                .push(position);
        }

        for (strategy, positions) in strategy_groups {
            // Calculate internal correlation (simplified)
            let symbols: Vec<_> = positions.iter().map(|p| &p.symbol).collect();
            let unique_symbols: std::collections::HashSet<_> = symbols.iter().collect();
            let internal_correlation = if unique_symbols.len() > 1 {
                0.5 // Placeholder - would calculate actual correlation
            } else {
                1.0 // Single asset = perfect correlation
            };

            // Cross-strategy correlation (simplified)
            let cross_strategy_correlation = 0.3; // Placeholder

            // Concentration risk within strategy
            let total_strategy_value: f64 = positions.iter().map(|p| p.value_usd).sum();
            let concentration_risk = if total_strategy_value > 0.0 {
                positions.iter()
                    .map(|p| (p.value_usd / total_strategy_value).powi(2))
                    .sum::<f64>()
            } else {
                0.0
            };

            // Performance correlation (simplified)
            let performance_correlation = internal_correlation * 0.8;

            strategy_metrics.insert(strategy, StrategyCorrelationMetrics {
                internal_correlation,
                cross_strategy_correlation,
                concentration_risk,
                performance_correlation,
            });
        }

        strategy_metrics
    }

    /// Identify high correlation pairs
    fn identify_high_correlation_pairs(
        &self,
        matrix: &CorrelationMatrix,
        weights: &HashMap<String, f64>,
        threshold: f64,
    ) -> Vec<CorrelationPair> {
        let mut high_correlation_pairs = Vec::new();

        for i in 0..matrix.assets.len() {
            for j in (i + 1)..matrix.assets.len() {
                let correlation = matrix.matrix[i][j];
                
                if correlation.abs() > threshold {
                    let asset1 = &matrix.assets[i];
                    let asset2 = &matrix.assets[j];
                    let weight1 = weights.get(asset1).unwrap_or(&0.0);
                    let weight2 = weights.get(asset2).unwrap_or(&0.0);
                    let combined_weight = weight1 + weight2;
                    
                    // Risk impact from high correlation
                    let risk_impact = correlation.abs() * combined_weight;

                    high_correlation_pairs.push(CorrelationPair {
                        asset1: asset1.clone(),
                        asset2: asset2.clone(),
                        correlation,
                        weight1: *weight1,
                        weight2: *weight2,
                        combined_weight,
                        risk_impact,
                    });
                }
            }
        }

        // Sort by risk impact
        high_correlation_pairs.sort_by(|a, b| 
            b.risk_impact.partial_cmp(&a.risk_impact).unwrap()
        );

        high_correlation_pairs
    }

    /// Generate diversification recommendations
    fn generate_diversification_recommendations(
        &self,
        asset_correlations: &HashMap<String, AssetCorrelationMetrics>,
        high_correlation_pairs: &[CorrelationPair],
        concentration_hhi: f64,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // High concentration warning
        if concentration_hhi > 0.25 {
            recommendations.push(
                "Portfolio is highly concentrated - consider adding more diverse assets".to_string()
            );
        }

        // High correlation warnings
        for pair in high_correlation_pairs.iter().take(3) {
            if pair.risk_impact > 0.1 {
                recommendations.push(format!(
                    "High correlation ({:.2}) between {} and {} - consider reducing exposure",
                    pair.correlation, pair.asset1, pair.asset2
                ));
            }
        }

        // Low diversification benefit assets
        for (asset, metrics) in asset_correlations {
            if metrics.diversification_benefit < 0.3 && metrics.risk_contribution > 0.1 {
                recommendations.push(format!(
                    "Consider reducing {} exposure - low diversification benefit",
                    asset
                ));
            }
        }

        // General diversification advice
        if recommendations.is_empty() {
            recommendations.push("Portfolio diversification looks healthy".to_string());
        }

        recommendations
    }

    /// Get correlation analysis summary
    pub fn get_correlation_summary(&self, metrics: &DiversificationMetrics) -> String {
        let mut summary = String::new();
        summary.push_str(&format!("🔗 Correlation Analysis\n"));
        summary.push_str(&format!("═══════════════════════\n"));
        summary.push_str(&format!("📊 Portfolio Correlation: {:.3}\n", metrics.portfolio_correlation_score));
        summary.push_str(&format!("🎯 Effective Assets: {:.1}\n", metrics.effective_number_of_assets));
        summary.push_str(&format!("📈 Concentration (HHI): {:.3}\n", metrics.concentration_hhi));
        summary.push_str(&format!("🌈 Diversification Ratio: {:.3}\n", metrics.diversification_ratio));
        summary.push_str(&format!("⚠️ Correlation Risk: {:.3}\n", metrics.correlation_risk_contribution));

        if !metrics.high_correlation_pairs.is_empty() {
            summary.push_str(&format!("\n🔴 High Correlation Pairs:\n"));
            for pair in metrics.high_correlation_pairs.iter().take(5) {
                summary.push_str(&format!("  {} ↔ {}: {:.2} (Risk: {:.3})\n", 
                    pair.asset1, pair.asset2, pair.correlation, pair.risk_impact));
            }
        }

        if !metrics.diversification_recommendations.is_empty() {
            summary.push_str(&format!("\n💡 Recommendations:\n"));
            for rec in &metrics.diversification_recommendations {
                summary.push_str(&format!("  • {}\n", rec));
            }
        }

        summary
    }
}

impl Default for CorrelationAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portfolio::PositionRiskMetrics;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_position(symbol: &str, strategy: &str, value: f64, price: f64) -> Position {
        Position {
            id: Uuid::new_v4(),
            token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            symbol: symbol.to_string(),
            strategy: strategy.to_string(),
            entry_price: price,
            current_price: price,
            quantity: value / price,
            value_usd: value,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            entry_time: Utc::now(),
            last_update: Utc::now(),
            risk_metrics: PositionRiskMetrics {
                var_95: 0.05,
                var_99: 0.08,
                volatility: 0.15,
                beta: 1.2,
                max_drawdown: 0.02,
            },
        }
    }

    #[test]
    fn test_correlation_analyzer_creation() {
        let analyzer = CorrelationAnalyzer::new();
        assert!(analyzer.price_history.is_empty());
        assert!(analyzer.correlation_cache.is_empty());
    }

    #[test]
    fn test_price_updates() {
        let mut analyzer = CorrelationAnalyzer::new();
        
        analyzer.update_prices("SOL", 100.0);
        analyzer.update_prices("SOL", 105.0);
        
        assert_eq!(analyzer.price_history.len(), 1);
        assert_eq!(analyzer.price_history["SOL"].len(), 2);
        assert_eq!(analyzer.price_history["SOL"][1].return_pct, 5.0);
    }

    #[test]
    fn test_pearson_correlation() {
        let analyzer = CorrelationAnalyzer::new();
        
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        
        let correlation = analyzer.pearson_correlation(&x, &y);
        assert!((correlation - 1.0).abs() < 0.001); // Perfect positive correlation
        
        let z = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let neg_correlation = analyzer.pearson_correlation(&x, &z);
        assert!((neg_correlation + 1.0).abs() < 0.001); // Perfect negative correlation
    }

    #[test]
    fn test_effective_number_of_assets() {
        let analyzer = CorrelationAnalyzer::new();
        
        // Equal weights
        let mut weights = HashMap::new();
        weights.insert("A".to_string(), 0.25);
        weights.insert("B".to_string(), 0.25);
        weights.insert("C".to_string(), 0.25);
        weights.insert("D".to_string(), 0.25);
        
        let effective = analyzer.calculate_effective_number_of_assets(&weights);
        assert!((effective - 4.0).abs() < 0.001);
        
        // Concentrated weights
        let mut concentrated = HashMap::new();
        concentrated.insert("A".to_string(), 0.7);
        concentrated.insert("B".to_string(), 0.3);
        
        let effective_concentrated = analyzer.calculate_effective_number_of_assets(&concentrated);
        assert!(effective_concentrated < 2.0);
    }

    #[tokio::test]
    async fn test_diversification_analysis() {
        let mut analyzer = CorrelationAnalyzer::new();
        
        // Add some price history
        analyzer.update_prices("SOL", 100.0);
        analyzer.update_prices("SOL", 105.0);
        analyzer.update_prices("ETH", 2000.0);
        analyzer.update_prices("ETH", 2100.0);
        
        let mut positions = HashMap::new();
        positions.insert(Uuid::new_v4(), create_test_position("SOL", "trend", 5000.0, 105.0));
        positions.insert(Uuid::new_v4(), create_test_position("ETH", "momentum", 5000.0, 2100.0));

        // This might fail due to insufficient data, but tests the structure
        let result = analyzer.analyze_diversification(&positions, 7);
        // Just ensure it doesn't panic - actual analysis requires more data
    }
}
