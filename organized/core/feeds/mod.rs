// ================================================================================
// PRICE FEEDS - Gestión de Feeds de Precios
// ================================================================================

use std::collections::HashMap;
use std::time::{Duration, Instant};
use anyhow::Result;
use tracing::{info, debug, warn, error};
use serde::{Serialize, Deserialize};
use tokio::time::sleep;

use crate::{CoreResult, CoreError};

/// Gestor de feeds de precios
pub struct PriceFeedManager {
    config: PriceFeedConfig,
    dex_feeds: HashMap<String, DexPriceFeed>,
    price_cache: HashMap<String, CachedPrice>,
    statistics: PriceFeedStatistics,
    last_update: Instant,
}

/// Configuración de price feeds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceFeedConfig {
    pub dexscreener_enabled: bool,
    pub coingecko_enabled: bool,
    pub birdeye_enabled: bool,
    pub helius_enabled: bool,
    pub update_interval_ms: u64,
    pub timeout_ms: u64,
    pub max_price_age_ms: u64,
    pub fallback_enabled: bool,
}

/// Feed de precios de DEX
#[derive(Debug, Clone)]
pub struct DexPriceFeed {
    pub name: String,
    pub api_endpoint: String,
    pub tokens: HashMap<String, TokenPriceData>,
    pub last_update: Instant,
    pub update_count: u64,
    pub error_count: u64,
    pub avg_response_time_ms: f64,
}

/// Datos de precio de token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPriceData {
    pub symbol: String,
    pub price: f64,
    pub volume: Option<f64>,
    pub liquidity: Option<f64>,
    pub price_change_24h: Option<f64>,
    pub market_cap: Option<f64>,
    pub confidence: Option<f64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: String,
}

/// Precio en caché
#[derive(Debug, Clone)]
pub struct CachedPrice {
    pub data: TokenPriceData,
    pub cached_at: Instant,
    pub access_count: u32,
}

/// Estadísticas de price feeds
#[derive(Debug, Clone, Default)]
pub struct PriceFeedStatistics {
    pub total_updates: u64,
    pub successful_updates: u64,
    pub failed_updates: u64,
    pub total_tokens_tracked: u32,
    pub avg_update_time_ms: f64,
    pub cache_hit_rate: f64,
    pub data_freshness_score: f64,
}

/// Configuración de API específica
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub rate_limit_per_second: u32,
    pub timeout_ms: u64,
    pub enabled: bool,
}

impl PriceFeedManager {
    /// Crear nuevo gestor de price feeds
    pub async fn new(config: &PriceFeedConfig) -> CoreResult<Self> {
        info!("📊 Inicializando PriceFeedManager");
        
        let mut dex_feeds = HashMap::new();
        
        // Inicializar feeds de DEXs principales
        if config.dexscreener_enabled {
            dex_feeds.insert("dexscreener".to_string(), DexPriceFeed::new(
                "dexscreener",
                "https://api.dexscreener.com/latest/dex",
            ));
        }
        
        if config.coingecko_enabled {
            dex_feeds.insert("coingecko".to_string(), DexPriceFeed::new(
                "coingecko", 
                "https://api.coingecko.com/api/v3",
            ));
        }
        
        let manager = Self {
            config: config.clone(),
            dex_feeds,
            price_cache: HashMap::new(),
            statistics: PriceFeedStatistics::default(),
            last_update: Instant::now(),
        };
        
        info!("✅ PriceFeedManager inicializado con {} fuentes", manager.dex_feeds.len());
        
        Ok(manager)
    }
    
