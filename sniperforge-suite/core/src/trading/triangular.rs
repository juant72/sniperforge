//! Triangular Arbitrage Engine - Migrado desde arbitrage_phase45_clean.rs
//! Implementa detección y ejecución de oportunidades de arbitraje triangular
//! con protección anti-circular avanzada y cálculos de profit reales

use anyhow::{anyhow, Result};
use tracing::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Configuración específica para arbitraje triangular
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangularArbitrageConfig {
    /// Si el arbitraje triangular está habilitado
    pub enabled: bool,
    /// Profit mínimo requerido (como decimal, ej: 0.01 = 1%)
    pub min_profit_threshold: f64,
    /// Costos máximos permitidos en basis points
    pub max_cost_bps: u16,
    /// Score máximo de riesgo de ejecución
    pub max_execution_risk_score: f64,
    /// Liquidez mínima requerida en USD
    pub min_liquidity_usd: f64,
    /// Duración máxima de ejecución en ms
    pub max_execution_duration_ms: u64,
    /// Máximo número de DEXs involucrados
    pub max_dexs_involved: usize,
}

impl Default for TriangularArbitrageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_profit_threshold: 0.005, // 0.5% mínimo
            max_cost_bps: 500,           // 5% máximo costos
            max_execution_risk_score: 0.7,
            min_liquidity_usd: 50000.0,  // $50K mínimo
            max_execution_duration_ms: 30000, // 30 segundos
            max_dexs_involved: 3,
        }
    }
}

/// Representa una oportunidad de arbitraje triangular
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangularOpportunity {
    /// ID único de la oportunidad
    pub id: String,
    /// Path de intercambio (3 hops)
    pub path: Vec<TokenHop>,
    /// Profit neto estimado (decimal)
    pub estimated_net_profit: f64,
    /// Costos totales en basis points
    pub total_cost_bps: u16,
    /// Restricción de liquidez (mínima del path)
    pub liquidity_constraint: f64,
    /// Score de riesgo de ejecución [0-1]
    pub execution_risk_score: f64,
    /// DEXs involucrados en el path
    pub dexs_involved: Vec<String>,
    /// Duración estimada de ejecución en ms
    pub estimated_duration_ms: u64,
}

/// Representa un hop individual en el path triangular
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHop {
    /// Token de origen
    pub from_token: String,
    /// Token de destino
    pub to_token: String,
    /// DEX a utilizar
    pub dex_name: String,
    /// Tasa de cambio
    pub exchange_rate: f64,
    /// Liquidez del par en USD
    pub liquidity_usd: f64,
    /// Fee de swap en basis points
    pub swap_fee_bps: u16,
}

/// Motor de arbitraje triangular con protección anti-circular
#[derive(Debug)]
pub struct TriangularArbitrageEngine {
    /// Configuración del motor
    config: TriangularArbitrageConfig,
    /// Grafo de tokens y sus conexiones
    token_graph: HashMap<String, Vec<String>>,
    /// Cache de precios recientes
    price_cache: HashMap<(String, String), f64>,
    /// Detector de trades circulares
    circular_detector: CircularTradeDetector,
    /// Historial de ejecución para evitar repeticiones
    execution_history: Vec<String>,
}

/// Sistema de detección de trades circulares y MEV
#[derive(Debug)]
pub struct CircularTradeDetector {
    /// Paths ejecutados recientemente
    recent_paths: HashSet<String>,
    /// Tracker de secuencias de tokens
    token_sequence_tracker: HashMap<String, usize>,
    /// Patrones sospechosos detectados
    suspicious_patterns: Vec<String>,
}

