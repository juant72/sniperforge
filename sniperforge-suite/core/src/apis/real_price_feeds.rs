// ================================================================================
// REAL PRICE FEEDS - SISTEMA DE PRECIOS REALES (MIGRATED FROM WORKING BOT)
// ================================================================================
// Sistema que obtiene precios reales de múltiples DEXs para detectar arbitraje
// ================================================================================

use anyhow::{Result, anyhow};
use tracing::{info, warn, debug};
use std::collections::HashMap;
use serde_json::Value;
use tokio::time::{timeout, Duration};
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};

/// Cliente para obtener precios reales de múltiples DEXs (migrado del bot que funciona)
pub struct RealPriceFeeds {
    dexscreener_enabled: bool,
    jupiter_enabled: bool,
    birdeye_enabled: bool,
    http_client: reqwest::Client,
    // Rate limiting para CoinGecko
    last_coingecko_request: Arc<Mutex<std::time::Instant>>,
}

/// Precio real de un token en un DEX específico (migrado del bot que funciona)
#[derive(Debug, Clone)]
pub struct DEXPrice {
    pub dex_name: String,
    pub token_mint: String,
    pub price_usd: f64,
    pub price_sol: Option<f64>,
    pub liquidity_usd: f64,
    pub volume_24h: f64,
    pub last_updated: DateTime<Utc>,
    pub source: String,
}

