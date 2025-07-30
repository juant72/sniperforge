// ================================================================================
// TRIANGULAR ARBITRAGE - Motor de Arbitraje Triangular
// ================================================================================

use std::collections::{HashMap, HashSet};
use std::time::Instant;
use anyhow::Result;
use tracing::{info, debug, warn};
use serde::{Serialize, Deserialize};

use sniperforge_core::{
    CoreResult, CoreError,
    feeds::PriceFeedManager,
    types::{TokenInfo, DexInfo, RouteInfo},
};

use crate::config::TriangularConfig;
use crate::engine::ArbitrageOpportunity;

/// Motor de arbitraje triangular
pub struct TriangularEngine {
    config: TriangularConfig,
    base_tokens: HashSet<String>,
    triangle_cache: HashMap<String, TriangleCache>,
    statistics: TriangularStatistics,
}

/// Oportunidad de arbitraje triangular
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangularOpportunity {
    pub id: String,
    pub triangle: TokenTriangle,
    pub expected_profit_sol: f64,
    pub profit_percentage: f64,
    pub confidence_score: f64,
    pub execution_steps: Vec<ExecutionStep>,
    pub total_price_impact: f64,
    pub estimated_gas: u64,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub risk_score: f64,
}

/// Triángulo de tokens (A -> B -> C -> A)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTriangle {
    pub token_a: TokenInfo,
    pub token_b: TokenInfo,
    pub token_c: TokenInfo,
    pub dex_ab: DexInfo,
    pub dex_bc: DexInfo,
    pub dex_ca: DexInfo,
}

/// Paso de ejecución en el arbitraje triangular
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_number: u8,
    pub action: String, // "swap", "buy", "sell"
    pub token_from: TokenInfo,
    pub token_to: TokenInfo,
    pub amount_in: u64,
    pub amount_out: u64,
    pub dex: DexInfo,
    pub price_impact: f64,
    pub estimated_time_ms: u64,
}

/// Cache de triángulos
#[derive(Debug, Clone)]
struct TriangleCache {
    triangle: TokenTriangle,
    last_profit: f64,
    last_updated: Instant,
    success_count: u32,
    total_attempts: u32,
}

/// Estadísticas del motor triangular
#[derive(Debug, Clone, Default)]
pub struct TriangularStatistics {
    pub triangles_analyzed: u64,
    pub opportunities_found: u64,
    pub successful_executions: u64,
    pub total_profit_sol: f64,
    pub avg_profit_percentage: f64,
    pub best_triangle: Option<String>,
    pub most_profitable_combination: Option<(String, String, String)>,
}

impl TriangularEngine {
    /// Crear nuevo motor de arbitraje triangular
    pub fn new(config: &TriangularConfig) -> CoreResult<Self> {
        let base_tokens: HashSet<String> = config.base_tokens.iter().cloned().collect();
        
        info!("🔺 Inicializando TriangularEngine con {} tokens base", base_tokens.len());
        
        Ok(Self {
            config: config.clone(),
            base_tokens,
            triangle_cache: HashMap::new(),
            statistics: TriangularStatistics::default(),
        })
    }
    
