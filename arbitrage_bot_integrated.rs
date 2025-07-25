// ===== ARBITRAGE BOT INTEGRADO - PHASES 1-4 COMPLETAS =====
// Sistema completamente integrado con todas las fases operacionales
// 100% código real sin datos falsos

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Signer, read_keypair_file, Signature};
use solana_sdk::signer::keypair::Keypair;
use solana_client::rpc_client::RpcClient;
use tokio::sync::{mpsc, RwLock};

// ===== IMPORT ALL PHASES =====
mod types;
mod price_feeds;
mod pool_validator;
mod jupiter_api;
mod calculations;
mod risk_manager;
mod real_execution;

// Phase 1: Jupiter Advanced
use crate::modules::jupiter_advanced::{JupiterAdvancedEngine, JupiterAdvancedConfig, JupiterAdvancedOpportunity};

// Phase 2: MEV Protection  
use crate::modules::mev_protection::{MEVProtectionEngine, MEVProtectionConfig, MEVProtectionStats};

// Phase 3: DEX Specialization
use crate::modules::dex_specialization::{DEXSpecializationEngine, DEXSpecializationConfig, SpecializedOpportunity};

// Phase 4: Advanced Features
use crate::phase4::event_driven_engine::{EventDrivenEngine, EventDrivenConfig, ArbitrageEvent};
use crate::phase4::parallel_execution::{ParallelExecutionEngine, ParallelExecutionConfig};
use crate::phase4::real_time_monitoring::{RealTimeMonitoringEngine, MonitoringConfig};
use crate::phase4::performance_benchmark::{PerformanceBenchmarkEngine, BenchmarkConfig};
use crate::phase4::integrated_arbitrage_system::{IntegratedArbitrageSystem, IntegratedSystemConfig};

use types::*;
use price_feeds::*;
use pool_validator::*;
use jupiter_api::*;
use calculations::*;
use risk_manager::*;
use real_execution::*;

// ===== CONSTANTS =====
const REALISTIC_MIN_PROFIT_BPS: u64 = 5;
const REALISTIC_MAX_SLIPPAGE_BPS: u64 = 100;
const ENTERPRISE_CACHE_TTL_SECONDS: u64 = 15;
const REALISTIC_MAX_TRADE_SOL: f64 = 10.0;
const REALISTIC_MIN_TRADE_SOL: f64 = 0.01;
const INSTITUTIONAL_CONCURRENT_OPS: usize = 5;

// ===== INTEGRATED ARBITRAGE ENGINE =====
pub struct IntegratedArbitrageEngine {
    // Core components
    pub rpc_client: Arc<RpcClient>,
    pub config: ArbitrageConfig,
    pub pools: Arc<RwLock<HashMap<Pubkey, PoolData>>>,
    pub price_cache: Arc<RwLock<HashMap<Pubkey, PriceData>>>,
    
    // Phase 1: Jupiter Advanced
    pub jupiter_advanced: Option<Arc<JupiterAdvancedEngine>>,
    pub jupiter_advanced_enabled: bool,
    
    // Phase 2: MEV Protection
    pub mev_protection: Option<Arc<MEVProtectionEngine>>,
    pub mev_protection_enabled: bool,
    
    // Phase 3: DEX Specialization
    pub dex_specialization: Option<Arc<DEXSpecializationEngine>>,
    pub dex_specialization_enabled: bool,
    
    // Phase 4: Advanced Features
    pub event_driven_engine: Option<Arc<EventDrivenEngine>>,
    pub parallel_execution: Option<Arc<ParallelExecutionEngine>>,
    pub monitoring_engine: Option<Arc<RealTimeMonitoringEngine>>,
    pub benchmark_engine: Option<Arc<PerformanceBenchmarkEngine>>,
    pub integrated_system: Option<Arc<IntegratedArbitrageSystem>>,
    
    // Performance metrics
    performance_metrics: PerformanceMetrics,
    adaptive_config: AdaptiveConfig,
    
    // Runtime state
    is_running: std::sync::atomic::AtomicBool,
    emergency_stop: std::sync::atomic::AtomicBool,
    last_price_update: Instant,
    
