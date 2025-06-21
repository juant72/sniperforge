// Cache-Free Trading Engine
// Phase 4 Implementation - Ultra-fast trading with real-time price validation

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::shared::pool_detector::{DetectedPool, TradingOpportunity, OpportunityType};

/// Cache-free trading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheFreeConfig {
    pub max_slippage_pct: f64,           // Maximum allowed slippage percentage
    pub price_staleness_ms: u64,         // Maximum price age in milliseconds
    pub confirmation_threshold: u8,       // Number of price confirmations required
    pub max_execution_time_ms: u64,      // Maximum time allowed for trade execution
    pub real_balance_check: bool,         // Whether to check real wallet balance
    pub safety_margin_pct: f64,          // Safety margin for position sizing
    pub min_profit_threshold_usd: f64,   // Minimum profit required to execute trade
}

impl Default for CacheFreeConfig {
    fn default() -> Self {
        Self {
            max_slippage_pct: 1.0,        // 1% max slippage
            price_staleness_ms: 500,      // 500ms max price age
            confirmation_threshold: 2,     // Require 2 price confirmations
            max_execution_time_ms: 2000,  // 2s max execution time
            real_balance_check: true,      // Always check real balance
            safety_margin_pct: 5.0,       // 5% safety margin
            min_profit_threshold_usd: 1.0, // $1 minimum profit
        }
    }
}

/// Real-time price data with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedPrice {
    pub token_address: String,
    pub price_usd: f64,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub confidence: f64,
    pub volume_24h: f64,
    pub liquidity_usd: f64,
}

/// Trade execution result with detailed metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheFreeTradeResult {
    pub trade_id: String,
    pub opportunity: TradingOpportunity,
    pub executed_at: DateTime<Utc>,
    pub execution_time_ms: u64,
    pub entry_price: f64,
    pub actual_slippage_pct: f64,
    pub profit_loss_usd: f64,
    pub gas_fees_usd: f64,
    pub net_profit_usd: f64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Real-time market data aggregator
#[derive(Debug)]
pub struct RealTimeMarketData {
    price_feeds: HashMap<String, Vec<ValidatedPrice>>,
    last_update: Instant,
    config: CacheFreeConfig,
}

impl RealTimeMarketData {
    pub fn new(config: CacheFreeConfig) -> Self {
        Self {
            price_feeds: HashMap::new(),
            last_update: Instant::now(),
            config,
        }
    }

    /// Update price data from multiple sources
    pub async fn update_prices(&mut self, token_addresses: &[String]) -> Result<()> {
        for address in token_addresses {
            let prices = self.fetch_real_time_prices(address).await?;
            self.price_feeds.insert(address.clone(), prices);
        }
        self.last_update = Instant::now();
        Ok(())
    }

    /// Fetch real-time prices from multiple sources
    async fn fetch_real_time_prices(&self, token_address: &str) -> Result<Vec<ValidatedPrice>> {
        let mut prices = Vec::new();
        let now = Utc::now();

        // Jupiter API price (most reliable)
        if let Ok(jupiter_price) = self.fetch_jupiter_price(token_address).await {
            prices.push(ValidatedPrice {
                token_address: token_address.to_string(),
                price_usd: jupiter_price.price,
                timestamp: now,
                source: "Jupiter".to_string(),
                confidence: 0.95,
                volume_24h: jupiter_price.volume_24h,
                liquidity_usd: jupiter_price.liquidity,
            });
        }

        // DexScreener price (secondary source)
        if let Ok(dex_price) = self.fetch_dexscreener_price(token_address).await {
            prices.push(ValidatedPrice {
                token_address: token_address.to_string(),
                price_usd: dex_price.price,
                timestamp: now,
                source: "DexScreener".to_string(),
                confidence: 0.85,
                volume_24h: dex_price.volume_24h,
                liquidity_usd: dex_price.liquidity,
            });
        }

        // Raydium direct price (ultra-fast)
        if let Ok(raydium_price) = self.fetch_raydium_price(token_address).await {
            prices.push(ValidatedPrice {
                token_address: token_address.to_string(),
                price_usd: raydium_price.price,
                timestamp: now,
                source: "Raydium".to_string(),
                confidence: 0.90,
                volume_24h: raydium_price.volume_24h,
                liquidity_usd: raydium_price.liquidity,
            });
        }

        Ok(prices)
    }

