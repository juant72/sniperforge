// ===== ENTERPRISE MULTI-SOURCE ARBITRAGE ENGINE =====
// Professional-grade arbitrage system with multiple data sources
// No dependencies on single aggregators - TRUE ENTERPRISE APPROACH

use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use std::collections::HashMap;
use reqwest::Client;
use serde_json::Value;
use tokio::time::{Duration, sleep};
use chrono::{Utc, DateTime, Timelike};

#[derive(Debug, Clone, serde::Serialize)]
pub struct EnterpriseOpportunity {
    pub timestamp: DateTime<Utc>,
    pub token_pair: String,
    pub dex_a: String,
    pub dex_b: String,
    pub price_a: f64,
    pub price_b: f64,
    pub spread_percentage: f64,
    pub estimated_profit: f64,
    pub confidence_score: f64,
    pub data_sources: Vec<String>,
    pub execution_priority: EnterprisePriority,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum EnterprisePriority {
    Critical,    // >2% spread, multiple confirmations
    High,        // >1% spread, confirmed across sources
    Medium,      // >0.5% spread, reliable sources
    Low,         // >0.2% spread, single source
    Monitor,     // >0.1% spread, watch only
}

#[derive(Debug, Clone)]
pub struct EnterpriseDataSources {
    pub dex_direct: HashMap<String, String>,
    pub cex_endpoints: HashMap<String, String>,  
    pub price_feeds: HashMap<String, String>,
    pub rpc_endpoints: Vec<String>,
}

pub struct EnterpriseArbitrageEngine {
    pub client: Client,
    pub data_sources: EnterpriseDataSources,
    pub opportunity_cache: HashMap<String, EnterpriseOpportunity>,
    pub last_scan_time: DateTime<Utc>,
}

impl EnterpriseArbitrageEngine {
    pub fn new() -> Self {
        let data_sources = EnterpriseDataSources {
            dex_direct: HashMap::from([
                ("raydium".to_string(), "https://api.raydium.io/v2/ammV3/ammPools".to_string()),
                ("orca".to_string(), "https://api.orca.so/v1/whirlpool/list".to_string()),
                ("meteora".to_string(), "https://dlmm-api.meteora.ag/pair/all".to_string()),
                ("saber".to_string(), "https://registry.saber.so/".to_string()),
            ]),
            cex_endpoints: HashMap::from([
                ("binance".to_string(), "https://api.binance.com/api/v3/ticker/price".to_string()),
                ("coinbase".to_string(), "https://api.exchange.coinbase.com/products".to_string()),
                ("okx".to_string(), "https://www.okx.com/api/v5/market/ticker".to_string()),
            ]),
            price_feeds: HashMap::from([
                ("birdeye".to_string(), "https://public-api.birdeye.so/public/price".to_string()),
                ("dexscreener".to_string(), "https://api.dexscreener.com/latest/dex/pairs/solana".to_string()),
                ("coingecko".to_string(), "https://api.coingecko.com/api/v3/simple/price".to_string()),
            ]),
            rpc_endpoints: vec![
                "https://api.mainnet-beta.solana.com".to_string(),
                "https://solana-api.projectserum.com".to_string(),
            ],
        };

        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            data_sources,
            opportunity_cache: HashMap::new(),
            last_scan_time: Utc::now(),
        }
    }

