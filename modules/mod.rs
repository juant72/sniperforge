// ===== MODULES DECLARATION =====
// Declaración de módulos para el sistema modular de arbitrage

pub mod safe_testing;
pub mod jupiter_scanner;
pub mod automated_monitor;
pub mod real_execution;

// Re-export main types for easy access
pub use safe_testing::{SafeTester, SafeTestResult, RiskLevel, execute_safe_arbitrage_test};
pub use jupiter_scanner::{JupiterScanner, OpportunityResult, Priority, execute_comprehensive_scan, execute_quick_scan};
pub use automated_monitor::{AutomatedMonitor, MonitorConfig, MonitorAlert, AlertType, MonitoringStatus, start_automated_monitoring, start_automated_monitoring_with_config};
pub use real_execution::{RealExecutor, ExecutionResult, ExecutionConfig, create_mainnet_executor, create_devnet_executor, execute_safe_arbitrage, simulate_arbitrage_execution};
