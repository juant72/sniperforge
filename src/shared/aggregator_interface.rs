use anyhow::{Result, anyhow};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 🚀 ENTERPRISE AGGREGATOR INTERFACE - Shared module for all bots
/// 
/// Multi-DEX aggregation with intelligent routing, price optimization,
/// and cross-platform arbitrage detection.

#[derive(Debug, Clone)]
pub struct EnterpriseAggregatorInterface {
    pub supported_dexs: Vec<DexConfiguration>,
    pub routing_engine: RoutingEngine,
    pub price_cache: HashMap<String, PriceQuote>,
    pub arbitrage_detector: ArbitrageDetector,
    pub performance_metrics: AggregatorPerformanceMetrics,
}

#[derive(Debug, Clone)]
pub struct DexConfiguration {
    pub dex_name: String,
    pub program_id: String,
    pub api_endpoint: Option<String>,
    pub fee_structure: FeeStructure,
    pub capabilities: DexCapabilities,
    pub weight: f64, // Preference weight for routing
}

#[derive(Debug, Clone)]
pub struct FeeStructure {
    pub base_fee_bps: u16,
    pub priority_fee_multiplier: f64,
    pub has_dynamic_fees: bool,
    pub fee_tiers: Vec<FeeTier>,
}

#[derive(Debug, Clone)]
pub struct FeeTier {
    pub liquidity_threshold: u64,
    pub fee_bps: u16,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct DexCapabilities {
    pub supports_partial_fills: bool,
    pub supports_multi_hop: bool,
    pub supports_stop_loss: bool,
    pub max_slippage_bps: u16,
    pub average_execution_time_ms: u64,
    pub reliability_score: f64,
}

#[derive(Debug, Clone)]
pub struct RoutingEngine {
    pub optimization_strategy: OptimizationStrategy,
    pub max_routes_to_evaluate: u8,
    pub enable_cross_dex_arbitrage: bool,
    pub prefer_single_dex: bool,
    pub gas_price_weight: f64,
}

#[derive(Debug, Clone)]
pub enum OptimizationStrategy {
    BestPrice,
    FastestExecution,
    LowestGas,
    BalancedScore,
    Custom(CustomOptimization),
}

#[derive(Debug, Clone)]
pub struct CustomOptimization {
    pub price_weight: f64,
    pub speed_weight: f64,
    pub gas_weight: f64,
    pub reliability_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceQuote {
    pub dex_name: String,
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: u64,
    pub output_amount: u64,
    pub price_impact: f64,
    pub estimated_gas: u64,
    pub execution_time_estimate_ms: u64,
    pub route_info: RouteInformation,
    pub timestamp: u64,
    pub validity_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInformation {
    pub route_id: String,
    pub steps: Vec<RouteStep>,
    pub total_fee_bps: u16,
    pub complexity_score: u8,
    pub reliability_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteStep {
    pub step_number: u8,
    pub dex_used: String,
    pub input_mint: String,
    pub output_mint: String,
    pub pool_address: Option<String>,
    pub estimated_gas: u64,
}

#[derive(Debug, Clone)]
pub struct ArbitrageDetector {
    pub min_profit_threshold_sol: f64,
    pub max_execution_time_ms: u64,
    pub enable_cross_dex_detection: bool,
    pub triangular_arbitrage_enabled: bool,
    pub opportunities_cache: HashMap<String, ArbitrageOpportunity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub opportunity_id: String,
    pub token_pair: (String, String),
    pub buy_dex: String,
    pub sell_dex: String,
    pub profit_estimate_sol: f64,
    pub confidence_score: f64,
    pub execution_complexity: u8,
    pub estimated_total_gas: u64,
    pub discovered_at: u64,
    pub expires_at: u64,
}

#[derive(Debug, Default, Clone)]
pub struct AggregatorPerformanceMetrics {
    pub total_quotes_generated: u64,
    pub successful_routes_found: u64,
    pub arbitrage_opportunities_found: u64,
    pub average_quote_time_ms: f64,
    pub best_price_accuracy: f64, // How often we found the actual best price
    pub gas_optimization_score: f64,
    pub cross_dex_success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedQuote {
    pub quote_id: String,
    pub best_route: PriceQuote,
    pub alternative_routes: Vec<PriceQuote>,
    pub arbitrage_opportunities: Vec<ArbitrageOpportunity>,
    pub execution_recommendation: ExecutionRecommendation,
    pub risk_assessment: RiskAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecommendation {
    pub recommended_dex: String,
    pub confidence_level: f64,
    pub expected_slippage: f64,
    pub optimal_slippage_tolerance: u16,
    pub suggested_priority_fee: u64,
    pub execution_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk_score: f64, // 0.0 = low risk, 1.0 = high risk
    pub liquidity_risk: f64,
    pub slippage_risk: f64,
    pub execution_risk: f64,
    pub counterparty_risk: f64,
    pub market_conditions_risk: f64,
}

/// 🚀 ENTERPRISE IMPLEMENTATIONS
impl EnterpriseAggregatorInterface {
    /// Create new enterprise aggregator with default DEX configurations
    pub fn new() -> Self {
        debug!("🏗️ Creating enterprise aggregator interface");
        
        let supported_dexs = vec![
            Self::create_jupiter_config(),
            Self::create_raydium_config(),
            Self::create_orca_config(),
        ];

        Self {
            supported_dexs,
            routing_engine: RoutingEngine::default(),
            price_cache: HashMap::new(),
            arbitrage_detector: ArbitrageDetector::default(),
            performance_metrics: AggregatorPerformanceMetrics::default(),
        }
    }

    /// Get aggregated quote across all supported DEXs
    pub async fn get_aggregated_quote(
        &mut self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        user_preferences: Option<&OptimizationStrategy>,
    ) -> Result<AggregatedQuote> {
        debug!("🔍 Getting aggregated quote for {}/{}", input_mint, output_mint);
        debug!("   Amount: {} lamports", amount);

        let start_time = std::time::Instant::now();

        // Get quotes from all supported DEXs
        let mut all_quotes = Vec::new();
        for dex_config in &self.supported_dexs {
            match self.get_dex_quote(dex_config, input_mint, output_mint, amount).await {
                Ok(quote) => {
                    info!("✅ Got quote from {}: {} output", dex_config.dex_name, quote.output_amount);
                    all_quotes.push(quote);
                },
                Err(e) => {
                    warn!("⚠️ Failed to get quote from {}: {}", dex_config.dex_name, e);
                }
            }
        }

        if all_quotes.is_empty() {
            self.performance_metrics.total_quotes_generated += 1;
            return Err(anyhow!("No valid quotes found from any DEX"));
        }

        // Find best route based on optimization strategy
        let optimization = user_preferences.unwrap_or(&self.routing_engine.optimization_strategy);
        let best_route = self.select_best_route(&all_quotes, optimization)?;

        // Sort alternatives by score (clone to avoid move)
        let mut alternatives = all_quotes.clone();
        alternatives.retain(|q| q.route_info.route_id != best_route.route_info.route_id);
        alternatives.sort_by(|a, b| {
            self.calculate_route_score(b, optimization)
                .partial_cmp(&self.calculate_route_score(a, optimization))
                .unwrap()
        });

        // Detect arbitrage opportunities
        let arbitrage_opportunities = if self.arbitrage_detector.enable_cross_dex_detection {
            self.detect_arbitrage_opportunities(&all_quotes).await?
        } else {
            Vec::new()
        };

        // Generate execution recommendation
        let execution_recommendation = self.generate_execution_recommendation(&best_route)?;

        // Assess risks
        let risk_assessment = self.assess_risks(&best_route, &alternatives)?;

        // Update performance metrics
        let quote_time = start_time.elapsed().as_millis() as f64;
        self.performance_metrics.total_quotes_generated += 1;
        self.performance_metrics.successful_routes_found += 1;
        self.performance_metrics.arbitrage_opportunities_found += arbitrage_opportunities.len() as u64;
        self.performance_metrics.average_quote_time_ms = 
            (self.performance_metrics.average_quote_time_ms * (self.performance_metrics.total_quotes_generated - 1) as f64 + quote_time) /
            self.performance_metrics.total_quotes_generated as f64;

        let aggregated_quote = AggregatedQuote {
            quote_id: format!("agg_quote_{}", chrono::Utc::now().timestamp()),
            best_route,
            alternative_routes: alternatives,
            arbitrage_opportunities,
            execution_recommendation,
            risk_assessment,
        };

        info!("✅ Generated aggregated quote with {} alternatives and {} arbitrage opportunities", 
            aggregated_quote.alternative_routes.len(),
            aggregated_quote.arbitrage_opportunities.len()
        );

        Ok(aggregated_quote)
    }

    /// Find best arbitrage opportunities across DEXs
    pub async fn find_arbitrage_opportunities(
        &mut self,
        token_pairs: &[(String, String)],
        min_profit_sol: f64,
    ) -> Result<Vec<ArbitrageOpportunity>> {
        debug!("🔍 Scanning for arbitrage opportunities across {} pairs", token_pairs.len());

        let mut opportunities = Vec::new();

        for (input_mint, output_mint) in token_pairs {
            // Get quotes from all DEXs for both directions
            let mut buy_quotes = Vec::new();
            let mut sell_quotes = Vec::new();

            let test_amount = 1_000_000_000; // 1 SOL equivalent for testing

            for dex_config in &self.supported_dexs {
                // Buy direction (input -> output)
                if let Ok(buy_quote) = self.get_dex_quote(dex_config, input_mint, output_mint, test_amount).await {
                    buy_quotes.push(buy_quote);
                }

                // Sell direction (output -> input)
                if let Ok(sell_quote) = self.get_dex_quote(dex_config, output_mint, input_mint, test_amount).await {
                    sell_quotes.push(sell_quote);
                }
            }

            // Find arbitrage opportunities
            for buy_quote in &buy_quotes {
                for sell_quote in &sell_quotes {
                    if buy_quote.dex_name == sell_quote.dex_name {
                        continue; // Skip same DEX
                    }

                    let profit = self.calculate_arbitrage_profit(buy_quote, sell_quote, test_amount)?;
                    
                    if profit >= min_profit_sol {
                        let opportunity = ArbitrageOpportunity {
                            opportunity_id: format!("arb_{}_{}", 
                                chrono::Utc::now().timestamp(), 
                                opportunities.len()
                            ),
                            token_pair: (input_mint.clone(), output_mint.clone()),
                            buy_dex: buy_quote.dex_name.clone(),
                            sell_dex: sell_quote.dex_name.clone(),
                            profit_estimate_sol: profit,
                            confidence_score: self.calculate_arbitrage_confidence(buy_quote, sell_quote)?,
                            execution_complexity: self.calculate_execution_complexity(buy_quote, sell_quote),
                            estimated_total_gas: buy_quote.estimated_gas + sell_quote.estimated_gas,
                            discovered_at: chrono::Utc::now().timestamp() as u64,
                            expires_at: (chrono::Utc::now().timestamp() + 30) as u64, // 30 second expiry
                        };

                        info!("💰 Found arbitrage opportunity: {:.6} SOL profit ({} -> {})", 
                            profit, opportunity.buy_dex, opportunity.sell_dex);
                        opportunities.push(opportunity);
                    }
                }
            }
        }

        // Sort by profit descending
        opportunities.sort_by(|a, b| b.profit_estimate_sol.partial_cmp(&a.profit_estimate_sol).unwrap());

        info!("✅ Found {} arbitrage opportunities with profit >= {:.6} SOL", 
            opportunities.len(), min_profit_sol);

        Ok(opportunities)
    }

    /// Optimize route across multiple DEXs for best execution
    pub async fn optimize_cross_dex_route(
        &mut self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        max_hops: u8,
    ) -> Result<RouteInformation> {
        debug!("🛣️ Optimizing cross-DEX route with max {} hops", max_hops);

        if !self.routing_engine.enable_cross_dex_arbitrage {
            return Err(anyhow!("Cross-DEX routing is disabled"));
        }

        // Find intermediate tokens for routing
        let intermediate_tokens = vec![
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            "So11111111111111111111111111111111111111112".to_string(),   // SOL
        ];

        let mut best_route: Option<RouteInformation> = None;
        let mut best_output = 0u64;

        // Try direct routes first
        for dex_config in &self.supported_dexs {
            if let Ok(quote) = self.get_dex_quote(dex_config, input_mint, output_mint, amount).await {
                if quote.output_amount > best_output {
                    best_output = quote.output_amount;
                    best_route = Some(quote.route_info);
                }
            }
        }

        // Try multi-hop routes if enabled and we have hops remaining
        if max_hops > 1 {
            for intermediate in &intermediate_tokens {
                if intermediate == input_mint || intermediate == output_mint {
                    continue;
                }

                // Get best route for first hop
                let mut best_first_hop = None;
                let mut best_first_output = 0u64;

                for dex_config in &self.supported_dexs {
                    if let Ok(quote) = self.get_dex_quote(dex_config, input_mint, intermediate, amount).await {
                        if quote.output_amount > best_first_output {
                            best_first_output = quote.output_amount;
                            best_first_hop = Some(quote);
                        }
                    }
                }

                if let Some(first_hop) = best_first_hop {
                    // Get best route for second hop
                    for dex_config in &self.supported_dexs {
                        if let Ok(second_quote) = self.get_dex_quote(dex_config, intermediate, output_mint, first_hop.output_amount).await {
                            if second_quote.output_amount > best_output {
                                best_output = second_quote.output_amount;
                                
                                // Create combined route
                                let combined_route = RouteInformation {
                                    route_id: format!("cross_dex_route_{}", chrono::Utc::now().timestamp()),
                                    steps: vec![
                                        RouteStep {
                                            step_number: 1,
                                            dex_used: first_hop.dex_name.clone(),
                                            input_mint: input_mint.to_string(),
                                            output_mint: intermediate.clone(),
                                            pool_address: None,
                                            estimated_gas: first_hop.estimated_gas,
                                        },
                                        RouteStep {
                                            step_number: 2,
                                            dex_used: second_quote.dex_name.clone(),
                                            input_mint: intermediate.clone(),
                                            output_mint: output_mint.to_string(),
                                            pool_address: None,
                                            estimated_gas: second_quote.estimated_gas,
                                        },
                                    ],
                                    total_fee_bps: 60, // Estimated 0.6% for 2 hops
                                    complexity_score: 2,
                                    reliability_score: 0.85, // Slightly lower for multi-hop
                                };

                                best_route = Some(combined_route);
                            }
                        }
                    }
                }
            }
        }

        match best_route {
            Some(route) => {
                info!("✅ Optimized cross-DEX route with {} steps, estimated output: {} lamports", 
                    route.steps.len(), best_output);
                self.performance_metrics.cross_dex_success_rate = 
                    (self.performance_metrics.cross_dex_success_rate * (self.performance_metrics.total_quotes_generated - 1) as f64 + 1.0) / 
                    self.performance_metrics.total_quotes_generated as f64;
                Ok(route)
            },
            None => {
                Err(anyhow!("No viable cross-DEX route found"))
            }
        }
    }

    /// Get quote from a specific DEX
    async fn get_dex_quote(
        &self,
        dex_config: &DexConfiguration,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
    ) -> Result<PriceQuote> {
        debug!("📋 Getting quote from {}", dex_config.dex_name);

        // Simulate getting quote from DEX (in production, would call actual APIs)
        let base_output = amount - (amount * dex_config.fee_structure.base_fee_bps as u64 / 10000);
        let price_impact = (amount as f64 / 1_000_000_000.0) * 0.001; // Simplified price impact
        
        let route_info = RouteInformation {
            route_id: format!("{}_{}", dex_config.dex_name.to_lowercase(), chrono::Utc::now().timestamp()),
            steps: vec![RouteStep {
                step_number: 1,
                dex_used: dex_config.dex_name.clone(),
                input_mint: input_mint.to_string(),
                output_mint: output_mint.to_string(),
                pool_address: Some(format!("pool_{}_{}", input_mint[..8].to_string(), output_mint[..8].to_string())),
                estimated_gas: 150_000 + (dex_config.capabilities.average_execution_time_ms * 10), // Rough gas estimate
            }],
            total_fee_bps: dex_config.fee_structure.base_fee_bps,
            complexity_score: 1,
            reliability_score: dex_config.capabilities.reliability_score,
        };

        Ok(PriceQuote {
            dex_name: dex_config.dex_name.clone(),
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            input_amount: amount,
            output_amount: base_output,
            price_impact,
            estimated_gas: route_info.steps[0].estimated_gas,
            execution_time_estimate_ms: dex_config.capabilities.average_execution_time_ms,
            route_info,
            timestamp: chrono::Utc::now().timestamp() as u64,
            validity_duration_ms: 30_000, // 30 second validity
        })
    }

    /// Select best route based on optimization strategy
    fn select_best_route(&self, quotes: &[PriceQuote], strategy: &OptimizationStrategy) -> Result<PriceQuote> {
        if quotes.is_empty() {
            return Err(anyhow!("No quotes to select from"));
        }

        let best_quote = quotes.iter()
            .max_by(|a, b| {
                self.calculate_route_score(a, strategy)
                    .partial_cmp(&self.calculate_route_score(b, strategy))
                    .unwrap()
            })
            .unwrap();

        Ok(best_quote.clone())
    }

    /// Calculate route score based on optimization strategy
    fn calculate_route_score(&self, quote: &PriceQuote, strategy: &OptimizationStrategy) -> f64 {
        match strategy {
            OptimizationStrategy::BestPrice => quote.output_amount as f64,
            OptimizationStrategy::FastestExecution => 1000.0 / quote.execution_time_estimate_ms as f64,
            OptimizationStrategy::LowestGas => 1_000_000.0 / quote.estimated_gas as f64,
            OptimizationStrategy::BalancedScore => {
                let price_score = quote.output_amount as f64 / 1_000_000_000.0;
                let speed_score = 100.0 / quote.execution_time_estimate_ms as f64;
                let gas_score = 100_000.0 / quote.estimated_gas as f64;
                let reliability_score = quote.route_info.reliability_score;
                
                (price_score * 0.4) + (speed_score * 0.2) + (gas_score * 0.2) + (reliability_score * 0.2)
            },
            OptimizationStrategy::Custom(custom) => {
                let price_score = quote.output_amount as f64 / 1_000_000_000.0;
                let speed_score = 100.0 / quote.execution_time_estimate_ms as f64;
                let gas_score = 100_000.0 / quote.estimated_gas as f64;
                let reliability_score = quote.route_info.reliability_score;
                
                (price_score * custom.price_weight) + 
                (speed_score * custom.speed_weight) + 
                (gas_score * custom.gas_weight) + 
                (reliability_score * custom.reliability_weight)
            }
        }
    }

    /// Create Jupiter configuration
    fn create_jupiter_config() -> DexConfiguration {
        DexConfiguration {
            dex_name: "Jupiter".to_string(),
            program_id: "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB".to_string(),
            api_endpoint: Some("https://quote-api.jup.ag/v6".to_string()),
            fee_structure: FeeStructure {
                base_fee_bps: 25, // 0.25% average
                priority_fee_multiplier: 1.2,
                has_dynamic_fees: true,
                fee_tiers: vec![
                    FeeTier { liquidity_threshold: 1_000_000, fee_bps: 30, description: "Small trades".to_string() },
                    FeeTier { liquidity_threshold: 10_000_000, fee_bps: 25, description: "Medium trades".to_string() },
                    FeeTier { liquidity_threshold: 100_000_000, fee_bps: 20, description: "Large trades".to_string() },
                ],
            },
            capabilities: DexCapabilities {
                supports_partial_fills: true,
                supports_multi_hop: true,
                supports_stop_loss: false,
                max_slippage_bps: 1000, // 10%
                average_execution_time_ms: 2000,
                reliability_score: 0.95,
            },
            weight: 1.0,
        }
    }

    /// Create Raydium configuration
    fn create_raydium_config() -> DexConfiguration {
        DexConfiguration {
            dex_name: "Raydium".to_string(),
            program_id: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(),
            api_endpoint: None,
            fee_structure: FeeStructure {
                base_fee_bps: 25, // 0.25%
                priority_fee_multiplier: 1.0,
                has_dynamic_fees: false,
                fee_tiers: vec![
                    FeeTier { liquidity_threshold: 0, fee_bps: 25, description: "Fixed fee".to_string() },
                ],
            },
            capabilities: DexCapabilities {
                supports_partial_fills: false,
                supports_multi_hop: false,
                supports_stop_loss: false,
                max_slippage_bps: 500, // 5%
                average_execution_time_ms: 1500,
                reliability_score: 0.90,
            },
            weight: 0.8,
        }
    }

    /// Create Orca configuration
    fn create_orca_config() -> DexConfiguration {
        DexConfiguration {
            dex_name: "Orca".to_string(),
            program_id: "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".to_string(),
            api_endpoint: None,
            fee_structure: FeeStructure {
                base_fee_bps: 30, // 0.30% for Whirlpools
                priority_fee_multiplier: 1.1,
                has_dynamic_fees: true,
                fee_tiers: vec![
                    FeeTier { liquidity_threshold: 0, fee_bps: 1, description: "0.01%".to_string() },
                    FeeTier { liquidity_threshold: 0, fee_bps: 5, description: "0.05%".to_string() },
                    FeeTier { liquidity_threshold: 0, fee_bps: 30, description: "0.30%".to_string() },
                    FeeTier { liquidity_threshold: 0, fee_bps: 100, description: "1.00%".to_string() },
                ],
            },
            capabilities: DexCapabilities {
                supports_partial_fills: true,
                supports_multi_hop: true,
                supports_stop_loss: false,
                max_slippage_bps: 300, // 3%
                average_execution_time_ms: 1800,
                reliability_score: 0.92,
            },
            weight: 0.9,
        }
    }

    /// Detect arbitrage opportunities between quotes
    async fn detect_arbitrage_opportunities(&mut self, quotes: &[PriceQuote]) -> Result<Vec<ArbitrageOpportunity>> {
        let mut opportunities = Vec::new();
        
        // Simple arbitrage detection: find price differences between DEXs
        for i in 0..quotes.len() {
            for j in (i + 1)..quotes.len() {
                let quote_a = &quotes[i];
                let quote_b = &quotes[j];
                
                // Calculate price difference
                let price_a = quote_a.output_amount as f64 / quote_a.input_amount as f64;
                let price_b = quote_b.output_amount as f64 / quote_b.input_amount as f64;
                
                let price_diff = (price_a - price_b).abs();
                let profit_estimate = price_diff * quote_a.input_amount as f64 / 1_000_000_000.0; // Convert to SOL
                
                if profit_estimate >= self.arbitrage_detector.min_profit_threshold_sol {
                    let opportunity = ArbitrageOpportunity {
                        opportunity_id: format!("arb_detect_{}", opportunities.len()),
                        token_pair: (quote_a.input_mint.clone(), quote_a.output_mint.clone()),
                        buy_dex: if price_a > price_b { quote_b.dex_name.clone() } else { quote_a.dex_name.clone() },
                        sell_dex: if price_a > price_b { quote_a.dex_name.clone() } else { quote_b.dex_name.clone() },
                        profit_estimate_sol: profit_estimate,
                        confidence_score: 0.7, // Default confidence
                        execution_complexity: 2,
                        estimated_total_gas: quote_a.estimated_gas + quote_b.estimated_gas,
                        discovered_at: chrono::Utc::now().timestamp() as u64,
                        expires_at: (chrono::Utc::now().timestamp() + 30) as u64,
                    };
                    opportunities.push(opportunity);
                }
            }
        }
        
        Ok(opportunities)
    }

    /// Calculate arbitrage profit between two quotes
    fn calculate_arbitrage_profit(&self, buy_quote: &PriceQuote, sell_quote: &PriceQuote, amount: u64) -> Result<f64> {
        let buy_price = buy_quote.output_amount as f64 / buy_quote.input_amount as f64;
        let sell_price = sell_quote.input_amount as f64 / sell_quote.output_amount as f64;
        
        let gross_profit = (sell_price - buy_price) * amount as f64;
        let gas_costs = (buy_quote.estimated_gas + sell_quote.estimated_gas) as f64 * 0.000005; // Estimate gas cost in SOL
        
        let net_profit = (gross_profit - gas_costs) / 1_000_000_000.0; // Convert to SOL
        Ok(net_profit.max(0.0))
    }

    /// Calculate confidence score for arbitrage opportunity
    fn calculate_arbitrage_confidence(&self, buy_quote: &PriceQuote, sell_quote: &PriceQuote) -> Result<f64> {
        let reliability_factor = (buy_quote.route_info.reliability_score + sell_quote.route_info.reliability_score) / 2.0;
        let execution_time_factor = 1.0 / (1.0 + (buy_quote.execution_time_estimate_ms + sell_quote.execution_time_estimate_ms) as f64 / 10000.0);
        let complexity_factor = 1.0 / (1.0 + (buy_quote.route_info.complexity_score + sell_quote.route_info.complexity_score) as f64 / 10.0);
        
        Ok((reliability_factor * 0.5) + (execution_time_factor * 0.3) + (complexity_factor * 0.2))
    }

    /// Calculate execution complexity
    fn calculate_execution_complexity(&self, buy_quote: &PriceQuote, sell_quote: &PriceQuote) -> u8 {
        buy_quote.route_info.complexity_score + sell_quote.route_info.complexity_score
    }

    /// Generate execution recommendation
    fn generate_execution_recommendation(&self, quote: &PriceQuote) -> Result<ExecutionRecommendation> {
        Ok(ExecutionRecommendation {
            recommended_dex: quote.dex_name.clone(),
            confidence_level: quote.route_info.reliability_score,
            expected_slippage: quote.price_impact,
            optimal_slippage_tolerance: ((quote.price_impact * 1.5) * 10000.0) as u16, // 1.5x expected slippage
            suggested_priority_fee: quote.estimated_gas / 4, // Rough estimate
            execution_strategy: if quote.route_info.steps.len() > 1 { "multi-hop".to_string() } else { "direct".to_string() },
        })
    }

    /// Assess risks for a quote
    fn assess_risks(&self, best_quote: &PriceQuote, alternatives: &[PriceQuote]) -> Result<RiskAssessment> {
        let liquidity_risk = if alternatives.is_empty() { 0.8 } else { 0.3 }; // Higher risk if no alternatives
        let slippage_risk = (best_quote.price_impact * 10.0).min(1.0);
        let execution_risk = (best_quote.execution_time_estimate_ms as f64 / 10000.0).min(1.0);
        let counterparty_risk = 1.0 - best_quote.route_info.reliability_score;
        let market_conditions_risk = 0.5; // Default market risk
        
        let overall_risk = (liquidity_risk + slippage_risk + execution_risk + counterparty_risk + market_conditions_risk) / 5.0;
        
        Ok(RiskAssessment {
            overall_risk_score: overall_risk,
            liquidity_risk,
            slippage_risk,
            execution_risk,
            counterparty_risk,
            market_conditions_risk,
        })
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> &AggregatorPerformanceMetrics {
        &self.performance_metrics
    }

    /// Update routing configuration
    pub fn update_routing_config(&mut self, config: RoutingEngine) {
        debug!("🔧 Updating aggregator routing configuration");
        self.routing_engine = config;
    }
}

impl Default for RoutingEngine {
    fn default() -> Self {
        Self {
            optimization_strategy: OptimizationStrategy::BalancedScore,
            max_routes_to_evaluate: 5,
            enable_cross_dex_arbitrage: true,
            prefer_single_dex: false,
            gas_price_weight: 0.2,
        }
    }
}

impl Default for ArbitrageDetector {
    fn default() -> Self {
        Self {
            min_profit_threshold_sol: 0.001, // 0.001 SOL minimum
            max_execution_time_ms: 5000,
            enable_cross_dex_detection: true,
            triangular_arbitrage_enabled: false, // Complex, disabled by default
            opportunities_cache: HashMap::new(),
        }
    }
}

/// Factory functions for different bot types
impl EnterpriseAggregatorInterface {
    /// Create aggregator optimized for liquidity sniping bots
    pub fn for_liquidity_sniper() -> Self {
        let mut aggregator = Self::new();
        aggregator.routing_engine = RoutingEngine {
            optimization_strategy: OptimizationStrategy::FastestExecution,
            max_routes_to_evaluate: 3, // Limit for speed
            enable_cross_dex_arbitrage: true,
            prefer_single_dex: true, // Prefer single DEX for speed
            gas_price_weight: 0.1, // Lower gas concern for sniping
        };
        aggregator.arbitrage_detector.min_profit_threshold_sol = 0.005; // Higher threshold for sniping
        aggregator
    }

    /// Create aggregator optimized for arbitrage bots
    pub fn for_arbitrage() -> Self {
        let mut aggregator = Self::new();
        aggregator.routing_engine = RoutingEngine {
            optimization_strategy: OptimizationStrategy::BestPrice,
            max_routes_to_evaluate: 8, // More thorough search
            enable_cross_dex_arbitrage: true,
            prefer_single_dex: false,
            gas_price_weight: 0.3, // Gas matters for arb profitability
        };
        aggregator.arbitrage_detector.min_profit_threshold_sol = 0.0005; // Lower threshold for arb detection
        aggregator.arbitrage_detector.triangular_arbitrage_enabled = true;
        aggregator
    }

    /// Create aggregator optimized for market making bots
    pub fn for_market_maker() -> Self {
        let mut aggregator = Self::new();
        aggregator.routing_engine = RoutingEngine {
            optimization_strategy: OptimizationStrategy::LowestGas,
            max_routes_to_evaluate: 5,
            enable_cross_dex_arbitrage: false, // MM focuses on single markets
            prefer_single_dex: true,
            gas_price_weight: 0.4, // High gas concern for MM
        };
        aggregator
    }
}
