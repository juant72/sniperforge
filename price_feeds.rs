// 💰 PRICE FEEDS MODULE - Professional multi-source price feed system
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};
use solana_sdk::pubkey::Pubkey;
use reqwest::Client;
use serde_json::Value;
use anyhow::{Result, anyhow};
use tracing::{info, warn, debug};
use crate::types::PriceData;

#[derive(Debug)]
pub struct ProfessionalPriceFeeds {
    client: Client,
    price_cache: HashMap<Pubkey, PriceData>,
    cache_duration: Duration,
    coingecko_calls: u64,
    jupiter_calls: u64,
    pyth_calls: u64,
    last_update: Instant,
}

impl ProfessionalPriceFeeds {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .user_agent("ProfessionalArbitrageBot/2.0")
                .build()
                .unwrap(),
            price_cache: HashMap::new(),
            cache_duration: Duration::from_secs(30),
            coingecko_calls: 0,
            jupiter_calls: 0,
            pyth_calls: 0,
            last_update: Instant::now() - Duration::from_secs(3600),
        }
    }

    pub async fn update_all_prices_professional(&mut self) -> Result<()> {
        info!("🔄 UPDATING PROFESSIONAL PRICE FEEDS");
        
        // Known Solana tokens
        let tokens = vec![
            ("So11111111111111111111111111111111111111112", "solana"),
            ("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "usd-coin"),
        ];

        for (mint, coingecko_id) in tokens {
            if let Ok(pubkey) = mint.parse::<Pubkey>() {
                if let Some(price_data) = self.fetch_multi_source_price(mint, coingecko_id).await {
                    self.price_cache.insert(pubkey, price_data);
                }
            }
        }

        self.last_update = Instant::now();
        info!("✅ PROFESSIONAL PRICE FEEDS UPDATED: {} tokens", self.price_cache.len());
        Ok(())
    }

    async fn fetch_multi_source_price(&mut self, mint: &str, coingecko_id: &str) -> Option<PriceData> {
        // Try CoinGecko first (most reliable)
        if let Some(cg_price) = self.fetch_coingecko_price(coingecko_id).await {
            return Some(PriceData {
                price: cg_price,
                timestamp: SystemTime::now(),
                source: "CoinGecko".to_string(),
                confidence: 0.95,
            });
        }

        // Try Jupiter API
        if let Some(jupiter_price) = self.fetch_jupiter_price(mint).await {
            return Some(PriceData {
                price: jupiter_price,
                timestamp: SystemTime::now(),
                source: "Jupiter".to_string(),
                confidence: 0.85,
            });
        }

        // Try Pyth Network
        if let Some(pyth_price) = self.fetch_pyth_price(mint).await {
            return Some(PriceData {
                price: pyth_price,
                timestamp: SystemTime::now(),
                source: "Pyth".to_string(),
                confidence: 0.90,
            });
        }

        warn!("⚠️ Could not fetch price for token: {}", mint);
        None
    }

    async fn fetch_coingecko_price(&mut self, coingecko_id: &str) -> Option<f64> {
        self.coingecko_calls += 1;
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
            coingecko_id
        );

        match self.client.get(&url).send().await {
            Ok(response) => {
                if let Ok(json) = response.json::<Value>().await {
                    if let Some(price) = json[coingecko_id]["usd"].as_f64() {
                        debug!("📈 CoinGecko price for {}: ${:.4}", coingecko_id, price);
                        return Some(price);
                    }
                }
            }
            Err(e) => {
                warn!("⚠️ CoinGecko API error: {}", e);
            }
        }
        None
    }

    async fn fetch_jupiter_price(&mut self, mint: &str) -> Option<f64> {
        self.jupiter_calls += 1;
        let url = format!(
            "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint=So11111111111111111111111111111111111111112&amount=1000000",
            mint
        );

        match self.client.get(&url).send().await {
            Ok(response) => {
                if let Ok(json) = response.json::<Value>().await {
                    if let Some(out_amount) = json["outAmount"].as_str() {
                        if let Ok(amount) = out_amount.parse::<u64>() {
                            let price_in_sol = amount as f64 / 1_000_000.0;
                            // Convert to USD (assuming SOL price from cache)
                            let sol_price = self.get_sol_price_from_cache().unwrap_or(200.0);
                            let price_usd = price_in_sol * sol_price;
                            debug!("📈 Jupiter price for {}: ${:.4}", mint, price_usd);
                            return Some(price_usd);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("⚠️ Jupiter API error: {}", e);
            }
        }
        None
    }

    async fn fetch_pyth_price(&mut self, mint: &str) -> Option<f64> {
        self.pyth_calls += 1;
        
        // Pyth price feed addresses (mainnet)
        let pyth_feed = match mint {
            "So11111111111111111111111111111111111111112" => "H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "Gnt27xtC473ZT2Mw5u8wZ68Z3gULkSTb5DuxJy7eJotD",
            _ => return None,
        };

        let url = format!("https://hermes.pyth.network/api/latest_price_feeds?ids[]={}", pyth_feed);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if let Ok(json) = response.json::<Value>().await {
                    if let Some(price_feeds) = json.as_array() {
                        if let Some(feed) = price_feeds.first() {
                            if let Some(price_str) = feed["price"]["price"].as_str() {
                                if let Ok(price_raw) = price_str.parse::<i64>() {
                                    if let Some(expo) = feed["price"]["expo"].as_i64() {
                                        let price = price_raw as f64 * 10_f64.powi(expo as i32);
                                        debug!("📈 Pyth price for {}: ${:.4}", mint, price);
                                        return Some(price);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("⚠️ Pyth API error: {}", e);
            }
        }
        None
    }

    fn get_sol_price_from_cache(&self) -> Option<f64> {
        if let Ok(sol_mint) = "So11111111111111111111111111111111111111112".parse::<Pubkey>() {
            self.price_cache.get(&sol_mint).map(|data| data.price)
        } else {
            None
        }
    }

    pub fn get_price_with_confidence(&self, token_mint: &Pubkey) -> Option<(f64, f64)> {
        if let Some(price_data) = self.price_cache.get(token_mint) {
            // Calculate volatility/confidence based on recency
            let age = price_data.timestamp.elapsed().unwrap_or(Duration::from_secs(0));
            let volatility = (age.as_secs() as f64 / 3600.0) * 0.01; // 1% per hour
            Some((price_data.price, volatility))
        } else {
            None
        }
    }

    pub fn are_prices_fresh(&self) -> bool {
        self.last_update.elapsed() < self.cache_duration
    }

    pub fn get_cached_price_count(&self) -> usize {
        self.price_cache.len()
    }

    pub fn get_api_stats(&self) -> (u64, u64, u64) {
        (self.coingecko_calls, self.jupiter_calls, self.pyth_calls)
    }
}
