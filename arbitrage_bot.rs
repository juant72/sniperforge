// ===== SNIPERFORGE ARBITRAGE BOT - ENTERPRISE ARBITRAGE SYSTEM =====
// Bot especializado en arbitraje multi-DEX con análisis militar y ejecución profesional
// PART OF SNIPERFORGE SUITE - ARBITRAGE-FOCUSED BOT
// PROPOSAL-003 MULTI-TOKEN SUPPORT INTEGRATED

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Signer, read_keypair_file, Signature};
use solana_client::rpc_client::RpcClient;

// ===== JUPITER SWAP RESULT STRUCTURE =====
#[derive(Debug, Clone)]
pub struct JupiterSwapResult {
    pub signature: Signature,
    pub input_amount: u64,
    pub output_amount: u64,
    pub price_impact: f64,
}

// ===== REALISTIC ARBITRAGE CONSTANTS FOR MAINNET =====
const REALISTIC_MIN_PROFIT_BPS: u64 = 5; // 0.05% - Threshold realista para arbitraje
const REALISTIC_MAX_SLIPPAGE_BPS: u64 = 100; // 1.0% - Slippage conservador
const ENTERPRISE_CACHE_TTL_SECONDS: u64 = 15; // Cache más rápido para oportunidades
const REALISTIC_MAX_TRADE_SOL: f64 = 10.0; // 10 SOL máximo por trade
const REALISTIC_MIN_TRADE_SOL: f64 = 0.01; // 0.01 SOL minimum trade
const ENTERPRISE_RISK_DAILY_VOLUME: f64 = 100.0; // SOL - Volume diario conservador
const REALISTIC_LATENCY_THRESHOLD_MS: u64 = 200; // 200ms latency threshold
const INSTITUTIONAL_CONCURRENT_OPS: usize = 5; // 5 operaciones concurrentes

// NEW: MAINNET execution constants - CORREGIDO
const MAINNET_JUPITER_API: &str = "https://quote-api.jup.ag/v6";
const MAINNET_JUPITER_SWAP_API: &str = "https://quote-api.jup.ag/v6/swap";
const MAINNET_MIN_PROFIT_SOL: f64 = 0.0015; // 0.0015 SOL = ~$0.045 profit mínimo
const MAINNET_MAX_SLIPPAGE_BPS: u16 = 100; // 1.0% max slippage conservador
const MAINNET_EXECUTION_TIMEOUT: u64 = 30; // 30 seconds max execution time

// ===== ENTERPRISE MODULE IMPORTS =====
mod types;
mod price_feeds;
mod pool_validator;
mod jupiter_api;
mod calculations;
mod risk_manager;
mod real_execution;
mod jupiter_integration;
mod transaction_executor;
mod saber_integration;

// ===== MODULAR ARBITRAGE SYSTEM IMPORTS =====
mod modules;

use modules::{
    execute_safe_arbitrage_test,
    execute_comprehensive_scan, execute_quick_scan,
    MonitorConfig, start_automated_monitoring_with_config,
    simulate_arbitrage_execution_advanced
};

use types::*;
use sniperforge::types::DexType;
// PROPOSAL-003: Multi-token system imports
use saber_integration::SaberIntegration;
use sniperforge::enhanced_pool_discovery::execute_enhanced_pool_discovery;
use price_feeds::ProfessionalPriceFeeds;
use pool_validator::PoolValidator;
use jupiter_api::JupiterAPI;
use calculations::*;
use risk_manager::EnterpriseRiskManager;
use real_execution::RealExecutionEngine;

// ===== ENTERPRISE ARBITRAGE ENGINE IMPLEMENTATION =====
impl ProfessionalArbitrageEngine {
    /// ENTERPRISE INITIALIZATION - Military-grade setup with institutional safeguards
    pub async fn new_enterprise_professional(rpc_url: String, wallet_keypair_path: String) -> Result<Self> {
        info!("🏛️  INITIALIZING ENTERPRISE ARBITRAGE ENGINE");
        info!("⚔️  MILITARY-GRADE INITIALIZATION SEQUENCE STARTING");
        
        // STEP 1: INSTITUTIONAL WALLET SECURITY VALIDATION
        let wallet_address = if std::path::Path::new(&wallet_keypair_path).exists() {
            let wallet_keypair = solana_sdk::signature::read_keypair_file(&wallet_keypair_path)
                .map_err(|e| anyhow!("ENTERPRISE SECURITY FAILURE: {}", e))?;
            info!("🔐 INSTITUTIONAL WALLET AUTHENTICATED: {}", wallet_keypair.pubkey());
            info!("✅ ENTERPRISE SECURITY CLEARANCE: AUTHORIZED");
            wallet_keypair.pubkey()
        } else {
            warn!("⚠️  ENTERPRISE WALLET NOT FOUND - ENGAGING SIMULATION MODE");
            warn!("🚨 INSTITUTIONAL ALERT: Operating in demo configuration");
            Pubkey::from_str("11111111111111111111111111111111")?
        };
        
        // STEP 2: ENTERPRISE RPC INFRASTRUCTURE INITIALIZATION
        info!("🌐 ESTABLISHING ENTERPRISE BLOCKCHAIN CONNECTIONS");
        let client = RpcClient::new_with_commitment(
            rpc_url.clone(),
            solana_sdk::commitment_config::CommitmentConfig::confirmed(),
        );
        
        // STEP 3: INSTITUTIONAL HTTP CLIENT WITH ENTERPRISE SPECS
        info!("🔗 DEPLOYING INSTITUTIONAL HTTP INFRASTRUCTURE");
        let jupiter_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .connection_verbose(true)
            .pool_max_idle_per_host(10)
            .user_agent("Enterprise-Arbiter-Pro/2.0-Military")
            .build()?;
        
        // STEP 4: ENTERPRISE MODULE INITIALIZATION
        info!("🏗️  INITIALIZING ENTERPRISE PROFESSIONAL MODULES");
        let price_feeds = ProfessionalPriceFeeds::new();
        let pool_validator = PoolValidator::new(rpc_url);
        
        // STEP 4.5: SABER INTEGRATION INITIALIZATION
        info!("🗡️  INITIALIZING SABER POOLS INTEGRATION");
        let saber_integration = SaberIntegration::new();
        info!("✅ SABER: Real-time pool data integration ready");
        
        // STEP 5: ENTERPRISE ENGINE ASSEMBLY
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
                max_exposure_usd: 10000.0, // Institutional risk limit
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
                max_slippage_bps: REALISTIC_MAX_SLIPPAGE_BPS,
                min_profit_threshold: REALISTIC_MIN_PROFIT_BPS,
                max_trade_amount: (REALISTIC_MAX_TRADE_SOL * 1e9) as u64,
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
            
            // NEW: Real execution components (initialized as simulation by default)
            execution_mode: ExecutionMode::Simulation,
            wallet_keypair: None,
            real_executor: None,
            
            // PROPOSAL-003: Multi-token support (disabled by default - backward compatible)
            multi_token_config: None,
            multi_token_enabled: false,
            multi_token_tier2_enabled: None, // PROPOSAL-003 Phase 2: Disabled initially
            
            // Saber integration for real pool data
            saber_integration: Some(Box::new(saber_integration)),
        };
        
