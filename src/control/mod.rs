pub mod bot_controller;
pub mod tcp_server;

// Re-export main types
pub use bot_controller::{BotController, BotSummary, SystemMetrics};
pub use tcp_server::{TcpControlServer, TcpCommand, TcpResponse};