    /// Buscar oportunidades de arbitraje triangular
    pub async fn find_opportunities(&mut self, price_feeds: &PriceFeedManager) -> CoreResult<Vec<ArbitrageOpportunity>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }
        
        debug!("🔺 Buscando oportunidades de arbitraje triangular");
        
        let mut opportunities = Vec::new();
        let start_time = Instant::now();
        
        // Generate all possible triangles
        let triangles = self.generate_triangles(price_feeds).await?;
        
        for triangle in triangles {
            if let Some(triangular_opp) = self.analyze_triangle(&triangle, price_feeds).await? {
                // Convert to ArbitrageOpportunity
                if let Some(arb_opp) = self.convert_to_arbitrage_opportunity(&triangular_opp).await? {
                    opportunities.push(arb_opp);
                }
            }
            
            // Check timeout
            if start_time.elapsed().as_millis() > self.config.timeout_ms {
                warn!("⏰ Timeout alcanzado en búsqueda triangular");
                break;
            }
            
            // Limit max combinations
            if opportunities.len() >= self.config.max_combinations as usize {
                debug!("📊 Límite de combinaciones alcanzado");
                break;
            }
        }
        
        // Update statistics
        self.statistics.triangles_analyzed += triangles.len() as u64;
        self.statistics.opportunities_found += opportunities.len() as u64;
        
        info!("✅ Encontradas {} oportunidades triangulares de {} triángulos analizados", 
              opportunities.len(), triangles.len());
        
        Ok(opportunities)
    }
    
    /// Generar todos los triángulos posibles
    async fn generate_triangles(&self, price_feeds: &PriceFeedManager) -> CoreResult<Vec<TokenTriangle>> {
        let mut triangles = Vec::new();
        
        // Get available tokens from price feeds
        let available_tokens = self.get_available_tokens(price_feeds).await?;
        
        // Generate triangles using base tokens as anchors
        for base_token in &self.base_tokens {
            if !available_tokens.contains(base_token) {
                continue;
            }
            
            // Find other tokens that have pairs with the base token
            let paired_tokens: Vec<String> = available_tokens.iter()
                .filter(|&token| token != base_token && self.has_trading_pair(base_token, token, price_feeds).await.unwrap_or(false))
                .cloned()
                .collect();
            
            // Create triangles: base -> token1 -> token2 -> base
            for i in 0..paired_tokens.len() {
                for j in (i + 1)..paired_tokens.len() {
                    let token1 = &paired_tokens[i];
                    let token2 = &paired_tokens[j];
                    
                    // Check if token1 and token2 have a trading pair
                    if self.has_trading_pair(token1, token2, price_feeds).await.unwrap_or(false) {
                        if let Some(triangle) = self.create_triangle(base_token, token1, token2, price_feeds).await? {
                            triangles.push(triangle);
                        }
                    }
                }
            }
        }
        
        Ok(triangles)
    }
    
    /// Obtener tokens disponibles
    async fn get_available_tokens(&self, price_feeds: &PriceFeedManager) -> CoreResult<HashSet<String>> {
        let mut tokens = HashSet::new();
        
        // Get tokens from all DEX price feeds
        let dex_names = vec!["Raydium", "Orca", "Serum", "Phoenix"];
        
        for dex_name in dex_names {
            if let Ok(prices) = price_feeds.get_dex_prices(dex_name).await {
                for (token_symbol, _) in prices {
                    tokens.insert(token_symbol);
                }
            }
        }
        
        Ok(tokens)
    }
    
    /// Verificar si existe par de trading entre tokens
    async fn has_trading_pair(&self, token_a: &str, token_b: &str, price_feeds: &PriceFeedManager) -> CoreResult<bool> {
        // Check if any DEX has both tokens
        let dex_names = vec!["Raydium", "Orca", "Serum", "Phoenix"];
        
        for dex_name in dex_names {
            if let Ok(prices) = price_feeds.get_dex_prices(dex_name).await {
                let has_a = prices.contains_key(token_a);
                let has_b = prices.contains_key(token_b);
                
                if has_a && has_b {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    /// Crear triángulo de tokens
    async fn create_triangle(&self, token_a: &str, token_b: &str, token_c: &str, price_feeds: &PriceFeedManager) -> CoreResult<Option<TokenTriangle>> {
        // Find best DEXs for each pair
        let dex_ab = self.find_best_dex_for_pair(token_a, token_b, price_feeds).await?;
        let dex_bc = self.find_best_dex_for_pair(token_b, token_c, price_feeds).await?;
        let dex_ca = self.find_best_dex_for_pair(token_c, token_a, price_feeds).await?;
        
        if let (Some(dex_ab), Some(dex_bc), Some(dex_ca)) = (dex_ab, dex_bc, dex_ca) {
            let triangle = TokenTriangle {
                token_a: TokenInfo {
                    symbol: token_a.to_string(),
                    mint: solana_sdk::pubkey::Pubkey::default(), // TODO: Get actual mint
                    decimals: 9,
                    name: token_a.to_string(),
                },
                token_b: TokenInfo {
                    symbol: token_b.to_string(),
                    mint: solana_sdk::pubkey::Pubkey::default(),
                    decimals: 9,
                    name: token_b.to_string(),
                },
                token_c: TokenInfo {
                    symbol: token_c.to_string(),
                    mint: solana_sdk::pubkey::Pubkey::default(),
                    decimals: 9,
                    name: token_c.to_string(),
                },
                dex_ab,
                dex_bc,
                dex_ca,
            };
            
            Ok(Some(triangle))
        } else {
            Ok(None)
        }
    }
    
    /// Encontrar mejor DEX para un par de tokens
    async fn find_best_dex_for_pair(&self, token_a: &str, token_b: &str, price_feeds: &PriceFeedManager) -> CoreResult<Option<DexInfo>> {
        let dex_names = vec!["Raydium", "Orca", "Serum", "Phoenix"];
        let mut best_dex = None;
        let mut best_liquidity = 0.0;
        
        for dex_name in dex_names {
            if let Ok(prices) = price_feeds.get_dex_prices(dex_name).await {
                if let (Some(price_a), Some(price_b)) = (prices.get(token_a), prices.get(token_b)) {
                    // Use volume as liquidity proxy
                    let liquidity = price_a.volume.unwrap_or(0.0) + price_b.volume.unwrap_or(0.0);
                    
                    if liquidity > best_liquidity {
                        best_liquidity = liquidity;
                        best_dex = Some(DexInfo {
                            name: dex_name.to_string(),
                            program_id: solana_sdk::pubkey::Pubkey::default(), // TODO: Get actual program ID
                            fee_bps: 25, // Typical DEX fee
                        });
                    }
                }
            }
        }
        
        Ok(best_dex)
    }
    
    /// Analizar triángulo para oportunidades
    async fn analyze_triangle(&mut self, triangle: &TokenTriangle, price_feeds: &PriceFeedManager) -> CoreResult<Option<TriangularOpportunity>> {
        // Calculate the triangular arbitrage profit
        let profit_result = self.calculate_triangular_profit(triangle, price_feeds).await?;
        
        if let Some((profit_percentage, profit_sol, execution_steps)) = profit_result {
            if profit_percentage > self.config.min_profit_sol && profit_sol > self.config.min_profit_sol {
                let opportunity = TriangularOpportunity {
                    id: format!("tri_{}_{}_{}", 
                              triangle.token_a.symbol, 
                              triangle.token_b.symbol, 
                              triangle.token_c.symbol),
                    triangle: triangle.clone(),
                    expected_profit_sol: profit_sol,
                    profit_percentage,
                    confidence_score: self.calculate_confidence_score(triangle).await?,
                    execution_steps,
                    total_price_impact: self.calculate_total_price_impact(&execution_steps),
                    estimated_gas: 150000, // Estimated for 3 swaps
                    detected_at: chrono::Utc::now(),
                    expires_at: chrono::Utc::now() + chrono::Duration::seconds(20), // Shorter expiry for triangular
                    risk_score: self.calculate_risk_score(profit_percentage, profit_sol),
                };
                
                // Update cache
                self.update_triangle_cache(triangle, profit_percentage);
                
                return Ok(Some(opportunity));
            }
        }
        
        Ok(None)
    }
    
    /// Calcular profit del arbitraje triangular
    async fn calculate_triangular_profit(&self, triangle: &TokenTriangle, price_feeds: &PriceFeedManager) -> CoreResult<Option<(f64, f64, Vec<ExecutionStep>)>> {
        // Get prices for all three pairs
        let price_ab = self.get_pair_price(&triangle.token_a.symbol, &triangle.token_b.symbol, &triangle.dex_ab.name, price_feeds).await?;
        let price_bc = self.get_pair_price(&triangle.token_b.symbol, &triangle.token_c.symbol, &triangle.dex_bc.name, price_feeds).await?;
        let price_ca = self.get_pair_price(&triangle.token_c.symbol, &triangle.token_a.symbol, &triangle.dex_ca.name, price_feeds).await?;
        
        if let (Some(pab), Some(pbc), Some(pca)) = (price_ab, price_bc, price_ca) {
            // Start with 1.0 unit of token A
            let initial_amount = 1000000u64; // 0.001 if 9 decimals
            let amount_a_start = initial_amount as f64;
            
            // Step 1: A -> B
            let amount_b = amount_a_start * pab * 0.9975; // Account for 0.25% DEX fee
            let step1 = ExecutionStep {
                step_number: 1,
                action: "swap".to_string(),
                token_from: triangle.token_a.clone(),
                token_to: triangle.token_b.clone(),
                amount_in: initial_amount,
                amount_out: (amount_b as u64),
                dex: triangle.dex_ab.clone(),
                price_impact: 0.001, // Estimated
                estimated_time_ms: 2000,
            };
            
            // Step 2: B -> C
            let amount_c = amount_b * pbc * 0.9975;
            let step2 = ExecutionStep {
                step_number: 2,
                action: "swap".to_string(),
                token_from: triangle.token_b.clone(),
                token_to: triangle.token_c.clone(),
                amount_in: step1.amount_out,
                amount_out: (amount_c as u64),
                dex: triangle.dex_bc.clone(),
                price_impact: 0.001,
                estimated_time_ms: 2000,
            };
            
            // Step 3: C -> A
            let amount_a_final = amount_c * pca * 0.9975;
            let step3 = ExecutionStep {
                step_number: 3,
                action: "swap".to_string(),
                token_from: triangle.token_c.clone(),
                token_to: triangle.token_a.clone(),
                amount_in: step2.amount_out,
                amount_out: (amount_a_final as u64),
                dex: triangle.dex_ca.clone(),
                price_impact: 0.001,
                estimated_time_ms: 2000,
            };
            
            // Calculate profit
            let profit_amount = amount_a_final - amount_a_start;
            if profit_amount > 0.0 {
                let profit_percentage = profit_amount / amount_a_start;
                let profit_sol = self.convert_to_sol(profit_amount, &triangle.token_a.symbol).await?;
                
                let execution_steps = vec![step1, step2, step3];
                return Ok(Some((profit_percentage, profit_sol, execution_steps)));
            }
        }
        
        Ok(None)
    }
    
    /// Obtener precio de un par
    async fn get_pair_price(&self, token_from: &str, token_to: &str, dex_name: &str, price_feeds: &PriceFeedManager) -> CoreResult<Option<f64>> {
        if let Ok(prices) = price_feeds.get_dex_prices(dex_name).await {
            if let (Some(price_from), Some(price_to)) = (prices.get(token_from), prices.get(token_to)) {
                return Ok(Some(price_from.price / price_to.price));
            }
        }
        Ok(None)
    }
    
    /// Convertir cantidad a SOL
    async fn convert_to_sol(&self, amount: f64, token: &str) -> CoreResult<f64> {
        if token == "SOL" {
            return Ok(amount / 1e9);
        }
        
        // For simplicity, assume 1 token unit = 0.001 SOL
        // In reality, you'd get the actual token/SOL price
        Ok(amount * 0.001 / 1e9)
    }
    
    /// Calcular score de confianza
    async fn calculate_confidence_score(&self, triangle: &TokenTriangle) -> CoreResult<f64> {
        let mut confidence = 0.7; // Base confidence
        
        // Check if triangle is in cache with good performance
        let triangle_key = format!("{}_{}_{}", 
                                  triangle.token_a.symbol, 
                                  triangle.token_b.symbol, 
                                  triangle.token_c.symbol);
        
        if let Some(cache_entry) = self.triangle_cache.get(&triangle_key) {
            let success_rate = cache_entry.success_count as f64 / cache_entry.total_attempts as f64;
            confidence = confidence * 0.5 + success_rate * 0.5;
        }
        
        // Higher confidence for base tokens
        if self.base_tokens.contains(&triangle.token_a.symbol) ||
           self.base_tokens.contains(&triangle.token_b.symbol) ||
           self.base_tokens.contains(&triangle.token_c.symbol) {
            confidence += 0.1;
        }
        
        Ok(confidence.min(1.0))
    }
    
    /// Calcular impacto total de precio
    fn calculate_total_price_impact(&self, steps: &[ExecutionStep]) -> f64 {
        steps.iter().map(|step| step.price_impact).sum()
    }
    
    /// Calcular score de riesgo
    fn calculate_risk_score(&self, profit_percentage: f64, profit_sol: f64) -> f64 {
        let mut risk = 0.6; // Higher base risk for triangular (more complex)
        
        if profit_percentage > 0.01 { // > 1%
            risk -= 0.1;
        }
        
        if profit_sol > 0.005 { // > 0.005 SOL
            risk -= 0.1;
        }
        
        // Higher risk for very high profits
        if profit_percentage > 0.05 { // > 5%
            risk += 0.2;
        }
        
        risk.max(0.0).min(1.0)
    }
    
    /// Actualizar cache de triángulos
    fn update_triangle_cache(&mut self, triangle: &TokenTriangle, profit: f64) {
        let key = format!("{}_{}_{}", 
                         triangle.token_a.symbol, 
                         triangle.token_b.symbol, 
                         triangle.token_c.symbol);
        
        if let Some(cache_entry) = self.triangle_cache.get_mut(&key) {
            cache_entry.last_profit = profit;
            cache_entry.last_updated = Instant::now();
            cache_entry.total_attempts += 1;
        } else {
            self.triangle_cache.insert(key, TriangleCache {
                triangle: triangle.clone(),
                last_profit: profit,
                last_updated: Instant::now(),
                success_count: 0,
                total_attempts: 1,
            });
        }
    }
    
    /// Convertir a oportunidad de arbitraje
    async fn convert_to_arbitrage_opportunity(&self, triangular_opp: &TriangularOpportunity) -> CoreResult<Option<ArbitrageOpportunity>> {
        // Convert triangular opportunity to standard arbitrage opportunity
        // Use the first execution step as the primary trade
        if let Some(first_step) = triangular_opp.execution_steps.first() {
            let opportunity = ArbitrageOpportunity {
                id: triangular_opp.id.clone(),
                token_in: first_step.token_from.clone(),
                token_out: triangular_opp.triangle.token_a.clone(), // Final token
                amount_in: first_step.amount_in,
                amount_out: triangular_opp.execution_steps.last().unwrap().amount_out,
                profit_percentage: triangular_opp.profit_percentage,
                expected_profit_sol: triangular_opp.expected_profit_sol,
                confidence_score: triangular_opp.confidence_score,
                route_info: RouteInfo {
                    hops: 3,
                    route_plan: vec![
                        triangular_opp.triangle.dex_ab.name.clone(),
                        triangular_opp.triangle.dex_bc.name.clone(),
                        triangular_opp.triangle.dex_ca.name.clone(),
                    ],
                    estimated_gas: triangular_opp.estimated_gas,
                    price_impact: triangular_opp.total_price_impact,
                },
                dex_from: first_step.dex.clone(),
                dex_to: triangular_opp.execution_steps.last().unwrap().dex.clone(),
                price_impact: triangular_opp.total_price_impact,
                execution_time_estimate_ms: triangular_opp.execution_steps.iter()
                    .map(|step| step.estimated_time_ms)
                    .sum(),
                detected_at: triangular_opp.detected_at,
                expires_at: triangular_opp.expires_at,
                risk_score: triangular_opp.risk_score,
                gas_estimate: triangular_opp.estimated_gas,
            };
            
            Ok(Some(opportunity))
        } else {
            Ok(None)
        }
    }
    
    /// Obtener estadísticas
    pub fn get_statistics(&self) -> &TriangularStatistics {
        &self.statistics
    }
    
    /// Reportar éxito de ejecución
    pub fn report_execution_success(&mut self, triangle_id: &str) {
        if let Some(cache_entry) = self.triangle_cache.values_mut()
            .find(|entry| format!("{}_{}_{}", 
                                 entry.triangle.token_a.symbol,
                                 entry.triangle.token_b.symbol,
                                 entry.triangle.token_c.symbol) == triangle_id) {
            cache_entry.success_count += 1;
        }
        
        self.statistics.successful_executions += 1;
    }
}
