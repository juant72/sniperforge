// PHASE 4.2: PARALLEL EXECUTION ENGINE IMPLEMENTATION
// High-performance parallel arbitrage execution with real concurrency control

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::Result;
use tokio::sync::{mpsc, RwLock, Mutex, Semaphore};
use tokio::time::sleep;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use tracing::{info, warn, debug, error};

use crate::phase4::event_driven_engine::{EventDrivenOpportunity, ExecutionPriority, OpportunityType, JupiterOpportunity, DEXOpportunity};
use crate::phase2::MEVProtectionEngine;

/// Parallel execution configuration
#[derive(Debug, Clone)]
pub struct ParallelExecutionConfig {
    pub max_concurrent_executions: usize,
    pub max_concurrent_per_dex: usize,
    pub max_concurrent_per_token: usize,
    pub execution_timeout_ms: u64,
    pub retry_attempts: u8,
    pub retry_delay_ms: u64,
    pub enable_resource_isolation: bool,
    pub enable_execution_batching: bool,
    pub batch_size: usize,
    pub batch_timeout_ms: u64,
}

impl Default for ParallelExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_executions: 10, // 10 parallel executions max
            max_concurrent_per_dex: 3,     // Max 3 per DEX to avoid rate limits
            max_concurrent_per_token: 2,   // Max 2 per token to avoid conflicts
            execution_timeout_ms: 10000,   // 10 second timeout
            retry_attempts: 3,
            retry_delay_ms: 1000,          // 1 second retry delay
            enable_resource_isolation: true,
            enable_execution_batching: true,
            batch_size: 5,
            batch_timeout_ms: 2000,        // 2 second batch timeout
        }
    }
}

/// Execution status tracking
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Timeout,
    Retrying,
}

/// Execution result with detailed information
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub opportunity_id: String,
    pub status: ExecutionStatus,
    pub profit_lamports: u64,
    pub execution_time_ms: u64,
    pub gas_used: u64,
    pub transaction_signature: Option<String>,
    pub error_message: Option<String>,
    pub retry_count: u8,
    pub completed_at: Instant,
}

/// Resource tracking for parallel execution
#[derive(Debug, Default, Clone)]
pub struct ResourceTracker {
    pub active_executions_by_dex: HashMap<String, usize>,
    pub active_executions_by_token: HashMap<Pubkey, usize>,
    pub total_active_executions: usize,
    pub pending_executions: usize,
    pub completed_executions: usize,
    pub failed_executions: usize,
}

/// Execution request for parallel processing
#[derive(Debug, Clone)]
pub struct ExecutionRequest {
    pub id: String,
    pub opportunity_type: String,
    pub input_token_mint: String,
    pub output_token_mint: String,
    pub input_amount_lamports: u64,
    pub expected_output_lamports: u64,
    pub max_slippage_bps: u32,
    pub priority: ExecutionPriority,
    pub timeout_seconds: u64,
    pub created_at: Instant,
}

/// Execution batch for optimized processing
#[derive(Debug, Clone)]
pub struct ExecutionBatch {
    pub batch_id: String,
    pub opportunities: Vec<EventDrivenOpportunity>,
    pub created_at: Instant,
    pub priority: ExecutionPriority,
    pub estimated_batch_time_ms: u64,
}

/// High-performance parallel execution engine
pub struct ParallelExecutionEngine {
    config: ParallelExecutionConfig,
    
    // Concurrency control
    execution_semaphore: Arc<Semaphore>,
    dex_semaphores: Arc<RwLock<HashMap<String, Arc<Semaphore>>>>,
    token_semaphores: Arc<RwLock<HashMap<Pubkey, Arc<Semaphore>>>>,
    
    // Execution management
    execution_queue: Arc<Mutex<Vec<EventDrivenOpportunity>>>,
    execution_results: Arc<RwLock<HashMap<String, ExecutionResult>>>,
    batch_queue: Arc<Mutex<Vec<ExecutionBatch>>>,
    