    /// Actualizar todos los feeds
    pub async fn update_all(&mut self) -> CoreResult<()> {
        let update_start = Instant::now();
        
        // Verificar si necesita actualización
        if update_start.duration_since(self.last_update).as_millis() < self.config.update_interval_ms as u128 {
            return Ok(()); // No necesita actualización aún
        }
        
        debug!("📊 Actualizando todos los price feeds");
        
        let mut successful_updates = 0;
        let mut failed_updates = 0;
        
        // Actualizar cada feed
        for (feed_name, feed) in self.dex_feeds.iter_mut() {
            match self.update_feed(feed).await {
                Ok(_) => {
                    successful_updates += 1;
                    debug!("✅ Feed {} actualizado", feed_name);
                },
                Err(e) => {
                    failed_updates += 1;
                    warn!("⚠️  Error actualizando feed {}: {}", feed_name, e);
                }
            }
        }
        
        // Actualizar estadísticas
        let update_time = update_start.elapsed().as_millis() as f64;
        self.update_statistics(successful_updates, failed_updates, update_time);
        
        self.last_update = Instant::now();
        
        info!("📊 Feeds actualizados: {} exitosos, {} fallidos ({:.2}ms)", 
              successful_updates, failed_updates, update_time);
        
        Ok(())
    }
    
    /// Actualizar un feed específico
    async fn update_feed(&mut self, feed: &mut DexPriceFeed) -> CoreResult<()> {
        let update_start = Instant::now();
        
        match feed.name.as_str() {
            "dexscreener" => self.update_dexscreener_feed(feed).await?,
            "coingecko" => self.update_coingecko_feed(feed).await?,
            _ => {
                return Err(CoreError::PriceFeed(format!("Unknown feed: {}", feed.name)));
            }
        }
        
        // Actualizar estadísticas del feed
        let response_time = update_start.elapsed().as_millis() as f64;
        feed.update_response_time(response_time);
        feed.last_update = Instant::now();
        feed.update_count += 1;
        
        // Actualizar caché
        self.update_cache_from_feed(feed)?;
        
        Ok(())
    }
    
    /// Actualizar feed de DexScreener
    async fn update_dexscreener_feed(&self, feed: &mut DexPriceFeed) -> CoreResult<()> {
        debug!("📡 Actualizando DexScreener feed");
        
        // Simular datos de DexScreener (en implementación real usaría HTTP client)
        let mock_tokens = self.get_mock_dexscreener_data();
        
        for (symbol, price_data) in mock_tokens {
            feed.tokens.insert(symbol, price_data);
        }
        
        Ok(())
    }
    
    /// Actualizar feed de CoinGecko
    async fn update_coingecko_feed(&self, feed: &mut DexPriceFeed) -> CoreResult<()> {
        debug!("📡 Actualizando CoinGecko feed");
        
        // Simular datos de CoinGecko (en implementación real usaría HTTP client)
        let mock_tokens = self.get_mock_coingecko_data();
        
        for (symbol, price_data) in mock_tokens {
            feed.tokens.insert(symbol, price_data);
        }
        
        Ok(())
    }
    
    /// Obtener datos mock de DexScreener
    fn get_mock_dexscreener_data(&self) -> HashMap<String, TokenPriceData> {
        let mut tokens = HashMap::new();
        
        // Datos simulados realistas
        tokens.insert("SOL".to_string(), TokenPriceData {
            symbol: "SOL".to_string(),
            price: 145.67,
            volume: Some(450_000_000.0),
            liquidity: Some(120_000_000.0),
            price_change_24h: Some(2.34),
            market_cap: Some(65_000_000_000.0),
            confidence: Some(0.95),
            timestamp: chrono::Utc::now(),
            source: "dexscreener".to_string(),
        });
        
        tokens.insert("USDC".to_string(), TokenPriceData {
            symbol: "USDC".to_string(),
            price: 1.0002,
            volume: Some(890_000_000.0),
            liquidity: Some(200_000_000.0),
            price_change_24h: Some(0.01),
            market_cap: Some(32_000_000_000.0),
            confidence: Some(0.99),
            timestamp: chrono::Utc::now(),
            source: "dexscreener".to_string(),
        });
        
        tokens.insert("USDT".to_string(), TokenPriceData {
            symbol: "USDT".to_string(),
            price: 0.9998,
            volume: Some(1_200_000_000.0),
            liquidity: Some(180_000_000.0),
            price_change_24h: Some(-0.02),
            market_cap: Some(95_000_000_000.0),
            confidence: Some(0.98),
            timestamp: chrono::Utc::now(),
            source: "dexscreener".to_string(),
        });
        
        tokens.insert("RAY".to_string(), TokenPriceData {
            symbol: "RAY".to_string(),
            price: 3.45,
            volume: Some(25_000_000.0),
            liquidity: Some(8_000_000.0),
            price_change_24h: Some(5.67),
            market_cap: Some(890_000_000.0),
            confidence: Some(0.85),
            timestamp: chrono::Utc::now(),
            source: "dexscreener".to_string(),
        });
        
        tokens.insert("ORCA".to_string(), TokenPriceData {
            symbol: "ORCA".to_string(),
            price: 1.89,
            volume: Some(15_000_000.0),
            liquidity: Some(5_000_000.0),
            price_change_24h: Some(-1.23),
            market_cap: Some(234_000_000.0),
            confidence: Some(0.82),
            timestamp: chrono::Utc::now(),
            source: "dexscreener".to_string(),
        });
        
        tokens
    }
    
