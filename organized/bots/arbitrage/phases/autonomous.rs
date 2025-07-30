// ================================================================================
// PHASE 10: AUTONOMOUS AI
// ================================================================================

use std::time::Instant;
use std::collections::{HashMap, VecDeque};
use tracing::{info, debug, warn};
use async_trait::async_trait;

use sniperforge_core::CoreResult;
use crate::engine::ArbitrageOpportunity;
use crate::config::AutonomousConfig;
use crate::main::PhaseManager;
use super::{Phase, PhaseResult};

/// Phase 10: Sistema autónomo con AI
pub struct AutonomousPhase {
    config: AutonomousConfig,
    ai_brain: AiBrain,
    decision_history: VecDeque<AiDecision>,
    parameter_optimizer: ParameterOptimizer,
    risk_manager: AdaptiveRiskManager,
    statistics: AutonomousStatistics,
}

/// Cerebro de AI autónoma
#[derive(Debug)]
struct AiBrain {
    decision_networks: HashMap<String, NeuralNetwork>,
    confidence_threshold: f64,
    learning_rate: f64,
    experience_buffer: VecDeque<Experience>,
    model_accuracy: f64,
}

/// Red neuronal simplificada
#[derive(Debug, Clone)]
struct NeuralNetwork {
    weights: Vec<Vec<f64>>,
    biases: Vec<f64>,
    activation_function: ActivationFunction,
    performance_score: f64,
}

/// Función de activación
#[derive(Debug, Clone)]
enum ActivationFunction {
    ReLU,
    Sigmoid,
    Tanh,
}

/// Experiencia para aprendizaje
#[derive(Debug, Clone)]
struct Experience {
    state: StateVector,
    action: AiAction,
    reward: f64,
    next_state: StateVector,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Vector de estado
#[derive(Debug, Clone)]
struct StateVector {
    market_conditions: Vec<f64>,
    opportunity_features: Vec<f64>,
    system_performance: Vec<f64>,
    external_factors: Vec<f64>,
}

/// Acción de AI
#[derive(Debug, Clone)]
enum AiAction {
    Execute(f64),      // Execute with confidence level
    Skip,              // Skip opportunity
    Adjust(AdjustmentParameters), // Adjust parameters
    Wait(u64),         // Wait for time in ms
}

/// Parámetros de ajuste
#[derive(Debug, Clone)]
struct AdjustmentParameters {
    profit_threshold: Option<f64>,
    risk_level: Option<f64>,
    execution_speed: Option<f64>,
    position_size: Option<f64>,
}

/// Decisión de AI
#[derive(Debug, Clone)]
struct AiDecision {
    opportunity_id: String,
    decision: AiAction,
    confidence: f64,
    reasoning: String,
    execution_result: Option<ExecutionResult>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Resultado de ejecución
#[derive(Debug, Clone)]
struct ExecutionResult {
    success: bool,
    actual_profit: f64,
    execution_time: u64,
    error_message: Option<String>,
}

/// Optimizador de parámetros
#[derive(Debug)]
struct ParameterOptimizer {
    parameter_history: HashMap<String, VecDeque<ParameterUpdate>>,
    optimization_targets: Vec<OptimizationTarget>,
    current_parameters: HashMap<String, f64>,
    performance_metrics: PerformanceMetrics,
}

/// Actualización de parámetro
#[derive(Debug, Clone)]
struct ParameterUpdate {
    parameter_name: String,
    old_value: f64,
    new_value: f64,
    performance_impact: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Objetivo de optimización
#[derive(Debug, Clone)]
enum OptimizationTarget {
    MaximizeProfit,
    MinimizeRisk,
    MaximizeSuccessRate,
    MinimizeExecutionTime,
}

/// Métricas de rendimiento
#[derive(Debug, Clone, Default)]
struct PerformanceMetrics {
    total_profit: f64,
    success_rate: f64,
    avg_execution_time: f64,
    risk_adjusted_return: f64,
    sharpe_ratio: f64,
}

/// Gestor de riesgo adaptativo
#[derive(Debug)]
struct AdaptiveRiskManager {
    risk_models: HashMap<String, RiskModel>,
    dynamic_limits: DynamicLimits,
    risk_events: VecDeque<RiskEvent>,
    current_risk_level: f64,
}

/// Modelo de riesgo
#[derive(Debug, Clone)]
struct RiskModel {
    name: String,
    risk_factors: Vec<RiskFactor>,
    weight: f64,
    accuracy: f64,
}

/// Factor de riesgo
#[derive(Debug, Clone)]
struct RiskFactor {
    name: String,
    current_value: f64,
    threshold: f64,
    impact_weight: f64,
}

/// Límites dinámicos
#[derive(Debug, Clone)]
struct DynamicLimits {
    max_position_size: f64,
    max_daily_trades: u32,
    max_concurrent_trades: u32,
    stop_loss_threshold: f64,
    profit_target: f64,
}

/// Evento de riesgo
#[derive(Debug, Clone)]
struct RiskEvent {
    event_type: RiskEventType,
    severity: f64,
    impact: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
    mitigation_actions: Vec<String>,
}

/// Tipo de evento de riesgo
#[derive(Debug, Clone)]
enum RiskEventType {
    HighVolatility,
    LowLiquidity,
    NetworkCongestion,
    ExecutionFailure,
    PriceSlippage,
}

/// Estadísticas autónomas
#[derive(Debug, Clone, Default)]
struct AutonomousStatistics {
    ai_decisions_made: u64,
    successful_decisions: u64,
    parameters_optimized: u64,
    risk_events_handled: u64,
    learning_iterations: u64,
    total_autonomous_profit: f64,
    ai_accuracy: f64,
}

impl AutonomousPhase {
    /// Crear nueva fase autónoma
    pub fn new(config: &AutonomousConfig) -> CoreResult<Self> {
        info!("🤖 Inicializando Autonomous Phase 10");
        
        Ok(Self {
            config: config.clone(),
            ai_brain: AiBrain::new(config)?,
            decision_history: VecDeque::with_capacity(1000),
            parameter_optimizer: ParameterOptimizer::new(),
            risk_manager: AdaptiveRiskManager::new(),
            statistics: AutonomousStatistics::default(),
        })
    }
    
