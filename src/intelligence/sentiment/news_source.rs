use super::types::{SentimentDataSource, SentimentScore, Mention, SentimentError};
use async_trait::async_trait;
use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use scraper::{Html, Selector};
use std::collections::HashMap;

/// News feeds aggregator for cryptocurrency sentiment analysis
pub struct NewsSentimentSource {
    client: Client,
    weight: f64,
    news_sources: Vec<NewsSource>,
}

#[derive(Debug, Clone)]
struct NewsSource {
    name: String,
    base_url: String,
    rss_url: Option<String>,
    search_endpoint: Option<String>,
    api_key: Option<String>,
    credibility_score: f64, // 0.0 to 1.0, higher = more credible
}

#[derive(Debug, Deserialize)]
struct RSSFeed {
    channel: RSSChannel,
}

#[derive(Debug, Deserialize)]
struct RSSChannel {
    item: Vec<RSSItem>,
}

#[derive(Debug, Deserialize)]
struct RSSItem {
    title: String,
    description: Option<String>,
    link: String,
    #[serde(rename = "pubDate")]
    pub_date: Option<String>,
    category: Option<Vec<String>>,
}

// CoinGecko News API Response
#[derive(Debug, Deserialize)]
struct CoinGeckoNewsResponse {
    data: Vec<CoinGeckoNewsItem>,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoNewsItem {
    id: String,
    title: String,
    description: String,
    url: String,
    published_at: String,
    source: CoinGeckoNewsSource,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoNewsSource {
    name: String,
    url: String,
}

// NewsAPI.org Response
#[derive(Debug, Deserialize)]
struct NewsApiResponse {
    status: String,
    #[serde(rename = "totalResults")]
    total_results: u32,
    articles: Vec<NewsApiArticle>,
}

#[derive(Debug, Deserialize)]
struct NewsApiArticle {
    source: NewsApiSource,
    title: String,
    description: Option<String>,
    url: String,
    #[serde(rename = "publishedAt")]
    published_at: String,
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NewsApiSource {
    id: Option<String>,
    name: String,
}

impl NewsSentimentSource {
    pub fn new(news_api_key: Option<String>) -> Self {
        let mut news_sources = vec![
            NewsSource {
                name: "CoinDesk".to_string(),
                base_url: "https://www.coindesk.com".to_string(),
                rss_url: Some("https://www.coindesk.com/arc/outboundfeeds/rss/".to_string()),
                search_endpoint: None,
                api_key: None,
                credibility_score: 0.9,
            },
            NewsSource {
                name: "CoinTelegraph".to_string(),
                base_url: "https://cointelegraph.com".to_string(),
                rss_url: Some("https://cointelegraph.com/rss".to_string()),
                search_endpoint: None,
                api_key: None,
                credibility_score: 0.8,
            },
            NewsSource {
                name: "Decrypt".to_string(),
                base_url: "https://decrypt.co".to_string(),
                rss_url: Some("https://decrypt.co/feed".to_string()),
                search_endpoint: None,
                api_key: None,
                credibility_score: 0.85,
            },
            NewsSource {
                name: "The Block".to_string(),
                base_url: "https://www.theblock.co".to_string(),
                rss_url: Some("https://www.theblock.co/rss.xml".to_string()),
                search_endpoint: None,
                api_key: None,
                credibility_score: 0.9,
            },
            NewsSource {
                name: "CoinGecko".to_string(),
                base_url: "https://www.coingecko.com".to_string(),
                rss_url: None,
                search_endpoint: Some("https://api.coingecko.com/api/v3/news".to_string()),
                api_key: None,
                credibility_score: 0.85,
            },
        ];
        
        // Add NewsAPI if key is provided
        if let Some(api_key) = news_api_key {
            news_sources.push(NewsSource {
                name: "NewsAPI".to_string(),
                base_url: "https://newsapi.org".to_string(),
                rss_url: None,
                search_endpoint: Some("https://newsapi.org/v2/everything".to_string()),
                api_key: Some(api_key),
                credibility_score: 0.75,
            });
        }
        
        Self {
            client: Client::new(),
            weight: 0.2, // 20% weight in sentiment calculation
            news_sources,
        }
    }
    
    async fn fetch_news_articles(&self, symbol: &str, timeframe_minutes: u64) -> Result<Vec<NewsArticle>> {
        let mut all_articles = Vec::new();
        let since_time = Utc::now() - Duration::minutes(timeframe_minutes as i64);
        
        // Fetch from each news source
        for source in &self.news_sources {
            if let Ok(articles) = self.fetch_from_source(source, symbol, since_time).await {
                all_articles.extend(articles);
            }
        }
        
        // Remove duplicates and sort by publication date
        all_articles.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        all_articles.dedup_by(|a, b| a.title.trim() == b.title.trim());
        
        Ok(all_articles)
    }
    
    async fn fetch_from_source(&self, source: &NewsSource, symbol: &str, since_time: DateTime<Utc>) -> Result<Vec<NewsArticle>> {
        match source.name.as_str() {
            "CoinGecko" => self.fetch_coingecko_news(symbol).await,
            "NewsAPI" => self.fetch_newsapi_articles(source, symbol, since_time).await,
            _ => self.fetch_rss_articles(source, symbol, since_time).await,
        }
    }
    
    async fn fetch_rss_articles(&self, source: &NewsSource, symbol: &str, since_time: DateTime<Utc>) -> Result<Vec<NewsArticle>> {
        let rss_url = source.rss_url.as_ref().ok_or_else(|| {
            anyhow::anyhow!("No RSS URL for source: {}", source.name)
        })?;
        
        let response = self.client
            .get(rss_url)
            .send()
            .await?;
        
        let content = response.text().await?;
        
        // Parse RSS XML (simplified - in production use a proper RSS parser)
        let articles = self.parse_rss_content(&content, symbol, since_time, source)?;
        
        Ok(articles)
    }
    
    async fn fetch_coingecko_news(&self, symbol: &str) -> Result<Vec<NewsArticle>> {
        let url = "https://api.coingecko.com/api/v3/news";
        
        let response = self.client
            .get(url)
            .send()
            .await?;
        
        let news_response: CoinGeckoNewsResponse = response.json().await?;
        
        let articles = news_response.data
            .into_iter()
            .filter(|item| {
                let content = format!("{} {}", item.title, item.description).to_lowercase();
                content.contains(&symbol.to_lowercase()) || content.contains(&format!("${}", symbol.to_lowercase()))
            })
            .map(|item| NewsArticle {
                title: item.title,
                content: item.description,
                url: item.url,
                published_at: item.published_at.parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now()),
                source: item.source.name,
                credibility_score: 0.85,
            })
            .collect();
        
        Ok(articles)
    }
    
    async fn fetch_newsapi_articles(&self, source: &NewsSource, symbol: &str, since_time: DateTime<Utc>) -> Result<Vec<NewsArticle>> {
        let api_key = source.api_key.as_ref().ok_or_else(|| {
            anyhow::anyhow!("No API key for NewsAPI")
        })?;
        
        let endpoint = source.search_endpoint.as_ref().unwrap();
        let query = format!("{} OR cryptocurrency OR crypto", symbol);
        let from_date = since_time.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        
        let params = [
            ("q", query),
            ("from", from_date),
            ("sortBy", "publishedAt".to_string()),
            ("apiKey", api_key.clone()),
            ("language", "en".to_string()),
            ("pageSize", "50".to_string()),
        ];
        
        let response = self.client
            .get(endpoint)
            .query(&params)
            .send()
            .await?;
        
        let news_response: NewsApiResponse = response.json().await?;
        
        let articles = news_response.articles
            .into_iter()
            .filter(|article| {
                let content = format!("{} {}", 
                    article.title, 
                    article.description.as_deref().unwrap_or("")
                ).to_lowercase();
                content.contains(&symbol.to_lowercase()) || content.contains(&format!("${}", symbol.to_lowercase()))
            })
            .map(|article| NewsArticle {
                title: article.title,
                content: article.description.unwrap_or_default(),
                url: article.url,
                published_at: article.published_at.parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now()),
                source: article.source.name,
                credibility_score: 0.75,
            })
            .collect();
        
        Ok(articles)
    }
    
    fn parse_rss_content(&self, content: &str, symbol: &str, since_time: DateTime<Utc>, source: &NewsSource) -> Result<Vec<NewsArticle>> {
        // Simplified RSS parsing - in production use a proper XML parser
        let document = Html::parse_document(content);
        let item_selector = Selector::parse("item").unwrap();
        let title_selector = Selector::parse("title").unwrap();
        let description_selector = Selector::parse("description").unwrap();
        let link_selector = Selector::parse("link").unwrap();
        let pubdate_selector = Selector::parse("pubDate").unwrap();
        
        let mut articles = Vec::new();
        
        for item in document.select(&item_selector) {
            let title = item.select(&title_selector)
                .next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default();
            
            let description = item.select(&description_selector)
                .next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default();
            
            let link = item.select(&link_selector)
                .next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default();
            
            // Check if article is relevant to the symbol
            let content = format!("{} {}", title, description).to_lowercase();
            if content.contains(&symbol.to_lowercase()) || content.contains(&format!("${}", symbol.to_lowercase())) {
                articles.push(NewsArticle {
                    title,
                    content: description,
                    url: link,
                    published_at: Utc::now(), // Simplified - parse actual date in production
                    source: source.name.clone(),
                    credibility_score: source.credibility_score,
                });
            }
        }
        
        Ok(articles)
    }
    
    fn analyze_article_sentiment(&self, article: &NewsArticle) -> f64 {
        let full_text = format!("{} {}", article.title, article.content).to_lowercase();
        
        // Financial/crypto-specific sentiment keywords
        let positive_keywords = [
            "surge", "rally", "breakthrough", "bullish", "gains", "profit",
            "growth", "adoption", "partnership", "integration", "launch",
            "success", "milestone", "achievement", "expansion", "upgrade",
            "institutional", "investment", "funding", "backing", "support",
            "innovation", "revolutionary", "game-changing", "promising",
            "optimistic", "confidence", "momentum", "upward", "rising",
            "breakthrough", "milestone", "record high", "all-time high"
        ];
        
        let negative_keywords = [
            "crash", "plunge", "decline", "bearish", "loss", "drop",
            "concern", "worry", "fear", "uncertainty", "volatility",
            "regulation", "ban", "restriction", "crackdown", "investigation",
            "hack", "breach", "scandal", "fraud", "scam", "risk",
            "warning", "caution", "downturn", "correction", "sell-off",
            "bearish", "pessimistic", "doubt", "skeptical", "challenging"
        ];
        
        let mut sentiment_score = 0.0;
        let mut word_count = 0;
        
        // Count positive keywords
        for keyword in &positive_keywords {
            let count = full_text.matches(keyword).count();
            sentiment_score += count as f64;
            word_count += count;
        }
        
        // Count negative keywords
        for keyword in &negative_keywords {
            let count = full_text.matches(keyword).count();
            sentiment_score -= count as f64;
            word_count += count;
        }
        
        // Apply credibility weighting
        sentiment_score *= article.credibility_score;
        
        // Normalize to -1.0 to 1.0 range
        if word_count > 0 {
            sentiment_score / word_count as f64
        } else {
            0.0 // Neutral if no sentiment keywords found
        }.max(-1.0).min(1.0)
    }
    
    fn calculate_confidence(&self, articles: &[NewsArticle]) -> f64 {
        if articles.is_empty() {
            return 0.0;
        }
        
        let avg_credibility: f64 = articles
            .iter()
            .map(|a| a.credibility_score)
            .sum::<f64>() / articles.len() as f64;
        
        // Higher confidence with more articles and higher credibility
        let volume_factor = (articles.len() as f64 / 10.0).min(1.0);
        let credibility_factor = avg_credibility;
        
        (volume_factor * 0.6 + credibility_factor * 0.4).min(1.0)
    }
}

#[derive(Debug, Clone)]
struct NewsArticle {
    title: String,
    content: String,
    url: String,
    published_at: DateTime<Utc>,
    source: String,
    credibility_score: f64,
}

#[async_trait]
impl SentimentDataSource for NewsSentimentSource {
    async fn get_sentiment(&self, symbol: &str, timeframe: u64) -> Result<SentimentScore> {
        let articles = self.fetch_news_articles(symbol, timeframe).await?;
        
        if articles.is_empty() {
            return Err(SentimentError::InsufficientData {
                message: format!("No news articles found for {}", symbol)
            }.into());
        }
        
        // Calculate weighted average sentiment
        let mut weighted_sentiment = 0.0;
        let mut total_weight = 0.0;
        
        for article in &articles {
            let sentiment = self.analyze_article_sentiment(article);
            let weight = article.credibility_score;
            
            weighted_sentiment += sentiment * weight;
            total_weight += weight;
        }
        
        let average_sentiment = if total_weight > 0.0 {
            weighted_sentiment / total_weight
        } else {
            0.0
        };
        
        let confidence = self.calculate_confidence(&articles);
        
        Ok(SentimentScore {
            sentiment: average_sentiment,
            confidence,
            mentions_count: articles.len() as u32,
            timestamp: Utc::now(),
            source: "News".to_string(),
        })
    }
    
    async fn get_mentions(&self, symbol: &str, timeframe: u64) -> Result<Vec<Mention>> {
        let articles = self.fetch_news_articles(symbol, timeframe).await?;
        
        let mentions = articles
            .into_iter()
            .map(|article| {
                let sentiment = self.analyze_article_sentiment(&article);
                
                Mention {
                    id: article.url.clone(),
                    content: format!("{}\n\n{}", article.title, article.content),
                    sentiment,
                    author: article.source.clone(),
                    timestamp: article.published_at,
                    source: format!("News: {}", article.source),
                    engagement: (article.credibility_score * 100.0) as u32, // Use credibility as engagement metric
                }
            })
            .collect();
        
        Ok(mentions)
    }
    
    fn get_source_name(&self) -> &'static str {
        "News"
    }
    
    fn get_weight(&self) -> f64 {
        self.weight
    }
}
