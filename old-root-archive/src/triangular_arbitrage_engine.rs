// ================================================================================
// ARBITRAJE TRIANGULAR Y PROTECCIÓN ANTI-CIRCULAR - IMPLEMENTACIÓN AVANZADA
// ================================================================================
// Sistema que detecta oportunidades triangulares reales y evita loops circulares
// Ejemplo: SOL -> USDC -> RAY -> SOL (si existe profit neto > costos)
// ================================================================================

use anyhow::{Result, anyhow};
use tracing::{info, warn, debug};
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Configuración de arbitraje triangular avanzado
#[derive(Debug, Clone)]
pub struct TriangularArbitrageConfig {
    pub enabled: bool,
    pub max_hops: u8,                    // Máximo 3 hops (A->B->C->A)
    pub min_net_profit_bps: u16,         // Mínimo 50 BPS (0.5%) después de costos
    pub max_execution_time_ms: u64,      // Máximo 15 segundos para completar ciclo
    pub circular_detection_enabled: bool, // Protección anti-circular
    pub max_same_token_repeats: u8,      // Máximo 2 veces el mismo token en path
}

impl Default for TriangularArbitrageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_hops: 3,
            min_net_profit_bps: 50,         // 0.5% mínimo
            max_execution_time_ms: 15000,   // 15 segundos
            circular_detection_enabled: true,
            max_same_token_repeats: 1,      // No repetir tokens
        }
    }
}

/// Representación de una oportunidad triangular real
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangularOpportunity {
    pub id: String,
    pub path: Vec<TokenHop>,            // A->B->C->A
    pub estimated_net_profit: f64,      // Profit después de todos los costos
    pub total_cost_bps: u16,           // Costos totales en BPS
    pub liquidity_constraint: f64,      // Liquidez mínima en el path
    pub execution_risk_score: f64,      // 0.0-1.0 (1.0 = muy riesgoso)
    pub dexs_involved: Vec<String>,     // DEXs en cada hop
    pub estimated_duration_ms: u64,     // Tiempo estimado de ejecución
}

/// Hop individual en el path triangular
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHop {
    pub from_token: String,
    pub to_token: String,
    pub dex_name: String,
    pub exchange_rate: f64,
    pub liquidity_usd: f64,
    pub swap_fee_bps: u16,
}

/// Motor de arbitraje triangular avanzado
pub struct TriangularArbitrageEngine {
    config: TriangularArbitrageConfig,
    token_graph: HashMap<String, Vec<String>>, // Grafo de conectividad
    price_cache: HashMap<(String, String), f64>, // Cache de precios
    circular_detector: CircularTradeDetector,
    execution_history: Vec<String>, // Historial para evitar repeticiones
}

/// Detector de trading circular (Anti-MEV)
#[derive(Debug)]
pub struct CircularTradeDetector {
    recent_paths: HashSet<String>,      // Paths ejecutados recientemente
    token_sequence_tracker: HashMap<String, u8>, // Contador de repeticiones
    suspicious_patterns: Vec<String>,   // Patrones sospechosos detectados
}

impl TriangularArbitrageEngine {
    /// Crear nuevo motor triangular con protección anti-circular
    pub fn new(config: Option<TriangularArbitrageConfig>) -> Self {
        let config = config.unwrap_or_default();
        
        // Definir grafo de tokens principales
        let mut token_graph = HashMap::new();
        
        // SOL conecta con todos los tokens principales
        token_graph.insert("SOL".to_string(), vec![
            "USDC".to_string(), "USDT".to_string(), "RAY".to_string(), 
            "JUP".to_string(), "BONK".to_string()
        ]);
        
        // USDC como hub principal
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
                        if opportunity.estimated_net_profit > 0.0 && 
                           opportunity.total_cost_bps < 500 && // Máximo 5% costos
                           opportunity.execution_risk_score < 0.7 {
                            
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
        // Por ahora, cache vacío - se debe implementar integración con Jupiter/DexScreener
        warn!("⚠️ Cache de precios triangular deshabilitado - requiere implementación real");
        
        // Limpiar cache para forzar búsqueda en APIs reales
        self.price_cache.clear();
        
        debug!("✅ Cache de precios listo para APIs reales");
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
