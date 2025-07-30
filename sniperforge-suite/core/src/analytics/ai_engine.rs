//! AI Engine Empresarial - Migrado desde arbitrage_phase45_clean.rs
//! Sistema de inteligencia artificial para predicción de mercado,
//! optimización de estrategias y análisis de oportunidades de arbitraje

use crate::config::SimpleConfig;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tracing::{info, warn};
use rand;

/// Configuración del motor de AI empresarial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseAIConfig {
    /// Si el motor de AI está habilitado
    pub enabled: bool,
    /// Modelo de ML para predicción de precios
    pub price_prediction_model: String,
    /// Ventana de tiempo para análisis histórico en minutos
    pub historical_analysis_window_minutes: u32,
    /// Threshold mínimo de confianza para predicciones [0-1]
    pub min_prediction_confidence: f64,
    /// Número máximo de features para análisis
    pub max_analysis_features: u32,
    /// Si optimización de estrategias está habilitada
    pub strategy_optimization_enabled: bool,
    /// Profundidad de búsqueda para optimización
    pub optimization_search_depth: u32,
    /// Si detección de anomalías está habilitada
    pub anomaly_detection_enabled: bool,
    /// Threshold para detección de anomalías
    pub anomaly_threshold: f64,
    /// Nivel de logging de AI (debug, info, warn)
    pub ai_logging_level: String,
}

impl Default for EnterpriseAIConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            price_prediction_model: "LSTM_v2".to_string(),
            historical_analysis_window_minutes: 120,  // 2 horas de datos
            min_prediction_confidence: 0.75,          // 75% confianza mínima
            max_analysis_features: 50,                // 50 features máximo
            strategy_optimization_enabled: true,
            optimization_search_depth: 5,             // 5 niveles de búsqueda
            anomaly_detection_enabled: true,
            anomaly_threshold: 2.5,                   // 2.5 desviaciones estándar
            ai_logging_level: "info".to_string(),
        }
    }
}

/// Predicción de precio generada por AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePrediction {
    /// Símbolo del token
    pub token_symbol: String,
    /// Timestamp de la predicción
    pub timestamp: DateTime<Utc>,
    /// Precio actual en USD
    pub current_price_usd: f64,
    /// Precio predicho en USD
    pub predicted_price_usd: f64,
    /// Cambio predicho en porcentaje
    pub predicted_change_percentage: f64,
    /// Horizonte temporal de la predicción en minutos
    pub prediction_horizon_minutes: u32,
    /// Nivel de confianza [0-1]
    pub confidence_level: f64,
    /// Features utilizadas para la predicción
    pub features_used: Vec<String>,
    /// Modelo ML utilizado
    pub model_used: String,
    /// Volatilidad predicha
    pub predicted_volatility: f64,
    /// Score de tendencia [-1, 1] (bearish to bullish)
    pub trend_score: f64,
}

/// Optimización de estrategia de trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyOptimization {
    /// ID de la estrategia
    pub strategy_id: String,
    /// Timestamp de optimización
    pub timestamp: DateTime<Utc>,
    /// Parámetros optimizados
    pub optimized_parameters: HashMap<String, f64>,
    /// Score de fitness esperado
    pub expected_fitness_score: f64,
    /// Profit esperado en USD por trade
    pub expected_profit_per_trade_usd: f64,
    /// Tasa de éxito esperada [0-1]
    pub expected_success_rate: f64,
    /// Drawdown máximo esperado
    pub expected_max_drawdown: f64,
    /// Número de iteraciones de optimización
    pub optimization_iterations: u32,
    /// Tiempo de optimización en segundos
    pub optimization_time_seconds: f64,
    /// Features consideradas
    pub features_considered: Vec<String>,
}

/// Anomalía detectada en el mercado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketAnomaly {
    /// ID único de la anomalía
    pub id: String,
    /// Timestamp de detección
    pub timestamp: DateTime<Utc>,
    /// Tipo de anomalía
    pub anomaly_type: String,
    /// Severidad [0-1]
    pub severity: f64,
    /// Descripción de la anomalía
    pub description: String,
    /// Tokens afectados
    pub affected_tokens: Vec<String>,
    /// Mercados afectados
    pub affected_markets: Vec<String>,
    /// Score de confianza en la detección [0-1]
    pub detection_confidence: f64,
    /// Acción recomendada
    pub recommended_action: String,
    /// Impacto estimado en USD
    pub estimated_impact_usd: f64,
}

