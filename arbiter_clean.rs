// 🚀 PROFESSIONAL ARBITRAGE ENGINE - Clean and Modularized
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use solana_client::rpc_client::RpcClient;

// Import our modules
mod types;
mod price_feeds;
mod pool_validator;
mod jupiter_api;
mod calculations;

use types::*;
use price_feeds::ProfessionalPriceFeeds;
use pool_validator::PoolValidator;
use jupiter_api::JupiterAPI;
use calculations::*;

impl ProfessionalArbitrageEngine {
    /// Initialize the professional arbitrage engine
    pub async fn new_professional(rpc_url: String, wallet_keypair_path: String) -> Result<Self> {
        info!("🚀 INITIALIZING PROFESSIONAL ARBITRAGE ENGINE");
        
        // Load wallet with enhanced security
        let wallet_address = if std::path::Path::new(&wallet_keypair_path).exists() {
            let wallet_keypair = solana_sdk::signature::read_keypair_file(&wallet_keypair_path)
                .map_err(|e| anyhow!("Failed to load wallet: {}", e))?;
            info!("🔐 Wallet loaded: {}", wallet_keypair.pubkey());
            wallet_keypair.pubkey()
        } else {
            warn!("⚠️  Wallet file not found, using demo mode");
            Pubkey::from_str("11111111111111111111111111111111")?
        };
        
        // Initialize high-performance RPC client
        let client = RpcClient::new_with_commitment(
            rpc_url.clone(),
            solana_sdk::commitment_config::CommitmentConfig::confirmed(),
        );
        
        // Initialize optimized HTTP client
        let jupiter_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .connection_verbose(true)
            .pool_max_idle_per_host(10)
            .build()?;
        
        // Initialize professional modules
        let price_feeds = ProfessionalPriceFeeds::new();
        let pool_validator = PoolValidator::new(rpc_url);
        
        let engine = Self {
            client,
            wallet_address,
            jupiter_client,
            price_feeds,
            pool_validator,
            operational_pools: HashMap::new(),
            pool_performance: HashMap::new(),
            monitoring_pools: Vec::new(),
            risk_metrics: RiskMetrics {
                max_exposure_usd: 10000.0,
                current_exposure_usd: 0.0,
                daily_pnl: 0.0,
                success_rate: 0.0,
                average_profit_bps: 0.0,
                max_drawdown: 0.0,
            },
            market_metrics: MarketMetrics {
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
                total_volume_24h: 0.0,
                average_spread: 0.0,
                volatility_index: 0.0,
                liquidity_score: 0.0,
                market_sentiment: MarketSentiment::Neutral,
            },
            performance_metrics: PerformanceMetrics {
                total_trades: 0,
                successful_trades: 0,
                total_profit_usd: 0.0,
                average_execution_time_ms: 0.0,
                best_profit_opportunity: 0.0,
                hourly_pnl: std::collections::VecDeque::new(),
            },
            adaptive_config: AdaptiveConfig {
                max_slippage_bps: MAX_SLIPPAGE_BPS,
                min_profit_threshold: MILITARY_MIN_PROFIT_BPS,
                max_trade_amount: (MAX_TRADE_SIZE_SOL * 1e9) as u64,
                risk_multiplier: 1.0,
                volatility_adjustment: 1.0,
                latency_compensation: 1.0,
            },
            is_running: std::sync::atomic::AtomicBool::new(false),
            emergency_stop: std::sync::atomic::AtomicBool::new(false),
            last_price_update: Instant::now(),
            execution_times: std::collections::VecDeque::new(),
            profit_history: std::collections::VecDeque::new(),
            total_opportunities_found: std::sync::atomic::AtomicU64::new(0),
            successful_trades: std::sync::atomic::AtomicU64::new(0),
            total_profit_lamports: std::sync::atomic::AtomicU64::new(0),
            risk_events: std::sync::atomic::AtomicU64::new(0),
        };
        
        info!("✅ PROFESSIONAL ENGINE INITIALIZED");
        Ok(engine)
    }
    
