// ===== MODULES DECLARATION =====
// Declaración de módulos para el sistema modular de arbitrage

pub mod safe_testing;
pub mod jupiter_scanner;
pub mod automated_monitor;
pub mod real_execution;
pub mod enterprise_multi_source;

// Re-export main types for easy access
pub use safe_testing::{RiskLevel, execute_safe_arbitrage_test};
pub use jupiter_scanner::{Priority, execute_comprehensive_scan, execute_quick_scan};
pub use automated_monitor::{MonitorConfig, start_automated_monitoring_with_config};
pub use real_execution::simulate_arbitrage_execution_advanced; // Export used functions
pub use enterprise_multi_source::{execute_enterprise_multi_source_scan, EnterpriseOpportunity, EnterprisePriority};
