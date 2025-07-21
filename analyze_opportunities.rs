use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as Base64Engine, Engine};
use chrono::{DateTime, Utc};
use futures::future::join_all;
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
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🔍 === ARBITRAGE OPPORTUNITY ANALYZER ===");
    info!("   📊 Testing current market conditions");
    info!("   🎯 Looking for ANY profitable opportunities");

    let analyzer = ArbitrageAnalyzer::new().await?;
    analyzer.analyze_current_market().await?;

    Ok(())
}

struct ArbitrageAnalyzer {
    http_client: reqwest::Client,
}

impl ArbitrageAnalyzer {
    async fn new() -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(20)
            .build()?;

        Ok(Self { http_client })
    }

    async fn analyze_current_market(&self) -> Result<()> {
        info!("🔍 Analyzing current arbitrage opportunities...");

        // Test high-volume pairs with different amounts
        let test_pairs = vec![
            // SOL pairs with different amounts
            (
                "SOL/USDC",
                0.001,
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
            // Stablecoin arbitrage
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
            (
                "USDC/USDT",
                50.0,
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
            ),
            (
                "USDC/USDT",
                100.0,
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
            ),
            // Meme tokens (high volatility)
            (
                "SOL/BONK",
                0.005,
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
        ];

        let mut profitable_opportunities = 0;
        let mut total_tested = 0;
        let fee_cost = 0.000015; // Standard fee

        for (pair_name, amount, mint_a, mint_b) in test_pairs {
            total_tested += 1;
            info!("\n📊 Testing {}: {} tokens", pair_name, amount);

            match self
                .test_arbitrage_opportunity(mint_a, mint_b, amount, pair_name)
                .await
            {
                Ok(Some((profit, route1_price, route2_price, slippage))) => {
                    let profit_multiple = profit / fee_cost;
                    let status = if profit > fee_cost * 3.0 {
                        "🚀 GOOD"
                    } else if profit > 0.0 {
                        "💡 SMALL"
                    } else {
                        "❌ LOSS"
                    };

                    info!(
                        "   {} Profit: {:.9} SOL ({:.1}x fees)",
                        status, profit, profit_multiple
                    );
                    info!(
                        "   📈 Route prices: {:.6} → {:.6}",
                        route1_price, route2_price
                    );
                    info!("   📉 Slippage: {:.4}%", slippage * 100.0);

                    if profit > 0.0 {
                        profitable_opportunities += 1;
                    }
                }
                Ok(None) => {
                    info!("   ❌ No valid route found");
                }
                Err(e) => {
                    error!("   ❌ Error testing {}: {}", pair_name, e);
                }
            }

            // Small delay between tests
            sleep(Duration::from_millis(200)).await;
        }

        info!("\n📊 === MARKET ANALYSIS SUMMARY ===");
        info!("   🔍 Total pairs tested: {}", total_tested);
        info!(
            "   💰 Profitable opportunities: {}",
            profitable_opportunities
        );
        info!(
            "   📈 Success rate: {:.1}%",
            (profitable_opportunities as f64 / total_tested as f64) * 100.0
        );

        if profitable_opportunities == 0 {
            info!("\n🔧 === RECOMMENDATIONS ===");
            info!("   📉 Market may be too efficient right now");
            info!("   ⏰ Try during high volatility periods");
            info!("   🎯 Consider lower profit thresholds");
            info!("   💎 Test with different token amounts");
            info!("   🌊 Wait for market movement");
        } else {
            info!(
                "\n✅ Found {} opportunities! Consider running the hunter.",
                profitable_opportunities
            );
        }

        Ok(())
    }

    async fn test_arbitrage_opportunity(
        &self,
        mint_a: &str,
        mint_b: &str,
        amount: f64,
        pair_name: &str,
    ) -> Result<Option<(f64, f64, f64, f64)>> {
        let is_sol_pair = mint_a == "So11111111111111111111111111111111111111112"
            || mint_b == "So11111111111111111111111111111111111111112";

        let (amount_units, divisor) = if is_sol_pair {
            const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
            (
                (amount * LAMPORTS_PER_SOL as f64) as u64,
                LAMPORTS_PER_SOL as f64,
            )
        } else {
            const UNITS_PER_TOKEN: u64 = 1_000_000;
            (
                (amount * UNITS_PER_TOKEN as f64) as u64,
                UNITS_PER_TOKEN as f64,
            )
        };

        // Route 1: A -> B
        if let Ok(Some(route_1_data)) = self.get_jupiter_quote(mint_a, mint_b, amount_units).await {
            let intermediate_amount: u64 = route_1_data["outAmount"]
                .as_str()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);
            let route1_price = intermediate_amount as f64 / amount_units as f64;

            if intermediate_amount > 0 {
                // Route 2: B -> A
                if let Ok(Some(route_2_data)) = self
                    .get_jupiter_quote(mint_b, mint_a, intermediate_amount)
                    .await
                {
                    let final_amount: u64 = route_2_data["outAmount"]
                        .as_str()
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                    let final_amount_f64 = final_amount as f64 / divisor;
                    let route2_price = final_amount as f64 / intermediate_amount as f64;

                    let profit_raw = final_amount_f64 - amount;
                    let profit_sol = if is_sol_pair {
                        profit_raw
                    } else {
                        profit_raw * 0.000056 // Approximate conversion to SOL
                    };

                    let slippage = (amount - final_amount_f64) / amount;

                    return Ok(Some((profit_sol, route1_price, route2_price, slippage)));
                }
            }
        }

        Ok(None)
    }

    async fn get_jupiter_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
    ) -> Result<Option<Value>> {
        // Use same aggressive settings as mega hunter
        let url = format!(
            "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=300&maxAccounts=64",
            input_mint, output_mint, amount
        );

        match self.http_client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Value>().await {
                        Ok(data) => {
                            if data.get("error").is_some() {
                                // Retry with even higher slippage
                                return self
                                    .get_jupiter_quote_fallback(input_mint, output_mint, amount)
                                    .await;
                            } else {
                                Ok(Some(data))
                            }
                        }
                        Err(_) => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }

    async fn get_jupiter_quote_fallback(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
    ) -> Result<Option<Value>> {
        let url = format!(
            "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=500",
            input_mint, output_mint, amount
        );

        match self.http_client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Value>().await {
                        Ok(data) => {
                            if data.get("error").is_some() {
                                Ok(None)
                            } else {
                                Ok(Some(data))
                            }
                        }
                        Err(_) => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }
}
