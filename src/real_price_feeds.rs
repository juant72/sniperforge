// ================================================================================
// REAL PRICE FEEDS - SISTEMA DE PRECIOS REALES
// ================================================================================
// Sistema que obtiene precios reales de múltiples DEXs para detectar arbitraje
// ================================================================================

use anyhow::{Result, anyhow};
use tracing::{info, warn, debug};
use std::collections::HashMap;
use serde_json::Value;
use tokio::time::{timeout, Duration};

/// Cliente para obtener precios reales de múltiples DEXs
pub struct RealPriceFeeds {
    dexscreener_enabled: bool,
    jupiter_enabled: bool,
    birdeye_enabled: bool,
    http_client: reqwest::Client,
}

/// Precio real de un token en un DEX específico
#[derive(Debug, Clone)]
pub struct DEXPrice {
    pub dex_name: String,
    pub token_mint: String,
    pub price_usd: f64,
    pub price_sol: Option<f64>,
    pub liquidity_usd: f64,
    pub volume_24h: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub source: String,
}

/// Oportunidad de arbitraje real detectada
#[derive(Debug, Clone)]
pub struct RealArbitrageOpportunity {
    pub id: String,
    pub token_mint: String,
    pub token_symbol: String,
    pub dex_a: DEXPrice,
    pub dex_b: DEXPrice,
    pub price_difference_pct: f64,
    pub estimated_profit_sol: f64,
    pub min_liquidity_usd: f64,
    pub confidence_score: f64,
    pub trade_amount_sol: f64,
}

