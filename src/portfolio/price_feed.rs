use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPrice {
    pub symbol: String,
    pub mint: String,
    pub price_usd: f64,
    pub price_change_24h: f64,
    pub volume_24h: f64,
    pub market_cap: Option<f64>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct PriceFeed {
    client: Client,
    network: String,
}

// DexScreener API response structures
#[derive(Debug, Deserialize)]
struct DexScreenerResponse {
    pairs: Vec<DexScreenerPair>,
}

#[derive(Debug, Deserialize)]
struct DexScreenerPair {
    #[serde(rename = "baseToken")]
    base_token: DexScreenerToken,
    #[serde(rename = "priceUsd")]
    price_usd: Option<String>,
    #[serde(rename = "priceChange")]
    price_change: Option<DexScreenerPriceChange>,
    #[serde(rename = "volume")]
    volume: Option<DexScreenerVolume>,
    #[serde(rename = "marketCap")]
    market_cap: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct DexScreenerToken {
    address: String,
    symbol: String,
}

#[derive(Debug, Deserialize)]
struct DexScreenerPriceChange {
    #[serde(rename = "h24")]
    h24: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct DexScreenerVolume {
    #[serde(rename = "h24")]
    h24: Option<f64>,
}

// CoinGecko API response structures
#[derive(Debug, Deserialize)]
struct CoinGeckoResponse {
    pub id: String,
    pub symbol: String,
    pub current_price: f64,
    pub price_change_percentage_24h: Option<f64>,
    pub total_volume: Option<f64>,
    pub market_cap: Option<f64>,
}

impl PriceFeed {
    pub fn new(network: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            network: network.to_string(),
        }
    }

    pub async fn get_token_price(&self, mint_address: &str) -> Result<TokenPrice> {
        // Try DexScreener first (better for Solana tokens)
        if let Ok(price) = self.get_price_from_dexscreener(mint_address).await {
            return Ok(price);
        }

        // NO FALLBACK - only real data
        eprintln!("❌ Failed to get real price for token: {}", mint_address);

        Err(anyhow::anyhow!(
            "No real price data available for token: {}",
            mint_address
        ))
    }

    async fn get_price_from_dexscreener(&self, mint_address: &str) -> Result<TokenPrice> {
        let url = format!(
            "https://api.dexscreener.com/latest/dex/tokens/{}",
            mint_address
        );

        let response = timeout(Duration::from_secs(10), async {
            self.client.get(&url).send().await
        })
        .await
        .context("Timeout getting price from DexScreener")?
        .context("Failed to send request to DexScreener")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "DexScreener API error: {}",
                response.status()
            ));
        }

        let dex_response: DexScreenerResponse = response
            .json()
            .await
            .context("Failed to parse DexScreener response")?;

        if let Some(pair) = dex_response.pairs.first() {
            let price_usd = pair
                .price_usd
                .as_ref()
                .and_then(|p| p.parse::<f64>().ok())
                .unwrap_or(0.0);

            let price_change_24h = pair
                .price_change
                .as_ref()
                .and_then(|pc| pc.h24)
                .unwrap_or(0.0);

            let volume_24h = pair.volume.as_ref().and_then(|v| v.h24).unwrap_or(0.0);

            Ok(TokenPrice {
                symbol: pair.base_token.symbol.clone(),
                mint: mint_address.to_string(),
                price_usd,
                price_change_24h,
                volume_24h,
                market_cap: pair.market_cap,
                last_updated: chrono::Utc::now(),
                source: "dexscreener".to_string(),
            })
        } else {
            Err(anyhow::anyhow!("No pairs found for token"))
        }
    }

    // REMOVED: get_known_token_price - only real API data allowed

    async fn get_price_from_coingecko(
        &self,
        coin_id: &str,
        symbol: &str,
        mint: &str,
    ) -> Result<TokenPrice> {
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true&include_24hr_vol=true&include_market_cap=true",
            coin_id
        );

        let response = timeout(Duration::from_secs(10), async {
            self.client.get(&url).send().await
        })
        .await
        .context("Timeout getting price from CoinGecko")?
        .context("Failed to send request to CoinGecko")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "CoinGecko API error: {}",
                response.status()
            ));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse CoinGecko response")?;

        if let Some(coin_data) = json.get(coin_id) {
            let price_usd = coin_data.get("usd").and_then(|v| v.as_f64()).unwrap_or(0.0);

            let price_change_24h = coin_data
                .get("usd_24h_change")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            let volume_24h = coin_data
                .get("usd_24h_vol")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            let market_cap = coin_data.get("usd_market_cap").and_then(|v| v.as_f64());

            Ok(TokenPrice {
                symbol: symbol.to_string(),
                mint: mint.to_string(),
                price_usd,
                price_change_24h,
                volume_24h,
                market_cap,
                last_updated: chrono::Utc::now(),
                source: "coingecko".to_string(),
            })
        } else {
            Err(anyhow::anyhow!(
                "Token data not found in CoinGecko response"
            ))
        }
    }
    pub async fn get_sol_price(&self) -> Result<TokenPrice> {
        println!("📡 Getting REAL SOL price from CoinGecko API...");

        // ONLY use real API data - no hardcoded prices
        self.fetch_sol_price_safe().await
    }

    async fn fetch_sol_price_safe(&self) -> Result<TokenPrice> {
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd&include_24hr_change=true&include_24hr_vol=true&include_market_cap=true";

        // Use ureq for blocking HTTP to avoid stack overflow
        let result = tokio::task::spawn_blocking(move || -> Result<TokenPrice> {
            let response = ureq::get(url)
                .header("User-Agent", "SniperForge/1.0")
                .call();

            match response {
                Ok(mut resp) => match resp.body_mut().read_to_string() {
                    Ok(response_text) => {
                        let json: serde_json::Value = serde_json::from_str(&response_text)?;

                        let solana_data = json
                            .get("solana")
                            .ok_or_else(|| anyhow::anyhow!("Solana data not found in response"))?;

                        let price_usd = solana_data
                            .get("usd")
                            .and_then(|v| v.as_f64())
                            .ok_or_else(|| anyhow::anyhow!("USD price not found"))?;

                        let price_change_24h = solana_data
                            .get("usd_24h_change")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);

                        let volume_24h = solana_data
                            .get("usd_24h_vol")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);

                        let market_cap = solana_data.get("usd_market_cap").and_then(|v| v.as_f64());

                        println!("✅ Got real SOL price from CoinGecko: ${:.2}", price_usd);

                        Ok(TokenPrice {
                            symbol: "SOL".to_string(),
                            mint: "So11111111111111111111111111111111111111112".to_string(),
                            price_usd,
                            price_change_24h,
                            volume_24h,
                            market_cap,
                            last_updated: chrono::Utc::now(),
                            source: "coingecko".to_string(),
                        })
                    }
                    Err(e) => Err(anyhow::anyhow!("Failed to read response: {}", e)),
                },
                Err(e) => Err(anyhow::anyhow!("HTTP request failed: {}", e)),
            }
        })
        .await;

        match result {
            Ok(result) => result,
            Err(e) => Err(anyhow::anyhow!("Task execution failed: {}", e)),
        }
    }

    // REMOVED: get_fallback_sol_price - only real data allowed

    pub async fn get_multiple_prices(
        &self,
        mint_addresses: &[String],
    ) -> HashMap<String, TokenPrice> {
        let mut prices = HashMap::new();

        // Get real prices from DexScreener API
        for mint in mint_addresses {
            match self.get_token_price(mint).await {
                Ok(price) => {
                    println!(
                        "✅ Got REAL price for {}: ${:.6}",
                        price.symbol, price.price_usd
                    );
                    prices.insert(mint.clone(), price);
                }
                Err(e) => {
                    eprintln!("❌ Failed to get price for {}: {}", mint, e);
                    // Don't insert anything for failed requests - let the caller handle missing data
                }
            }

            // Add small delay to avoid rate limiting
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        prices
    }

    pub fn get_network(&self) -> &str {
        &self.network
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_price_feed_creation() {
        let feed = PriceFeed::new("devnet");
        assert_eq!(feed.get_network(), "devnet");
    }

    #[tokio::test]
    async fn test_sol_price() {
        let feed = PriceFeed::new("mainnet");
        // This test might fail if internet is not available
        if let Ok(price) = feed.get_sol_price().await {
            assert_eq!(price.symbol, "SOL");
            assert!(price.price_usd > 0.0);
        }
    }
}
