use super::types::{SentimentDataSource, SentimentScore, Mention, SentimentError};
use async_trait::async_trait;
use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Twitter API v2 integration for real sentiment analysis
pub struct TwitterSentimentSource {
    client: Client,
    bearer_token: String,
    weight: f64,
    rate_limit_tracker: RateLimitTracker,
}

#[derive(Debug, Deserialize)]
struct TwitterApiResponse {
    data: Option<Vec<Tweet>>,
    meta: Option<TwitterMeta>,
}

#[derive(Debug, Deserialize)]
struct Tweet {
    id: String,
    text: String,
    created_at: String,
    author_id: String,
    public_metrics: Option<TweetMetrics>,
    context_annotations: Option<Vec<ContextAnnotation>>,
}

#[derive(Debug, Deserialize)]
struct TweetMetrics {
    retweet_count: u32,
    like_count: u32,
    reply_count: u32,
    quote_count: u32,
}

#[derive(Debug, Deserialize)]
struct ContextAnnotation {
    domain: ContextDomain,
    entity: ContextEntity,
}

#[derive(Debug, Deserialize)]
struct ContextDomain {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct ContextEntity {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct TwitterMeta {
    result_count: u32,
    newest_id: Option<String>,
    oldest_id: Option<String>,
}

#[derive(Debug)]
struct RateLimitTracker {
    requests_made: u32,
    window_start: DateTime<Utc>,
    requests_per_15min: u32,
}

impl RateLimitTracker {
    fn new() -> Self {
        Self {
            requests_made: 0,
            window_start: Utc::now(),
            requests_per_15min: 300, // Twitter API v2 limit
        }
    }
    
    fn can_make_request(&mut self) -> bool {
        let now = Utc::now();
        
        // Reset counter if 15 minutes have passed
        if now - self.window_start > Duration::minutes(15) {
            self.requests_made = 0;
            self.window_start = now;
        }
        
        self.requests_made < self.requests_per_15min
    }
    
    fn record_request(&mut self) {
        self.requests_made += 1;
    }
}

impl TwitterSentimentSource {
    pub fn new(bearer_token: String) -> Self {
        Self {
            client: Client::new(),
            bearer_token,
            weight: 0.4, // 40% weight in sentiment calculation
            rate_limit_tracker: RateLimitTracker::new(),
        }
    }
    
    async fn search_tweets(&mut self, query: &str, max_results: u32, timeframe_minutes: u64) -> Result<Vec<Tweet>> {
        if !self.rate_limit_tracker.can_make_request() {
            return Err(SentimentError::RateLimitExceeded { 
                source: "Twitter".to_string() 
            }.into());
        }
        
        let since_time = Utc::now() - Duration::minutes(timeframe_minutes as i64);
        let since_iso = since_time.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        
        let url = "https://api.twitter.com/2/tweets/search/recent";
        let params = [
            ("query", format!("{} -is:retweet lang:en", query)),
            ("max_results", max_results.to_string()),
            ("start_time", since_iso),
            ("tweet.fields", "created_at,author_id,public_metrics,context_annotations".to_string()),
        ];
        
        let response = self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.bearer_token))
            .query(&params)
            .send()
            .await?;
        
        self.rate_limit_tracker.record_request();
        
        if !response.status().is_success() {
            if response.status().as_u16() == 401 {
                return Err(SentimentError::AuthenticationFailed { 
                    source: "Twitter".to_string() 
                }.into());
            }
            return Err(SentimentError::NetworkError(
                reqwest::Error::from(response.error_for_status().unwrap_err())
            ).into());
        }
        
        let api_response: TwitterApiResponse = response.json().await?;
        Ok(api_response.data.unwrap_or_default())
    }
    