    /// Obtener datos mock de CoinGecko
    fn get_mock_coingecko_data(&self) -> HashMap<String, TokenPriceData> {
        let mut tokens = HashMap::new();
        
        // Datos ligeramente diferentes para simular variación entre fuentes
        tokens.insert("SOL".to_string(), TokenPriceData {
            symbol: "SOL".to_string(),
            price: 145.89, // Precio ligeramente diferente
            volume: Some(455_000_000.0),
            liquidity: Some(118_000_000.0),
            price_change_24h: Some(2.41),
            market_cap: Some(65_100_000_000.0),
            confidence: Some(0.97),
            timestamp: chrono::Utc::now(),
            source: "coingecko".to_string(),
        });
        
        tokens.insert("USDC".to_string(), TokenPriceData {
            symbol: "USDC".to_string(),
            price: 1.0001,
            volume: Some(885_000_000.0),
            liquidity: Some(198_000_000.0),
            price_change_24h: Some(0.01),
            market_cap: Some(32_050_000_000.0),
            confidence: Some(0.99),
            timestamp: chrono::Utc::now(),
            source: "coingecko".to_string(),
        });
        
        tokens
    }
    
    /// Actualizar caché desde feed
    fn update_cache_from_feed(&mut self, feed: &DexPriceFeed) -> CoreResult<()> {
        for (symbol, price_data) in &feed.tokens {
            let cache_key = format!("{}_{}", feed.name, symbol);
            
            self.price_cache.insert(cache_key, CachedPrice {
                data: price_data.clone(),
                cached_at: Instant::now(),
                access_count: 0,
            });
        }
        
        Ok(())
    }
    
    /// Obtener precios de un DEX específico
    pub async fn get_dex_prices(&self, dex_name: &str) -> CoreResult<HashMap<String, TokenPriceData>> {
        if let Some(feed) = self.dex_feeds.get(dex_name) {
            Ok(feed.tokens.clone())
        } else {
            // Buscar en caché con el nombre del DEX
            let mut prices = HashMap::new();
            
            for (cache_key, cached_price) in &self.price_cache {
                if cache_key.starts_with(&format!("{}_", dex_name)) {
                    if let Some(symbol) = cache_key.split('_').nth(1) {
                        prices.insert(symbol.to_string(), cached_price.data.clone());
                    }
                }
            }
            
            if prices.is_empty() {
                Err(CoreError::PriceFeed(format!("No prices found for DEX: {}", dex_name)))
            } else {
                Ok(prices)
            }
        }
    }
    