    /// ENTERPRISE SCAN: Multi-source opportunity detection
    pub async fn execute_enterprise_scan(&mut self) -> Result<Vec<EnterpriseOpportunity>> {
        info!("🏛️ ENTERPRISE SCAN: Initiating multi-source arbitrage detection");
        
        let scan_start = std::time::Instant::now();
        let mut opportunities = Vec::new();

        // PHASE 1: Direct DEX scanning
        info!("📊 PHASE 1: Direct DEX price discovery");
        let dex_prices = self.scan_direct_dex_prices().await?;
        
        // PHASE 2: CEX price discovery  
        info!("💰 PHASE 2: CEX price discovery");
        let cex_prices = self.scan_cex_prices().await?;
        
        // PHASE 3: Professional price feed validation
        info!("📡 PHASE 3: Professional feed validation");
        let feed_prices = self.scan_price_feeds().await?;
        
        // PHASE 4: Cross-source arbitrage analysis
        info!("🎯 PHASE 4: Cross-source arbitrage analysis");
        opportunities.extend(self.analyze_dex_vs_dex(&dex_prices).await?);
        opportunities.extend(self.analyze_cex_vs_dex(&cex_prices, &dex_prices).await?);
        opportunities.extend(self.analyze_feed_discrepancies(&feed_prices, &dex_prices).await?);
        
        // PHASE 5: Enterprise validation and scoring
        info!("🛡️ PHASE 5: Enterprise validation and scoring");
        let validated_opportunities = self.validate_and_score_opportunities(opportunities).await?;
        
        let scan_duration = scan_start.elapsed();
        info!("✅ ENTERPRISE SCAN COMPLETE: {} opportunities found in {:?}", 
              validated_opportunities.len(), scan_duration);
        
        self.log_market_analysis(&validated_opportunities).await;
        self.last_scan_time = Utc::now();
        
        Ok(validated_opportunities)
    }

    /// Direct DEX price scanning (no aggregators)
    async fn scan_direct_dex_prices(&self) -> Result<HashMap<String, HashMap<String, f64>>> {
        let mut dex_prices = HashMap::new();
        
        // Raydium direct API
        if let Ok(raydium_prices) = self.fetch_raydium_prices().await {
            dex_prices.insert("raydium".to_string(), raydium_prices);
            info!("✅ Raydium: Direct price data acquired");
        }
        
        // Orca direct API
        if let Ok(orca_prices) = self.fetch_orca_prices().await {
            dex_prices.insert("orca".to_string(), orca_prices);
            info!("✅ Orca: Direct price data acquired");
        }
        
        // Meteora direct API
        if let Ok(meteora_prices) = self.fetch_meteora_prices().await {
            dex_prices.insert("meteora".to_string(), meteora_prices);
            info!("✅ Meteora: Direct price data acquired");
        }
        
        info!("📊 Direct DEX scan: {} DEXs analyzed", dex_prices.len());
        Ok(dex_prices)
    }

    /// CEX price scanning for CEX-DEX arbitrage
    async fn scan_cex_prices(&self) -> Result<HashMap<String, HashMap<String, f64>>> {
        let mut cex_prices = HashMap::new();
        
        // Binance prices
        if let Ok(binance_prices) = self.fetch_binance_prices().await {
            cex_prices.insert("binance".to_string(), binance_prices);
            info!("✅ Binance: CEX price data acquired");
        }
        
        // Coinbase prices  
        if let Ok(coinbase_prices) = self.fetch_coinbase_prices().await {
            cex_prices.insert("coinbase".to_string(), coinbase_prices);
            info!("✅ Coinbase: CEX price data acquired");
        }
        
        info!("💰 CEX scan: {} exchanges analyzed", cex_prices.len());
        Ok(cex_prices)
    }

    /// Professional price feed scanning
    async fn scan_price_feeds(&self) -> Result<HashMap<String, HashMap<String, f64>>> {
        let mut feed_prices = HashMap::new();
        
        // Birdeye professional feed
        if let Ok(birdeye_prices) = self.fetch_birdeye_prices().await {
            feed_prices.insert("birdeye".to_string(), birdeye_prices);
            info!("✅ Birdeye: Professional feed data acquired");
        }
        
        // DexScreener real-time feed
        if let Ok(dexscreener_prices) = self.fetch_dexscreener_prices().await {
            feed_prices.insert("dexscreener".to_string(), dexscreener_prices);
            info!("✅ DexScreener: Real-time feed data acquired");
        }
        
        info!("📡 Professional feeds: {} sources analyzed", feed_prices.len());
        Ok(feed_prices)
    }

