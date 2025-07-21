// 🎯 ENTERPRISE OPPORTUNITY ENGINE - REAL ARBITRAGE DETECTION
// Motor avanzado de detección de oportunidades para Binance Labs Demo

use anyhow::{Result, anyhow};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{info, warn, error, debug};
use std::time::{SystemTime, UNIX_EPOCH, Instant};

use crate::types::*;
use crate::enterprise_pool_discovery::EnterprisePoolData;
use crate::jupiter_api::JupiterAPI;
use crate::calculations::*;

/// Enterprise Opportunity Detection Engine
pub struct EnterpriseOpportunityEngine {
    jupiter_api: JupiterAPI,
    active_pools: HashMap<Pubkey, EnterprisePoolData>,
    opportunity_history: Vec<EnterpriseOpportunity>,
    detection_metrics: OpportunityMetrics,
    last_scan_time: Instant,
}

#[derive(Debug, Clone)]
pub struct EnterpriseOpportunity {
    pub id: String,
    pub opportunity_type: OpportunityType,
    pub pool_a: EnterprisePoolData,
    pub pool_b: EnterprisePoolData,
    pub pool_c: Option<EnterprisePoolData>, // For triangle arbitrage
    pub trade_path: Vec<TradeLeg>,
    pub input_amount: u64,
    pub expected_output: u64,
    pub estimated_profit: i64,
    pub profit_percentage: f64,
    pub execution_complexity: u8, // 1-10 scale
    pub risk_score: f64, // 0.0-1.0
    pub confidence_score: f64, // 0.0-1.0
    pub estimated_execution_time_ms: u64,
    pub gas_cost_estimate: u64,
    pub discovered_at: u64,
    pub expires_at: u64,
}

#[derive(Debug, Clone)]
pub enum OpportunityType {
    DirectArbitrage,     // A->B->A
    TriangleArbitrage,   // A->B->C->A
    CrossDexArbitrage,   // Same pair different DEXs
    FlashLoanArbitrage,  // Using flash loans
}

#[derive(Debug, Clone)]
pub struct TradeLeg {
    pub dex: PoolType,
    pub pool_address: Pubkey,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub input_amount: u64,
    pub output_amount: u64,
    pub fee_amount: u64,
    pub price_impact: f64,
}

#[derive(Debug, Default)]
pub struct OpportunityMetrics {
    pub total_opportunities_found: u64,
    pub profitable_opportunities: u64,
    pub executed_opportunities: u64,
    pub total_profit_generated: f64,
    pub average_profit_percentage: f64,
    pub average_execution_time_ms: f64,
    pub success_rate: f64,
}

impl EnterpriseOpportunityEngine {
    pub fn new() -> Self {
        Self {
            jupiter_api: JupiterAPI::new(),
            active_pools: HashMap::new(),
            opportunity_history: Vec::new(),
            detection_metrics: OpportunityMetrics::default(),
            last_scan_time: Instant::now(),
        }
    }
    
    pub fn update_pools(&mut self, pools: Vec<EnterprisePoolData>) {
        info!("🏛️ ENTERPRISE OPPORTUNITY ENGINE: Updating pool data");
        
        self.active_pools.clear();
        for pool in pools {
            self.active_pools.insert(pool.address, pool);
        }
        
        info!("✅ ENTERPRISE POOLS UPDATED: {} active pools loaded", self.active_pools.len());
    }
    