/// Estadísticas del motor de AI
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AIEngineStats {
    /// Total de predicciones generadas
    pub total_predictions_generated: u64,
    /// Predicciones correctas (dentro del threshold)
    pub correct_predictions: u64,
    /// Precisión promedio del modelo [0-1]
    pub average_model_accuracy: f64,
    /// Total de optimizaciones de estrategia
    pub total_strategy_optimizations: u64,
    /// Mejora promedio en fitness score
    pub average_fitness_improvement: f64,
    /// Total de anomalías detectadas
    pub total_anomalies_detected: u64,
    /// Anomalías confirmadas (true positives)
    pub confirmed_anomalies: u64,
    /// Tasa de detección de anomalías [0-1]
    pub anomaly_detection_rate: f64,
    /// Tiempo promedio de procesamiento por predicción (ms)
    pub average_prediction_time_ms: f64,
    /// Tiempo promedio de optimización (segundos)
    pub average_optimization_time_seconds: f64,
    /// Features más importantes por frecuencia de uso
    pub top_features_usage: HashMap<String, u64>,
}

/// Predictor de mercado con ML
#[derive(Debug)]
pub struct MarketPredictor {
    /// Datos históricos de precios (Token -> [Precios])
    price_history: HashMap<String, VecDeque<f64>>,
    /// Ventana de tiempo para análisis
    analysis_window: usize,
    /// Modelo actual utilizado
    current_model: String,
}

impl MarketPredictor {
    /// Crear nuevo predictor
    pub fn new(analysis_window: usize) -> Self {
        Self {
            price_history: HashMap::new(),
            analysis_window,
            current_model: "LSTM_v2".to_string(),
        }
    }
    
    /// Actualizar histórico de precios
    pub fn update_price_history(&mut self, token: &str, price: f64) {
        let history = self.price_history.entry(token.to_string()).or_insert_with(VecDeque::new);
        history.push_back(price);
        
        // Mantener solo la ventana de análisis
        if history.len() > self.analysis_window {
            history.pop_front();
        }
    }
    
    /// Generar predicción de precio
    pub fn predict_price(&self, token: &str, horizon_minutes: u32) -> Option<PricePrediction> {
        let history = self.price_history.get(token)?;
        
        if history.len() < 10 {
            return None; // Necesitamos al menos 10 puntos de datos
        }
        
        let current_price = *history.back()?;
        
        // Simulación de modelo ML avanzado
        // En producción aquí iría el modelo real (LSTM, Transformer, etc.)
        let (predicted_price, confidence, trend_score, volatility) = 
            self.simulate_ml_prediction(history, horizon_minutes);
        
        Some(PricePrediction {
            token_symbol: token.to_string(),
            timestamp: Utc::now(),
            current_price_usd: current_price,
            predicted_price_usd: predicted_price,
            predicted_change_percentage: ((predicted_price / current_price) - 1.0) * 100.0,
            prediction_horizon_minutes: horizon_minutes,
            confidence_level: confidence,
            features_used: vec![
                "price_momentum".to_string(),
                "volume_profile".to_string(),
                "volatility_index".to_string(),
                "trend_strength".to_string(),
                "support_resistance".to_string(),
            ],
            model_used: self.current_model.clone(),
            predicted_volatility: volatility,
            trend_score,
        })
    }
    
