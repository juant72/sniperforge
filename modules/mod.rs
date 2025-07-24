// ===== MODULES DECLARATION =====
// Declaración de módulos para el sistema modular de arbitrage

pub mod safe_testing;
pub mod jupiter_scanner;
pub mod automated_monitor;
pub mod real_execution;
pub mod enterprise_multi_source;
pub mod cex_dex_arbitrage;
pub mod enterprise_auto_scanner_real; // NEW: 100% REAL APIs NO SIMULATION

// ===== PHASE 1: EXPERT DeFi MODULES =====
pub mod jupiter_advanced;     // PHASE 1: Expert Jupiter auto-routing

// ===== PHASE 2: MEV PROTECTION =====
pub mod mev_protection;       // PHASE 2: Real MEV protection with Jito integration

// ===== PHASE 3: DEX SPECIALIZATION =====
pub mod dex_specialization;   // PHASE 3: DEX-specific strategies

// Re-export main types for easy access
pub use safe_testing::{RiskLevel, execute_safe_arbitrage_test};
pub use jupiter_scanner::{Priority, execute_comprehensive_scan, execute_quick_scan};
pub use automated_monitor::{MonitorConfig, start_automated_monitoring_with_config};
pub use real_execution::simulate_arbitrage_execution_advanced; // Export used functions
pub use enterprise_multi_source::{execute_enterprise_multi_source_scan, EnterprisePriority};
pub use cex_dex_arbitrage::{execute_cex_dex_analysis, ArbitrageDirection, ExecutionComplexity};
pub use enterprise_auto_scanner_real::start_real_enterprise_auto_scanner; // REAL function

// ===== PHASE 1 EXPERT EXPORTS =====
pub use jupiter_advanced::{JupiterAdvancedEngine, JupiterAdvancedConfig, JupiterAdvancedOpportunity};

// ===== PHASE 2 MEV PROTECTION EXPORTS =====
pub use mev_protection::{
    MEVProtectionEngine, MEVProtectionConfig, MEVProtectionStats,
    SandwichDetection, RiskLevel as MEVRiskLevel, RecommendedAction,
    BundleSubmissionResult, BundleStatus, NetworkCongestion
};

// ===== PHASE 3 DEX SPECIALIZATION EXPORTS =====
pub use dex_specialization::{
    DEXSpecializationEngine, DEXSpecializationConfig, SpecializedOpportunity,
    DEXType, StrategyType, DEXMetadata, DEXStats,
    create_default_dex_engine, create_custom_dex_engine
};