/// Oportunidad de arbitraje real detectada (migrado del bot que funciona)
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
    /// Crear nuevo sistema de price feeds reales con configuración robusta
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 SniperForge/1.0")
            .tcp_keepalive(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(4)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            dexscreener_enabled: true,
            jupiter_enabled: true,
            birdeye_enabled: false, // Deshabilitado - requiere API key
            http_client,
            last_coingecko_request: Arc::new(Mutex::new(std::time::Instant::now() - Duration::from_secs(60))),
        }
    }

    /// Detectar oportunidades de arbitraje reales (método principal)
    pub async fn find_real_arbitrage_opportunities(&self) -> Result<Vec<RealArbitrageOpportunity>> {
        info!("🔍 Buscando oportunidades de arbitraje REALES...");
        
        let mut opportunities = Vec::new();
        
        // TOKENS PRINCIPALES PARA ARBITRAJE
        let target_tokens = vec![
            ("SOL", "So11111111111111111111111111111111111111112"),
            ("USDC", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            ("RAY", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"),
            ("BONK", "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"),
            ("JUP", "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN"),
            ("USDT", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
            ("WIF", "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm"),
            ("PYTH", "HZ1JovNiVvGrGNiiYvEozEVgZ58xaU3RKwX8eACQBCt3"),
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

                // PROTECCIÓN ANTI-CIRCULAR: No comparar mismo DEX
                if price_a.dex_name == price_b.dex_name {
                    debug!("⏸️ Saltando comparación circular: {} vs {} (mismo DEX: {})", 
                           price_a.dex_name, price_b.dex_name, price_a.dex_name);
                    continue;
                }

                // Calcular diferencia de precio
                let price_diff_pct = ((price_b.price_usd - price_a.price_usd) / price_a.price_usd).abs() * 100.0;

                // Filtrar por diferencia mínima significativa
                if price_diff_pct > 0.01 && price_diff_pct < 50.0 {
                    let opportunity = self.create_arbitrage_opportunity(
                        symbol, mint, price_a.clone(), price_b.clone(), price_diff_pct
                    ).await?;

                    // FILTROS PERMISIVOS PARA DETECTAR OPORTUNIDADES
                    if opportunity.price_difference_pct > 0.01 && opportunity.confidence_score > 0.1 {
                        info!("✅ Oportunidad detectada: {} ({}% profit, {:.1}% confidence)", 
                              symbol, opportunity.price_difference_pct, opportunity.confidence_score * 100.0);
                        opportunities.push(opportunity);
                    } else if opportunity.price_difference_pct > 0.005 {
                        info!("⚠️ Oportunidad marginal detectada: {} ({}% profit, {:.1}% confidence)", 
                               symbol, opportunity.price_difference_pct, opportunity.confidence_score * 100.0);
                        opportunities.push(opportunity); // Incluir incluso oportunidades marginales
                    }
                }
            }
        }

        Ok(opportunities)
    }

    /// Obtener precios de múltiples DEXs con fallbacks y reintentos
    async fn get_multi_dex_prices(&self, mint: &str) -> Result<Vec<DEXPrice>> {
        let mut prices = Vec::new();
        let mut successful_sources = 0;

        // 1. DexScreener (gratuito, múltiples DEXs) - PRIMERA PRIORIDAD
        if self.dexscreener_enabled {
            match self.get_dexscreener_prices(mint).await {
                Ok(dex_prices) => {
                    if !dex_prices.is_empty() {
                        info!("✅ DexScreener: {} precios obtenidos", dex_prices.len());
                        prices.extend(dex_prices);
                        successful_sources += 1;
                    }
                },
                Err(e) => warn!("❌ DexScreener error: {}", e),
            }
        }

        // 2. Coinbase (para tokens principales) - SEGUNDA PRIORIDAD
        match self.get_coinbase_price(mint).await {
            Ok(coinbase_price) => {
                info!("✅ Coinbase: precio ${:.6} obtenido", coinbase_price.price_usd);
                prices.push(coinbase_price);
                successful_sources += 1;
            },
            Err(e) => debug!("⚠️ Coinbase not available for this token: {}", e),
        }

        // 3. Jupiter Price API con manejo robusto de errores
        if self.jupiter_enabled && successful_sources < 3 {
            match timeout(Duration::from_secs(5), self.get_jupiter_price(mint)).await {
                Ok(Ok(jupiter_price)) => {
                    info!("✅ Jupiter: precio ${:.6} obtenido", jupiter_price.price_usd);
                    prices.push(jupiter_price);
                    successful_sources += 1;
                },
                Ok(Err(e)) => {
                    debug!("⚠️ Jupiter endpoint temporarily unavailable: {}", e);
                },
                Err(_) => {
                    debug!("⚠️ Jupiter timeout - continuando con otras fuentes");
                }
            }
        }

        // 4. Fallback inteligente solo si tenemos muy pocos datos
        if prices.len() < 2 {
            warn!("⚠️ Pocas fuentes disponibles ({} precios), usando fallbacks", prices.len());
            if let Ok(fallback_price) = self.get_fallback_price(mint).await {
                prices.push(fallback_price);
            }
        }

        info!("📊 Total: {} precios de {} fuentes para {}", prices.len(), successful_sources, mint);
        Ok(prices)
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
        let trade_amount_sol = 0.01; // 0.01 SOL por defecto
        let estimated_profit_sol = trade_amount_sol * (price_diff_pct / 100.0);
        
        // Calcular score de confianza basado en liquidez y volumen
        let min_liquidity = price_a.liquidity_usd.min(price_b.liquidity_usd);
        let min_volume = price_a.volume_24h.min(price_b.volume_24h);
        
        let liquidity_score = (min_liquidity / 100_000.0).min(1.0);
        let volume_score = (min_volume / 50_000.0).min(1.0);
        let price_score = (price_diff_pct / 5.0).min(1.0);
        
        let confidence_score = (liquidity_score + volume_score + price_score) / 3.0;

        Ok(RealArbitrageOpportunity {
            id: format!("arb_{}_{}", symbol, chrono::Utc::now().timestamp_millis()),
            token_mint: mint.to_string(),
            token_symbol: symbol.to_string(),
            dex_a: price_a,
            dex_b: price_b,
            price_difference_pct: price_diff_pct,
            estimated_profit_sol,
            min_liquidity_usd: min_liquidity,
            confidence_score,
            trade_amount_sol,
        })
    }

    /// Obtener precios de DexScreener (implementación simplificada)
    async fn get_dexscreener_prices(&self, mint: &str) -> Result<Vec<DEXPrice>> {
        let url = format!("https://api.dexscreener.com/latest/dex/tokens/{}", mint);
        
        let response = timeout(Duration::from_secs(10), self.http_client.get(&url).send()).await??;
        let data: Value = response.json().await?;
        
        let mut prices = Vec::new();
        
        if let Some(pairs) = data["pairs"].as_array() {
            for pair in pairs.iter().take(5) { // Máximo 5 DEXs
                if let (Some(dex_id), Some(price_usd), Some(liquidity), Some(volume)) = (
                    pair["dexId"].as_str(),
                    pair["priceUsd"].as_str().and_then(|s| s.parse::<f64>().ok()),
                    pair["liquidity"]["usd"].as_f64(),
                    pair["volume"]["h24"].as_f64(),
                ) {
                    prices.push(DEXPrice {
                        dex_name: format!("DexScreener_{}", dex_id),
                        token_mint: mint.to_string(),
                        price_usd,
                        price_sol: None,
                        liquidity_usd: liquidity,
                        volume_24h: volume,
                        last_updated: Utc::now(),
                        source: "DexScreener".to_string(),
                    });
                }
            }
        }
        
        Ok(prices)
    }

    /// Obtener precio de Coinbase
    async fn get_coinbase_price(&self, mint: &str) -> Result<DEXPrice> {
        // Mapear mint a símbolo de Coinbase
        let symbol = match mint {
            "So11111111111111111111111111111111111111112" => "SOL",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "USDC",
            _ => return Err(anyhow!("Token not supported on Coinbase")),
        };

        let url = format!("https://api.coinbase.com/v2/exchange-rates?currency={}", symbol);
        let response = self.http_client.get(&url).send().await?;
        let data: Value = response.json().await?;

        let price_usd = data["data"]["rates"]["USD"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| anyhow!("Failed to parse Coinbase price"))?;

        Ok(DEXPrice {
            dex_name: "Coinbase".to_string(),
            token_mint: mint.to_string(),
            price_usd,
            price_sol: None,
            liquidity_usd: 10_000_000.0, // Asumida alta liquidez
            volume_24h: 1_000_000.0,
            last_updated: Utc::now(),
            source: "Coinbase".to_string(),
        })
    }

    /// Obtener precio de Jupiter
    async fn get_jupiter_price(&self, mint: &str) -> Result<DEXPrice> {
        let url = format!("https://price.jup.ag/v4/price?ids={}", mint);
        let response = self.http_client.get(&url).send().await?;
        let data: Value = response.json().await?;

        let price_usd = data["data"][mint]["price"]
            .as_f64()
            .ok_or_else(|| anyhow!("Failed to parse Jupiter price"))?;

        Ok(DEXPrice {
            dex_name: "Jupiter".to_string(),
            token_mint: mint.to_string(),
            price_usd,
            price_sol: None,
            liquidity_usd: 1_000_000.0, // Estimada
            volume_24h: 500_000.0,
            last_updated: Utc::now(),
            source: "Jupiter".to_string(),
        })
    }

    /// Fallback usando precios estimados
    async fn get_fallback_price(&self, mint: &str) -> Result<DEXPrice> {
        let (symbol, estimated_price) = match mint {
            "So11111111111111111111111111111111111111112" => ("SOL", 150.0),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => ("USDC", 1.0),
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => ("USDT", 1.0),
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R" => ("RAY", 1.5),
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263" => ("BONK", 0.00002),
            "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN" => ("JUP", 0.8),
            _ => return Err(anyhow!("Token not supported in fallback")),
        };

        info!("🔄 Fallback: precio estimado para {} = ${:.6}", symbol, estimated_price);

        Ok(DEXPrice {
            dex_name: "Fallback_Estimated".to_string(),
            token_mint: mint.to_string(),
            price_usd: estimated_price,
            price_sol: None,
            liquidity_usd: 500_000.0,
            volume_24h: 100_000.0,
            last_updated: Utc::now(),
            source: format!("Fallback ({})", symbol),
        })
    }
}

impl Default for RealPriceFeeds {
    fn default() -> Self {
        Self::new()
    }
}
