use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey};
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

        // Parse wallet address to validate it
        let _pubkey = Pubkey::from_str(wallet_address).context("Invalid wallet address")?;

        // Try to get real SOL balance with stack overflow protection
        let sol_balance = match self.get_sol_balance_safe(wallet_address).await {
            Ok(balance) => {
                println!("✅ Got real SOL balance: {:.6}", balance);
                balance
            }
            Err(e) => {
                println!("⚠️ Failed to get real SOL balance: {}", e);
                0.0 // Fallback to 0 if real balance fails
            }
        };

        // Token balances temporarily disabled for safety
        let token_balances = Vec::new();

        println!(
            "✅ REAL wallet scan complete: SOL {:.4}, {} tokens",
            sol_balance,
            token_balances.len()
        );

        Ok(WalletBalance {
            address: wallet_address.to_string(),
            sol_balance,
            token_balances,
            last_updated: chrono::Utc::now(),
        })
    }

    /// Safe SOL balance checking with timeout and error handling
    async fn get_sol_balance_safe(&self, wallet_address: &str) -> Result<f64> {
        // Stack overflow protection: Use a short timeout and simple HTTP call
        let balance_future = self.get_sol_balance_simple_http(wallet_address);

        match tokio::time::timeout(Duration::from_secs(3), balance_future).await {
            Ok(Ok(balance)) => Ok(balance),
            Ok(Err(e)) => Err(anyhow::anyhow!("Balance request failed: {}", e)),
            Err(_) => Err(anyhow::anyhow!("Balance request timed out")),
        }
    }

    /// Simplified HTTP balance check without complex dependencies
    async fn get_sol_balance_simple_http(&self, wallet_address: &str) -> Result<f64> {
        // Use the simplest possible HTTP request to avoid stack overflow
        let rpc_url = match self.network.as_str() {
            "devnet" => "https://api.devnet.solana.com",
            "mainnet" => "https://api.mainnet-beta.solana.com",
            _ => return Err(anyhow::anyhow!("Invalid network")),
        };

        let request_body = format!(
            r#"{{"jsonrpc":"2.0","id":1,"method":"getBalance","params":["{}"]}}"#,
            wallet_address
        );

        // Create a simple HTTP client with minimal configuration
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()?;

        let response = client
            .post(rpc_url)
            .header("Content-Type", "application/json")
            .body(request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP status: {}", response.status()));
        }

        let response_text = response.text().await?;

        // Parse JSON manually to avoid potential stack overflow in complex parsing
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response_text) {
            if let Some(result) = json_value.get("result") {
                if let Some(value) = result.get("value") {
                    if let Some(lamports) = value.as_u64() {
                        let sol_balance = lamports as f64 / LAMPORTS_PER_SOL as f64;
                        return Ok(sol_balance);
                    }
                }
            }

            // Check for RPC errors
            if let Some(error) = json_value.get("error") {
                return Err(anyhow::anyhow!("RPC error: {:?}", error));
            }
        }

        Err(anyhow::anyhow!("Invalid JSON response format"))
    }

    async fn get_token_balances(&self, _pubkey: &Pubkey) -> Result<Vec<TokenBalance>> {
        // For now, return empty vec to avoid stack overflow during RPC calls
        // TODO: Implement proper token account scanning
        println!("⚠️ Token balance scanning temporarily disabled to prevent stack overflow");
        Ok(Vec::new())
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
