use anyhow::Result;
use std::fmt;

/// Enterprise-grade error types for SniperForge arbitrage system
/// Provides structured error handling with proper categorization and context

#[derive(Debug, Clone)]
pub enum SniperForgeError {
    /// Configuration related errors
    Configuration(String),

    /// Network and connectivity errors
    Network { source: String, message: String },

    /// Trading and arbitrage specific errors
    Trading { operation: String, reason: String },

    /// Price feed and market data errors
    PriceFeed { source: String, message: String },

    /// Wallet and transaction errors
    Wallet(String),

    /// Security and validation errors
    Security(String),

    /// Rate limiting and API quota errors
    RateLimit { service: String, message: String },

    /// Data parsing and serialization errors
    DataParsing(String),

    /// Internal system errors
    Internal(String),

    /// Resource availability errors
    Resource { resource: String, status: String },
}

impl fmt::Display for SniperForgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            Self::Network { source, message } => write!(f, "Network error from {}: {}", source, message),
            Self::Trading { operation, reason } => write!(f, "Trading operation '{}' failed: {}", operation, reason),
            Self::PriceFeed { source, message } => write!(f, "Price feed error from {}: {}", source, message),
            Self::Wallet(msg) => write!(f, "Wallet error: {}", msg),
            Self::Security(msg) => write!(f, "Security error: {}", msg),
            Self::RateLimit { service, message } => write!(f, "Rate limit exceeded for {}: {}", service, message),
            Self::DataParsing(msg) => write!(f, "Data parsing error: {}", msg),
            Self::Internal(msg) => write!(f, "Internal system error: {}", msg),
            Self::Resource { resource, status } => write!(f, "Resource '{}' is {}", resource, status),
        }
    }
}

impl std::error::Error for SniperForgeError {}

impl SniperForgeError {
    /// Create a configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Configuration(message.into())
    }

    /// Create a network error
    pub fn network<S: Into<String>>(source: S, message: S) -> Self {
        Self::Network {
            source: source.into(),
            message: message.into(),
        }
    }

    /// Create a trading error
    pub fn trading<S: Into<String>>(operation: S, reason: S) -> Self {
        Self::Trading {
            operation: operation.into(),
            reason: reason.into(),
        }
    }

    /// Create a price feed error
    pub fn price_feed<S: Into<String>>(source: S, message: S) -> Self {
        Self::PriceFeed {
            source: source.into(),
            message: message.into(),
        }
    }

    /// Create a wallet error
    pub fn wallet<S: Into<String>>(message: S) -> Self {
        Self::Wallet(message.into())
    }

    /// Create a security error
    pub fn security<S: Into<String>>(message: S) -> Self {
        Self::Security(message.into())
    }

    /// Create a rate limit error
    pub fn rate_limit<S: Into<String>>(service: S, message: S) -> Self {
        Self::RateLimit {
            service: service.into(),
            message: message.into(),
        }
    }

    /// Create a data parsing error
    pub fn data_parsing<S: Into<String>>(message: S) -> Self {
        Self::DataParsing(message.into())
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal(message.into())
    }

    /// Create a resource error
    pub fn resource<S: Into<String>>(resource: S, status: S) -> Self {
        Self::Resource {
            resource: resource.into(),
            status: status.into(),
        }
    }

    /// Get error category for logging and metrics
    pub fn category(&self) -> &'static str {
        match self {
            Self::Configuration(..) => "configuration",
            Self::Network { .. } => "network",
            Self::Trading { .. } => "trading",
            Self::PriceFeed { .. } => "price_feed",
            Self::Wallet(..) => "wallet",
            Self::Security(..) => "security",
            Self::RateLimit { .. } => "rate_limit",
            Self::DataParsing(..) => "data_parsing",
            Self::Internal(..) => "internal",
            Self::Resource { .. } => "resource",
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Network { .. } => true,
            Self::RateLimit { .. } => true,
            Self::Resource { .. } => true,
            Self::PriceFeed { .. } => true,
            Self::Configuration(..) => false,
            Self::Security(..) => false,
            Self::Wallet(..) => false,
            Self::Trading { .. } => false, // Depends on specific case
            Self::DataParsing(..) => false,
            Self::Internal(..) => false,
        }
    }

    /// Get retry delay for recoverable errors (in milliseconds)
    pub fn retry_delay_ms(&self) -> Option<u64> {
        if !self.is_recoverable() {
            return None;
        }

        match self {
            Self::Network { .. } => Some(1000),
            Self::RateLimit { .. } => Some(5000),
            Self::Resource { .. } => Some(2000),
            Self::PriceFeed { .. } => Some(500),
            _ => None,
        }
    }
}

/// Result type alias for SniperForge operations
pub type SniperResult<T> = Result<T, SniperForgeError>;

/// Error extension trait for adding context to standard errors
pub trait ErrorExt<T> {
    fn with_config_context(self, context: &str) -> SniperResult<T>;
    fn with_network_context(self, source: &str, context: &str) -> SniperResult<T>;
    fn with_trading_context(self, operation: &str, context: &str) -> SniperResult<T>;
}

impl<T, E> ErrorExt<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_config_context(self, context: &str) -> SniperResult<T> {
        self.map_err(|e| SniperForgeError::config(format!("{}: {}", context, e)))
    }

    fn with_network_context(self, source: &str, context: &str) -> SniperResult<T> {
        self.map_err(|e| SniperForgeError::network(source.to_string(), format!("{}: {}", context, e)))
    }

    fn with_trading_context(self, operation: &str, context: &str) -> SniperResult<T> {
        self.map_err(|e| SniperForgeError::trading(operation.to_string(), format!("{}: {}", context, e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = SniperForgeError::config("Invalid RPC URL");
        assert_eq!(error.category(), "configuration");
        assert!(!error.is_recoverable());
    }

    #[test]
    fn test_recoverable_errors() {
        let network_error = SniperForgeError::network("Solana RPC", "Connection timeout");
        assert!(network_error.is_recoverable());
        assert_eq!(network_error.retry_delay_ms(), Some(1000));
    }

    #[test]
    fn test_error_context() {
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found"
        ));
        
        let with_context = result.with_config_context("Loading wallet file");
        assert!(with_context.is_err());
    }
}