    /// Main enterprise opportunity detection engine
    pub async fn discover_enterprise_opportunities(&mut self) -> Result<Vec<EnterpriseOpportunity>> {
        let scan_start = Instant::now();
        info!("🎯 ENTERPRISE OPPORTUNITY SCAN: Initiating comprehensive market analysis");
        
        let mut all_opportunities = Vec::new();
        
        // PHASE 1: Direct Arbitrage Detection (Same token pair, different DEXs)
        info!("📊 PHASE 1: Direct Cross-DEX Arbitrage Scanning");
        let direct_opportunities = self.detect_direct_arbitrage().await?;
        info!("💎 Direct Opportunities: {} found", direct_opportunities.len());
        all_opportunities.extend(direct_opportunities);
        
        // PHASE 2: Triangle Arbitrage Detection (A->B->C->A)
        info!("📊 PHASE 2: Triangle Arbitrage Pattern Detection");
        let triangle_opportunities = self.detect_triangle_arbitrage().await?;
        info!("💎 Triangle Opportunities: {} found", triangle_opportunities.len());
        all_opportunities.extend(triangle_opportunities);
        
        // PHASE 3: Advanced Cross-DEX Analysis
        info!("📊 PHASE 3: Advanced Cross-DEX Pattern Analysis");
        let advanced_opportunities = self.detect_advanced_cross_dex().await?;
        info!("💎 Advanced Opportunities: {} found", advanced_opportunities.len());
        all_opportunities.extend(advanced_opportunities);
        
        // PHASE 4: Enterprise Quality Filtering
        let filtered_opportunities = self.apply_enterprise_filters(all_opportunities).await?;
        
        // PHASE 5: Risk Scoring and Ranking
        let ranked_opportunities = self.rank_opportunities_by_enterprise_criteria(filtered_opportunities).await?;
        
        let scan_duration = scan_start.elapsed();
        self.last_scan_time = Instant::now();
        
        // Update metrics
        self.detection_metrics.total_opportunities_found += ranked_opportunities.len() as u64;
        if !ranked_opportunities.is_empty() {
            self.detection_metrics.average_profit_percentage = 
                ranked_opportunities.iter().map(|o| o.profit_percentage).sum::<f64>() / ranked_opportunities.len() as f64;
        }
        
        info!("🏛️ ENTERPRISE OPPORTUNITY SCAN COMPLETE:");
        info!("   💰 Total Opportunities: {}", ranked_opportunities.len());
        info!("   ⚡ Scan Duration: {}ms", scan_duration.as_millis());
        info!("   🎯 Average Profit: {:.3}%", self.detection_metrics.average_profit_percentage);
        
        Ok(ranked_opportunities)
    }
    
    /// Detect direct arbitrage opportunities (same pair, different DEXs)
    async fn detect_direct_arbitrage(&self) -> Result<Vec<EnterpriseOpportunity>> {
        let mut opportunities = Vec::new();
        
        // Group pools by token pair
        let mut token_pairs: HashMap<(Pubkey, Pubkey), Vec<&EnterprisePoolData>> = HashMap::new();
        
        for pool in self.active_pools.values() {
            let pair = (pool.token_a_mint, pool.token_b_mint);
            token_pairs.entry(pair).or_insert_with(Vec::new).push(pool);
            
            // Also add reversed pair
            let reversed_pair = (pool.token_b_mint, pool.token_a_mint);
            token_pairs.entry(reversed_pair).or_insert_with(Vec::new).push(pool);
        }
        
        // Find arbitrage opportunities between pools with same token pairs
        for ((_token_a, _token_b), pools_for_pair) in token_pairs {
            if pools_for_pair.len() >= 2 {
                for i in 0..pools_for_pair.len() {
                    for j in (i + 1)..pools_for_pair.len() {
                        let pool_a = pools_for_pair[i];
                        let pool_b = pools_for_pair[j];
                        
                        // Skip if same DEX
                        if pool_a.dex_type == pool_b.dex_type {
                            continue;
                        }
                        
                        if let Some(opportunity) = self.calculate_direct_arbitrage(pool_a, pool_b).await? {
                            opportunities.push(opportunity);
                        }
                    }
                }
            }
        }
        
        Ok(opportunities)
    }
    
