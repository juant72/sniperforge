// ================================================================================
// PHASE 11: ECOSYSTEM EXPANSION
// ================================================================================

use std::time::Instant;
use std::collections::{HashMap, HashSet};
use tracing::{info, debug, warn};
use async_trait::async_trait;

use sniperforge_core::CoreResult;
use crate::engine::ArbitrageOpportunity;
use crate::config::EcosystemConfig;
use crate::main::PhaseManager;
use super::{Phase, PhaseResult};

/// Phase 11: Expansión del ecosistema
pub struct EcosystemPhase {
    config: EcosystemConfig,
    ecosystem_manager: EcosystemManager,
    cross_chain_bridge: CrossChainBridge,
    network_analyzer: NetworkAnalyzer,
    ecosystem_multiplier: EcosystemMultiplier,
    statistics: EcosystemStatistics,
}

/// Gestor del ecosistema
#[derive(Debug)]
struct EcosystemManager {
    active_networks: HashMap<String, NetworkInfo>,
    ecosystem_connections: Vec<EcosystemConnection>,
    expansion_opportunities: Vec<ExpansionOpportunity>,
    ecosystem_health: f64,
}

/// Información de red
#[derive(Debug, Clone)]
struct NetworkInfo {
    name: String,
    chain_id: String,
    native_token: String,
    total_tvl: f64,
    active_protocols: u32,
    avg_fees: f64,
    network_status: NetworkStatus,
    integration_level: IntegrationLevel,
}

/// Estado de la red
#[derive(Debug, Clone)]
enum NetworkStatus {
    Active,
    Congested,
    Maintenance,
    Inactive,
}

/// Nivel de integración
#[derive(Debug, Clone)]
enum IntegrationLevel {
    Native,     // Full integration
    Bridge,     // Cross-chain bridge
    Oracle,     // Price oracle only
    Monitor,    // Monitoring only
}

/// Conexión del ecosistema
#[derive(Debug, Clone)]
struct EcosystemConnection {
    from_network: String,
    to_network: String,
    connection_type: ConnectionType,
    efficiency: f64,
    cost: f64,
    reliability: f64,
}

/// Tipo de conexión
#[derive(Debug, Clone)]
enum ConnectionType {
    NativeBridge,
    ThirdPartyBridge,
    AtomicSwap,
    Oracle,
}

/// Oportunidad de expansión
#[derive(Debug, Clone)]
struct ExpansionOpportunity {
    target_network: String,
    opportunity_type: ExpansionType,
    potential_profit: f64,
    integration_cost: f64,
    time_to_implement: u32, // days
    risk_score: f64,
}

/// Tipo de expansión
#[derive(Debug, Clone)]
enum ExpansionType {
    CrossChainArbitrage,
    NewDexIntegration,
    TokenExpansion,
    ProtocolIntegration,
}

/// Puente cross-chain
#[derive(Debug)]
struct CrossChainBridge {
    supported_chains: HashSet<String>,
    bridge_contracts: HashMap<String, BridgeContract>,
    active_bridges: Vec<ActiveBridge>,
    bridge_statistics: BridgeStatistics,
}

/// Contrato de puente
#[derive(Debug, Clone)]
struct BridgeContract {
    contract_address: String,
    source_chain: String,
    target_chain: String,
    supported_tokens: Vec<String>,
    fees: BridgeFees,
    security_level: SecurityLevel,
}

/// Fees del puente
#[derive(Debug, Clone)]
struct BridgeFees {
    base_fee: f64,
    percentage_fee: f64,
    gas_estimate: u64,
}

/// Nivel de seguridad
#[derive(Debug, Clone)]
enum SecurityLevel {
    High,
    Medium,
    Low,
}

/// Puente activo
#[derive(Debug, Clone)]
struct ActiveBridge {
    bridge_id: String,
    transaction_hash: String,
    status: BridgeStatus,
    amount: f64,
    start_time: chrono::DateTime<chrono::Utc>,
    estimated_completion: chrono::DateTime<chrono::Utc>,
}

/// Estado del puente
#[derive(Debug, Clone)]
enum BridgeStatus {
    Initiated,
    Confirming,
    Bridging,
    Completed,
    Failed,
}

/// Estadísticas del puente
#[derive(Debug, Clone, Default)]
struct BridgeStatistics {
    total_bridges: u64,
    successful_bridges: u64,
    total_volume: f64,
    avg_bridge_time: f64,
    total_fees_paid: f64,
}