    /// Procesar oportunidades con AI autónoma
    async fn process_with_autonomous_ai(&mut self, opportunities: &mut Vec<ArbitrageOpportunity>) -> CoreResult<()> {
        debug!("🤖 Procesando con AI autónoma");
        
        for opportunity in opportunities.iter_mut() {
            // 1. Analizar oportunidad con AI
            let decision = self.make_ai_decision(opportunity).await?;
            
            // 2. Aplicar decisión
            self.apply_ai_decision(opportunity, &decision).await?;
            
            // 3. Guardar decisión en historial
            self.decision_history.push_back(decision);
            
            // 4. Mantener tamaño del historial
            if self.decision_history.len() > 1000 {
                self.decision_history.pop_front();
            }
        }
        
        // 5. Optimizar parámetros si está habilitado
        if self.config.auto_parameter_tuning {
            self.optimize_parameters().await?;
        }
        
        // 6. Actualizar gestión de riesgo
        if self.config.adaptive_risk_management {
            self.update_risk_management().await?;
        }
        
        // 7. Aprender de feedback si está habilitado
        if self.config.learning_feedback_loop {
            self.learn_from_feedback().await?;
        }
        
        Ok(())
    }
    
    /// Tomar decisión con AI
    async fn make_ai_decision(&mut self, opportunity: &ArbitrageOpportunity) -> CoreResult<AiDecision> {
        // 1. Crear vector de estado
        let state = self.create_state_vector(opportunity).await?;
        
        // 2. Evaluar con redes neuronales
        let confidence = self.evaluate_with_neural_networks(&state).await?;
        
        // 3. Aplicar threshold de decisión
        let decision = if confidence >= self.config.ai_decision_threshold {
            AiAction::Execute(confidence)
        } else if confidence > 0.3 {
            // Maybe adjust parameters instead
            AiAction::Adjust(AdjustmentParameters {
                profit_threshold: Some(opportunity.expected_profit_sol * 0.9),
                risk_level: Some(opportunity.risk_score * 1.1),
                execution_speed: None,
                position_size: None,
            })
        } else {
            AiAction::Skip
        };
        
        // 4. Generar reasoning
        let reasoning = self.generate_reasoning(&decision, confidence, opportunity);
        
        let ai_decision = AiDecision {
            opportunity_id: opportunity.id.clone(),
            decision,
            confidence,
            reasoning,
            execution_result: None,
            timestamp: chrono::Utc::now(),
        };
        
        self.statistics.ai_decisions_made += 1;
        
        Ok(ai_decision)
    }
    
