pub mod real_analyzer;

/// Real Sentiment Analysis Module
/// Provides comprehensive sentiment analysis with REAL data sources
pub mod twitter_client; // ✅ NEW: Twitter API integration

pub use real_analyzer::*;
pub use twitter_client::*; // ✅ Export Twitter client