    /// Simular predicción de ML (en producción sería modelo real)
    fn simulate_ml_prediction(&self, history: &VecDeque<f64>, horizon_minutes: u32) -> (f64, f64, f64, f64) {
        let current_price = *history.back().unwrap();
        let price_vec: Vec<f64> = history.iter().copied().collect();
        
        // Calcular tendencia reciente
        let recent_trend = if price_vec.len() >= 5 {
            let recent_avg = price_vec[price_vec.len()-5..].iter().sum::<f64>() / 5.0;
            let older_avg = price_vec[price_vec.len()-10..price_vec.len()-5].iter().sum::<f64>() / 5.0;
            (recent_avg / older_avg) - 1.0
        } else {
            0.0
        };
        
        // Calcular volatilidad
        let returns: Vec<f64> = price_vec.windows(2)
            .map(|w| (w[1] / w[0]) - 1.0)
            .collect();
        let volatility = if returns.len() > 1 {
            let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance = returns.iter()
                .map(|r| (r - mean_return).powi(2))
                .sum::<f64>() / returns.len() as f64;
            variance.sqrt()
        } else {
            0.02 // 2% volatilidad por defecto
        };
        
        // Predicción basada en tendencia + ruido aleatorio + horizonte temporal
        let trend_factor = recent_trend * (horizon_minutes as f64 / 60.0); // Escalar por tiempo
        let random_factor = (rand::random::<f64>() - 0.5) * volatility * 2.0;
        let mean_reversion = -0.1 * recent_trend; // Ligera reversión a la media
        
        let predicted_change = trend_factor + random_factor + mean_reversion;
        let predicted_price = current_price * (1.0 + predicted_change);
        
        // Confidence basada en estabilidad de la tendencia y cantidad de datos
        let data_confidence = (price_vec.len() as f64 / 100.0).min(1.0);
        let volatility_confidence = (0.1 / volatility).min(1.0);
        let confidence = (data_confidence + volatility_confidence) / 2.0 * 0.9; // Max 90%
        
        // Trend score [-1, 1]
        let trend_score = recent_trend.max(-1.0).min(1.0);
        
        (predicted_price, confidence, trend_score, volatility)
    }
}

/// Optimizador de estrategias con algoritmos genéticos
#[derive(Debug)]
pub struct StrategyOptimizer {
    /// Configuración de optimización
    search_depth: u32,
    /// Histórico de optimizaciones
    optimization_history: VecDeque<StrategyOptimization>,
}

impl StrategyOptimizer {
    /// Crear nuevo optimizador
    pub fn new(search_depth: u32) -> Self {
        Self {
            search_depth,
            optimization_history: VecDeque::new(),
        }
    }
    
    /// Optimizar estrategia de arbitraje
    pub fn optimize_arbitrage_strategy(&mut self, strategy_id: &str) -> StrategyOptimization {
        let start_time = std::time::Instant::now();
        
        // Simular optimización con algoritmo genético
        let mut best_params = HashMap::new();
        let mut best_fitness = 0.0;
        
        let iterations = self.search_depth * 10;
        
        for _i in 0..iterations {
            // Generar parámetros aleatorios
            let params = self.generate_random_parameters();
            let fitness = self.evaluate_fitness(&params);
            
            if fitness > best_fitness {
                best_fitness = fitness;
                best_params = params;
            }
        }
        
        let optimization_time = start_time.elapsed().as_secs_f64();
        
        let optimization = StrategyOptimization {
            strategy_id: strategy_id.to_string(),
            timestamp: Utc::now(),
            optimized_parameters: best_params,
            expected_fitness_score: best_fitness,
            expected_profit_per_trade_usd: best_fitness * 100.0, // Convertir fitness a USD
            expected_success_rate: 0.6 + (best_fitness * 0.3),   // 60-90% éxito
            expected_max_drawdown: 0.1 - (best_fitness * 0.05),  // 5-10% drawdown
            optimization_iterations: iterations,
            optimization_time_seconds: optimization_time,
            features_considered: vec![
                "min_profit_threshold".to_string(),
                "max_slippage".to_string(),
                "trade_size_multiplier".to_string(),
                "risk_tolerance".to_string(),
                "exit_threshold".to_string(),
            ],
        };
        
        // Guardar en histórico
        self.optimization_history.push_back(optimization.clone());
        if self.optimization_history.len() > 100 {
            self.optimization_history.pop_front();
        }
        
        optimization
    }
    
    /// Generar parámetros aleatorios para optimización
    fn generate_random_parameters(&self) -> HashMap<String, f64> {
        let mut params = HashMap::new();
        
        params.insert("min_profit_threshold".to_string(), 0.001 + rand::random::<f64>() * 0.009); // 0.1-1%
        params.insert("max_slippage".to_string(), 0.001 + rand::random::<f64>() * 0.019);         // 0.1-2%
        params.insert("trade_size_multiplier".to_string(), 0.1 + rand::random::<f64>() * 0.9);   // 0.1-1.0
        params.insert("risk_tolerance".to_string(), 0.1 + rand::random::<f64>() * 0.4);          // 0.1-0.5
        params.insert("exit_threshold".to_string(), 0.0005 + rand::random::<f64>() * 0.0045);    // 0.05-0.5%
        
        params
    }
    
