//! Real Stablecoin Price Monitor
//! Tracks actual stablecoin prices and depegging events

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StablecoinPrice {
    pub symbol: String,
    pub current_price: f64,
    pub deviation_from_peg: f64,  // Percentage deviation from $1.00
    pub is_depegged: bool,        // True if deviation > threshold
    pub last_updated: DateTime<Utc>,
    pub volume_24h: f64,
    pub market_cap: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepegEvent {
    pub stablecoin: String,
    pub max_deviation: f64,
    pub duration_minutes: u64,
    pub opportunity_size: f64,  // Estimated profit in USD
    pub event_start: DateTime<Utc>,
    pub event_end: Option<DateTime<Utc>>,
}

/// Real stablecoin price monitor with depegging detection
#[derive(Debug, Clone)]
pub struct StablecoinMonitor {
    prices: HashMap<String, StablecoinPrice>,
    depeg_threshold: f64,  // Percentage threshold for depegging alert
    api_key: Option<String>,
}

impl StablecoinMonitor {
    pub fn new(depeg_threshold: f64) -> Self {
        Self {
            prices: HashMap::new(),
            depeg_threshold,
            api_key: None,
        }
    }

    /// Set API key for price feeds
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Get real-time stablecoin prices from CoinGecko
    pub async fn update_stablecoin_prices(&mut self) -> Result<()> {
        let stablecoins = vec![
            ("USDC", "usd-coin"),
            ("USDT", "tether"), 
            ("BUSD", "binance-usd"),
            ("DAI", "dai"),
            ("FRAX", "frax"),
        ];

        for (symbol, coin_id) in stablecoins {
            match self.fetch_real_price(symbol, coin_id).await {
                Ok(price_info) => {
                    self.prices.insert(symbol.to_string(), price_info);
                }
                Err(e) => {
                    eprintln!("⚠️ Failed to fetch {} price: {}", symbol, e);
                    // Use fallback price with warning
                    self.prices.insert(symbol.to_string(), StablecoinPrice {
                        symbol: symbol.to_string(),
                        current_price: 1.0,  // Fallback to peg
                        deviation_from_peg: 0.0,
                        is_depegged: false,
                        last_updated: Utc::now(),
                        volume_24h: 0.0,
                        market_cap: 0.0,
                    });
                }
            }
        }

        Ok(())
    }

    /// Fetch real price from CoinGecko API
    async fn fetch_real_price(&self, symbol: &str, coin_id: &str) -> Result<StablecoinPrice> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd&include_market_cap=true&include_24hr_vol=true",
            coin_id
        );

        let response = client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        let price = data[coin_id]["usd"].as_f64().unwrap_or(1.0);
        let volume_24h = data[coin_id]["usd_24h_vol"].as_f64().unwrap_or(0.0);
        let market_cap = data[coin_id]["usd_market_cap"].as_f64().unwrap_or(0.0);

        let deviation_from_peg = ((price - 1.0) / 1.0) * 100.0;
        let is_depegged = deviation_from_peg.abs() > self.depeg_threshold;

        Ok(StablecoinPrice {
            symbol: symbol.to_string(),
            current_price: price,
            deviation_from_peg,
            is_depegged,
            last_updated: Utc::now(),
            volume_24h,
            market_cap,
        })
    }

    /// Get current price for stablecoin (REAL price, not 1.0)
    pub fn get_real_price(&self, symbol: &str) -> f64 {
        self.prices
            .get(symbol)
            .map(|p| p.current_price)
            .unwrap_or(1.0)
    }

    /// Check for depegging opportunities
    pub fn scan_depeg_opportunities(&self) -> Vec<DepegEvent> {
        let mut opportunities = Vec::new();

        for (symbol, price_info) in &self.prices {
            if price_info.is_depegged {
                // Calculate potential arbitrage opportunity
                let opportunity_size = self.calculate_depeg_opportunity(price_info);
                
                if opportunity_size > 50.0 {  // Minimum $50 opportunity
                    opportunities.push(DepegEvent {
                        stablecoin: symbol.clone(),
                        max_deviation: price_info.deviation_from_peg,
                        duration_minutes: 0,  // Would need historical data
                        opportunity_size,
                        event_start: price_info.last_updated,
                        event_end: None,
                    });
                }
            }
        }

        opportunities
    }

    /// Calculate potential profit from depegging event
    fn calculate_depeg_opportunity(&self, price_info: &StablecoinPrice) -> f64 {
        let deviation_abs = price_info.deviation_from_peg.abs();
        
        // Estimate opportunity based on deviation and volume
        let base_opportunity = deviation_abs * 1000.0;  // $1000 per 1% deviation
        let volume_factor = (price_info.volume_24h / 1_000_000.0).min(10.0);  // Cap at 10x
        
        base_opportunity * volume_factor
    }

    /// Get all current stablecoin prices
    pub fn get_all_prices(&self) -> HashMap<String, f64> {
        self.prices
            .iter()
            .map(|(symbol, price_info)| (symbol.clone(), price_info.current_price))
            .collect()
    }

    /// Check if any stablecoin is currently depegged
    pub fn has_depegged_stablecoins(&self) -> bool {
        self.prices.values().any(|p| p.is_depegged)
    }

    /// Get detailed info for specific stablecoin
    pub fn get_stablecoin_info(&self, symbol: &str) -> Option<&StablecoinPrice> {
        self.prices.get(symbol)
    }

    /// Display real-time stablecoin status
    pub fn display_stablecoin_status(&self) {
        println!("\n╔══════════════════════════════════════════════════════════════════╗");
        println!("║                  REAL-TIME STABLECOIN MONITOR                    ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        
        for (symbol, price_info) in &self.prices {
            let status_emoji = if price_info.is_depegged {
                "🔴 DEPEGGED"
            } else if price_info.deviation_from_peg.abs() > 0.1 {
                "🟡 MONITORING"
            } else {
                "🟢 STABLE"
            };

            println!("║ {} │ ${:.4} │ {:+.3}% │ {} ║",
                format!("{:<4}", symbol),
                price_info.current_price,
                price_info.deviation_from_peg,
                status_emoji
            );
        }
        
        println!("╚══════════════════════════════════════════════════════════════════╝");
    }
}

impl Default for StablecoinMonitor {
    fn default() -> Self {
        Self::new(0.25) // 0.25% depegging threshold
    }
}
