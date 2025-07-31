use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait for sentiment data sources
#[async_trait]
pub trait SentimentDataSource: Send + Sync {
    async fn get_sentiment(&self, symbol: &str, timeframe: u64) -> Result<SentimentScore>;
    async fn get_mentions(&self, symbol: &str, timeframe: u64) -> Result<Vec<Mention>>;
    fn get_source_name(&self) -> &'static str;
    fn get_weight(&self) -> f64;
}

/// Sentiment score from a data source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentScore {
    pub sentiment: f64,      // -1.0 to 1.0 (negative to positive)
    pub confidence: f64,     // 0.0 to 1.0 (confidence in the score)
    pub mentions_count: u32, // Number of mentions analyzed
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

/// Individual mention/post/article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mention {
    pub id: String,
    pub content: String,
    pub sentiment: f64,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub engagement: u32, // likes, shares, comments, etc.
}

/// Aggregated sentiment analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedSentiment {
    pub overall_sentiment: f64,    // Weighted average sentiment
    pub confidence: f64,           // Overall confidence
    pub total_mentions: u32,       // Total mentions across all sources
    pub source_breakdown: HashMap<String, SentimentScore>,
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
}

/// Sentiment trend across multiple timeframes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentTrend {
    pub symbol: String,
    pub short_term: f64,   // 1 hour sentiment
    pub medium_term: f64,  // 6 hour sentiment
    pub long_term: f64,    // 24 hour sentiment
    pub momentum: f64,     // short_term - medium_term (positive = improving)
    pub timestamp: DateTime<Utc>,
}

/// Configuration for sentiment analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentConfig {
    pub twitter_enabled: bool,
    pub reddit_enabled: bool,
    pub news_enabled: bool,
    pub fear_greed_enabled: bool,
    
    // Weights for different sources (must sum to 1.0)
    pub twitter_weight: f64,    // Default: 0.4
    pub reddit_weight: f64,     // Default: 0.3
    pub news_weight: f64,       // Default: 0.2
    pub fear_greed_weight: f64, // Default: 0.1
    
    // Timeframes in minutes
    pub short_term_minutes: u64,  // Default: 60 (1 hour)
    pub medium_term_minutes: u64, // Default: 360 (6 hours)
    pub long_term_minutes: u64,   // Default: 1440 (24 hours)
    
    // Quality filters
    pub min_confidence: f64,      // Minimum confidence to include
    pub min_mentions: u32,        // Minimum mentions required
    pub bot_detection: bool,      // Enable bot filtering
    pub spam_detection: bool,     // Enable spam filtering
}

impl Default for SentimentConfig {
    fn default() -> Self {
        Self {
            twitter_enabled: true,
            reddit_enabled: true,
            news_enabled: true,
            fear_greed_enabled: true,
            
            twitter_weight: 0.4,
            reddit_weight: 0.3,
            news_weight: 0.2,
            fear_greed_weight: 0.1,
            
            short_term_minutes: 60,
            medium_term_minutes: 360,
            long_term_minutes: 1440,
            
            min_confidence: 0.5,
            min_mentions: 5,
            bot_detection: true,
            spam_detection: true,
        }
    }
}

/// Error types for sentiment analysis
#[derive(Debug, thiserror::Error)]
pub enum SentimentError {
    #[error("API rate limit exceeded for source: {source}")]
    RateLimitExceeded { source: String },
    
    #[error("Authentication failed for source: {source}")]
    AuthenticationFailed { source: String },
    
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("Parsing error: {0}")]
    ParsingError(String),
    
    #[error("Insufficient data: {message}")]
    InsufficientData { message: String },
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Sentiment analysis engine that coordinates multiple data sources
pub struct SentimentEngine {
    sources: Vec<Box<dyn SentimentDataSource>>,
    config: SentimentConfig,
    cache: HashMap<String, (AggregatedSentiment, DateTime<Utc>)>,
    cache_duration_minutes: u64,
}

impl SentimentEngine {
    pub fn new(config: SentimentConfig) -> Self {
        Self {
            sources: Vec::new(),
            config,
            cache: HashMap::new(),
            cache_duration_minutes: 5, // Cache for 5 minutes
        }
    }
    
