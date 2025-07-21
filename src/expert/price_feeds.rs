use anyhow::{anyhow, Result};
use solana_sdk::pubkey::Pubkey;
use tracing::{info, warn, error};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time;
use crate::expert::constants::*;

// ===== EXPERT PRICE FEEDS SYSTEM =====
// Sistema robusto de feeds de precios desde múltiples fuentes

pub struct ExpertPriceFeeds {
    client: reqwest::Client,
    cache: HashMap<String, PriceData>,
    last_update: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct PriceData {
    pub price: f64,
    pub timestamp: std::time::Instant,
    pub source: String,
    pub confidence: f64, // 0.0 to 1.0
}

impl ExpertPriceFeeds {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            cache: HashMap::new(),
            last_update: std::time::Instant::now(),
        }
    }
    
    /// Get price for a token symbol with automatic caching
    pub async fn get_price(&mut self, symbol: &str) -> Option<PriceData> {
        // Check cache first
        if let Some(cached_price) = self.cache.get(symbol) {
            if cached_price.timestamp.elapsed() < Duration::from_secs(30) {
                return Some(cached_price.clone());
            }
        }
        
        // Fetch fresh price
        match self.fetch_price_multi_source(symbol).await {
            Ok(price_data) => {
                self.cache.insert(symbol.to_string(), price_data.clone());
                Some(price_data)
            }
            Err(e) => {
                warn!("Failed to fetch price for {}: {}", symbol, e);
                self.cache.get(symbol).cloned() // Return stale data if available
            }
        }
    }
    
    /// Fetch price from multiple sources for reliability
    async fn fetch_price_multi_source(&self, symbol: &str) -> Result<PriceData> {
        // Try CoinGecko first
        if let Ok(price) = self.fetch_coingecko_price(symbol).await {
            info!("✅ Got {} price from CoinGecko: ${:.4}", symbol, price);
            return Ok(PriceData {
                price,
                timestamp: std::time::Instant::now(),
                source: "CoinGecko".to_string(),
                confidence: self.get_source_confidence("CoinGecko"),
            });
        }
        
        // Try Jupiter as fallback
        if let Ok(price) = self.fetch_jupiter_price(symbol).await {
            info!("✅ Got {} price from Jupiter: ${:.4}", symbol, price);
            return Ok(PriceData {
                price,
                timestamp: std::time::Instant::now(),
                source: "Jupiter".to_string(),
                confidence: self.get_source_confidence("Jupiter"),
            });
        }
        
        // Try fallback as last resort
        if let Ok(price) = self.get_fallback_price(symbol).await {
            info!("✅ Got {} price from Fallback: ${:.4}", symbol, price);
            return Ok(PriceData {
                price,
                timestamp: std::time::Instant::now(),
                source: "Fallback".to_string(),
                confidence: self.get_source_confidence("Fallback"),
            });
        }
        
        Err(anyhow!("All price sources failed for {}", symbol))
    }
    
    /// Fetch price from CoinGecko API
    async fn fetch_coingecko_price(&self, symbol: &str) -> Result<f64> {
        let coingecko_id = match symbol {
            "SOL" => "solana",
            "USDC" => "usd-coin",
            "USDT" => "tether",
            "RAY" => "raydium",
            "ORCA" => "orca",
            _ => return Err(anyhow!("Unknown symbol for CoinGecko: {}", symbol)),
        };
        
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
            coingecko_id
        );
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("CoinGecko API error: {}", response.status()));
        }
        
        let json: serde_json::Value = response.json().await?;
        let price = json[coingecko_id]["usd"]
            .as_f64()
            .ok_or_else(|| anyhow!("Invalid price data from CoinGecko"))?;
        
        Ok(price)
    }
    
    /// Fetch price from Jupiter API
    async fn fetch_jupiter_price(&self, symbol: &str) -> Result<f64> {
        if symbol == "USDC" || symbol == "USDT" {
            return Ok(1.0); // Stablecoins
        }
        
        if symbol != "SOL" {
            return Err(anyhow!("Jupiter price only available for SOL"));
        }
        
        // Jupiter price endpoint (simplified)
        let url = "https://price.jup.ag/v4/price?ids=SOL";
        
        let response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Jupiter API error: {}", response.status()));
        }
        
        let json: serde_json::Value = response.json().await?;
        let price = json["data"]["SOL"]["price"]
            .as_f64()
            .ok_or_else(|| anyhow!("Invalid price data from Jupiter"))?;
        
        Ok(price)
    }
    
    /// Get fallback price when all APIs fail
    async fn get_fallback_price(&self, symbol: &str) -> Result<f64> {
        // Hardcoded fallback prices (update manually)
        let fallback_prices = vec![
            ("SOL", 200.0),
            ("USDC", 1.0),
            ("USDT", 1.0),
            ("RAY", 2.5),
            ("ORCA", 3.8),
        ];
        
        for (token, price) in fallback_prices {
            if token == symbol {
                warn!("⚠️  Using fallback price for {}: ${:.2}", symbol, price);
                return Ok(price);
            }
        }
        
        Err(anyhow!("No fallback price available for {}", symbol))
    }
    
    /// Get confidence score for different price sources
    fn get_source_confidence(&self, source: &str) -> f64 {
        match source {
            "CoinGecko" => 0.9,
            "Jupiter" => 0.85,
            "Fallback" => 0.5,
            _ => 0.3,
        }
    }
    
    /// Update all major token prices in batch
    pub async fn update_all_prices(&mut self) -> Result<()> {
        info!("🔄 Updating all major token prices...");
        
        let major_tokens = vec!["SOL", "USDC", "USDT", "RAY", "ORCA"];
        
        for symbol in major_tokens {
            if let Some(price_data) = self.get_price(symbol).await {
                info!("✅ {}: ${:.4} ({})", symbol, price_data.price, price_data.source);
            } else {
                warn!("❌ Failed to update price for {}", symbol);
            }
            
            // Rate limiting
            time::sleep(Duration::from_millis(100)).await;
        }
        
        self.last_update = std::time::Instant::now();
        info!("✅ Price update complete");
        Ok(())
    }
    
    /// Get SOL price specifically (most important)
    pub async fn get_sol_price(&mut self) -> Result<f64> {
        match self.get_price("SOL").await {
            Some(price_data) => Ok(price_data.price),
            None => Err(anyhow!("Failed to get SOL price from any source")),
        }
    }
    
    /// Check if prices are fresh
    pub fn are_prices_fresh(&self) -> bool {
        self.last_update.elapsed() < Duration::from_secs(60)
    }
    
    /// Get cached price count
    pub fn get_cached_price_count(&self) -> usize {
        self.cache.len()
    }
}

impl Default for ExpertPriceFeeds {
    fn default() -> Self {
        Self::new()
    }
}