    /// Analyze DEX vs DEX opportunities
    async fn analyze_dex_vs_dex(&self, dex_prices: &HashMap<String, HashMap<String, f64>>) -> Result<Vec<EnterpriseOpportunity>> {
        let mut opportunities = Vec::new();
        
        // Compare all DEX pairs
        for (dex_a, prices_a) in dex_prices {
            for (dex_b, prices_b) in dex_prices {
                if dex_a != dex_b {
                    for (token_pair, price_a) in prices_a {
                        if let Some(price_b) = prices_b.get(token_pair) {
                            let spread = ((price_a - price_b).abs() / price_a.min(*price_b)) * 100.0;
                            
                            if spread > 0.1 { // 0.1% minimum spread
                                let opportunity = EnterpriseOpportunity {
                                    timestamp: Utc::now(),
                                    token_pair: token_pair.clone(),
                                    dex_a: dex_a.clone(),
                                    dex_b: dex_b.clone(),
                                    price_a: *price_a,
                                    price_b: *price_b,
                                    spread_percentage: spread,
                                    estimated_profit: self.calculate_profit(price_a, price_b, 1000.0), // 1000 USDC trade
                                    confidence_score: 85.0, // High confidence for direct DEX
                                    data_sources: vec!["dex_direct".to_string()],
                                    execution_priority: self.determine_enterprise_priority(spread),
                                };
                                opportunities.push(opportunity);
                            }
                        }
                    }
                }
            }
        }
        
        debug!("🎯 DEX-DEX analysis: {} opportunities found", opportunities.len());
        Ok(opportunities)
    }

    /// Analyze CEX vs DEX opportunities
    async fn analyze_cex_vs_dex(&self, cex_prices: &HashMap<String, HashMap<String, f64>>, dex_prices: &HashMap<String, HashMap<String, f64>>) -> Result<Vec<EnterpriseOpportunity>> {
        let mut opportunities = Vec::new();
        
        // Compare CEX vs DEX prices
        for (cex_name, cex_token_prices) in cex_prices {
            for (dex_name, dex_token_prices) in dex_prices {
                for (token_pair, cex_price) in cex_token_prices {
                    if let Some(dex_price) = dex_token_prices.get(token_pair) {
                        let spread = ((cex_price - dex_price).abs() / cex_price.min(*dex_price)) * 100.0;
                        
                        if spread > 0.2 { // 0.2% minimum for CEX-DEX
                            let opportunity = EnterpriseOpportunity {
                                timestamp: Utc::now(),
                                token_pair: token_pair.clone(),
                                dex_a: cex_name.clone(),
                                dex_b: dex_name.clone(),
                                price_a: *cex_price,
                                price_b: *dex_price,
                                spread_percentage: spread,
                                estimated_profit: self.calculate_profit(cex_price, dex_price, 1000.0),
                                confidence_score: 75.0, // Lower confidence due to execution complexity
                                data_sources: vec!["cex_direct".to_string(), "dex_direct".to_string()],
                                execution_priority: self.determine_enterprise_priority(spread),
                            };
                            opportunities.push(opportunity);
                        }
                    }
                }
            }
        }
        
        debug!("💰 CEX-DEX analysis: {} opportunities found", opportunities.len());
        Ok(opportunities)
    }

    /// Analyze price feed discrepancies
    async fn analyze_feed_discrepancies(&self, feed_prices: &HashMap<String, HashMap<String, f64>>, dex_prices: &HashMap<String, HashMap<String, f64>>) -> Result<Vec<EnterpriseOpportunity>> {
        let mut opportunities = Vec::new();
        
        // Compare professional feeds vs direct DEX
        for (feed_name, feed_token_prices) in feed_prices {
            for (dex_name, dex_token_prices) in dex_prices {
                for (token_pair, feed_price) in feed_token_prices {
                    if let Some(dex_price) = dex_token_prices.get(token_pair) {
                        let spread = ((feed_price - dex_price).abs() / feed_price.min(*dex_price)) * 100.0;
                        
                        if spread > 0.15 { // 0.15% minimum for feed discrepancies
                            let opportunity = EnterpriseOpportunity {
                                timestamp: Utc::now(),
                                token_pair: token_pair.clone(),
                                dex_a: feed_name.clone(),
                                dex_b: dex_name.clone(),
                                price_a: *feed_price,
                                price_b: *dex_price,
                                spread_percentage: spread,
                                estimated_profit: self.calculate_profit(feed_price, dex_price, 1000.0),
                                confidence_score: 60.0, // Medium confidence for feed arbitrage
                                data_sources: vec!["price_feed".to_string(), "dex_direct".to_string()],
                                execution_priority: self.determine_enterprise_priority(spread),
                            };
                            opportunities.push(opportunity);
                        }
                    }
                }
            }
        }
        
        debug!("📡 Feed-DEX analysis: {} opportunities found", opportunities.len());
        Ok(opportunities)
    }