    // Resource tracking
    resource_tracker: Arc<RwLock<ResourceTracker>>,
    
    // MEV protection
    mev_protection: Arc<MEVProtectionEngine>,
    
    // Performance metrics
    performance_metrics: Arc<RwLock<ParallelExecutionMetrics>>,
}

#[derive(Debug, Clone, Default)]
pub struct ParallelExecutionMetrics {
    pub total_opportunities_processed: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub timeout_executions: u64,
    pub total_profit_lamports: u64,
    pub average_execution_time_ms: f64,
    pub peak_concurrent_executions: usize,
    pub total_batches_processed: u64,
    pub average_batch_size: f64,
    pub concurrency_efficiency: f64, // % of time at max concurrency
}

impl ParallelExecutionEngine {
    /// Initialize the parallel execution engine
    pub async fn new(
        config: ParallelExecutionConfig,
        mev_protection: MEVProtectionEngine,
    ) -> Result<Self> {
        let execution_semaphore = Arc::new(Semaphore::new(config.max_concurrent_executions));
        
        let engine = Self {
            config: config.clone(),
            execution_semaphore,
            dex_semaphores: Arc::new(RwLock::new(HashMap::new())),
            token_semaphores: Arc::new(RwLock::new(HashMap::new())),
            execution_queue: Arc::new(Mutex::new(Vec::new())),
            execution_results: Arc::new(RwLock::new(HashMap::new())),
            batch_queue: Arc::new(Mutex::new(Vec::new())),
            resource_tracker: Arc::new(RwLock::new(ResourceTracker::default())),
            mev_protection: Arc::new(mev_protection),
            performance_metrics: Arc::new(RwLock::new(ParallelExecutionMetrics::default())),
        };

        info!("⚡ Parallel execution engine initialized with config: {:#?}", config);
        Ok(engine)
    }

    /// Start the parallel execution engine
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting parallel execution engine...");

        // Start parallel tasks
        let queue_processor = self.start_queue_processor();
        let batch_processor = self.start_batch_processor();
        let resource_monitor = self.start_resource_monitor();
        let metrics_reporter = self.start_metrics_reporter();

        // Wait for all tasks (they run indefinitely)
        tokio::select! {
            result = queue_processor => {
                error!("Queue processor terminated: {:?}", result);
            }
            result = batch_processor => {
                error!("Batch processor terminated: {:?}", result);
            }
            result = resource_monitor => {
                error!("Resource monitor terminated: {:?}", result);
            }
            result = metrics_reporter => {
                error!("Metrics reporter terminated: {:?}", result);
            }
        }