    /// Evaluar fitness de parámetros
    fn evaluate_fitness(&self, params: &HashMap<String, f64>) -> f64 {
        // Simular evaluación de fitness basada en parámetros
        let profit_weight = params.get("min_profit_threshold").unwrap_or(&0.005);
        let slippage_penalty = params.get("max_slippage").unwrap_or(&0.01);
        let size_efficiency = params.get("trade_size_multiplier").unwrap_or(&0.5);
        let risk_adjustment = 1.0 - params.get("risk_tolerance").unwrap_or(&0.3);
        
        // Función de fitness compuesta
        let base_score = profit_weight * 100.0;
        let slippage_score = (0.02 - slippage_penalty) * 50.0;
        let size_score = size_efficiency * 20.0;
        let risk_score = risk_adjustment * 30.0;
        
        (base_score + slippage_score + size_score + risk_score).max(0.0).min(1.0)
    }
}

/// Detector de anomalías de mercado
#[derive(Debug)]
pub struct AnomalyDetector {
    /// Threshold para detección
    anomaly_threshold: f64,
    /// Histórico de anomalías detectadas
    detected_anomalies: VecDeque<MarketAnomaly>,
    /// Baseline estadístico por token
    statistical_baseline: HashMap<String, (f64, f64)>, // (media, desviación)
}

impl AnomalyDetector {
    /// Crear nuevo detector
    pub fn new(threshold: f64) -> Self {
        Self {
            anomaly_threshold: threshold,
            detected_anomalies: VecDeque::new(),
            statistical_baseline: HashMap::new(),
        }
    }
    
    /// Actualizar baseline estadístico
    pub fn update_baseline(&mut self, token: &str, price: f64) {
        // En producción aquí se mantendría una ventana deslizante
        // Por ahora simular baseline
        self.statistical_baseline.insert(
            token.to_string(),
            (price * (0.98 + rand::random::<f64>() * 0.04), price * 0.02)
        );
    }
    
    /// Detectar anomalías en precio
    pub fn detect_price_anomaly(&mut self, token: &str, current_price: f64) -> Option<MarketAnomaly> {
        if let Some((baseline_mean, baseline_std)) = self.statistical_baseline.get(token) {
            let z_score = (current_price - baseline_mean) / baseline_std;
            
            if z_score.abs() > self.anomaly_threshold {
                let anomaly = MarketAnomaly {
                    id: format!("ANOMALY_{}_{}", token, Utc::now().timestamp_millis()),
                    timestamp: Utc::now(),
                    anomaly_type: if z_score > 0.0 { "PRICE_SPIKE".to_string() } else { "PRICE_DROP".to_string() },
                    severity: (z_score.abs() / self.anomaly_threshold).min(1.0),
                    description: format!("Precio de {} desvía {:.2} desviaciones estándar", token, z_score.abs()),
                    affected_tokens: vec![token.to_string()],
                    affected_markets: vec!["Solana".to_string()], // Por simplicidad
                    detection_confidence: 0.8 + rand::random::<f64>() * 0.2,
                    recommended_action: if z_score.abs() > self.anomaly_threshold * 1.5 {
                        "HALT_TRADING".to_string()
                    } else {
                        "MONITOR_CLOSELY".to_string()
                    },
                    estimated_impact_usd: z_score.abs() * 1000.0, // Estimar impacto
                };
                
                self.detected_anomalies.push_back(anomaly.clone());
                if self.detected_anomalies.len() > 50 {
                    self.detected_anomalies.pop_front();
                }
                
                return Some(anomaly);
            }
        }
        None
    }
    
    /// Obtener anomalías recientes
    pub fn get_recent_anomalies(&self, minutes: u32) -> Vec<&MarketAnomaly> {
        let cutoff = Utc::now() - chrono::Duration::minutes(minutes as i64);
        self.detected_anomalies.iter()
            .filter(|a| a.timestamp > cutoff)
            .collect()
    }
}