    async fn calculate_direct_arbitrage(&self, pool_a: &EnterprisePoolData, pool_b: &EnterprisePoolData) -> Result<Option<EnterpriseOpportunity>> {
        // Calculate optimal trade size (conservative approach)
        let max_trade_amount = std::cmp::min(
            pool_a.token_a_reserve / 20, // Max 5% of pool A
            pool_b.token_b_reserve / 20  // Max 5% of pool B
        );
        
        let trade_amount = std::cmp::min(max_trade_amount, 10_000_000_000); // Max 10 SOL
        
        if trade_amount < 100_000_000 { // Min 0.1 SOL
            return Ok(None);
        }
        
        // Step 1: Trade in Pool A (SOL -> USDC)
        let output_a = calculate_amm_output_exact(
            pool_a.token_a_reserve,
            pool_a.token_b_reserve,
            trade_amount,
            pool_a.fee_rate,
        )?;
        
        // Step 2: Trade in Pool B (USDC -> SOL)
        let output_b = calculate_amm_output_exact(
            pool_b.token_b_reserve,
            pool_b.token_a_reserve,
            output_a,
            pool_b.fee_rate,
        )?;
        
        // Calculate profit
        if output_b <= trade_amount {
            return Ok(None);
        }
        
        let gross_profit = output_b - trade_amount;
        let gas_cost = 50_000; // Estimated gas cost in lamports
        
        if gross_profit <= gas_cost {
            return Ok(None);
        }
        
        let net_profit = gross_profit - gas_cost;
        let profit_percentage = (net_profit as f64 / trade_amount as f64) * 100.0;
        
        // Only consider profitable opportunities
        if profit_percentage < 0.1 { // Minimum 0.1% profit
            return Ok(None);
        }
        
        let trade_path = vec![
            TradeLeg {
                dex: pool_a.dex_type.clone(),
                pool_address: pool_a.address,
                input_mint: pool_a.token_a_mint,
                output_mint: pool_a.token_b_mint,
                input_amount: trade_amount,
                output_amount: output_a,
                fee_amount: (trade_amount * pool_a.fee_rate) / 10000,
                price_impact: (trade_amount as f64 / pool_a.token_a_reserve as f64) * 100.0,
            },
            TradeLeg {
                dex: pool_b.dex_type.clone(),
                pool_address: pool_b.address,
                input_mint: pool_b.token_b_mint,
                output_mint: pool_b.token_a_mint,
                input_amount: output_a,
                output_amount: output_b,
                fee_amount: (output_a * pool_b.fee_rate) / 10000,
                price_impact: (output_a as f64 / pool_b.token_b_reserve as f64) * 100.0,
            },
        ];
        
        let opportunity = EnterpriseOpportunity {
            id: format!("DIRECT_{}_{}", 
                pool_a.address.to_string()[..8].to_uppercase(),
                pool_b.address.to_string()[..8].to_uppercase()),
            opportunity_type: OpportunityType::DirectArbitrage,
            pool_a: pool_a.clone(),
            pool_b: pool_b.clone(),
            pool_c: None,
            trade_path,
            input_amount: trade_amount,
            expected_output: output_b,
            estimated_profit: net_profit as i64,
            profit_percentage,
            execution_complexity: 3, // Medium complexity
            risk_score: self.calculate_risk_score(pool_a, pool_b, profit_percentage),
            confidence_score: self.calculate_confidence_score(pool_a, pool_b),
            estimated_execution_time_ms: 3000, // 3 seconds estimated
            gas_cost_estimate: gas_cost,
            discovered_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            expires_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 60, // 1 minute expiry
        };
        
        debug!("💎 DIRECT ARBITRAGE FOUND: {:.3}% profit, {} -> {} -> {}", 
               profit_percentage,
               pool_a.dex_type as u8,
               pool_b.dex_type as u8,
               pool_a.dex_type as u8);
        
        Ok(Some(opportunity))
    }
    
    /// Detect triangle arbitrage opportunities (A->B->C->A)
    async fn detect_triangle_arbitrage(&self) -> Result<Vec<EnterpriseOpportunity>> {
        let mut opportunities = Vec::new();
        
        // For triangle arbitrage, we need to find cycles like SOL->USDC->USDT->SOL
        let pools: Vec<_> = self.active_pools.values().collect();
        
        for pool_a in &pools {
            for pool_b in &pools {
                for pool_c in &pools {
                    // Skip if same pool
                    if pool_a.address == pool_b.address || 
                       pool_b.address == pool_c.address || 
                       pool_a.address == pool_c.address {
                        continue;
                    }
                    
                    // Check if we can form a triangle: A->B->C->A
                    if self.can_form_triangle(pool_a, pool_b, pool_c) {
                        if let Some(opportunity) = self.calculate_triangle_arbitrage(pool_a, pool_b, pool_c).await? {
                            opportunities.push(opportunity);
                        }
                    }
                }
            }
        }
        
        Ok(opportunities)
    }
    
    fn can_form_triangle(&self, pool_a: &EnterprisePoolData, pool_b: &EnterprisePoolData, pool_c: &EnterprisePoolData) -> bool {
        // Check if tokens form a valid triangle
        // Example: SOL->USDC (pool_a), USDC->USDT (pool_b), USDT->SOL (pool_c)
        
        let sol = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
        let usdc = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
        let usdt = Pubkey::from_str("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB").unwrap();
        
        // SOL->USDC->USDT->SOL triangle
        (pool_a.token_a_mint == sol && pool_a.token_b_mint == usdc &&
         pool_b.token_a_mint == usdc && pool_b.token_b_mint == usdt &&
         pool_c.token_a_mint == usdt && pool_c.token_b_mint == sol) ||
        // Reverse direction
        (pool_a.token_a_mint == sol && pool_a.token_b_mint == usdt &&
         pool_b.token_a_mint == usdt && pool_b.token_b_mint == usdc &&
         pool_c.token_a_mint == usdc && pool_c.token_b_mint == sol)
    }
    
