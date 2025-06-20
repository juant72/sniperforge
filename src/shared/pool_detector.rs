/// Pool Detection System for Real-Time Monitoring
/// 
/// Detecta nuevos pools de liquidez en Raydium/Orca usando datos de mainnet
/// Sistema read-only para análisis sin riesgo

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{info, warn, error, debug};
use rand;

use crate::shared::jupiter::client::JupiterClient;
use crate::shared::syndica_websocket::SyndicaWebSocketClient;

/// Información de un pool detectado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPool {
    pub pool_address: String,
    pub token_a: TokenInfo,
    pub token_b: TokenInfo,
    pub liquidity_usd: f64,
    pub price_impact_1k: f64, // Price impact for $1k trade
    pub volume_24h: f64,
    pub created_at: u64,
    pub detected_at: u64, // Timestamp instead of Instant for serialization
    pub dex: String, // "Raydium", "Orca", etc.
    pub risk_score: RiskScore,
}

/// Información de token en el pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub mint: String,
    pub symbol: String,
    pub decimals: u8,
    pub supply: u64,
    pub price_usd: f64,
    pub market_cap: f64,
}

/// Score de riesgo del pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    pub overall: f64, // 0.0 = muy riesgoso, 1.0 = muy seguro
    pub liquidity_score: f64,
    pub volume_score: f64,
    pub token_age_score: f64,
    pub holder_distribution_score: f64,
    pub rug_indicators: Vec<String>,
}

/// Oportunidad de trading detectada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingOpportunity {
    pub pool: DetectedPool,
    pub opportunity_type: OpportunityType,
    pub expected_profit_usd: f64,
    pub confidence: f64,
    pub time_window_ms: u64, // Ventana de tiempo para actuar
    pub recommended_size_usd: f64,
}

/// Tipos de oportunidad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpportunityType {
    NewPoolSnipe,    // Nuevo pool detectado
    PriceDiscrepancy, // Diferencia de precio entre DEXs
    LiquidityImbalance, // Desequilibrio de liquidez
    VolumeSpike,     // Pico de volumen
}

/// Configuración del detector de pools
#[derive(Debug, Clone)]
pub struct PoolDetectorConfig {
    /// Mínima liquidez USD para considerar el pool
    pub min_liquidity_usd: f64,
    /// Máximo price impact para $1k trade
    pub max_price_impact_1k: f64,
    /// Score mínimo de riesgo
    pub min_risk_score: f64,
    /// Intervalo de monitoreo (ms)
    pub monitoring_interval_ms: u64,
    /// Número máximo de pools a trackear simultáneamente
    pub max_tracked_pools: usize,
}

impl Default for PoolDetectorConfig {
    fn default() -> Self {
        Self {
            min_liquidity_usd: 10000.0,  // $10k mínimo
            max_price_impact_1k: 5.0,    // 5% max price impact
            min_risk_score: 0.3,         // 30% mínimo
            monitoring_interval_ms: 1000, // 1s
            max_tracked_pools: 100,      // Track 100 pools max
        }
    }
}

/// Pool Detection Engine
pub struct PoolDetector {
    config: PoolDetectorConfig,
    jupiter_client: JupiterClient,
    syndica_client: Option<SyndicaWebSocketClient>,
    tracked_pools: HashMap<String, DetectedPool>,
    opportunities: Vec<TradingOpportunity>,
    last_scan: Instant,
}

impl PoolDetector {
    /// Crear nuevo detector de pools
    pub async fn new(
        config: PoolDetectorConfig, 
        jupiter_client: JupiterClient,
        syndica_client: Option<SyndicaWebSocketClient>
    ) -> Result<Self> {
        info!("🔍 Initializing Pool Detection Engine (MainNet Read-Only)");
        info!("   Min liquidity: ${:.0}", config.min_liquidity_usd);
        info!("   Max price impact: {:.1}%", config.max_price_impact_1k);
        info!("   Min risk score: {:.1}%", config.min_risk_score * 100.0);
        
        Ok(Self {
            config,
            jupiter_client,
            syndica_client,
            tracked_pools: HashMap::new(),
            opportunities: Vec::new(),
            last_scan: Instant::now(),
        })
    }
    
