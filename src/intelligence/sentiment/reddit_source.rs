use super::types::{SentimentDataSource, SentimentScore, Mention, SentimentError};
use async_trait::async_trait;
use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reddit API integration for cryptocurrency sentiment analysis
pub struct RedditSentimentSource {
    client: Client,
    user_agent: String,
    weight: f64,
    target_subreddits: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RedditResponse {
    data: RedditData,
}

#[derive(Debug, Deserialize)]
struct RedditData {
    children: Vec<RedditPost>,
    after: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RedditPost {
    data: PostData,
}

#[derive(Debug, Deserialize)]
struct PostData {
    id: String,
    title: String,
    selftext: String,
    author: String,
    created_utc: f64,
    score: i32,
    num_comments: u32,
    subreddit: String,
    url: String,
    ups: i32,
    downs: i32,
}

#[derive(Debug, Deserialize)]
struct RedditCommentsResponse {
    #[serde(rename = "1")]
    comments: RedditResponse,
}

impl RedditSentimentSource {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            user_agent: "SniperForge/1.0 (Crypto Trading Bot)".to_string(),
            weight: 0.3, // 30% weight in sentiment calculation
            target_subreddits: vec![
                "CryptoCurrency".to_string(),
                "solana".to_string(),
                "Bitcoin".to_string(),
                "ethereum".to_string(),
                "defi".to_string(),
                "SolanaToken".to_string(),
                "CryptoMarkets".to_string(),
                "altcoin".to_string(),
                "CryptoMoonShots".to_string(),
                "SatoshiStreetBets".to_string(),
            ],
        }
    }
    
    async fn search_posts(&self, symbol: &str, timeframe_minutes: u64) -> Result<Vec<PostData>> {
        let mut all_posts = Vec::new();
        let since_timestamp = (Utc::now() - Duration::minutes(timeframe_minutes as i64)).timestamp();
        
        // Search in each target subreddit
        for subreddit in &self.target_subreddits {
            if let Ok(posts) = self.get_subreddit_posts(subreddit, symbol, since_timestamp).await {
                all_posts.extend(posts);
            }
        }
        
        // Also search across all of Reddit
        if let Ok(posts) = self.search_all_reddit(symbol, since_timestamp).await {
            all_posts.extend(posts);
        }
        
        // Remove duplicates and sort by relevance
        all_posts.sort_by(|a, b| b.score.cmp(&a.score));
        all_posts.dedup_by(|a, b| a.id == b.id);
        
        Ok(all_posts)
    }
    
    async fn get_subreddit_posts(&self, subreddit: &str, symbol: &str, since_timestamp: i64) -> Result<Vec<PostData>> {
        let url = format!("https://www.reddit.com/r/{}/search.json", subreddit);
        let query = format!("{} OR ${}", symbol, symbol);
        
        let params = [
            ("q", query),
            ("restrict_sr", "1".to_string()),
            ("sort", "new".to_string()),
            ("t", "day".to_string()),
            ("limit", "50".to_string()),
        ];
        
        let response = self.client
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .query(&params)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Ok(Vec::new()); // Return empty vec on error instead of failing
        }
        
        let reddit_response: RedditResponse = response.json().await?;
        
        // Filter posts by timestamp
        let posts = reddit_response.data.children
            .into_iter()
            .map(|post| post.data)
            .filter(|post| post.created_utc >= since_timestamp as f64)
            .collect();
        
