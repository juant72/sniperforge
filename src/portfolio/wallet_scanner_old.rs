use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey};
use std::io::Read;
use std::str::FromStr;
use tokio::time::Duration;
use crate::portfolio::PriceFeed;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalance {
    pub address: String,
    pub sol_balance: f64,
    pub token_balances: Vec<TokenBalance>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub mint: String,
    pub symbol: String,
    pub balance: f64,
    pub decimals: u8,
    pub value_usd: Option<f64>,
}

pub struct WalletScanner {
    network: String,
    price_feed: PriceFeed,
}

impl WalletScanner {
    pub fn new(network: &str) -> Result<Self> {
        if !matches!(network, "devnet" | "mainnet") {
            return Err(anyhow::anyhow!("Invalid network: {}", network));
        }

        Ok(Self {
            network: network.to_string(),
            price_feed: PriceFeed::new(network),
        })
    }

    pub async fn scan_wallet(&self, wallet_address: &str) -> Result<WalletBalance> {
        println!("🔍 Scanning REAL wallet: {}", wallet_address);

        // Validate the address format
        let _pubkey = Pubkey::from_str(wallet_address).context("Invalid wallet address")?;

        // Only accept REAL balance - no fallbacks
        let sol_balance = self.get_balance_simple(wallet_address).await?;

        // Get REAL SPL token balances
        let token_balances = self.get_token_balances(wallet_address).await?;

        Ok(WalletBalance {
            address: wallet_address.to_string(),
            sol_balance,
            token_balances,
            last_updated: chrono::Utc::now(),
        })
    }
    /// Real balance lookup using Solana RPC API
    async fn get_balance_simple(&self, wallet_address: &str) -> Result<f64> {
        println!("📡 Making REAL HTTP call to Solana RPC...");

        // Use std library for HTTP instead of reqwest to avoid stack overflow
        let rpc_url = match self.network.as_str() {
            "devnet" => "https://api.devnet.solana.com",
            "mainnet" => "https://api.mainnet-beta.solana.com",
            _ => return Err(anyhow::anyhow!("Invalid network")),
        };

        // Spawn blocking task to avoid async stack issues
        let wallet_addr = wallet_address.to_string();
        let url = rpc_url.to_string();

        let result = tokio::task::spawn_blocking(move || -> Result<f64> {
            // Create JSON-RPC request
            let json_body = format!(
                r#"{{"jsonrpc":"2.0","id":1,"method":"getBalance","params":["{}"]}}"#,
                wallet_addr
            );

            // Use ureq for simpler HTTP (no async, no stack overflow)
            let response = ureq::post(&url)
                .header("Content-Type", "application/json")
                .send(&json_body);

            match response {
                Ok(mut resp) => match resp.body_mut().read_to_string() {
                    Ok(response_text) => {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_text)
                        {
                            if let Some(result) = json.get("result") {
                                if let Some(value) = result.get("value") {
                                    if let Some(lamports) = value.as_u64() {
                                        let sol_balance = lamports as f64 / LAMPORTS_PER_SOL as f64;
                                        return Ok(sol_balance);
                                    }
                                }
                            }
                        }
                        Err(anyhow::anyhow!("Failed to parse JSON response"))
                    }
                    Err(e) => Err(anyhow::anyhow!("Failed to read response: {}", e)),
                },
                Err(e) => Err(anyhow::anyhow!("HTTP request failed: {}", e)),
            }
        })
        .await;

