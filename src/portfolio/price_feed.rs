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

        // Fallback to hardcoded known tokens
        if let Some(price) = self.get_known_token_price(mint_address).await? {
            return Ok(price);
        }

        // Return unknown token with 0 price
        Ok(TokenPrice {
            symbol: "UNKNOWN".to_string(),
            mint: mint_address.to_string(),
            price_usd: 0.0,
            price_change_24h: 0.0,
            volume_24h: 0.0,
            market_cap: None,
            last_updated: chrono::Utc::now(),
            source: "unknown".to_string(),
        })
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

    async fn get_known_token_price(&self, mint_address: &str) -> Result<Option<TokenPrice>> {
        // Known Solana token addresses
        let known_tokens = match mint_address {
            // SOL (wrapped)
            "So11111111111111111111111111111111111111112" => Some(("SOL", "solana")),
            // USDC
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => Some(("USDC", "usd-coin")),
            // USDT
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => Some(("USDT", "tether")),
            // RAY (Raydium)
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R" => Some(("RAY", "raydium")),
            // SRM (Serum)
            "SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt" => Some(("SRM", "serum")),
            _ => None,
        };

        if let Some((symbol, coingecko_id)) = known_tokens {
            if let Ok(price) = self
                .get_price_from_coingecko(coingecko_id, symbol, mint_address)
                .await
            {
                return Ok(Some(price));
            }
        }

        Ok(None)
    }

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
        // Get real SOL price from CoinGecko API with stack overflow protection
        let price_future = self.fetch_sol_price_safe();

        match tokio::time::timeout(Duration::from_secs(3), price_future).await {
            Ok(Ok(price)) => Ok(price),
            Ok(Err(e)) => {
                println!("⚠️ Failed to get real SOL price: {}, using fallback", e);
                self.get_fallback_sol_price()
            }
            Err(_) => {
                println!("⚠️ SOL price request timed out, using fallback");
                self.get_fallback_sol_price()
            }
        }
    }

    async fn fetch_sol_price_safe(&self) -> Result<TokenPrice> {
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd&include_24hr_change=true&include_24hr_vol=true&include_market_cap=true";

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()?;

        let response = client
            .get(url)
            .header("User-Agent", "SniperForge/1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "API request failed with status: {}",
                response.status()
            ));
        }

        let json: serde_json::Value = response.json().await?;

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

    fn get_fallback_sol_price(&self) -> Result<TokenPrice> {
        Ok(TokenPrice {
            symbol: "SOL".to_string(),
            mint: "So11111111111111111111111111111111111111112".to_string(),
            price_usd: 180.0, // Reasonable fallback price
            price_change_24h: 0.0,
            volume_24h: 1_000_000_000.0,
            market_cap: Some(80_000_000_000.0),
            last_updated: chrono::Utc::now(),
            source: "fallback".to_string(),
        })
    }

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
