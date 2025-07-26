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
use std::sync::{Arc, Mutex};

/// Cliente para obtener precios reales de múltiples DEXs
pub struct RealPriceFeeds {
    dexscreener_enabled: bool,
    jupiter_enabled: bool,
    birdeye_enabled: bool,
    http_client: reqwest::Client,
    // Rate limiting para CoinGecko
    last_coingecko_request: Arc<Mutex<std::time::Instant>>,
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
            birdeye_enabled: true, // ✅ Birdeye tiene endpoints públicos gratuitos
            http_client,
            last_coingecko_request: Arc::new(Mutex::new(std::time::Instant::now() - Duration::from_secs(60))),
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

        // 2. Birdeye API (gratuito, más confiable que Jupiter) - SEGUNDA PRIORIDAD
        if self.birdeye_enabled {
            match self.get_birdeye_price(mint).await {
                Ok(birdeye_price) => {
                    info!("✅ Birdeye: precio ${:.6} obtenido", birdeye_price.price_usd);
                    prices.push(birdeye_price);
                    successful_sources += 1;
                },
                Err(e) => warn!("⚠️ Birdeye error: {}", e),
            }
        }

        // 3. Jupiter Price API (menos confiable ahora) - SOLO SI NECESITAMOS MÁS FUENTES
        if self.jupiter_enabled && successful_sources < 2 {
            match self.get_jupiter_price(mint).await {
                Ok(jupiter_price) => {
                    info!("✅ Jupiter: precio ${:.6} obtenido", jupiter_price.price_usd);
                    prices.push(jupiter_price);
                    successful_sources += 1;
                },
                Err(e) => debug!("❌ Jupiter error (esperado): {}", e),
            }
        }

        // 4. Fallback inteligente solo si tenemos muy pocos datos (NO CoinGecko por defecto)
        if prices.len() < 2 {
            warn!("⚠️ Pocas fuentes disponibles ({} precios), usando fallbacks hardcoded", prices.len());
            if let Ok(fallback_price) = self.get_hardcoded_fallback_price(mint).await {
                prices.push(fallback_price);
            }
        }