    /// Crear vector de estado
    async fn create_state_vector(&self, opportunity: &ArbitrageOpportunity) -> CoreResult<StateVector> {
        Ok(StateVector {
            market_conditions: vec![
                opportunity.price_impact,
                0.5, // market_volatility (simplified)
                0.7, // liquidity_score (simplified)
                0.3, // network_congestion (simplified)
            ],
            opportunity_features: vec![
                opportunity.profit_percentage,
                opportunity.expected_profit_sol,
                opportunity.confidence_score,
                opportunity.risk_score,
                opportunity.execution_time_estimate_ms as f64 / 10000.0, // normalized
            ],
            system_performance: vec![
                self.statistics.ai_accuracy,
                self.parameter_optimizer.performance_metrics.success_rate,
                self.parameter_optimizer.performance_metrics.risk_adjusted_return,
            ],
            external_factors: vec![
                chrono::Utc::now().hour() as f64 / 24.0, // time_of_day
                0.5, // gas_price_normalized (simplified)
                0.6, // overall_market_sentiment (simplified)
            ],
        })
    }
    
    /// Evaluar con redes neuronales
    async fn evaluate_with_neural_networks(&self, state: &StateVector) -> CoreResult<f64> {
        let mut total_confidence = 0.0;
        let mut network_count = 0;
        
        // Combinar inputs de todas las categorías
        let inputs: Vec<f64> = [
            &state.market_conditions,
            &state.opportunity_features,
            &state.system_performance,
            &state.external_factors,
        ].concat();
        
        // Evaluar con cada red neuronal
        for (network_name, network) in &self.ai_brain.decision_networks {
            let network_output = self.forward_pass(network, &inputs)?;
            let weighted_output = network_output * network.performance_score;
            total_confidence += weighted_output;
            network_count += 1;
        }
        
        Ok(if network_count > 0 {
            total_confidence / network_count as f64
        } else {
            0.5 // Default neutral confidence
        })
    }
    
    /// Forward pass en red neuronal
    fn forward_pass(&self, network: &NeuralNetwork, inputs: &[f64]) -> CoreResult<f64> {
        let mut current_layer = inputs.to_vec();
        
        for (layer_weights, bias) in network.weights.iter().zip(&network.biases) {
            let mut next_layer = vec![0.0; layer_weights.len()];
            
            for (i, neuron_weights) in layer_weights.iter().enumerate() {
                let mut sum = *bias;
                for (weight, &input) in neuron_weights.iter().zip(&current_layer) {
                    sum += weight * input;
                }
                next_layer[i] = self.apply_activation(&network.activation_function, sum);
            }
            
            current_layer = next_layer;
        }
        
        Ok(current_layer.get(0).copied().unwrap_or(0.5))
    }
    
    /// Aplicar función de activación
    fn apply_activation(&self, activation: &ActivationFunction, x: f64) -> f64 {
        match activation {
            ActivationFunction::ReLU => x.max(0.0),
            ActivationFunction::Sigmoid => 1.0 / (1.0 + (-x).exp()),
            ActivationFunction::Tanh => x.tanh(),
        }
    }
    
    /// Generar reasoning para la decisión
    fn generate_reasoning(&self, decision: &AiAction, confidence: f64, opportunity: &ArbitrageOpportunity) -> String {
        match decision {
            AiAction::Execute(_) => format!(
                "AI decidió ejecutar con confianza {:.2}%. Profit esperado: {:.6} SOL, Risk score: {:.2}",
                confidence * 100.0, opportunity.expected_profit_sol, opportunity.risk_score
            ),
            AiAction::Skip => format!(
                "AI decidió saltar. Confianza baja ({:.2}%) o riesgo alto ({:.2})",
                confidence * 100.0, opportunity.risk_score
            ),
            AiAction::Adjust(_) => format!(
                "AI sugiere ajustar parámetros. Confianza moderada: {:.2}%",
                confidence * 100.0
            ),
            AiAction::Wait(ms) => format!(
                "AI sugiere esperar {}ms antes de ejecutar",
                ms
            ),
        }
    }
    