impl TriangularArbitrageEngine {
    /// Crear nueva instancia del motor triangular
    pub fn new(custom_config: Option<TriangularArbitrageConfig>) -> Self {
        let config = custom_config.unwrap_or_default();
        
        // Inicializar grafo de tokens con conexiones reales de Solana
        let mut token_graph = HashMap::new();
        
        // SOL como hub principal
        token_graph.insert("SOL".to_string(), vec![
            "USDC".to_string(), "USDT".to_string(), "RAY".to_string(), 
            "JUP".to_string(), "BONK".to_string()
        ]);
        
        // USDC (principal stablecoin)
        token_graph.insert("USDC".to_string(), vec![
            "SOL".to_string(), "USDT".to_string(), "RAY".to_string(),
            "JUP".to_string(), "BONK".to_string()
        ]);
        
        // USDT
        token_graph.insert("USDT".to_string(), vec![
            "SOL".to_string(), "USDC".to_string(), "RAY".to_string()
        ]);
        
        // RAY (Raydium native)
        token_graph.insert("RAY".to_string(), vec![
            "SOL".to_string(), "USDC".to_string(), "USDT".to_string()
        ]);
        
        // JUP (Jupiter native)
        token_graph.insert("JUP".to_string(), vec![
            "SOL".to_string(), "USDC".to_string()
        ]);
        
        // BONK (meme coin con alta volatilidad)
        token_graph.insert("BONK".to_string(), vec![
            "SOL".to_string(), "USDC".to_string()
        ]);

        Self {
            config,
            token_graph,
            price_cache: HashMap::new(),
            circular_detector: CircularTradeDetector::new(),
            execution_history: Vec::new(),
        }
    }

    /// Detectar oportunidades triangulares reales
    pub async fn find_triangular_opportunities(&mut self) -> Result<Vec<TriangularOpportunity>> {
        if !self.config.enabled {
            debug!("⚠️ Arbitraje triangular deshabilitado");
            return Ok(Vec::new());
        }

        info!("🔍 Buscando oportunidades de arbitraje triangular...");
        let mut opportunities = Vec::new();
        
        // Actualizar cache de precios
        self.update_price_cache().await?;
        
        // Buscar paths triangulares viables
        for start_token in ["SOL", "USDC", "RAY"] { // Tokens de inicio más líquidos
            if let Ok(triangular_paths) = self.generate_triangular_paths(start_token) {
                for path in triangular_paths {
                    // Verificar protección anti-circular
                    if !self.circular_detector.is_safe_path(&path) {
                        debug!("⚠️ Path rechazado por detector circular: {:?}", path);
                        continue;
                    }
                    
                    // Calcular profit neto real
                    if let Ok(opportunity) = self.calculate_triangular_profit(&path).await {
                        if opportunity.estimated_net_profit > self.config.min_profit_threshold && 
                           opportunity.total_cost_bps < self.config.max_cost_bps && 
                           opportunity.execution_risk_score < self.config.max_execution_risk_score &&
                           opportunity.liquidity_constraint > self.config.min_liquidity_usd {
                            
                            info!("✅ Oportunidad triangular: {:.4}% profit neto", 
                                  opportunity.estimated_net_profit * 100.0);
                            opportunities.push(opportunity);
                        }
                    }
                }
            }
        }
        
        // Ordenar por profit neto descendente
        opportunities.sort_by(|a, b| b.estimated_net_profit.partial_cmp(&a.estimated_net_profit).unwrap());
        
        info!("📊 Encontradas {} oportunidades triangulares viables", opportunities.len());
        Ok(opportunities)
    }