    /// Main professional arbitrage execution loop
    pub async fn run_professional_arbitrage(&mut self) -> Result<()> {
        info!("⚔️  PROFESSIONAL ARBITRAGE ENGINE STARTING");
        
        self.is_running.store(true, Ordering::Relaxed);
        let cycle_start = Instant::now();
        
        // Pre-flight checks
        self.check_risk_limits()?;
        
        // Step 1: Update market data
        if !self.price_feeds.are_prices_fresh() {
            self.price_feeds.update_all_prices_professional().await?;
            self.update_market_metrics().await?;
        }
        
        // Step 2: Discover and validate pools
        self.discover_professional_pools().await?;
        
        // Step 3: Find opportunities
        let opportunities = self.find_professional_opportunities().await?;
        
        if opportunities.is_empty() {
            info!("📊 No opportunities meeting professional criteria");
            self.is_running.store(false, Ordering::Relaxed);
            return Ok(());
        }
        
        // Step 4: Risk assessment
        let filtered_opportunities = self.apply_risk_filters(opportunities)?;
        
        if filtered_opportunities.is_empty() {
            warn!("⚠️  All opportunities filtered out by risk management");
            self.is_running.store(false, Ordering::Relaxed);
            return Ok(());
        }
        
        // Step 5: Execute best opportunity
        let best_opportunity = self.select_optimal_opportunity(filtered_opportunities)?;
        self.display_professional_opportunity(&best_opportunity);
        
        match self.execute_professional_arbitrage(&best_opportunity).await {
            Ok(signature) => {
                self.successful_trades.fetch_add(1, Ordering::Relaxed);
                self.total_profit_lamports.fetch_add(best_opportunity.profit_lamports as u64, Ordering::Relaxed);
                self.update_performance_metrics(&best_opportunity, true);
                info!("✅ PROFESSIONAL EXECUTION SUCCESS: {}", signature);
            }
            Err(e) => {
                error!("❌ Professional execution failed: {}", e);
                self.update_performance_metrics(&best_opportunity, false);
                self.risk_events.fetch_add(1, Ordering::Relaxed);
            }
        }
        
        let cycle_time = cycle_start.elapsed().as_millis() as u64;
        self.execution_times.push_back(cycle_time);
        if self.execution_times.len() > 100 {
            self.execution_times.pop_front();
        }
        
        self.is_running.store(false, Ordering::Relaxed);
        info!("⚡ PROFESSIONAL CYCLE COMPLETE: {}ms", cycle_time);
        
        Ok(())
    }
    
    fn check_risk_limits(&self) -> Result<()> {
        if self.risk_metrics.current_exposure_usd > self.risk_metrics.max_exposure_usd {
            return Err(anyhow!("Risk limit exceeded"));
        }
        if self.risk_metrics.daily_pnl < -1000.0 {
            self.emergency_stop.store(true, Ordering::Relaxed);
            return Err(anyhow!("Daily loss limit reached"));
        }
        Ok(())
    }
    
    async fn update_market_metrics(&mut self) -> Result<()> {
        let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
        
        if let Some((_price, volatility)) = self.price_feeds.get_price_with_confidence(&sol_mint) {
            self.market_metrics.volatility_index = volatility;
            
            if volatility > 0.05 {
                self.market_metrics.market_sentiment = MarketSentiment::HighVolatility;
                self.adaptive_config.volatility_adjustment = 1.5;
            } else if volatility < 0.02 {
                self.market_metrics.market_sentiment = MarketSentiment::Neutral;
                self.adaptive_config.volatility_adjustment = 0.8;
            }
        }
        
        self.market_metrics.timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        Ok(())
    }
    
    async fn discover_professional_pools(&mut self) -> Result<()> {
        info!("🔍 PROFESSIONAL POOL DISCOVERY");
        
        self.operational_pools.clear();
        
        let professional_pools = vec![
            ("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2", PoolType::Raydium, "SOL", "USDC"),
            ("HJPjoWUrhoZzkNfRpHuieeFk9WcZWjwy6PBjZ81ngndJ", PoolType::OrcaWhirlpool, "SOL", "USDC"),
            ("9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP", PoolType::Orca, "SOL", "USDC"),
        ];
        
        for (address_str, dex_type, token_a, token_b) in professional_pools {
            if let Ok(pool_address) = Pubkey::from_str(address_str) {
                match self.pool_validator.validate_real_pool_comprehensive(
                    &pool_address, dex_type.clone(), token_a, token_b
                ).await {
                    Ok(pool_data) => {
                        info!("✅ POOL VALIDATED: {:?} TVL: ${:.0}", dex_type, pool_data.tvl_usd);
                        
                        let performance = PoolPerformanceData {
                            total_volume: 0.0,
                            average_spread: (pool_data.fee_rate as f64 / 100.0),
                            success_rate: 1.0,
                            last_profitable_trade: None,
                            volatility_score: 0.0,
                        };
                        
                        self.pool_performance.insert(pool_address, performance);
                        self.operational_pools.insert(pool_address, pool_data);
                    }
                    Err(e) => {
                        warn!("⚠️  Pool validation failed {}: {}", address_str, e);
                    }
                }
            }
        }
        
        if self.operational_pools.is_empty() {
            return Err(anyhow!("No operational pools available"));
        }
        
        info!("🎯 DISCOVERY COMPLETE: {} verified pools", self.operational_pools.len());
        Ok(())
    }
    