    pub fn add_source(&mut self, source: Box<dyn SentimentDataSource>) {
        self.sources.push(source);
    }
    
    /// Get aggregated sentiment for a symbol
    pub async fn get_sentiment(&mut self, symbol: &str) -> Result<AggregatedSentiment> {
        let cache_key = format!("{}_{}", symbol, self.config.short_term_minutes);
        
        // Check cache first
        if let Some((cached_sentiment, cached_time)) = self.cache.get(&cache_key) {
            let age_minutes = (Utc::now() - *cached_time).num_minutes() as u64;
            if age_minutes < self.cache_duration_minutes {
                return Ok(cached_sentiment.clone());
            }
        }
        
        let mut source_scores = HashMap::new();
        let mut total_weighted_sentiment = 0.0;
        let mut total_weight = 0.0;
        let mut total_mentions = 0;
        let mut overall_confidence = 0.0;
        
        // Collect sentiment from all enabled sources
        for source in &self.sources {
            if let Ok(score) = source.get_sentiment(symbol, self.config.short_term_minutes).await {
                let weight = source.get_weight();
                
                // Apply quality filters
                if score.confidence >= self.config.min_confidence 
                   && score.mentions_count >= self.config.min_mentions {
                    
                    total_weighted_sentiment += score.sentiment * weight;
                    total_weight += weight;
                    total_mentions += score.mentions_count;
                    overall_confidence += score.confidence * weight;
                    
                    source_scores.insert(source.get_source_name().to_string(), score);
                }
            }
        }
        
        if total_weight == 0.0 {
            return Err(SentimentError::InsufficientData {
                message: format!("No reliable sentiment data found for {}", symbol)
            }.into());
        }
        
        let aggregated = AggregatedSentiment {
            overall_sentiment: total_weighted_sentiment / total_weight,
            confidence: overall_confidence / total_weight,
            total_mentions,
            source_breakdown: source_scores,
            timestamp: Utc::now(),
            symbol: symbol.to_string(),
        };
        
        // Cache the result
        self.cache.insert(cache_key, (aggregated.clone(), Utc::now()));
        
        Ok(aggregated)
    }
    
    /// Get sentiment trend over multiple timeframes
    pub async fn get_sentiment_trend(&mut self, symbol: &str) -> Result<SentimentTrend> {
        let short_term = self.get_sentiment_for_timeframe(symbol, self.config.short_term_minutes).await?;
        let medium_term = self.get_sentiment_for_timeframe(symbol, self.config.medium_term_minutes).await?;
        let long_term = self.get_sentiment_for_timeframe(symbol, self.config.long_term_minutes).await?;
        
        Ok(SentimentTrend {
            symbol: symbol.to_string(),
            short_term: short_term.overall_sentiment,
            medium_term: medium_term.overall_sentiment,
            long_term: long_term.overall_sentiment,
            momentum: short_term.overall_sentiment - medium_term.overall_sentiment,
            timestamp: Utc::now(),
        })
    }
    
    async fn get_sentiment_for_timeframe(&mut self, symbol: &str, minutes: u64) -> Result<AggregatedSentiment> {
        let mut total_weighted_sentiment = 0.0;
        let mut total_weight = 0.0;
        let mut total_mentions = 0;
        let mut source_scores = HashMap::new();
        
        for source in &self.sources {
            if let Ok(score) = source.get_sentiment(symbol, minutes).await {
                let weight = source.get_weight();
                total_weighted_sentiment += score.sentiment * weight;
                total_weight += weight;
                total_mentions += score.mentions_count;
                source_scores.insert(source.get_source_name().to_string(), score);
            }
        }
        
        Ok(AggregatedSentiment {
            overall_sentiment: if total_weight > 0.0 { total_weighted_sentiment / total_weight } else { 0.0 },
            confidence: if total_weight > 0.0 { total_weight } else { 0.0 },
            total_mentions,
            source_breakdown: source_scores,
            timestamp: Utc::now(),
            symbol: symbol.to_string(),
        })
    }
}