/// Analizador de red
#[derive(Debug)]
struct NetworkAnalyzer {
    network_metrics: HashMap<String, NetworkMetrics>,
    trend_analysis: TrendAnalysis,
    performance_benchmarks: PerformanceBenchmarks,
}

/// Métricas de red
#[derive(Debug, Clone)]
struct NetworkMetrics {
    tps: f64,              // Transactions per second
    finality_time: f64,    // seconds
    avg_fee: f64,          // in native token
    tvl: f64,              // Total value locked
    active_addresses: u64,
    protocol_count: u32,
}

/// Análisis de tendencias
#[derive(Debug, Clone)]
struct TrendAnalysis {
    growth_rate: f64,
    volatility: f64,
    adoption_trend: AdoptionTrend,
    innovation_score: f64,
}

/// Tendencia de adopción
#[derive(Debug, Clone)]
enum AdoptionTrend {
    Rapid,
    Steady,
    Slow,
    Declining,
}

/// Benchmarks de rendimiento
#[derive(Debug, Clone)]
struct PerformanceBenchmarks {
    best_performing_network: String,
    fastest_network: String,
    most_cost_effective: String,
    highest_tvl: String,
}

/// Multiplicador del ecosistema
#[derive(Debug)]
struct EcosystemMultiplier {
    network_effect_bonus: f64,
    cross_chain_synergy: f64,
    ecosystem_diversity_score: f64,
    total_ecosystem_value: f64,
}

/// Estadísticas del ecosistema
#[derive(Debug, Clone, Default)]
struct EcosystemStatistics {
    networks_integrated: u32,
    cross_chain_arbitrages: u64,
    ecosystem_profit_bonus: f64,
    network_effect_contributions: f64,
    expansion_opportunities_identified: u32,
    total_ecosystem_volume: f64,
}

impl EcosystemPhase {
    /// Crear nueva fase de ecosistema
    pub fn new(config: &EcosystemConfig) -> CoreResult<Self> {
        info!("🌐 Inicializando Ecosystem Phase 11");
        
        Ok(Self {
            config: config.clone(),
            ecosystem_manager: EcosystemManager::new(config)?,
            cross_chain_bridge: CrossChainBridge::new(),
            network_analyzer: NetworkAnalyzer::new(),
            ecosystem_multiplier: EcosystemMultiplier::new(config),
            statistics: EcosystemStatistics::default(),
        })
    }
    
    /// Procesar con expansión del ecosistema
    async fn process_with_ecosystem_expansion(&mut self, opportunities: &mut Vec<ArbitrageOpportunity>) -> CoreResult<()> {
        debug!("🌐 Procesando con expansión del ecosistema");
        
        // 1. Analizar oportunidades cross-chain si está habilitado
        if self.config.cross_chain_enabled {
            self.analyze_cross_chain_opportunities(opportunities).await?;
        }
        
        // 2. Aplicar multiplicador de ecosistema
        self.apply_ecosystem_multiplier(opportunities).await?;
        
        // 3. Calcular efecto de red
        self.calculate_network_effects(opportunities).await?;
        
        // 4. Identificar oportunidades de expansión
        self.identify_expansion_opportunities().await?;
        
        // 5. Optimizar para múltiples redes
        self.optimize_for_multi_network(opportunities).await?;
        
        Ok(())
    }
    
    /// Analizar oportunidades cross-chain
    async fn analyze_cross_chain_opportunities(&mut self, opportunities: &mut Vec<ArbitrageOpportunity>) -> CoreResult<()> {
        debug!("🔗 Analizando oportunidades cross-chain");
        
        for opportunity in opportunities.iter_mut() {
            // Buscar oportunidades en otras chains
            let cross_chain_potential = self.evaluate_cross_chain_potential(opportunity).await?;
            
            if cross_chain_potential.profit_increase > 0.01 { // 1% minimum improvement
                // Aplicar bonus cross-chain
                opportunity.expected_profit_sol += cross_chain_potential.profit_increase;
                opportunity.confidence_score *= cross_chain_potential.confidence_factor;
                
                // Ajustar tiempo de ejecución (cross-chain toma más tiempo)
                opportunity.execution_time_estimate_ms += cross_chain_potential.additional_time_ms;
                
                self.statistics.cross_chain_arbitrages += 1;
            }
        }
        
        Ok(())
    }
    