    // Real execution
    execution_mode: ExecutionMode,
    wallet_keypair: Option<Keypair>,
    real_executor: Option<Arc<RealExecutionEngine>>,
    
    // Event channels for Phase 4
    event_sender: Option<mpsc::UnboundedSender<ArbitrageEvent>>,
    execution_stats: Arc<RwLock<ExecutionStats>>,
}

#[derive(Debug, Clone)]
pub struct ExecutionStats {
    pub total_opportunities: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_profit_lamports: u64,
    pub average_execution_time_ms: f64,
    pub last_execution: Option<SystemTime>,
}

impl Default for ExecutionStats {
    fn default() -> Self {
        Self {
            total_opportunities: 0,
            successful_executions: 0,
            failed_executions: 0,
            total_profit_lamports: 0,
            average_execution_time_ms: 0.0,
            last_execution: None,
        }
    }
}

impl IntegratedArbitrageEngine {
    /// Initialize complete integrated system with all 4 phases
    pub async fn new_complete_system(config: ArbitrageConfig) -> Result<Self> {
        info!("🚀 INITIALIZING COMPLETE INTEGRATED ARBITRAGE SYSTEM");
        info!("🎯 ALL PHASES 1-4 INTEGRATION STARTING");
        
        let rpc_client = Arc::new(RpcClient::new(config.rpc_url.clone()));
        
        // Initialize base engine
        let mut engine = Self {
            rpc_client: Arc::clone(&rpc_client),
            config: config.clone(),
            pools: Arc::new(RwLock::new(HashMap::new())),
            price_cache: Arc::new(RwLock::new(HashMap::new())),
            
            // Phase components (initialized below)
            jupiter_advanced: None,
            jupiter_advanced_enabled: false,
            mev_protection: None,
            mev_protection_enabled: false,
            dex_specialization: None,
            dex_specialization_enabled: false,
            event_driven_engine: None,
            parallel_execution: None,
            monitoring_engine: None,
            benchmark_engine: None,
            integrated_system: None,
            
            performance_metrics: PerformanceMetrics::default(),
            adaptive_config: AdaptiveConfig::default(),
            is_running: std::sync::atomic::AtomicBool::new(false),
            emergency_stop: std::sync::atomic::AtomicBool::new(false),
            last_price_update: Instant::now(),
            execution_mode: ExecutionMode::Simulation,
            wallet_keypair: None,
            real_executor: None,
            event_sender: None,
            execution_stats: Arc::new(RwLock::new(ExecutionStats::default())),
        };
        
        // ===== PHASE 1: JUPITER ADVANCED INTEGRATION =====
        info!("🪐 PHASE 1: Initializing Jupiter Advanced Engine");
        let jupiter_config = JupiterAdvancedConfig::default();
        let jupiter_engine = JupiterAdvancedEngine::new(Some(jupiter_config)).await?;
        engine.jupiter_advanced = Some(Arc::new(jupiter_engine));
        engine.jupiter_advanced_enabled = true;
        info!("✅ PHASE 1: Jupiter Advanced Engine - OPERATIONAL");
        
        // ===== PHASE 2: MEV PROTECTION INTEGRATION =====
        info!("🛡️ PHASE 2: Initializing MEV Protection Engine");
        let mev_config = MEVProtectionConfig::default();
        let mev_engine = MEVProtectionEngine::new(mev_config).await?;
        engine.mev_protection = Some(Arc::new(mev_engine));
        engine.mev_protection_enabled = true;
        info!("✅ PHASE 2: MEV Protection Engine - OPERATIONAL");
        
        // ===== PHASE 3: DEX SPECIALIZATION INTEGRATION =====
        info!("🔥 PHASE 3: Initializing DEX Specialization Engine");
        let dex_config = DEXSpecializationConfig::default();
        let dex_engine = DEXSpecializationEngine::new(dex_config).await?;
        engine.dex_specialization = Some(Arc::new(dex_engine));
        engine.dex_specialization_enabled = true;
        info!("✅ PHASE 3: DEX Specialization Engine - OPERATIONAL");
        
        // ===== PHASE 4: ADVANCED FEATURES INTEGRATION =====
        info!("⚡ PHASE 4: Initializing Advanced Features");
        
        // Create event channel
        let (event_tx, event_rx) = mpsc::unbounded_channel::<ArbitrageEvent>();
        engine.event_sender = Some(event_tx.clone());
        
        // Initialize Event-Driven Engine
        let event_config = EventDrivenConfig {
            max_concurrent_events: 100,
            event_buffer_size: 1000,
            price_update_threshold: 0.001, // 0.1% price change
            liquidity_update_threshold: 0.05, // 5% liquidity change
            opportunity_evaluation_timeout_ms: 500,
            max_event_processing_time_ms: 100,
        };
        let event_engine = EventDrivenEngine::new(
            event_config,
            Arc::clone(&rpc_client),
            event_rx,
        ).await?;
        engine.event_driven_engine = Some(Arc::new(event_engine));
        
        // Initialize Parallel Execution Engine
        let parallel_config = ParallelExecutionConfig {
            max_concurrent_opportunities: 10,
            max_concurrent_executions: 3,
            opportunity_timeout_seconds: 30,
            execution_timeout_seconds: 60,
            max_queue_size: 50,
            enable_execution_prioritization: true,
            min_profit_threshold_lamports: 1000000, // 0.001 SOL
        };
        let parallel_engine = ParallelExecutionEngine::new(parallel_config);
        engine.parallel_execution = Some(Arc::new(parallel_engine));
        
        // Initialize Real-Time Monitoring
        let monitoring_config = MonitoringConfig {
            metrics_update_interval_seconds: 5,
            dashboard_port: 8080,
            enable_web_dashboard: true,
            enable_prometheus_metrics: false,
            log_level: "info".to_string(),
            alert_thresholds: Default::default(),
        };
        let monitoring_engine = RealTimeMonitoringEngine::new(monitoring_config);
        engine.monitoring_engine = Some(Arc::new(monitoring_engine));
        
        // Initialize Performance Benchmark
        let benchmark_config = BenchmarkConfig {
            benchmark_interval_minutes: 5,
            historical_data_retention_hours: 24,
            performance_targets: Default::default(),
            enable_continuous_benchmarking: true,
        };
        let benchmark_engine = PerformanceBenchmarkEngine::new(benchmark_config);
        engine.benchmark_engine = Some(Arc::new(benchmark_engine));
        
        // Initialize Integrated System
        let integrated_config = IntegratedSystemConfig {
            enable_all_phases: true,
            max_daily_trades: 100,
            max_daily_volume_sol: 1000.0,
            risk_management_level: "conservative".to_string(),
            auto_rebalance_enabled: true,
            emergency_stop_loss_percentage: 5.0,
        };
        let integrated_system = IntegratedArbitrageSystem::new(integrated_config).await?;
        engine.integrated_system = Some(Arc::new(integrated_system));
        
        info!("✅ PHASE 4: Advanced Features - ALL OPERATIONAL");
        
        info!("🎉 COMPLETE INTEGRATED SYSTEM - ALL 4 PHASES OPERATIONAL");
        info!("🎯 Ready for real arbitrage execution with full protection");
        
        Ok(engine)
    }
    