    /// Get validated price with confidence scoring
    pub fn get_validated_price(&self, token_address: &str) -> Option<ValidatedPrice> {
        let prices = self.price_feeds.get(token_address)?;
        
        if prices.is_empty() {
            return None;
        }

        // Check price staleness
        let now = Instant::now();
        let max_age = Duration::from_millis(self.config.price_staleness_ms);
        
        let fresh_prices: Vec<_> = prices.iter()
            .filter(|p| {
                let age = now.duration_since(Instant::now() - Duration::from_millis(
                    (Utc::now() - p.timestamp).num_milliseconds() as u64
                ));
                age < max_age
            })
            .collect();

        if fresh_prices.len() < self.config.confirmation_threshold as usize {
            return None;
        }

        // Calculate weighted average price based on confidence
        let total_weight: f64 = fresh_prices.iter().map(|p| p.confidence).sum();
        let weighted_price: f64 = fresh_prices.iter()
            .map(|p| p.price_usd * p.confidence)
            .sum::<f64>() / total_weight;

        // Use the most recent price data as base
        let base_price = fresh_prices.iter()
            .max_by_key(|p| p.timestamp)?;

        Some(ValidatedPrice {
            token_address: token_address.to_string(),
            price_usd: weighted_price,
            timestamp: Utc::now(),
            source: "Aggregated".to_string(),
            confidence: total_weight / fresh_prices.len() as f64,
            volume_24h: base_price.volume_24h,
            liquidity_usd: base_price.liquidity_usd,
        })
    }

    // Mock API calls for now - to be replaced with real implementations
    async fn fetch_jupiter_price(&self, _token_address: &str) -> Result<MockPriceData> {
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(MockPriceData {
            price: 0.00001234 + (rand::random::<f64>() - 0.5) * 0.000001,
            volume_24h: 50000.0,
            liquidity: 25000.0,
        })
    }

    async fn fetch_dexscreener_price(&self, _token_address: &str) -> Result<MockPriceData> {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(MockPriceData {
            price: 0.00001234 + (rand::random::<f64>() - 0.5) * 0.000001,
            volume_24h: 45000.0,
            liquidity: 23000.0,
        })
    }

    async fn fetch_raydium_price(&self, _token_address: &str) -> Result<MockPriceData> {
        tokio::time::sleep(Duration::from_millis(30)).await;
        Ok(MockPriceData {
            price: 0.00001234 + (rand::random::<f64>() - 0.5) * 0.000001,
            volume_24h: 48000.0,
            liquidity: 24000.0,
        })
    }
}

#[derive(Debug)]
struct MockPriceData {
    price: f64,
    volume_24h: f64,
    liquidity: f64,
}

/// Cache-free trading engine with real-time validation
#[derive(Debug)]
pub struct CacheFreeTradeEngine {
    config: CacheFreeConfig,
    market_data: RealTimeMarketData,
    active_trades: HashMap<String, CacheFreeTradeResult>,
    performance_metrics: CacheFreePerformanceMetrics,
}

impl CacheFreeTradeEngine {
    pub fn new(config: CacheFreeConfig) -> Self {
        let market_data = RealTimeMarketData::new(config.clone());
        
        Self {
            config,
            market_data,
            active_trades: HashMap::new(),
            performance_metrics: CacheFreePerformanceMetrics::new(),
        }
    }

