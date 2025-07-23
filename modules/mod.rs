// ===== MODULES DECLARATION =====
// Declaración de módulos para el sistema modular de arbitrage

pub mod safe_testing;
pub mod jupiter_scanner;
pub mod automated_monitor;
pub mod real_execution;

// Re-export main types for easy access
pub use safe_testing::{SafeTester, RiskLevel, execute_safe_arbitrage_test};
pub use jupiter_scanner::{JupiterScanner, Priority, execute_comprehensive_scan, execute_quick_scan};
pub use automated_monitor::{AutomatedMonitor, MonitorConfig, start_automated_monitoring_with_config};
pub use real_execution::{RealExecutor, create_mainnet_executor, simulate_arbitrage_execution_advanced}; // Fixed function name
