// ================================================================================
// PHASES MODULE - Sistemas de Fases Avanzadas
// ================================================================================

//! # Phases Module
//! 
//! Implementa las fases avanzadas del sistema de arbitraje:
//! - Phase 9: Quantum Optimization
//! - Phase 10: Autonomous AI
//! - Phase 11: Ecosystem Expansion

pub mod quantum;
pub mod autonomous;
pub mod ecosystem;

// Re-export phase implementations
pub use quantum::QuantumPhase;
pub use autonomous::AutonomousPhase;
pub use ecosystem::EcosystemPhase;

// Re-export phase manager trait
pub use crate::main::PhaseManager;

use sniperforge_core::CoreResult;
use crate::engine::ArbitrageOpportunity;

/// Resultado de procesamiento de fase
#[derive(Debug, Clone)]
pub struct PhaseResult {
    pub phase_name: String,
    pub opportunities_processed: usize,
    pub opportunities_enhanced: usize,
    pub total_profit_added: f64,
    pub processing_time_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Trait común para todas las fases
#[async_trait::async_trait]
pub trait Phase: Send + Sync {
    /// Nombre de la fase
    fn name(&self) -> &str;
    
    /// Número de la fase
    fn phase_number(&self) -> u8;
    
    /// Verificar si la fase está habilitada
    fn is_enabled(&self) -> bool;
    
    /// Procesar oportunidades con esta fase
    async fn process(&mut self, opportunities: Vec<ArbitrageOpportunity>) -> CoreResult<Vec<ArbitrageOpportunity>>;
    
    /// Obtener estadísticas de la fase
    fn get_stats(&self) -> PhaseResult;
    
    /// Reiniciar estadísticas de la fase
    fn reset_stats(&mut self);
}