    /// Execute trade with real-time validation
    pub async fn execute_trade_with_validation(&mut self, opportunity: &TradingOpportunity) -> Result<CacheFreeTradeResult> {
        let trade_id = Uuid::new_v4().to_string();
        let start_time = Instant::now();
        let execution_start = Utc::now();

        println!("🚀 CACHE-FREE TRADE EXECUTION");
        println!("   Trade ID: {}", &trade_id[..8]);
        println!("   Type: {} Opportunity", match opportunity.opportunity_type {
            OpportunityType::NewPoolSnipe => "New Pool",
            OpportunityType::PriceDiscrepancy => "Arbitrage",
            OpportunityType::LiquidityImbalance => "Low Slippage",
            OpportunityType::VolumeSpike => "Volume Spike",
        });

        // Step 1: Update real-time prices
        let token_addresses = vec![
            opportunity.pool.token_a.mint.clone(),
            opportunity.pool.token_b.mint.clone(),
        ];
        
        self.market_data.update_prices(&token_addresses).await?;

        // Step 2: Validate prices
        let token_a_price = self.market_data.get_validated_price(&opportunity.pool.token_a.mint)
            .ok_or_else(|| anyhow::anyhow!("Unable to get validated price for token A"))?;
        
        let token_b_price = self.market_data.get_validated_price(&opportunity.pool.token_b.mint)
            .ok_or_else(|| anyhow::anyhow!("Unable to get validated price for token B"))?;        // Step 3: Calculate real-time slippage
        let expected_price = opportunity.expected_profit_usd / opportunity.recommended_size_usd;
        let current_price = token_a_price.price_usd / token_b_price.price_usd;
        let actual_slippage = ((current_price - expected_price).abs() / expected_price) * 100.0;

        println!("   💰 Price Validation:");
        println!("      Token A: ${:.8} (confidence: {:.1}%)", token_a_price.price_usd, token_a_price.confidence * 100.0);
        println!("      Token B: ${:.8} (confidence: {:.1}%)", token_b_price.price_usd, token_b_price.confidence * 100.0);
        println!("      Slippage: {:.2}% (max: {:.1}%)", actual_slippage, self.config.max_slippage_pct);

        // Step 4: Safety checks
        if actual_slippage > self.config.max_slippage_pct {
            let error_msg = format!("Slippage too high: {:.2}% > {:.1}%", actual_slippage, self.config.max_slippage_pct);
            return self.create_failed_trade_result(trade_id, opportunity, execution_start, start_time, error_msg);
        }

        if token_a_price.confidence < 0.7 || token_b_price.confidence < 0.7 {
            let error_msg = "Price confidence too low for safe execution".to_string();
            return self.create_failed_trade_result(trade_id, opportunity, execution_start, start_time, error_msg);
        }

        // Step 5: Calculate potential profit after slippage
        let adjusted_profit = opportunity.expected_profit_usd * (1.0 - actual_slippage / 100.0);
        let estimated_gas_fees = 0.005; // ~$0.005 SOL transaction fee
        let net_profit = adjusted_profit - estimated_gas_fees;

        if net_profit < self.config.min_profit_threshold_usd {
            let error_msg = format!("Profit too low after fees: ${:.4} < ${:.2}", net_profit, self.config.min_profit_threshold_usd);
            return self.create_failed_trade_result(trade_id, opportunity, execution_start, start_time, error_msg);
        }

        // Step 6: Simulate trade execution (in real implementation, this would call Solana)
        println!("   ✅ All safety checks passed, executing trade...");
        
        // Simulate execution time
        tokio::time::sleep(Duration::from_millis(200 + rand::random::<u64>() % 300)).await;
        
        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // Check execution time limit
        if execution_time_ms > self.config.max_execution_time_ms {
            let error_msg = format!("Execution timeout: {}ms > {}ms", execution_time_ms, self.config.max_execution_time_ms);
            return self.create_failed_trade_result(trade_id, opportunity, execution_start, start_time, error_msg);
        }

        // Step 7: Create successful trade result
        let trade_result = CacheFreeTradeResult {
            trade_id: trade_id.clone(),
            opportunity: opportunity.clone(),
            executed_at: execution_start,
            execution_time_ms,
            entry_price: current_price,
            actual_slippage_pct: actual_slippage,
            profit_loss_usd: adjusted_profit,
            gas_fees_usd: estimated_gas_fees,
            net_profit_usd: net_profit,
            success: true,
            error_message: None,
        };

        println!("   🎯 TRADE EXECUTED SUCCESSFULLY");
        println!("      Execution time: {}ms", execution_time_ms);
        println!("      Net profit: ${:.4}", net_profit);
        println!("      Gas fees: ${:.4}", estimated_gas_fees);

        // Update performance metrics
        self.performance_metrics.record_trade(&trade_result);
        self.active_trades.insert(trade_id, trade_result.clone());

        Ok(trade_result)
    }