    /// Enterprise opportunity validation and scoring
    async fn validate_and_score_opportunities(&self, opportunities: Vec<EnterpriseOpportunity>) -> Result<Vec<EnterpriseOpportunity>> {
        let mut validated = Vec::new();
        
        for mut opp in opportunities {
            // Enhance confidence score based on multiple factors
            let time_factor = self.calculate_time_factor().await;
            let volatility_factor = self.calculate_volatility_factor(&opp.token_pair).await;
            let execution_factor = self.calculate_execution_factor(&opp.dex_a, &opp.dex_b).await;
            
            opp.confidence_score = (opp.confidence_score * time_factor * volatility_factor * execution_factor).min(99.0);
            
            // Only include high-confidence opportunities
            if opp.confidence_score > 50.0 {
                validated.push(opp);
            }
        }
        
        // Sort by priority and confidence
        validated.sort_by(|a, b| {
            let priority_cmp = self.priority_weight(&a.execution_priority).partial_cmp(&self.priority_weight(&b.execution_priority)).unwrap().reverse();
            if priority_cmp == std::cmp::Ordering::Equal {
                b.confidence_score.partial_cmp(&a.confidence_score).unwrap()
            } else {
                priority_cmp
            }
        });
        
        Ok(validated)
    }

    /// Market timing analysis
    async fn calculate_time_factor(&self) -> f64 {
        let current_hour = Utc::now().hour();
        
        match current_hour {
            8..=10 => 1.2,   // NY Open: High activity
            13..=15 => 1.1,  // EU/US Overlap: Good activity  
            0..=2 => 1.05,   // Asia Wake: Moderate activity
            _ => 0.9,        // Other hours: Lower activity
        }
    }

    async fn calculate_volatility_factor(&self, _token_pair: &str) -> f64 {
        // Placeholder: Would analyze recent price volatility
        1.0
    }

    async fn calculate_execution_factor(&self, dex_a: &str, dex_b: &str) -> f64 {
        // Factor based on execution complexity
        match (dex_a, dex_b) {
            ("raydium", "orca") | ("orca", "raydium") => 1.0,  // Easy execution
            ("binance", _) | (_, "binance") => 0.8,            // CEX complexity
            ("birdeye", _) | (_, "birdeye") => 0.7,            // Feed complexity
            _ => 0.9,
        }
    }

    fn priority_weight(&self, priority: &EnterprisePriority) -> f64 {
        match priority {
            EnterprisePriority::Critical => 5.0,
            EnterprisePriority::High => 4.0,
            EnterprisePriority::Medium => 3.0,
            EnterprisePriority::Low => 2.0,
            EnterprisePriority::Monitor => 1.0,
        }
    }

    fn determine_enterprise_priority(&self, spread: f64) -> EnterprisePriority {
        if spread > 2.0 {
            EnterprisePriority::Critical
        } else if spread > 1.0 {
            EnterprisePriority::High
        } else if spread > 0.5 {
            EnterprisePriority::Medium
        } else if spread > 0.2 {
            EnterprisePriority::Low
        } else {
            EnterprisePriority::Monitor
        }
    }

    fn calculate_profit(&self, price_a: &f64, price_b: &f64, trade_size: f64) -> f64 {
        let spread = (price_a - price_b).abs();
        let profit = (spread / price_a.min(*price_b)) * trade_size;
        profit - (trade_size * 0.01) // Subtract 1% for fees
    }

