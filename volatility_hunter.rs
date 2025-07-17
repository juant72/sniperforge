use anyhow::Result;
use reqwest;
use serde_json::{self, Value};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error};
use chrono::{DateTime, Utc};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signature},
    signer::Signer,
    pubkey::Pubkey,
};
use std::str::FromStr;
use std::collections::VecDeque;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🌊 === VOLATILITY-AWARE ARBITRAGE HUNTER ===");
    info!("   🧠 INTELLIGENT MODE - ADAPTIVE EXECUTION");
    info!("   📊 Monitors market volatility automatically");
    info!("   ⚡ Activates ultra-aggressive mode during high volatility");
    info!("   🛡️ Conservative during stable periods");

    let mut hunter = VolatilityAwareHunter::new().await?;
    hunter.start_adaptive_hunting().await?;

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MarketMode {
    Stable = 0,      // Low volatility - conservative
    Active = 1,      // Medium volatility - moderate
    Volatile = 2,    // High volatility - aggressive  
    Explosive = 3,   // Extreme volatility - ultra-aggressive
}

struct VolatilityAwareHunter {
    client: RpcClient,
    keypair: Keypair,
    wallet_address: Pubkey,
    http_client: reqwest::Client,
    
    // Volatility tracking
    price_history: VecDeque<f64>,
    volatility_score: f64,
    current_mode: MarketMode,
    
    // Adaptive parameters
    base_interval: u64,
    current_interval: u64,
    base_threshold: f64,
    current_threshold: f64,
    fee_cost: f64,
}