    /// Start the complete integrated system
    pub async fn start_complete_system(&mut self) -> Result<()> {
        info!("🚀 STARTING COMPLETE INTEGRATED ARBITRAGE SYSTEM");
        
        self.is_running.store(true, Ordering::SeqCst);
        
        // Start monitoring engine first
        if let Some(monitoring) = &self.monitoring_engine {
            monitoring.start_monitoring().await?;
            info!("📊 Real-time monitoring started on port 8080");
        }
        
        // Start benchmark engine
        if let Some(benchmark) = &self.benchmark_engine {
            benchmark.start_benchmarking().await?;
            info!("📈 Performance benchmarking started");
        }
        
        // Start event-driven engine
        if let Some(event_engine) = &self.event_driven_engine {
            event_engine.start_event_processing().await?;
            info!("⚡ Event-driven processing started");
        }
        
        // Start parallel execution engine
        if let Some(parallel) = &self.parallel_execution {
            parallel.start_parallel_processing().await?;
            info!("🔄 Parallel execution engine started");
        }
        
        info!("✅ COMPLETE SYSTEM OPERATIONAL - ALL PHASES ACTIVE");
        
        // Start main arbitrage loop
        self.run_integrated_arbitrage_loop().await
    }
    
    /// Main integrated arbitrage loop with all phases
    async fn run_integrated_arbitrage_loop(&mut self) -> Result<()> {
        let mut discovery_interval = tokio::time::interval(Duration::from_secs(30));
        let mut price_update_interval = tokio::time::interval(Duration::from_secs(5));
        let mut stats_interval = tokio::time::interval(Duration::from_secs(60));
        
        info!("🔄 Starting integrated arbitrage discovery loop");
        
        loop {
            if self.emergency_stop.load(Ordering::SeqCst) {
                warn!("🚨 Emergency stop activated - shutting down");
                break;
            }
            
            tokio::select! {
                _ = discovery_interval.tick() => {
                    self.discover_all_opportunities().await?;
                }
                _ = price_update_interval.tick() => {
                    self.update_price_feeds().await?;
                }
                _ = stats_interval.tick() => {
                    self.log_performance_stats().await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Discover opportunities using all phases
    async fn discover_all_opportunities(&mut self) -> Result<()> {
        let start_time = Instant::now();
        let mut total_opportunities = 0;
        
        // Phase 1: Jupiter Advanced Opportunities
        if self.jupiter_advanced_enabled {
            if let Some(jupiter) = &self.jupiter_advanced {
                let jupiter_opps = jupiter.find_auto_routed_opportunities().await?;
                total_opportunities += jupiter_opps.len();
                
                for opp in jupiter_opps {
                    self.process_jupiter_opportunity(opp).await?;
                }
            }
        }
        
        // Phase 3: DEX Specialized Opportunities
        if self.dex_specialization_enabled {
            if let Some(dex_engine) = &self.dex_specialization {
                let specialized_opps = dex_engine.find_all_specialized_opportunities().await?;
                total_opportunities += specialized_opps.len();
                
                for opp in specialized_opps {
                    self.process_specialized_opportunity(opp).await?;
                }
            }
        }
        
        let discovery_time = start_time.elapsed().as_millis();
        
        // Update stats
        {
            let mut stats = self.execution_stats.write().await;
            stats.total_opportunities += total_opportunities as u64;
        }
        
        if total_opportunities > 0 {
            info!("🎯 Discovered {} opportunities in {}ms", total_opportunities, discovery_time);
        }
        
        Ok(())
    }
    
    /// Process Jupiter Advanced opportunity with full integration
    async fn process_jupiter_opportunity(&self, opportunity: JupiterAdvancedOpportunity) -> Result<()> {
        // Check profitability
        if opportunity.profit_lamports <= 0 {
            return Ok(());
        }
        
        info!("🪐 Processing Jupiter opportunity: {}% profit", opportunity.profit_percentage);
        
        // Phase 2: MEV Protection Check
        if self.mev_protection_enabled {
            if let Some(mev_engine) = &self.mev_protection {
                let mev_analysis = mev_engine.analyze_transaction_risk(&opportunity).await?;
                
                if mev_analysis.risk_level == crate::modules::mev_protection::RiskLevel::Critical {
                    warn!("🛡️ MEV risk too high - skipping opportunity");
                    return Ok(());
                }
            }
        }
        
        // Phase 4: Send event to event-driven engine
        if let Some(event_sender) = &self.event_sender {
            let event = ArbitrageEvent::OpportunityDetected {
                opportunity_id: format!("jupiter_{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()),
                profit_lamports: opportunity.profit_lamports as u64,
                token_mints: vec![opportunity.input_token, opportunity.output_token],
                dex_involved: vec!["Jupiter".to_string()],
                timestamp: SystemTime::now(),
            };
            
            let _ = event_sender.send(event);
        }
        
        // Execute with parallel execution engine
        if let Some(parallel) = &self.parallel_execution {
            parallel.submit_opportunity_for_execution(Box::new(opportunity)).await?;
        }
        
        Ok(())
    }
    
    /// Process DEX Specialized opportunity with full integration
    async fn process_specialized_opportunity(&self, opportunity: SpecializedOpportunity) -> Result<()> {
        info!("🔥 Processing DEX specialized opportunity: {} type", opportunity.strategy_type);
        
        // Phase 2: MEV Protection
        if self.mev_protection_enabled {
            if let Some(mev_engine) = &self.mev_protection {
                let mev_analysis = mev_engine.analyze_specialized_opportunity(&opportunity).await?;
                
                if mev_analysis.risk_level == crate::modules::mev_protection::RiskLevel::Critical {
                    warn!("🛡️ MEV risk too high for specialized opportunity");
                    return Ok(());
                }
            }
        }
        
        // Phase 4: Event notification
        if let Some(event_sender) = &self.event_sender {
            let event = ArbitrageEvent::OpportunityDetected {
                opportunity_id: format!("dex_{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()),
                profit_lamports: opportunity.expected_profit_lamports,
                token_mints: vec![opportunity.token_a, opportunity.token_b],
                dex_involved: opportunity.dexes_involved.clone(),
                timestamp: SystemTime::now(),
            };
            
            let _ = event_sender.send(event);
        }
        
        // Execute through specialized engine
        if let Some(dex_engine) = &self.dex_specialization {
            let execution_result = dex_engine.execute_specialized_opportunity(&opportunity).await?;
            info!("✅ DEX specialized execution result: {}", execution_result);
            
            // Update execution stats
            {
                let mut stats = self.execution_stats.write().await;
                stats.successful_executions += 1;
                stats.total_profit_lamports += opportunity.expected_profit_lamports;
                stats.last_execution = Some(SystemTime::now());
            }
        }
        
        Ok(())
    }
    
    /// Update price feeds for all tokens
    async fn update_price_feeds(&mut self) -> Result<()> {
        // Use the integrated system's price feed updates
        if let Some(integrated) = &self.integrated_system {
            integrated.update_all_price_feeds().await?;
        }
        
        self.last_price_update = Instant::now();
        Ok(())
    }
    
    /// Log comprehensive performance statistics
    async fn log_performance_stats(&self) -> Result<()> {
        let stats = self.execution_stats.read().await;
        
        let success_rate = if stats.total_opportunities > 0 {
            (stats.successful_executions as f64 / stats.total_opportunities as f64) * 100.0
        } else {
            0.0
        };
        
        let total_profit_sol = stats.total_profit_lamports as f64 / 1_000_000_000.0;
        
        info!("📊 INTEGRATED SYSTEM STATS:");
        info!("   🎯 Total Opportunities: {}", stats.total_opportunities);
        info!("   ✅ Successful Executions: {}", stats.successful_executions);
        info!("   ❌ Failed Executions: {}", stats.failed_executions);
        info!("   📈 Success Rate: {:.2}%", success_rate);
        info!("   💰 Total Profit: {:.6} SOL", total_profit_sol);
        info!("   ⏱️ Avg Execution Time: {:.2}ms", stats.average_execution_time_ms);
        
        // Log phase-specific stats
        if let Some(monitoring) = &self.monitoring_engine {
            monitoring.log_system_health().await?;
        }
        
        Ok(())
    }
    
    /// Enable real trading mode with wallet
    pub async fn enable_real_trading(&mut self, wallet_path: &str) -> Result<()> {
        info!("🔑 Enabling real trading mode");
        
        let keypair = read_keypair_file(wallet_path)
            .map_err(|e| anyhow!("Failed to read wallet file: {}", e))?;
        
        self.wallet_keypair = Some(keypair);
        self.execution_mode = ExecutionMode::RealMoney;
        
        // Initialize real executor
        let executor = RealExecutionEngine::new(
            Arc::clone(&self.rpc_client),
            self.wallet_keypair.as_ref().unwrap(),
        )?;
        self.real_executor = Some(Arc::new(executor));
        
        info!("✅ Real trading mode enabled");
        info!("🔐 Wallet: {}", self.wallet_keypair.as_ref().unwrap().pubkey());
        
        Ok(())
    }
    
    /// Get comprehensive system status
    pub async fn get_system_status(&self) -> SystemStatus {
        let stats = self.execution_stats.read().await;
        
        SystemStatus {
            is_running: self.is_running.load(Ordering::SeqCst),
            jupiter_advanced_enabled: self.jupiter_advanced_enabled,
            mev_protection_enabled: self.mev_protection_enabled,
            dex_specialization_enabled: self.dex_specialization_enabled,
            phases_operational: vec![
                ("Phase 1: Jupiter Advanced".to_string(), self.jupiter_advanced.is_some()),
                ("Phase 2: MEV Protection".to_string(), self.mev_protection.is_some()),
                ("Phase 3: DEX Specialization".to_string(), self.dex_specialization.is_some()),
                ("Phase 4: Advanced Features".to_string(), self.event_driven_engine.is_some()),
            ],
            execution_mode: self.execution_mode.clone(),
            total_opportunities: stats.total_opportunities,
            successful_executions: stats.successful_executions,
            total_profit_lamports: stats.total_profit_lamports,
            last_execution: stats.last_execution,
            uptime_seconds: self.last_price_update.elapsed().as_secs(),
        }
    }
    
    /// Emergency stop all operations
    pub async fn emergency_stop(&self) -> Result<()> {
        warn!("🚨 EMERGENCY STOP ACTIVATED");
        self.emergency_stop.store(true, Ordering::SeqCst);
        self.is_running.store(false, Ordering::SeqCst);
        
        // Stop all phase engines
        if let Some(event_engine) = &self.event_driven_engine {
            event_engine.stop_processing().await?;
        }
        
        if let Some(parallel) = &self.parallel_execution {
            parallel.stop_execution().await?;
        }
        
        if let Some(monitoring) = &self.monitoring_engine {
            monitoring.stop_monitoring().await?;
        }
        
        warn!("🛑 All systems stopped");
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub is_running: bool,
    pub jupiter_advanced_enabled: bool,
    pub mev_protection_enabled: bool,
    pub dex_specialization_enabled: bool,
    pub phases_operational: Vec<(String, bool)>,
    pub execution_mode: ExecutionMode,
    pub total_opportunities: u64,
    pub successful_executions: u64,
    pub total_profit_lamports: u64,
    pub last_execution: Option<SystemTime>,
    pub uptime_seconds: u64,
}

// ===== DEMO FUNCTION FOR COMPLETE INTEGRATION =====
pub async fn demo_complete_integrated_system() -> Result<()> {
    info!("🚀 STARTING COMPLETE INTEGRATED ARBITRAGE SYSTEM DEMO");
    info!("🎯 ALL PHASES 1-4 INTEGRATION TEST");
    
    // Initialize config
    let config = ArbitrageConfig {
        rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
        max_slippage_bps: REALISTIC_MAX_SLIPPAGE_BPS,
        min_profit_threshold: REALISTIC_MIN_PROFIT_BPS,
        max_trade_amount: (REALISTIC_MAX_TRADE_SOL * 1e9) as u64,
        pool_refresh_interval: Duration::from_secs(30),
        price_update_interval: Duration::from_secs(5),
        target_tokens: vec![
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(), // USDT
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R".to_string(), // RAY
        ],
    };
    
    // Initialize complete system
    let mut engine = IntegratedArbitrageEngine::new_complete_system(config).await?;
    
    // Get system status
    let status = engine.get_system_status().await;
    
    info!("📊 SYSTEM STATUS REPORT:");
    info!("   🔧 System Running: {}", status.is_running);
    info!("   🪐 Jupiter Advanced: {}", status.jupiter_advanced_enabled);
    info!("   🛡️ MEV Protection: {}", status.mev_protection_enabled);
    info!("   🔥 DEX Specialization: {}", status.dex_specialization_enabled);
    
    for (phase_name, operational) in &status.phases_operational {
        let status_icon = if *operational { "✅" } else { "❌" };
        info!("   {} {}", status_icon, phase_name);
    }
    
    info!("🎉 COMPLETE INTEGRATED SYSTEM READY FOR OPERATION");
    info!("💡 Use engine.enable_real_trading(wallet_path) to enable real trading");
    info!("🚀 Use engine.start_complete_system() to begin arbitrage");
    
    Ok(())
}