impl RealPriceFeeds {
    /// Crear nuevo sistema de price feeds reales
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("SniperForge/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            dexscreener_enabled: true,
            jupiter_enabled: true,
            birdeye_enabled: false, // Requiere API key
            http_client,
        }
    }

    /// Detectar oportunidades de arbitraje reales
    pub async fn find_real_arbitrage_opportunities(&self) -> Result<Vec<RealArbitrageOpportunity>> {
        info!("🔍 Buscando oportunidades de arbitraje REALES...");
        
        let mut opportunities = Vec::new();
        
        // 1. TOKENS PRINCIPALES PARA ARBITRAJE
        let target_tokens = vec![
            ("USDC", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            ("RAY", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"),
            ("BONK", "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"),
            ("JUP", "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN"),
        ];

        for (symbol, mint) in target_tokens {
            match self.check_token_arbitrage(symbol, mint).await {
                Ok(token_opportunities) => {
                    opportunities.extend(token_opportunities);
                }
                Err(e) => {
                    warn!("⚠️ Error checking {}: {}", symbol, e);
                }
            }
        }

        info!("✅ Encontradas {} oportunidades reales", opportunities.len());
        Ok(opportunities)
    }

    /// Verificar arbitraje para un token específico
    async fn check_token_arbitrage(&self, symbol: &str, mint: &str) -> Result<Vec<RealArbitrageOpportunity>> {
        debug!("🔍 Checking arbitrage for {} ({})", symbol, mint);
        
        // Obtener precios de múltiples DEXs
        let prices = self.get_multi_dex_prices(mint).await?;
        
        if prices.len() < 2 {
            return Ok(Vec::new()); // No hay suficientes DEXs para arbitraje
        }

        let mut opportunities = Vec::new();

        // Comparar todos los pares de DEXs
        for i in 0..prices.len() {
            for j in (i + 1)..prices.len() {
                let price_a = &prices[i];
                let price_b = &prices[j];

                // Calcular diferencia de precio
                let price_diff_pct = ((price_b.price_usd - price_a.price_usd) / price_a.price_usd).abs() * 100.0;

                // Filtrar por diferencia mínima significativa
                if price_diff_pct > 0.5 { // Mínimo 0.5% diferencia
                    let opportunity = self.create_arbitrage_opportunity(
                        symbol, mint, price_a.clone(), price_b.clone(), price_diff_pct
                    ).await?;

                    if opportunity.confidence_score > 0.6 { // Solo alta confianza
                        opportunities.push(opportunity);
                    }
                }
            }
        }

        Ok(opportunities)
    }

    /// Obtener precios de múltiples DEXs
    async fn get_multi_dex_prices(&self, mint: &str) -> Result<Vec<DEXPrice>> {
        let mut prices = Vec::new();

        // 1. DexScreener (gratuito, múltiples DEXs)
        if self.dexscreener_enabled {
            match self.get_dexscreener_prices(mint).await {
                Ok(dex_prices) => prices.extend(dex_prices),
                Err(e) => warn!("DexScreener error: {}", e),
            }
        }

        // 2. Jupiter Price API (gratuito)
        if self.jupiter_enabled {
            match self.get_jupiter_price(mint).await {
                Ok(jupiter_price) => prices.push(jupiter_price),
                Err(e) => warn!("Jupiter price error: {}", e),
            }
        }

        Ok(prices)
    }

    /// Obtener precios de DexScreener
    async fn get_dexscreener_prices(&self, mint: &str) -> Result<Vec<DEXPrice>> {
        let url = format!("https://api.dexscreener.com/latest/dex/tokens/{}", mint);
        
        let response = timeout(Duration::from_secs(5), 
            self.http_client.get(&url).send()
        ).await??;

        if !response.status().is_success() {
            return Err(anyhow!("DexScreener API error: {}", response.status()));
        }

        let data: Value = response.json().await?;
        let mut prices = Vec::new();

        if let Some(pairs) = data["pairs"].as_array() {
            for pair in pairs.iter().take(5) { // Top 5 pools
                if let (Some(dex_id), Some(price_usd), Some(liquidity)) = (
                    pair["dexId"].as_str(),
                    pair["priceUsd"].as_str().and_then(|s| s.parse::<f64>().ok()),
                    pair["liquidity"]["usd"].as_f64(),
                ) {
                    prices.push(DEXPrice {
                        dex_name: dex_id.to_string(),
                        token_mint: mint.to_string(),
                        price_usd,
                        price_sol: None, // Se calcula después
                        liquidity_usd: liquidity,
                        volume_24h: pair["volume"]["h24"].as_f64().unwrap_or(0.0),
                        last_updated: chrono::Utc::now(),
                        source: "DexScreener".to_string(),
                    });
                }
            }
        }

        debug!("📊 DexScreener: {} prices for {}", prices.len(), mint);
        Ok(prices)
    }

    /// Obtener precio de Jupiter
    async fn get_jupiter_price(&self, mint: &str) -> Result<DEXPrice> {
        let url = format!("https://price.jup.ag/v4/price?ids={}", mint);
        
        let response = timeout(Duration::from_secs(5),
            self.http_client.get(&url).send()
        ).await??;

        if !response.status().is_success() {
            return Err(anyhow!("Jupiter price API error: {}", response.status()));
        }

        let data: Value = response.json().await?;
        
        if let Some(price_data) = data["data"][mint].as_object() {
            let price_usd = price_data["price"].as_f64()
                .ok_or_else(|| anyhow!("Invalid Jupiter price format"))?;

            Ok(DEXPrice {
                dex_name: "Jupiter".to_string(),
                token_mint: mint.to_string(),
                price_usd,
                price_sol: None,
                liquidity_usd: 0.0, // Jupiter agregado, no tiene liquidez específica
                volume_24h: 0.0,
                last_updated: chrono::Utc::now(),
                source: "Jupiter".to_string(),
            })
        } else {
            Err(anyhow!("Token not found in Jupiter price API"))
        }
    }

    /// Crear oportunidad de arbitraje
    async fn create_arbitrage_opportunity(
        &self,
        symbol: &str,
        mint: &str,
        price_a: DEXPrice,
        price_b: DEXPrice,
        price_diff_pct: f64,
    ) -> Result<RealArbitrageOpportunity> {
        
        // Determinar DEX con menor precio (comprar) y mayor precio (vender)
        let (buy_dex, sell_dex) = if price_a.price_usd < price_b.price_usd {
            (price_a, price_b)
        } else {
            (price_b, price_a)
        };

        // Calcular profit estimado (conservador)
        let base_trade_amount = 0.001; // 1 mSOL base
        let estimated_profit_pct = price_diff_pct - 1.0; // Menos 1% de fees/slippage
        let estimated_profit_sol = base_trade_amount * (estimated_profit_pct / 100.0);

        // Calcular confidence score
        let liquidity_score = ((buy_dex.liquidity_usd + sell_dex.liquidity_usd) / 20000.0).min(1.0);
        let volume_score = ((buy_dex.volume_24h + sell_dex.volume_24h) / 100000.0).min(1.0);
        let price_diff_score = (price_diff_pct / 5.0).min(1.0); // 5% = score perfecto
        
        let confidence_score = (liquidity_score + volume_score + price_diff_score) / 3.0;

        Ok(RealArbitrageOpportunity {
            id: format!("REAL_{}_{}_{}_{}", symbol, buy_dex.dex_name, sell_dex.dex_name, chrono::Utc::now().timestamp()),
            token_mint: mint.to_string(),
            token_symbol: symbol.to_string(),
            dex_a: buy_dex,
            dex_b: sell_dex,
            price_difference_pct: price_diff_pct,
            estimated_profit_sol: estimated_profit_sol.max(0.0), // No profits negativos
            min_liquidity_usd: (buy_dex.liquidity_usd).min(sell_dex.liquidity_usd),
            confidence_score,
            trade_amount_sol: base_trade_amount,
        })
    }

    /// Validar oportunidad en tiempo real
    pub async fn validate_opportunity(&self, opportunity: &RealArbitrageOpportunity) -> Result<bool> {
        info!("🔍 Validando oportunidad: {}", opportunity.id);
        
        // Re-obtener precios actualizados
        let current_prices = self.get_multi_dex_prices(&opportunity.token_mint).await?;
        
        if current_prices.len() < 2 {
            return Ok(false);
        }

        // Verificar que la diferencia de precio sigue existiendo
        let current_diff = self.calculate_current_price_difference(&current_prices)?;
        
        // La oportunidad sigue siendo válida si la diferencia es al menos 70% de la original
        let is_valid = current_diff >= (opportunity.price_difference_pct * 0.7);
        
        if is_valid {
            info!("✅ Oportunidad validada: {:.2}% diferencia actual", current_diff);
        } else {
            warn!("❌ Oportunidad invalidada: {:.2}% diferencia actual", current_diff);
        }

        Ok(is_valid)
    }

    /// Calcular diferencia de precio actual
    fn calculate_current_price_difference(&self, prices: &[DEXPrice]) -> Result<f64> {
        if prices.len() < 2 {
            return Ok(0.0);
        }

        let min_price = prices.iter().map(|p| p.price_usd).fold(f64::INFINITY, f64::min);
        let max_price = prices.iter().map(|p| p.price_usd).fold(f64::NEG_INFINITY, f64::max);

        Ok(((max_price - min_price) / min_price) * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dexscreener_integration() {
        let feeds = RealPriceFeeds::new();
        
        // Test con USDC (siempre disponible)
        let result = feeds.get_dexscreener_prices("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").await;
        
        match result {
            Ok(prices) => {
                println!("✅ DexScreener test: {} prices found", prices.len());
                for price in prices.iter().take(3) {
                    println!("   {} - ${:.4} (${:.0}K liquidity)", 
                             price.dex_name, price.price_usd, price.liquidity_usd / 1000.0);
                }
            }
            Err(e) => println!("❌ DexScreener test failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_real_opportunity_detection() {
        let feeds = RealPriceFeeds::new();
        
        let opportunities = feeds.find_real_arbitrage_opportunities().await.unwrap_or_default();
        
        println!("🎯 Found {} real arbitrage opportunities", opportunities.len());
        
        for opp in opportunities.iter().take(3) {
            println!("   {} {}: {:.2}% profit, {:.1}% confidence", 
                     opp.token_symbol, opp.id, 
                     opp.price_difference_pct, opp.confidence_score * 100.0);
        }
    }
}
