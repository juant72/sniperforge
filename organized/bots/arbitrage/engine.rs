// ================================================================================
// ARBITRAGE ENGINE - Motor Principal de Detección de Oportunidades
// ================================================================================

use std::collections::{HashMap, HashSet};
use std::time::{Instant, Duration};
use anyhow::Result;
use tracing::{info, debug, warn, error};
use serde::{Serialize, Deserialize};
use solana_sdk::pubkey::Pubkey;

use sniperforge_core::{
    CoreResult, CoreError, TradingOpportunity,
    feeds::PriceFeedManager,
    types::{TokenInfo, DexInfo, RouteInfo},
};

use crate::config::EngineConfig;

/// Motor de arbitraje principal
pub struct ArbitrageEngine {
    config: EngineConfig,
    supported_dexes: HashSet<String>,
    price_cache: HashMap<String, CachedPrice>,
    opportunity_history: Vec<ArbitrageOpportunity>,
    last_scan: Instant,
    statistics: EngineStatistics,
}

/// Oportunidad de arbitraje detectada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub id: String,
    pub token_in: TokenInfo,
    pub token_out: TokenInfo,
    pub amount_in: u64,
    pub amount_out: u64,
    pub profit_percentage: f64,
    pub expected_profit_sol: f64,
    pub confidence_score: f64,
    pub route_info: RouteInfo,
    pub dex_from: DexInfo,
    pub dex_to: DexInfo,
    pub price_impact: f64,
    pub execution_time_estimate_ms: u64,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub risk_score: f64,
    pub gas_estimate: u64,
}

/// Precio en caché con timestamp
#[derive(Debug, Clone)]
struct CachedPrice {
    price: f64,
    timestamp: Instant,
    source: String,
    confidence: f64,
}

/// Estadísticas del motor
#[derive(Debug, Clone, Default)]
pub struct EngineStatistics {
    pub total_scans: u64,
    pub opportunities_found: u64,
    pub opportunities_expired: u64,
    pub avg_scan_time_ms: f64,
    pub best_profit_found: f64,
    pub total_profit_potential: f64,
    pub dex_performance: HashMap<String, DexPerformance>,
}

/// Performance por DEX
#[derive(Debug, Clone, Default)]
pub struct DexPerformance {
    pub opportunities_count: u64,
    pub avg_profit: f64,
    pub success_rate: f64,
    pub avg_execution_time: f64,
}

impl ArbitrageEngine {
    /// Crear nuevo motor de arbitraje
    pub fn new(config: &EngineConfig) -> CoreResult<Self> {
        let supported_dexes: HashSet<String> = config.supported_dexes.iter().cloned().collect();
        
        info!("🔧 Inicializando ArbitrageEngine con {} DEXs soportados", supported_dexes.len());
        
        Ok(Self {
            config: config.clone(),
            supported_dexes,
            price_cache: HashMap::new(),
            opportunity_history: Vec::new(),
            last_scan: Instant::now(),
            statistics: EngineStatistics::default(),
        })
    }
    
    /// Buscar oportunidades de arbitraje
    pub async fn find_opportunities(&mut self, price_feeds: &PriceFeedManager) -> CoreResult<Vec<ArbitrageOpportunity>> {
        let scan_start = Instant::now();
        self.statistics.total_scans += 1;
        
        debug!("🔍 Iniciando búsqueda de oportunidades de arbitraje");
        
        // 1. Update price cache
        self.update_price_cache(price_feeds).await?;
        
        // 2. Find arbitrage opportunities
        let mut opportunities = Vec::new();
        
        // Get all token pairs with price differences
        let token_pairs = self.get_token_pairs_with_price_differences().await?;
        
        for (token_a, token_b) in token_pairs {
            // Find opportunities between different DEXs
            if let Some(opportunity) = self.analyze_token_pair(&token_a, &token_b).await? {
                if self.is_opportunity_viable(&opportunity) {
                    opportunities.push(opportunity);
                }
            }
        }
        
        // 3. Sort by profitability
        opportunities.sort_by(|a, b| b.expected_profit_sol.partial_cmp(&a.expected_profit_sol).unwrap());
        
        // 4. Update statistics
        let scan_time = scan_start.elapsed().as_millis() as f64;
        self.update_scan_statistics(scan_time, opportunities.len());
        
        info!("✅ Encontradas {} oportunidades en {:.2}ms", opportunities.len(), scan_time);
        
        Ok(opportunities)
    }
    