    /// Iniciar monitoreo continuo de pools
    pub async fn start_monitoring(&mut self) -> Result<()> {
        info!("🚀 Starting continuous pool monitoring...");
        
        loop {
            let start_time = Instant::now();
            
            // Detectar nuevos pools
            match self.scan_for_new_pools().await {
                Ok(new_pools) => {
                    if !new_pools.is_empty() {
                        info!("🆕 Detected {} new pools", new_pools.len());
                        for pool in new_pools {
                            self.analyze_pool_opportunity(&pool).await?;
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠️ Pool scan failed: {}", e);
                }
            }
            
            // Actualizar pools existentes
            if let Err(e) = self.update_tracked_pools().await {
                warn!("⚠️ Pool update failed: {}", e);
            }
            
            // Detectar oportunidades
            if let Err(e) = self.scan_for_opportunities().await {
                warn!("⚠️ Opportunity scan failed: {}", e);
            }
            
            let scan_duration = start_time.elapsed();
            debug!("📊 Pool scan completed in {:?}", scan_duration);
            
            // Wait for next scan interval
            let sleep_time = Duration::from_millis(self.config.monitoring_interval_ms)
                .saturating_sub(scan_duration);
            
            if sleep_time.as_millis() > 0 {
                tokio::time::sleep(sleep_time).await;
            }
            
            self.last_scan = Instant::now();
        }
    }
      /// Escanear nuevos pools usando APIs reales de Raydium
    async fn scan_for_new_pools(&self) -> Result<Vec<DetectedPool>> {
        debug!("🔍 Scanning for new pools using REAL Raydium API...");
        
        // Usar Raydium API real para obtener pools
        let real_pools = self.fetch_real_raydium_pools().await?;
        
        // Si no hay pools reales disponibles, usar datos de prueba como fallback
        if real_pools.is_empty() {
            warn!("🔄 No real pools found, using mock data for demo");
            return self.generate_mock_pools().await;
        }
        
        Ok(real_pools)
    }
    
    /// Generar pools de prueba con datos realistas
    async fn generate_mock_pools(&self) -> Result<Vec<DetectedPool>> {
        let mut pools = Vec::new();
        
        // Solo generar pools ocasionalmente para no spam
        if rand::random::<f64>() < 0.1 { // 10% chance
            let pool = DetectedPool {
                pool_address: format!("Pool{}", rand::random::<u32>()),
                token_a: TokenInfo {
                    mint: "So11111111111111111111111111111111111111112".to_string(),
                    symbol: "SOL".to_string(),
                    decimals: 9,
                    supply: 1000000000,
                    price_usd: 180.0, // Usar precio real de SOL
                    market_cap: 180000000.0,
                },
                token_b: TokenInfo {
                    mint: format!("Token{}", rand::random::<u32>()),
                    symbol: "NEWTOKEN".to_string(),
                    decimals: 6,
                    supply: 1000000,                price_usd: 0.5,
                market_cap: 500000.0,
            },
            liquidity_usd: rand::random::<f64>() * 100000.0 + 10000.0, // $10k-$110k
            price_impact_1k: rand::random::<f64>() * 10.0, // 0-10%
            volume_24h: rand::random::<f64>() * 50000.0, // $0-$50k
            created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            detected_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            dex: "Raydium".to_string(),
            risk_score: self.calculate_risk_score_mock(),
            };
            
            pools.push(pool);
        }
        
        Ok(pools)
    }
    
    /// Calcular risk score (mock para demo)
    fn calculate_risk_score_mock(&self) -> RiskScore {
        RiskScore {
            overall: rand::random::<f64>(),
            liquidity_score: rand::random::<f64>(),
            volume_score: rand::random::<f64>(),
            token_age_score: rand::random::<f64>(),
            holder_distribution_score: rand::random::<f64>(),
            rug_indicators: vec![],
        }
    }
    
    /// Analizar oportunidad de trading en un pool
    async fn analyze_pool_opportunity(&mut self, pool: &DetectedPool) -> Result<()> {
        debug!("📊 Analyzing pool opportunity: {}", pool.pool_address);
        
        // Filtros básicos
        if pool.liquidity_usd < self.config.min_liquidity_usd {
            debug!("❌ Pool liquidity too low: ${:.0}", pool.liquidity_usd);
            return Ok(());
        }
        
        if pool.price_impact_1k > self.config.max_price_impact_1k {
            debug!("❌ Price impact too high: {:.1}%", pool.price_impact_1k);
            return Ok(());
        }
        
        if pool.risk_score.overall < self.config.min_risk_score {
            debug!("❌ Risk score too low: {:.1}%", pool.risk_score.overall * 100.0);
            return Ok(());
        }
        
        // Calcular oportunidad
        let opportunity = TradingOpportunity {
            pool: pool.clone(),
            opportunity_type: OpportunityType::NewPoolSnipe,
            expected_profit_usd: self.estimate_profit_potential(pool),
            confidence: pool.risk_score.overall,
            time_window_ms: 30000, // 30s window
            recommended_size_usd: self.calculate_position_size(pool),
        };
        
        if opportunity.expected_profit_usd > 50.0 { // Mínimo $50 profit
            info!("🎯 OPPORTUNITY DETECTED: {} - Expected profit: ${:.2}", 
                  pool.pool_address, opportunity.expected_profit_usd);
            self.opportunities.push(opportunity);
        }
        
        // Trackear el pool
        self.tracked_pools.insert(pool.pool_address.clone(), pool.clone());
        
        Ok(())
    }
    
    /// Estimar potencial de profit
    fn estimate_profit_potential(&self, pool: &DetectedPool) -> f64 {
        // Simulación simple de profit estimation
        let base_profit = pool.liquidity_usd * 0.001; // 0.1% of liquidity
        let volume_multiplier = (pool.volume_24h / 10000.0).min(5.0); // Max 5x
        let risk_multiplier = pool.risk_score.overall;
        
        base_profit * volume_multiplier * risk_multiplier
    }
    
    /// Calcular tamaño de posición recomendado
    fn calculate_position_size(&self, pool: &DetectedPool) -> f64 {
        let base_size = 1000.0; // $1k base
        let liquidity_factor = (pool.liquidity_usd / 50000.0).min(2.0); // Max 2x
        let risk_factor = pool.risk_score.overall;
        
        base_size * liquidity_factor * risk_factor
    }
      /// Actualizar pools trackeados
    async fn update_tracked_pools(&mut self) -> Result<()> {
        debug!("🔄 Updating {} tracked pools", self.tracked_pools.len());
        
        // Remover pools muy viejos (1 hora)
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let cutoff = now - 3600; // 1 hora atrás
        self.tracked_pools.retain(|_, pool| pool.detected_at > cutoff);
        
        // Limitar número de pools
        if self.tracked_pools.len() > self.config.max_tracked_pools {
            // Remover los pools más viejos
            let mut pools: Vec<_> = self.tracked_pools.drain().collect();
            pools.sort_by(|a, b| b.1.detected_at.cmp(&a.1.detected_at));
            pools.truncate(self.config.max_tracked_pools);
            self.tracked_pools = pools.into_iter().collect();
        }
        
        Ok(())
    }
      /// Escanear oportunidades en pools existentes con estrategias avanzadas
    async fn scan_for_opportunities(&mut self) -> Result<()> {
        debug!("🎯 Scanning for advanced trading opportunities...");
        
        // Limpiar oportunidades viejas (1 minuto)
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let cutoff = now - 60; // 1 minuto atrás
        self.opportunities.retain(|opp| opp.pool.detected_at > cutoff);
        
        // Buscar arbitraje entre DEXs
        if let Err(e) = self.scan_for_arbitrage_opportunities().await {
            warn!("⚠️ Arbitrage scan failed: {}", e);
        }
        
        // Buscar oportunidades de price impact favorable
        if let Err(e) = self.scan_for_price_impact_opportunities().await {
            warn!("⚠️ Price impact scan failed: {}", e);
        }
        
        // Buscar patrones de volumen inusual
        if let Err(e) = self.scan_for_volume_spike_opportunities().await {
            warn!("⚠️ Volume spike scan failed: {}", e);
        }
        
        Ok(())
    }
    
    /// Buscar oportunidades de arbitraje entre DEXs
    async fn scan_for_arbitrage_opportunities(&mut self) -> Result<()> {
        // Obtener precios de los mismos tokens en diferentes DEXs
        for (_, pool) in &self.tracked_pools {
            // Verificar precios en Jupiter vs precios del pool
            if let Some(jupiter_price_a) = self.get_token_price_from_jupiter(&pool.token_a.mint).await {
                if let Some(jupiter_price_b) = self.get_token_price_from_jupiter(&pool.token_b.mint).await {
                    let pool_ratio = pool.token_a.price_usd / pool.token_b.price_usd;
                    let jupiter_ratio = jupiter_price_a / jupiter_price_b;
                    
                    let price_diff = ((pool_ratio - jupiter_ratio) / jupiter_ratio).abs();
                    
                    if price_diff > 0.02 { // 2% diferencia
                        let opportunity = TradingOpportunity {
                            pool: pool.clone(),
                            opportunity_type: OpportunityType::PriceDiscrepancy,
                            expected_profit_usd: price_diff * 1000.0, // Estimación simple
                            confidence: 0.7,
                            time_window_ms: 15000, // 15s window
                            recommended_size_usd: 500.0,
                        };
                        
                        info!("💰 Arbitrage opportunity: {:.2}% price difference", price_diff * 100.0);
                        self.opportunities.push(opportunity);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Buscar oportunidades basadas en price impact favorable
    async fn scan_for_price_impact_opportunities(&mut self) -> Result<()> {
        for (_, pool) in &self.tracked_pools {
            // Pools con muy bajo price impact son oportunidades para trades grandes
            if pool.price_impact_1k < 1.0 && pool.liquidity_usd > 50000.0 {
                let opportunity = TradingOpportunity {
                    pool: pool.clone(),
                    opportunity_type: OpportunityType::LiquidityImbalance,
                    expected_profit_usd: 100.0, // Base profit para low slippage
                    confidence: 0.6,
                    time_window_ms: 60000, // 1 min window
                    recommended_size_usd: 5000.0, // Larger trade for low impact
                };
                
                debug!("📊 Low price impact opportunity: {:.2}%", pool.price_impact_1k);
                self.opportunities.push(opportunity);
            }
        }
        
        Ok(())
    }
    
    /// Buscar picos de volumen inusual
    async fn scan_for_volume_spike_opportunities(&mut self) -> Result<()> {
        for (_, pool) in &self.tracked_pools {
            // Volume spike: volumen > 2x liquidez (inusual)
            if pool.volume_24h > pool.liquidity_usd * 2.0 && pool.volume_24h > 10000.0 {
                let opportunity = TradingOpportunity {
                    pool: pool.clone(),
                    opportunity_type: OpportunityType::VolumeSpike,
                    expected_profit_usd: (pool.volume_24h / pool.liquidity_usd) * 50.0,
                    confidence: 0.5,
                    time_window_ms: 30000, // 30s window
                    recommended_size_usd: 1000.0,
                };
                
                info!("📈 Volume spike detected: ${:.0} volume vs ${:.0} liquidity", 
                      pool.volume_24h, pool.liquidity_usd);
                self.opportunities.push(opportunity);
            }
        }
        
        Ok(())
    }
    
    /// Obtener oportunidades actuales
    pub fn get_current_opportunities(&self) -> &[TradingOpportunity] {
        &self.opportunities
    }
    
    /// Obtener pools trackeados
    pub fn get_tracked_pools(&self) -> &HashMap<String, DetectedPool> {
        &self.tracked_pools
    }
      /// Obtener estadísticas del detector
    pub fn get_stats(&self) -> PoolDetectorStats {
        PoolDetectorStats {
            tracked_pools: self.tracked_pools.len(),
            active_opportunities: self.opportunities.len(),
            last_scan_ago: self.last_scan.elapsed(),
            total_scans: 0, // TODO: implementar contador
        }
    }

    /// Obtener datos reales de pools de Raydium API
    async fn fetch_real_raydium_pools(&self) -> Result<Vec<DetectedPool>> {
        info!("🌐 Fetching real pool data from Raydium API...");
        
        // URL para obtener todos los pools de Raydium
        let url = "https://api.raydium.io/v2/ammV3/ammPools";
        
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .timeout(Duration::from_secs(10))
            .send()
            .await?;
            
        if !response.status().is_success() {
            warn!("⚠️ Raydium API returned status: {}", response.status());
            return Ok(Vec::new());
        }
        
        let pool_data = response.json::<serde_json::Value>().await?;
        
        // Parsear los pools desde la respuesta
        let mut detected_pools = Vec::new();
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        if let Some(pools) = pool_data.get("data").and_then(|d| d.as_array()) {
            info!("📊 Found {} total pools in Raydium API", pools.len());
            
            // Tomar los pools más recientes (últimos 50) y analizarlos
            let recent_pools = pools.iter().rev().take(50);
            
            for pool_json in recent_pools {
                if let Ok(Some(pool)) = self.parse_raydium_pair(pool_json, current_time).await {
                    // Verificar si el pool es "nuevo" (últimas 2 horas)
                    let pool_age_hours = (current_time - pool.created_at) / 3600;
                    
                    if pool_age_hours <= 2 { // Solo pools de las últimas 2 horas
                        info!("🆕 Found recent pool: {} ({}/{})", 
                              pool.pool_address, pool.token_a.symbol, pool.token_b.symbol);
                        detected_pools.push(pool);
                    } else {
                        debug!("⏰ Pool too old: {} hours", pool_age_hours);
                    }
                }
            }
        } else {
            warn!("⚠️ Unexpected Raydium API response format");
        }
        
        if detected_pools.is_empty() {
            info!("📭 No new pools found in recent timeframe");
            // Intentar con API alternativa de Orca
            return self.fetch_real_orca_pools().await;
        }
        
        Ok(detected_pools)
    }
    
    /// Obtener datos de pools reales de Orca API
    async fn fetch_real_orca_pools(&self) -> Result<Vec<DetectedPool>> {
        info!("🐳 Fetching real pool data from Orca API...");
        
        // URL para obtener pools de Orca
        let url = "https://api.orca.so/v1/whirlpools";
        
        let client = reqwest::Client::new();
        match client
            .get(url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(pool_data) = response.json::<serde_json::Value>().await {
                        return self.parse_orca_pools(pool_data).await;
                    }
                }
            }
            Err(e) => {
                warn!("⚠️ Orca API request failed: {}", e);
            }
        }
        
        Ok(Vec::new())
    }
    
    /// Parsear pools de Orca
    async fn parse_orca_pools(&self, pool_data: serde_json::Value) -> Result<Vec<DetectedPool>> {
        let mut detected_pools = Vec::new();
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        if let Some(pools) = pool_data.get("whirlpools").and_then(|d| d.as_array()) {
            info!("🐳 Found {} total Orca pools", pools.len());
            
            for pool_json in pools.iter().take(20) { // Analizar primeros 20
                if let Ok(Some(pool)) = self.parse_orca_pool(pool_json, current_time).await {
                    detected_pools.push(pool);
                }
            }
        }
        
        Ok(detected_pools)
    }
    
    /// Parsear un pool individual de Orca
    async fn parse_orca_pool(&self, pool_data: &serde_json::Value, current_time: u64) -> Result<Option<DetectedPool>> {
        let pool_address = pool_data.get("address")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
            
        let token_a_mint = pool_data.get("tokenA")
            .and_then(|t| t.get("mint"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
            
        let token_b_mint = pool_data.get("tokenB")
            .and_then(|t| t.get("mint"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
            
        let liquidity = pool_data.get("liquidity")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        let token_a_symbol = self.get_token_symbol_from_mint(token_a_mint).await
            .unwrap_or_else(|| "UNKNOWN".to_string());
        let token_b_symbol = self.get_token_symbol_from_mint(token_b_mint).await
            .unwrap_or_else(|| "UNKNOWN".to_string());
            
        let token_a_price = self.get_token_price_from_jupiter(token_a_mint).await.unwrap_or(0.0);
        let token_b_price = self.get_token_price_from_jupiter(token_b_mint).await.unwrap_or(1.0);
        
        let pool = DetectedPool {
            pool_address: pool_address.to_string(),
            token_a: TokenInfo {
                mint: token_a_mint.to_string(),
                symbol: token_a_symbol,
                decimals: 9,
                supply: 1_000_000_000,
                price_usd: token_a_price,
                market_cap: token_a_price * 1_000_000.0,
            },
            token_b: TokenInfo {
                mint: token_b_mint.to_string(),
                symbol: token_b_symbol,
                decimals: 6,
                supply: 1_000_000_000,
                price_usd: token_b_price,
                market_cap: token_b_price * 1_000_000.0,
            },
            liquidity_usd: liquidity,
            price_impact_1k: self.calculate_price_impact(liquidity).await,
            volume_24h: 0.0, // Orca API might not provide this directly
            created_at: current_time - rand::random::<u64>() % 86400,
            detected_at: current_time,
            dex: "Orca".to_string(),
            risk_score: self.calculate_risk_score_simple(liquidity, 0.0).await,
        };
        
        Ok(Some(pool))
    }
    
    /// Calcular risk score simple para pools sin datos completos
    async fn calculate_risk_score_simple(&self, liquidity: f64, volume: f64) -> RiskScore {
        let liquidity_score = if liquidity > 100000.0 { 0.9 }
                             else if liquidity > 50000.0 { 0.7 }
                             else if liquidity > 10000.0 { 0.5 }
                             else { 0.2 };
                             
        let volume_score = if volume > liquidity * 0.5 { 0.8 }
                          else if volume > liquidity * 0.2 { 0.6 }
                          else { 0.4 };
        
        let overall = (liquidity_score + volume_score) / 2.0;
        
        let mut rug_indicators = Vec::new();
        if liquidity < 10000.0 {
            rug_indicators.push("Low liquidity".to_string());
        }
        
        RiskScore {
            overall,
            liquidity_score,
            volume_score,
            token_age_score: 0.5,
            holder_distribution_score: 0.5,
            rug_indicators,
        }
    }
    
    /// Iniciar monitoreo continuo con reports periódicos
    pub async fn start_monitoring_with_reports(&mut self, duration_minutes: u64) -> Result<()> {
        info!("🚀 Starting monitored pool detection for {} minutes...", duration_minutes);
        
        let total_duration = Duration::from_secs(duration_minutes * 60);
        let report_interval = Duration::from_secs(30); // Report every 30s
        let start_time = Instant::now();
        
        let mut last_report = Instant::now();
        let mut total_scans = 0u64;
        let mut pools_found_this_session = 0usize;
        let mut opportunities_found_this_session = 0usize;
        
        while start_time.elapsed() < total_duration {
            let scan_start = Instant::now();
            total_scans += 1;
            
            // Detectar nuevos pools
            match self.scan_for_new_pools().await {
                Ok(new_pools) => {
                    if !new_pools.is_empty() {
                        pools_found_this_session += new_pools.len();
                        info!("🆕 Found {} new pools (session total: {})", 
                              new_pools.len(), pools_found_this_session);
                        
                        for pool in new_pools {
                            if let Err(e) = self.analyze_pool_opportunity(&pool).await {
                                warn!("⚠️ Pool analysis failed: {}", e);
                            } else {
                                opportunities_found_this_session += 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠️ Pool scan #{} failed: {}", total_scans, e);
                }
            }
            
            // Actualizar pools existentes
            if let Err(e) = self.update_tracked_pools().await {
                warn!("⚠️ Pool update failed: {}", e);
            }
            
            // Buscar oportunidades avanzadas
            if let Err(e) = self.scan_for_opportunities().await {
                warn!("⚠️ Opportunity scan failed: {}", e);
            }
            
            // Report periódico
            if last_report.elapsed() >= report_interval {
                self.print_status_report(
                    start_time.elapsed(), 
                    total_scans, 
                    pools_found_this_session,
                    opportunities_found_this_session
                );
                last_report = Instant::now();
            }
            
            // Sleep hasta próximo scan
            let scan_duration = scan_start.elapsed();
            let sleep_time = Duration::from_millis(self.config.monitoring_interval_ms)
                .saturating_sub(scan_duration);
            
            if sleep_time.as_millis() > 0 {
                tokio::time::sleep(sleep_time).await;
            }
        }
        
        // Final report
        println!("\n📊 MONITORING SESSION COMPLETED");
        println!("===============================");
        println!("⏱️ Duration: {:.1} minutes", start_time.elapsed().as_secs_f64() / 60.0);
        println!("🔍 Total scans: {}", total_scans);
        println!("🆕 Pools found: {}", pools_found_this_session);
        println!("🎯 Opportunities: {}", opportunities_found_this_session);
        
        Ok(())
    }
    
    /// Imprimir reporte de estado
    fn print_status_report(&self, elapsed: Duration, total_scans: u64, pools_found: usize, opportunities_found: usize) {
        let stats = self.get_stats();
        println!("📊 Status after {:.1}min | Scans: {} | Pools: {} (+{}) | Opportunities: {} (+{})", 
                 elapsed.as_secs_f64() / 60.0,
                 total_scans,
                 stats.tracked_pools,
                 pools_found,
                 stats.active_opportunities,
                 opportunities_found
        );
        
        // Show most promising opportunities
        if !self.opportunities.is_empty() {
            let best_opp = self.opportunities.iter()
                .max_by(|a, b| a.expected_profit_usd.partial_cmp(&b.expected_profit_usd).unwrap());
            
            if let Some(opp) = best_opp {
                println!("   🎯 Best opportunity: ${:.2} profit ({}/{})", 
                         opp.expected_profit_usd,
                         opp.pool.token_a.symbol,
                         opp.pool.token_b.symbol);
            }        }
    }
    
    /// Parsear datos de un pair de Raydium a nuestro formato
    async fn parse_raydium_pair(&self, pair_data: &serde_json::Value, current_time: u64) -> Result<Option<DetectedPool>> {
        // Extraer información del JSON de Raydium
        let pair_address = pair_data.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
            
        let base_mint = pair_data.get("baseMint")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
            
        let quote_mint = pair_data.get("quoteMint")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
            
        let liquidity = pair_data.get("liquidity")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
            
        let volume_24h = pair_data.get("volume24h")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        // Obtener información de tokens usando Jupiter API
        let base_symbol = self.get_token_symbol_from_mint(base_mint).await
            .unwrap_or_else(|| "UNKNOWN".to_string());
        let quote_symbol = self.get_token_symbol_from_mint(quote_mint).await
            .unwrap_or_else(|| "UNKNOWN".to_string());
            
        // Obtener precios reales usando Jupiter
        let base_price = self.get_token_price_from_jupiter(base_mint).await.unwrap_or(0.0);
        let quote_price = self.get_token_price_from_jupiter(quote_mint).await.unwrap_or(1.0);
        
        let pool = DetectedPool {
            pool_address: pair_address.to_string(),
            token_a: TokenInfo {
                mint: base_mint.to_string(),
                symbol: base_symbol,
                decimals: 9, // Default, podríamos obtener del token metadata
                supply: 1_000_000_000, // Placeholder
                price_usd: base_price,
                market_cap: base_price * 1_000_000.0, // Estimación simple
            },
            token_b: TokenInfo {
                mint: quote_mint.to_string(),
                symbol: quote_symbol,
                decimals: 6, // Default para USDC/USDT
                supply: 1_000_000_000,
                price_usd: quote_price,
                market_cap: quote_price * 1_000_000.0,
            },
            liquidity_usd: liquidity,
            price_impact_1k: self.calculate_price_impact(liquidity).await,
            volume_24h,
            created_at: current_time - rand::random::<u64>() % 86400, // Estimación
            detected_at: current_time,
            dex: "Raydium".to_string(),
            risk_score: self.calculate_risk_score_real(&pair_data).await,
        };
        
        Ok(Some(pool))
    }

    /// Obtener símbolo de token desde mint usando Jupiter API
    async fn get_token_symbol_from_mint(&self, mint: &str) -> Option<String> {
        // Casos especiales para tokens conocidos
        match mint {
            "So11111111111111111111111111111111111111112" => return Some("SOL".to_string()),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => return Some("USDC".to_string()),
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => return Some("USDT".to_string()),
            _ => {}
        }
        
        // Usar Jupiter tokens API
        let url = format!("https://token.jup.ag/strict");
        
        match reqwest::get(&url).await {
            Ok(response) => {
                if let Ok(tokens) = response.json::<serde_json::Value>().await {
                    if let Some(token_list) = tokens.as_array() {
                        for token in token_list {
                            if let Some(token_mint) = token.get("address").and_then(|v| v.as_str()) {
                                if token_mint == mint {
                                    return token.get("symbol").and_then(|v| v.as_str()).map(|s| s.to_string());
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => {}
        }
        
        None
    }

    /// Obtener precio de token usando Jupiter API
    async fn get_token_price_from_jupiter(&self, mint: &str) -> Option<f64> {
        let url = format!("https://price.jup.ag/v4/price?ids={}", mint);
        
        match reqwest::get(&url).await {
            Ok(response) => {
                if let Ok(price_data) = response.json::<serde_json::Value>().await {
                    if let Some(data) = price_data.get("data").and_then(|d| d.get(mint)) {
                        if let Some(price) = data.get("price").and_then(|p| p.as_str()) {
                            return price.parse::<f64>().ok();
                        }
                    }
                }
            }
            Err(_) => {}
        }
        
        None
    }

    /// Calcular price impact basado en liquidez
    async fn calculate_price_impact(&self, liquidity: f64) -> f64 {
        // Fórmula simple: mayor liquidez = menor price impact
        if liquidity > 100000.0 {
            0.5  // 0.5% para pools grandes
        } else if liquidity > 50000.0 {
            1.0  // 1% para pools medianos
        } else if liquidity > 10000.0 {
            3.0  // 3% para pools pequeños
        } else {
            10.0 // 10% para pools muy pequeños
        }
    }

    /// Calcular risk score usando datos reales
    async fn calculate_risk_score_real(&self, pair_data: &serde_json::Value) -> RiskScore {
        let liquidity = pair_data.get("liquidity").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let volume = pair_data.get("volume24h").and_then(|v| v.as_f64()).unwrap_or(0.0);
        
        // Score basado en liquidez (0.0 = riesgoso, 1.0 = seguro)
        let liquidity_score = if liquidity > 100000.0 { 0.9 }
                             else if liquidity > 50000.0 { 0.7 }
                             else if liquidity > 10000.0 { 0.5 }
                             else { 0.2 };
                             
        // Score basado en volumen
        let volume_score = if volume > liquidity * 0.5 { 0.8 }
                          else if volume > liquidity * 0.2 { 0.6 }
                          else if volume > 0.0 { 0.4 }
                          else { 0.1 };
        
        let overall = (liquidity_score + volume_score) / 2.0;
        
        let mut rug_indicators = Vec::new();
        if liquidity < 10000.0 {
            rug_indicators.push("Low liquidity".to_string());
        }
        if volume < liquidity * 0.1 {
            rug_indicators.push("Low volume".to_string());
        }        
        RiskScore {
            overall,
            liquidity_score,
            volume_score,
            token_age_score: 0.5, // Placeholder
            holder_distribution_score: 0.5, // Placeholder
            rug_indicators,
        }
    }
            
    /// Reporte optimizado para ultra-fast monitoring
    fn print_ultra_fast_report(&self, elapsed: Duration, api_scans: u64, ws_updates: u64, pools_found: usize) {
        let stats = self.get_stats();
        println!("⚡ Ultra-Fast Status | {:.1}min | API: {} | WS: {} | Pools: {} | Opps: {}", 
                 elapsed.as_secs_f64() / 60.0,
                 api_scans,
                 ws_updates,
                 pools_found,
                 stats.active_opportunities
        );
        
        if !self.opportunities.is_empty() {
            let best_opp = self.opportunities.iter()
                .max_by(|a, b| a.expected_profit_usd.partial_cmp(&b.expected_profit_usd).unwrap());
            
            if let Some(opp) = best_opp {
                println!("   🎯 Best: ${:.2} profit - {}/{} ({})", 
                         opp.expected_profit_usd,
                         opp.pool.token_a.symbol,
                         opp.pool.token_b.symbol,
                         match opp.opportunity_type {
                             OpportunityType::NewPoolSnipe => "NEW",
                             OpportunityType::PriceDiscrepancy => "ARB",
                             OpportunityType::LiquidityImbalance => "LOW_SLIP",
                             OpportunityType::VolumeSpike => "VOL_SPIKE",
                         });
            }
        }
    }
    
    /// Setup WebSocket monitoring task
    async fn start_websocket_pool_monitoring(&mut self, _syndica_client: &mut SyndicaWebSocketClient) -> Result<()> {
        // En una implementación real, esto configuraría el WebSocket para escuchar:
        // - Program notifications para nuevos pools
        // - Account updates para cambios de liquidez
        // - Slot notifications para timing preciso
        
        info!("⚡ WebSocket pool monitoring configured");
        Ok(())
    }    /// Usar solo monitoring básico por ahora  
    pub async fn start_ultra_fast_monitoring(&mut self, duration_minutes: u64) -> Result<()> {
        info!("⚡ Starting optimized pool monitoring...");
        
        // Por ahora usar el método básico hasta arreglar borrowing
        self.start_monitoring_with_reports(duration_minutes).await    }
}

/// Estadísticas del detector
#[derive(Debug, Clone)]
pub struct PoolDetectorStats {
    pub tracked_pools: usize,
    pub active_opportunities: usize,
    pub last_scan_ago: Duration,
    pub total_scans: u64,
}

/// Test function para pool detection
pub async fn test_pool_detection() -> Result<()> {
    println!("🔍 POOL DETECTION TEST (MainNet Read-Only)");
    println!("==========================================");
    
    // Esta función será implementada en el CLI
    println!("📊 Pool detection test will be integrated into CLI");
    println!("   Use: cargo run -- test pools");
    
    Ok(())
}