    fn analyze_tweet_sentiment(&self, tweet: &Tweet) -> f64 {
        let text = &tweet.text.to_lowercase();
        
        // Positive crypto keywords
        let positive_keywords = [
            "moon", "pump", "bullish", "buy", "hodl", "diamond hands", "to the moon",
            "breakthrough", "surge", "rally", "green", "profit", "gains", "up",
            "breakout", "explosion", "rocket", "skyrocket", "bull run", "accumulate",
            "strong", "solid", "massive", "huge", "incredible", "amazing", "fantastic"
        ];
        
        // Negative crypto keywords
        let negative_keywords = [
            "dump", "crash", "bear", "sell", "panic", "fear", "red", "loss",
            "down", "drop", "fall", "collapse", "disaster", "terrible", "awful",
            "scam", "rug pull", "dead", "rekt", "liquidated", "bearish", "weak",
            "disappointed", "concerned", "worried", "scared", "dip", "bleeding"
        ];
        
        let mut sentiment_score = 0.0;
        let mut word_count = 0;
        
        // Count positive keywords
        for keyword in &positive_keywords {
            if text.contains(keyword) {
                sentiment_score += 1.0;
                word_count += 1;
            }
        }
        
        // Count negative keywords
        for keyword in &negative_keywords {
            if text.contains(keyword) {
                sentiment_score -= 1.0;
                word_count += 1;
            }
        }
        
        // Adjust for engagement metrics (higher engagement = higher confidence)
        if let Some(metrics) = &tweet.public_metrics {
            let engagement = metrics.like_count + metrics.retweet_count + metrics.reply_count;
            let engagement_multiplier = 1.0 + (engagement as f64 / 100.0).min(2.0);
            sentiment_score *= engagement_multiplier;
        }
        
        // Normalize to -1.0 to 1.0 range
        if word_count > 0 {
            sentiment_score / word_count as f64
        } else {
            0.0 // Neutral if no sentiment keywords found
        }.max(-1.0).min(1.0)
    }
    
    fn calculate_confidence(&self, tweets: &[Tweet]) -> f64 {
        if tweets.is_empty() {
            return 0.0;
        }
        
        let total_engagement: u32 = tweets
            .iter()
            .filter_map(|t| t.public_metrics.as_ref())
            .map(|m| m.like_count + m.retweet_count + m.reply_count)
            .sum();
        
        let avg_engagement = total_engagement as f64 / tweets.len() as f64;
        
        // Higher confidence with more tweets and higher engagement
        let volume_factor = (tweets.len() as f64 / 50.0).min(1.0);
        let engagement_factor = (avg_engagement / 10.0).min(1.0);
        
        (volume_factor * 0.7 + engagement_factor * 0.3).min(1.0)
    }
}

#[async_trait]
impl SentimentDataSource for TwitterSentimentSource {
    async fn get_sentiment(&self, symbol: &str, timeframe: u64) -> Result<SentimentScore> {
        // Create search queries for the symbol
        let queries = [
            format!("${}", symbol),      // $SOL, $BTC, etc.
            format!("{} crypto", symbol), // SOL crypto, BTC crypto
            symbol.to_string(),          // Direct symbol
        ];
        
        let mut all_tweets = Vec::new();
        let mut source = self.clone(); // Clone for mutability
        
        // Search with multiple query variations
        for query in &queries {
            if let Ok(tweets) = source.search_tweets(query, 50, timeframe).await {
                all_tweets.extend(tweets);
            }
        }
        
        if all_tweets.is_empty() {
            return Err(SentimentError::InsufficientData {
                message: format!("No Twitter mentions found for {}", symbol)
            }.into());
        }
        
        // Calculate average sentiment
        let sentiment_sum: f64 = all_tweets
            .iter()
            .map(|tweet| source.analyze_tweet_sentiment(tweet))
            .sum();
        
        let average_sentiment = sentiment_sum / all_tweets.len() as f64;
        let confidence = source.calculate_confidence(&all_tweets);
        
        Ok(SentimentScore {
            sentiment: average_sentiment,
            confidence,
            mentions_count: all_tweets.len() as u32,
            timestamp: Utc::now(),
            source: "Twitter".to_string(),
        })
    }
    
    async fn get_mentions(&self, symbol: &str, timeframe: u64) -> Result<Vec<Mention>> {
        let mut source = self.clone();
        let query = format!("${} OR {} crypto", symbol, symbol);
        let tweets = source.search_tweets(&query, 100, timeframe).await?;
        
        let mentions = tweets
            .into_iter()
            .map(|tweet| {
                let sentiment = source.analyze_tweet_sentiment(&tweet);
                let engagement = tweet.public_metrics
                    .map(|m| m.like_count + m.retweet_count + m.reply_count)
                    .unwrap_or(0);
                
                Mention {
                    id: tweet.id,
                    content: tweet.text,
                    sentiment,
                    author: tweet.author_id,
                    timestamp: tweet.created_at.parse::<DateTime<Utc>>()
                        .unwrap_or_else(|_| Utc::now()),
                    source: "Twitter".to_string(),
                    engagement,
                }
            })
            .collect();
        
        Ok(mentions)
    }
    
    fn get_source_name(&self) -> &'static str {
        "Twitter"
    }
    
    fn get_weight(&self) -> f64 {
        self.weight
    }
}

// Clone implementation for TwitterSentimentSource
impl Clone for TwitterSentimentSource {
    fn clone(&self) -> Self {
        Self {
            client: Client::new(),
            bearer_token: self.bearer_token.clone(),
            weight: self.weight,
            rate_limit_tracker: RateLimitTracker::new(),
        }
    }
}