    async fn calculate_triangle_arbitrage(&self, pool_a: &EnterprisePoolData, pool_b: &EnterprisePoolData, pool_c: &EnterprisePoolData) -> Result<Option<EnterpriseOpportunity>> {
        let trade_amount = 5_000_000_000u64; // 5 SOL
        
        // Step 1: SOL -> USDC
        let output_1 = calculate_amm_output_exact(
            pool_a.token_a_reserve,
            pool_a.token_b_reserve,
            trade_amount,
            pool_a.fee_rate,
        )?;
        
        // Step 2: USDC -> USDT
        let output_2 = calculate_amm_output_exact(
            pool_b.token_a_reserve,
            pool_b.token_b_reserve,
            output_1,
            pool_b.fee_rate,
        )?;
        
        // Step 3: USDT -> SOL
        let output_3 = calculate_amm_output_exact(
            pool_c.token_a_reserve,
            pool_c.token_b_reserve,
            output_2,
            pool_c.fee_rate,
        )?;
        
        if output_3 <= trade_amount {
            return Ok(None);
        }
        
        let gross_profit = output_3 - trade_amount;
        let gas_cost = 100_000; // Higher gas cost for 3 transactions
        
        if gross_profit <= gas_cost {
            return Ok(None);
        }
        
        let net_profit = gross_profit - gas_cost;
        let profit_percentage = (net_profit as f64 / trade_amount as f64) * 100.0;
        
        if profit_percentage < 0.2 { // Higher threshold for triangle arbitrage
            return Ok(None);
        }
        
        let trade_path = vec![
            TradeLeg {
                dex: pool_a.dex_type.clone(),
                pool_address: pool_a.address,
                input_mint: pool_a.token_a_mint,
                output_mint: pool_a.token_b_mint,
                input_amount: trade_amount,
                output_amount: output_1,
                fee_amount: (trade_amount * pool_a.fee_rate) / 10000,
                price_impact: (trade_amount as f64 / pool_a.token_a_reserve as f64) * 100.0,
            },
            TradeLeg {
                dex: pool_b.dex_type.clone(),
                pool_address: pool_b.address,
                input_mint: pool_b.token_a_mint,
                output_mint: pool_b.token_b_mint,
                input_amount: output_1,
                output_amount: output_2,
                fee_amount: (output_1 * pool_b.fee_rate) / 10000,
                price_impact: (output_1 as f64 / pool_b.token_a_reserve as f64) * 100.0,
            },
            TradeLeg {
                dex: pool_c.dex_type.clone(),
                pool_address: pool_c.address,
                input_mint: pool_c.token_a_mint,
                output_mint: pool_c.token_b_mint,
                input_amount: output_2,
                output_amount: output_3,
                fee_amount: (output_2 * pool_c.fee_rate) / 10000,
                price_impact: (output_2 as f64 / pool_c.token_a_reserve as f64) * 100.0,
            },
        ];
        
        let opportunity = EnterpriseOpportunity {
            id: format!("TRIANGLE_{}_{}_{}", 
                pool_a.address.to_string()[..6].to_uppercase(),
                pool_b.address.to_string()[..6].to_uppercase(),
                pool_c.address.to_string()[..6].to_uppercase()),
            opportunity_type: OpportunityType::TriangleArbitrage,
            pool_a: pool_a.clone(),
            pool_b: pool_b.clone(),
            pool_c: Some(pool_c.clone()),
            trade_path,
            input_amount: trade_amount,
            expected_output: output_3,
            estimated_profit: net_profit as i64,
            profit_percentage,
            execution_complexity: 7, // High complexity
            risk_score: self.calculate_triangle_risk_score(pool_a, pool_b, pool_c, profit_percentage),
            confidence_score: self.calculate_triangle_confidence_score(pool_a, pool_b, pool_c),
            estimated_execution_time_ms: 5000, // 5 seconds estimated
            gas_cost_estimate: gas_cost,
            discovered_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            expires_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 30, // 30 seconds expiry
        };
        
        debug!("🔺 TRIANGLE ARBITRAGE FOUND: {:.3}% profit", profit_percentage);
        
        Ok(Some(opportunity))
    }
    
    async fn detect_advanced_cross_dex(&self) -> Result<Vec<EnterpriseOpportunity>> {
        // Placeholder for advanced cross-DEX detection
        // This would include more sophisticated strategies
        Ok(Vec::new())
    }
    
