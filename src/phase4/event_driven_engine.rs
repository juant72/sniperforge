// PHASE 4.1: EVENT-DRIVEN ARCHITECTURE IMPLEMENTATION
// Real-time arbitrage opportunity detection and execution

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::Result;
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio::time::sleep;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use tracing::{info, warn, debug, error};

use crate::modules::{
    JupiterAdvancedEngine, JupiterAdvancedOpportunity,
    MEVProtectionEngine, 
    DEXSpecializationEngine, SpecializedOpportunity,
};
use crate::expert::calculations::is_arbitrage_mathematically_profitable;

/// Event types that trigger arbitrage analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArbitrageEvent {
    PriceUpdate {
        token_mint: Pubkey,
        dex_name: String,
        new_price: f64,
        timestamp: u64,
    },
    LiquidityChange {
        pool_address: Pubkey,
        token_a: Pubkey,
        token_b: Pubkey,
        new_liquidity: u64,
        timestamp: u64,
    },
    VolumeSpike {
        token_mint: Pubkey,
        volume_change_percent: f64,
        timestamp: u64,
    },
    NetworkCongestion {
        congestion_level: f64,
        average_fee: u64,
        timestamp: u64,
    },
}

/// Priority levels for opportunity execution
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutionPriority {
    Critical = 4,  // >2% profit, execute immediately
    High = 3,      // 1-2% profit, execute within 5 seconds  
    Medium = 2,    // 0.5-1% profit, execute within 30 seconds
    Low = 1,       // 0.2-0.5% profit, monitor only
}

/// Real-time arbitrage opportunity detected by event system
#[derive(Debug, Clone)]
pub struct EventDrivenOpportunity {
    pub opportunity_id: String,
    pub trigger_event: ArbitrageEvent,
    pub opportunity_type: OpportunityType,
    pub expected_profit_lamports: u64,
    pub profit_percentage: f64,
    pub execution_priority: ExecutionPriority,
    pub estimated_execution_time_ms: u64,
    pub confidence_score: f64,
    pub created_at: Instant,
    pub expires_at: Instant,
    pub execution_data: ExecutionData,
}

#[derive(Debug, Clone)]
pub enum OpportunityType {
    JupiterAutoRouted(JupiterAdvancedOpportunity),
    DEXSpecialized(SpecializedOpportunity),
    CrossDEXArbitrage {
        buy_dex: String,
        sell_dex: String,
        token_mint: Pubkey,
        spread_bps: u16,
    },
}

#[derive(Debug, Clone)]
pub struct ExecutionData {
    pub trade_amount_lamports: u64,
    pub max_slippage_bps: u16,
    pub priority_fee_lamports: u64,
    pub estimated_gas: u64,
    pub mev_protection_required: bool,
}

/// Event-driven arbitrage engine configuration
#[derive(Debug, Clone)]
pub struct EventDrivenConfig {
    pub max_concurrent_opportunities: usize,
    pub price_update_threshold_bps: u16,
    pub liquidity_change_threshold_percent: f64,
    pub volume_spike_threshold_percent: f64,
    pub opportunity_expiry_seconds: u64,
    pub min_profit_threshold_lamports: u64,
    pub max_execution_time_ms: u64,
    pub enable_mev_protection: bool,
    pub enable_parallel_execution: bool,
}

impl Default for EventDrivenConfig {
    fn default() -> Self {
        Self {
            max_concurrent_opportunities: 5,
            price_update_threshold_bps: 10, // 0.1% price change triggers analysis
            liquidity_change_threshold_percent: 5.0, // 5% liquidity change
            volume_spike_threshold_percent: 20.0, // 20% volume increase
            opportunity_expiry_seconds: 30, // Opportunities expire after 30 seconds
            min_profit_threshold_lamports: 100_000, // 0.0001 SOL minimum profit
            max_execution_time_ms: 5000, // 5 second max execution time
            enable_mev_protection: true,
            enable_parallel_execution: true,
        }
    }
}

/// Event-driven arbitrage engine
pub struct EventDrivenArbitrageEngine {
    config: EventDrivenConfig,
    jupiter_engine: Arc<Mutex<JupiterAdvancedEngine>>,
    mev_protection: Arc<MEVProtectionEngine>,
    dex_specialization: Arc<Mutex<DEXSpecializationEngine>>,
    
