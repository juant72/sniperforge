use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🤖 === SNIPERFORGE ARBITRAGE MONITOR ===");
    info!("   🛡️  SAFE MONITORING - NO TRANSACTIONS");
    info!("   🔍 Scanning for profitable opportunities");
    info!("   ⏰ Continuous monitoring mode");
    info!("   🎯 Alert when profit > 3x fees (0.000045+ SOL)");

    let monitor = ArbitrageMonitor::new();
    monitor.start_monitoring().await?;

    Ok(())
}

struct ArbitrageMonitor {
    check_interval_minutes: u64,
    profit_threshold: f64,
    fee_cost: f64,
}

impl ArbitrageMonitor {
    fn new() -> Self {
        Self {
            check_interval_minutes: 15, // Check every 15 minutes
            profit_threshold: 0.000045, // 3x fees for safety
            fee_cost: 0.000015,         // Fixed MainNet fees
        }
    }

    async fn start_monitoring(&self) -> Result<()> {
        info!("🚀 Starting arbitrage monitoring...");
        info!(
            "   ⏰ Check interval: {} minutes",
            self.check_interval_minutes
        );
        info!("   💰 Profit threshold: {:.9} SOL", self.profit_threshold);
        info!("   📊 Fee cost: {:.9} SOL", self.fee_cost);
        info!(
            "   🎯 Safety margin: {:.1}x fees",
            self.profit_threshold / self.fee_cost
        );

        let mut cycle_count = 0;
        let mut best_opportunity_ever: Option<OpportunityAlert> = None;

        loop {
            cycle_count += 1;
            let now: DateTime<Utc> = Utc::now();

            info!("\n🔍 === MONITORING CYCLE {} ===", cycle_count);
            info!("   📅 Time: {}", now.format("%Y-%m-%d %H:%M:%S UTC"));

            match self.scan_all_opportunities().await {
                Ok(opportunities) => {
                    if opportunities.is_empty() {
                        info!("   ❌ No profitable opportunities detected");
                        info!("   💡 Market conditions: Spreads below threshold");
                    } else {
                        info!(
                            "   🎯 {} PROFITABLE OPPORTUNITIES FOUND!",
                            opportunities.len()
                        );

                        for (i, opp) in opportunities.iter().enumerate() {
                            info!("   {}. {} ({} SOL)", i + 1, opp.pair, opp.amount);
                            info!("      💰 Profit: {:.9} SOL", opp.profit);
                            info!("      📈 ROI: {:.4}%", opp.roi);
                            info!("      🛡️  Safety: {:.1}x fees", opp.profit / self.fee_cost);
                        }

                        // Track best opportunity
                        let best_current = opportunities
                            .iter()
                            .max_by(|a, b| a.profit.partial_cmp(&b.profit).unwrap())
                            .unwrap();

                        if best_opportunity_ever.is_none()
                            || best_current.profit > best_opportunity_ever.as_ref().unwrap().profit
                        {
                            best_opportunity_ever = Some(best_current.clone());
                            info!("   🏆 NEW BEST OPPORTUNITY DETECTED!");
                            self.send_alert(best_current).await;
                        }

                        // Check for immediate execution worthy opportunities
                        let excellent_opportunities: Vec<_> = opportunities
                            .iter()
                            .filter(|opp| opp.profit > self.profit_threshold * 2.0) // 6x fees
                            .collect();

                        if !excellent_opportunities.is_empty() {
                            info!("   🚨 EXCELLENT OPPORTUNITIES (6x+ fees):");
                            for opp in excellent_opportunities {
                                info!("      🎯 {} - {:.9} SOL profit", opp.pair, opp.profit);
                                self.send_urgent_alert(opp).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("   ❌ Monitoring cycle failed: {}", e);
                }
            }

            // Summary stats
            if cycle_count % 4 == 0 {
                // Every hour (4 cycles of 15 min)
                info!("\n📊 === HOURLY SUMMARY ===");
                info!("   🔍 Cycles completed: {}", cycle_count);
                info!(
                    "   ⏰ Running time: {} hours",
                    (cycle_count * self.check_interval_minutes) / 60
                );

                if let Some(ref best) = best_opportunity_ever {
                    info!("   🏆 Best opportunity seen:");
                    info!("      💰 Pair: {}", best.pair);
                    info!("      💰 Profit: {:.9} SOL", best.profit);
                    info!("      📈 ROI: {:.4}%", best.roi);
                    info!("      📅 Time: {}", best.timestamp);
                } else {
                    info!("   ⚠️  No profitable opportunities detected yet");
                }
            }

            // Wait for next cycle
            info!(
                "   ⏰ Waiting {} minutes until next scan...",
                self.check_interval_minutes
            );
            sleep(Duration::from_secs(self.check_interval_minutes * 60)).await;
        }
    }

    async fn scan_all_opportunities(&self) -> Result<Vec<OpportunityAlert>> {
        let test_scenarios = vec![
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
                "SOL/RAY",
                0.005,
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
                "SOL/BONK",
                0.02,
                "So11111111111111111111111111111111111111112",
                "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
            ),
            (
                "SOL/BONK",
                0.03,
                "So11111111111111111111111111111111111111112",
                "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
            ),
        ];

        let mut profitable_opportunities = Vec::new();

        for (pair_name, amount_sol, mint_a, mint_b) in test_scenarios {
            match self
                .check_opportunity(mint_a, mint_b, amount_sol, pair_name)
                .await
            {
                Ok(Some(opportunity)) => {
                    if opportunity.profit > self.profit_threshold {
                        profitable_opportunities.push(opportunity);
                    }
                }
                Ok(None) => {} // No opportunity
                Err(e) => {
                    warn!("   ⚠️  Failed to check {}: {}", pair_name, e);
                }
            }

            // Rate limiting
            sleep(Duration::from_millis(200)).await;
        }

        Ok(profitable_opportunities)
    }

    async fn check_opportunity(
        &self,
        mint_a: &str,
        mint_b: &str,
        amount_sol: f64,
        pair_name: &str,
    ) -> Result<Option<OpportunityAlert>> {
        const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
        let amount_lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;

        // Route 1: A → B
        let route_1 = self
            .get_jupiter_quote(mint_a, mint_b, amount_lamports)
            .await?;

        if let Some(route_1_data) = route_1 {
            let intermediate_amount: u64 = route_1_data["outAmount"]
                .as_str()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);

            if intermediate_amount == 0 {
                return Ok(None);
            }

            sleep(Duration::from_millis(300)).await;

            // Route 2: B → A
            let route_2 = self
                .get_jupiter_quote(mint_b, mint_a, intermediate_amount)
                .await?;

            if let Some(route_2_data) = route_2 {
                let final_amount: u64 = route_2_data["outAmount"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
                let final_sol = final_amount as f64 / LAMPORTS_PER_SOL as f64;

                let profit = final_sol - amount_sol;
                let roi = (profit / amount_sol) * 100.0;

                return Ok(Some(OpportunityAlert {
                    pair: pair_name.to_string(),
                    amount: amount_sol,
                    profit,
                    roi,
                    timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                }));
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
        let client = reqwest::Client::new();

        let url = format!(
            "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=50",
            input_mint, output_mint, amount
        );

        match client.get(&url).send().await {
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

    async fn send_alert(&self, opportunity: &OpportunityAlert) {
        info!("🚨 === PROFIT ALERT ===");
        info!("   💰 Pair: {}", opportunity.pair);
        info!("   💰 Amount: {} SOL", opportunity.amount);
        info!("   📈 Profit: {:.9} SOL", opportunity.profit);
        info!("   📈 ROI: {:.4}%", opportunity.roi);
        info!("   📅 Time: {}", opportunity.timestamp);
        info!(
            "   🛡️  Safety margin: {:.1}x fees",
            opportunity.profit / self.fee_cost
        );
        info!("   ⚠️  CONSIDER MANUAL EXECUTION");
    }

    async fn send_urgent_alert(&self, opportunity: &OpportunityAlert) {
        info!("🔥 === URGENT PROFIT ALERT ===");
        info!("   🎯 EXCELLENT OPPORTUNITY DETECTED!");
        info!("   💰 Pair: {}", opportunity.pair);
        info!("   💰 Amount: {} SOL", opportunity.amount);
        info!("   📈 Profit: {:.9} SOL", opportunity.profit);
        info!("   📈 ROI: {:.4}%", opportunity.roi);
        info!(
            "   🛡️  Safety margin: {:.1}x fees (VERY SAFE)",
            opportunity.profit / self.fee_cost
        );
        info!("   🚀 RECOMMENDED FOR IMMEDIATE EXECUTION");
        info!("   📋 Run: cargo run --bin safe_arbitrage_test (verify)");
    }
}

#[derive(Debug, Clone)]
struct OpportunityAlert {
    pair: String,
    amount: f64,
    profit: f64,
    roi: f64,
    timestamp: String,
}