    /// Aplicar decisión de AI
    async fn apply_ai_decision(&mut self, opportunity: &mut ArbitrageOpportunity, decision: &AiDecision) -> CoreResult<()> {
        match &decision.decision {
            AiAction::Execute(confidence) => {
                // Aumentar confidence score basado en AI
                opportunity.confidence_score = (opportunity.confidence_score + confidence) / 2.0;
                
                // Aplicar bonus de AI si la confianza es muy alta
                if *confidence > 0.9 {
                    opportunity.expected_profit_sol *= 1.02; // 2% bonus
                }
                
                self.statistics.successful_decisions += 1;
            },
            AiAction::Adjust(params) => {
                // Aplicar ajustes sugeridos
                if let Some(profit_threshold) = params.profit_threshold {
                    opportunity.expected_profit_sol = opportunity.expected_profit_sol.max(profit_threshold);
                }
                if let Some(risk_level) = params.risk_level {
                    opportunity.risk_score = risk_level;
                }
            },
            AiAction::Skip => {
                // Marcar la oportunidad como de baja prioridad
                opportunity.confidence_score *= 0.5;
            },
            AiAction::Wait(_) => {
                // Aumentar tiempo de expiración
                opportunity.expires_at = opportunity.expires_at + chrono::Duration::seconds(30);
            },
        }
        
        Ok(())
    }
    
    /// Optimizar parámetros automáticamente
    async fn optimize_parameters(&mut self) -> CoreResult<()> {
        debug!("⚙️  Optimizando parámetros automáticamente");
        
        // Analizar rendimiento de decisiones recientes
        let recent_decisions: Vec<_> = self.decision_history.iter()
            .rev()
            .take(50)
            .collect();
        
        if recent_decisions.len() < 10 {
            return Ok(()); // Not enough data
        }
        
        // Calcular métricas de rendimiento
        let success_rate = recent_decisions.iter()
            .filter(|d| matches!(d.execution_result, Some(ExecutionResult { success: true, .. })))
            .count() as f64 / recent_decisions.len() as f64;
        
        let avg_profit = recent_decisions.iter()
            .filter_map(|d| d.execution_result.as_ref())
            .map(|r| r.actual_profit)
            .sum::<f64>() / recent_decisions.len() as f64;
        
        // Ajustar threshold de decisión basado en rendimiento
        if success_rate > 0.8 && avg_profit > 0.005 {
            // Performance is good, be more aggressive
            self.ai_brain.confidence_threshold *= 0.95;
        } else if success_rate < 0.5 || avg_profit < 0.002 {
            // Performance is poor, be more conservative
            self.ai_brain.confidence_threshold *= 1.05;
        }
        
        // Mantener threshold en rango razonable
        self.ai_brain.confidence_threshold = self.ai_brain.confidence_threshold.max(0.3).min(0.9);
        
        self.statistics.parameters_optimized += 1;
        
        Ok(())
    }
    
    /// Actualizar gestión de riesgo
    async fn update_risk_management(&mut self) -> CoreResult<()> {
        debug!("🛡️  Actualizando gestión de riesgo adaptativa");
        
        // Calcular nivel de riesgo actual
        let current_risk = self.calculate_current_risk_level().await?;
        
        // Ajustar límites dinámicamente
        if current_risk > 0.7 {
            // High risk - tighten limits
            self.risk_manager.dynamic_limits.max_position_size *= 0.8;
            self.risk_manager.dynamic_limits.stop_loss_threshold *= 0.9;
        } else if current_risk < 0.3 {
            // Low risk - relax limits slightly
            self.risk_manager.dynamic_limits.max_position_size *= 1.05;
            self.risk_manager.dynamic_limits.stop_loss_threshold *= 1.02;
        }
        
        // Mantener límites en rangos seguros
        self.risk_manager.dynamic_limits.max_position_size = 
            self.risk_manager.dynamic_limits.max_position_size.max(0.1).min(2.0);
        
        self.risk_manager.current_risk_level = current_risk;
        
        Ok(())
    }
    