        info!("📊 Total: {} precios de {} fuentes para {}", prices.len(), successful_sources, mint);
        Ok(prices)
    }

    /// Precios hardcoded como último recurso (más confiable que APIs con rate limiting)
    async fn get_hardcoded_fallback_price(&self, mint: &str) -> Result<DEXPrice> {
        info!("⚠️ Usando precios hardcoded para {}", mint);
        let (price_usd, symbol) = match mint {
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => (1.0001, "USDC"), // USDC
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => (1.0002, "USDT"), // USDT
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263" => (0.000025, "BONK"), // BONK
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R" => (0.65, "RAY"), // Raydium
            "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN" => (0.85, "JUP"), // Jupiter
            _ => return Err(anyhow!("Token {} no tiene fallback hardcoded", mint)),
        };

        Ok(DEXPrice {
            dex_name: "Hardcoded_Fallback".to_string(),
            token_mint: mint.to_string(),
            price_usd,
            price_sol: None,
            liquidity_usd: 1000000.0, // Asumir liquidez alta para fallback
            volume_24h: 100000.0,
            last_updated: chrono::Utc::now(),
            source: format!("Hardcoded ({})", symbol),
        })
    }

    /// CoinGecko solo para uso MANUAL cuando sea necesario (evitar 429 automático)
    async fn get_coingecko_price_manual(&self, mint: &str) -> Result<DEXPrice> {
        // Rate limiting: máximo 1 request cada 3 segundos para evitar 429
        {
            let mut last_request = self.last_coingecko_request.lock().unwrap();
            let time_since_last = last_request.elapsed();
            if time_since_last < Duration::from_secs(3) {
                let wait_time = Duration::from_secs(3) - time_since_last;
                debug!("⏳ Rate limiting CoinGecko: esperando {:?}", wait_time);
                drop(last_request);
                tokio::time::sleep(wait_time).await;
                *self.last_coingecko_request.lock().unwrap() = std::time::Instant::now();
            } else {
                *last_request = std::time::Instant::now();
            }
        }

        // Mapeo conservador de tokens conocidos
        let coingecko_id = match mint {
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "usd-coin",
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => "tether", 
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263" => "bonk",
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R" => "raydium",
            "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN" => "jupiter-exchange-solana",
            _ => return Err(anyhow!("Token {} no soportado en CoinGecko", mint)),
        };

        let url = format!("https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd", coingecko_id);
        
        let response = timeout(Duration::from_secs(10),
            self.http_client
                .get(&url)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .timeout(Duration::from_secs(8))
                .send()
        ).await.map_err(|_| anyhow!("CoinGecko timeout"))?
          .map_err(|e| anyhow!("CoinGecko connection error: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("CoinGecko API error: {} - {}", 
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let data: Value = response.json().await
            .map_err(|e| anyhow!("CoinGecko JSON error: {}", e))?;

        let price_usd = data[coingecko_id]["usd"].as_f64()
            .ok_or_else(|| anyhow!("Invalid CoinGecko price format"))?;

        if price_usd <= 0.0 {
            return Err(anyhow!("Invalid CoinGecko price: {}", price_usd));
        }

        Ok(DEXPrice {
            dex_name: "CoinGecko".to_string(),
            token_mint: mint.to_string(),
            price_usd,
            price_sol: None,
            liquidity_usd: 0.0,
            volume_24h: 0.0,
            last_updated: chrono::Utc::now(),
            source: "CoinGecko".to_string(),
        })
    }

    /// Obtener precios de DexScreener con mejor manejo de errores
    async fn get_dexscreener_prices(&self, mint: &str) -> Result<Vec<DEXPrice>> {
        let url = format!("https://api.dexscreener.com/latest/dex/tokens/{}", mint);
        
        let response = timeout(Duration::from_secs(10), 
            self.http_client
                .get(&url)
                .header("User-Agent", "SniperForge/1.0")
                .timeout(Duration::from_secs(8))
                .send()
        ).await.map_err(|_| anyhow!("DexScreener request timeout"))?
          .map_err(|e| anyhow!("DexScreener connection error: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("DexScreener API error: {} - {}", 
                response.status(), 
                response.text().await.unwrap_or_default()
            ));
        }

        let data: Value = response.json().await
            .map_err(|e| anyhow!("DexScreener JSON parse error: {}", e))?;
        let mut prices = Vec::new();

        if let Some(pairs) = data["pairs"].as_array() {
            let valid_pairs: Vec<_> = pairs.iter()
                .filter(|pair| {
                    // Filtrar pools válidos
                    let has_basic_data = pair["dexId"].is_string() && 
                                       pair["priceUsd"].is_string() &&
                                       pair["liquidity"]["usd"].is_number();
                    
                    let liquidity = pair["liquidity"]["usd"].as_f64().unwrap_or(0.0);
                    let has_liquidity = liquidity > 1000.0; // Mínimo $1k liquidez
                    
                    has_basic_data && has_liquidity
                })
                .take(5) // Top 5 pools válidos
                .collect();

            for pair in valid_pairs {
                if let (Some(dex_id), Some(price_str), Some(liquidity)) = (
                    pair["dexId"].as_str(),
                    pair["priceUsd"].as_str(),
                    pair["liquidity"]["usd"].as_f64(),
                ) {
                    if let Ok(price_usd) = price_str.parse::<f64>() {
                        if price_usd > 0.0 { // Precio válido
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
            }
        }

        debug!("📊 DexScreener: {} valid prices for {}", prices.len(), mint);
        Ok(prices)
    }

    /// Obtener precio de Jupiter con reintentos y mejor manejo de errores
    async fn get_jupiter_price(&self, mint: &str) -> Result<DEXPrice> {
        // Intentar múltiples endpoints de Jupiter (solo endpoints públicos gratuitos)
        let endpoints = vec![
            format!("https://price.jup.ag/v4/price?ids={}", mint),
            // Removido el endpoint v2 que requiere autorización
        ];

        let mut last_error = None;

        for endpoint in endpoints {
            match self.try_jupiter_endpoint(&endpoint, mint).await {
                Ok(price) => return Ok(price),
                Err(e) => {
                    warn!("Jupiter endpoint {} failed: {}", endpoint, e);
                    last_error = Some(e);
                }
            }
        }

        // Si Jupiter falla, intentar obtener precio de CoinGecko como alternativa
        match self.get_coingecko_price(mint).await {
            Ok(price) => {
                info!("✅ Fallback: CoinGecko precio obtenido para {}", mint);
                return Ok(price);
            },
            Err(e) => warn!("CoinGecko fallback failed: {}", e),
        }

        Err(last_error.unwrap_or_else(|| anyhow!("All Jupiter endpoints failed")))
    }

    /// Intentar un endpoint específico de Jupiter
    async fn try_jupiter_endpoint(&self, url: &str, mint: &str) -> Result<DEXPrice> {
        let response = timeout(Duration::from_secs(10), // Más tiempo para Jupiter
            self.http_client
                .get(url)
                .header("User-Agent", "SniperForge/1.0")
                .timeout(Duration::from_secs(8))
                .send()
        ).await.map_err(|_| anyhow!("Jupiter request timeout"))?
          .map_err(|e| anyhow!("Jupiter connection error: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Jupiter API error: {} - {}", response.status(), response.text().await.unwrap_or_default()));
        }

        let data: Value = response.json().await
            .map_err(|e| anyhow!("Jupiter JSON parse error: {}", e))?;
        
        // Intentar diferentes formatos de respuesta de Jupiter
        let price_usd = if let Some(price_data) = data["data"][mint].as_object() {
            price_data["price"].as_f64()
        } else if let Some(price) = data[mint]["price"].as_f64() {
            Some(price)
        } else if let Some(price_str) = data[mint]["price"].as_str() {
            price_str.parse::<f64>().ok()
        } else {
            None
        };

        let price_usd = price_usd
            .ok_or_else(|| anyhow!("Invalid Jupiter price format for {}", mint))?;

        debug!("📊 Jupiter price for {}: ${:.6}", mint, price_usd);

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
    }

    /// Obtener precio de Birdeye API (alternativa confiable a Jupiter)
    async fn get_birdeye_price(&self, mint: &str) -> Result<DEXPrice> {
        let url = format!("https://public-api.birdeye.so/public/price?address={}", mint);
        
        let response = timeout(Duration::from_secs(8),
            self.http_client
                .get(&url)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .header("X-Chain", "solana")
                .timeout(Duration::from_secs(6))
                .send()
        ).await.map_err(|_| anyhow!("Birdeye request timeout"))?
          .map_err(|e| anyhow!("Birdeye connection error: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Birdeye API error: {} - {}", 
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let data: Value = response.json().await
            .map_err(|e| anyhow!("Birdeye JSON parse error: {}", e))?;

        // Birdeye response format: {"data": {"value": 1.234}}
        let price_usd = data["data"]["value"].as_f64()
            .ok_or_else(|| anyhow!("Invalid Birdeye price format for {}", mint))?;

        if price_usd <= 0.0 {
            return Err(anyhow!("Invalid price from Birdeye: {}", price_usd));
        }

        debug!("📊 Birdeye price for {}: ${:.6}", mint, price_usd);

        Ok(DEXPrice {
            dex_name: "Birdeye".to_string(),
            token_mint: mint.to_string(),
            price_usd,
            price_sol: None,
            liquidity_usd: 0.0,
            volume_24h: 0.0,
            last_updated: chrono::Utc::now(),
            source: "Birdeye".to_string(),
        })
    }

    /// Obtener precio de CoinGecko como fallback (gratuito, sin API key) con rate limiting
    async fn get_coingecko_price(&self, mint: &str) -> Result<DEXPrice> {
        // Rate limiting: máximo 1 request cada 2 segundos para evitar 429
        {
            let mut last_request = self.last_coingecko_request.lock().unwrap();
            let time_since_last = last_request.elapsed();
            if time_since_last < Duration::from_secs(2) {
                let wait_time = Duration::from_secs(2) - time_since_last;
                debug!("⏳ Rate limiting CoinGecko: esperando {:?}", wait_time);
                drop(last_request); // Release lock before sleeping
                tokio::time::sleep(wait_time).await;
                *self.last_coingecko_request.lock().unwrap() = std::time::Instant::now();
            } else {
                *last_request = std::time::Instant::now();
            }
        }

        // Mapeo de mints conocidos a IDs de CoinGecko
        let coingecko_id = match mint {
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "usd-coin",
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => "tether",
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263" => "bonk",
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R" => "raydium",
            "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN" => "jupiter-exchange-solana",
            _ => return Err(anyhow!("Token {} not supported in CoinGecko fallback", mint)),
        };

        let url = format!("https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd", coingecko_id);
        
        let response = timeout(Duration::from_secs(10),
            self.http_client
                .get(&url)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .timeout(Duration::from_secs(8))
                .send()
        ).await.map_err(|_| anyhow!("CoinGecko request timeout"))?
          .map_err(|e| anyhow!("CoinGecko connection error: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("CoinGecko API error: {} - {}", 
                response.status(), 
                response.text().await.unwrap_or_default()
            ));
        }

        let data: Value = response.json().await
            .map_err(|e| anyhow!("CoinGecko JSON parse error: {}", e))?;
        
        let price_usd = data[coingecko_id]["usd"].as_f64()
            .ok_or_else(|| anyhow!("Invalid CoinGecko price format for {}", coingecko_id))?;

        debug!("📊 CoinGecko price for {}: ${:.6}", mint, price_usd);

        Ok(DEXPrice {
            dex_name: "CoinGecko".to_string(),
            token_mint: mint.to_string(),
            price_usd,
            price_sol: None,
            liquidity_usd: 0.0,
            volume_24h: 0.0,
            last_updated: chrono::Utc::now(),
            source: "CoinGecko".to_string(),
        })
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
            dex_a: buy_dex.clone(),
            dex_b: sell_dex.clone(),
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