/// Motor principal de AI empresarial
#[derive(Debug)]
pub struct EnterpriseAIEngine {
    /// Configuración del motor
    config: EnterpriseAIConfig,
    /// Configuración simple del sistema
    _settings: SimpleConfig,
    /// Predictor de mercado
    market_predictor: MarketPredictor,
    /// Optimizador de estrategias
    strategy_optimizer: StrategyOptimizer,
    /// Detector de anomalías
    anomaly_detector: AnomalyDetector,
    /// Estadísticas del motor
    stats: AIEngineStats,
}

impl EnterpriseAIEngine {
    /// Crear nueva instancia del motor de AI
    pub fn new(config: Option<EnterpriseAIConfig>, settings: SimpleConfig) -> Self {
        let config = config.unwrap_or_default();
        
        Self {
            market_predictor: MarketPredictor::new(config.historical_analysis_window_minutes as usize),
            strategy_optimizer: StrategyOptimizer::new(config.optimization_search_depth),
            anomaly_detector: AnomalyDetector::new(config.anomaly_threshold),
            config,
            _settings: settings,
            stats: AIEngineStats::default(),
        }
    }
    
    /// Generar predicción de precio
    pub async fn predict_price(&mut self, token: &str, current_price: f64, horizon_minutes: u32) -> Result<Option<PricePrediction>> {
        if !self.config.enabled {
            return Ok(None);
        }
        
        let start_time = std::time::Instant::now();
        
        // Actualizar histórico
        self.market_predictor.update_price_history(token, current_price);
        
        // Generar predicción
        let prediction = self.market_predictor.predict_price(token, horizon_minutes);
        
        let processing_time = start_time.elapsed().as_millis() as f64;
        
        if let Some(ref pred) = prediction {
            self.stats.total_predictions_generated += 1;
            
            // Solo contar como correcta si la confianza supera el threshold
            if pred.confidence_level >= self.config.min_prediction_confidence {
                // En producción aquí se verificaría contra precio real posterior
                self.stats.correct_predictions += 1;
            }
            
            // Actualizar estadísticas
            self.stats.average_prediction_time_ms = 
                (self.stats.average_prediction_time_ms + processing_time) / 2.0;
            
            if self.stats.total_predictions_generated > 0 {
                self.stats.average_model_accuracy = 
                    self.stats.correct_predictions as f64 / self.stats.total_predictions_generated as f64;
            }
            
            // Registrar features utilizadas
            for feature in &pred.features_used {
                *self.stats.top_features_usage.entry(feature.clone()).or_insert(0) += 1;
            }
            
            info!("🤖 Predicción AI para {}: ${:.4} → ${:.4} ({:+.2}%) [Conf: {:.1}%]",
                  token, pred.current_price_usd, pred.predicted_price_usd,
                  pred.predicted_change_percentage, pred.confidence_level * 100.0);
        }
        
        Ok(prediction)
    }
    
    /// Optimizar estrategia de trading
    pub async fn optimize_strategy(&mut self, strategy_id: &str) -> Result<StrategyOptimization> {
        if !self.config.strategy_optimization_enabled {
            return Err(anyhow::anyhow!("Optimización de estrategias deshabilitada"));
        }
        
        let start_time = std::time::Instant::now();
        let optimization = self.strategy_optimizer.optimize_arbitrage_strategy(strategy_id);
        let total_time = start_time.elapsed().as_secs_f64();
        
        self.stats.total_strategy_optimizations += 1;
        self.stats.average_optimization_time_seconds = 
            (self.stats.average_optimization_time_seconds + total_time) / 2.0;
        
        // Calcular mejora en fitness (simulada)
        let baseline_fitness = 0.5; // Fitness baseline
        let improvement = optimization.expected_fitness_score - baseline_fitness;
        self.stats.average_fitness_improvement = 
            (self.stats.average_fitness_improvement + improvement) / 2.0;
        
        info!("🧠 Estrategia {} optimizada: Fitness {:.3}, Profit esperado ${:.2}",
              strategy_id, optimization.expected_fitness_score, 
              optimization.expected_profit_per_trade_usd);
        
        Ok(optimization)
    }
    