    /// Evaluar potencial cross-chain
    async fn evaluate_cross_chain_potential(&self, opportunity: &ArbitrageOpportunity) -> CoreResult<CrossChainPotential> {
        let mut best_potential = CrossChainPotential::default();
        
        // Evaluar cada red soportada
        for chain_name in &self.config.supported_chains {
            if chain_name == "Solana" {
                continue; // Skip current chain
            }
            
            // Simular arbitraje cross-chain
            let potential = self.simulate_cross_chain_arbitrage(opportunity, chain_name).await?;
            
            if potential.profit_increase > best_potential.profit_increase {
                best_potential = potential;
            }
        }
        
        Ok(best_potential)
    }
    
    /// Simular arbitraje cross-chain
    async fn simulate_cross_chain_arbitrage(&self, opportunity: &ArbitrageOpportunity, target_chain: &str) -> CoreResult<CrossChainPotential> {
        // Obtener información de la red objetivo
        let target_network = self.ecosystem_manager.active_networks.get(target_chain);
        
        if let Some(network) = target_network {
            // Calcular costos de bridge
            let bridge_cost = self.calculate_bridge_cost(opportunity, target_chain).await?;
            
            // Estimar profit en la red objetivo (simplificado)
            let target_profit_multiplier = match target_chain {
                "Ethereum" => 1.5,  // Higher profits but higher fees
                "BSC" => 1.2,       // Moderate profits, lower fees
                "Polygon" => 1.1,   // Similar to Solana
                "Avalanche" => 1.3, // Good profits, moderate fees
                _ => 1.0,
            };
            
            let potential_profit = opportunity.expected_profit_sol * target_profit_multiplier;
            let net_profit_increase = potential_profit - opportunity.expected_profit_sol - bridge_cost;
            
            if net_profit_increase > 0.0 {
                Ok(CrossChainPotential {
                    target_chain: target_chain.to_string(),
                    profit_increase: net_profit_increase,
                    confidence_factor: 0.8, // Lower confidence for cross-chain
                    additional_time_ms: 30000, // 30 seconds for bridge
                    bridge_cost,
                })
            } else {
                Ok(CrossChainPotential::default())
            }
        } else {
            Ok(CrossChainPotential::default())
        }
    }
    
    /// Calcular costo de bridge
    async fn calculate_bridge_cost(&self, opportunity: &ArbitrageOpportunity, target_chain: &str) -> CoreResult<f64> {
        // Costo base del bridge
        let base_cost = match target_chain {
            "Ethereum" => 0.02,   // High due to gas fees
            "BSC" => 0.005,       // Moderate
            "Polygon" => 0.002,   // Low
            "Avalanche" => 0.008, // Moderate-high
            _ => 0.01,
        };
        
        // Costo proporcional al monto
        let proportional_cost = opportunity.expected_profit_sol * 0.003; // 0.3%
        
        Ok(base_cost + proportional_cost)
    }
    
    /// Aplicar multiplicador de ecosistema
    async fn apply_ecosystem_multiplier(&mut self, opportunities: &mut Vec<ArbitrageOpportunity>) -> CoreResult<()> {
        debug!("💫 Aplicando multiplicador de ecosistema");
        
        let multiplier = self.calculate_ecosystem_multiplier().await?;
        
        for opportunity in opportunities.iter_mut() {
            // Aplicar multiplicador basado en la diversidad del ecosistema
            let profit_bonus = opportunity.expected_profit_sol * (multiplier - 1.0);
            opportunity.expected_profit_sol += profit_bonus;
            
            // Bonus de confianza por ecosistema robusto
            let confidence_bonus = (multiplier - 1.0) * 0.1;
            opportunity.confidence_score += confidence_bonus;
            opportunity.confidence_score = opportunity.confidence_score.min(1.0);
            
            self.statistics.ecosystem_profit_bonus += profit_bonus;
        }
        
        Ok(())
    }
    
    /// Calcular multiplicador de ecosistema
    async fn calculate_ecosystem_multiplier(&self) -> CoreResult<f64> {
        let base_multiplier = self.config.ecosystem_multiplier;
        
        // Factor basado en número de redes integradas
        let network_factor = 1.0 + (self.ecosystem_manager.active_networks.len() as f64 * 0.05);
        
        // Factor basado en salud del ecosistema
        let health_factor = 1.0 + (self.ecosystem_manager.ecosystem_health * 0.1);
        
        // Factor basado en efecto de red
        let network_effect = 1.0 + self.config.network_effect_bonus;
        
        Ok(base_multiplier * network_factor * health_factor * network_effect)
    }
    
