//! Twitter API Integration for Real-Time Sentiment Analysis
//! Requires Twitter Developer Account credentials

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterCredentials {
    pub api_key: String,
    pub api_secret: String,
    pub bearer_token: String,
    pub access_token: String,
    pub access_token_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterSentimentData {
    pub symbol: String,
    pub tweet_count: u32,
    pub positive_tweets: u32,
    pub negative_tweets: u32,
    pub neutral_tweets: u32,
    pub sentiment_score: f64,
    pub trending_hashtags: Vec<String>,
    pub influencer_sentiment: HashMap<String, f64>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweetData {
    pub id: String,
    pub text: String,
    pub author_id: String,
    pub public_metrics: TweetMetrics,
    pub created_at: String,
    pub sentiment_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweetMetrics {
    pub retweet_count: u32,
    pub like_count: u32,
    pub reply_count: u32,
    pub quote_count: u32,
}

/// Twitter API client for real sentiment analysis
#[derive(Debug, Clone)]
pub struct TwitterSentimentClient {
    credentials: Option<TwitterCredentials>,
    client: reqwest::Client,
    rate_limit_remaining: u32,
    rate_limit_reset: DateTime<Utc>,
}

impl TwitterSentimentClient {
    pub fn new() -> Self {
        Self {
            credentials: None,
            client: reqwest::Client::new(),
            rate_limit_remaining: 0,
            rate_limit_reset: Utc::now(),
        }
    }

    /// Set Twitter API credentials from your developer account
    pub fn with_credentials(mut self, credentials: TwitterCredentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Load credentials from config file
    pub fn load_credentials_from_config(&mut self, config_path: &str) -> Result<()> {
        let config_content = std::fs::read_to_string(config_path)?;
        let credentials: TwitterCredentials = serde_json::from_str(&config_content)?;
        self.credentials = Some(credentials);
        Ok(())
    }

    /// Search recent tweets for crypto sentiment analysis
    pub async fn analyze_crypto_sentiment(&mut self, symbol: &str) -> Result<TwitterSentimentData> {
        if self.credentials.is_none() {
            return Err(anyhow::anyhow!("Twitter credentials not configured"));
        }

        // Check rate limits
        if self.rate_limit_remaining == 0 && Utc::now() < self.rate_limit_reset {
            return Err(anyhow::anyhow!("Twitter API rate limit exceeded"));
        }

        let search_queries = self.build_search_queries(symbol);
        let mut all_tweets = Vec::new();

        for query in &search_queries {
            match self.search_recent_tweets(query, 100).await {
                Ok(mut tweets) => all_tweets.append(&mut tweets),
                Err(e) => eprintln!("Failed to search tweets for {}: {}", query, e),
            }
        }

        Ok(self.calculate_sentiment_from_tweets(symbol, all_tweets))
    }

    /// Search recent tweets using Twitter API v2
    async fn search_recent_tweets(&mut self, query: &str, max_results: u32) -> Result<Vec<TweetData>> {
        let credentials = self.credentials.as_ref().unwrap();
        
        let url = "https://api.twitter.com/2/tweets/search/recent";
        let params = [
            ("query", query),
            ("max_results", &max_results.to_string()),
            ("tweet.fields", "created_at,public_metrics,author_id"),
            ("user.fields", "username,verified"),
        ];

        let response = self.client
            .get(url)
            .bearer_auth(&credentials.bearer_token)
            .query(&params)
            .send()
            .await?;

        // Update rate limit info from headers
        if let Some(remaining) = response.headers().get("x-rate-limit-remaining") {
            self.rate_limit_remaining = remaining.to_str()?.parse()?;
        }
        if let Some(reset) = response.headers().get("x-rate-limit-reset") {
            let reset_timestamp: i64 = reset.to_str()?.parse()?;
            self.rate_limit_reset = DateTime::from_timestamp(reset_timestamp, 0)
                .unwrap_or_else(Utc::now);
        }

        let json_response: serde_json::Value = response.json().await?;
        let mut tweets = Vec::new();

        if let Some(data) = json_response["data"].as_array() {
            for tweet in data {
                if let Ok(tweet_data) = self.parse_tweet_data(tweet) {
                    tweets.push(tweet_data);
                }
            }
        }

        Ok(tweets)
    }

    /// Build optimized search queries for different crypto symbols
    fn build_search_queries(&self, symbol: &str) -> Vec<String> {
        match symbol.to_uppercase().as_str() {
            "BTC" => vec![
                "#Bitcoin OR #BTC -is:retweet lang:en".to_string(),
                "$BTC OR \"Bitcoin\" -is:retweet lang:en".to_string(),
                "Bitcoin price OR BTC pump OR BTC dump -is:retweet lang:en".to_string(),
            ],
            "ETH" => vec![
                "#Ethereum OR #ETH -is:retweet lang:en".to_string(),
                "$ETH OR \"Ethereum\" -is:retweet lang:en".to_string(),
                "Ethereum price OR ETH pump OR ETH dump -is:retweet lang:en".to_string(),
            ],
            "SOL" => vec![
                "#Solana OR #SOL -is:retweet lang:en".to_string(),
                "$SOL OR \"Solana\" -is:retweet lang:en".to_string(),
                "Solana price OR SOL pump OR SOL dump -is:retweet lang:en".to_string(),
            ],
            _ => vec![
                format!("#{} OR ${} -is:retweet lang:en", symbol, symbol),
                format!("\"{}\" price -is:retweet lang:en", symbol),
            ],
        }
    }

    /// Parse tweet data from Twitter API response
    fn parse_tweet_data(&self, tweet: &serde_json::Value) -> Result<TweetData> {
        let id = tweet["id"].as_str().unwrap_or("").to_string();
        let text = tweet["text"].as_str().unwrap_or("").to_string();
        let author_id = tweet["author_id"].as_str().unwrap_or("").to_string();
        let created_at = tweet["created_at"].as_str().unwrap_or("").to_string();

        let public_metrics = TweetMetrics {
            retweet_count: tweet["public_metrics"]["retweet_count"].as_u64().unwrap_or(0) as u32,
            like_count: tweet["public_metrics"]["like_count"].as_u64().unwrap_or(0) as u32,
            reply_count: tweet["public_metrics"]["reply_count"].as_u64().unwrap_or(0) as u32,
            quote_count: tweet["public_metrics"]["quote_count"].as_u64().unwrap_or(0) as u32,
        };

        // Calculate sentiment score for this tweet
        let sentiment_score = self.calculate_tweet_sentiment(&text);

        Ok(TweetData {
            id,
            text,
            author_id,
            public_metrics,
            created_at,
            sentiment_score,
        })
    }

    /// Calculate sentiment from collected tweets
    fn calculate_sentiment_from_tweets(&self, symbol: &str, tweets: Vec<TweetData>) -> TwitterSentimentData {
        let mut positive_tweets = 0;
        let mut negative_tweets = 0;
        let mut neutral_tweets = 0;
        let mut total_sentiment = 0.0;
        let mut trending_hashtags = HashMap::new();

        for tweet in &tweets {
            // Weight by engagement (likes + retweets)
            let engagement_weight = 1.0 + ((tweet.public_metrics.like_count + tweet.public_metrics.retweet_count) as f64).log10().max(0.0);
            let weighted_sentiment = tweet.sentiment_score * engagement_weight;
            
            total_sentiment += weighted_sentiment;

            if tweet.sentiment_score > 0.1 {
                positive_tweets += 1;
            } else if tweet.sentiment_score < -0.1 {
                negative_tweets += 1;
            } else {
                neutral_tweets += 1;
            }

            // Extract hashtags
            for word in tweet.text.split_whitespace() {
                if word.starts_with('#') && word.len() > 1 {
                    *trending_hashtags.entry(word.to_lowercase()).or_insert(0) += 1;
                }
            }
        }

        // Get top trending hashtags
        let mut hashtag_vec: Vec<_> = trending_hashtags.into_iter().collect();
        hashtag_vec.sort_by(|a, b| b.1.cmp(&a.1));
        let trending_hashtags: Vec<String> = hashtag_vec.into_iter()
            .take(5)
            .map(|(hashtag, _)| hashtag)
            .collect();

        let sentiment_score = if tweets.is_empty() {
            0.0
        } else {
            total_sentiment / tweets.len() as f64
        };

        TwitterSentimentData {
            symbol: symbol.to_string(),
            tweet_count: tweets.len() as u32,
            positive_tweets,
            negative_tweets,
            neutral_tweets,
            sentiment_score,
            trending_hashtags,
            influencer_sentiment: HashMap::new(), // Would need influencer detection
            timestamp: Utc::now(),
        }
    }

    /// Calculate sentiment score for individual tweet
    fn calculate_tweet_sentiment(&self, text: &str) -> f64 {
        let text_lower = text.to_lowercase();
        let mut sentiment: f64 = 0.0;

        // Positive crypto keywords
        let positive_words = [
            "bullish", "moon", "pump", "buy", "long", "hodl", "up", "rise", "gain",
            "profit", "bull", "green", "rally", "breakout", "surge", "rocket",
            "diamond", "hands", "ath", "new high", "lambo", "to the moon",
            "breaking out", "golden cross", "support", "accumulate"
        ];

        // Negative crypto keywords
        let negative_words = [
            "bearish", "dump", "sell", "short", "down", "drop", "loss", "crash",
            "bear", "red", "correction", "decline", "fall", "panic", "fear",
            "fud", "scam", "rug", "hack", "regulation", "ban", "bubble",
            "death cross", "resistance", "overbought", "capitulation"
        ];

        // Count positive words
        for word in &positive_words {
            if text_lower.contains(word) {
                sentiment += 0.1;
            }
        }

        // Count negative words
        for word in &negative_words {
            if text_lower.contains(word) {
                sentiment -= 0.1;
            }
        }

        // Normalize sentiment
        sentiment.max(-1.0).min(1.0)
    }

    /// Get current rate limit status
    pub fn get_rate_limit_status(&self) -> (u32, DateTime<Utc>) {
        (self.rate_limit_remaining, self.rate_limit_reset)
    }

    /// Check if credentials are configured
    pub fn has_credentials(&self) -> bool {
        self.credentials.is_some()
    }
}

impl Default for TwitterSentimentClient {
    fn default() -> Self {
        Self::new()
    }
}