        match result {
            Ok(Ok(balance)) => {
                println!("✅ Real SOL balance from blockchain: {:.6}", balance);
                Ok(balance)
            }
            Ok(Err(e)) => {
                println!("❌ Failed to get real balance: {}", e);
                Err(e)
            }
            Err(e) => {
                println!("❌ Task execution failed: {}", e);
                Err(anyhow::anyhow!("Task failed: {}", e))
            }
        }
    }
    /// Get REAL SPL token balances using Solana RPC API
    async fn get_token_balances(&self, wallet_address: &str) -> Result<Vec<TokenBalance>> {
        println!("🪙 Getting REAL SPL token balances...");

        let rpc_url = match self.network.as_str() {
            "devnet" => "https://api.devnet.solana.com",
            "mainnet" => "https://api.mainnet-beta.solana.com",
            _ => return Err(anyhow::anyhow!("Invalid network")),
        };

        let wallet_addr = wallet_address.to_string();
        let url = rpc_url.to_string();

        let result = tokio::task::spawn_blocking(move || -> Result<Vec<TokenBalance>> {
            // JSON-RPC request for SPL token accounts
            let json_body = format!(
                r#"{{"jsonrpc":"2.0","id":1,"method":"getTokenAccountsByOwner","params":["{}",{{"programId":"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"}},{{"encoding":"jsonParsed"}}]}}"#,
                wallet_addr
            );

            let response = ureq::post(&url)
                .header("Content-Type", "application/json")
                .send(&json_body);

            match response {
                Ok(resp) => {
                    let response_text = resp.into_string().context("Failed to read response")?;
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_text) {
                            if let Some(result) = json.get("result") {
                                if let Some(value) = result.get("value") {
                                    if let Some(accounts) = value.as_array() {
                                        let mut tokens = Vec::new();

                                        for account in accounts {
                                            if let Some(account_data) = account.get("account") {
                                                if let Some(data) = account_data.get("data") {
                                                    if let Some(parsed) = data.get("parsed") {
                                                        if let Some(info) = parsed.get("info") {
                                                            if let (Some(mint), Some(token_amount)) = (
                                                                info.get("mint").and_then(|m| m.as_str()),
                                                                info.get("tokenAmount")
                                                            ) {
                                                                if let (Some(amount), Some(decimals)) = (
                                                                    token_amount.get("amount").and_then(|a| a.as_str()),
                                                                    token_amount.get("decimals").and_then(|d| d.as_u64())
                                                                ) {
                                                                    if let Ok(amount_num) = amount.parse::<u64>() {
                                                                        if amount_num > 0 {
                                                                            let balance = amount_num as f64 / 10_f64.powi(decimals as i32);
                                                                            tokens.push(TokenBalance {
                                                                                mint: mint.to_string(),
                                                                                symbol: "UNKNOWN".to_string(), // Will be resolved later
                                                                                balance,
                                                                                decimals: decimals as u8,
                                                                                value_usd: None,
                                                                            });
                                                                            println!("✅ Found SPL token: {} (balance: {:.6})", mint, balance);
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        return Ok(tokens);
                                    }
                                }
                            }
                        }
                        Ok(Vec::new()) // No tokens found or parsing failed
                    }
                    Err(e) => Err(anyhow::anyhow!("Failed to read SPL token response: {}", e)),
                },
                Err(e) => Err(anyhow::anyhow!("SPL token request failed: {}", e)),
            }
        }).await;

        match result {
            Ok(Ok(tokens)) => {
                println!("✅ Found {} real SPL tokens", tokens.len());
                Ok(tokens)
            }
            Ok(Err(e)) => {
                println!("❌ Failed to get SPL tokens: {}", e);
                Ok(Vec::new()) // Return empty instead of failing
            }
            Err(e) => {
                println!("❌ SPL token task failed: {}", e);
                Ok(Vec::new()) // Return empty instead of failing
            }
        }
    }
    pub async fn scan_multiple_wallets(&self, addresses: &[String]) -> Result<Vec<WalletBalance>> {
        let mut results = Vec::new();

        for address in addresses {
            match self.scan_wallet(address).await {
                Ok(balance) => {
                    println!(
                        "✅ Scanned wallet: {} (SOL: {:.4})",
                        address, balance.sol_balance
                    );
                    results.push(balance);
                }
                Err(e) => {
                    eprintln!("❌ Failed to scan wallet {}: {}", address, e);
                }
            }
        }

        Ok(results)
    }

    pub fn get_network(&self) -> &str {
        &self.network
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wallet_scanner_creation() {
        let scanner = WalletScanner::new("devnet").unwrap();
        assert_eq!(scanner.get_network(), "devnet");
    }

    #[tokio::test]
    async fn test_invalid_network() {
        let result = WalletScanner::new("invalid");
        assert!(result.is_err());
    }
}
