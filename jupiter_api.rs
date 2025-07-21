// 🚀 JUPITER API MODULE - Real Jupiter aggregator integration
use reqwest::Client;
use anyhow::{Result, anyhow};
use tracing::{info, warn};
use serde_json::Value;
use crate::types::JupiterQuote;

pub struct JupiterAPI {
    client: Client,
    base_url: String,
}

impl JupiterAPI {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .user_agent("ProfessionalArbitrageBot/2.0")
                .build()
                .unwrap(),
            base_url: "https://quote-api.jup.ag/v6".to_string(),
        }
    }

    /// Get real quote from Jupiter for actual arbitrage calculation
    pub async fn get_real_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
    ) -> Result<JupiterQuote> {
        info!("🚀 REQUESTING JUPITER QUOTE: {} -> {} ({})", 
              input_mint[..8].to_uppercase(),
              output_mint[..8].to_uppercase(),
              amount);

        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps=50",
            self.base_url, input_mint, output_mint, amount
        );

        match self.client.get(&url).send().await {
            Ok(response) => {
                let status = response.status();
                let text = response.text().await?;
                
                if status.is_success() {
                    match self.parse_jupiter_response(&text) {
                        Ok(quote) => {
                            info!("✅ Jupiter quote successful: {} -> {} (impact: {:.4}%)",
                                  amount, quote.out_amount, quote.price_impact_pct);
                            Ok(quote)
                        }
                        Err(e) => {
                            warn!("⚠️ Failed to parse Jupiter response: {}", e);
                            Err(anyhow!("Parse error: {}", e))
                        }
                    }
                } else {
                    warn!("⚠️ Jupiter API returned error: {} - {}", status, text);
                    Err(anyhow!("API error: {}", status))
                }
            }
            Err(e) => {
                warn!("⚠️ Jupiter API request failed: {}", e);
                Err(anyhow!("Request failed: {}", e))
            }
        }
    }

    fn parse_jupiter_response(&self, response: &str) -> Result<JupiterQuote> {
        let json: Value = serde_json::from_str(response)?;

        let out_amount = json["outAmount"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing outAmount"))?
            .parse::<u64>()?;

        let price_impact_pct = json["priceImpactPct"]
            .as_str()
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or(0.0);

        let route_plan = self.extract_route_plan(response);

        Ok(JupiterQuote {
            out_amount,
            price_impact_pct,
            route_plan,
        })
    }

    fn extract_route_plan(&self, response: &str) -> Vec<String> {
        if let Ok(json) = serde_json::from_str::<Value>(response) {
            if let Some(route_plan) = json["routePlan"].as_array() {
                return route_plan
                    .iter()
                    .filter_map(|step| {
                        step["swapInfo"]["ammKey"]
                            .as_str()
                            .map(|s| s[..8].to_uppercase())
                    })
                    .collect();
            }
        }
        vec!["DIRECT".to_string()]
    }

    pub async fn get_token_list(&self) -> Result<Vec<Value>> {
        info!("📋 FETCHING JUPITER TOKEN LIST");
        
        let url = format!("{}/tokens", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let tokens: Vec<Value> = response.json().await?;
                    info!("✅ Retrieved {} tokens from Jupiter", tokens.len());
                    Ok(tokens)
                } else {
                    Err(anyhow!("Failed to fetch token list: {}", response.status()))
                }
            }
            Err(e) => {
                warn!("⚠️ Jupiter token list request failed: {}", e);
                Err(anyhow!("Request failed: {}", e))
            }
        }
    }

    pub async fn get_swap_quote_with_fees(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u64,
    ) -> Result<JupiterQuote> {
        info!("🔄 REQUESTING JUPITER SWAP QUOTE WITH FEES");
        
        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}&onlyDirectRoutes=false&asLegacyTransaction=false",
            self.base_url, input_mint, output_mint, amount, slippage_bps
        );

        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let text = response.text().await?;
            self.parse_jupiter_response(&text)
        } else {
            Err(anyhow!("Jupiter API error: {}", response.status()))
        }
    }
}