    /// Obtener precio de un token específico
    pub async fn get_token_price(&mut self, symbol: &str, preferred_source: Option<&str>) -> CoreResult<TokenPriceData> {
        // Intentar obtener del source preferido primero
        if let Some(source) = preferred_source {
            let cache_key = format!("{}_{}", source, symbol);
            if let Some(cached_price) = self.price_cache.get_mut(&cache_key) {
                if self.is_price_fresh(&cached_price) {
                    cached_price.access_count += 1;
                    return Ok(cached_price.data.clone());
                }
            }
        }
        
        // Buscar en cualquier fuente disponible
        let cache_pattern = format!("_{}", symbol);
        for (cache_key, cached_price) in self.price_cache.iter_mut() {
            if cache_key.ends_with(&cache_pattern) && self.is_price_fresh(cached_price) {
                cached_price.access_count += 1;
                return Ok(cached_price.data.clone());
            }
        }
        
        Err(CoreError::PriceFeed(format!("No fresh price found for token: {}", symbol)))
    }
    
    /// Verificar si el precio está fresco
    fn is_price_fresh(&self, cached_price: &CachedPrice) -> bool {
        let age_ms = cached_price.cached_at.elapsed().as_millis();
        age_ms < self.config.max_price_age_ms as u128
    }
    
    /// Obtener mejores precios (promedio ponderado por confianza)
    pub async fn get_best_prices(&self, symbols: &[String]) -> CoreResult<HashMap<String, TokenPriceData>> {
        let mut best_prices = HashMap::new();
        
        for symbol in symbols {
            if let Ok(price) = self.calculate_weighted_price(symbol).await {
                best_prices.insert(symbol.clone(), price);
            }
        }
        
        Ok(best_prices)
    }
    
    /// Calcular precio ponderado por confianza
    async fn calculate_weighted_price(&self, symbol: &str) -> CoreResult<TokenPriceData> {
        let mut prices = Vec::new();
        
        // Recopilar todos los precios disponibles para este token
        for (cache_key, cached_price) in &self.price_cache {
            if cache_key.ends_with(&format!("_{}", symbol)) && self.is_price_fresh(cached_price) {
                prices.push(&cached_price.data);
            }
        }
        
        if prices.is_empty() {
            return Err(CoreError::PriceFeed(format!("No prices available for {}", symbol)));
        }
        
        if prices.len() == 1 {
            return Ok(prices[0].clone());
        }
        
        // Calcular precio ponderado por confianza
        let mut weighted_price = 0.0;
        let mut total_confidence = 0.0;
        let mut total_volume = 0.0;
        let mut total_liquidity = 0.0;
        
        for price_data in &prices {
            let confidence = price_data.confidence.unwrap_or(0.5);
            weighted_price += price_data.price * confidence;
            total_confidence += confidence;
            
            if let Some(volume) = price_data.volume {
                total_volume += volume;
            }
            
            if let Some(liquidity) = price_data.liquidity {
                total_liquidity += liquidity;
            }
        }
        
        let final_price = if total_confidence > 0.0 {
            weighted_price / total_confidence
        } else {
            prices.iter().map(|p| p.price).sum::<f64>() / prices.len() as f64
        };
        
        // Calcular cambio de precio promedio
        let avg_change_24h = prices.iter()
            .filter_map(|p| p.price_change_24h)
            .sum::<f64>() / prices.len() as f64;
        
        Ok(TokenPriceData {
            symbol: symbol.to_string(),
            price: final_price,
            volume: if total_volume > 0.0 { Some(total_volume) } else { None },
            liquidity: if total_liquidity > 0.0 { Some(total_liquidity) } else { None },
            price_change_24h: Some(avg_change_24h),
            market_cap: prices.first().and_then(|p| p.market_cap),
            confidence: Some(total_confidence / prices.len() as f64),
            timestamp: chrono::Utc::now(),
            source: "weighted_average".to_string(),
        })
    }
    
    /// Limpiar caché antiguo
    pub fn cleanup_cache(&mut self) {
        let now = Instant::now();
        let max_age = Duration::from_millis(self.config.max_price_age_ms * 2); // Double the max age for cleanup
        
        self.price_cache.retain(|_, cached_price| {
            now.duration_since(cached_price.cached_at) < max_age
        });
    }
    
