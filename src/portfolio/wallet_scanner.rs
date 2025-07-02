use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey};
use std::collections::HashMap;
use std::str::FromStr;
use tokio::time::{timeout, Duration};

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
    rpc_client: RpcClient,
    network: String,
}

impl WalletScanner {
    pub fn new(network: &str) -> Result<Self> {
        let rpc_url = match network {
            "devnet" => "https://api.devnet.solana.com",
            "mainnet" => "https://api.mainnet-beta.solana.com",
            _ => return Err(anyhow::anyhow!("Invalid network: {}", network)),
        };

        let rpc_client = RpcClient::new(rpc_url.to_string());

        Ok(Self {
            rpc_client,
            network: network.to_string(),
        })
    }

    pub async fn scan_wallet(&self, wallet_address: &str) -> Result<WalletBalance> {
        println!("🔍 Scanning REAL wallet: {}", wallet_address);

        // Parse wallet address
        let pubkey = Pubkey::from_str(wallet_address).context("Invalid wallet address")?;

        // Get SOL balance from the blockchain
        let lamports = timeout(Duration::from_secs(10), async {
            tokio::task::spawn_blocking({
                let url = self.rpc_client.url();
                let pubkey = pubkey.clone();
                move || {
                    let client = RpcClient::new(url);
                    client.get_balance(&pubkey)
                }
            })
            .await
            .map_err(|e| anyhow::anyhow!("Join error: {}", e))?
        })
        .await
        .context("Timeout getting SOL balance")?
        .context("RPC call failed")?;

        let sol_balance = lamports as f64 / LAMPORTS_PER_SOL as f64;

        // Get token balances from the blockchain
        let token_balances =
            match timeout(Duration::from_secs(15), self.get_token_balances(&pubkey)).await {
                Ok(Ok(balances)) => balances,
                Ok(Err(e)) => {
                    eprintln!("⚠️ Failed to get token balances: {}", e);
                    Vec::new()
                }
                Err(_) => {
                    eprintln!("⚠️ Timeout getting token balances");
                    Vec::new()
                }
            };

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

    async fn get_token_balances(&self, pubkey: &Pubkey) -> Result<Vec<TokenBalance>> {
        use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
        use solana_client::rpc_filter::{Memcmp, RpcFilterType};
        use solana_sdk::program_pack::Pack;

        // Get token accounts owned by this wallet
        let token_program = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")?;

        let filters = vec![RpcFilterType::Memcmp(Memcmp::new(
            32, // owner offset in token account
            solana_client::rpc_filter::MemcmpEncodedBytes::Base58(pubkey.to_string()),
        ))];

        let config = RpcProgramAccountsConfig {
            filters: Some(filters),
            account_config: RpcAccountInfoConfig {
                encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
                ..Default::default()
            },
            ..Default::default()
        };

        let accounts = self
            .rpc_client
            .get_program_accounts_with_config(&token_program, config)?;

        let mut token_balances = Vec::new();

        for (_, account) in accounts {
            if let Ok(token_account) = spl_token::state::Account::unpack(&account.data) {
                if token_account.amount > 0 {
                    // Get mint info to determine decimals
                    if let Ok(mint_account) = self.rpc_client.get_account(&token_account.mint) {
                        if let Ok(mint) = spl_token::state::Mint::unpack(&mint_account.data) {
                            let balance =
                                token_account.amount as f64 / 10_f64.powi(mint.decimals as i32);

                            token_balances.push(TokenBalance {
                                mint: token_account.mint.to_string(),
                                symbol: "UNKNOWN".to_string(), // Will be resolved by price feed
                                balance,
                                decimals: mint.decimals,
                                value_usd: None, // Will be calculated by price feed
                            });
                        }
                    }
                }
            }
        }

        Ok(token_balances)
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
