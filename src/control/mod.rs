pub mod bot_controller;
pub mod tcp_server;
pub mod desired_state_reconciler;

// Re-export main types
pub use bot_controller::{BotController, BotSummary, SystemMetrics, SystemStateSummary, MassControlResult, SystemResourceStatus};
pub use tcp_server::{TcpControlServer, TcpCommand, TcpResponse};
pub use desired_state_reconciler::{
    DesiredStateReconciler, ReconciliationEvent, ReconciliationStats, 
    ReconciliationAction, ReconciliationResult, StateDriftAnalysis, 
    StatusMismatch, ConfigDrift
};