    /// Calcular efectos de red
    async fn calculate_network_effects(&mut self, opportunities: &[ArbitrageOpportunity]) -> CoreResult<()> {
        debug!("🔗 Calculando efectos de red");
        
        // Calcular valor total de oportunidades
        let total_value: f64 = opportunities.iter()
            .map(|o| o.expected_profit_sol)
            .sum();
        
        // Efecto de red: más oportunidades = mayor eficiencia
        let network_effect = (opportunities.len() as f64).sqrt() * 0.01;
        
        // Efecto de sinergia entre diferentes tipos de oportunidades
        let synergy_effect = self.calculate_synergy_effect(opportunities).await?;
        
        let total_network_effect = network_effect + synergy_effect;
        
        self.statistics.network_effect_contributions += total_network_effect * total_value;
        
        Ok(())
    }
    
    /// Calcular efecto de sinergia
    async fn calculate_synergy_effect(&self, opportunities: &[ArbitrageOpportunity]) -> CoreResult<f64> {
        let mut unique_dexes = HashSet::new();
        let mut unique_tokens = HashSet::new();
        
        for opportunity in opportunities {
            unique_dexes.insert(&opportunity.dex_from.name);
            unique_dexes.insert(&opportunity.dex_to.name);
            unique_tokens.insert(&opportunity.token_in.symbol);
            unique_tokens.insert(&opportunity.token_out.symbol);
        }
        
        // Sinergia basada en diversidad
        let dex_diversity = unique_dexes.len() as f64 / 10.0; // Normalize
        let token_diversity = unique_tokens.len() as f64 / 50.0; // Normalize
        
        Ok((dex_diversity + token_diversity) * 0.02) // 2% max synergy
    }
    
    /// Identificar oportunidades de expansión
    async fn identify_expansion_opportunities(&mut self) -> CoreResult<()> {
        debug!("🎯 Identificando oportunidades de expansión");
        
        self.ecosystem_manager.expansion_opportunities.clear();
        
        // Analizar redes no integradas
        for chain_name in &self.config.supported_chains {
            if !self.ecosystem_manager.active_networks.contains_key(chain_name) {
                let opportunity = self.evaluate_expansion_to_network(chain_name).await?;
                if opportunity.potential_profit > opportunity.integration_cost {
                    self.ecosystem_manager.expansion_opportunities.push(opportunity);
                    self.statistics.expansion_opportunities_identified += 1;
                }
            }
        }
        
        // Priorizar oportunidades por ROI
        self.ecosystem_manager.expansion_opportunities.sort_by(|a, b| {
            let roi_a = a.potential_profit / a.integration_cost;
            let roi_b = b.potential_profit / b.integration_cost;
            roi_b.partial_cmp(&roi_a).unwrap()
        });
        
        info!("🎯 Identificadas {} oportunidades de expansión", 
              self.ecosystem_manager.expansion_opportunities.len());
        
        Ok(())
    }
    
    /// Evaluar expansión a una red
    async fn evaluate_expansion_to_network(&self, chain_name: &str) -> CoreResult<ExpansionOpportunity> {
        // Estimar potencial de profit (simplificado)
        let potential_profit = match chain_name {
            "Ethereum" => 0.5,   // High volume, high potential
            "BSC" => 0.3,        // Good volume, moderate potential
            "Polygon" => 0.2,    // Growing ecosystem
            "Avalanche" => 0.25, // Emerging opportunities
            _ => 0.1,
        };
        
        // Estimar costo de integración
        let integration_cost = match chain_name {
            "Ethereum" => 0.1,   // Complex integration
            "BSC" => 0.05,       // Moderate complexity
            "Polygon" => 0.03,   // Easier integration
            "Avalanche" => 0.04, // Moderate complexity
            _ => 0.02,
        };
        
        Ok(ExpansionOpportunity {
            target_network: chain_name.to_string(),
            opportunity_type: ExpansionType::CrossChainArbitrage,
            potential_profit,
            integration_cost,
            time_to_implement: match chain_name {
                "Ethereum" => 30,
                "BSC" => 14,
                "Polygon" => 7,
                "Avalanche" => 10,
                _ => 21,
            },
            risk_score: match chain_name {
                "Ethereum" => 0.3, // Lower risk (established)
                "BSC" => 0.4,      // Moderate risk
                "Polygon" => 0.5,  // Higher risk (newer)
                "Avalanche" => 0.4, // Moderate risk
                _ => 0.6,
            },
        })
    }
    
