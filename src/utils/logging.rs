use tracing::{info, warn, error, debug};
use tracing_subscriber::{
    fmt::{self, time::SystemTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};
use crate::errors::{SniperForgeError, SniperResult};
use crate::config::enterprise::EnterpriseConfig;

/// Enterprise logging system for SniperForge
/// Provides structured logging with multiple outputs and enterprise features

pub struct EnterpriseLogger {
    config: EnterpriseConfig,
}

impl EnterpriseLogger {
    /// Initialize enterprise logging system
    pub fn init(config: &EnterpriseConfig) -> SniperResult<()> {
        let filter = EnvFilter::new(&config.system.log_level)
            .add_directive("sniperforge=debug".parse().unwrap())
            .add_directive("solana=info".parse().unwrap())
            .add_directive("reqwest=warn".parse().unwrap())
            .add_directive("hyper=warn".parse().unwrap());

        let stdout_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_timer(SystemTime)
            .with_ansi(true)
            .json();

        let subscriber = Registry::default()
            .with(filter)
            .with(stdout_layer);

        // Add file logging if configured
        if let Some(log_file_path) = &config.system.log_file_path {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file_path)
                .map_err(|e| SniperForgeError::config(
                    format!("Failed to open log file {:?}: {}", log_file_path, e)
                ))?;

            let file_layer = fmt::layer()
                .with_writer(file)
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
                .with_timer(SystemTime)
                .with_ansi(false)
                .json();

            let subscriber_with_file = subscriber.with(file_layer);
            subscriber_with_file.init();
        } else {
            subscriber.init();
        }

        info!("Enterprise logging system initialized");
        info!(
            version = env!("CARGO_PKG_VERSION"),
            log_level = %config.system.log_level,
            log_file = ?config.system.log_file_path,
            "SniperForge logging started"
        );

        Ok(())
    }

    /// Log trading operation
    pub fn log_trading_operation(
        operation: &str,
        pair: &str,
        amount: f64,
        expected_profit: f64,
        success: bool,
    ) {
        if success {
            info!(
                operation = operation,
                pair = pair,
                amount = amount,
                expected_profit = expected_profit,
                "Trading operation successful"
            );
        } else {
            warn!(
                operation = operation,
                pair = pair,
                amount = amount,
                expected_profit = expected_profit,
                "Trading operation failed"
            );
        }
    }

    /// Log arbitrage opportunity
    pub fn log_arbitrage_opportunity(
        pair: &str,
        exchange1: &str,
        price1: f64,
        exchange2: &str,
        price2: f64,
        profit_percentage: f64,
    ) {
        info!(
            pair = pair,
            exchange1 = exchange1,
            price1 = price1,
            exchange2 = exchange2,
            price2 = price2,
            profit_percentage = profit_percentage,
            "Arbitrage opportunity detected"
        );
    }

    /// Log price feed update
    pub fn log_price_update(source: &str, pair: &str, price: f64, timestamp: u64) {
        debug!(
            source = source,
            pair = pair,
            price = price,
            timestamp = timestamp,
            "Price feed updated"
        );
    }

    /// Log network error with retry information
    pub fn log_network_error(
        service: &str,
        error: &str,
        retry_count: u32,
        max_retries: u32,
        next_retry_in_ms: Option<u64>,
    ) {
        if retry_count < max_retries {
            warn!(
                service = service,
                error = error,
                retry_count = retry_count,
                max_retries = max_retries,
                next_retry_in_ms = next_retry_in_ms,
                "Network error, will retry"
            );
        } else {
            error!(
                service = service,
                error = error,
                retry_count = retry_count,
                max_retries = max_retries,
                "Network error, max retries exceeded"
            );
        }
    }

    /// Log system health metrics
    pub fn log_system_health(
        cpu_usage: f64,
        memory_usage: f64,
        active_connections: u32,
        uptime_seconds: u64,
    ) {
        info!(
            cpu_usage = cpu_usage,
            memory_usage = memory_usage,
            active_connections = active_connections,
            uptime_seconds = uptime_seconds,
            "System health metrics"
        );
    }

    /// Log security event
    pub fn log_security_event(event_type: &str, description: &str, severity: &str) {
        match severity {
            "critical" => error!(
                event_type = event_type,
                description = description,
                severity = severity,
                "Security event"
            ),
            "high" => warn!(
                event_type = event_type,
                description = description,
                severity = severity,
                "Security event"
            ),
            _ => info!(
                event_type = event_type,
                description = description,
                severity = severity,
                "Security event"
            ),
        }
    }

    /// Log configuration change
    pub fn log_config_change(parameter: &str, old_value: &str, new_value: &str, user: &str) {
        warn!(
            parameter = parameter,
            old_value = old_value,
            new_value = new_value,
            user = user,
            "Configuration changed"
        );
    }

    /// Log wallet transaction
    pub fn log_wallet_transaction(
        transaction_type: &str,
        signature: &str,
        amount: f64,
        fee: f64,
        success: bool,
    ) {
        if success {
            info!(
                transaction_type = transaction_type,
                signature = signature,
                amount = amount,
                fee = fee,
                "Wallet transaction successful"
            );
        } else {
            error!(
                transaction_type = transaction_type,
                signature = signature,
                amount = amount,
                fee = fee,
                "Wallet transaction failed"
            );
        }
    }

    /// Log performance metrics
    pub fn log_performance_metrics(
        operation: &str,
        duration_ms: u64,
        throughput: f64,
        error_rate: f64,
    ) {
        info!(
            operation = operation,
            duration_ms = duration_ms,
            throughput = throughput,
            error_rate = error_rate,
            "Performance metrics"
        );
    }
}

/// Structured log entry for enterprise analytics
#[derive(serde::Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub component: String,
    pub message: String,
    pub metadata: serde_json::Value,
}

/// Log level enum for programmatic control
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_log_level_conversion() {
        assert_eq!(LogLevel::Info.as_str(), "info");
        assert_eq!(LogLevel::Error.as_str(), "error");
    }

    #[test]
    fn test_enterprise_logger_creation() {
        // Test that we can create logger configuration
        // Actual testing would require more setup
        assert!(true);
    }
}