    async fn apply_enterprise_filters(&self, opportunities: Vec<EnterpriseOpportunity>) -> Result<Vec<EnterpriseOpportunity>> {
        let original_count = opportunities.len();
        
        let filtered: Vec<_> = opportunities.into_iter()
            .filter(|opp| {
                // Enterprise quality filters
                opp.profit_percentage >= 0.1 && // Minimum 0.1% profit
                opp.risk_score <= 0.7 && // Maximum 70% risk
                opp.confidence_score >= 0.6 && // Minimum 60% confidence
                opp.estimated_execution_time_ms <= 10000 && // Max 10 seconds
                opp.trade_path.iter().all(|leg| leg.price_impact <= 5.0) // Max 5% price impact per leg
            })
            .collect();
        
        info!("🛡️ ENTERPRISE FILTERS APPLIED: {}/{} opportunities passed", filtered.len(), original_count);
        
        Ok(filtered)
    }
    
    async fn rank_opportunities_by_enterprise_criteria(&self, mut opportunities: Vec<EnterpriseOpportunity>) -> Result<Vec<EnterpriseOpportunity>> {
        // Sort by enterprise score (profit * confidence / risk)
        opportunities.sort_by(|a, b| {
            let score_a = a.profit_percentage * a.confidence_score / (1.0 + a.risk_score);
            let score_b = b.profit_percentage * b.confidence_score / (1.0 + b.risk_score);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Take top 20 opportunities
        opportunities.truncate(20);
        
        Ok(opportunities)
    }
    
    fn calculate_risk_score(&self, pool_a: &EnterprisePoolData, pool_b: &EnterprisePoolData, profit_percentage: f64) -> f64 {
        let tvl_risk = if pool_a.tvl_usd < 1_000_000.0 || pool_b.tvl_usd < 1_000_000.0 { 0.3 } else { 0.1 };
        let volume_risk = if pool_a.volume_24h_usd < 100_000.0 || pool_b.volume_24h_usd < 100_000.0 { 0.2 } else { 0.05 };
        let profit_risk = if profit_percentage > 5.0 { 0.4 } else { 0.1 }; // High profit might be too good to be true
        
        (tvl_risk + volume_risk + profit_risk).min(1.0)
    }
    
    fn calculate_confidence_score(&self, pool_a: &EnterprisePoolData, pool_b: &EnterprisePoolData) -> f64 {
        let tvl_confidence = ((pool_a.tvl_usd + pool_b.tvl_usd) / 20_000_000.0).min(1.0);
        let volume_confidence = ((pool_a.volume_24h_usd + pool_b.volume_24h_usd) / 2_000_000.0).min(1.0);
        let liquidity_confidence = (pool_a.liquidity_score + pool_b.liquidity_score) / 2.0;
        
        (tvl_confidence * 0.4 + volume_confidence * 0.3 + liquidity_confidence * 0.3).min(1.0)
    }
    
    fn calculate_triangle_risk_score(&self, pool_a: &EnterprisePoolData, pool_b: &EnterprisePoolData, pool_c: &EnterprisePoolData, profit_percentage: f64) -> f64 {
        let base_risk = self.calculate_risk_score(pool_a, pool_b, profit_percentage);
        let additional_risk = 0.2; // Extra risk for triangle arbitrage complexity
        (base_risk + additional_risk).min(1.0)
    }
    
    fn calculate_triangle_confidence_score(&self, pool_a: &EnterprisePoolData, pool_b: &EnterprisePoolData, pool_c: &EnterprisePoolData) -> f64 {
        let base_confidence = self.calculate_confidence_score(pool_a, pool_b);
        let pool_c_factor = (pool_c.tvl_usd / 10_000_000.0).min(1.0) * 0.3;
        (base_confidence * 0.7 + pool_c_factor).min(1.0)
    }
    
    pub fn get_enterprise_metrics(&self) -> &OpportunityMetrics {
        &self.detection_metrics
    }
    
    pub fn get_opportunity_summary(&self) -> String {
        format!(
            "🎯 ENTERPRISE OPPORTUNITY ENGINE METRICS:\n\
             💰 Total Opportunities Found: {}\n\
             📈 Profitable Opportunities: {}\n\
             ✅ Executed Successfully: {}\n\
             💎 Average Profit: {:.3}%\n\
             ⚡ Success Rate: {:.1}%\n\
             🕐 Last Scan: {:.1}s ago",
            self.detection_metrics.total_opportunities_found,
            self.detection_metrics.profitable_opportunities,
            self.detection_metrics.executed_opportunities,
            self.detection_metrics.average_profit_percentage,
            self.detection_metrics.success_rate * 100.0,
            self.last_scan_time.elapsed().as_secs_f64()
        )
    }
}