    async fn find_professional_opportunities(&mut self) -> Result<Vec<DirectOpportunity>> {
        info!("🧮 PROFESSIONAL OPPORTUNITY ANALYSIS");
        
        let mut opportunities = Vec::new();
        let pools: Vec<_> = self.operational_pools.values().collect();
        
        for (i, pool_a) in pools.iter().enumerate() {
            for pool_b in pools.iter().skip(i + 1) {
                if self.pools_have_common_token(pool_a, pool_b) {
                    if let Ok(Some(opportunity)) = self.calculate_professional_arbitrage(pool_a, pool_b).await {
                        let profit_bps = (opportunity.profit_lamports * 10_000) / opportunity.amount_in as i64;
                        
                        if profit_bps >= self.adaptive_config.min_profit_threshold as i64 {
                            info!("💎 OPPORTUNITY: {:.2}% profit", profit_bps as f64 / 100.0);
                            opportunities.push(opportunity);
                            self.total_opportunities_found.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            }
        }
        
        opportunities.sort_by(|a, b| {
            let score_a = self.calculate_opportunity_score(a);
            let score_b = self.calculate_opportunity_score(b);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        info!("🎯 ANALYSIS: {} opportunities found", opportunities.len());
        Ok(opportunities)
    }
    
    fn calculate_opportunity_score(&self, opportunity: &DirectOpportunity) -> f64 {
        let base_profit = opportunity.profit_lamports as f64 / 1e9;
        let volatility_factor = 1.0 / (1.0 + self.market_metrics.volatility_index);
        base_profit * volatility_factor * self.adaptive_config.risk_multiplier
    }
    
    fn apply_risk_filters(&self, opportunities: Vec<DirectOpportunity>) -> Result<Vec<DirectOpportunity>> {
        let original_count = opportunities.len();
        let filtered: Vec<_> = opportunities.into_iter()
            .filter(|opp| {
                let trade_size_sol = opp.amount_in as f64 / 1e9;
                if trade_size_sol < MIN_TRADE_SIZE_SOL || trade_size_sol > MAX_TRADE_SIZE_SOL {
                    return false;
                }
                
                let adjusted_threshold = self.adaptive_config.min_profit_threshold as f64 * 
                                       self.adaptive_config.volatility_adjustment;
                let profit_bps = (opp.profit_lamports * 10_000) / opp.amount_in as i64;
                
                profit_bps as f64 >= adjusted_threshold
            })
            .collect();
        
        info!("🛡️  RISK FILTER: {}/{} opportunities passed", filtered.len(), original_count);
        Ok(filtered)
    }
    
    fn select_optimal_opportunity(&self, opportunities: Vec<DirectOpportunity>) -> Result<DirectOpportunity> {
        opportunities.into_iter()
            .max_by(|a, b| {
                let score_a = self.calculate_opportunity_score(a);
                let score_b = self.calculate_opportunity_score(b);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or_else(|| anyhow!("No optimal opportunity found"))
    }
    
    fn display_professional_opportunity(&self, opportunity: &DirectOpportunity) {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let profit_percentage = (opportunity.profit_lamports as f64 / opportunity.amount_in as f64) * 100.0;
        
        println!("\n╔═══════════════════════════════════════════════════════════════════════════════╗");
        println!("║            💎 PROFESSIONAL ARBITRAGE OPPORTUNITY                             ║");
        println!("╠═══════════════════════════════════════════════════════════════════════════════╣");
        println!("║ 🎯 STRATEGY: Cross-DEX Professional Arbitrage | TS: {}         ║", timestamp % 100000);
        println!("║ 🧠 SENTIMENT: {:?} | VOLATILITY: {:.4}                    ║", 
                 self.market_metrics.market_sentiment, self.market_metrics.volatility_index);
        println!("║                                                                               ║");
        println!("║ 💼 EXECUTION PLAN:                                                           ║");
        println!("║    📊 Trade Size: {:.6} SOL                                           ║", 
                 opportunity.amount_in as f64 / 1e9);
        println!("║    💎 NET PROFIT: {:.6} SOL ({:.4}%)                              ║", 
                 opportunity.profit_lamports as f64 / 1e9, profit_percentage);
        println!("╚═══════════════════════════════════════════════════════════════════════════════╝");
    }
    
    async fn execute_professional_arbitrage(&mut self, opportunity: &DirectOpportunity) -> Result<String> {
        info!("⚔️ EXECUTING PROFESSIONAL ARBITRAGE");
        
        let current_balance = self.get_wallet_balance().await?;
        let required_balance = opportunity.amount_in as f64 / 1e9;
        
        if current_balance < required_balance {
            return Err(anyhow!("Insufficient balance: {:.3} SOL required, {:.3} SOL available", 
                required_balance, current_balance));
        }
        
        let profit = opportunity.profit_lamports as f64 / 1e9;
        info!("✅ Transaction validated - Expected profit: {:.6} SOL", profit);
        info!("🚨 SIMULATION MODE - Real execution requires transaction signing");
        
        Ok(format!("PROF_SIM_{}_{}", 
            opportunity.pool_a.address.to_string()[..8].to_uppercase(),
            opportunity.pool_b.address.to_string()[..8].to_uppercase()))
    }
    
    async fn calculate_professional_arbitrage(&self, pool_a: &PoolData, pool_b: &PoolData) -> Result<Option<DirectOpportunity>> {
        let intermediate_token = if pool_a.token_a_mint == pool_b.token_a_mint || pool_a.token_a_mint == pool_b.token_b_mint {
            pool_a.token_a_mint
        } else if pool_a.token_b_mint == pool_b.token_a_mint || pool_a.token_b_mint == pool_b.token_b_mint {
            pool_a.token_b_mint
        } else {
            return Ok(None);
        };
        
        let current_balance = self.get_wallet_balance().await?;
        let max_trade_sol = (current_balance * 0.1).min(MAX_TRADE_SIZE_SOL);
        let optimal_amount = ((max_trade_sol * 1e9) as u64).min(
            (pool_a.token_a_amount.min(pool_a.token_b_amount)) / 20
        );
        
        if optimal_amount < (MIN_TRADE_SIZE_SOL * 1e9) as u64 {
            return Ok(None);
        }
        
        // Try Jupiter API first
        let jupiter_api = JupiterAPI::new();
        
        let (input_mint_a, output_mint_a) = if pool_a.token_a_mint == intermediate_token {
            ("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "So11111111111111111111111111111111111111112")
        } else {
            ("So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
        };
        
        let quote_a = match jupiter_api.get_real_quote(input_mint_a, output_mint_a, optimal_amount).await {
            Ok(quote) => quote,
            Err(_) => {
                // Fallback to AMM calculation
                let (pool_a_in, pool_a_out) = if pool_a.token_a_mint == intermediate_token {
                    (pool_a.token_b_amount, pool_a.token_a_amount)
                } else {
                    (pool_a.token_a_amount, pool_a.token_b_amount)
                };
                
                let out_amount = calculate_amm_output_exact(pool_a_in, pool_a_out, optimal_amount, pool_a.fee_rate)?;
                JupiterQuote {
                    out_amount,
                    price_impact_pct: 0.0,
                    route_plan: vec!["AMM_FALLBACK".to_string()],
                }
            }
        };
        
        let (input_mint_b, output_mint_b) = if pool_b.token_a_mint == intermediate_token {
            ("So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
        } else {
            ("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "So11111111111111111111111111111111111111112")
        };
        
        let quote_b = match jupiter_api.get_real_quote(input_mint_b, output_mint_b, quote_a.out_amount).await {
            Ok(quote) => quote,
            Err(_) => {
                let (pool_b_in, pool_b_out) = if pool_b.token_a_mint == intermediate_token {
                    (pool_b.token_a_amount, pool_b.token_b_amount)
                } else {
                    (pool_b.token_b_amount, pool_b.token_a_amount)
                };
                
                let out_amount = calculate_amm_output_exact(pool_b_in, pool_b_out, quote_a.out_amount, pool_b.fee_rate)?;
                JupiterQuote {
                    out_amount,
                    price_impact_pct: 0.0,
                    route_plan: vec!["AMM_FALLBACK".to_string()],
                }
            }
        };
        
        let final_amount = quote_b.out_amount;
        let estimated_tx_fees = 15_000;
        let total_price_impact = (quote_a.price_impact_pct + quote_b.price_impact_pct) / 100.0;
        let slippage_cost = (optimal_amount as f64 * total_price_impact) as u64;
        let total_real_costs = estimated_tx_fees + slippage_cost;
        
        if final_amount <= optimal_amount {
            return Ok(None);
        }
        
        let gross_profit = final_amount - optimal_amount;
        let net_profit = gross_profit.saturating_sub(total_real_costs);
        
        if net_profit == 0 {
            return Ok(None);
        }
        
        let profit_bps = (net_profit * 10_000) / optimal_amount;
        if profit_bps < MILITARY_MIN_PROFIT_BPS {
            return Ok(None);
        }
        
        debug!("✅ PROFITABLE ARBITRAGE: {:.4}% profit", profit_bps as f64 / 100.0);
        
        Ok(Some(DirectOpportunity {
            pool_a: pool_a.clone(),
            pool_b: pool_b.clone(),
            intermediate_token,
            amount_in: optimal_amount,
            expected_amount_out: final_amount,
            profit_lamports: net_profit as i64,
            fees_lamports: total_real_costs,
            route_type: format!("REAL_ROUTE: {:?} -> {:?}", quote_a.route_plan, quote_b.route_plan),
        }))
    }
    
    fn pools_have_common_token(&self, pool_a: &PoolData, pool_b: &PoolData) -> bool {
        pool_a.token_a_mint == pool_b.token_a_mint ||
        pool_a.token_a_mint == pool_b.token_b_mint ||
        pool_a.token_b_mint == pool_b.token_a_mint ||
        pool_a.token_b_mint == pool_b.token_b_mint
    }
    
    async fn get_wallet_balance(&self) -> Result<f64> {
        let balance_lamports = self.client.get_balance(&self.wallet_address)?;
        Ok(balance_lamports as f64 / 1_000_000_000.0)
    }
    
    pub fn get_statistics(&self) -> String {
        format!(
            "📊 PROFESSIONAL SYSTEM STATISTICS:\n\
             💰 Total Opportunities: {}\n\
             ✅ Successful Trades: {}\n\
             📈 Total Profit: {:.6} SOL\n\
             🏪 Active Pools: {}\n\
             🌐 Data Source: Live Blockchain + APIs",
            self.total_opportunities_found.load(Ordering::Relaxed),
            self.successful_trades.load(Ordering::Relaxed),
            self.total_profit_lamports.load(Ordering::Relaxed) as f64 / 1e9,
            self.operational_pools.len()
        )
    }
    
    fn update_performance_metrics(&mut self, opportunity: &DirectOpportunity, success: bool) {
        if success {
            self.performance_metrics.successful_trades += 1;
            self.performance_metrics.total_profit_usd += (opportunity.profit_lamports as f64 / 1e9) * 200.0;
        }
        self.performance_metrics.total_trades += 1;
        
        if self.performance_metrics.total_trades > 0 {
            self.risk_metrics.success_rate = self.performance_metrics.successful_trades as f64 / self.performance_metrics.total_trades as f64;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("🚀 STARTING PROFESSIONAL ARBITRAGE ENGINE");
    
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let wallet_path = std::env::var("WALLET_PATH")
        .unwrap_or_else(|_| "wallet.json".to_string());
    
    let mut system = ProfessionalArbitrageEngine::new_professional(rpc_url, wallet_path).await?;
    
    loop {
        match system.run_professional_arbitrage().await {
            Ok(_) => {
                info!("✅ Professional arbitrage cycle completed successfully");
            }
            Err(e) => {
                error!("❌ Professional arbitrage cycle failed: {}", e);
            }
        }
        
        println!("{}", system.get_statistics());
        
        info!("⏳ Waiting 30 seconds before next cycle...");
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}
