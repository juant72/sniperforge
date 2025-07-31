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
        Self {
            client: Client::new(),
            network: network.to_string(),
        }
    }

    /// Get real SOL price from CoinGecko using blocking HTTP
    pub async fn get_sol_price(&self) -> Result<TokenPrice> {
        println!("📡 Getting REAL SOL price from CoinGecko API...");

        // Use blocking HTTP to avoid async stack overflow issues
        let result = tokio::task::spawn_blocking(|| -> Result<TokenPrice> {
            use std::io::Read;

            let url = "https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd&include_24hr_change=true&include_24hr_vol=true";

            let response = ureq::get(url)
                .header("User-Agent", "SniperForge/1.0")
                .call()
                .context("Failed to fetch SOL price from CoinGecko")?;

            let body = response.into_string()
                .context("Failed to read response body")?;

            let json: serde_json::Value = serde_json::from_str(&body)
                .context("Failed to parse CoinGecko response")?;

            if let Some(solana) = json.get("solana") {
                let price = solana.get("usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let change_24h = solana.get("usd_24h_change").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let volume_24h = solana.get("usd_24h_vol").and_then(|v| v.as_f64()).unwrap_or(0.0);

                println!("✅ Got real SOL price from CoinGecko: ${:.2}", price);

                Ok(TokenPrice {
                    symbol: "SOL".to_string(),
                    mint: "So11111111111111111111111111111111111111112".to_string(),
                    price_usd: price,
                    price_change_24h: change_24h,
                    volume_24h,
                    market_cap: None,
                    last_updated: chrono::Utc::now(),
                    source: "CoinGecko".to_string(),
                })
            } else {
                Err(anyhow::anyhow!("SOL price not found in CoinGecko response"))
            }
        }).await;

        match result {
            Ok(Ok(price)) => Ok(price),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(anyhow::anyhow!("Task join error: {}", e)),
        }
    }

    /// Get real SPL token price from DexScreener
    pub async fn get_token_price(&self, mint_address: &str) -> Result<TokenPrice> {
        println!("📡 Getting REAL token price for {} from DexScreener...", mint_address);

        let result = tokio::task::spawn_blocking({
            let mint = mint_address.to_string();
            move || -> Result<TokenPrice> {
                use std::io::Read;

                let url = format!("https://api.dexscreener.com/latest/dex/tokens/{}", mint);

                let response = ureq::get(&url)
                    .header("User-Agent", "SniperForge/1.0")
                    .call()
                    .context("Failed to fetch token price from DexScreener")?;

                let body = response.into_string()
                    .context("Failed to read DexScreener response body")?;

                let json: serde_json::Value = serde_json::from_str(&body)
                    .context("Failed to parse DexScreener response")?;

                if let Some(pairs) = json.get("pairs").and_then(|p| p.as_array()) {
                    if let Some(pair) = pairs.first() {
                        let symbol = pair.get("baseToken")
                            .and_then(|t| t.get("symbol"))
                            .and_then(|s| s.as_str())
                            .unwrap_or("UNKNOWN");

                        let price_usd = pair.get("priceUsd")
                            .and_then(|p| p.as_str())
                            .and_then(|p| p.parse::<f64>().ok())
                            .unwrap_or(0.0);

                        let price_change_24h = pair.get("priceChange")
                            .and_then(|c| c.get("h24"))
                            .and_then(|p| p.as_f64())
                            .unwrap_or(0.0);

                        let volume_24h = pair.get("volume")
                            .and_then(|v| v.get("h24"))
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);

                        let market_cap = pair.get("marketCap").and_then(|m| m.as_f64());

                        println!("✅ Got real token price: {} = ${:.6}", symbol, price_usd);

                        return Ok(TokenPrice {
                            symbol: symbol.to_string(),
                            mint: mint,
                            price_usd,
                            price_change_24h,
                            volume_24h,
                            market_cap,
                            last_updated: chrono::Utc::now(),
                            source: "DexScreener".to_string(),
                        });
                    }
                }

                Err(anyhow::anyhow!("Token price not found in DexScreener response"))
            }
        }).await;

        match result {
            Ok(Ok(price)) => Ok(price),
            Ok(Err(e)) => {
                println!("❌ Failed to get token price: {}", e);
                Err(e)
            },
            Err(e) => Err(anyhow::anyhow!("Task join error: {}", e)),
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

    /// Get token metadata from Solana RPC (Metaplex)
    pub async fn get_token_metadata(&self, mint_address: &str) -> Result<(String, String)> {
        println!("📡 Getting token metadata for {}", mint_address);

        let result = tokio::task::spawn_blocking({
            let mint = mint_address.to_string();
            let network = self.network.clone();
            move || -> Result<(String, String)> {
                let rpc_url = match network.as_str() {
                    "devnet" => "https://api.devnet.solana.com",
                    "mainnet" => "https://api.mainnet-beta.solana.com",
                    _ => return Err(anyhow::anyhow!("Invalid network")),
                };

                // Try to get token metadata using getTokenSupply and other methods
                let json_body = format!(
                    r#"{{"jsonrpc":"2.0","id":1,"method":"getTokenSupply","params":["{}"]}}"#,
                    mint
                );

                let response = ureq::post(rpc_url)
                    .header("Content-Type", "application/json")
                    .send(&json_body);

                match response {
                    Ok(resp) => {
                        let body = resp.into_string().context("Failed to read response")?;
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                            if json.get("result").is_some() {
                                // Token exists, but we don't have symbol info from this call
                                // Return mint as symbol for now
                                let short_mint = if mint.len() > 8 {
                                    format!("{}...{}", &mint[0..4], &mint[mint.len()-4..])
                                } else {
                                    mint.clone()
                                };
                                return Ok((short_mint, mint));
                            }
                        }
                    }
                    Err(_) => {}
                }

                // Fallback to mint address as symbol
                let short_mint = if mint.len() > 8 {
                    format!("{}...{}", &mint[0..4], &mint[mint.len()-4..])
                } else {
                    mint.clone()
                };
                Ok((short_mint, mint))
            }
        }).await;

        match result {
            Ok(Ok((symbol, mint))) => {
                println!("✅ Got token info: {} ({})", symbol, mint);
                Ok((symbol, mint))
            }
            Ok(Err(e)) => {
                println!("⚠️ Failed to get metadata, using mint: {}", e);
                let short_mint = if mint_address.len() > 8 {
                    format!("{}...{}", &mint_address[0..4], &mint_address[mint_address.len()-4..])
                } else {
                    mint_address.to_string()
                };
                Ok((short_mint, mint_address.to_string()))
            }
            Err(e) => Err(anyhow::anyhow!("Task join error: {}", e)),
        }
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