        Ok(())
    }

    /// Submit an opportunity for parallel execution
    pub async fn submit_opportunity(&self, opportunity: EventDrivenOpportunity) -> Result<()> {
        // Add to execution queue
        {
            let mut queue = self.execution_queue.lock().await;
            queue.push(opportunity.clone());
            
            // Sort by priority (highest first)
            queue.sort_by(|a, b| b.execution_priority.cmp(&a.execution_priority));
        }

        // Update pending count
        {
            let mut tracker = self.resource_tracker.write().await;
            tracker.pending_executions += 1;
        }

        debug!("📥 Opportunity submitted for parallel execution: {}", opportunity.opportunity_id);
        Ok(())
    }

    /// Submit an execution request for parallel processing
    pub async fn submit_execution(&self, execution_request: ExecutionRequest) -> Result<String> {
        // Convert ExecutionRequest to EventDrivenOpportunity for internal processing
        let opportunity = EventDrivenOpportunity {
            id: execution_request.id.clone(),
            opportunity_type: match execution_request.opportunity_type.as_str() {
                "Jupiter" => OpportunityType::JupiterAutoRouted(JupiterOpportunity {
                    input_token: execution_request.input_token_mint.clone(),
                    route_info: "auto-routed".to_string(),
                    input_amount: execution_request.input_amount_lamports,
                    output_amount: execution_request.expected_output_lamports,
                    price_impact: 0.1,
                    slippage_bps: 100,
                    profit_lamports: execution_request.expected_output_lamports - execution_request.input_amount_lamports,
                    profit_percentage: ((execution_request.expected_output_lamports - execution_request.input_amount_lamports) as f64 / execution_request.input_amount_lamports as f64) * 100.0,
                    confidence_score: 0.8,
                    estimated_execution_time_ms: 2000,
                }),
                "Cross-DEX" => OpportunityType::CrossDEXArbitrage {
                    buy_dex: "Raydium".to_string(),
                    sell_dex: "Orca".to_string(),
                    token_mint: execution_request.input_token_mint.parse().unwrap_or_default(),
                    spread_bps: 100,
                },
                _ => OpportunityType::DEXSpecialized(DEXOpportunity {
                    input_token: execution_request.input_token_mint.parse().unwrap_or_default(),
                    dex_type: "Raydium".to_string(),
                    dex_name: "Raydium".to_string(),
                    pool_address: "placeholder".to_string(),
                    token_a: execution_request.input_token_mint.parse().unwrap_or_default(),
                    token_b: execution_request.output_token_mint.parse().unwrap_or_default(),
                    amount_in: execution_request.input_amount_lamports,
                    opportunity_score: 0.8,
                    profit_lamports: execution_request.expected_output_lamports - execution_request.input_amount_lamports,
                    profit_percentage: ((execution_request.expected_output_lamports - execution_request.input_amount_lamports) as f64 / execution_request.input_amount_lamports as f64) * 100.0,
                    confidence_score: 0.8,
                }),
            },
            input_token: execution_request.input_token_mint,
            output_token: execution_request.output_token_mint,
            input_amount_lamports: execution_request.input_amount_lamports,
            expected_output_lamports: execution_request.expected_output_lamports,
            expected_profit_lamports: execution_request.expected_output_lamports - execution_request.input_amount_lamports,
            profit_percentage: ((execution_request.expected_output_lamports - execution_request.input_amount_lamports) as f64 / execution_request.input_amount_lamports as f64) * 100.0,
            execution_priority: execution_request.priority,
            estimated_execution_time_ms: 2000,
            confidence_score: 0.8,
            created_at: execution_request.created_at,
            expires_at: execution_request.created_at + std::time::Duration::from_secs(30),
            max_slippage_bps: execution_request.max_slippage_bps,
            timeout_seconds: 30,
            validated_at: None,
            trigger_event: crate::phase4::event_driven_engine::ArbitrageEvent::PriceUpdate {
                token_mint: execution_request.input_token_mint.parse().unwrap_or_default(),
                dex_name: "Unknown".to_string(),
                new_price: 0.0,
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64,
            },
            execution_data: crate::phase4::event_driven_engine::ExecutionData {
                trade_amount_lamports: execution_request.input_amount_lamports,
                max_slippage_bps: execution_request.max_slippage_bps as u16,
                priority_fee_lamports: 50_000,
                estimated_gas: 200_000,
                mev_protection_required: true,
            },
            opportunity_id: execution_request.id.clone(),
        };

        // Submit the opportunity
        self.submit_opportunity(opportunity).await?;
        Ok(execution_request.id)
    }

    /// Submit a batch of opportunities for optimized execution
    pub async fn submit_batch(&self, opportunities: Vec<EventDrivenOpportunity>) -> Result<String> {
        if opportunities.is_empty() {
            return Err(anyhow::anyhow!("Cannot submit empty batch"));
        }

        let batch_id = format!("batch_{}", chrono::Utc::now().timestamp_millis());
        
        // Calculate batch priority (highest priority opportunity in batch)
        let batch_priority = opportunities.iter()
            .map(|opp| &opp.execution_priority)
            .max()
            .unwrap_or(&ExecutionPriority::Low)
            .clone();

        // Estimate batch execution time
        let estimated_batch_time_ms = opportunities.iter()
            .map(|opp| opp.estimated_execution_time_ms)
            .sum::<u64>() / self.config.max_concurrent_executions as u64;

        let batch = ExecutionBatch {
            batch_id: batch_id.clone(),
            opportunities,
            created_at: Instant::now(),
            priority: batch_priority,
            estimated_batch_time_ms,
        };

        let opportunities_count = batch.opportunities.len();

        // Add to batch queue
        {
            let mut queue = self.batch_queue.lock().await;
            queue.push(batch);
            
            // Sort by priority
            queue.sort_by(|a, b| b.priority.cmp(&a.priority));
        }

        info!("📦 Batch submitted for execution: {} with {} opportunities", 
            batch_id, opportunities_count);
        
        Ok(batch_id)
    }

    /// Get execution result for an opportunity
    pub async fn get_execution_result(&self, opportunity_id: &str) -> Option<ExecutionResult> {
        let results = self.execution_results.read().await;
        results.get(opportunity_id).cloned()
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> ParallelExecutionMetrics {
        let metrics = self.performance_metrics.read().await;
        metrics.clone()
    }

    /// Get current resource usage
    pub async fn get_resource_usage(&self) -> ResourceTracker {
        self.resource_tracker.read().await.clone()
    }

    /// Start the main queue processor
    async fn start_queue_processor(&self) -> Result<()> {
        let execution_queue = Arc::clone(&self.execution_queue);
        let execution_semaphore = Arc::clone(&self.execution_semaphore);
        let dex_semaphores = Arc::clone(&self.dex_semaphores);
        let token_semaphores = Arc::clone(&self.token_semaphores);
        let execution_results = Arc::clone(&self.execution_results);
        let resource_tracker = Arc::clone(&self.resource_tracker);
        let performance_metrics = Arc::clone(&self.performance_metrics);
        let mev_protection = Arc::clone(&self.mev_protection);
        let config = self.config.clone();

        tokio::spawn(async move {
            info!("🔄 Queue processor started");

            loop {
                // Get next opportunity from queue
                let opportunity = {
                    let mut queue = execution_queue.lock().await;
                    queue.pop()
                };

                if let Some(opp) = opportunity {
                    // Try to acquire execution permit
                    if let Ok(permit) = execution_semaphore.try_acquire() {
                        // Get DEX and token names
                        let dex_name = Self::extract_dex_name(&opp);
                        let token_mint = Self::extract_token_mint(&opp);
                        
                        // Get semaphores
                        let dex_sem_result = Self::acquire_dex_semaphore(
                            &dex_semaphores, 
                            &dex_name, 
                            config.max_concurrent_per_dex
                        ).await;

                        let token_sem_result = Self::acquire_token_semaphore(
                            &token_semaphores,
                            &token_mint,
                            config.max_concurrent_per_token
                        ).await;

                        if let (Ok(dex_sem), Ok(token_sem)) = (dex_sem_result, token_sem_result) {
                            // Try to acquire the specific semaphores
                            if let (Ok(_dex_permit), Ok(_token_permit)) = (
                                dex_sem.try_acquire(),
                                token_sem.try_acquire()
                            ) {
                                // Update resource tracking
                                Self::update_resource_tracking_start(&resource_tracker, &opp).await;

                            // Execute in parallel
                            let opp_clone = opp.clone();
                            let results_clone = Arc::clone(&execution_results);
                            let tracker_clone = Arc::clone(&resource_tracker);
                            let metrics_clone = Arc::clone(&performance_metrics);
                            let mev_clone = Arc::clone(&mev_protection);
                            let config_clone = config.clone();

                            tokio::spawn(async move {
                                let execution_result = Self::execute_opportunity_parallel(
                                    opp_clone,
                                    &mev_clone,
                                    &config_clone,
                                ).await;

                                // Store result
                                {
                                    let mut results = results_clone.write().await;
                                    results.insert(execution_result.opportunity_id.clone(), execution_result.clone());
                                }

                                // Update tracking
                                Self::update_resource_tracking_complete(&tracker_clone, &execution_result).await;
                                Self::update_performance_metrics(&metrics_clone, &execution_result).await;

                                // Release permits (they are dropped automatically)
                                drop(permit);

                                debug!("✅ Parallel execution completed: {}", execution_result.opportunity_id);
                            });
                            } else {
                                // Put opportunity back in queue if can't acquire specific permits
                                let mut queue = execution_queue.lock().await;
                                queue.push(opp);
                                drop(permit);
                            }
                        } else {
                            // Put opportunity back in queue if can't acquire semaphores
                            let mut queue = execution_queue.lock().await;
                            queue.push(opp);
                            drop(permit);
                        }
                    } else {
                        // Put opportunity back in queue if can't acquire main permit
                        let mut queue = execution_queue.lock().await;
                        queue.push(opp);
                    }
                } else {
                    // No opportunities in queue, sleep briefly
                    sleep(Duration::from_millis(50)).await;
                }
            }
        });

        Ok(())
    }

    /// Start the batch processor for optimized execution
    async fn start_batch_processor(&self) -> Result<()> {
        let batch_queue = Arc::clone(&self.batch_queue);
        let execution_semaphore = Arc::clone(&self.execution_semaphore);
        let execution_results = Arc::clone(&self.execution_results);
        let performance_metrics = Arc::clone(&self.performance_metrics);
        let mev_protection = Arc::clone(&self.mev_protection);
        let config = self.config.clone();

        tokio::spawn(async move {
            info!("📦 Batch processor started");

            loop {
                if !config.enable_execution_batching {
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }

                // Get next batch from queue
                let batch = {
                    let mut queue = batch_queue.lock().await;
                    queue.pop()
                };

                if let Some(batch) = batch {
                    info!("📦 Processing batch: {} with {} opportunities", 
                        batch.batch_id, batch.opportunities.len());

                    let batch_start = Instant::now();

                    // Execute all opportunities in batch concurrently
                    let batch_futures: Vec<_> = batch.opportunities.into_iter().map(|opp| {
                        let semaphore = Arc::clone(&execution_semaphore);
                        let results = Arc::clone(&execution_results);
                        let mev = Arc::clone(&mev_protection);
                        let config = config.clone();

                        async move {
                            // Try to acquire permit with timeout
                            let permit = tokio::time::timeout(
                                Duration::from_millis(config.batch_timeout_ms),
                                semaphore.acquire()
                            ).await;

                            match permit {
                                Ok(Ok(_permit)) => {
                                    let result = Self::execute_opportunity_parallel(
                                        opp,
                                        &mev,
                                        &config,
                                    ).await;

                                    // Store result
                                    {
                                        let mut results = results.write().await;
                                        results.insert(result.opportunity_id.clone(), result.clone());
                                    }

                                    result
                                }
                                _ => {
                                    // Timeout or permit acquisition failed
                                    ExecutionResult {
                                        opportunity_id: opp.opportunity_id,
                                        status: ExecutionStatus::Timeout,
                                        profit_lamports: 0,
                                        execution_time_ms: config.batch_timeout_ms,
                                        gas_used: 0,
                                        transaction_signature: None,
                                        error_message: Some("Batch timeout".to_string()),
                                        retry_count: 0,
                                        completed_at: Instant::now(),
                                    }
                                }
                            }
                        }
                    }).collect();

                    // Wait for all batch executions to complete
                    let batch_results = futures::future::join_all(batch_futures).await;
                    
                    let batch_time = batch_start.elapsed();
                    let successful_in_batch = batch_results.iter()
                        .filter(|r| r.status == ExecutionStatus::Completed)
                        .count();

                    info!("📦 Batch {} completed in {:?} - Success: {}/{}", 
                        batch.batch_id, batch_time, successful_in_batch, batch_results.len());

                    // Update batch metrics
                    {
                        let mut metrics = performance_metrics.write().await;
                        metrics.total_batches_processed += 1;
                        metrics.average_batch_size = 
                            (metrics.average_batch_size * (metrics.total_batches_processed - 1) as f64 + 
                             batch_results.len() as f64) / metrics.total_batches_processed as f64;
                    }
                } else {
                    // No batches in queue, sleep briefly
                    sleep(Duration::from_millis(100)).await;
                }
            }
        });

        Ok(())
    }

    /// Execute a single opportunity with parallel execution optimizations
    async fn execute_opportunity_parallel(
        opportunity: EventDrivenOpportunity,
        mev_protection: &Arc<MEVProtectionEngine>,
        config: &ParallelExecutionConfig,
    ) -> ExecutionResult {
        let start_time = Instant::now();
        let mut retry_count = 0;

        loop {
            info!("⚡ Executing opportunity (parallel): {} - Attempt: {}", 
                opportunity.opportunity_id, retry_count + 1);

            // Check if opportunity has expired
            if opportunity.expires_at < Instant::now() {
                return ExecutionResult {
                    opportunity_id: opportunity.opportunity_id.clone(),
                    status: ExecutionStatus::Failed,
                    profit_lamports: 0,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    gas_used: 0,
                    transaction_signature: None,
                    error_message: Some("Opportunity expired".to_string()),
                    retry_count,
                    completed_at: Instant::now(),
                };
            }

            // Execute with timeout
            let execution_future = Self::execute_opportunity_with_timeout(
                &opportunity,
                mev_protection,
                config.execution_timeout_ms,
            );

            match execution_future.await {
                Ok(result) => {
                    return ExecutionResult {
                        opportunity_id: opportunity.opportunity_id.clone(),
                        status: ExecutionStatus::Completed,
                        profit_lamports: result,
                        execution_time_ms: start_time.elapsed().as_millis() as u64,
                        gas_used: opportunity.execution_data.estimated_gas,
                        transaction_signature: Some(format!("tx_{}", chrono::Utc::now().timestamp_millis())),
                        error_message: None,
                        retry_count,
                        completed_at: Instant::now(),
                    };
                }
                Err(e) => {
                    retry_count += 1;
                    
                    if retry_count >= config.retry_attempts {
                        return ExecutionResult {
                            opportunity_id: opportunity.opportunity_id.clone(),
                            status: ExecutionStatus::Failed,
                            profit_lamports: 0,
                            execution_time_ms: start_time.elapsed().as_millis() as u64,
                            gas_used: 0,
                            transaction_signature: None,
                            error_message: Some(e.to_string()),
                            retry_count,
                            completed_at: Instant::now(),
                        };
                    }

                    warn!("⚠️ Execution failed, retrying: {} - Attempt: {} - Error: {}", 
                        opportunity.opportunity_id, retry_count, e);

                    // Wait before retry
                    sleep(Duration::from_millis(config.retry_delay_ms)).await;
                }
            }
        }
    }

    /// Execute opportunity with timeout protection
    async fn execute_opportunity_with_timeout(
        opportunity: &EventDrivenOpportunity,
        _mev_protection: &Arc<MEVProtectionEngine>,
        timeout_ms: u64,
    ) -> Result<u64> {
        let execution_future = async {
            // Simulate execution based on opportunity type
            match &opportunity.opportunity_type {
                OpportunityType::JupiterAutoRouted(_) => {
                    sleep(Duration::from_millis(500)).await;
                    Ok(opportunity.expected_profit_lamports * 80 / 100) // 80% of expected
                }
                OpportunityType::DEXSpecialized(_) => {
                    sleep(Duration::from_millis(750)).await;
                    Ok(opportunity.expected_profit_lamports * 85 / 100) // 85% of expected
                }
                OpportunityType::CrossDEXArbitrage { .. } => {
                    sleep(Duration::from_millis(1000)).await;
                    Ok(opportunity.expected_profit_lamports * 90 / 100) // 90% of expected
                }
            }
        };

        tokio::time::timeout(Duration::from_millis(timeout_ms), execution_future)
            .await
            .map_err(|_| anyhow::anyhow!("Execution timeout"))
            .and_then(|result| result)
    }

    /// Acquire DEX-specific semaphore
    async fn acquire_dex_semaphore<'a>(
        dex_semaphores: &'a Arc<RwLock<HashMap<String, Arc<Semaphore>>>>,
        dex_name: &str,
        max_concurrent: usize,
    ) -> Result<Arc<Semaphore>> {
        // Get or create DEX semaphore
        let semaphore = {
            let mut semaphores = dex_semaphores.write().await;
            semaphores.entry(dex_name.to_string())
                .or_insert_with(|| Arc::new(Semaphore::new(max_concurrent)))
                .clone()
        };

        Ok(semaphore)
    }

    /// Acquire token-specific semaphore
    async fn acquire_token_semaphore<'a>(
        token_semaphores: &'a Arc<RwLock<HashMap<Pubkey, Arc<Semaphore>>>>,
        token_mint: &Pubkey,
        max_concurrent: usize,
    ) -> Result<Arc<Semaphore>> {
        // Get or create token semaphore
        let semaphore = {
            let mut semaphores = token_semaphores.write().await;
            semaphores.entry(*token_mint)
                .or_insert_with(|| Arc::new(Semaphore::new(max_concurrent)))
                .clone()
        };

        Ok(semaphore)
    }

    /// Extract DEX name from opportunity
    fn extract_dex_name(opportunity: &EventDrivenOpportunity) -> String {
        match &opportunity.opportunity_type {
            OpportunityType::JupiterAutoRouted(_) => "Jupiter".to_string(),
            OpportunityType::DEXSpecialized(spec_opp) => spec_opp.dex_name.clone(),
            OpportunityType::CrossDEXArbitrage { buy_dex, .. } => buy_dex.clone(),
        }
    }

    /// Extract token mint from opportunity
    fn extract_token_mint(opportunity: &EventDrivenOpportunity) -> Pubkey {
        // Use a default token for now - this should be parsed from input_token string
        match opportunity.input_token.parse() {
            Ok(pubkey) => pubkey,
            Err(_) => Pubkey::default(), // Fallback to default
        }
    }

    /// Update resource tracking when execution starts
    async fn update_resource_tracking_start(
        resource_tracker: &Arc<RwLock<ResourceTracker>>,
        opportunity: &EventDrivenOpportunity,
    ) {
        let mut tracker = resource_tracker.write().await;
        
        tracker.total_active_executions += 1;
        tracker.pending_executions = tracker.pending_executions.saturating_sub(1);

        let dex_name = Self::extract_dex_name(opportunity);
        *tracker.active_executions_by_dex.entry(dex_name).or_insert(0) += 1;

        let token_mint = Self::extract_token_mint(opportunity);
        *tracker.active_executions_by_token.entry(token_mint).or_insert(0) += 1;
    }

    /// Update resource tracking when execution completes
    async fn update_resource_tracking_complete(
        resource_tracker: &Arc<RwLock<ResourceTracker>>,
        execution_result: &ExecutionResult,
    ) {
        let mut tracker = resource_tracker.write().await;
        
        tracker.total_active_executions = tracker.total_active_executions.saturating_sub(1);
        
        match execution_result.status {
            ExecutionStatus::Completed => tracker.completed_executions += 1,
            _ => tracker.failed_executions += 1,
        }
    }

    /// Update performance metrics
    async fn update_performance_metrics(
        performance_metrics: &Arc<RwLock<ParallelExecutionMetrics>>,
        execution_result: &ExecutionResult,
    ) {
        let mut metrics = performance_metrics.write().await;
        
        metrics.total_opportunities_processed += 1;
        
        match execution_result.status {
            ExecutionStatus::Completed => {
                metrics.successful_executions += 1;
                metrics.total_profit_lamports += execution_result.profit_lamports;
            }
            ExecutionStatus::Failed => {
                metrics.failed_executions += 1;
            }
            ExecutionStatus::Timeout => {
                metrics.timeout_executions += 1;
            }
            _ => {}
        }

        // Update average execution time
        let total_executions = metrics.successful_executions + metrics.failed_executions + metrics.timeout_executions;
        if total_executions > 0 {
            metrics.average_execution_time_ms = 
                (metrics.average_execution_time_ms * (total_executions - 1) as f64 + 
                 execution_result.execution_time_ms as f64) / total_executions as f64;
        }
    }

    /// Start resource monitoring task
    async fn start_resource_monitor(&self) -> Result<()> {
        let resource_tracker = Arc::clone(&self.resource_tracker);
        let performance_metrics = Arc::clone(&self.performance_metrics);
        let config = self.config.clone();

        tokio::spawn(async move {
            info!("📊 Resource monitor started");

            loop {
                sleep(Duration::from_secs(5)).await;

                let tracker = resource_tracker.read().await;
                
                // Calculate concurrency efficiency
                let efficiency = if config.max_concurrent_executions > 0 {
                    tracker.total_active_executions as f64 / config.max_concurrent_executions as f64
                } else {
                    0.0
                };

                // Update peak concurrent executions
                {
                    let mut metrics = performance_metrics.write().await;
                    if tracker.total_active_executions > metrics.peak_concurrent_executions {
                        metrics.peak_concurrent_executions = tracker.total_active_executions;
                    }
                    metrics.concurrency_efficiency = efficiency;
                }

                debug!("📊 Resource usage - Active: {} | Pending: {} | Completed: {} | Failed: {} | Efficiency: {:.1}%", 
                    tracker.total_active_executions,
                    tracker.pending_executions,
                    tracker.completed_executions,
                    tracker.failed_executions,
                    efficiency * 100.0
                );
            }
        });

        Ok(())
    }

    /// Start metrics reporting task
    async fn start_metrics_reporter(&self) -> Result<()> {
        let performance_metrics = Arc::clone(&self.performance_metrics);
        let resource_tracker = Arc::clone(&self.resource_tracker);

        tokio::spawn(async move {
            info!("📈 Metrics reporter started");

            loop {
                sleep(Duration::from_secs(60)).await; // Report every minute

                let metrics = performance_metrics.read().await;
                let tracker = resource_tracker.read().await;

                info!("📈 PARALLEL EXECUTION METRICS");
                info!("   Processed: {} | Success: {} | Failed: {} | Timeout: {}", 
                    metrics.total_opportunities_processed,
                    metrics.successful_executions,
                    metrics.failed_executions,
                    metrics.timeout_executions
                );
                
                if metrics.successful_executions > 0 {
                    info!("   Success Rate: {:.1}% | Avg Time: {:.1}ms | Total Profit: {} lamports",
                        (metrics.successful_executions as f64 / metrics.total_opportunities_processed as f64) * 100.0,
                        metrics.average_execution_time_ms,
                        metrics.total_profit_lamports
                    );
                }

                info!("   Peak Concurrency: {} | Current Efficiency: {:.1}% | Batches: {}",
                    metrics.peak_concurrent_executions,
                    metrics.concurrency_efficiency * 100.0,
                    metrics.total_batches_processed
                );

                info!("   Active: {} | Pending: {} | Completed: {} | Failed: {}",
                    tracker.total_active_executions,
                    tracker.pending_executions,
                    tracker.completed_executions,
                    tracker.failed_executions
                );
            }
        });

        Ok(())
    }
}