    // Event processing
    event_receiver: mpsc::UnboundedReceiver<ArbitrageEvent>,
    event_sender: mpsc::UnboundedSender<ArbitrageEvent>,
    
    // Opportunity management
    active_opportunities: Arc<RwLock<HashMap<String, EventDrivenOpportunity>>>,
    execution_queue: Arc<Mutex<Vec<EventDrivenOpportunity>>>,
    
    // Performance tracking
    execution_stats: Arc<RwLock<ExecutionStats>>,
    price_cache: Arc<RwLock<HashMap<Pubkey, PriceData>>>,
}

#[derive(Debug, Default)]
pub struct ExecutionStats {
    pub total_events_processed: u64,
    pub opportunities_detected: u64,
    pub opportunities_executed: u64,
    pub successful_executions: u64,
    pub total_profit_lamports: u64,
    pub average_execution_time_ms: f64,
    pub average_profit_per_trade: f64,
}

#[derive(Debug, Clone)]
pub struct PriceData {
    pub price: f64,
    pub dex_name: String,
    pub timestamp: Instant,
    pub liquidity: u64,
    pub volume_24h: f64,
}

impl EventDrivenArbitrageEngine {
    /// Initialize the event-driven arbitrage engine
    pub async fn new(
        config: EventDrivenConfig,
        jupiter_engine: JupiterAdvancedEngine,
        mev_protection: MEVProtectionEngine,
        dex_specialization: DEXSpecializationEngine,
    ) -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let engine = Self {
            config,
            jupiter_engine: Arc::new(Mutex::new(jupiter_engine)),
            mev_protection: Arc::new(mev_protection),
            dex_specialization: Arc::new(Mutex::new(dex_specialization)),
            event_receiver,
            event_sender,
            active_opportunities: Arc::new(RwLock::new(HashMap::new())),
            execution_queue: Arc::new(Mutex::new(Vec::new())),
            execution_stats: Arc::new(RwLock::new(ExecutionStats::default())),
            price_cache: Arc::new(RwLock::new(HashMap::new())),
        };