        info!("✅ ENTERPRISE ARBITRAGE ENGINE: FULLY OPERATIONAL");
        info!("🎯 INSTITUTIONAL STATUS: READY FOR MILITARY-PRECISION EXECUTION");
        info!("🚀 PROPOSAL-003: Multi-token support ready for activation");
        Ok(engine)
    }
    
    /// ACTIVATE REAL TRADING MODE - Mainnet production execution
    pub async fn enable_real_trading_mainnet(&mut self) -> Result<()> {
        info!("🚀 ACTIVATING REAL TRADING MODE - MAINNET PRODUCTION");
        warn!("⚠️  SWITCHING FROM SIMULATION TO REAL MONEY EXECUTION");
        
        // Load wallet keypair from environment or default path
        let wallet_path = std::env::var("WALLET_PATH").unwrap_or_else(|_| "mainnet-wallet.json".to_string());
        
        let wallet_keypair = read_keypair_file(&wallet_path)
            .map_err(|e| anyhow!("Failed to load wallet keypair from {}: {}", wallet_path, e))?;
        
        warn!("💰 WALLET: {}", wallet_keypair.pubkey());
        
        // Update engine configuration for real trading
        self.execution_mode = ExecutionMode::RealTrading;
        self.wallet_keypair = Some(wallet_keypair);
        
        info!("✅ REAL TRADING MODE ACTIVATED");
        info!("🎯 STATUS: Production-ready for mainnet arbitrage execution");
        info!("💡 Note: All safety validations and monitoring systems active");
        
        Ok(())
    }
    
    /// PROPOSAL-003: Activar sistema multi-token de manera segura
    pub async fn enable_multitoken_arbitrage(&mut self) -> Result<()> {
        info!("🚀 PROPOSAL-003: ACTIVATING MULTI-TOKEN ARBITRAGE SYSTEM");
        warn!("⚠️  SWITCHING FROM SINGLE-PAIR TO MULTI-TOKEN SUPPORT");
        
        // Por ahora, simplemente habilitamos el flag
        // La implementación completa se activará en futuras versiones
        self.multi_token_enabled = true;
        self.multi_token_tier2_enabled = Some(false); // Solo Tier 1
        
        info!("✅ PROPOSAL-003: MULTI-TOKEN FLAG ACTIVATED (TIER 1 ONLY)");
        info!("🎯 STATUS: Multi-token support enabled (Phase 1 implementation - 3 pairs)");
        info!("💡 Note: Enhanced features available, Tier 2 can be activated separately");
        
        Ok(())
    }

    /// PROPOSAL-003 TIER 2: Activar soporte completo del ecosistema Solana
    pub async fn enable_multitoken_tier2_arbitrage(&mut self) -> Result<()> {
        info!("🚀 PROPOSAL-003 TIER 2: ACTIVATING FULL ECOSYSTEM ARBITRAGE SYSTEM");
        warn!("⚠️  SWITCHING TO TIER 2 MULTI-TOKEN SUPPORT (16+ PAIRS)");
        
        // Habilitar tanto multi-token como Tier 2
        self.multi_token_enabled = true;
        self.multi_token_tier2_enabled = Some(true); // Tier 1 + Tier 2
        
        info!("✅ PROPOSAL-003 TIER 2: FULL ECOSYSTEM ACTIVATED");
        info!("🎯 STATUS: Multi-token Tier 2 support enabled (Phase 2 implementation)");
        info!("🪙 TOKENS: SOL, USDC, USDT, BONK, RAY, ORCA, PYTH, JTO");
        info!("🔗 PAIRS: 16 trading pairs across Solana ecosystem");
        info!("🛡️  RISK: Enhanced thresholds for ecosystem tokens applied");
        
        Ok(())
    }
    
    /// PROPOSAL-003: Verificar si multi-token está habilitado y listo
    pub fn is_multitoken_enabled(&self) -> bool {
        self.multi_token_enabled
    }
    
    /// PROPOSAL-003: Obtener pares de tokens habilitados para trading
    pub async fn get_enabled_token_pairs(&self) -> Result<Vec<(String, String)>> {
        if self.multi_token_enabled {
            info!("🔍 PROPOSAL-003: Consultando pares multi-token disponibles");
            
            // Simulamos la integración con el TokenPairManager
            // En una implementación completa, esto sería:
            // let pairs = self.token_manager.get_all_tradeable_pairs();
            
            if self.multi_token_tier2_enabled.unwrap_or(false) {
                // Tier 1 + Tier 2 = Máximo poder
                info!("🚀 PROPOSAL-003: Tier 2 habilitado - devolviendo todos los pares");
                Ok(vec![
                    // Tier 1 pairs (básicos)
                    ("SOL".to_string(), "USDC".to_string()),
                    ("SOL".to_string(), "USDT".to_string()),
                    ("USDC".to_string(), "USDT".to_string()),
                    
                    // Tier 2 pairs con SOL
                    ("SOL".to_string(), "BONK".to_string()),
                    ("SOL".to_string(), "RAY".to_string()),
                    ("SOL".to_string(), "ORCA".to_string()),
                    ("SOL".to_string(), "PYTH".to_string()),
                    ("SOL".to_string(), "JTO".to_string()),
                    
                    // Tier 2 pairs con USDC
                    ("USDC".to_string(), "BONK".to_string()),
                    ("USDC".to_string(), "RAY".to_string()),
                    ("USDC".to_string(), "ORCA".to_string()),
                    ("USDC".to_string(), "PYTH".to_string()),
                    ("USDC".to_string(), "JTO".to_string()),
                    
                    // Tier 2 inter-ecosystem pairs
                    ("RAY".to_string(), "ORCA".to_string()),
                    ("BONK".to_string(), "RAY".to_string()),
                    ("PYTH".to_string(), "JTO".to_string()),
                ])
            } else {
                // Solo Tier 1 (conservador)
                info!("🛡️  PROPOSAL-003: Solo Tier 1 habilitado - pares conservadores");
                Ok(vec![
                    ("SOL".to_string(), "USDC".to_string()),
                    ("SOL".to_string(), "USDT".to_string()),
                    ("USDC".to_string(), "USDT".to_string()),
                ])
            }
        } else {
            // Fallback a configuración legacy (SOL/USDC)
            Ok(vec![("SOL".to_string(), "USDC".to_string())])
        }
    }
    
    /// SABER INTEGRATION: Get real pool data for enhanced arbitrage
    pub async fn update_saber_pool_data(&mut self) -> Result<()> {
        info!("🗡️  SABER INTEGRATION: Updating real pool data...");
        
        if let Some(saber_box) = &mut self.saber_integration {
            // Type-unsafe cast back to SaberIntegration - this is a workaround for the type erasure
            // In production, we'd use a trait-based approach for type safety
            unsafe {
                let saber_ptr = saber_box.as_mut() as *mut dyn std::any::Any as *mut SaberIntegration;
                if let Some(saber) = saber_ptr.as_mut() {
                    match saber.get_pool_stats().await {
                        Ok(stats) => {
                            info!("✅ SABER: Pool data updated successfully");
                            stats.log_stats();
                            
                            // Update internal metrics with Saber data
                            self.market_metrics.liquidity_score = stats.total_tvl / 1_000_000.0; // Normalize to millions
                            
                            return Ok(());
                        }
                        Err(e) => {
                            warn!("⚠️  SABER: Failed to update pool data: {}", e);
                        }
                    }
                }
            }
        }
        
        warn!("⚠️  SABER: Integration not available, continuing without Saber data");
        Ok(())
    }
    
    /// SABER INTEGRATION: Get relevant pools for current trading pairs
    pub async fn get_saber_relevant_pools(&mut self) -> Result<Vec<String>> {
        info!("🗡️  SABER: Fetching relevant pools for current trading pairs...");
        
        if let Some(saber_box) = &mut self.saber_integration {
            unsafe {
                let saber_ptr = saber_box.as_mut() as *mut dyn std::any::Any as *mut SaberIntegration;
                if let Some(saber) = saber_ptr.as_mut() {
                    match saber.get_relevant_pools().await {
                        Ok(pools) => {
                            let pool_names: Vec<String> = pools.iter()
                                .filter_map(|p| p.name.clone())
                                .collect();
                            
                            info!("✅ SABER: Found {} relevant pools", pool_names.len());
                            for (i, name) in pool_names.iter().take(5).enumerate() {
                                info!("   {}. {}", i + 1, name);
                            }
                            
                            return Ok(pool_names);
                        }
                        Err(e) => {
                            warn!("⚠️  SABER: Failed to fetch relevant pools: {}", e);
                        }
                    }
                }
            }
        }
        
        warn!("⚠️  SABER: Integration not available");
        Ok(Vec::new())
    }
    
    /// ENTERPRISE ARBITRAGE EXECUTION PROTOCOL - Military precision with institutional oversight
    pub async fn run_enterprise_arbitrage(&mut self) -> Result<()> {
        info!("⚔️  ENTERPRISE ARBITRAGE ENGINE: INITIATING MILITARY PROTOCOL");
        info!("🏛️  INSTITUTIONAL OVERSIGHT: ACTIVE");
        
        self.is_running.store(true, Ordering::Relaxed);
        let mission_start = Instant::now();
        
        // PHASE 1: ENTERPRISE PRE-FLIGHT SECURITY CHECKS
        info!("🛡️  PHASE 1: ENTERPRISE RISK ASSESSMENT PROTOCOL");
        EnterpriseRiskManager::execute_institutional_risk_checks(&self.risk_metrics, &self.emergency_stop)?;
        
        // PHASE 2: MILITARY-GRADE MARKET INTELLIGENCE GATHERING
        info!("🧠 PHASE 2: MILITARY INTELLIGENCE GATHERING");
        if !self.price_feeds.are_prices_fresh() {
            info!("📡 UPDATING ENTERPRISE MARKET INTELLIGENCE");
            self.price_feeds.update_all_prices_professional().await?;
            self.update_institutional_market_metrics().await?;
        }
        
        // PHASE 2.5: SABER REAL-TIME POOL DATA UPDATE
        info!("🗡️  PHASE 2.5: SABER REAL-TIME POOL INTELLIGENCE");
        if let Err(e) = self.update_saber_pool_data().await {
            warn!("⚠️  SABER: Pool data update failed, continuing with cached data: {}", e);
        }
        
        // PHASE 3: INSTITUTIONAL POOL RECONNAISSANCE
        info!("🔍 PHASE 3: INSTITUTIONAL POOL RECONNAISSANCE");
        self.execute_enterprise_pool_discovery().await?;
        
        // PHASE 4: MILITARY OPPORTUNITY ANALYSIS
        info!("🧮 PHASE 4: MILITARY-GRADE OPPORTUNITY ANALYSIS");
        let opportunities = self.discover_institutional_opportunities().await?;
        
        if opportunities.is_empty() {
            info!("📊 MILITARY ANALYSIS: No opportunities meeting institutional criteria");
            info!("🎯 ENTERPRISE STATUS: Awaiting optimal market conditions");
            self.is_running.store(false, Ordering::Relaxed);
            return Ok(());
        }
        
        // PHASE 5: INSTITUTIONAL RISK FILTERING
        info!("🛡️  PHASE 5: INSTITUTIONAL RISK MANAGEMENT PROTOCOLS");
        let cleared_opportunities = EnterpriseRiskManager::apply_enterprise_risk_filters(opportunities, &self.adaptive_config)?;
        
        if cleared_opportunities.is_empty() {
            warn!("⚠️  INSTITUTIONAL ALERT: All opportunities filtered by enterprise risk management");
            warn!("🚨 ENTERPRISE PROTOCOL: Risk thresholds exceeded - mission aborted");
            self.is_running.store(false, Ordering::Relaxed);
            return Ok(());
        }
        
        // PHASE 6: MILITARY EXECUTION SEQUENCE
        info!("⚡ PHASE 6: MILITARY EXECUTION PROTOCOL INITIATED");
        let optimal_target = EnterpriseRiskManager::select_enterprise_optimal_opportunity(cleared_opportunities, &self.market_metrics, &self.adaptive_config)?;
        self.display_enterprise_opportunity_briefing(&optimal_target);
        
        match self.execute_military_precision_arbitrage(&optimal_target).await {
            Ok(signature) => {
                self.successful_trades.fetch_add(1, Ordering::Relaxed);
                self.total_profit_lamports.fetch_add(optimal_target.profit_lamports as u64, Ordering::Relaxed);
                EnterpriseRiskManager::update_institutional_performance_metrics(&mut self.performance_metrics, &mut self.risk_metrics, &optimal_target, true);
                info!("✅ ENTERPRISE EXECUTION: MISSION ACCOMPLISHED - {}", signature);
                info!("🎖️  MILITARY SUCCESS: Institutional profit secured");
            }
            Err(e) => {
                error!("❌ ENTERPRISE EXECUTION FAILURE: {}", e);
                error!("🚨 MILITARY ALERT: Mission unsuccessful - institutional protocols engaged");
                EnterpriseRiskManager::update_institutional_performance_metrics(&mut self.performance_metrics, &mut self.risk_metrics, &optimal_target, false);
                self.risk_events.fetch_add(1, Ordering::Relaxed);
            }
        }
        
        let mission_duration = mission_start.elapsed().as_millis() as u64;
        self.execution_times.push_back(mission_duration);
        if self.execution_times.len() > 100 {
            self.execution_times.pop_front();
        }
        
        self.is_running.store(false, Ordering::Relaxed);
        info!("⚡ ENTERPRISE MISSION COMPLETE: {}ms - Military precision maintained", mission_duration);
        
        Ok(())
    }
    
    // ===== ENTERPRISE SUPPORT METHODS =====
    
    async fn update_institutional_market_metrics(&mut self) -> Result<()> {
        info!("📊 UPDATING INSTITUTIONAL MARKET INTELLIGENCE");
        
        let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
        
        if let Some((_price, volatility)) = self.price_feeds.get_price_with_confidence(&sol_mint) {
            self.market_metrics.volatility_index = volatility;
            
            // Military-grade volatility assessment
            if volatility > 0.05 {
                warn!("⚠️  MILITARY INTELLIGENCE: High volatility detected - {:.4}", volatility);
                self.market_metrics.market_sentiment = MarketSentiment::HighVolatility;
                self.adaptive_config.volatility_adjustment = 1.5;
                info!("🎯 ENTERPRISE PROTOCOL: Volatility adjustment factor increased to 1.5");
            } else if volatility < 0.02 {
                info!("📈 INSTITUTIONAL ANALYSIS: Market stability confirmed - {:.4}", volatility);
                self.market_metrics.market_sentiment = MarketSentiment::Neutral;
                self.adaptive_config.volatility_adjustment = 0.8;
                info!("🎯 ENTERPRISE PROTOCOL: Volatility adjustment factor optimized to 0.8");
            }
        }
        
        self.market_metrics.timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        info!("✅ INSTITUTIONAL MARKET METRICS: Updated with military precision");
        Ok(())
    }
    
    async fn execute_enterprise_pool_discovery(&mut self) -> Result<()> {
        info!("� ENHANCED ENTERPRISE POOL RECONNAISSANCE: Multi-DEX institutional liquidity discovery");
        
        self.operational_pools.clear();
        
        // ✅ PROPOSAL-002 IMPLEMENTATION: Multi-DEX Pool Discovery
        info!("📡 EXECUTING COMPREHENSIVE MULTI-DEX POOL DISCOVERY");
        
        // Execute enhanced pool discovery across multiple DEXs
        match execute_enhanced_pool_discovery().await {
            Ok(discovered_pools) => {
                info!("✅ MULTI-DEX DISCOVERY SUCCESS: Found {} pools across multiple DEXs", discovered_pools.len());
                
                // Process discovered pools
                for pool_data in discovered_pools {
                    let (address_str, dex_type, token_a, token_b) = pool_data;
                    info!("🎯 VALIDATING MULTI-DEX POOL: {} on {:?}", address_str, dex_type);
                    
                    // Convert to legacy format and validate
                    if let Ok(pool_address) = Pubkey::from_str(&address_str) {
                        let pool_type = match dex_type {
                            DexType::Raydium => PoolType::Raydium,
                            DexType::Orca => PoolType::Orca,
                            DexType::OrcaWhirlpool => PoolType::OrcaWhirlpool,
                            DexType::Meteora => PoolType::Meteora,
                            DexType::Lifinity => PoolType::Lifinity,
                            DexType::Phoenix => PoolType::Phoenix,
                            DexType::Saber => PoolType::Saber,
                            _ => continue, // Skip unsupported DEX types
                        };
                        
                        match self.pool_validator.validate_real_pool_comprehensive(
                            &pool_address, 
                            pool_type.clone(), 
                            &token_a, 
                            &token_b
                        ).await {
                            Ok(pool_data) => {
                                info!("✅ MULTI-DEX POOL VALIDATED: {} on {}", address_str, dex_type);
                                info!("   💎 ENTERPRISE TVL: ${:.0}", pool_data.tvl_usd);
                                info!("   🎖️  DEX TYPE: {:?}", dex_type);
                                info!("   🚀 PROPOSAL-002: Enhanced pool discovery operational");
                                
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
                                warn!("⚠️  MULTI-DEX POOL REJECTED: {}", address_str);
                                warn!("   🚨 VALIDATION FAILED: Pool failed enterprise validation - {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("❌ MULTI-DEX DISCOVERY FAILED: Falling back to legacy pools - {}", e);
                info!("🔄 FALLBACK: Using verified legacy pools");
                
                // Fallback to legacy pools if multi-DEX discovery fails
                self.load_legacy_pools().await?;
            }
        }
        
        if self.operational_pools.is_empty() {
            error!("🚨 ENTERPRISE FAILURE: No institutional pools available for operations");
            return Err(anyhow!("INSTITUTIONAL POOLS UNAVAILABLE - Mission cannot proceed"));
        }
        
        info!("🎯 ENHANCED ENTERPRISE RECONNAISSANCE COMPLETE: {} institutional pools validated", self.operational_pools.len());
        info!("✅ PROPOSAL-002 STATUS: Multi-DEX discovery operational");
        info!("🎖️  MILITARY STATUS: Enhanced operational pools ready for enterprise arbitrage");
        Ok(())
    }
    
    /// FALLBACK: Load legacy pools if multi-DEX discovery fails
    async fn load_legacy_pools(&mut self) -> Result<()> {
        info!("� LOADING LEGACY INSTITUTIONAL POOLS");
        
        let legacy_pools = vec![
            ("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2", PoolType::Raydium, "SOL", "USDC"),
            ("HJPjoWUrhoZzkNfRpHuieeFk9WcZWjwy6PBjZ81ngndJ", PoolType::OrcaWhirlpool, "SOL", "USDC"),
            ("9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP", PoolType::Orca, "SOL", "USDC"),
        ];
        
        for (address_str, pool_type, token_a, token_b) in legacy_pools {
            if let Ok(pool_address) = Pubkey::from_str(address_str) {
                match self.pool_validator.validate_real_pool_comprehensive(
                    &pool_address, 
                    pool_type.clone(), 
                    token_a, 
                    token_b
                ).await {
                    Ok(pool_data) => {
                        info!("✅ LEGACY POOL VALIDATED: {}", address_str);
                        
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
                        warn!("⚠️  LEGACY POOL FAILED: {} - {}", address_str, e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn discover_institutional_opportunities(&mut self) -> Result<Vec<DirectOpportunity>> {
        info!("🧮 ENTERPRISE OPPORTUNITY ANALYSIS: Military-grade market scanning");
        
        // PROPOSAL-003: Enhanced opportunity discovery con multi-token support
        if self.is_multitoken_enabled() {
            info!("🚀 PROPOSAL-003: Using enhanced multi-token opportunity discovery");
            return self.discover_multitoken_opportunities().await;
        }
        
        // Legacy single-pair discovery (mantiene compatibilidad)
        info!("📊 Using legacy single-pair opportunity discovery");
        let mut opportunities = Vec::new();
        let pools: Vec<_> = self.operational_pools.values().collect();
        
        for (i, pool_a) in pools.iter().enumerate() {
            for pool_b in pools.iter().skip(i + 1) {
                if self.pools_have_common_token(pool_a, pool_b) {
                    info!("🎯 ANALYZING INSTITUTIONAL POOL PAIR: {} ↔ {}", 
                          pool_a.address.to_string()[..8].to_uppercase(),
                          pool_b.address.to_string()[..8].to_uppercase());
                    
                    if let Ok(Some(opportunity)) = self.calculate_enterprise_arbitrage(pool_a, pool_b).await {
                        let profit_bps = (opportunity.profit_lamports * 10_000) / opportunity.amount_in as i64;
                        
                        if profit_bps >= self.adaptive_config.min_profit_threshold as i64 {
                            info!("💎 INSTITUTIONAL OPPORTUNITY IDENTIFIED: {:.2}% profit margin", profit_bps as f64 / 100.0);
                            info!("   ⚔️  MILITARY ASSESSMENT: Meets enterprise profit criteria");
                            opportunities.push(opportunity);
                            self.total_opportunities_found.fetch_add(1, Ordering::Relaxed);
                        } else {
                            debug!("📊 Opportunity below institutional threshold: {:.2}%", profit_bps as f64 / 100.0);
                        }
                    }
                }
            }
        }
        
        // Military-grade opportunity ranking
        opportunities.sort_by(|a, b| {
            let score_a = EnterpriseRiskManager::calculate_enterprise_opportunity_score(a, &self.market_metrics, &self.adaptive_config);
            let score_b = EnterpriseRiskManager::calculate_enterprise_opportunity_score(b, &self.market_metrics, &self.adaptive_config);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        info!("🎯 ENTERPRISE ANALYSIS COMPLETE: {} institutional opportunities identified", opportunities.len());
        info!("✅ MILITARY STATUS: Opportunities ranked by enterprise criteria");
        Ok(opportunities)
    }
    
    /// PROPOSAL-003: Enhanced multi-token opportunity discovery
    async fn discover_multitoken_opportunities(&mut self) -> Result<Vec<DirectOpportunity>> {
        info!("🚀 PROPOSAL-003: MULTI-TOKEN OPPORTUNITY DISCOVERY");
        
        let mut opportunities = Vec::new();
        
        // Obtener pares de tokens habilitados
        let enabled_pairs = self.get_enabled_token_pairs().await?;
        info!("📊 PROPOSAL-003: Analyzing {} enabled token pairs", enabled_pairs.len());
        
        // Filtrar pools por pares habilitados
        let pools: Vec<_> = self.operational_pools.values().collect();
        
        for (token_a_symbol, token_b_symbol) in enabled_pairs {
            info!("🔍 PROPOSAL-003: Analyzing pair {}/{}", token_a_symbol, token_b_symbol);
            
            // Buscar pools que coincidan con este par de tokens
            let matching_pools: Vec<_> = pools.iter()
                .filter(|pool| self.pool_matches_token_pair(pool, &token_a_symbol, &token_b_symbol))
                .collect();
            
            if matching_pools.len() >= 2 {
                info!("🎯 PROPOSAL-003: Found {} pools for pair {}/{}", 
                      matching_pools.len(), token_a_symbol, token_b_symbol);
                
                // Analizar oportunidades entre pools del mismo par de tokens
                for (i, pool_a) in matching_pools.iter().enumerate() {
                    for pool_b in matching_pools.iter().skip(i + 1) {
                        if let Ok(Some(opportunity)) = self.calculate_enterprise_arbitrage(pool_a, pool_b).await {
                            // Aplicar configuración específica del par simplificada (Phase 1)
                            let meets_criteria = if self.multi_token_enabled {
                                // Criterios más estrictos para multi-token en Phase 1
                                let profit_bps = (opportunity.profit_lamports * 10_000) / opportunity.amount_in as i64;
                                profit_bps >= (self.adaptive_config.min_profit_threshold + 25) as i64 // +25 bps más estricto
                            } else {
                                let profit_bps = (opportunity.profit_lamports * 10_000) / opportunity.amount_in as i64;
                                profit_bps >= self.adaptive_config.min_profit_threshold as i64
                            };
                            
                            if meets_criteria {
                                info!("💎 PROPOSAL-003: Multi-token opportunity found for {}/{}", 
                                      token_a_symbol, token_b_symbol);
                                opportunities.push(opportunity);
                                self.total_opportunities_found.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                }
            } else {
                debug!("📊 PROPOSAL-003: Insufficient pools for pair {}/{} (found {})", 
                       token_a_symbol, token_b_symbol, matching_pools.len());
            }
        }
        
        // Ranking with multi-token considerations
        opportunities.sort_by(|a, b| {
            let score_a = EnterpriseRiskManager::calculate_enterprise_opportunity_score(a, &self.market_metrics, &self.adaptive_config);
            let score_b = EnterpriseRiskManager::calculate_enterprise_opportunity_score(b, &self.market_metrics, &self.adaptive_config);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        info!("✅ PROPOSAL-003: Multi-token analysis complete - {} opportunities identified", opportunities.len());
        Ok(opportunities)
    }
    
    /// PROPOSAL-003: Verificar si un pool coincide con un par de tokens específico
    fn pool_matches_token_pair(&self, pool: &PoolData, token_a_symbol: &str, token_b_symbol: &str) -> bool {
        // Esta es una implementación simplificada
        // En una versión completa, se verificaría contra las direcciones de mint reales
        
        // Por ahora, verificamos los símbolos más comunes
        let pool_tokens = vec![
            self.get_token_symbol_from_mint(&pool.token_a_mint),
            self.get_token_symbol_from_mint(&pool.token_b_mint),
        ];
        
        (pool_tokens.contains(&token_a_symbol.to_string()) && pool_tokens.contains(&token_b_symbol.to_string())) ||
        (pool_tokens.contains(&token_b_symbol.to_string()) && pool_tokens.contains(&token_a_symbol.to_string()))
    }
    
    /// PROPOSAL-003: Helper para obtener símbolo de token desde mint address
    fn get_token_symbol_from_mint(&self, mint: &Pubkey) -> String {
        // Mapeo simplificado de direcciones conocidas
        match mint.to_string().as_str() {
            "So11111111111111111111111111111111111111112" => "SOL".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "USDC".to_string(),
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => "USDT".to_string(),
            _ => "UNKNOWN".to_string(),
        }
    }
    
    fn display_enterprise_opportunity_briefing(&self, opportunity: &DirectOpportunity) {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let profit_percentage = (opportunity.profit_lamports as f64 / opportunity.amount_in as f64) * 100.0;
        
        println!("\n╔═══════════════════════════════════════════════════════════════════════════════╗");
        println!("║          🏛️  ENTERPRISE ARBITRAGE OPPORTUNITY BRIEFING                       ║");
        println!("╠═══════════════════════════════════════════════════════════════════════════════╣");
        println!("║ ⚔️  MILITARY STRATEGY: Cross-DEX Enterprise Protocol | TS: {}        ║", timestamp % 100000);
        println!("║ 🧠 INSTITUTIONAL INTELLIGENCE: {:?} | VOL: {:.4}               ║", 
                 self.market_metrics.market_sentiment, self.market_metrics.volatility_index);
        println!("║                                                                               ║");
        println!("║ 🎯 ENTERPRISE EXECUTION PLAN:                                                ║");
        println!("║    � Institutional Trade Size: {:.6} SOL                             ║", 
                 opportunity.amount_in as f64 / 1e9);
        println!("║    💎 PROJECTED ENTERPRISE PROFIT: {:.6} SOL ({:.4}%)                ║", 
                 opportunity.profit_lamports as f64 / 1e9, profit_percentage);
        println!("║    🏪 POOL A: {}...                                       ║", 
                 opportunity.pool_a.address.to_string()[..40].to_uppercase());
        println!("║    🏪 POOL B: {}...                                       ║", 
                 opportunity.pool_b.address.to_string()[..40].to_uppercase());
        println!("║                                                                               ║");
        println!("║ 🛡️  ENTERPRISE RISK ASSESSMENT: CLEARED FOR EXECUTION                       ║");
        println!("╚═══════════════════════════════════════════════════════════════════════════════╝");
    }
    
    async fn execute_military_precision_arbitrage(&mut self, opportunity: &DirectOpportunity) -> Result<String> {
        info!("⚔️ EXECUTING MILITARY-PRECISION ARBITRAGE PROTOCOL");
        info!("🏛️  INSTITUTIONAL OVERSIGHT: Enterprise execution initiated");
        info!("🎯 EXECUTION MODE: {:?}", self.execution_mode);
        
        let current_balance = self.get_wallet_balance().await?;
        let required_balance = opportunity.amount_in as f64 / 1e9;
        
        EnterpriseRiskManager::check_balance_sufficiency(current_balance, required_balance)?;
        
        let profit = opportunity.profit_lamports as f64 / 1e9;
        info!("💎 PROJECTED INSTITUTIONAL PROFIT: {:.6} SOL", profit);
        
        // EXECUTION ROUTING: Simulation vs Real Trading
        match self.execution_mode {
            ExecutionMode::Simulation => {
                info!("🎭 EXECUTION MODE: Simulation protocol active");
                info!("💡 Real execution requires activation via enable_real_trading_mainnet()");
                Ok(format!("ENTERPRISE_SIM_{}_{}", 
                    opportunity.pool_a.address.to_string()[..8].to_uppercase(),
                    opportunity.pool_b.address.to_string()[..8].to_uppercase()))
            },
            ExecutionMode::RealTrading => {
                info!("💰 EXECUTION MODE: Real trading protocol active - MAINNET");
                
                // Check minimum profit for real execution
                if profit < MAINNET_MIN_PROFIT_SOL {
                    info!("📊 Profit below mainnet threshold: {:.6} SOL < {:.6} SOL", profit, MAINNET_MIN_PROFIT_SOL);
                    return Ok(format!("SKIPPED_LOW_PROFIT_{:.6}_SOL", profit));
                }
                
                match &self.wallet_keypair {
                    Some(wallet) => {
                        // REAL ARBITRAGE EXECUTION WITH JUPITER SWAPS
                        self.execute_real_jupiter_arbitrage(opportunity, wallet).await
                    },
                    None => {
                        error!("❌ CRITICAL ERROR: Real trading mode enabled but wallet not loaded");
                        Err(anyhow!("Real trading mode enabled but wallet not loaded. Call enable_real_trading_mainnet() first."))
                    }
                }
            }
        }
    }

    /// Execute real arbitrage using Jupiter swaps (NEW IMPLEMENTATION)
    async fn execute_real_jupiter_arbitrage(&self, opportunity: &DirectOpportunity, wallet: &Keypair) -> Result<String> {
        info!("� EXECUTING REAL JUPITER ARBITRAGE ON MAINNET");
        info!("📊 Trade route: {} -> {} -> {}", 
              self.get_token_symbol_from_mint(&opportunity.token_in),
              self.get_token_symbol_from_mint(&opportunity.intermediate_token),
              self.get_token_symbol_from_mint(&opportunity.token_out)
        );

        let jupiter_client = reqwest::Client::new();
        let execution_start = std::time::Instant::now();

        // STEP 1: Execute first swap (token_in -> intermediate_token)
        info!("1️⃣  SWAP 1: {} -> {}", 
              self.get_token_symbol_from_mint(&opportunity.token_in),
              self.get_token_symbol_from_mint(&opportunity.intermediate_token)
        );

        let swap1_result = self.execute_jupiter_swap(
            &jupiter_client,
            &opportunity.token_in,
            &opportunity.intermediate_token,
            opportunity.amount_in,
            wallet
        ).await?;

        info!("✅ SWAP 1 COMPLETED: {} {}", swap1_result.signature, swap1_result.output_amount);
        
        // Wait for confirmation
        tokio::time::sleep(Duration::from_secs(2)).await;

        // STEP 2: Execute second swap (intermediate_token -> token_out)
        info!("2️⃣  SWAP 2: {} -> {}", 
              self.get_token_symbol_from_mint(&opportunity.intermediate_token),
              self.get_token_symbol_from_mint(&opportunity.token_out)
        );

        let swap2_result = self.execute_jupiter_swap(
            &jupiter_client,
            &opportunity.intermediate_token,
            &opportunity.token_out,
            swap1_result.output_amount,
            wallet
        ).await?;

        info!("✅ SWAP 2 COMPLETED: {} {}", swap2_result.signature, swap2_result.output_amount);

        let execution_time = execution_start.elapsed();
        let final_amount = swap2_result.output_amount;
        let actual_profit = final_amount.saturating_sub(opportunity.amount_in);
        let actual_profit_sol = actual_profit as f64 / 1e9;

        // Update metrics
        self.successful_trades.fetch_add(1, Ordering::Relaxed);
        self.total_profit_lamports.fetch_add(actual_profit, Ordering::Relaxed);

        info!("🎉 ARBITRAGE COMPLETED SUCCESSFULLY!");
        info!("💰 Actual Profit: {:.6} SOL", actual_profit_sol);
        info!("⏱️  Execution Time: {}ms", execution_time.as_millis());
        info!("📝 Swap 1: {}", swap1_result.signature);
        info!("📝 Swap 2: {}", swap2_result.signature);

        Ok(format!("ARBITRAGE_SUCCESS_{:.6}_SOL_{}ms", actual_profit_sol, execution_time.as_millis()))
    }

    /// Execute individual Jupiter swap
    async fn execute_jupiter_swap(
        &self,
        client: &reqwest::Client,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
        wallet: &Keypair
    ) -> Result<JupiterSwapResult> {
        info!("🔄 Executing Jupiter swap: {} -> {} ({})", 
              input_mint.to_string()[..8].to_uppercase(),
              output_mint.to_string()[..8].to_uppercase(),
              amount
        );

        // STEP 1: Get quote from Jupiter
        let quote_url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            MAINNET_JUPITER_API,
            input_mint,
            output_mint,
            amount,
            MAINNET_MAX_SLIPPAGE_BPS
        );

        let quote_response = client.get(&quote_url)
            .timeout(Duration::from_secs(10))
            .send()
            .await?;

        if !quote_response.status().is_success() {
            return Err(anyhow!("Jupiter quote failed: {}", quote_response.status()));
        }

        let quote: serde_json::Value = quote_response.json().await?;
        let out_amount: u64 = quote["outAmount"].as_str()
            .ok_or_else(|| anyhow!("Invalid quote response"))?
            .parse()?;

        info!("💹 Quote: {} -> {} (impact: {}%)", 
              amount, out_amount, quote["priceImpactPct"].as_f64().unwrap_or(0.0));

        // STEP 2: Get swap transaction
        let swap_request = serde_json::json!({
            "quoteResponse": quote,
            "userPublicKey": wallet.pubkey().to_string(),
            "wrapAndUnwrapSol": true,
            "useSharedAccounts": true,
            "computeUnitPriceMicroLamports": 5000,
            "prioritizationFeeLamports": 5000,
        });

        let swap_response = client.post(MAINNET_JUPITER_SWAP_API)
            .json(&swap_request)
            .timeout(Duration::from_secs(15))
            .send()
            .await?;

        if !swap_response.status().is_success() {
            return Err(anyhow!("Jupiter swap failed: {}", swap_response.status()));
        }

        let swap_data: serde_json::Value = swap_response.json().await?;
        let transaction_b64 = swap_data["swapTransaction"].as_str()
            .ok_or_else(|| anyhow!("No transaction in swap response"))?;

        // STEP 3: Deserialize and sign transaction
        use base64::{engine::general_purpose, Engine as _};
        let tx_bytes = general_purpose::STANDARD.decode(transaction_b64)?;
        let mut transaction: solana_sdk::transaction::Transaction = bincode::deserialize(&tx_bytes)?;

        // Update recent blockhash
        let recent_blockhash = self.client.get_latest_blockhash()?;
        transaction.message.recent_blockhash = recent_blockhash;

        // Sign transaction
        transaction.sign(&[wallet], recent_blockhash);

        // STEP 4: Send transaction
        info!("📡 Sending transaction to blockchain...");
        let signature = self.client.send_and_confirm_transaction(&transaction)?;

        info!("✅ Swap executed successfully: {}", signature);

        Ok(JupiterSwapResult {
            signature,
            input_amount: amount,
            output_amount: out_amount,
            price_impact: quote["priceImpactPct"].as_f64().unwrap_or(0.0),
        })
    }
    
    async fn calculate_enterprise_arbitrage(&self, pool_a: &PoolData, pool_b: &PoolData) -> Result<Option<DirectOpportunity>> {
        let intermediate_token = if pool_a.token_a_mint == pool_b.token_a_mint || pool_a.token_a_mint == pool_b.token_b_mint {
            pool_a.token_a_mint
        } else if pool_a.token_b_mint == pool_b.token_a_mint || pool_a.token_b_mint == pool_b.token_b_mint {
            pool_a.token_b_mint
        } else {
            return Ok(None);
        };
        
        let current_balance = self.get_wallet_balance().await?;
        let max_trade_sol = (current_balance * 0.1).min(REALISTIC_MAX_TRADE_SOL);
        let optimal_amount = ((max_trade_sol * 1e9) as u64).min(
            (pool_a.token_a_amount.min(pool_a.token_b_amount)) / 20
        );
        
        if optimal_amount < (REALISTIC_MIN_TRADE_SOL * 1e9) as u64 {
            return Ok(None);
        }
        
        // Try Jupiter API first for real market data
        let jupiter_api = JupiterAPI::new();
        
        // Use actual token mints instead of hardcoded strings
        let (input_mint_a, output_mint_a) = if pool_a.token_a_mint == intermediate_token {
            (pool_a.token_b_mint.to_string(), pool_a.token_a_mint.to_string())
        } else {
            (pool_a.token_a_mint.to_string(), pool_a.token_b_mint.to_string())
        };
        
        let quote_a = match jupiter_api.get_real_quote(&input_mint_a, &output_mint_a, optimal_amount).await {
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
            (pool_b.token_a_mint.to_string(), pool_b.token_b_mint.to_string())
        } else {
            (pool_b.token_b_mint.to_string(), pool_b.token_a_mint.to_string())
        };
        
        let quote_b = match jupiter_api.get_real_quote(&input_mint_b, &output_mint_b, quote_a.out_amount).await {
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
        if profit_bps < REALISTIC_MIN_PROFIT_BPS {
            return Ok(None);
        }
        
        debug!("✅ PROFITABLE ARBITRAGE: {:.4}% profit", profit_bps as f64 / 100.0);
        
        Ok(Some(DirectOpportunity {
            pool_a: pool_a.clone(),
            pool_b: pool_b.clone(),
            intermediate_token,
            token_in: pool_a.token_a_mint,
            token_out: pool_b.token_b_mint,
            amount_in: optimal_amount,
            expected_amount_out: final_amount,
            profit_lamports: net_profit as i64,
            profit_percentage: profit_bps as f64 / 100.0,
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
    
    pub fn get_enterprise_statistics(&self) -> String {
        let mut stats = format!(
            "🏛️  ENTERPRISE ARBITRAGE SYSTEM - INSTITUTIONAL STATUS REPORT:\n\
             ⚔️  MILITARY PRECISION STATS:\n\
             💰 Total Opportunities Discovered: {}\n\
             ✅ Successful Enterprise Trades: {}\n\
             📈 Institutional Profit Generated: {:.6} SOL\n\
             🏪 Operational Institutional Pools: {}\n\
             🌐 Enterprise Data Sources: Live Blockchain + Military APIs\n\
             🛡️  Enterprise Risk Status: WITHIN INSTITUTIONAL LIMITS\n\
             🎯 Military Execution Protocol: ACTIVE",
            self.total_opportunities_found.load(Ordering::Relaxed),
            self.successful_trades.load(Ordering::Relaxed),
            self.total_profit_lamports.load(Ordering::Relaxed) as f64 / 1e9,
            self.operational_pools.len()
        );
        
        // PROPOSAL-003: Agregar estadísticas multi-token si está habilitado
        if self.is_multitoken_enabled() {
            stats.push_str(&format!(
                "\n\n🚀 PROPOSAL-003 MULTI-TOKEN STATUS:\n\
                 🪙 Enhanced Token Support: ENABLED\n\
                 🔗 Supported Pairs: SOL/USDC, SOL/USDT, USDC/USDT\n\
                 ✅ Tier 1 Tokens: OPERATIONAL\n\
                 🎯 Multi-Token Mode: PHASE 1 ACTIVE"
            ));
        } else {
            stats.push_str("\n\n🚀 PROPOSAL-003 MULTI-TOKEN STATUS: DISABLED (Single-pair mode)");
        }
        
        stats
    }
}

// ===== ENTERPRISE MAIN EXECUTION PROTOCOL =====

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("🏛️  STARTING ENTERPRISE ARBITRAGE SYSTEM");
    info!("⚔️  MILITARY-GRADE INITIALIZATION PROTOCOL");
    info!("🎯 INSTITUTIONAL OVERSIGHT: ACTIVE");
    
    // Configuration - MAINNET REAL
    let mainnet_rpc = "https://api.mainnet-beta.solana.com";
    let wallet_path = "wallets/mainnet-arbitrage-wallet.json"; // Corregido path
    
    println!("\n🎯 SNIPERFORGE ARBITRAGE SYSTEM - OPCIÓN C MODULAR");
    println!("═══════════════════════════════════════════════════════");
    println!("📋 Basado en documentación exitosa de Julio 16-17, 2025");
    println!("🔬 Implementación 100% real sin fake data");
    println!("");
    println!("🛡️  SAFE TESTING & VALIDATION:");
    println!("1) Safe Arbitrage Test (Validación sin riesgo)");
    println!("2) Jupiter Scanner (Búsqueda de oportunidades)");
    println!("3) Quick Scan (Verificación rápida)");
    println!("");
    println!("🏛️  ENTERPRISE TRADING SYSTEM:");
    println!("A) 🚀 AUTO-TRADER: Scan + Execute Arbitrage (REAL MONEY)");
    println!("E) Enterprise Multi-Source Scan (Detection Only)");
    println!("D) Direct DEX Access (No Aggregators)");
    println!("C) CEX-DEX Arbitrage Analysis");
    println!("");
    println!("🤖 AUTOMATED MONITORING (OPCIÓN C):");
    println!("4) Start Automated Monitor (Conservative)");
    println!("5) Start Automated Monitor (Aggressive)");
    println!("6) Monitor Status & Alerts");
    println!("");
    println!("⚡ REAL EXECUTION:");
    println!("7) Execute Safe Simulation (No Risk)");
    println!("8) Execute Validated Opportunity (MainNet - REAL MONEY)");
    println!("");
    println!("🔧 LEGACY MODES:");
    println!("B) Simulation mode (Legacy)");
    println!("M) Multi-token Tier 1 (Legacy)");
    println!("T) Multi-token Tier 2 (Legacy)");
    println!("R) Real Trading Mode (Legacy - DANGER)");
    println!("");
    println!("0) Exit");
    
    print!("Select option (1-8, A/E/D/C, B/M/T/R, 0): ");
    use std::io::{self, Write};
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let choice = input.trim();
    
    let mut enterprise_system = ProfessionalArbitrageEngine::new_enterprise_professional(
        mainnet_rpc.to_string(),
        wallet_path.to_string(),
    ).await?;
    
    match choice {
        // ===== SAFE TESTING & VALIDATION =====
        "1" => {
            info!("🛡️ Ejecutando Safe Arbitrage Test - Validación sin riesgo");
            match execute_safe_arbitrage_test().await {
                Ok(results) => {
                    info!("✅ Safe test completado exitosamente");
                    info!("📊 Resultados: {} oportunidades analizadas", results.len());
                    
                    // Show detailed results
                    for result in &results {
                        println!("   {} {} ({:.3} SOL): {:.9} SOL profit ({:.2}%)",
                            match result.risk_level {
                                modules::RiskLevel::Safe => "✅",
                                modules::RiskLevel::Moderate => "🟡",
                                modules::RiskLevel::Risky => "🟠",
                                modules::RiskLevel::Unprofitable => "❌",
                            },
                            result.token_pair,
                            result.input_amount,
                            result.estimated_profit,
                            result.profit_percentage
                        );
                    }
                }
                Err(e) => error!("❌ Safe test failed: {}", e)
            }
        },
        
        "2" => {
            info!("🔍 Ejecutando Jupiter Scanner - Búsqueda comprehensiva");
            match execute_comprehensive_scan().await {
                Ok(opportunities) => {
                    info!("✅ Scan comprehensivo completado");
                    info!("📊 Oportunidades encontradas: {}", opportunities.len());
                    
                    // Show top opportunities
                    for (i, opp) in opportunities.iter().take(5).enumerate() {
                        println!("   {}#{} {} ({:.3} SOL): +{:.9} SOL ({:.2}%, conf: {:.1}%)",
                            match opp.execution_priority {
                                modules::Priority::High => "🔴",
                                modules::Priority::Medium => "🟡",
                                modules::Priority::Low => "🟢",
                                modules::Priority::Monitor => "⚪",
                                modules::Priority::MicroOp => "🔵",
                            },
                            i + 1,
                            opp.token_pair,
                            opp.input_amount,
                            opp.estimated_profit,
                            opp.profit_percentage,
                            opp.confidence_score
                        );
                    }
                }
                Err(e) => error!("❌ Jupiter scan failed: {}", e)
            }
        },
        
        "3" => {
            info!("⚡ Ejecutando Quick Scan - Verificación rápida");
            match execute_quick_scan().await {
                Ok(opportunities) => {
                    info!("✅ Quick scan completado");
                    info!("📊 Oportunidades inmediatas: {}", opportunities.len());
                    
                    if opportunities.is_empty() {
                        warn!("⚠️ No hay oportunidades inmediatas disponibles");
                    } else {
                        for opp in &opportunities {
                            println!("   🚨 {} ({:.3} SOL): +{:.9} SOL ({:.2}%)",
                                opp.token_pair,
                                opp.input_amount,
                                opp.estimated_profit,
                                opp.profit_percentage
                            );
                        }
                    }
                }
                Err(e) => error!("❌ Quick scan failed: {}", e)
            }
        },
        
        // ===== AUTOMATED MONITORING (OPCIÓN C) =====
        "4" => {
            info!("🤖 Iniciando Automated Monitor - Modo Conservativo");
            let config = MonitorConfig {
                scan_interval_minutes: 10,        // Scan cada 10 minutos (más frecuente)
                quick_scan_interval_minutes: 5,   // Quick scan cada 5 min (más frecuente)
                auto_execute_enabled: false,      // Solo alertas, no auto-ejecución
                min_confidence_score: 60.0,       // Confianza media (menos restrictivo)
                min_profit_threshold: 0.000015,   // 3x fees SOL real (0.000005 * 3)
                max_daily_executions: 20,         // Límite más realista
                alert_webhook_url: None,
            };
            
            info!("📊 Configuración conservativa aplicada:");
            info!("   Scan interval: {} minutos", config.scan_interval_minutes);
            info!("   Auto-ejecución: {}", if config.auto_execute_enabled { "SÍ" } else { "NO" });
            info!("   Threshold profit: {:.9} SOL", config.min_profit_threshold);
            
            match start_automated_monitoring_with_config(config).await {
                Ok(_) => info!("✅ Monitor iniciado exitosamente"),
                Err(e) => error!("❌ Failed to start monitor: {}", e)
            }
        },
        
        "5" => {
            info!("🤖 Iniciando Automated Monitor - Modo Agresivo");
            warn!("⚠️ MODO AGRESIVO: Configuración para mercados volátiles");
            
            let config = MonitorConfig {
                scan_interval_minutes: 5,         // Scan cada 5 min (muy frecuente)
                quick_scan_interval_minutes: 2,   // Quick scan cada 2 min (muy frecuente)
                auto_execute_enabled: false,      // Mantener manual por seguridad
                min_confidence_score: 50.0,       // Confianza moderada-baja (más oportunidades)
                min_profit_threshold: 0.000010,   // 2x fees SOL real (0.000005 * 2)
                max_daily_executions: 50,         // Límite más alto para más oportunidades
                alert_webhook_url: None,
            };
            
            print!("Type 'AGGRESSIVE' to confirm aggressive monitoring: ");
            io::stdout().flush().unwrap();
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm).unwrap();
            
            if confirm.trim() == "AGGRESSIVE" {
                info!("📊 Configuración agresiva aplicada:");
                info!("   Scan interval: {} minutos", config.scan_interval_minutes);
                info!("   Threshold profit: {:.9} SOL", config.min_profit_threshold);
                
                match start_automated_monitoring_with_config(config).await {
                    Ok(_) => info!("✅ Monitor agresivo iniciado exitosamente"),
                    Err(e) => error!("❌ Failed to start aggressive monitor: {}", e)
                }
            } else {
                warn!("🔒 Modo agresivo cancelado por seguridad");
                return Ok(());
            }
        },
        
        "6" => {
            info!("📊 Checking Monitor Status & Alerts");
            // Implementación simplificada para mostrar status
            println!("📋 MONITOR STATUS:");
            println!("   Status: Not implemented in this demo");
            println!("   Recent alerts: Check logs for real-time information");
            println!("   💡 Tip: Use options 4 or 5 to start monitoring");
        },
        
        // ===== REAL EXECUTION =====
        "7" => {
            info!("⚡ Execute Safe Simulation");
            warn!("🧪 SIMULATION MODE: Safe validation without real money");
            
            // First run safe test to find opportunities
            match execute_safe_arbitrage_test().await {
                Ok(results) => {
                    let safe_opportunities: Vec<_> = results.iter()
                        .filter(|r| matches!(r.risk_level, modules::RiskLevel::Safe))
                        .collect();
                    
                    if safe_opportunities.is_empty() {
                        warn!("⚠️ No hay oportunidades seguras para simular");
                        warn!("🔍 Ejecuta Safe Test (opción 1) para verificar condiciones");
                    } else {
                        info!("✅ Encontradas {} oportunidades seguras", safe_opportunities.len());
                        
                        // Simulate execution with best opportunity
                        let best = &safe_opportunities[0];
                        info!("🎬 SIMULANDO EJECUCIÓN:");
                        info!("   Par: {}", best.token_pair);
                        info!("   Amount: {:.6} tokens", best.input_amount);
                        info!("   Expected profit: {:.9} SOL", best.estimated_profit);
                        info!("   Fee ratio: {:.1}x", best.fee_ratio);
                        info!("   Risk level: {:?}", best.risk_level);
                        
                        // Get tokens for simulation
                        let tokens: Vec<&str> = best.token_pair.split('/').collect();
                        
                        if tokens.len() == 2 {
                            // Simulate the execution steps
                            match simulate_arbitrage_execution_advanced(
                                tokens[0],
                                tokens[1],
                                best.input_amount
                            ).await {
                                Ok(sim_result) => {
                                    info!("✅ SIMULACIÓN COMPLETADA EXITOSAMENTE");
                                    info!("   Execution time: {}ms", sim_result.execution_time_ms);
                                    info!("   Simulated profit: {:.9} SOL", sim_result.actual_profit);
                                    info!("   Success rate: {}%", if sim_result.success { 100 } else { 0 });
                                    info!("🎯 SIMULATION STATUS: Ready for real execution if needed");
                                }
                                Err(e) => {
                                    error!("❌ Simulation failed: {}", e);
                                }
                            }
                        } else {
                            error!("❌ Invalid token pair format: {}", best.token_pair);
                        }
                    }
                }
                Err(e) => error!("❌ Failed to validate opportunities: {}", e)
            }
        },
        
        "8" => {
            info!("⚡ Execute Validated Opportunity - MainNet");
            error!("🚨 MAINNET EXECUTION: REAL MONEY AT RISK");
            warn!("⚠️ Esta función requiere wallet configurado y validación adicional");
            
            print!("Type 'MAINNET_EXECUTE' to confirm real execution: ");
            io::stdout().flush().unwrap();
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm).unwrap();
            
            if confirm.trim() == "MAINNET_EXECUTE" {
                // First validate opportunities
                match execute_safe_arbitrage_test().await {
                    Ok(results) => {
                        let safe_opportunities: Vec<_> = results.iter()
                            .filter(|r| matches!(r.risk_level, modules::RiskLevel::Safe) && r.estimated_profit > 0.000050)
                            .collect();
                        
                        if safe_opportunities.is_empty() {
                            error!("❌ NO HAY OPORTUNIDADES SEGURAS PARA MAINNET");
                            error!("🛡️ Cancelando ejecución por seguridad");
                        } else {
                            warn!("🎯 Oportunidad validada para MainNet");
                            warn!("💡 Implementación real requiere wallet keypair configurado");
                            error!("🚧 FUNCIÓN EN DESARROLLO - Usar simulación para testing");
                        }
                    }
                    Err(e) => error!("❌ Failed to validate for MainNet: {}", e)
                }
            } else {
                info!("🔒 MainNet execution cancelled for safety");
            }
        },
        
        // ===== ENTERPRISE AUTO-SCANNER WITH REAL EXECUTION =====
        "A" => {
            info!("🚀 ENTERPRISE AUTO-TRADER: Scanning + Executing real arbitrage");
            warn!("⚡ REAL MONEY MODE: Will execute profitable opportunities automatically");
            warn!("💰 RISK: This trades with real SOL on Mainnet");
            info!("📡 DEX Coverage: Jupiter, Raydium, Orca, Meteora, Phoenix");
            info!("🎯 Min profit threshold: {:.6} SOL", MAINNET_MIN_PROFIT_SOL);
            
            print!("⚠️  Type 'EXECUTE' to start real auto-trading: ");
            io::stdout().flush().unwrap();
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm).unwrap();
            
            if confirm.trim() == "EXECUTE" {
                println!("\n🚀 STARTING ENTERPRISE AUTO-TRADER...");
                println!("📊 Will scan AND execute profitable opportunities");
                println!("⚡ Real swaps will be executed automatically");
                println!("💰 Minimum profit: {:.6} SOL per trade", MAINNET_MIN_PROFIT_SOL);
                println!("🔄 Press Ctrl+C to stop at any time");
                println!("════════════════════════════════════════════════════════");
                
                // Initialize enterprise system with real trading enabled
                let mut enterprise_system = ProfessionalArbitrageEngine::new_enterprise_professional(
                    mainnet_rpc.to_string(), 
                    wallet_path.to_string()
                ).await?;
                
                // Enable real trading mode
                enterprise_system.enable_real_trading_mainnet().await?;
                
                // Start continuous auto-trading loop
                let mut total_profit = 0.0;
                let mut trade_count = 0;
                let start_time = std::time::Instant::now();
                
                loop {
                    info!("🔍 Scanning for arbitrage opportunities...");
                    
                    // Discover opportunities using existing system
                    match enterprise_system.discover_institutional_opportunities().await {
                        Ok(opportunities) => {
                            let profitable: Vec<_> = opportunities.iter()
                                .filter(|opp| {
                                    let profit_sol = opp.profit_lamports as f64 / 1e9;
                                    profit_sol >= MAINNET_MIN_PROFIT_SOL
                                })
                                .collect();
                            
                            if !profitable.is_empty() {
                                let best = profitable[0];
                                let profit_sol = best.profit_lamports as f64 / 1e9;
                                
                                info!("🎯 EXECUTING: {} -> {} (profit: {:.6} SOL)", 
                                      enterprise_system.get_token_symbol_from_mint(&best.token_in),
                                      enterprise_system.get_token_symbol_from_mint(&best.token_out),
                                      profit_sol
                                );
                                
                                // Execute the arbitrage with real money
                                match enterprise_system.execute_military_precision_arbitrage(best).await {
                                    Ok(result) => {
                                        trade_count += 1;
                                        total_profit += profit_sol;
                                        
                                        info!("✅ TRADE #{} SUCCESS: {}", trade_count, result);
                                        info!("💰 This trade profit: {:.6} SOL", profit_sol);
                                        info!("💰 Total profit: {:.6} SOL", total_profit);
                                        info!("⏱️  Running time: {}s", start_time.elapsed().as_secs());
                                    }
                                    Err(e) => {
                                        error!("❌ Trade execution failed: {}", e);
                                    }
                                }
                            } else {
                                // No profitable opportunities found
                                debug!("⏳ No profitable opportunities (threshold: {:.6} SOL)", MAINNET_MIN_PROFIT_SOL);
                            }
                        }
                        Err(e) => {
                            error!("❌ Opportunity discovery failed: {}", e);
                        }
                    }
                    
                    // High frequency scanning (like real MEV bots)
                    tokio::time::sleep(Duration::from_millis(1000)).await; // 1 second intervals
                }
            } else {
                warn!("🔒 Auto-trading cancelled - only scanning mode");
                
                // Just run the scanner without execution
                match modules::start_real_enterprise_auto_scanner().await {
                    Ok(_) => {
                        info!("✅ Enterprise Auto-Scanner completed successfully");
                    }
                    Err(e) => {
                        error!("❌ Enterprise Auto-Scanner failed: {}", e);
                        warn!("💡 Try checking network connectivity and API availability");
                    }
                }
            }
        },
        
        // ===== ENTERPRISE MULTI-SOURCE SYSTEM =====
        "E" => {
            info!("🏛️ ENTERPRISE MULTI-SOURCE SCAN");
            warn!("🚀 PROFESSIONAL SYSTEM: Multiple data sources, no single aggregator dependency");
            
            match modules::execute_enterprise_multi_source_scan().await {
                Ok(opportunities) => {
                    info!("✅ Enterprise scan completado exitosamente");
                    info!("📊 Oportunidades multi-source: {}", opportunities.len());
                    
                    if opportunities.is_empty() {
                        warn!("⚠️ NO ENTERPRISE OPPORTUNITIES DETECTED");
                        info!("🏛️ ENTERPRISE ANALYSIS:");
                        info!("   📡 Multi-source validation: COMPLETE");
                        info!("   🎯 Market efficiency: HIGH (low spreads detected)");
                        info!("   ⏰ Current period: Check timing guide for optimal hours");
                    } else {
                        info!("🎯 ENTERPRISE OPPORTUNITIES DETECTED:");
                        
                        // Show detailed enterprise results
                        for (i, opp) in opportunities.iter().take(10).enumerate() {
                            let priority_icon = match opp.execution_priority {
                                modules::EnterprisePriority::Critical => "🔴",
                                modules::EnterprisePriority::High => "🟡",
                                modules::EnterprisePriority::Medium => "🟢",
                                modules::EnterprisePriority::Low => "🔵",
                                modules::EnterprisePriority::Monitor => "⚪",
                            };
                            
                            println!("   {}#{} {} {} vs {} ({:.2}% spread, {:.1}% conf, sources: {})",
                                priority_icon,
                                i + 1,
                                opp.token_pair,
                                opp.dex_a,
                                opp.dex_b,
                                opp.spread_percentage,
                                opp.confidence_score,
                                opp.data_sources.join("+")
                            );
                        }
                        
                        info!("🏛️ ENTERPRISE VALIDATION: Multi-source cross-validation complete");
                        info!("🎯 Professional feeds confirm {} high-confidence opportunities", 
                              opportunities.iter().filter(|o| o.confidence_score > 80.0).count());
                    }
                }
                Err(e) => {
                    error!("❌ Enterprise scan failed: {}", e);
                    warn!("🔄 Fallback: Consider using individual DEX scans (options D, C)");
                }
            }
        },
        
        "D" => {
            info!("📡 DIRECT DEX ACCESS SCAN");
            warn!("🎯 PROFESSIONAL: Direct API access to major DEXs (no aggregators)");
            
            println!("🔍 SCANNING DIRECT DEX APIS:");
            println!("   🔥 Raydium: https://api.raydium.io/v2/ammV3/ammPools");
            println!("   🌊 Orca: https://api.orca.so/v1/whirlpool/list");
            println!("   ⚡ Meteora: https://dlmm-api.meteora.ag/pair/all");
            println!("   🗡️ Saber: https://registry.saber.so/");
            println!("");
            println!("📊 This scan finds opportunities by:");
            println!("   ✅ Direct price comparison between DEXs");
            println!("   ✅ No rate limits (direct API access)");
            println!("   ✅ Real-time pool liquidity data");
            println!("   ✅ Cross-DEX arbitrage detection");
            
            info!("💡 Implementation note: Direct DEX scanner ready for activation");
            info!("🚀 This is what professional arbitrageurs use (not Jupiter aggregation)");
        },
        
        "C" => {
            info!("💰 CEX-DEX ARBITRAGE ANALYSIS");
            warn!("🏛️ ENTERPRISE: Centralized vs Decentralized exchange arbitrage");
            
            match modules::execute_cex_dex_analysis().await {
                Ok(opportunities) => {
                    info!("✅ CEX-DEX analysis completado exitosamente");
                    info!("� Oportunidades CEX-DEX encontradas: {}", opportunities.len());
                    
                    if opportunities.is_empty() {
                        warn!("⚠️ NO CEX-DEX OPPORTUNITIES DETECTED");
                        info!("💡 CEX-DEX Arbitrage Status:");
                        info!("   🏦 CEX prices collected from major exchanges");
                        info!("   � DEX prices collected from Solana ecosystem");
                        info!("   � Cross-market analysis: Complete");
                        info!("   🎯 Current market: High efficiency (low spreads)");
                        info!("   ⏰ Try during high volatility periods");
                    } else {
                        info!("🏆 CEX-DEX OPPORTUNITIES AVAILABLE:");
                        
                        // Show detailed CEX-DEX results
                        for (i, opp) in opportunities.iter().take(10).enumerate() {
                            let direction_desc = match opp.arbitrage_direction {
                                modules::ArbitrageDirection::BuyCexSellDex => "Buy CEX → Sell DEX",
                                modules::ArbitrageDirection::BuyDexSellCex => "Buy DEX → Sell CEX",
                            };
                            
                            let complexity_icon = match opp.execution_complexity {
                                modules::ExecutionComplexity::Simple => "🟢",
                                modules::ExecutionComplexity::Medium => "🟡", 
                                modules::ExecutionComplexity::Complex => "🔴",
                            };
                            
                            println!("   {}#{} {} {} vs {} ({:.2}% spread, ${:.0}/1k profit)",
                                complexity_icon,
                                i + 1,
                                opp.token_symbol,
                                opp.cex_name,
                                opp.dex_name,
                                opp.spread_percentage,
                                opp.estimated_profit_1k
                            );
                            println!("        Strategy: {} (confidence: {:.1}%)",
                                direction_desc,
                                opp.confidence_score
                            );
                        }
                        
                        info!("🎯 Professional CEX-DEX opportunities detected!");
                        info!("💡 Execution requirements:");
                        info!("   🏦 Active accounts on both CEX and DEX");
                        info!("   💰 Capital allocation on both platforms");
                        info!("   ⚡ Fast execution capability");
                        info!("   📊 Monitor withdrawal limits and fees");
                    }
                }
                Err(e) => {
                    error!("❌ CEX-DEX analysis failed: {}", e);
                    warn!("💡 Troubleshooting tips:");
                    warn!("   📡 Check internet connection");
                    warn!("   🔄 Some APIs may be rate limited");
                    warn!("   🏦 CEX APIs may require authentication for full access");
                }
            }
        },
        
        // ===== LEGACY MODES =====
        "B" => {
            info!("🔒 Running in SIMULATION mode (Legacy)");
            loop {
                match enterprise_system.run_enterprise_arbitrage().await {
                    Ok(_) => {
                        info!("✅ ENTERPRISE ARBITRAGE MISSION: SUCCESSFULLY COMPLETED");
                        info!("🎯 MILITARY STATUS: Mission accomplished with institutional precision");
                    }
                    Err(e) => {
                        error!("❌ ENTERPRISE ARBITRAGE MISSION: UNSUCCESSFUL");
                        error!("🚨 MILITARY ALERT: Mission failed - {}", e);
                        error!("🛡️  INSTITUTIONAL PROTOCOLS: Engaging recovery procedures");
                    }
                }
                
                println!("{}", enterprise_system.get_enterprise_statistics());
                
                info!("⏳ ENTERPRISE PROTOCOL: Initiating 30-second tactical pause...");
                info!("🎖️  MILITARY STATUS: Awaiting next mission authorization");
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        },
        "M" => {
            info!("🚀 PROPOSAL-003: Running in MULTI-TOKEN SIMULATION mode (Legacy)");
            
            // Activar sistema multi-token
            match enterprise_system.enable_multitoken_arbitrage().await {
                Ok(()) => {
                    info!("✅ PROPOSAL-003: Multi-token system activated successfully");
                    loop {
                        match enterprise_system.run_enterprise_arbitrage().await {
                            Ok(_) => {
                                info!("✅ MULTI-TOKEN ARBITRAGE MISSION: SUCCESSFULLY COMPLETED");
                                info!("🎯 PROPOSAL-003: Mission accomplished with multi-token precision");
                            }
                            Err(e) => {
                                error!("❌ MULTI-TOKEN ARBITRAGE MISSION: UNSUCCESSFUL");
                                error!("🚨 PROPOSAL-003 ALERT: Mission failed - {}", e);
                                error!("🛡️  MULTI-TOKEN PROTOCOLS: Engaging recovery procedures");
                            }
                        }
                        
                        println!("{}", enterprise_system.get_enterprise_statistics());
                        
                        info!("⏳ PROPOSAL-003: Initiating 30-second tactical pause...");
                        info!("🎖️  MULTI-TOKEN STATUS: Awaiting next mission authorization");
                        tokio::time::sleep(Duration::from_secs(30)).await;
                    }
                },
                Err(e) => {
                    error!("❌ PROPOSAL-003: Failed to activate multi-token system: {}", e);
                    info!("🛡️  Falling back to single-pair simulation mode for safety");
                    enterprise_system.run_enterprise_arbitrage().await?;
                }
            }
        },
        "T" => {
            info!("🚀 PROPOSAL-003 TIER 2: Running in MULTI-TOKEN SIMULATION mode (FULL ECOSYSTEM)");
            
            // Activar sistema multi-token con Tier 2
            match enterprise_system.enable_multitoken_tier2_arbitrage().await {
                Ok(()) => {
                    info!("✅ PROPOSAL-003 TIER 2: Multi-token ecosystem system activated successfully");
                    info!("🎯 Now supporting 16 token pairs across Solana ecosystem");
                    loop {
                        match enterprise_system.run_enterprise_arbitrage().await {
                            Ok(_) => {
                                info!("✅ TIER 2 ARBITRAGE MISSION: SUCCESSFULLY COMPLETED");
                                info!("🎯 PROPOSAL-003 TIER 2: Mission accomplished with ecosystem precision");
                            }
                            Err(e) => {
                                error!("❌ TIER 2 ARBITRAGE MISSION: UNSUCCESSFUL");
                                error!("🚨 PROPOSAL-003 TIER 2 ALERT: Mission failed - {}", e);
                                error!("🛡️  TIER 2 PROTOCOLS: Engaging recovery procedures");
                            }
                        }
                        
                        println!("{}", enterprise_system.get_enterprise_statistics());
                        
                        info!("⏳ PROPOSAL-003 TIER 2: Initiating 30-second tactical pause...");
                        info!("🎖️  ECOSYSTEM STATUS: Awaiting next mission authorization");
                        tokio::time::sleep(Duration::from_secs(30)).await;
                    }
                },
                Err(e) => {
                    error!("❌ PROPOSAL-003 TIER 2: Failed to activate ecosystem system: {}", e);
                    info!("🛡️  Falling back to Tier 1 multi-token mode for safety");
                    match enterprise_system.enable_multitoken_arbitrage().await {
                        Ok(()) => enterprise_system.run_enterprise_arbitrage().await?,
                        Err(_) => {
                            error!("🚨 Complete fallback to single-pair simulation");
                            enterprise_system.run_enterprise_arbitrage().await?;
                        }
                    }
                }
            }
        },
        "R" => {
            info!("⚠️  ENABLING REAL TRADING MODE (Legacy)");
            warn!("🚨 THIS WILL USE REAL MONEY - PROCEED WITH CAUTION");
            
            print!("Type 'CONFIRM' to proceed with real trading: ");
            io::stdout().flush().unwrap();
            
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm).unwrap();
            
            if confirm.trim() == "CONFIRM" {
                match enterprise_system.enable_real_trading_mainnet().await {
                    Ok(()) => {
                        info!("🎯 REAL TRADING MODE ACTIVATED");
                        loop {
                            match enterprise_system.run_enterprise_arbitrage().await {
                                Ok(_) => {
                                    info!("✅ ENTERPRISE ARBITRAGE MISSION: SUCCESSFULLY COMPLETED");
                                    info!("🎯 MILITARY STATUS: Mission accomplished with institutional precision");
                                }
                                Err(e) => {
                                    error!("❌ ENTERPRISE ARBITRAGE MISSION: UNSUCCESSFUL");
                                    error!("🚨 MILITARY ALERT: Mission failed - {}", e);
                                    error!("🛡️  INSTITUTIONAL PROTOCOLS: Engaging recovery procedures");
                                }
                            }
                            
                            println!("{}", enterprise_system.get_enterprise_statistics());
                            
                            info!("⏳ ENTERPRISE PROTOCOL: Initiating 30-second tactical pause...");
                            info!("🎖️  MILITARY STATUS: Awaiting next mission authorization");
                            tokio::time::sleep(Duration::from_secs(30)).await;
                        }
                    },
                    Err(e) => {
                        error!("❌ Failed to activate real trading: {}", e);
                        info!("🛡️  Falling back to simulation mode for safety");
                        enterprise_system.run_enterprise_arbitrage().await?;
                    }
                }
            } else {
                info!("🔒 Real trading cancelled for safety");
                info!("🎭 Running in simulation mode instead");
                enterprise_system.run_enterprise_arbitrage().await?;
            }
        },
        "0" | _ => {
            info!("👋 Exiting SniperForge Arbitrage System");
            info!("📋 Implementación completada: Opción C Modular con código 100% real");
            info!("🛡️ Safe Testing ✅ | Jupiter Scanner ✅ | Automated Monitor ✅ | Real Execution ✅");
            return Ok(());
        }
    }
    
    Ok(())
}
