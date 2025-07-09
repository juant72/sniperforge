use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, debug, error};

use crate::shared::network_config::NetworkConfig;
use crate::shared::jupiter_api::Jupiter;
use crate::strategies::arbitrage::{ArbitrageStrategy, ArbitrageOpportunity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageDetectorConfig {
    pub min_profit_threshold: f64,
    pub max_slippage: f64,
    pub detection_interval_ms: u64,
    pub execution_timeout_ms: u64,
    pub enabled: bool,
}

impl Default for ArbitrageDetectorConfig {
    fn default() -> Self {
        Self {
            min_profit_threshold: 0.005, // 0.5%
            max_slippage: 0.01, // 1%
            detection_interval_ms: 1000,
            execution_timeout_ms: 30000,
            enabled: true,
        }
    }
}

pub struct ArbitrageDetector {
    config: NetworkConfig,
    detector_config: ArbitrageDetectorConfig,
    jupiter: Jupiter,
    arbitrage_strategy: ArbitrageStrategy,
}

impl ArbitrageDetector {
    pub async fn new(config: NetworkConfig) -> Result<Self> {
        debug!("Initializing ArbitrageDetector with config: {}", config.network);
        
        let detector_config = config.arbitrage_settings
            .as_ref()
            .map(|settings| ArbitrageDetectorConfig {
                min_profit_threshold: settings.min_profit_threshold,
                max_slippage: settings.max_slippage,
                detection_interval_ms: settings.detection_interval_ms,
                execution_timeout_ms: settings.execution_timeout_ms,
                enabled: settings.enabled,
            })
            .unwrap_or_default();

        let jupiter = Jupiter::new(config.clone()).await?;
        let arbitrage_strategy = ArbitrageStrategy::new();

        Ok(Self {
            config,
            detector_config,
            jupiter,
            arbitrage_strategy,
        })
    }

    pub async fn detect_opportunities(
        &self,
        from_token: &str,
        to_token: &str,
        amount: f64,
    ) -> Result<Vec<ArbitrageOpportunity>> {
        if !self.detector_config.enabled {
            info!("Arbitrage detection is disabled");
            return Ok(vec![]);
        }

        debug!("Detecting arbitrage opportunities for {} -> {} (amount: {})", 
               from_token, to_token, amount);

        let mut opportunities = Vec::new();

        // Get Jupiter quote
        match self.jupiter.get_quote(from_token, to_token, amount).await {
            Ok(jupiter_quote) => {
                debug!("Jupiter quote received: {:.6} tokens out", jupiter_quote.out_amount);
                
                // For now, simulate other DEX prices (in a real implementation, 
                // you'd query actual DEX APIs like Orca, Raydium, etc.)
                let jupiter_price = jupiter_quote.out_amount / amount;
                
                // Simulate price variations to detect arbitrage opportunities
                let price_variations = vec![
                    ("Jupiter", jupiter_price),
                    ("Orca", jupiter_price * 1.002), // Slightly higher
                    ("Raydium", jupiter_price * 0.998), // Slightly lower
                ];

                // Find arbitrage opportunities
                for (i, (buy_dex, buy_price)) in price_variations.iter().enumerate() {
                    for (j, (sell_dex, sell_price)) in price_variations.iter().enumerate() {
                        if i != j && sell_price > buy_price {
                            let profit_percentage = (sell_price - buy_price) / buy_price;
                            
                            if profit_percentage >= self.detector_config.min_profit_threshold {
                                let profit_amount = amount * profit_percentage;
                                
                                let opportunity = ArbitrageOpportunity {
                                    buy_exchange: buy_dex.to_string(),
                                    sell_exchange: sell_dex.to_string(),
                                    buy_price: *buy_price,
                                    sell_price: *sell_price,
                                    profit_percentage,
                                    estimated_profit: profit_amount,
                                    liquidity_buy: jupiter_quote.out_amount, // Use Jupiter liquidity as estimate
                                    liquidity_sell: jupiter_quote.out_amount,
                                    confidence: if profit_percentage > 0.01 { 0.9 } else { 0.7 },
                                };

                                opportunities.push(opportunity);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to get Jupiter quote: {}", e);
                return Err(e);
            }
        }

        // Sort opportunities by profit percentage (highest first)
        opportunities.sort_by(|a, b| b.profit_percentage.partial_cmp(&a.profit_percentage).unwrap());

        info!("Found {} arbitrage opportunities", opportunities.len());
        for (i, opp) in opportunities.iter().enumerate() {
            debug!("Opportunity {}: {} -> {} (profit: {:.2}%)", 
                   i + 1, opp.buy_exchange, opp.sell_exchange, opp.profit_percentage * 100.0);
        }

        Ok(opportunities)
    }

    pub fn get_config(&self) -> &ArbitrageDetectorConfig {
        &self.detector_config
    }

    pub fn is_enabled(&self) -> bool {
        self.detector_config.enabled
    }

    pub async fn validate_opportunity(&self, opportunity: &ArbitrageOpportunity) -> Result<bool> {
        // Basic validation
        if opportunity.profit_percentage < self.detector_config.min_profit_threshold {
            return Ok(false);
        }

        if opportunity.confidence < 0.5 {
            return Ok(false);
        }

        // Additional validation can be added here
        // - Check token balances
        // - Verify DEX liquidity
        // - Validate price feeds are recent
        
        Ok(true)
    }
}