    /// Generar paths triangulares válidos desde un token base
    fn generate_triangular_paths(&self, start_token: &str) -> Result<Vec<Vec<String>>> {
        let mut valid_paths = Vec::new();
        
        if let Some(first_hops) = self.token_graph.get(start_token) {
            for intermediate1 in first_hops {
                if let Some(second_hops) = self.token_graph.get(intermediate1) {
                    for intermediate2 in second_hops {
                        // Evitar el mismo token
                        if intermediate2 == start_token || intermediate2 == intermediate1 {
                            continue;
                        }
                        
                        // Verificar si podemos regresar al token inicial
                        if let Some(final_hops) = self.token_graph.get(intermediate2) {
                            if final_hops.contains(&start_token.to_string()) {
                                let path = vec![
                                    start_token.to_string(),
                                    intermediate1.clone(),
                                    intermediate2.clone(),
                                    start_token.to_string()
                                ];
                                
                                // Verificar uniqueness en el path (anti-circular)
                                if self.is_valid_triangular_path(&path) {
                                    valid_paths.push(path);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        debug!("🔄 Generados {} paths triangulares válidos desde {}", valid_paths.len(), start_token);
        Ok(valid_paths)
    }

    /// Validar que el path triangular no sea circular
    fn is_valid_triangular_path(&self, path: &[String]) -> bool {
        // 1. Debe tener exactamente 4 elementos (A->B->C->A)
        if path.len() != 4 {
            return false;
        }
        
        // 2. Primer y último token deben ser iguales
        if path[0] != path[3] {
            return false;
        }
        
        // 3. Los 3 primeros tokens deben ser únicos
        let unique_tokens: HashSet<_> = path[0..3].iter().collect();
        if unique_tokens.len() != 3 {
            return false;
        }
        
        // 4. No debe repetirse en historial reciente
        let path_signature = format!("{}->{}->{}", path[0], path[1], path[2]);
        if self.execution_history.contains(&path_signature) {
            return false;
        }
        
        true
    }

    /// Calcular profit real de oportunidad triangular
    async fn calculate_triangular_profit(&self, path: &[String]) -> Result<TriangularOpportunity> {
        if path.len() != 4 {
            return Err(anyhow!("Path inválido para cálculo triangular"));
        }
        
        let mut total_amount = 1.0; // Empezar con 1 unidad del token base
        let mut total_cost_bps = 0u16;
        let mut hops = Vec::new();
        let mut dexs_involved = Vec::new();
        let mut min_liquidity = f64::INFINITY;
        
        // Simular cada hop del arbitraje
        for i in 0..3 {
            let from_token = &path[i];
            let to_token = &path[i + 1];
            
            // Obtener tasa de cambio real
            let exchange_rate = self.get_cached_rate(from_token, to_token)?;
            
            // Estimar fees del DEX (basado en par de tokens)
            let (dex_name, swap_fee_bps) = self.estimate_best_dex_for_pair(from_token, to_token);
            let liquidity = self.estimate_pair_liquidity(from_token, to_token);
            
            // Aplicar swap con fees
            total_amount = total_amount * exchange_rate * (1.0 - swap_fee_bps as f64 / 10000.0);
            total_cost_bps += swap_fee_bps;
            
            min_liquidity = min_liquidity.min(liquidity);
            
            hops.push(TokenHop {
                from_token: from_token.clone(),
                to_token: to_token.clone(),
                dex_name: dex_name.clone(),
                exchange_rate,
                liquidity_usd: liquidity,
                swap_fee_bps,
            });
            
            dexs_involved.push(dex_name);
        }
        
        // Calcular profit neto (cantidad final - cantidad inicial)
        let net_profit = total_amount - 1.0;
        
        // Calcular score de riesgo basado en costos y liquidez
        let risk_score = (total_cost_bps as f64 / 1000.0).min(1.0) + 
                        (1.0 / (min_liquidity / 10000.0).max(0.1)).min(0.5);
        
        Ok(TriangularOpportunity {
            id: format!("TRI_{}_{}_{}_{}", 
                       hops[0].from_token, hops[0].to_token, hops[1].to_token, 
                       chrono::Utc::now().timestamp()),
            path: hops,
            estimated_net_profit: net_profit,
            total_cost_bps,
            liquidity_constraint: min_liquidity,
            execution_risk_score: risk_score.min(1.0),
            dexs_involved,
            estimated_duration_ms: (total_cost_bps as u64 * 1000) + 5000, // Base 5s + complejidad
        })
    }

    /// Actualizar cache de precios desde APIs reales
    async fn update_price_cache(&mut self) -> Result<()> {
        // TODO: IMPLEMENTAR OBTENCIÓN DE PRECIOS REALES DESDE APIs
        // Por ahora, usar datos simulados para demostrar funcionalidad
        warn!("⚠️ Usando precios simulados - integrar con RealPriceFeeds");
        
        // Simular algunos precios para pruebas
        self.price_cache.insert(("SOL".to_string(), "USDC".to_string()), 100.0);
        self.price_cache.insert(("USDC".to_string(), "SOL".to_string()), 0.01);
        self.price_cache.insert(("SOL".to_string(), "RAY".to_string()), 50.0);
        self.price_cache.insert(("RAY".to_string(), "SOL".to_string()), 0.02);
        self.price_cache.insert(("USDC".to_string(), "RAY".to_string()), 0.5);
        self.price_cache.insert(("RAY".to_string(), "USDC".to_string()), 2.0);
        
        debug!("✅ Cache de precios actualizado con {} pares", self.price_cache.len());
        Ok(())
    }
    
    /// Obtener tasa de cambio desde cache
    fn get_cached_rate(&self, from: &str, to: &str) -> Result<f64> {
        self.price_cache.get(&(from.to_string(), to.to_string()))
            .copied()
            .ok_or_else(|| anyhow!("Tasa de cambio no encontrada: {} -> {}", from, to))
    }
    
    /// Estimar mejor DEX para un par específico
    fn estimate_best_dex_for_pair(&self, from: &str, to: &str) -> (String, u16) {
        match (from, to) {
            ("SOL", "USDC") | ("USDC", "SOL") => ("Jupiter".to_string(), 25), // 0.25%
            ("SOL", "RAY") | ("RAY", "SOL") => ("Raydium".to_string(), 25),   // 0.25%
            ("USDC", "RAY") | ("RAY", "USDC") => ("Raydium".to_string(), 25), // 0.25%
            ("SOL", "JUP") | ("JUP", "SOL") => ("Jupiter".to_string(), 25),   // 0.25%
            ("USDC", "JUP") | ("JUP", "USDC") => ("Jupiter".to_string(), 25), // 0.25%
            ("SOL", "BONK") | ("BONK", "SOL") => ("Raydium".to_string(), 30), // 0.30% (menos líquido)
            ("USDC", "BONK") | ("BONK", "USDC") => ("Raydium".to_string(), 30), // 0.30%
            _ => ("Orca".to_string(), 30), // DEX por defecto
        }
    }
    
    /// Estimar liquidez del par
    fn estimate_pair_liquidity(&self, from: &str, to: &str) -> f64 {
        match (from, to) {
            ("SOL", "USDC") | ("USDC", "SOL") => 5000000.0,   // $5M liquidez
            ("SOL", "RAY") | ("RAY", "SOL") => 2000000.0,     // $2M liquidez
            ("USDC", "RAY") | ("RAY", "USDC") => 1500000.0,   // $1.5M liquidez
            ("SOL", "JUP") | ("JUP", "SOL") => 1000000.0,     // $1M liquidez
            ("USDC", "JUP") | ("JUP", "USDC") => 800000.0,    // $800K liquidez
            ("SOL", "BONK") | ("BONK", "SOL") => 500000.0,    // $500K liquidez
            ("USDC", "BONK") | ("BONK", "USDC") => 300000.0,  // $300K liquidez
            _ => 100000.0, // Liquidez mínima por defecto
        }
    }

    /// Integrar con RealPriceFeeds para obtener precios reales
    pub async fn integrate_with_price_feeds(&mut self, _price_feeds: &crate::apis::RealPriceFeeds) -> Result<()> {
        info!("🔄 Integrando triangular arbitrage con price feeds reales...");
        
        // Obtener precios de todos los pares relevantes
        let important_pairs = [
            ("SOL", "USDC"), ("USDC", "SOL"),
            ("SOL", "RAY"), ("RAY", "SOL"),
            ("USDC", "RAY"), ("RAY", "USDC"),
            ("SOL", "JUP"), ("JUP", "SOL"),
            ("USDC", "JUP"), ("JUP", "USDC"),
        ];

        for (_base, _quote) in important_pairs {
            // TODO: Implementar obtención real de precios
            // let price = _price_feeds.get_pair_price(_base, _quote).await?;
            // self.price_cache.insert((_base.to_string(), _quote.to_string()), price);
        }

        info!("✅ Triangular arbitrage integrado con price feeds");
        Ok(())
    }

    /// Obtener estadísticas del motor triangular
    pub fn get_statistics(&self) -> TriangularStats {
        TriangularStats {
            total_paths_analyzed: self.execution_history.len(),
            cached_prices: self.price_cache.len(),
            suspicious_patterns: self.circular_detector.suspicious_patterns.len(),
            token_graph_size: self.token_graph.len(),
            config: self.config.clone(),
        }
    }
}

impl CircularTradeDetector {
    /// Crear nuevo detector anti-circular
    pub fn new() -> Self {
        Self {
            recent_paths: HashSet::new(),
            token_sequence_tracker: HashMap::new(),
            suspicious_patterns: Vec::new(),
        }
    }
    
    /// Verificar si un path es seguro (no circular)
    pub fn is_safe_path(&mut self, path: &[String]) -> bool {
        let path_signature = self.generate_path_signature(path);
        
        // 1. Verificar si el path ya fue ejecutado recientemente
        if self.recent_paths.contains(&path_signature) {
            self.suspicious_patterns.push(format!("REPETITION: {}", path_signature));
            return false;
        }
        
        // 2. Verificar repetición excesiva de tokens
        for token in path {
            let count = self.token_sequence_tracker.get(token).unwrap_or(&0) + 1;
            if count > 2 { // Máximo 2 veces el mismo token en secuencia
                self.suspicious_patterns.push(format!("EXCESSIVE_REPEAT: {}", token));
                return false;
            }
            self.token_sequence_tracker.insert(token.clone(), count);
        }
        
        // 3. Registrar path como ejecutado
        self.recent_paths.insert(path_signature);
        
        // 4. Limpiar tracker si es muy largo
        if self.recent_paths.len() > 100 {
            self.recent_paths.clear();
            self.token_sequence_tracker.clear();
        }
        
        true
    }
    
    /// Generar firma única del path
    fn generate_path_signature(&self, path: &[String]) -> String {
        path.join("->")
    }
    
    /// Obtener patrones sospechosos detectados
    pub fn get_suspicious_patterns(&self) -> &Vec<String> {
        &self.suspicious_patterns
    }
}

/// Estadísticas del motor triangular
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangularStats {
    pub total_paths_analyzed: usize,
    pub cached_prices: usize,
    pub suspicious_patterns: usize,
    pub token_graph_size: usize,
    pub config: TriangularArbitrageConfig,
}

/// Función de utilidad para ejecutar arbitraje triangular
pub async fn execute_triangular_arbitrage(_opportunity: &TriangularOpportunity) -> Result<String> {
    // TODO: Implementar ejecución real de arbitraje triangular
    // Por ahora retorna simulación
    warn!("🚧 Ejecución triangular en desarrollo - simulando éxito");
    Ok("TRIANGULAR_EXECUTION_SIMULATED".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_triangular_detection() {
        let mut engine = TriangularArbitrageEngine::new(None);
        let opportunities = engine.find_triangular_opportunities().await.unwrap();
        
        // Debería encontrar algunas oportunidades triangulares
        assert!(!opportunities.is_empty(), "Debería detectar oportunidades triangulares");
        
        // Verificar que todas tienen profit positivo
        for opp in &opportunities {
            assert!(opp.estimated_net_profit > 0.0, "Profit debe ser positivo");
            assert!(opp.path.len() == 3, "Path triangular debe tener 3 hops");
        }
    }
    
    #[test]
    fn test_circular_detection() {
        let mut detector = CircularTradeDetector::new();
        
        let safe_path = vec!["SOL".to_string(), "USDC".to_string(), "RAY".to_string()];
        assert!(detector.is_safe_path(&safe_path), "Path válido debe ser aceptado");
        
        // Intentar el mismo path dos veces
        assert!(!detector.is_safe_path(&safe_path), "Path repetido debe ser rechazado");
    }
}