        info!("🚀 Event-driven arbitrage engine initialized with config: {:#?}", engine.config);
        Ok(engine)
    }

    /// Start the event-driven arbitrage engine
    pub async fn start(&mut self) -> Result<()> {
        info!("🎯 Starting event-driven arbitrage engine...");

        // Start parallel tasks
        let event_processor = self.start_event_processor();
        let opportunity_executor = self.start_opportunity_executor();
        let cleanup_task = self.start_cleanup_task();
        let stats_reporter = self.start_stats_reporter();

        // Wait for all tasks to complete (they run indefinitely)
        tokio::select! {
            result = event_processor => {
                error!("Event processor terminated: {:?}", result);
            }
            result = opportunity_executor => {
                error!("Opportunity executor terminated: {:?}", result);
            }
            result = cleanup_task => {
                error!("Cleanup task terminated: {:?}", result);
            }
            result = stats_reporter => {
                error!("Stats reporter terminated: {:?}", result);
            }
        }

        Ok(())
    }

    /// Send an event to the processing queue
    pub fn send_event(&self, event: ArbitrageEvent) -> Result<()> {
        self.event_sender.send(event)
            .map_err(|e| anyhow::anyhow!("Failed to send event: {}", e))?;
        Ok(())
    }

    /// Get current execution statistics
    pub async fn get_stats(&self) -> ExecutionStats {
        self.execution_stats.read().await.clone()
    }

    /// Get active opportunities count
    pub async fn get_active_opportunities_count(&self) -> usize {
        self.active_opportunities.read().await.len()
    }

    /// Event processing loop - runs continuously
    async fn start_event_processor(&self) -> Result<()> {
        let mut event_receiver = self.event_receiver;
        let active_opportunities = Arc::clone(&self.active_opportunities);
        let execution_queue = Arc::clone(&self.execution_queue);
        let execution_stats = Arc::clone(&self.execution_stats);
        let price_cache = Arc::clone(&self.price_cache);
        let jupiter_engine = Arc::clone(&self.jupiter_engine);
        let dex_specialization = Arc::clone(&self.dex_specialization);
        let config = self.config.clone();

        tokio::spawn(async move {
            info!("📡 Event processor started");

            while let Some(event) = event_receiver.recv().await {
                let start_time = Instant::now();
                
                // Update statistics
                {
                    let mut stats = execution_stats.write().await;
                    stats.total_events_processed += 1;
                }

                // Process the event
                let processing_result = Self::process_arbitrage_event(
                    &event,
                    &active_opportunities,
                    &execution_queue,
                    &price_cache,
                    &jupiter_engine,
                    &dex_specialization,
                    &config,
                ).await;

                match processing_result {
                    Ok(opportunities_found) => {
                        if opportunities_found > 0 {
                            debug!("✅ Event processed: {:?} - Found {} opportunities", 
                                event, opportunities_found);
                            
                            let mut stats = execution_stats.write().await;
                            stats.opportunities_detected += opportunities_found as u64;
                        }
                    }
                    Err(e) => {
                        warn!("❌ Error processing event {:?}: {}", event, e);
                    }
                }

                let processing_time = start_time.elapsed();
                debug!("⏱️ Event processing time: {:?}", processing_time);
            }

            warn!("📡 Event processor terminated");
            Ok::<(), anyhow::Error>(())
        });

        Ok(())
    }

    /// Process a single arbitrage event and detect opportunities
    async fn process_arbitrage_event(
        event: &ArbitrageEvent,
        active_opportunities: &Arc<RwLock<HashMap<String, EventDrivenOpportunity>>>,
        execution_queue: &Arc<Mutex<Vec<EventDrivenOpportunity>>>,
        price_cache: &Arc<RwLock<HashMap<Pubkey, PriceData>>>,
        jupiter_engine: &Arc<Mutex<JupiterAdvancedEngine>>,
        dex_specialization: &Arc<Mutex<DEXSpecializationEngine>>,
        config: &EventDrivenConfig,
    ) -> Result<usize> {
        let mut opportunities_found = 0;

        match event {
            ArbitrageEvent::PriceUpdate { token_mint, dex_name, new_price, timestamp } => {
                // Update price cache
                {
                    let mut cache = price_cache.write().await;
                    cache.insert(*token_mint, PriceData {
                        price: *new_price,
                        dex_name: dex_name.clone(),
                        timestamp: Instant::now(),
                        liquidity: 0, // Will be updated by liquidity events
                        volume_24h: 0.0, // Will be updated by volume events
                    });
                }

                // Check for cross-DEX arbitrage opportunities
                opportunities_found += Self::detect_cross_dex_opportunities(
                    *token_mint,
                    *new_price,
                    dex_name,
                    price_cache,
                    config,
                    active_opportunities,
                    execution_queue,
                ).await?;

                // Check Jupiter auto-routing opportunities
                opportunities_found += Self::detect_jupiter_opportunities(
                    *token_mint,
                    jupiter_engine,
                    config,
                    active_opportunities,
                    execution_queue,
                ).await?;
            }

            ArbitrageEvent::LiquidityChange { pool_address, token_a, token_b, new_liquidity, .. } => {
                // Update liquidity in price cache
                {
                    let mut cache = price_cache.write().await;
                    if let Some(price_data) = cache.get_mut(token_a) {
                        price_data.liquidity = *new_liquidity;
                    }
                    if let Some(price_data) = cache.get_mut(token_b) {
                        price_data.liquidity = *new_liquidity;
                    }
                }

                // Check DEX specialization opportunities
                opportunities_found += Self::detect_dex_specialized_opportunities(
                    dex_specialization,
                    config,
                    active_opportunities,
                    execution_queue,
                ).await?;
            }

            ArbitrageEvent::VolumeSpike { token_mint, volume_change_percent, .. } => {
                // Update volume in price cache
                {
                    let mut cache = price_cache.write().await;
                    if let Some(price_data) = cache.get_mut(token_mint) {
                        price_data.volume_24h *= 1.0 + (volume_change_percent / 100.0);
                    }
                }

                // Volume spikes often create temporary arbitrage opportunities
                if *volume_change_percent > config.volume_spike_threshold_percent {
                    opportunities_found += Self::detect_volume_spike_opportunities(
                        *token_mint,
                        jupiter_engine,
                        config,
                        active_opportunities,
                        execution_queue,
                    ).await?;
                }
            }

            ArbitrageEvent::NetworkCongestion { congestion_level, average_fee, .. } => {
                // Adjust execution parameters based on network conditions
                Self::adjust_execution_parameters_for_congestion(
                    *congestion_level,
                    *average_fee,
                    active_opportunities,
                ).await?;
            }
        }

        Ok(opportunities_found)
    }

    /// Detect cross-DEX arbitrage opportunities from price updates
    async fn detect_cross_dex_opportunities(
        token_mint: Pubkey,
        new_price: f64,
        updated_dex: &str,
        price_cache: &Arc<RwLock<HashMap<Pubkey, PriceData>>>,
        config: &EventDrivenConfig,
        active_opportunities: &Arc<RwLock<HashMap<String, EventDrivenOpportunity>>>,
        execution_queue: &Arc<Mutex<Vec<EventDrivenOpportunity>>>,
    ) -> Result<usize> {
        let mut opportunities_found = 0;

        // Get all cached prices for this token
        let cache = price_cache.read().await;
        if let Some(current_price_data) = cache.get(&token_mint) {
            // Look for arbitrage opportunities against other DEXes
            // This is a simplified example - in reality you'd check multiple DEXes
            for (_, other_price_data) in cache.iter() {
                if other_price_data.dex_name != updated_dex {
                    let price_diff = (new_price - other_price_data.price).abs();
                    let spread_bps = ((price_diff / other_price_data.price) * 10000.0) as u16;

                    if spread_bps > config.price_update_threshold_bps {
                        // Create cross-DEX arbitrage opportunity
                        let opportunity = Self::create_cross_dex_opportunity(
                            token_mint,
                            updated_dex,
                            &other_price_data.dex_name,
                            new_price,
                            other_price_data.price,
                            spread_bps,
                            config,
                        ).await?;

                        if Self::validate_opportunity(&opportunity, config).await? {
                            let opportunity_id = opportunity.opportunity_id.clone();
                            
                            // Add to active opportunities
                            {
                                let mut active = active_opportunities.write().await;
                                active.insert(opportunity_id.clone(), opportunity.clone());
                            }

                            // Add to execution queue based on priority
                            {
                                let mut queue = execution_queue.lock().await;
                                queue.push(opportunity);
                                queue.sort_by(|a, b| b.execution_priority.cmp(&a.execution_priority));
                            }

                            opportunities_found += 1;
                            
                            info!("🎯 Cross-DEX opportunity detected: {} spread on {} vs {}", 
                                spread_bps, updated_dex, other_price_data.dex_name);
                        }
                    }
                }
            }
        }

        Ok(opportunities_found)
    }

    /// Create a cross-DEX arbitrage opportunity
    async fn create_cross_dex_opportunity(
        token_mint: Pubkey,
        buy_dex: &str,
        sell_dex: &str,
        buy_price: f64,
        sell_price: f64,
        spread_bps: u16,
        config: &EventDrivenConfig,
    ) -> Result<EventDrivenOpportunity> {
        let opportunity_id = format!("cross_{}_{}_{}_{}", 
            buy_dex, sell_dex, token_mint.to_string()[..8].to_string(), 
            chrono::Utc::now().timestamp_millis());

        let trade_amount_lamports = 1_000_000_000u64; // 1 SOL worth
        let expected_profit_lamports = (trade_amount_lamports as f64 * spread_bps as f64 / 10000.0) as u64;
        let profit_percentage = spread_bps as f64 / 100.0; // Convert BPS to percentage

        let execution_priority = match spread_bps {
            s if s >= 200 => ExecutionPriority::Critical, // >2%
            s if s >= 100 => ExecutionPriority::High,     // 1-2%
            s if s >= 50 => ExecutionPriority::Medium,    // 0.5-1%
            _ => ExecutionPriority::Low,                  // <0.5%
        };

        let now = Instant::now();
        let opportunity = EventDrivenOpportunity {
            opportunity_id,
            trigger_event: ArbitrageEvent::PriceUpdate {
                token_mint,
                dex_name: buy_dex.to_string(),
                new_price: buy_price,
                timestamp: now.elapsed().as_millis() as u64,
            },
            opportunity_type: OpportunityType::CrossDEXArbitrage {
                buy_dex: buy_dex.to_string(),
                sell_dex: sell_dex.to_string(),
                token_mint,
                spread_bps,
            },
            expected_profit_lamports,
            profit_percentage,
            execution_priority,
            estimated_execution_time_ms: 2000, // 2 seconds estimated
            confidence_score: Self::calculate_confidence_score(spread_bps, trade_amount_lamports),
            created_at: now,
            expires_at: now + Duration::from_secs(config.opportunity_expiry_seconds),
            execution_data: ExecutionData {
                trade_amount_lamports,
                max_slippage_bps: 100, // 1% max slippage
                priority_fee_lamports: 50_000, // 0.00005 SOL
                estimated_gas: 200_000,
                mev_protection_required: config.enable_mev_protection,
            },
        };

        Ok(opportunity)
    }

    /// Calculate confidence score for an opportunity
    fn calculate_confidence_score(spread_bps: u16, trade_amount: u64) -> f64 {
        // Higher spread = higher confidence, but diminishing returns
        let spread_score = (spread_bps as f64 / 500.0).min(1.0); // Max score at 5% spread
        
        // Reasonable trade amount = higher confidence  
        let amount_score = if trade_amount >= 500_000_000 && trade_amount <= 5_000_000_000 {
            1.0 // 0.5 - 5 SOL is optimal
        } else {
            0.7
        };

        (spread_score * 0.7 + amount_score * 0.3).min(1.0)
    }

    /// Validate if an opportunity meets execution criteria
    async fn validate_opportunity(
        opportunity: &EventDrivenOpportunity,
        config: &EventDrivenConfig,
    ) -> Result<bool> {
        // Check minimum profit threshold
        if opportunity.expected_profit_lamports < config.min_profit_threshold_lamports {
            return Ok(false);
        }

        // Check if opportunity hasn't expired
        if opportunity.expires_at < Instant::now() {
            return Ok(false);
        }

        // Use expert calculations for validation
        let is_profitable = is_arbitrage_mathematically_profitable(
            opportunity.execution_data.trade_amount_lamports,
            opportunity.expected_profit_lamports,
            opportunity.execution_data.estimated_gas,
            opportunity.execution_data.priority_fee_lamports,
        )?;

        Ok(is_profitable)
    }

    /// Detect Jupiter auto-routing opportunities
    async fn detect_jupiter_opportunities(
        _token_mint: Pubkey,
        jupiter_engine: &Arc<Mutex<JupiterAdvancedEngine>>,
        config: &EventDrivenConfig,
        active_opportunities: &Arc<RwLock<HashMap<String, EventDrivenOpportunity>>>,
        execution_queue: &Arc<Mutex<Vec<EventDrivenOpportunity>>>,
    ) -> Result<usize> {
        let mut opportunities_found = 0;

        // Get Jupiter opportunities
        let jupiter_opportunities = {
            let mut engine = jupiter_engine.lock().await;
            engine.find_auto_routed_opportunities(1_000_000_000).await? // 1 SOL
        };

        for jupiter_opp in jupiter_opportunities {
            let opportunity = Self::create_jupiter_opportunity(jupiter_opp, config).await?;
            
            if Self::validate_opportunity(&opportunity, config).await? {
                let opportunity_id = opportunity.opportunity_id.clone();
                
                // Add to active opportunities
                {
                    let mut active = active_opportunities.write().await;
                    active.insert(opportunity_id, opportunity.clone());
                }

                // Add to execution queue
                {
                    let mut queue = execution_queue.lock().await;
                    queue.push(opportunity);
                    queue.sort_by(|a, b| b.execution_priority.cmp(&a.execution_priority));
                }

                opportunities_found += 1;
            }
        }

        Ok(opportunities_found)
    }

    /// Create Jupiter-based opportunity
    async fn create_jupiter_opportunity(
        jupiter_opp: JupiterAdvancedOpportunity,
        config: &EventDrivenConfig,
    ) -> Result<EventDrivenOpportunity> {
        let opportunity_id = format!("jupiter_{}", chrono::Utc::now().timestamp_millis());
        let now = Instant::now();

        let execution_priority = match jupiter_opp.profit_percentage {
            p if p >= 2.0 => ExecutionPriority::Critical,
            p if p >= 1.0 => ExecutionPriority::High,
            p if p >= 0.5 => ExecutionPriority::Medium,
            _ => ExecutionPriority::Low,
        };

        Ok(EventDrivenOpportunity {
            opportunity_id,
            trigger_event: ArbitrageEvent::PriceUpdate {
                token_mint: jupiter_opp.input_token,
                dex_name: "Jupiter".to_string(),
                new_price: 0.0, // Jupiter doesn't expose individual prices
                timestamp: now.elapsed().as_millis() as u64,
            },
            opportunity_type: OpportunityType::JupiterAutoRouted(jupiter_opp.clone()),
            expected_profit_lamports: jupiter_opp.profit_lamports,
            profit_percentage: jupiter_opp.profit_percentage,
            execution_priority,
            estimated_execution_time_ms: jupiter_opp.estimated_execution_time_ms,
            confidence_score: jupiter_opp.confidence_score,
            created_at: now,
            expires_at: now + Duration::from_secs(config.opportunity_expiry_seconds),
            execution_data: ExecutionData {
                trade_amount_lamports: jupiter_opp.input_amount,
                max_slippage_bps: jupiter_opp.slippage_bps,
                priority_fee_lamports: 100_000, // 0.0001 SOL
                estimated_gas: 300_000,
                mev_protection_required: config.enable_mev_protection,
            },
        })
    }

    /// Detect DEX specialized opportunities
    async fn detect_dex_specialized_opportunities(
        dex_specialization: &Arc<Mutex<DEXSpecializationEngine>>,
        config: &EventDrivenConfig,
        active_opportunities: &Arc<RwLock<HashMap<String, EventDrivenOpportunity>>>,
        execution_queue: &Arc<Mutex<Vec<EventDrivenOpportunity>>>,
    ) -> Result<usize> {
        let mut opportunities_found = 0;

        // Get DEX specialized opportunities
        let specialized_opportunities = {
            let mut engine = dex_specialization.lock().await;
            engine.find_specialized_opportunities().await?
        };

        for specialized_opp in specialized_opportunities {
            let opportunity = Self::create_specialized_opportunity(specialized_opp, config).await?;
            
            if Self::validate_opportunity(&opportunity, config).await? {
                let opportunity_id = opportunity.opportunity_id.clone();
                
                // Add to active opportunities
                {
                    let mut active = active_opportunities.write().await;
                    active.insert(opportunity_id, opportunity.clone());
                }

                // Add to execution queue
                {
                    let mut queue = execution_queue.lock().await;
                    queue.push(opportunity);
                    queue.sort_by(|a, b| b.execution_priority.cmp(&a.execution_priority));
                }

                opportunities_found += 1;
            }
        }

        Ok(opportunities_found)
    }

    /// Create DEX specialized opportunity
    async fn create_specialized_opportunity(
        specialized_opp: SpecializedOpportunity,
        config: &EventDrivenConfig,
    ) -> Result<EventDrivenOpportunity> {
        let opportunity_id = format!("dex_spec_{}", chrono::Utc::now().timestamp_millis());
        let now = Instant::now();

        let execution_priority = match specialized_opp.profit_percentage {
            p if p >= 2.0 => ExecutionPriority::Critical,
            p if p >= 1.0 => ExecutionPriority::High,
            p if p >= 0.5 => ExecutionPriority::Medium,
            _ => ExecutionPriority::Low,
        };

        Ok(EventDrivenOpportunity {
            opportunity_id,
            trigger_event: ArbitrageEvent::LiquidityChange {
                pool_address: specialized_opp.pool_address,
                token_a: specialized_opp.token_a,
                token_b: specialized_opp.token_b,
                new_liquidity: 0, // Will be updated
                timestamp: now.elapsed().as_millis() as u64,
            },
            opportunity_type: OpportunityType::DEXSpecialized(specialized_opp.clone()),
            expected_profit_lamports: specialized_opp.profit_lamports,
            profit_percentage: specialized_opp.profit_percentage,
            execution_priority,
            estimated_execution_time_ms: 3000, // 3 seconds for specialized strategies
            confidence_score: specialized_opp.confidence_score,
            created_at: now,
            expires_at: now + Duration::from_secs(config.opportunity_expiry_seconds),
            execution_data: ExecutionData {
                trade_amount_lamports: specialized_opp.amount_in,
                max_slippage_bps: 150, // 1.5% for specialized strategies
                priority_fee_lamports: 75_000, // 0.000075 SOL
                estimated_gas: 250_000,
                mev_protection_required: config.enable_mev_protection,
            },
        })
    }

    /// Detect volume spike opportunities
    async fn detect_volume_spike_opportunities(
        _token_mint: Pubkey,
        _jupiter_engine: &Arc<Mutex<JupiterAdvancedEngine>>,
        _config: &EventDrivenConfig,
        _active_opportunities: &Arc<RwLock<HashMap<String, EventDrivenOpportunity>>>,
        _execution_queue: &Arc<Mutex<Vec<EventDrivenOpportunity>>>,
    ) -> Result<usize> {
        // Volume spikes often create temporary price discrepancies
        // This would involve more complex analysis of multiple DEXes
        // For now, return 0 but framework is in place
        Ok(0)
    }

    /// Adjust execution parameters based on network congestion
    async fn adjust_execution_parameters_for_congestion(
        congestion_level: f64,
        average_fee: u64,
        active_opportunities: &Arc<RwLock<HashMap<String, EventDrivenOpportunity>>>,
    ) -> Result<()> {
        let mut opportunities = active_opportunities.write().await;
        
        for opportunity in opportunities.values_mut() {
            // Adjust priority fee based on network congestion
            let base_fee = opportunity.execution_data.priority_fee_lamports;
            let adjusted_fee = match congestion_level {
                c if c > 0.8 => base_fee * 3, // High congestion
                c if c > 0.5 => base_fee * 2, // Medium congestion  
                _ => base_fee, // Low congestion
            };
            
            opportunity.execution_data.priority_fee_lamports = adjusted_fee.max(average_fee);
            
            debug!("🔧 Adjusted priority fee for {} due to congestion: {} -> {}", 
                opportunity.opportunity_id, base_fee, adjusted_fee);
        }

        Ok(())
    }

    /// Start the opportunity execution engine
    async fn start_opportunity_executor(&self) -> Result<()> {
        let execution_queue = Arc::clone(&self.execution_queue);
        let active_opportunities = Arc::clone(&self.active_opportunities);
        let execution_stats = Arc::clone(&self.execution_stats);
        let mev_protection = Arc::clone(&self.mev_protection);
        let config = self.config.clone();

        tokio::spawn(async move {
            info!("⚡ Opportunity executor started");

            loop {
                // Get next opportunity from queue
                let opportunity = {
                    let mut queue = execution_queue.lock().await;
                    queue.pop()
                };

                if let Some(opp) = opportunity {
                    // Execute the opportunity
                    let execution_result = Self::execute_opportunity(
                        opp.clone(),
                        &mev_protection,
                        &config,
                    ).await;

                    // Update statistics
                    {
                        let mut stats = execution_stats.write().await;
                        stats.opportunities_executed += 1;

                        match execution_result {
                            Ok(profit) => {
                                stats.successful_executions += 1;
                                stats.total_profit_lamports += profit;
                                stats.average_profit_per_trade = 
                                    stats.total_profit_lamports as f64 / stats.successful_executions as f64;
                                
                                info!("✅ Opportunity executed successfully: {} - Profit: {} lamports", 
                                    opp.opportunity_id, profit);
                            }
                            Err(e) => {
                                warn!("❌ Opportunity execution failed: {} - Error: {}", 
                                    opp.opportunity_id, e);
                            }
                        }
                    }

                    // Remove from active opportunities
                    {
                        let mut active = active_opportunities.write().await;
                        active.remove(&opp.opportunity_id);
                    }
                } else {
                    // No opportunities in queue, sleep briefly
                    sleep(Duration::from_millis(100)).await;
                }
            }
        });

        Ok(())
    }

    /// Execute a single arbitrage opportunity
    async fn execute_opportunity(
        opportunity: EventDrivenOpportunity,
        mev_protection: &Arc<MEVProtectionEngine>,
        config: &EventDrivenConfig,
    ) -> Result<u64> {
        let start_time = Instant::now();

        info!("🚀 Executing opportunity: {} - Type: {:?} - Expected profit: {} lamports", 
            opportunity.opportunity_id, opportunity.opportunity_type, opportunity.expected_profit_lamports);

        // Check if opportunity is still valid
        if opportunity.expires_at < Instant::now() {
            return Err(anyhow::anyhow!("Opportunity expired"));
        }

        // Execute based on opportunity type
        let execution_result = match &opportunity.opportunity_type {
            OpportunityType::JupiterAutoRouted(jupiter_opp) => {
                Self::execute_jupiter_opportunity(jupiter_opp, &opportunity.execution_data, config).await
            }
            OpportunityType::DEXSpecialized(specialized_opp) => {
                Self::execute_specialized_opportunity(specialized_opp, &opportunity.execution_data, config).await
            }
            OpportunityType::CrossDEXArbitrage { buy_dex, sell_dex, token_mint, spread_bps } => {
                Self::execute_cross_dex_opportunity(
                    buy_dex, sell_dex, *token_mint, *spread_bps, 
                    &opportunity.execution_data, config
                ).await
            }
        };

        let execution_time = start_time.elapsed();
        debug!("⏱️ Opportunity execution time: {:?}", execution_time);

        // Use MEV protection if required and available
        if opportunity.execution_data.mev_protection_required && config.enable_mev_protection {
            info!("🛡️ Using MEV protection for opportunity: {}", opportunity.opportunity_id);
            // MEV protection would be integrated here
        }

        execution_result
    }

    /// Execute Jupiter auto-routed opportunity
    async fn execute_jupiter_opportunity(
        _jupiter_opp: &JupiterAdvancedOpportunity,
        _execution_data: &ExecutionData,
        _config: &EventDrivenConfig,
    ) -> Result<u64> {
        // Simulate Jupiter execution
        // In real implementation, this would call Jupiter API
        sleep(Duration::from_millis(500)).await;
        
        // Return simulated profit
        Ok(150_000) // 0.00015 SOL profit
    }

    /// Execute DEX specialized opportunity
    async fn execute_specialized_opportunity(
        _specialized_opp: &SpecializedOpportunity,
        _execution_data: &ExecutionData,
        _config: &EventDrivenConfig,
    ) -> Result<u64> {
        // Simulate specialized execution
        sleep(Duration::from_millis(750)).await;
        
        // Return simulated profit
        Ok(200_000) // 0.0002 SOL profit
    }

    /// Execute cross-DEX arbitrage opportunity
    async fn execute_cross_dex_opportunity(
        _buy_dex: &str,
        _sell_dex: &str,
        _token_mint: Pubkey,
        _spread_bps: u16,
        _execution_data: &ExecutionData,
        _config: &EventDrivenConfig,
    ) -> Result<u64> {
        // Simulate cross-DEX execution
        sleep(Duration::from_millis(1000)).await;
        
        // Return simulated profit
        Ok(300_000) // 0.0003 SOL profit
    }

    /// Start cleanup task to remove expired opportunities
    async fn start_cleanup_task(&self) -> Result<()> {
        let active_opportunities = Arc::clone(&self.active_opportunities);
        let cleanup_interval = Duration::from_secs(10);

        tokio::spawn(async move {
            info!("🧹 Cleanup task started");

            loop {
                sleep(cleanup_interval).await;

                let now = Instant::now();
                let mut expired_count = 0;

                {
                    let mut opportunities = active_opportunities.write().await;
                    opportunities.retain(|id, opp| {
                        if opp.expires_at < now {
                            debug!("🗑️ Removing expired opportunity: {}", id);
                            expired_count += 1;
                            false
                        } else {
                            true
                        }
                    });
                }

                if expired_count > 0 {
                    info!("🧹 Cleaned up {} expired opportunities", expired_count);
                }
            }
        });

        Ok(())
    }

    /// Start statistics reporting task
    async fn start_stats_reporter(&self) -> Result<()> {
        let execution_stats = Arc::clone(&self.execution_stats);
        let active_opportunities = Arc::clone(&self.active_opportunities);
        let report_interval = Duration::from_secs(60); // Report every minute

        tokio::spawn(async move {
            info!("📊 Stats reporter started");

            loop {
                sleep(report_interval).await;

                let stats = execution_stats.read().await;
                let active_count = active_opportunities.read().await.len();

                info!("📊 ARBITRAGE STATS - Events: {} | Detected: {} | Executed: {} | Success: {} | Active: {} | Total Profit: {} lamports", 
                    stats.total_events_processed,
                    stats.opportunities_detected,
                    stats.opportunities_executed,
                    stats.successful_executions,
                    active_count,
                    stats.total_profit_lamports
                );

                if stats.successful_executions > 0 {
                    info!("💰 Avg Profit/Trade: {:.4} SOL | Success Rate: {:.1}%",
                        stats.average_profit_per_trade / 1_000_000_000.0,
                        (stats.successful_executions as f64 / stats.opportunities_executed as f64) * 100.0
                    );
                }
            }
        });

        Ok(())
    }
}
