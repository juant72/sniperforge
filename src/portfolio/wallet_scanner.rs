use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey};
use std::io::Read;
use std::str::FromStr;
use tokio::time::Duration;

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
}

impl WalletScanner {
    pub fn new(network: &str) -> Result<Self> {
        if !matches!(network, "devnet" | "mainnet") {
            return Err(anyhow::anyhow!("Invalid network: {}", network));
        }

        Ok(Self {
            network: network.to_string(),
        })
    }

    pub async fn scan_wallet(&self, wallet_address: &str) -> Result<WalletBalance> {
        println!("🔍 Scanning REAL wallet: {}", wallet_address);

        // Validate the address format
        let _pubkey = Pubkey::from_str(wallet_address).context("Invalid wallet address")?;

        // Try to get real balance with minimal HTTP call
        let sol_balance = match self.get_balance_simple(wallet_address).await {
            Ok(balance) => {
                println!("✅ Got real SOL balance: {:.6}", balance);
                balance
            }
            Err(e) => {
                println!("⚠️ Failed to get balance: {}, using 0.0", e);
                0.0
            }
        };

        Ok(WalletBalance {
            address: wallet_address.to_string(),
            sol_balance,
            token_balances: Vec::new(),
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