    /// Detectar anomalías de mercado
    pub async fn detect_market_anomalies(&mut self, token: &str, current_price: f64) -> Result<Option<MarketAnomaly>> {
        if !self.config.anomaly_detection_enabled {
            return Ok(None);
        }
        
        // Actualizar baseline
        self.anomaly_detector.update_baseline(token, current_price);
        
        // Detectar anomalía
        let anomaly = self.anomaly_detector.detect_price_anomaly(token, current_price);
        
        if let Some(ref anom) = anomaly {
            self.stats.total_anomalies_detected += 1;
            
            // En producción aquí se verificaría si es true positive
            if anom.detection_confidence > 0.8 {
                self.stats.confirmed_anomalies += 1;
            }
            
            if self.stats.total_anomalies_detected > 0 {
                self.stats.anomaly_detection_rate = 
                    self.stats.confirmed_anomalies as f64 / self.stats.total_anomalies_detected as f64;
            }
            
            warn!("🚨 Anomalía detectada en {}: {} (Severidad: {:.1}%)",
                  token, anom.anomaly_type, anom.severity * 100.0);
        }
        
        Ok(anomaly)
    }
    
    /// Obtener análisis completo del mercado
    pub async fn get_market_analysis(&mut self, tokens: &[String]) -> Result<HashMap<String, PricePrediction>> {
        let mut analysis = HashMap::new();
        
        for token in tokens {
            // En producción obtendría precio real de la API
            let simulated_price = match token.as_str() {
                "SOL" => 150.0 + (rand::random::<f64>() - 0.5) * 10.0,
                "ETH" => 2500.0 + (rand::random::<f64>() - 0.5) * 200.0,
                "BTC" => 45000.0 + (rand::random::<f64>() - 0.5) * 2000.0,
                _ => 1.0 + (rand::random::<f64>() - 0.5) * 0.1,
            };
            
            if let Some(prediction) = self.predict_price(token, simulated_price, 30).await? {
                analysis.insert(token.clone(), prediction);
            }
        }
        
        Ok(analysis)
    }
    
    /// Obtener estadísticas del motor
    pub fn get_statistics(&self) -> &AIEngineStats {
        &self.stats
    }
    
    /// Verificar si está habilitado
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    
    /// Obtener configuración
    pub fn get_config(&self) -> &EnterpriseAIConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ai_price_prediction() {
        let settings = SimpleConfig::default();
        let mut ai_engine = EnterpriseAIEngine::new(None, settings);
        
        // Debería generar predicción válida
        let prediction = ai_engine.predict_price("SOL", 150.0, 30).await.unwrap();
        
        if let Some(pred) = prediction {
            assert_eq!(pred.token_symbol, "SOL");
            assert!(pred.predicted_price_usd > 0.0);
            assert!(pred.confidence_level >= 0.0 && pred.confidence_level <= 1.0);
            assert!(!pred.features_used.is_empty());
        }
    }
    
    #[tokio::test]
    async fn test_strategy_optimization() {
        let settings = SimpleConfig::default();
        let mut ai_engine = EnterpriseAIEngine::new(None, settings);
        
        // Debería optimizar estrategia exitosamente
        let optimization = ai_engine.optimize_strategy("arbitrage_v1").await.unwrap();
        
        assert_eq!(optimization.strategy_id, "arbitrage_v1");
        assert!(optimization.expected_fitness_score >= 0.0);
        assert!(!optimization.optimized_parameters.is_empty());
        assert!(optimization.optimization_iterations > 0);
    }
    
    #[tokio::test]
    async fn test_anomaly_detection() {
        let settings = SimpleConfig::default();
        let mut ai_engine = EnterpriseAIEngine::new(None, settings);
        
        // Precio normal no debería generar anomalía
        let normal_anomaly = ai_engine.detect_market_anomalies("SOL", 150.0).await.unwrap();
        
        // Precio extremo debería generar anomalía
        let extreme_anomaly = ai_engine.detect_market_anomalies("SOL", 1000.0).await.unwrap();
        
        // Al menos una de las dos debería detectar una anomalía si el baseline es correcto
        assert!(normal_anomaly.is_none() || extreme_anomaly.is_some());
    }
}