    /// Actualizar estadísticas
    fn update_statistics(&mut self, successful: u32, failed: u32, update_time_ms: f64) {
        self.statistics.total_updates += 1;
        self.statistics.successful_updates += successful as u64;
        self.statistics.failed_updates += failed as u64;
        
        // Actualizar tiempo promedio de actualización
        let total_updates = self.statistics.total_updates as f64;
        self.statistics.avg_update_time_ms = 
            (self.statistics.avg_update_time_ms * (total_updates - 1.0) + update_time_ms) / total_updates;
        
        // Calcular hit rate del caché
        let total_accesses: u32 = self.price_cache.values().map(|c| c.access_count).sum();
        let cache_hits = self.price_cache.len() as u32;
        self.statistics.cache_hit_rate = if total_accesses > 0 {
            cache_hits as f64 / total_accesses as f64
        } else {
            0.0
        };
        
        // Calcular score de freshness
        let fresh_prices = self.price_cache.values()
            .filter(|c| self.is_price_fresh(c))
            .count();
        self.statistics.data_freshness_score = if !self.price_cache.is_empty() {
            fresh_prices as f64 / self.price_cache.len() as f64
        } else {
            0.0
        };
        
        self.statistics.total_tokens_tracked = self.price_cache.len() as u32;
    }
    
    /// Obtener estadísticas
    pub fn get_statistics(&self) -> &PriceFeedStatistics {
        &self.statistics
    }
    
    /// Obtener estado de salud de los feeds
    pub fn get_health_status(&self) -> HashMap<String, f64> {
        let mut health_scores = HashMap::new();
        
        for (name, feed) in &self.dex_feeds {
            let success_rate = if feed.update_count > 0 {
                1.0 - (feed.error_count as f64 / feed.update_count as f64)
            } else {
                0.5
            };
            
            let freshness_score = if feed.last_update.elapsed().as_secs() < 60 {
                1.0
            } else {
                0.5
            };
            
            let response_time_score = if feed.avg_response_time_ms < 1000.0 {
                1.0
            } else if feed.avg_response_time_ms < 5000.0 {
                0.7
            } else {
                0.3
            };
            
            let overall_health = (success_rate + freshness_score + response_time_score) / 3.0;
            health_scores.insert(name.clone(), overall_health);
        }
        
        health_scores
    }
}

impl DexPriceFeed {
    fn new(name: &str, api_endpoint: &str) -> Self {
        Self {
            name: name.to_string(),
            api_endpoint: api_endpoint.to_string(),
            tokens: HashMap::new(),
            last_update: Instant::now(),
            update_count: 0,
            error_count: 0,
            avg_response_time_ms: 0.0,
        }
    }
    
    fn update_response_time(&mut self, response_time_ms: f64) {
        if self.update_count == 0 {
            self.avg_response_time_ms = response_time_ms;
        } else {
            self.avg_response_time_ms = 
                (self.avg_response_time_ms * self.update_count as f64 + response_time_ms) / (self.update_count + 1) as f64;
        }
    }
}

impl Default for PriceFeedConfig {
    fn default() -> Self {
        Self {
            dexscreener_enabled: true,
            coingecko_enabled: true,
            birdeye_enabled: false,
            helius_enabled: false,
            update_interval_ms: 10000,  // 10 seconds
            timeout_ms: 5000,           // 5 seconds
            max_price_age_ms: 30000,    // 30 seconds
            fallback_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_price_feed_creation() {
        let config = PriceFeedConfig::default();
        let manager = PriceFeedManager::new(&config).await;
        assert!(manager.is_ok());
    }
    
    #[tokio::test]
    async fn test_mock_data_generation() {
        let config = PriceFeedConfig::default();
        let manager = PriceFeedManager::new(&config).await.unwrap();
        
        let dexscreener_data = manager.get_mock_dexscreener_data();
        assert!(dexscreener_data.contains_key("SOL"));
        assert!(dexscreener_data.contains_key("USDC"));
        
        let sol_price = &dexscreener_data["SOL"];
        assert!(sol_price.price > 0.0);
        assert!(sol_price.confidence.unwrap_or(0.0) > 0.0);
    }
}
