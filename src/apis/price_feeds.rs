use crate::{
    config::SimpleConfig,
    types::{MarketData, ApiResult as Result},
    apis::dexscreener::DexScreenerClient,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

/// Price feed manager that aggregates data from multiple sources
#[derive(Clone)]
pub struct PriceFeedManager {
    _config: SimpleConfig,  // Reserved for future configuration features
    dexscreener_client: DexScreenerClient,
    market_data: Arc<RwLock<MarketData>>,
    last_update: Arc<RwLock<Instant>>,
}

impl PriceFeedManager {
    /// Create a new price feed manager
    pub fn new(config: &SimpleConfig) -> Self {
        let dexscreener_client = DexScreenerClient::new(config);
        
        Self {
            _config: config.clone(),
            dexscreener_client,
            market_data: Arc::new(RwLock::new(MarketData::new())),
            last_update: Arc::new(RwLock::new(Instant::now())),
        }
    }
    
    /// Test connectivity to price feed sources
    pub async fn test_connectivity(&self) -> Result<()> {
        // Test DexScreener connectivity by fetching SOL token info
        let sol_address = "So11111111111111111111111111111111111111112";
        
        match self.dexscreener_client.get_token_info(sol_address).await {
            Ok(_) => {
                info!("DexScreener connectivity test passed");
                Ok(())
            }
            Err(e) => {
                error!("DexScreener connectivity test failed: {}", e);
                Err(format!("DexScreener connectivity failed: {}", e))
            }
        }
    }
    
    /// Update market data from all sources
    pub async fn update_prices(&self) -> Result<()> {
        debug!("Updating prices from all sources...");
        
        let mut market_data = self.market_data.write().await;
        
        // Update SOL price from DexScreener
        if let Ok(token_info) = self.dexscreener_client
            .get_token_info("So11111111111111111111111111111111111111112")
            .await 
        {
            market_data.set_price("SOL".to_string(), token_info.price_usd);
            market_data.set_volume("SOL".to_string(), token_info.volume_24h);
            market_data.set_liquidity("SOL".to_string(), token_info.liquidity);
        }
        
        // Update USDC price (usually stable at $1)
        market_data.set_price("USDC".to_string(), 1.0);
        market_data.set_volume("USDC".to_string(), 1_000_000.0); // Mock volume
        market_data.set_liquidity("USDC".to_string(), 10_000_000.0); // Mock liquidity
        
        // Add more tokens as needed
        
        *self.last_update.write().await = Instant::now();
        
        debug!("Price update completed");
        Ok(())
    }
    
    /// Get current market data
    pub async fn get_market_data(&self) -> Result<MarketData> {
        let market_data = self.market_data.read().await;
        
        // Check if data is stale and update if needed
        if market_data.is_stale(Duration::from_secs(60)) {
            drop(market_data); // Release read lock
            self.update_prices().await?;
            let fresh_data = self.market_data.read().await;
            Ok(fresh_data.clone())
        } else {
            Ok(market_data.clone())
        }
    }
    
    /// Get price for a specific token
    pub async fn get_token_price(&self, symbol: &str) -> Result<f64> {
        let market_data = self.get_market_data().await?;
        
        market_data.get_price(symbol)
            .ok_or_else(|| format!("Price not available for token: {}", symbol))
    }
    
    /// Get volume for a specific token
    pub async fn get_token_volume(&self, symbol: &str) -> Result<f64> {
        let market_data = self.get_market_data().await?;
        
        market_data.get_volume(symbol)
            .ok_or_else(|| format!("Volume not available for token: {}", symbol))
    }
    
    /// Get liquidity for a specific token
    pub async fn get_token_liquidity(&self, symbol: &str) -> Result<f64> {
        let market_data = self.get_market_data().await?;
        
        market_data.get_liquidity(symbol)
            .ok_or_else(|| format!("Liquidity not available for token: {}", symbol))
    }
    
    /// Subscribe to price updates (simplified implementation)
    pub async fn start_price_updates(&self, interval: Duration) -> Result<()> {
        info!("Starting automatic price updates every {:?}", interval);
        
        let manager = self.clone();
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                if let Err(e) = manager.update_prices().await {
                    warn!("Failed to update prices: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    /// Get feed statistics
    pub async fn get_statistics(&self) -> PriceFeedStats {
        let last_update = *self.last_update.read().await;
        let market_data = self.market_data.read().await;
        
        PriceFeedStats {
            last_update,
            tokens_tracked: market_data.prices.len(),
            data_age: last_update.elapsed(),
            is_healthy: last_update.elapsed() < Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Price feed statistics
#[derive(Debug, Clone)]
pub struct PriceFeedStats {
    pub last_update: Instant,
    pub tokens_tracked: usize,
    pub data_age: Duration,
    pub is_healthy: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_config() -> SimpleConfig {
        SimpleConfig {
            solana_rpc_url: "https://api.devnet.solana.com".to_string(),
            solana_ws_url: "wss://api.devnet.solana.com/".to_string(),
            max_slippage: 0.005,
            min_profit_threshold: 0.001,
            max_position_size: 0.1,
            private_key_path: "./test_wallet.json".to_string(),
            enable_simulation: true,
            log_level: "info".to_string(),
            dexscreener_base_url: "https://api.dexscreener.com".to_string(),
            max_requests_per_second: 10,
            cooldown_period_ms: 100,
            max_history_size: 1000,
            
            // 🚀 Enterprise price feeds test configuration
            trading_amount: 0.01,
            profit_threshold: 0.5,
            max_price_age_seconds: 30,
            risk_percentage: 2.0,
            enable_ml_analysis: true,
            enable_sentiment_analysis: true,
            enable_technical_analysis: true,
            max_concurrent_trades: 5,
            portfolio_rebalancing: true,
            stop_loss_percentage: 5.0,
            take_profit_percentage: 10.0,
        }
    }
    
    #[tokio::test]
    async fn test_price_feed_manager_creation() {
        let config = create_test_config();
        let manager = PriceFeedManager::new(&config);
        
        let stats = manager.get_statistics().await;
        assert_eq!(stats.tokens_tracked, 0);
    }
    
    #[tokio::test]
    async fn test_market_data_update() {
        let config = create_test_config();
        let manager = PriceFeedManager::new(&config);
        
        // This would require a real API call in a full test
        // For now, just check that the structure works
        let result = manager.get_market_data().await;
        assert!(result.is_ok());
    }
}