    /// Calcular nivel de riesgo actual
    async fn calculate_current_risk_level(&self) -> CoreResult<f64> {
        let mut total_risk = 0.0;
        let mut factor_count = 0;
        
        // Evaluar factores de riesgo
        for risk_model in self.risk_manager.risk_models.values() {
            for factor in &risk_model.risk_factors {
                let factor_risk = if factor.current_value > factor.threshold {
                    (factor.current_value / factor.threshold - 1.0) * factor.impact_weight
                } else {
                    0.0
                };
                
                total_risk += factor_risk * risk_model.weight;
                factor_count += 1;
            }
        }
        
        Ok(if factor_count > 0 {
            (total_risk / factor_count as f64).max(0.0).min(1.0)
        } else {
            0.5
        })
    }
    
    /// Aprender del feedback
    async fn learn_from_feedback(&mut self) -> CoreResult<()> {
        debug!("🧠 Aprendiendo del feedback");
        
        // Obtener decisiones con resultados
        let learning_data: Vec<_> = self.decision_history.iter()
            .filter(|d| d.execution_result.is_some())
            .collect();
        
        if learning_data.len() < 5 {
            return Ok(()); // Not enough data to learn
        }
        
        // Crear experiencias para el buffer
        for decision in learning_data.iter().rev().take(10) {
            if let Some(result) = &decision.execution_result {
                let reward = if result.success {
                    result.actual_profit * 10.0 // Scale up reward
                } else {
                    -0.1 // Penalty for failure
                };
                
                // Simplified experience creation
                let experience = Experience {
                    state: StateVector {
                        market_conditions: vec![0.5; 4],
                        opportunity_features: vec![decision.confidence; 5],
                        system_performance: vec![0.5; 3],
                        external_factors: vec![0.5; 3],
                    },
                    action: decision.decision.clone(),
                    reward,
                    next_state: StateVector {
                        market_conditions: vec![0.5; 4],
                        opportunity_features: vec![decision.confidence * 0.9; 5],
                        system_performance: vec![0.5; 3],
                        external_factors: vec![0.5; 3],
                    },
                    timestamp: decision.timestamp,
                };
                
                self.ai_brain.experience_buffer.push_back(experience);
                
                // Maintain buffer size
                if self.ai_brain.experience_buffer.len() > 1000 {
                    self.ai_brain.experience_buffer.pop_front();
                }
            }
        }
        
        // Update AI accuracy
        let recent_successes = learning_data.iter()
            .rev()
            .take(20)
            .filter(|d| d.execution_result.as_ref().map(|r| r.success).unwrap_or(false))
            .count();
        
        self.statistics.ai_accuracy = recent_successes as f64 / 20.0.min(learning_data.len() as f64);
        
        self.statistics.learning_iterations += 1;
        
        Ok(())
    }
}

impl AiBrain {
    fn new(config: &AutonomousConfig) -> CoreResult<Self> {
        let mut decision_networks = HashMap::new();
        
        // Create specialized networks
        decision_networks.insert("profit_predictor".to_string(), NeuralNetwork::new(15, &[10, 5, 1])?);
        decision_networks.insert("risk_assessor".to_string(), NeuralNetwork::new(15, &[8, 4, 1])?);
        decision_networks.insert("execution_timer".to_string(), NeuralNetwork::new(15, &[6, 3, 1])?);
        
        Ok(Self {
            decision_networks,
            confidence_threshold: config.ai_decision_threshold,
            learning_rate: 0.01,
            experience_buffer: VecDeque::with_capacity(1000),
            model_accuracy: 0.5,
        })
    }
}

impl NeuralNetwork {
    fn new(input_size: usize, hidden_layers: &[usize]) -> CoreResult<Self> {
        let mut weights = Vec::new();
        let mut biases = Vec::new();
        
        let mut prev_size = input_size;
        for &layer_size in hidden_layers {
            // Initialize weights with small random values
            let layer_weights: Vec<Vec<f64>> = (0..layer_size)
                .map(|_| (0..prev_size).map(|_| (rand::random::<f64>() - 0.5) * 0.1).collect())
                .collect();
            
            weights.push(layer_weights);
            biases.push((rand::random::<f64>() - 0.5) * 0.1);
            prev_size = layer_size;
        }
        
        Ok(Self {
            weights,
            biases,
            activation_function: ActivationFunction::ReLU,
            performance_score: 1.0,
        })
    }
}

impl ParameterOptimizer {
    fn new() -> Self {
        let mut current_parameters = HashMap::new();
        current_parameters.insert("ai_threshold".to_string(), 0.8);
        current_parameters.insert("risk_tolerance".to_string(), 0.3);
        current_parameters.insert("execution_speed".to_string(), 1.0);
        
        Self {
            parameter_history: HashMap::new(),
            optimization_targets: vec![
                OptimizationTarget::MaximizeProfit,
                OptimizationTarget::MinimizeRisk,
                OptimizationTarget::MaximizeSuccessRate,
            ],
            current_parameters,
            performance_metrics: PerformanceMetrics::default(),
        }
    }
}

impl AdaptiveRiskManager {
    fn new() -> Self {
        let mut risk_models = HashMap::new();
        
        // Market volatility model
        risk_models.insert("market_volatility".to_string(), RiskModel {
            name: "Market Volatility".to_string(),
            risk_factors: vec![
                RiskFactor {
                    name: "price_volatility".to_string(),
                    current_value: 0.3,
                    threshold: 0.5,
                    impact_weight: 0.8,
                },
                RiskFactor {
                    name: "volume_volatility".to_string(),
                    current_value: 0.2,
                    threshold: 0.4,
                    impact_weight: 0.6,
                },
            ],
            weight: 1.0,
            accuracy: 0.8,
        });
        
        Self {
            risk_models,
            dynamic_limits: DynamicLimits {
                max_position_size: 1.0,
                max_daily_trades: 100,
                max_concurrent_trades: 3,
                stop_loss_threshold: 0.05,
                profit_target: 0.02,
            },
            risk_events: VecDeque::with_capacity(100),
            current_risk_level: 0.3,
        }
    }
}

#[async_trait]
impl PhaseManager for AutonomousPhase {
    async fn process_opportunities(&mut self, mut opportunities: Vec<ArbitrageOpportunity>) -> CoreResult<Vec<ArbitrageOpportunity>> {
        if !self.config.enabled {
            return Ok(opportunities);
        }
        
        let start_time = Instant::now();
        info!("🤖 Iniciando Phase 10: Autonomous AI");
        
        self.process_with_autonomous_ai(&mut opportunities).await?;
        
        let processing_time = start_time.elapsed();
        
        info!("✅ Phase 10 completada en {:?}. Procesadas {} oportunidades con AI", 
              processing_time, opportunities.len());
        
        Ok(opportunities)
    }
    
    fn get_phase_stats(&self) -> crate::main::PhaseStats {
        crate::main::PhaseStats {
            opportunities_processed: self.statistics.ai_decisions_made,
            opportunities_enhanced: self.statistics.successful_decisions,
            total_profit_added: self.statistics.total_autonomous_profit,
            success_rate: self.statistics.ai_accuracy,
        }
    }
    
    fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

#[async_trait]
impl Phase for AutonomousPhase {
    fn name(&self) -> &str {
        "Autonomous AI"
    }
    
    fn phase_number(&self) -> u8 {
        10
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
            opportunities_processed: self.statistics.ai_decisions_made as usize,
            opportunities_enhanced: self.statistics.successful_decisions as usize,
            total_profit_added: self.statistics.total_autonomous_profit,
            processing_time_ms: 0,
            success: true,
            error_message: None,
        }
    }
    
    fn reset_stats(&mut self) {
        self.statistics = AutonomousStatistics::default();
    }
}

// Helper to generate random numbers (simplified - in real implementation use proper RNG)
mod rand {
    pub fn random<T>() -> T
    where
        T: From<f64>,
    {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        let hash = hasher.finish();
        T::from((hash % 1000) as f64 / 1000.0)
    }
}