    /// Optimizar para múltiples redes
    async fn optimize_for_multi_network(&mut self, opportunities: &mut Vec<ArbitrageOpportunity>) -> CoreResult<()> {
        debug!("⚡ Optimizando para múltiples redes");
        
        // Agrupar oportunidades por tipo de optimización
        let mut network_groups: HashMap<String, Vec<&mut ArbitrageOpportunity>> = HashMap::new();
        
        for opportunity in opportunities.iter_mut() {
            let group_key = format!("{}_{}", 
                                   opportunity.token_in.symbol, 
                                   opportunity.token_out.symbol);
            network_groups.entry(group_key).or_insert_with(Vec::new).push(opportunity);
        }
        
        // Aplicar optimización a cada grupo
        for (group_name, group_opportunities) in network_groups {
            if group_opportunities.len() > 1 {
                self.optimize_opportunity_group(&group_name, group_opportunities).await?;
            }
        }
        
        Ok(())
    }
    
    /// Optimizar grupo de oportunidades
    async fn optimize_opportunity_group(&self, group_name: &str, opportunities: Vec<&mut ArbitrageOpportunity>) -> CoreResult<()> {
        debug!("🔧 Optimizando grupo: {}", group_name);
        
        // Calcular bonus de coordinación entre oportunidades similares
        let coordination_bonus = 0.01 * (opportunities.len() as f64 - 1.0); // 1% per additional opportunity
        
        for opportunity in opportunities {
            opportunity.expected_profit_sol += opportunity.expected_profit_sol * coordination_bonus;
            
            // Reducir riesgo por diversificación
            opportunity.risk_score *= 0.95;
        }
        
        Ok(())
    }
}

/// Potencial cross-chain
#[derive(Debug, Clone)]
struct CrossChainPotential {
    target_chain: String,
    profit_increase: f64,
    confidence_factor: f64,
    additional_time_ms: u64,
    bridge_cost: f64,
}

impl Default for CrossChainPotential {
    fn default() -> Self {
        Self {
            target_chain: String::new(),
            profit_increase: 0.0,
            confidence_factor: 1.0,
            additional_time_ms: 0,
            bridge_cost: 0.0,
        }
    }
}

impl EcosystemManager {
    fn new(config: &EcosystemConfig) -> CoreResult<Self> {
        let mut active_networks = HashMap::new();
        
        // Initialize supported networks
        for chain_name in &config.supported_chains {
            let network_info = NetworkInfo {
                name: chain_name.clone(),
                chain_id: Self::get_chain_id(chain_name),
                native_token: Self::get_native_token(chain_name),
                total_tvl: Self::get_estimated_tvl(chain_name),
                active_protocols: Self::get_protocol_count(chain_name),
                avg_fees: Self::get_avg_fees(chain_name),
                network_status: NetworkStatus::Active,
                integration_level: if chain_name == "Solana" {
                    IntegrationLevel::Native
                } else {
                    IntegrationLevel::Monitor
                },
            };
            
            active_networks.insert(chain_name.clone(), network_info);
        }
        
        Ok(Self {
            active_networks,
            ecosystem_connections: Vec::new(),
            expansion_opportunities: Vec::new(),
            ecosystem_health: 0.8, // Initial health score
        })
    }
    
    fn get_chain_id(chain_name: &str) -> String {
        match chain_name {
            "Solana" => "solana-mainnet".to_string(),
            "Ethereum" => "ethereum-mainnet".to_string(),
            "BSC" => "bsc-mainnet".to_string(),
            "Polygon" => "polygon-mainnet".to_string(),
            "Avalanche" => "avalanche-mainnet".to_string(),
            _ => format!("{}-mainnet", chain_name.to_lowercase()),
        }
    }
    
    fn get_native_token(chain_name: &str) -> String {
        match chain_name {
            "Solana" => "SOL".to_string(),
            "Ethereum" => "ETH".to_string(),
            "BSC" => "BNB".to_string(),
            "Polygon" => "MATIC".to_string(),
            "Avalanche" => "AVAX".to_string(),
            _ => "UNKNOWN".to_string(),
        }
    }
    
    fn get_estimated_tvl(chain_name: &str) -> f64 {
        match chain_name {
            "Ethereum" => 50_000_000_000.0, // $50B
            "BSC" => 8_000_000_000.0,       // $8B
            "Solana" => 5_000_000_000.0,    // $5B
            "Polygon" => 3_000_000_000.0,   // $3B
            "Avalanche" => 2_000_000_000.0, // $2B
            _ => 500_000_000.0,             // $500M
        }
    }
    