    /// Actualizar caché de precios
    async fn update_price_cache(&mut self, price_feeds: &PriceFeedManager) -> CoreResult<()> {
        debug!("📊 Actualizando caché de precios");
        
        // Get prices from all supported DEXs
        for dex_name in &self.supported_dexes {
            match price_feeds.get_dex_prices(dex_name).await {
                Ok(prices) => {
                    for (token_symbol, price_data) in prices {
                        let cache_key = format!("{}_{}", dex_name, token_symbol);
                        self.price_cache.insert(cache_key, CachedPrice {
                            price: price_data.price,
                            timestamp: Instant::now(),
                            source: dex_name.clone(),
                            confidence: price_data.confidence.unwrap_or(1.0),
                        });
                    }
                }
                Err(e) => {
                    warn!("⚠️  Error obteniendo precios de {}: {}", dex_name, e);
                }
            }
        }
        
        // Clean old cache entries (older than 30 seconds)
        let now = Instant::now();
        self.price_cache.retain(|_, cached_price| {
            now.duration_since(cached_price.timestamp) < Duration::from_secs(30)
        });
        
        Ok(())
    }
    
    /// Obtener pares de tokens con diferencias de precio
    async fn get_token_pairs_with_price_differences(&self) -> CoreResult<Vec<(String, String)>> {
        let mut token_pairs = Vec::new();
        let mut processed_pairs = HashSet::new();
        
        // Extract unique tokens from cache
        let mut tokens = HashSet::new();
        for cache_key in self.price_cache.keys() {
            if let Some(token) = cache_key.split('_').nth(1) {
                tokens.insert(token.to_string());
            }
        }
        
        // Generate all combinations of token pairs
        let token_vec: Vec<String> = tokens.into_iter().collect();
        for i in 0..token_vec.len() {
            for j in (i + 1)..token_vec.len() {
                let pair = (token_vec[i].clone(), token_vec[j].clone());
                let reverse_pair = (token_vec[j].clone(), token_vec[i].clone());
                
                if !processed_pairs.contains(&pair) && !processed_pairs.contains(&reverse_pair) {
                    // Check if there's a significant price difference across DEXs
                    if self.has_price_difference(&pair.0, &pair.1).await? {
                        token_pairs.push(pair.clone());
                        processed_pairs.insert(pair);
                    }
                }
            }
        }
        
        Ok(token_pairs)
    }
    
    /// Verificar si hay diferencia de precio significativa
    async fn has_price_difference(&self, token_a: &str, token_b: &str) -> CoreResult<bool> {
        let mut prices_a = Vec::new();
        let mut prices_b = Vec::new();
        
        // Collect prices from all DEXs
        for dex_name in &self.supported_dexes {
            let key_a = format!("{}_{}", dex_name, token_a);
            let key_b = format!("{}_{}", dex_name, token_b);
            
            if let (Some(price_a), Some(price_b)) = (self.price_cache.get(&key_a), self.price_cache.get(&key_b)) {
                prices_a.push(price_a.price);
                prices_b.push(price_b.price);
            }
        }
        
        if prices_a.len() < 2 || prices_b.len() < 2 {
            return Ok(false);
        }
        
        // Calculate price ratios
        let ratios: Vec<f64> = prices_a.iter().zip(prices_b.iter())
            .map(|(a, b)| a / b)
            .collect();
        
        if ratios.is_empty() {
            return Ok(false);
        }
        
        let min_ratio = ratios.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_ratio = ratios.iter().fold(0.0, |a, &b| a.max(b));
        
        let price_difference = (max_ratio - min_ratio) / min_ratio;
        
        Ok(price_difference > self.config.min_profit_percentage)
    }
    
