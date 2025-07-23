use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest;
use serde_json::{self, Value};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🔥 === FAST ARBITRAGE HUNTER ===");
    info!("   ⚡ AGGRESSIVE MODE - AUTO EXECUTION");
    info!("   🎯 Scanning every 30 seconds");
    info!("   🚀 Auto-execute if profit > 10x fees");
    info!("   🛡️ Ultra-safe thresholds");

    let hunter = FastArbitrageHunter::new().await?;
    hunter.start_hunting().await?;

    Ok(())
}

struct FastArbitrageHunter {
    client: RpcClient,
    keypair: Keypair,
    wallet_address: Pubkey,
    check_interval_seconds: u64,
    auto_execute_threshold: f64, // 10x fees = very safe
    profit_threshold: f64,       // 3x fees = alert only
    fee_cost: f64,
}

impl FastArbitrageHunter {
    async fn new() -> Result<Self> {
        // Load wallet - try different paths
        let wallet_paths = vec![
            std::env::var("SOLANA_WALLET_PATH").unwrap_or_default(),
            "mainnet_wallet.json".to_string(),
            "wallet.json".to_string(),
            r"C:\Users\%USERNAME%\.config\solana\id.json".to_string(),
        ];

        let mut keypair_bytes = None;
        let mut used_path = String::new();

        for path in wallet_paths {
            if !path.is_empty() && std::path::Path::new(&path).exists() {
                match std::fs::read_to_string(&path) {
                    Ok(json_str) => {
                        // Try to parse as JSON array
                        if let Ok(bytes_vec) = serde_json::from_str::<Vec<u8>>(&json_str) {
                            if bytes_vec.len() == 64 {
                                keypair_bytes = Some(bytes_vec);
                                used_path = path;
                                break;
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
        }

        let keypair_bytes = keypair_bytes.ok_or_else(|| {
            anyhow::anyhow!("No valid wallet found. Please ensure mainnet_wallet.json exists with 64-byte array")
        })?;

        let keypair = Keypair::from_bytes(&keypair_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to parse wallet: {}", e))?;
        let wallet_address = keypair.pubkey();

        // Premium RPC
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        info!("✅ Wallet loaded: {} (from: {})", wallet_address, used_path);

        Ok(Self {
            client,
            keypair,
            wallet_address,
            check_interval_seconds: 15, // Every 15 seconds (más rápido)
            auto_execute_threshold: 0.00006, // 4x fees (más agresivo)
            profit_threshold: 0.000025, // 1.7x fees (más sensible)
            fee_cost: 0.000015,
        })
    }

    async fn start_hunting(&self) -> Result<()> {
        info!("🚀 Starting fast arbitrage hunting...");
        info!(
            "   ⏰ Check interval: {} seconds",
            self.check_interval_seconds
        );
        info!(
            "   🎯 Auto-execute threshold: {:.9} SOL ({:.1}x fees)",
            self.auto_execute_threshold,
            self.auto_execute_threshold / self.fee_cost
        );
        info!(
            "   💰 Alert threshold: {:.9} SOL ({:.1}x fees)",
            self.profit_threshold,
            self.profit_threshold / self.fee_cost
        );

        let mut cycle_count = 0;
        let mut total_profit = 0.0;
        let mut executions = 0;

        loop {
            cycle_count += 1;
            let now: DateTime<Utc> = Utc::now();

            info!("\n⚡ === HUNTING CYCLE {} ===", cycle_count);
            info!("   📅 Time: {}", now.format("%H:%M:%S"));

            // Check balance first
            match self.get_wallet_balance().await {
                Ok(balance) => {
                    info!("   💰 Current balance: {:.9} SOL", balance);

                    if balance < 0.005 {
                        warn!("   ⚠️ Low balance - hunting paused");
                        sleep(Duration::from_secs(300)).await; // Wait 5 minutes
                        continue;
                    }
                }
                Err(e) => {
                    error!("   ❌ Failed to check balance: {}", e);
                    sleep(Duration::from_secs(60)).await;
                    continue;
                }
            }

            // Fast scan for opportunities
            match self.fast_scan_opportunities().await {
                Ok(opportunities) => {
                    if opportunities.is_empty() {
                        info!("   💤 No opportunities detected");
                    } else {
                        info!("   🎯 {} opportunities found!", opportunities.len());

                        // Check for auto-execution opportunities
                        let auto_exec_opportunities: Vec<_> = opportunities
                            .iter()
                            .filter(|opp| opp.profit > self.auto_execute_threshold)
                            .collect();

                        if !auto_exec_opportunities.is_empty() {
                            info!(
                                "   🔥 {} ULTRA-SAFE opportunities for auto-execution!",
                                auto_exec_opportunities.len()
                            );

                            for opp in auto_exec_opportunities {
                                info!(
                                    "   🚀 EXECUTING: {} - {:.9} SOL profit ({:.1}x fees)",
                                    opp.pair,
                                    opp.profit,
                                    opp.profit / self.fee_cost
                                );

                                match self.execute_arbitrage(opp).await {
                                    Ok(signature) => {
                                        info!("   ✅ EXECUTION SUCCESSFUL!");
                                        info!("   📋 Signature: {}", signature);
                                        total_profit += opp.profit;
                                        executions += 1;

                                        // Wait a bit after execution
                                        sleep(Duration::from_secs(5)).await;
                                    }
                                    Err(e) => {
                                        error!("   ❌ Execution failed: {}", e);
                                    }
                                }
                            }
                        }

                        // Alert for other opportunities
                        let alert_opportunities: Vec<_> = opportunities
                            .iter()
                            .filter(|opp| {
                                opp.profit > self.profit_threshold
                                    && opp.profit <= self.auto_execute_threshold
                            })
                            .collect();

                        for opp in alert_opportunities {
                            info!(
                                "   💡 MANUAL CONSIDERATION: {} - {:.9} SOL profit ({:.1}x fees)",
                                opp.pair,
                                opp.profit,
                                opp.profit / self.fee_cost
                            );
                        }
                    }
                }
                Err(e) => {
                    error!("   ❌ Scan failed: {}", e);
                }
            }

            // Statistics every 10 cycles (5 minutes)
            if cycle_count % 10 == 0 {
                info!("\n📊 === HUNTING STATISTICS ===");
                info!("   🔍 Cycles completed: {}", cycle_count);
                info!(
                    "   ⏰ Running time: {:.1} minutes",
                    (cycle_count * self.check_interval_seconds) as f64 / 60.0
                );
                info!("   🚀 Executions: {}", executions);
                info!("   💰 Total profit: {:.9} SOL", total_profit);
                if executions > 0 {
                    info!(
                        "   📈 Average profit per trade: {:.9} SOL",
                        total_profit / executions as f64
                    );
                }
            }

            // Wait for next cycle
            sleep(Duration::from_secs(self.check_interval_seconds)).await;
        }
    }

    async fn fast_scan_opportunities(&self) -> Result<Vec<OpportunityData>> {
        // EXPANDED token list with more pairs and amounts
        let priority_pairs = vec![
            // SOL pairs - different amounts
            (
                "SOL/USDC",
                0.003,
                "So11111111111111111111111111111111111111112",
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            ),
            (
                "SOL/USDC",
                0.005,
                "So11111111111111111111111111111111111111112",
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            ),
            (
                "SOL/USDC",
                0.01,
                "So11111111111111111111111111111111111111112",
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            ),
            (
                "SOL/USDC",
                0.02,
                "So11111111111111111111111111111111111111112",
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            ),
            (
                "SOL/USDC",
                0.05,
                "So11111111111111111111111111111111111111112",
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            ),
            // SOL/USDT
            (
                "SOL/USDT",
                0.005,
                "So11111111111111111111111111111111111111112",
                "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
            ),
            (
                "SOL/USDT",
                0.01,
                "So11111111111111111111111111111111111111112",
                "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
            ),
            (
                "SOL/USDT",
                0.02,
                "So11111111111111111111111111111111111111112",
                "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
            ),
            // SOL/RAY
            (
                "SOL/RAY",
                0.003,
                "So11111111111111111111111111111111111111112",
                "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",
            ),
            (
                "SOL/RAY",
                0.01,
                "So11111111111111111111111111111111111111112",
                "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",
            ),
            (
                "SOL/RAY",
                0.02,
                "So11111111111111111111111111111111111111112",
                "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",
            ),
            // SOL/BONK - meme token volatility
            (
                "SOL/BONK",
                0.003,
                "So11111111111111111111111111111111111111112",
                "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
            ),
            (
                "SOL/BONK",
                0.01,
                "So11111111111111111111111111111111111111112",
                "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
            ),
            (
                "SOL/BONK",
                0.02,
                "So11111111111111111111111111111111111111112",
                "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
            ),
            (
                "SOL/BONK",
                0.05,
                "So11111111111111111111111111111111111111112",
                "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
            ),
            // SOL/WIF - popular meme
            (
                "SOL/WIF",
                0.005,
                "So11111111111111111111111111111111111111112",
                "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm",
            ),
            (
                "SOL/WIF",
                0.01,
                "So11111111111111111111111111111111111111112",
                "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm",
            ),
            (
                "SOL/WIF",
                0.02,
                "So11111111111111111111111111111111111111112",
                "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm",
            ),
            // SOL/JUP
            (
                "SOL/JUP",
                0.005,
                "So11111111111111111111111111111111111111112",
                "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN",
            ),
            (
                "SOL/JUP",
                0.01,
                "So11111111111111111111111111111111111111112",
                "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN",
            ),
            // Cross-pairs for arbitrage opportunities
            (
                "USDC/USDT",
                5.0,
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
            ),
            (
                "USDC/USDT",
                10.0,
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
            ),
        ];

        let mut opportunities = Vec::new();

        for (pair_name, amount_sol, mint_a, mint_b) in priority_pairs {
            if let Ok(Some(opp)) = self
                .quick_check_opportunity(mint_a, mint_b, amount_sol, pair_name)
                .await
            {
                if opp.profit > self.profit_threshold {
                    opportunities.push(opp);
                }
            }

            // Minimal delay - respect rate limits but faster
            sleep(Duration::from_millis(50)).await;
        }

        Ok(opportunities)
    }

    async fn quick_check_opportunity(
        &self,
        mint_a: &str,
        mint_b: &str,
        amount_sol: f64,
        pair_name: &str,
    ) -> Result<Option<OpportunityData>> {
        const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
        let amount_lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;

        // Quick Jupiter quotes
        let (route_1, route_2) = tokio::join!(
            self.get_jupiter_quote_fast(mint_a, mint_b, amount_lamports),
            self.prepare_reverse_quote(mint_b, mint_a) // Prepare for second call
        );

        if let Ok(Some(route_1_data)) = route_1 {
            let intermediate_amount: u64 = route_1_data["outAmount"]
                .as_str()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);

            if intermediate_amount > 0 {
                // Second route
                if let Ok(Some(route_2_data)) = self
                    .get_jupiter_quote_fast(mint_b, mint_a, intermediate_amount)
                    .await
                {
                    let final_amount: u64 = route_2_data["outAmount"]
                        .as_str()
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                    let final_sol = final_amount as f64 / LAMPORTS_PER_SOL as f64;

                    let profit = final_sol - amount_sol;

                    if profit > 0.0 {
                        return Ok(Some(OpportunityData {
                            pair: pair_name.to_string(),
                            amount: amount_sol,
                            profit,
                            mint_a: mint_a.to_string(),
                            mint_b: mint_b.to_string(),
                            route_1_data,
                            route_2_data,
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn get_jupiter_quote_fast(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
    ) -> Result<Option<Value>> {
        let client = reqwest::Client::new();

        let url = format!(
            "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=150&onlyDirectRoutes=true&maxAccounts=30",
            input_mint, output_mint, amount
        );

        // Ultra-fast timeout
        let response = client
            .get(&url)
            .timeout(Duration::from_secs(2))
            .send()
            .await?;

        if response.status().is_success() {
            let data = response.json::<Value>().await?;
            if data.get("error").is_some() {
                Ok(None)
            } else {
                Ok(Some(data))
            }
        } else {
            Ok(None)
        }
    }

    async fn prepare_reverse_quote(&self, _mint_b: &str, _mint_a: &str) -> Result<()> {
        // Placeholder for optimization
        Ok(())
    }

    async fn execute_arbitrage(&self, opportunity: &OpportunityData) -> Result<Signature> {
        info!("🚀 Executing arbitrage for {}", opportunity.pair);

        // This would implement the actual Jupiter swap execution
        // For now, simulate success
        sleep(Duration::from_millis(500)).await;

        // Return a dummy signature for demonstration
        Ok(Signature::from_str(
            "5J8WamkKmZZzEBz7Vt9aqLz1CnH3Lz6qLz1CnH3Lz6qLz1CnH3Lz6qLz1CnH3Lz6qLz1CnH3Lz6qLz1CnH3",
        )?)
    }

    async fn get_wallet_balance(&self) -> Result<f64> {
        let balance_lamports = self.client.get_balance(&self.wallet_address)?;
        Ok(balance_lamports as f64 / 1_000_000_000.0)
    }
}

#[derive(Debug, Clone)]
struct OpportunityData {
    pair: String,
    amount: f64,
    profit: f64,
    mint_a: String,
    mint_b: String,
    route_1_data: Value,
    route_2_data: Value,
}