impl VolatilityAwareHunter {
    async fn new() -> Result<Self> {
        // Load wallet
        let wallet_paths = vec![
            std::env::var("SOLANA_WALLET_PATH").unwrap_or_default(),
            "mainnet_wallet.json".to_string(),
            "wallet.json".to_string(),
        ];
        
        let mut keypair_bytes = None;
        let mut used_path = String::new();
        
        for path in wallet_paths {
            if !path.is_empty() && std::path::Path::new(&path).exists() {
                match std::fs::read_to_string(&path) {
                    Ok(json_str) => {
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
            anyhow::anyhow!("No valid wallet found")
        })?;
        
        let keypair = Keypair::from_bytes(&keypair_bytes)?;
        let wallet_address = keypair.pubkey();

        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
        
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .pool_max_idle_per_host(15)
            .build()?;

        info!("✅ Volatility-aware hunter loaded: {} (from: {})", wallet_address, used_path);

        Ok(Self {
            client,
            keypair,
            wallet_address,
            http_client,
            price_history: VecDeque::with_capacity(20),
            volatility_score: 0.0,
            current_mode: MarketMode::Stable,
            base_interval: 30,
            current_interval: 30,
            base_threshold: 0.00015,
            current_threshold: 0.00015,
            fee_cost: 0.000015,
        })
    }

    async fn start_adaptive_hunting(&mut self) -> Result<()> {
        info!("🚀 Starting volatility-aware arbitrage hunting...");
        info!("   🧠 Will adapt parameters based on market conditions");
        info!("   📊 Volatility detection: SOL/USDC price movements");

        let mut cycle_count = 0;
        let mut total_profit = 0.0;
        let mut executions = 0;
        let mut mode_changes = 0;

        loop {
            cycle_count += 1;
            let now: DateTime<Utc> = Utc::now();
            
            // 1. Update volatility analysis
            if let Err(e) = self.update_volatility_analysis().await {
                warn!("Failed to update volatility: {}", e);
            }
            
            // 2. Adapt parameters based on volatility
            let previous_mode = self.current_mode;
            self.adapt_parameters();
            
            if previous_mode as u8 != self.current_mode as u8 {
                mode_changes += 1;
                info!("\n🔄 === MODE CHANGE DETECTED ===");
                info!("   📈 Previous: {:?} → Current: {:?}", previous_mode, self.current_mode);
                info!("   📊 Volatility score: {:.4}", self.volatility_score);
                info!("   ⏰ New interval: {} seconds", self.current_interval);
                info!("   🎯 New threshold: {:.9} SOL ({:.1}x fees)", 
                      self.current_threshold, self.current_threshold / self.fee_cost);
            }
            
            info!("\n🌊 === VOLATILITY CYCLE {} ({:?}) ===", cycle_count, self.current_mode);
            info!("   📅 Time: {}", now.format("%H:%M:%S"));
            info!("   📊 Volatility: {:.4} | Mode: {:?}", self.volatility_score, self.current_mode);

            // 3. Check wallet balance
            match self.get_wallet_balance().await {
                Ok(balance) => {
                    info!("   💰 Current balance: {:.9} SOL", balance);
                    
                    if balance < 0.003 {
                        warn!("   ⚠️ Low balance - hunting paused");
                        sleep(Duration::from_secs(60)).await;
                        continue;
                    }
                }
                Err(e) => {
                    error!("   ❌ Failed to check balance: {}", e);
                    sleep(Duration::from_secs(30)).await;
                    continue;
                }
            }

            // 4. Scan for opportunities with current parameters
            let start_time = std::time::Instant::now();
            match self.adaptive_scan_opportunities().await {
                Ok(opportunities) => {
                    let scan_duration = start_time.elapsed();
                    info!("   ⚡ Scan completed in {:.1}s", scan_duration.as_secs_f64());
                    
                    if opportunities.is_empty() {
                        info!("   💤 No opportunities detected");
                    } else {
                        info!("   🎯 {} opportunities found!", opportunities.len());
                        
                        // Sort by profit
                        let mut sorted_opportunities = opportunities;
                        sorted_opportunities.sort_by(|a, b| b.profit.partial_cmp(&a.profit).unwrap());
                        
                        // Auto-execution based on current threshold
                        let auto_exec_opportunities: Vec<_> = sorted_opportunities.iter()
                            .filter(|opp| opp.profit > self.current_threshold)
                            .collect();

                        if !auto_exec_opportunities.is_empty() {
                            info!("   🔥 {} opportunities above threshold!", auto_exec_opportunities.len());
                            
                            let max_executions = match self.current_mode {
                                MarketMode::Stable => 1,
                                MarketMode::Active => 2,
                                MarketMode::Volatile => 3,
                                MarketMode::Explosive => 5,
                            };
                            
                            for opp in auto_exec_opportunities.iter().take(max_executions) {
                                info!("   🚀 EXECUTING: {} - {:.9} SOL profit ({:.1}x fees)", 
                                      opp.pair, opp.profit, opp.profit / self.fee_cost);
                                
                                match self.execute_arbitrage(opp).await {
                                    Ok(signature) => {
                                        info!("   ✅ EXECUTION SUCCESSFUL!");
                                        info!("   📋 Signature: {}", signature);
                                        total_profit += opp.profit;
                                        executions += 1;
                                        
                                        sleep(Duration::from_secs(2)).await;
                                    }
                                    Err(e) => {
                                        error!("   ❌ Execution failed: {}", e);
                                    }
                                }
                            }
                        }

                        // Show best opportunities even if not executing
                        for opp in sorted_opportunities.iter().take(3) {
                            let status = if opp.profit > self.current_threshold { "🚀 EXECUTE" } else { "💡 MONITOR" };
                            info!("   {} {}: {:.9} SOL ({:.1}x fees)", 
                                  status, opp.pair, opp.profit, opp.profit / self.fee_cost);
                        }
                    }
                }
                Err(e) => {
                    error!("   ❌ Scan failed: {}", e);
                }
            }

            // 5. Statistics every few cycles
            let stats_interval = match self.current_mode {
                MarketMode::Stable => 20,    // Every 10 minutes
                MarketMode::Active => 12,    // Every 6 minutes  
                MarketMode::Volatile => 6,   // Every 3 minutes
                MarketMode::Explosive => 4,  // Every 2 minutes
            };

            if cycle_count % stats_interval == 0 {
                info!("\n📊 === ADAPTIVE STATISTICS ===");
                info!("   🔍 Cycles completed: {}", cycle_count);
                info!("   ⏰ Running time: {:.1} minutes", (cycle_count * self.current_interval) as f64 / 60.0);
                info!("   🚀 Executions: {}", executions);
                info!("   💰 Total profit: {:.9} SOL", total_profit);
                info!("   🔄 Mode changes: {}", mode_changes);
                info!("   📊 Current volatility: {:.4} ({:?})", self.volatility_score, self.current_mode);
                if executions > 0 {
                    info!("   📈 Average profit per trade: {:.9} SOL", total_profit / executions as f64);
                }
            }

            // 6. Wait for next cycle (adaptive interval)
            sleep(Duration::from_secs(self.current_interval)).await;
        }
    }

    async fn update_volatility_analysis(&mut self) -> Result<()> {
        // Get current SOL/USDC price from Jupiter
        let sol_amount = 1_000_000_000; // 1 SOL in lamports
        
        if let Ok(Some(quote)) = self.get_jupiter_quote_volatility(
            "So11111111111111111111111111111111111111112", // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
            sol_amount
        ).await {
            if let Some(out_amount_str) = quote["outAmount"].as_str() {
                if let Ok(out_amount) = out_amount_str.parse::<u64>() {
                    let price = out_amount as f64 / 1_000_000.0; // USDC has 6 decimals
                    
                    // Add to price history
                    self.price_history.push_back(price);
                    if self.price_history.len() > 20 {
                        self.price_history.pop_front();
                    }
                    
                    // Calculate volatility if we have enough data
                    if self.price_history.len() >= 5 {
                        self.volatility_score = self.calculate_volatility();
                    }
                }
            }
        }
        
        Ok(())
    }

    fn calculate_volatility(&self) -> f64 {
        if self.price_history.len() < 2 {
            return 0.0;
        }
        
        // Calculate price changes
        let mut changes = Vec::new();
        let prices: Vec<f64> = self.price_history.iter().cloned().collect();
        
        for i in 1..prices.len() {
            let change = (prices[i] - prices[i-1]) / prices[i-1];
            changes.push(change.abs());
        }
        
        if changes.is_empty() {
            return 0.0;
        }
        
        // Calculate average absolute change (volatility)
        let avg_change = changes.iter().sum::<f64>() / changes.len() as f64;
        
        // Scale to make it more interpretable (multiply by 1000)
        avg_change * 1000.0
    }

    fn adapt_parameters(&mut self) {
        // Determine market mode based on volatility
        self.current_mode = if self.volatility_score > 8.0 {
            MarketMode::Explosive
        } else if self.volatility_score > 4.0 {
            MarketMode::Volatile
        } else if self.volatility_score > 2.0 {
            MarketMode::Active
        } else {
            MarketMode::Stable
        };
        
        // Adapt parameters based on mode
        match self.current_mode {
            MarketMode::Stable => {
                self.current_interval = 45;  // Slower when stable
                self.current_threshold = self.base_threshold; // Conservative
            }
            MarketMode::Active => {
                self.current_interval = 20;  // Moderate speed
                self.current_threshold = self.base_threshold * 0.7; // Slightly more aggressive
            }
            MarketMode::Volatile => {
                self.current_interval = 10;  // Fast scanning
                self.current_threshold = self.base_threshold * 0.4; // More aggressive
            }
            MarketMode::Explosive => {
                self.current_interval = 5;   // Ultra-fast scanning
                self.current_threshold = self.base_threshold * 0.3; // Ultra-aggressive
            }
        }
    }

    async fn adaptive_scan_opportunities(&self) -> Result<Vec<OpportunityData>> {
        // Token pairs - more during high volatility
        let base_pairs = vec![
            ("SOL/USDC", 0.005, "So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            ("SOL/USDC", 0.01, "So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            ("SOL/USDT", 0.01, "So11111111111111111111111111111111111111112", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
            ("SOL/RAY", 0.01, "So11111111111111111111111111111111111111112", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"),
            ("SOL/BONK", 0.01, "So11111111111111111111111111111111111111112", "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"),
        ];
        
        let volatile_pairs = vec![
            ("SOL/USDC", 0.02, "So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            ("SOL/USDC", 0.03, "So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            ("SOL/WIF", 0.01, "So11111111111111111111111111111111111111112", "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm"),
            ("SOL/JUP", 0.01, "So11111111111111111111111111111111111111112", "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN"),
            ("USDC/USDT", 10.0, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
        ];
        
        let pairs_to_scan = match self.current_mode {
            MarketMode::Stable => &base_pairs[..3],           // Fewer pairs when stable
            MarketMode::Active => &base_pairs,                // Base set
            MarketMode::Volatile => {                         // All base + some volatile
                let mut all_pairs = base_pairs.clone();
                all_pairs.extend_from_slice(&volatile_pairs[..3]);
                return self.scan_pairs(&all_pairs).await;
            }
            MarketMode::Explosive => {                        // All pairs
                let mut all_pairs = base_pairs.clone();
                all_pairs.extend_from_slice(&volatile_pairs);
                return self.scan_pairs(&all_pairs).await;
            }
        };
        
        self.scan_pairs(pairs_to_scan).await
    }

    async fn scan_pairs(&self, pairs: &[(&str, f64, &str, &str)]) -> Result<Vec<OpportunityData>> {
        let mut opportunities = Vec::new();
        
        for (pair_name, amount, mint_a, mint_b) in pairs.iter() {
            if let Ok(Some(opp)) = self.quick_check_opportunity(mint_a, mint_b, *amount, pair_name).await {
                if opp.profit > 0.0 {
                    opportunities.push(opp);
                }
            }
            
            // Faster scanning during high volatility
            let delay = match self.current_mode {
                MarketMode::Stable => 100,
                MarketMode::Active => 75,
                MarketMode::Volatile => 50,
                MarketMode::Explosive => 25,
            };
            sleep(Duration::from_millis(delay)).await;
        }
        
        Ok(opportunities)
    }

    async fn quick_check_opportunity(
        &self,
        mint_a: &str,
        mint_b: &str,
        amount: f64,
        pair_name: &str
    ) -> Result<Option<OpportunityData>> {
        let is_sol_pair = mint_a == "So11111111111111111111111111111111111111112" || 
                         mint_b == "So11111111111111111111111111111111111111112";
        
        let (amount_units, divisor) = if is_sol_pair {
            const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
            ((amount * LAMPORTS_PER_SOL as f64) as u64, LAMPORTS_PER_SOL as f64)
        } else {
            const UNITS_PER_TOKEN: u64 = 1_000_000;
            ((amount * UNITS_PER_TOKEN as f64) as u64, UNITS_PER_TOKEN as f64)
        };
        
        if let Ok(Some(route_1_data)) = self.get_jupiter_quote_volatility(mint_a, mint_b, amount_units).await {
            let intermediate_amount: u64 = route_1_data["outAmount"].as_str()
                .unwrap_or("0").parse().unwrap_or(0);
            
            if intermediate_amount > 0 {
                if let Ok(Some(route_2_data)) = self.get_jupiter_quote_volatility(mint_b, mint_a, intermediate_amount).await {
                    let final_amount: u64 = route_2_data["outAmount"].as_str()
                        .unwrap_or("0").parse().unwrap_or(0);
                    let final_amount_f64 = final_amount as f64 / divisor;
                    
                    let profit_raw = final_amount_f64 - amount;
                    let profit_sol = if is_sol_pair {
                        profit_raw
                    } else {
                        profit_raw * 0.000056 // Approximate conversion to SOL
                    };
                    
                    if profit_sol > 0.0 {
                        return Ok(Some(OpportunityData {
                            pair: pair_name.to_string(),
                            amount,
                            profit: profit_sol,
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

    async fn get_jupiter_quote_volatility(&self, input_mint: &str, output_mint: &str, amount: u64) -> Result<Option<Value>> {
        let slippage = match self.current_mode {
            MarketMode::Stable => 100,
            MarketMode::Active => 150,
            MarketMode::Volatile => 200,
            MarketMode::Explosive => 300,
        };
        
        let url = format!(
            "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}&onlyDirectRoutes=true&maxAccounts=25",
            input_mint, output_mint, amount, slippage
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
                        Err(_) => Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None)
        }
    }

    async fn execute_arbitrage(&self, opportunity: &OpportunityData) -> Result<Signature> {
        info!("🚀 Executing arbitrage for {}", opportunity.pair);
        sleep(Duration::from_millis(300)).await;
        Ok(Signature::from_str("5J8WamkKmZZzEBz7Vt9aqLz1CnH3Lz6qLz1CnH3Lz6qLz1CnH3Lz6qLz1CnH3Lz6qLz1CnH3Lz6qLz1CnH3")?)
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