    fn create_failed_trade_result(
        &self,
        trade_id: String,
        opportunity: &TradingOpportunity,
        execution_start: DateTime<Utc>,
        start_time: Instant,
        error_message: String,
    ) -> Result<CacheFreeTradeResult> {
        println!("   ❌ TRADE REJECTED: {}", error_message);
        
        Ok(CacheFreeTradeResult {
            trade_id,
            opportunity: opportunity.clone(),
            executed_at: execution_start,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            entry_price: 0.0,
            actual_slippage_pct: 0.0,
            profit_loss_usd: 0.0,
            gas_fees_usd: 0.0,
            net_profit_usd: 0.0,
            success: false,
            error_message: Some(error_message),
        })
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> &CacheFreePerformanceMetrics {
        &self.performance_metrics
    }

    /// Get active trades
    pub fn get_active_trades(&self) -> &HashMap<String, CacheFreeTradeResult> {
        &self.active_trades
    }
}

/// Performance metrics for cache-free trading
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheFreePerformanceMetrics {
    pub total_opportunities_evaluated: u64,
    pub total_trades_executed: u64,
    pub total_trades_rejected: u64,
    pub total_profit_usd: f64,
    pub total_gas_fees_usd: f64,
    pub average_execution_time_ms: f64,
    pub average_slippage_pct: f64,
    pub success_rate_pct: f64,
    pub rejection_reasons: HashMap<String, u64>,
}

impl CacheFreePerformanceMetrics {
    pub fn new() -> Self {
        Self {
            total_opportunities_evaluated: 0,
            total_trades_executed: 0,
            total_trades_rejected: 0,
            total_profit_usd: 0.0,
            total_gas_fees_usd: 0.0,
            average_execution_time_ms: 0.0,
            average_slippage_pct: 0.0,
            success_rate_pct: 0.0,
            rejection_reasons: HashMap::new(),
        }
    }

    pub fn record_trade(&mut self, trade_result: &CacheFreeTradeResult) {
        self.total_opportunities_evaluated += 1;

        if trade_result.success {
            self.total_trades_executed += 1;
            self.total_profit_usd += trade_result.profit_loss_usd;
            self.total_gas_fees_usd += trade_result.gas_fees_usd;
            
            // Update averages
            let total_executions = self.total_trades_executed as f64;
            self.average_execution_time_ms = (self.average_execution_time_ms * (total_executions - 1.0) + trade_result.execution_time_ms as f64) / total_executions;
            self.average_slippage_pct = (self.average_slippage_pct * (total_executions - 1.0) + trade_result.actual_slippage_pct) / total_executions;
        } else {
            self.total_trades_rejected += 1;
            if let Some(error) = &trade_result.error_message {
                let count = self.rejection_reasons.get(error).unwrap_or(&0) + 1;
                self.rejection_reasons.insert(error.clone(), count);
            }
        }

        // Update success rate
        self.success_rate_pct = (self.total_trades_executed as f64 / self.total_opportunities_evaluated as f64) * 100.0;
    }
}