    fn get_protocol_count(chain_name: &str) -> u32 {
        match chain_name {
            "Ethereum" => 3000,
            "BSC" => 1500,
            "Solana" => 1000,
            "Polygon" => 800,
            "Avalanche" => 600,
            _ => 100,
        }
    }
    
    fn get_avg_fees(chain_name: &str) -> f64 {
        match chain_name {
            "Ethereum" => 0.02,   // High fees
            "BSC" => 0.001,       // Low fees
            "Solana" => 0.00025,  // Very low fees
            "Polygon" => 0.0005,  // Very low fees
            "Avalanche" => 0.005, // Moderate fees
            _ => 0.01,
        }
    }
}

impl CrossChainBridge {
    fn new() -> Self {
        let mut supported_chains = HashSet::new();
        supported_chains.insert("Solana".to_string());
        supported_chains.insert("Ethereum".to_string());
        supported_chains.insert("BSC".to_string());
        supported_chains.insert("Polygon".to_string());
        supported_chains.insert("Avalanche".to_string());
        
        Self {
            supported_chains,
            bridge_contracts: HashMap::new(),
            active_bridges: Vec::new(),
            bridge_statistics: BridgeStatistics::default(),
        }
    }
}

impl NetworkAnalyzer {
    fn new() -> Self {
        Self {
            network_metrics: HashMap::new(),
            trend_analysis: TrendAnalysis {
                growth_rate: 0.15,
                volatility: 0.3,
                adoption_trend: AdoptionTrend::Steady,
                innovation_score: 0.8,
            },
            performance_benchmarks: PerformanceBenchmarks {
                best_performing_network: "Solana".to_string(),
                fastest_network: "Solana".to_string(),
                most_cost_effective: "Polygon".to_string(),
                highest_tvl: "Ethereum".to_string(),
            },
        }
    }
}

impl EcosystemMultiplier {
    fn new(config: &EcosystemConfig) -> Self {
        Self {
            network_effect_bonus: config.network_effect_bonus,
            cross_chain_synergy: 1.0,
            ecosystem_diversity_score: 0.7,
            total_ecosystem_value: 0.0,
        }
    }
}

#[async_trait]
impl PhaseManager for EcosystemPhase {
    async fn process_opportunities(&mut self, mut opportunities: Vec<ArbitrageOpportunity>) -> CoreResult<Vec<ArbitrageOpportunity>> {
        if !self.config.enabled {
            return Ok(opportunities);
        }
        
        let start_time = Instant::now();
        info!("🌐 Iniciando Phase 11: Ecosystem Expansion");
        
        self.process_with_ecosystem_expansion(&mut opportunities).await?;
        
        let processing_time = start_time.elapsed();
        self.statistics.networks_integrated = self.ecosystem_manager.active_networks.len() as u32;
        
        info!("✅ Phase 11 completada en {:?}. Expandido ecosistema con {} redes", 
              processing_time, self.statistics.networks_integrated);
        
        Ok(opportunities)
    }
    
    fn get_phase_stats(&self) -> crate::main::PhaseStats {
        crate::main::PhaseStats {
            opportunities_processed: self.statistics.cross_chain_arbitrages,
            opportunities_enhanced: self.statistics.expansion_opportunities_identified as u64,
            total_profit_added: self.statistics.ecosystem_profit_bonus,
            success_rate: 0.85, // Ecosystem expansion success rate
        }
    }
    
    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

#[async_trait]
impl Phase for EcosystemPhase {
    fn name(&self) -> &str {
        "Ecosystem Expansion"
    }
    
    fn phase_number(&self) -> u8 {
        11
    }
    
    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    
    async fn process(&mut self, opportunities: Vec<ArbitrageOpportunity>) -> CoreResult<Vec<ArbitrageOpportunity>> {
        self.process_opportunities(opportunities).await
    }
    
    fn get_stats(&self) -> PhaseResult {
        PhaseResult {
            phase_name: self.name().to_string(),
            opportunities_processed: self.statistics.cross_chain_arbitrages as usize,
            opportunities_enhanced: self.statistics.expansion_opportunities_identified as usize,
            total_profit_added: self.statistics.ecosystem_profit_bonus,
            processing_time_ms: 0,
            success: true,
            error_message: None,
        }
    }
    
    fn reset_stats(&mut self) {
        self.statistics = EcosystemStatistics::default();
    }
}