        Ok(posts)
    }
    
    async fn search_all_reddit(&self, symbol: &str, since_timestamp: i64) -> Result<Vec<PostData>> {
        let url = "https://www.reddit.com/search.json";
        let query = format!("({} OR ${}) AND (crypto OR cryptocurrency OR trading)", symbol, symbol);
        
        let params = [
            ("q", query),
            ("sort", "new".to_string()),
            ("t", "day".to_string()),
            ("limit", "25".to_string()),
        ];
        
        let response = self.client
            .get(url)
            .header("User-Agent", &self.user_agent)
            .query(&params)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Ok(Vec::new());
        }
        
        let reddit_response: RedditResponse = response.json().await?;
        
        let posts = reddit_response.data.children
            .into_iter()
            .map(|post| post.data)
            .filter(|post| post.created_utc >= since_timestamp as f64)
            .collect();
        
        Ok(posts)
    }
    
    fn analyze_post_sentiment(&self, post: &PostData) -> f64 {
        let full_text = format!("{} {}", post.title, post.selftext).to_lowercase();
        
        // Enhanced sentiment keywords for crypto
        let positive_keywords = [
            "bullish", "moon", "pump", "hodl", "diamond hands", "buy the dip",
            "to the moon", "breakthrough", "surge", "rally", "green", "profit",
            "gains", "up", "breakout", "explosion", "rocket", "bull run",
            "accumulate", "strong", "solid", "massive", "huge", "incredible",
            "amazing", "fantastic", "optimistic", "confident", "excited",
            "pumping", "mooning", "lambo", "rich", "wealth", "success",
            "bullish af", "lfg", "lets go", "holy grail", "golden", "diamond"
        ];
        
        let negative_keywords = [
            "bearish", "dump", "crash", "bear", "sell", "panic", "fear",
            "red", "loss", "down", "drop", "fall", "collapse", "disaster",
            "terrible", "awful", "scam", "rug pull", "dead", "rekt",
            "liquidated", "weak", "disappointed", "concerned", "worried",
            "scared", "dip", "bleeding", "fucked", "shit", "crap",
            "worthless", "trash", "garbage", "avoid", "stay away", "risky"
        ];
        
        let mut sentiment_score = 0.0;
        let mut word_count = 0;
        
        // Count positive keywords with weighted importance
        for keyword in &positive_keywords {
            let count = full_text.matches(keyword).count();
            if count > 0 {
                sentiment_score += count as f64;
                word_count += count;
            }
        }
        
        // Count negative keywords
        for keyword in &negative_keywords {
            let count = full_text.matches(keyword).count();
            if count > 0 {
                sentiment_score -= count as f64;
                word_count += count;
            }
        }
        
        // Adjust for Reddit-specific metrics
        let score_multiplier = if post.score > 0 {
            1.0 + (post.score as f64 / 100.0).min(3.0)
        } else {
            0.5 // Heavily downvoted posts get less weight
        };
        
        sentiment_score *= score_multiplier;
        
        // Adjust for comment engagement
        let comment_multiplier = 1.0 + (post.num_comments as f64 / 50.0).min(2.0);
        sentiment_score *= comment_multiplier;
        
        // Normalize to -1.0 to 1.0 range
        if word_count > 0 {
            sentiment_score / word_count as f64
        } else {
            // Use upvote ratio as fallback sentiment
            if post.score > 5 {
                0.2 // Mildly positive for upvoted posts
            } else if post.score < -2 {
                -0.2 // Mildly negative for downvoted posts
            } else {
                0.0 // Neutral
            }
        }.max(-1.0).min(1.0)
    }
    
    fn calculate_confidence(&self, posts: &[PostData]) -> f64 {
        if posts.is_empty() {
            return 0.0;
        }
        
        let total_score: i32 = posts.iter().map(|p| p.score.max(0)).sum();
        let total_comments: u32 = posts.iter().map(|p| p.num_comments).sum();
        let avg_score = total_score as f64 / posts.len() as f64;
        let avg_comments = total_comments as f64 / posts.len() as f64;
        
        // High-quality subreddits get higher confidence
        let quality_multiplier = posts
            .iter()
            .map(|post| match post.subreddit.as_str() {
                "CryptoCurrency" | "Bitcoin" | "ethereum" => 1.0,
                "solana" | "defi" => 0.9,
                "CryptoMarkets" | "altcoin" => 0.8,
                _ => 0.7,
            })
            .sum::<f64>() / posts.len() as f64;
        
        // Higher confidence with more posts, higher scores, and more engagement
        let volume_factor = (posts.len() as f64 / 20.0).min(1.0);
        let score_factor = (avg_score / 10.0).min(1.0);
        let comment_factor = (avg_comments / 5.0).min(1.0);
        
        (volume_factor * 0.4 + score_factor * 0.3 + comment_factor * 0.3) * quality_multiplier
    }
}

#[async_trait]
impl SentimentDataSource for RedditSentimentSource {
    async fn get_sentiment(&self, symbol: &str, timeframe: u64) -> Result<SentimentScore> {
        let posts = self.search_posts(symbol, timeframe).await?;
        
        if posts.is_empty() {
            return Err(SentimentError::InsufficientData {
                message: format!("No Reddit posts found for {}", symbol)
            }.into());
        }
        
        // Calculate average sentiment weighted by post score
        let mut weighted_sentiment = 0.0;
        let mut total_weight = 0.0;
        
        for post in &posts {
            let sentiment = self.analyze_post_sentiment(post);
            let weight = (post.score.max(1) as f64).sqrt(); // sqrt to reduce impact of viral posts
            
            weighted_sentiment += sentiment * weight;
            total_weight += weight;
        }
        
        let average_sentiment = if total_weight > 0.0 {
            weighted_sentiment / total_weight
        } else {
            0.0
        };
        
        let confidence = self.calculate_confidence(&posts);
        
        Ok(SentimentScore {
            sentiment: average_sentiment,
            confidence,
            mentions_count: posts.len() as u32,
            timestamp: Utc::now(),
            source: "Reddit".to_string(),
        })
    }
    
    async fn get_mentions(&self, symbol: &str, timeframe: u64) -> Result<Vec<Mention>> {
        let posts = self.search_posts(symbol, timeframe).await?;
        
        let mentions = posts
            .into_iter()
            .map(|post| {
                let sentiment = self.analyze_post_sentiment(&post);
                let engagement = post.num_comments + post.score.max(0) as u32;
                
                Mention {
                    id: post.id,
                    content: format!("{}\n\n{}", post.title, post.selftext),
                    sentiment,
                    author: post.author,
                    timestamp: DateTime::from_timestamp(post.created_utc as i64, 0)
                        .unwrap_or_else(|| Utc::now()),
                    source: format!("Reddit r/{}", post.subreddit),
                    engagement,
                }
            })
            .collect();
        
        Ok(mentions)
    }
    
    fn get_source_name(&self) -> &'static str {
        "Reddit"
    }
    
    fn get_weight(&self) -> f64 {
        self.weight
    }
}