    /// Analizar par de tokens para oportunidades
    async fn analyze_token_pair(&self, token_a: &str, token_b: &str) -> CoreResult<Option<ArbitrageOpportunity>> {
        let mut best_opportunity = None;
        let mut best_profit = 0.0;
        
        // Check all DEX combinations
        for dex_from in &self.supported_dexes {
            for dex_to in &self.supported_dexes {
                if dex_from == dex_to {
                    continue;
                }
                
                if let Some(opportunity) = self.calculate_arbitrage_opportunity(
                    token_a, token_b, dex_from, dex_to
                ).await? {
                    if opportunity.expected_profit_sol > best_profit {
                        best_profit = opportunity.expected_profit_sol;
                        best_opportunity = Some(opportunity);
                    }
                }
            }
        }
        
        Ok(best_opportunity)
    }
    
    /// Calcular oportunidad de arbitraje específica
    async fn calculate_arbitrage_opportunity(
        &self,
        token_a: &str,
        token_b: &str,
        dex_from: &str,
        dex_to: &str,
    ) -> CoreResult<Option<ArbitrageOpportunity>> {
        let key_from_a = format!("{}_{}", dex_from, token_a);
        let key_from_b = format!("{}_{}", dex_from, token_b);
        let key_to_a = format!("{}_{}", dex_to, token_a);
        let key_to_b = format!("{}_{}", dex_to, token_b);
        
        let price_from_a = self.price_cache.get(&key_from_a).map(|p| p.price);
        let price_from_b = self.price_cache.get(&key_from_b).map(|p| p.price);
        let price_to_a = self.price_cache.get(&key_to_a).map(|p| p.price);
        let price_to_b = self.price_cache.get(&key_to_b).map(|p| p.price);
        
        if let (Some(pfa), Some(pfb), Some(pta), Some(ptb)) = (price_from_a, price_from_b, price_to_a, price_to_b) {
            // Calculate arbitrage: Buy A with B on dex_from, Sell A for B on dex_to
            let ratio_from = pfa / pfb; // A/B ratio on dex_from
            let ratio_to = pta / ptb;   // A/B ratio on dex_to
            
            if ratio_to > ratio_from {
                let profit_percentage = (ratio_to - ratio_from) / ratio_from;
                
                if profit_percentage > self.config.min_profit_percentage {
                    // Calculate amounts (using reasonable test amount)
                    let amount_b = 1000000u64; // 0.001 if token has 9 decimals
                    let amount_a_bought = (amount_b as f64 / pfb * pfa) as u64;
                    let amount_b_sold = (amount_a_bought as f64 / pta * ptb) as u64;
                    let profit_b = amount_b_sold.saturating_sub(amount_b);
                    
                    // Convert to SOL (assuming token B is SOL or calculate via SOL price)
                    let profit_sol = self.convert_to_sol(profit_b as f64, token_b).await?;
                    
                    if profit_sol > self.config.min_profit_sol {
                        let opportunity = ArbitrageOpportunity {
                            id: format!("arb_{}_{}_{}_{}", token_a, token_b, dex_from, dex_to),
                            token_in: TokenInfo {
                                symbol: token_b.to_string(),
                                mint: Pubkey::default(), // TODO: Get actual mint
                                decimals: 9,
                                name: token_b.to_string(),
                            },
                            token_out: TokenInfo {
                                symbol: token_a.to_string(),
                                mint: Pubkey::default(), // TODO: Get actual mint
                                decimals: 9,
                                name: token_a.to_string(),
                            },
                            amount_in: amount_b,
                            amount_out: amount_a_bought,
                            profit_percentage,
                            expected_profit_sol: profit_sol,
                            confidence_score: 0.8, // Base confidence
                            route_info: RouteInfo {
                                hops: 2,
                                route_plan: vec![dex_from.to_string(), dex_to.to_string()],
                                estimated_gas: 100000,
                                price_impact: self.estimate_price_impact(amount_b as f64, token_b).await?,
                            },
                            dex_from: DexInfo {
                                name: dex_from.to_string(),
                                program_id: Pubkey::default(), // TODO: Get actual program IDs
                                fee_bps: 25, // Typical DEX fee
                            },
                            dex_to: DexInfo {
                                name: dex_to.to_string(),
                                program_id: Pubkey::default(),
                                fee_bps: 25,
                            },
                            price_impact: self.estimate_price_impact(amount_b as f64, token_b).await?,
                            execution_time_estimate_ms: 5000, // Estimated execution time
                            detected_at: chrono::Utc::now(),
                            expires_at: chrono::Utc::now() + chrono::Duration::seconds(30),
                            risk_score: self.calculate_risk_score(profit_percentage, profit_sol),
                            gas_estimate: 100000,
                        };
                        
                        return Ok(Some(opportunity));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Convertir cantidad a SOL
    async fn convert_to_sol(&self, amount: f64, token: &str) -> CoreResult<f64> {
        if token == "SOL" {
            return Ok(amount / 1e9); // Convert lamports to SOL
        }
        
        // For other tokens, get SOL price
        if let Some(sol_price) = self.get_token_sol_price(token).await? {
            Ok(amount * sol_price / 1e9)
        } else {
            Ok(0.0)
        }
    }
    
    /// Obtener precio del token en SOL
    async fn get_token_sol_price(&self, token: &str) -> CoreResult<Option<f64>> {
        // Try to find token/SOL price in any DEX
        for dex_name in &self.supported_dexes {
            let key = format!("{}_{}", dex_name, token);
            if let Some(cached_price) = self.price_cache.get(&key) {
                // This is a simplified implementation
                // In reality, you'd need to get the actual SOL price
                return Ok(Some(cached_price.price));
            }
        }
        Ok(None)
    }
    
    /// Estimar impacto de precio
    async fn estimate_price_impact(&self, amount: f64, token: &str) -> CoreResult<f64> {
        // Simplified price impact calculation
        // In reality, this would be much more sophisticated
        let base_impact = amount / 1000000.0; // Very simplified
        Ok(base_impact.min(0.1)) // Cap at 10%
    }
    
    /// Calcular score de riesgo
    fn calculate_risk_score(&self, profit_percentage: f64, profit_sol: f64) -> f64 {
        let mut risk_score = 0.5; // Base risk
        
        // Lower risk for higher profits
        if profit_percentage > 0.02 { // > 2%
            risk_score -= 0.2;
        }
        
        if profit_sol > 0.01 { // > 0.01 SOL
            risk_score -= 0.1;
        }
        
        // Higher risk for very high profits (might be too good to be true)
        if profit_percentage > 0.1 { // > 10%
            risk_score += 0.3;
        }
        
        risk_score.max(0.0).min(1.0)
    }
    
    /// Verificar si la oportunidad es viable
    fn is_opportunity_viable(&self, opportunity: &ArbitrageOpportunity) -> bool {
        // Check minimum profit requirements
        if opportunity.expected_profit_sol < self.config.min_profit_sol {
            return false;
        }
        
        if opportunity.profit_percentage < self.config.min_profit_percentage {
            return false;
        }
        
        // Check price impact threshold
        if opportunity.price_impact > self.config.price_impact_threshold {
            return false;
        }
        
        // Check if not expired
        if opportunity.expires_at < chrono::Utc::now() {
            return false;
        }
        
        true
    }
    
    /// Actualizar estadísticas de escaneo
    fn update_scan_statistics(&mut self, scan_time_ms: f64, opportunities_found: usize) {
        self.statistics.opportunities_found += opportunities_found as u64;
        
        // Update average scan time
        let total_scans = self.statistics.total_scans as f64;
        self.statistics.avg_scan_time_ms = 
            (self.statistics.avg_scan_time_ms * (total_scans - 1.0) + scan_time_ms) / total_scans;
    }
    
    /// Obtener estadísticas del motor
    pub fn get_statistics(&self) -> &EngineStatistics {
        &self.statistics
    }
    
    /// Limpiar historial de oportunidades antiguas
    pub fn cleanup_expired_opportunities(&mut self) {
        let now = chrono::Utc::now();
        let before_count = self.opportunity_history.len();
        
        self.opportunity_history.retain(|opp| opp.expires_at > now);
        
        let expired_count = before_count - self.opportunity_history.len();
        self.statistics.opportunities_expired += expired_count as u64;
        
        if expired_count > 0 {
            debug!("🧹 Limpiadas {} oportunidades expiradas", expired_count);
        }
    }
}