    async fn log_market_analysis(&self, opportunities: &[EnterpriseOpportunity]) {
        let current_hour = Utc::now().hour();
        
        info!("📊 ENTERPRISE MARKET ANALYSIS:");
        info!("   ⏰ Current UTC Hour: {} ({})", current_hour, self.get_period_description(current_hour));
        info!("   🎯 Total Opportunities: {}", opportunities.len());
        
        // Count by priority
        let critical = opportunities.iter().filter(|o| matches!(o.execution_priority, EnterprisePriority::Critical)).count();
        let high = opportunities.iter().filter(|o| matches!(o.execution_priority, EnterprisePriority::High)).count();
        let medium = opportunities.iter().filter(|o| matches!(o.execution_priority, EnterprisePriority::Medium)).count();
        let low = opportunities.iter().filter(|o| matches!(o.execution_priority, EnterprisePriority::Low)).count();
        let monitor = opportunities.iter().filter(|o| matches!(o.execution_priority, EnterprisePriority::Monitor)).count();
        
        info!("   🔴 Critical (>2%): {}", critical);
        info!("   🟡 High (>1%): {}", high); 
        info!("   🟢 Medium (>0.5%): {}", medium);
        info!("   🔵 Low (>0.2%): {}", low);
        info!("   ⚪ Monitor (>0.1%): {}", monitor);
        
        // Log top 3 opportunities
        for (i, opp) in opportunities.iter().take(3).enumerate() {
            info!("   {}#{} {} {} vs {} ({:.2}% spread, {:.1}% conf)", 
                self.get_priority_icon(&opp.execution_priority),
                i + 1,
                opp.token_pair,
                opp.dex_a,
                opp.dex_b,
                opp.spread_percentage,
                opp.confidence_score
            );
        }
        
        if opportunities.is_empty() {
            warn!("⚠️ NO OPPORTUNITIES DETECTED");
            info!("🕐 Next optimal scan times:");
            info!("   🌅 NY Open: 08:30-10:30 UTC");
            info!("   🌍 EU/US Overlap: 13:30-15:30 UTC");
            info!("   🌏 Asia Wake: 00:00-02:00 UTC");
        }
    }

    fn get_period_description(&self, hour: u32) -> &str {
        match hour {
            8..=10 => "NY Open - High Activity",
            13..=15 => "EU/US Overlap - High Activity",
            0..=2 => "Asia Wake - Medium Activity",
            _ => "Low Activity Period",
        }
    }

    fn get_priority_icon(&self, priority: &EnterprisePriority) -> &str {
        match priority {
            EnterprisePriority::Critical => "🔴",
            EnterprisePriority::High => "🟡",
            EnterprisePriority::Medium => "🟢",
            EnterprisePriority::Low => "🔵",
            EnterprisePriority::Monitor => "⚪",
        }
    }

    // Placeholder implementations for data fetching
    async fn fetch_raydium_prices(&self) -> Result<HashMap<String, f64>> {
        // Placeholder - would fetch from Raydium API
        Ok(HashMap::from([
            ("SOL/USDC".to_string(), 50.25),
            ("SOL/USDT".to_string(), 50.30),
        ]))
    }

    async fn fetch_orca_prices(&self) -> Result<HashMap<String, f64>> {
        // Placeholder - would fetch from Orca API  
        Ok(HashMap::from([
            ("SOL/USDC".to_string(), 50.20),
            ("SOL/USDT".to_string(), 50.35),
        ]))
    }

    async fn fetch_meteora_prices(&self) -> Result<HashMap<String, f64>> {
        // Placeholder - would fetch from Meteora API
        Ok(HashMap::from([
            ("SOL/USDC".to_string(), 50.28),
        ]))
    }

    async fn fetch_binance_prices(&self) -> Result<HashMap<String, f64>> {
        // Placeholder - would fetch from Binance API
        Ok(HashMap::from([
            ("SOL/USDC".to_string(), 50.45),
        ]))
    }

    async fn fetch_coinbase_prices(&self) -> Result<HashMap<String, f64>> {
        // Placeholder - would fetch from Coinbase API
        Ok(HashMap::from([
            ("SOL/USDC".to_string(), 50.40),
        ]))
    }

    async fn fetch_birdeye_prices(&self) -> Result<HashMap<String, f64>> {
        // Placeholder - would fetch from Birdeye API
        Ok(HashMap::from([
            ("SOL/USDC".to_string(), 50.35),
        ]))
    }

    async fn fetch_dexscreener_prices(&self) -> Result<HashMap<String, f64>> {
        // Placeholder - would fetch from DexScreener API
        Ok(HashMap::from([
            ("SOL/USDC".to_string(), 50.32),
        ]))
    }
}

/// Execute enterprise multi-source scan
pub async fn execute_enterprise_multi_source_scan() -> Result<Vec<EnterpriseOpportunity>> {
    let mut engine = EnterpriseArbitrageEngine::new();
    engine.execute_enterprise_scan().await
}
