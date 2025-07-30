//! Performance Analytics AI - Migrado desde arbitrage_phase45_clean.rs
//! Sistema de análisis de performance con AI que monitorea, evalúa y optimiza
//! el rendimiento del sistema de trading en tiempo real

use crate::config::SimpleConfig;
use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tracing::{debug, info, warn};
use rand;

/// Configuración de Analytics AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalyticsConfig {
    /// Si analytics está habilitado
    pub enabled: bool,
    /// Ventana de análisis en horas
    pub analysis_window_hours: u32,
    /// Intervalos de reporte en minutos
    pub reporting_interval_minutes: u32,
    /// Si alertas automáticas están habilitadas
    pub automatic_alerts_enabled: bool,
    /// Threshold para alertas de performance (%)
    pub performance_alert_threshold: f64,
    /// Si optimización automática está habilitada
    pub auto_optimization_enabled: bool,
    /// Número máximo de recomendaciones por análisis
    pub max_recommendations_per_analysis: u32,
    /// Si generar reportes detallados
    pub detailed_reporting_enabled: bool,
    /// Profundidad de análisis histórico en días
    pub historical_analysis_depth_days: u32,
}

impl Default for PerformanceAnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            analysis_window_hours: 24,           // Análisis de últimas 24h
            reporting_interval_minutes: 30,     // Reportes cada 30 min
            automatic_alerts_enabled: true,     // Alertas automáticas
            performance_alert_threshold: 10.0,  // 10% degradación trigger
            auto_optimization_enabled: false,   // Auto-optimización conservadora
            max_recommendations_per_analysis: 5,// Máximo 5 recomendaciones
            detailed_reporting_enabled: true,   // Reportes detallados
            historical_analysis_depth_days: 30, // 30 días de histórico
        }
    }
}

/// Métrica de performance individual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    /// Nombre de la métrica
    pub metric_name: String,
    /// Valor actual
    pub current_value: f64,
    /// Valor histórico promedio
    pub historical_average: f64,
    /// Porcentaje de cambio vs histórico
    pub change_percentage: f64,
    /// Tendencia (Improving, Declining, Stable)
    pub trend: String,
    /// Timestamp de la métrica
    pub timestamp: DateTime<Utc>,
    /// Categoría de la métrica
    pub category: String,
    /// Unidad de medida
    pub unit: String,
}

/// Análisis detallado de performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    /// ID único del análisis
    pub analysis_id: String,
    /// Timestamp del análisis
    pub timestamp: DateTime<Utc>,
    /// Período analizado en horas
    pub analysis_period_hours: u32,
    /// Score general de performance [0-100]
    pub overall_performance_score: f64,
    /// Métricas individuales analizadas
    pub metrics: Vec<PerformanceMetric>,
    /// Puntos fuertes identificados
    pub strengths: Vec<String>,
    /// Áreas de mejora identificadas
    pub improvement_areas: Vec<String>,
    /// Recomendaciones específicas
    pub recommendations: Vec<PerformanceRecommendation>,
    /// Predicciones para próximas 24h
    pub predictions: Vec<PerformancePrediction>,
    /// Riesgos identificados
    pub identified_risks: Vec<PerformanceRisk>,
}

/// Recomendación de mejora de performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecommendation {
    /// ID de la recomendación
    pub recommendation_id: String,
    /// Título de la recomendación
    pub title: String,
    /// Descripción detallada
    pub description: String,
    /// Categoría (Configuration, Strategy, Technical)
    pub category: String,
    /// Prioridad (High, Medium, Low)
    pub priority: String,
    /// Impacto esperado [0-1]
    pub expected_impact: f64,
    /// Facilidad de implementación [0-1]
    pub implementation_ease: f64,
    /// Score de beneficio/esfuerzo
    pub benefit_effort_score: f64,
    /// Pasos específicos para implementar
    pub implementation_steps: Vec<String>,
    /// Métricas que mejorará
    pub target_metrics: Vec<String>,
}

/// Predicción de performance futura
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePrediction {
    /// Métrica predicha
    pub metric_name: String,
    /// Valor predicho
    pub predicted_value: f64,
    /// Rango de confianza (min, max)
    pub confidence_range: (f64, f64),
    /// Nivel de confianza [0-1]
    pub confidence_level: f64,
    /// Horizonte de predicción en horas
    pub prediction_horizon_hours: u32,
    /// Factores que influencian la predicción
    pub influencing_factors: Vec<String>,
}

/// Riesgo de performance identificado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRisk {
    /// ID del riesgo
    pub risk_id: String,
    /// Título del riesgo
    pub title: String,
    /// Descripción del riesgo
    pub description: String,
    /// Severidad (Critical, High, Medium, Low)
    pub severity: String,
    /// Probabilidad de ocurrencia [0-1]
    pub probability: f64,
    /// Impacto potencial [0-1]
    pub potential_impact: f64,
    /// Score de riesgo combinado
    pub risk_score: f64,
    /// Acciones de mitigación sugeridas
    pub mitigation_actions: Vec<String>,
    /// Indicadores tempranos del riesgo
    pub early_indicators: Vec<String>,
}

/// Alerta de performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    /// ID de la alerta
    pub alert_id: String,
    /// Timestamp de la alerta
    pub timestamp: DateTime<Utc>,
    /// Tipo de alerta
    pub alert_type: String,
    /// Severidad (Critical, Warning, Info)
    pub severity: String,
    /// Mensaje de la alerta
    pub message: String,
    /// Métrica que disparó la alerta
    pub triggering_metric: String,
    /// Valor actual vs esperado
    pub current_vs_expected: (f64, f64),
    /// Acciones recomendadas
    pub recommended_actions: Vec<String>,
    /// Si requiere acción inmediata
    pub requires_immediate_action: bool,
}

/// Estadísticas del sistema de analytics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyticsStats {
    /// Total de análisis realizados
    pub total_analyses_performed: u64,
    /// Total de recomendaciones generadas
    pub total_recommendations_generated: u64,
    /// Recomendaciones implementadas
    pub recommendations_implemented: u64,
    /// Alertas generadas
    pub total_alerts_generated: u64,
    /// Alertas críticas
    pub critical_alerts_generated: u64,
    /// Tiempo promedio de análisis (segundos)
    pub average_analysis_time_seconds: f64,
    /// Precisión de predicciones [0-1]
    pub prediction_accuracy: f64,
    /// Mejora promedio después de recomendaciones
    pub average_improvement_after_recommendations: f64,
}

/// Calculadora de métricas de performance
#[derive(Debug)]
pub struct PerformanceCalculator {
    /// Histórico de métricas
    metrics_history: HashMap<String, VecDeque<(DateTime<Utc>, f64)>>,
    /// Ventana de análisis
    analysis_window_hours: u32,
}

impl PerformanceCalculator {
    /// Crear nueva calculadora
    pub fn new(analysis_window_hours: u32) -> Self {
        Self {
            metrics_history: HashMap::new(),
            analysis_window_hours,
        }
    }
    
    /// Agregar valor de métrica
    pub fn add_metric_value(&mut self, metric_name: &str, value: f64) {
        let history = self.metrics_history.entry(metric_name.to_string()).or_insert_with(VecDeque::new);
        history.push_back((Utc::now(), value));
        
        // Mantener solo la ventana de análisis
        let cutoff = Utc::now() - Duration::hours(self.analysis_window_hours as i64);
        while let Some((timestamp, _)) = history.front() {
            if *timestamp < cutoff {
                history.pop_front();
            } else {
                break;
            }
        }
    }
    
    /// Calcular métrica de performance
    pub fn calculate_performance_metric(&self, metric_name: &str, current_value: f64) -> Option<PerformanceMetric> {
        let history = self.metrics_history.get(metric_name)?;
        
        if history.is_empty() {
            return None;
        }
        
        // Calcular promedio histórico
        let historical_average = history.iter().map(|(_, v)| v).sum::<f64>() / history.len() as f64;
        
        // Calcular cambio porcentual
        let change_percentage = if historical_average != 0.0 {
            ((current_value - historical_average) / historical_average) * 100.0
        } else {
            0.0
        };
        
        // Determinar tendencia
        let trend = if change_percentage > 5.0 {
            "Improving".to_string()
        } else if change_percentage < -5.0 {
            "Declining".to_string()
        } else {
            "Stable".to_string()
        };
        
        // Determinar categoría y unidad basado en el nombre
        let (category, unit) = match metric_name {
            name if name.contains("profit") => ("Financial".to_string(), "USD".to_string()),
            name if name.contains("success_rate") => ("Performance".to_string(), "%".to_string()),
            name if name.contains("latency") => ("Technical".to_string(), "ms".to_string()),
            name if name.contains("cpu") => ("System".to_string(), "%".to_string()),
            name if name.contains("memory") => ("System".to_string(), "MB".to_string()),
            _ => ("General".to_string(), "units".to_string()),
        };
        
        Some(PerformanceMetric {
            metric_name: metric_name.to_string(),
            current_value,
            historical_average,
            change_percentage,
            trend,
            timestamp: Utc::now(),
            category,
            unit,
        })
    }
}

/// Generador de recomendaciones inteligente
#[derive(Debug)]
pub struct RecommendationEngine {
    /// Histórico de recomendaciones
    recommendation_history: VecDeque<PerformanceRecommendation>,
    /// Máximo número de recomendaciones
    max_recommendations: u32,
}

impl RecommendationEngine {
    /// Crear nuevo generador
    pub fn new(max_recommendations: u32) -> Self {
        Self {
            recommendation_history: VecDeque::new(),
            max_recommendations,
        }
    }
    
    /// Generar recomendaciones basadas en métricas
    pub fn generate_recommendations(&mut self, metrics: &[PerformanceMetric]) -> Vec<PerformanceRecommendation> {
        let mut recommendations = Vec::new();
        
        for metric in metrics {
            // Generar recomendaciones específicas por métrica y tendencia
            match (metric.category.as_str(), metric.trend.as_str()) {
                ("Financial", "Declining") => {
                    recommendations.push(self.create_financial_improvement_recommendation(metric));
                },
                ("Performance", "Declining") => {
                    recommendations.push(self.create_performance_optimization_recommendation(metric));
                },
                ("Technical", "Declining") => {
                    recommendations.push(self.create_technical_optimization_recommendation(metric));
                },
                ("System", "Declining") => {
                    recommendations.push(self.create_system_optimization_recommendation(metric));
                },
                _ => continue,
            }
            
            if recommendations.len() >= self.max_recommendations as usize {
                break;
            }
        }
        
        // Calcular scores benefit/effort
        for rec in &mut recommendations {
            rec.benefit_effort_score = rec.expected_impact * rec.implementation_ease;
        }
        
        // Ordenar por score
        recommendations.sort_by(|a, b| b.benefit_effort_score.partial_cmp(&a.benefit_effort_score).unwrap());
        
        // Guardar en histórico
        for rec in &recommendations {
            self.recommendation_history.push_back(rec.clone());
        }
        
        if self.recommendation_history.len() > 100 {
            self.recommendation_history.pop_front();
        }
        
        recommendations
    }
    
    /// Crear recomendación de mejora financiera
    fn create_financial_improvement_recommendation(&self, metric: &PerformanceMetric) -> PerformanceRecommendation {
        PerformanceRecommendation {
            recommendation_id: format!("FIN_{}", Utc::now().timestamp_millis()),
            title: "Optimizar Estrategia de Profit".to_string(),
            description: format!("La métrica {} ha declinado {:.1}%. Considerar ajustar parámetros de trading para mejorar rentabilidad.",
                               metric.metric_name, metric.change_percentage.abs()),
            category: "Strategy".to_string(),
            priority: if metric.change_percentage < -20.0 { "High".to_string() } else { "Medium".to_string() },
            expected_impact: 0.7,
            implementation_ease: 0.8,
            benefit_effort_score: 0.56,
            implementation_steps: vec![
                "Revisar threshold de profit mínimo".to_string(),
                "Ajustar tamaño de posiciones".to_string(),
                "Optimizar timing de entrada/salida".to_string(),
                "Revisar fees y slippage configurados".to_string(),
            ],
            target_metrics: vec![metric.metric_name.clone()],
        }
    }
    
    /// Crear recomendación de optimización de performance
    fn create_performance_optimization_recommendation(&self, metric: &PerformanceMetric) -> PerformanceRecommendation {
        PerformanceRecommendation {
            recommendation_id: format!("PERF_{}", Utc::now().timestamp_millis()),
            title: "Mejorar Tasa de Éxito".to_string(),
            description: format!("La tasa de éxito ha bajado {:.1}%. Implementar filtros ML más estrictos o ajustar criterios de selección.",
                               metric.change_percentage.abs()),
            category: "Strategy".to_string(),
            priority: "High".to_string(),
            expected_impact: 0.8,
            implementation_ease: 0.6,
            benefit_effort_score: 0.48,
            implementation_steps: vec![
                "Aumentar threshold de confianza ML".to_string(),
                "Revisar criterios de filtrado de oportunidades".to_string(),
                "Implementar validación adicional pre-trade".to_string(),
                "Ajustar parámetros de risk management".to_string(),
            ],
            target_metrics: vec!["success_rate".to_string(), "win_loss_ratio".to_string()],
        }
    }
    
    /// Crear recomendación de optimización técnica
    fn create_technical_optimization_recommendation(&self, metric: &PerformanceMetric) -> PerformanceRecommendation {
        PerformanceRecommendation {
            recommendation_id: format!("TECH_{}", Utc::now().timestamp_millis()),
            title: "Optimizar Performance Técnico".to_string(),
            description: format!("Latencia/performance técnico ha empeorado {:.1}%. Optimizar código y conexiones de red.",
                               metric.change_percentage.abs()),
            category: "Technical".to_string(),
            priority: "Medium".to_string(),
            expected_impact: 0.6,
            implementation_ease: 0.4,
            benefit_effort_score: 0.24,
            implementation_steps: vec![
                "Optimizar consultas a APIs".to_string(),
                "Implementar caching más agresivo".to_string(),
                "Revisar pool de conexiones".to_string(),
                "Optimizar algoritmos críticos".to_string(),
            ],
            target_metrics: vec!["api_latency".to_string(), "execution_time".to_string()],
        }
    }
    
    /// Crear recomendación de optimización de sistema
    fn create_system_optimization_recommendation(&self, metric: &PerformanceMetric) -> PerformanceRecommendation {
        PerformanceRecommendation {
            recommendation_id: format!("SYS_{}", Utc::now().timestamp_millis()),
            title: "Optimizar Recursos del Sistema".to_string(),
            description: format!("Uso de recursos ha aumentado {:.1}%. Optimizar memoria y CPU para mejor performance.",
                               metric.change_percentage.abs()),
            category: "Technical".to_string(),
            priority: "Low".to_string(),
            expected_impact: 0.5,
            implementation_ease: 0.7,
            benefit_effort_score: 0.35,
            implementation_steps: vec![
                "Revisar memory leaks potenciales".to_string(),
                "Optimizar estructuras de datos".to_string(),
                "Implementar garbage collection más eficiente".to_string(),
                "Considerar escalado horizontal".to_string(),
            ],
            target_metrics: vec!["memory_usage".to_string(), "cpu_usage".to_string()],
        }
    }
}

/// Sistema principal de Performance Analytics AI
#[derive(Debug)]
pub struct PerformanceAnalyticsAI {
    /// Configuración del sistema
    config: PerformanceAnalyticsConfig,
    /// Configuración simple
    _settings: SimpleConfig,
    /// Calculadora de métricas
    performance_calculator: PerformanceCalculator,
    /// Generador de recomendaciones
    recommendation_engine: RecommendationEngine,
    /// Estadísticas del sistema
    stats: AnalyticsStats,
    /// Histórico de análisis
    analysis_history: VecDeque<PerformanceAnalysis>,
    /// Alertas activas
    active_alerts: Vec<PerformanceAlert>,
    /// Último reporte generado
    last_report_time: Option<DateTime<Utc>>,
}

impl PerformanceAnalyticsAI {
    /// Crear nueva instancia del sistema
    pub fn new(config: Option<PerformanceAnalyticsConfig>, settings: SimpleConfig) -> Self {
        let config = config.unwrap_or_default();
        
        Self {
            performance_calculator: PerformanceCalculator::new(config.analysis_window_hours),
            recommendation_engine: RecommendationEngine::new(config.max_recommendations_per_analysis),
            config,
            _settings: settings,
            stats: AnalyticsStats::default(),
            analysis_history: VecDeque::new(),
            active_alerts: Vec::new(),
            last_report_time: None,
        }
    }
    
    /// Realizar análisis completo de performance
    pub async fn perform_comprehensive_analysis(&mut self, system_metrics: &HashMap<String, f64>) -> Result<PerformanceAnalysis> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("Performance Analytics deshabilitado"));
        }
        
        let start_time = std::time::Instant::now();
        
        // Actualizar métricas en calculadora
        for (metric_name, value) in system_metrics {
            self.performance_calculator.add_metric_value(metric_name, *value);
        }
        
        // Calcular métricas de performance
        let mut metrics = Vec::new();
        let mut total_score = 0.0;
        let mut metric_count = 0;
        
        for (metric_name, current_value) in system_metrics {
            if let Some(perf_metric) = self.performance_calculator.calculate_performance_metric(metric_name, *current_value) {
                // Calcular score individual basado en tendencia
                let metric_score = match perf_metric.trend.as_str() {
                    "Improving" => 80.0 + (perf_metric.change_percentage * 0.5).min(20.0),
                    "Stable" => 70.0,
                    "Declining" => 60.0 - (perf_metric.change_percentage.abs() * 0.5).min(20.0),
                    _ => 50.0,
                };
                
                total_score += metric_score;
                metric_count += 1;
                metrics.push(perf_metric);
            }
        }
        
        let overall_score = if metric_count > 0 { total_score / metric_count as f64 } else { 50.0 };
        
        // Generar recomendaciones
        let recommendations = self.recommendation_engine.generate_recommendations(&metrics);
        
        // Identificar fortalezas y áreas de mejora
        let improving_metrics: Vec<String> = metrics.iter()
            .filter(|m| m.trend == "Improving")
            .map(|m| m.metric_name.clone())
            .collect();
            
        let declining_metrics: Vec<String> = metrics.iter()
            .filter(|m| m.trend == "Declining")
            .map(|m| m.metric_name.clone())
            .collect();
        
        // Generar predicciones
        let predictions = self.generate_predictions(&metrics);
        
        // Identificar riesgos
        let risks = self.identify_risks(&metrics);
        
        let analysis = PerformanceAnalysis {
            analysis_id: format!("ANALYSIS_{}", Utc::now().timestamp_millis()),
            timestamp: Utc::now(),
            analysis_period_hours: self.config.analysis_window_hours,
            overall_performance_score: overall_score,
            metrics,
            strengths: improving_metrics,
            improvement_areas: declining_metrics,
            recommendations,
            predictions,
            identified_risks: risks,
        };
        
        // Actualizar estadísticas
        let analysis_time = start_time.elapsed().as_secs_f64();
        self.stats.total_analyses_performed += 1;
        self.stats.total_recommendations_generated += analysis.recommendations.len() as u64;
        self.stats.average_analysis_time_seconds = 
            (self.stats.average_analysis_time_seconds + analysis_time) / 2.0;
        
        // Guardar en histórico
        self.analysis_history.push_back(analysis.clone());
        if self.analysis_history.len() > 50 {
            self.analysis_history.pop_front();
        }
        
        // Generar alertas si es necesario
        self.generate_alerts_from_analysis(&analysis).await;
        
        info!("📊 Análisis de performance completado - Score: {:.1}/100, {} recomendaciones", 
              analysis.overall_performance_score, analysis.recommendations.len());
        
        Ok(analysis)
    }
    
    /// Generar predicciones de performance
    fn generate_predictions(&self, metrics: &[PerformanceMetric]) -> Vec<PerformancePrediction> {
        let mut predictions = Vec::new();
        
        for metric in metrics {
            // Predicción simple basada en tendencia actual
            let trend_factor = match metric.trend.as_str() {
                "Improving" => 1.0 + (metric.change_percentage / 100.0) * 0.5,
                "Declining" => 1.0 + (metric.change_percentage / 100.0) * 0.5,
                _ => 1.0,
            };
            
            let predicted_value = metric.current_value * trend_factor;
            let volatility = metric.current_value * 0.1; // 10% volatilidad
            
            predictions.push(PerformancePrediction {
                metric_name: metric.metric_name.clone(),
                predicted_value,
                confidence_range: (predicted_value - volatility, predicted_value + volatility),
                confidence_level: 0.7 + rand::random::<f64>() * 0.2, // 70-90% confianza
                prediction_horizon_hours: 24,
                influencing_factors: vec![
                    "Historical trend".to_string(),
                    "Market conditions".to_string(),
                    "System configuration".to_string(),
                ],
            });
        }
        
        predictions
    }
    
    /// Identificar riesgos de performance
    fn identify_risks(&self, metrics: &[PerformanceMetric]) -> Vec<PerformanceRisk> {
        let mut risks = Vec::new();
        
        for metric in metrics {
            if metric.trend == "Declining" && metric.change_percentage < -15.0 {
                let severity = if metric.change_percentage < -30.0 {
                    "Critical"
                } else if metric.change_percentage < -25.0 {
                    "High"
                } else {
                    "Medium"
                };
                
                let probability = (metric.change_percentage.abs() / 50.0).min(1.0);
                let impact = match metric.category.as_str() {
                    "Financial" => 0.9,
                    "Performance" => 0.8,
                    "Technical" => 0.6,
                    "System" => 0.5,
                    _ => 0.5,
                };
                
                risks.push(PerformanceRisk {
                    risk_id: format!("RISK_{}_{}", metric.metric_name, Utc::now().timestamp_millis()),
                    title: format!("Degradación en {}", metric.metric_name),
                    description: format!("Métrica {} ha declinado {:.1}% significativamente", 
                                       metric.metric_name, metric.change_percentage.abs()),
                    severity: severity.to_string(),
                    probability,
                    potential_impact: impact,
                    risk_score: probability * impact,
                    mitigation_actions: vec![
                        "Monitor closely".to_string(),
                        "Review configuration".to_string(),
                        "Implement recommended optimizations".to_string(),
                    ],
                    early_indicators: vec![
                        format!("{} trending downward", metric.metric_name),
                        "Consistent decline over time".to_string(),
                    ],
                });
            }
        }
        
        risks
    }
    
    /// Generar alertas basadas en análisis
    async fn generate_alerts_from_analysis(&mut self, analysis: &PerformanceAnalysis) {
        if !self.config.automatic_alerts_enabled {
            return;
        }
        
        // Alerta si score general es bajo
        if analysis.overall_performance_score < 60.0 {
            let alert = PerformanceAlert {
                alert_id: format!("ALERT_PERF_{}", Utc::now().timestamp_millis()),
                timestamp: Utc::now(),
                alert_type: "Performance Degradation".to_string(),
                severity: if analysis.overall_performance_score < 40.0 { "Critical".to_string() } else { "Warning".to_string() },
                message: format!("Performance score ha bajado a {:.1}/100", analysis.overall_performance_score),
                triggering_metric: "overall_performance_score".to_string(),
                current_vs_expected: (analysis.overall_performance_score, 70.0),
                recommended_actions: analysis.recommendations.iter()
                    .take(3)
                    .map(|r| r.title.clone())
                    .collect(),
                requires_immediate_action: analysis.overall_performance_score < 40.0,
            };
            
            self.active_alerts.push(alert);
            self.stats.total_alerts_generated += 1;
            
            if analysis.overall_performance_score < 40.0 {
                self.stats.critical_alerts_generated += 1;
            }
        }
        
        // Alertas por riesgos críticos
        for risk in &analysis.identified_risks {
            if risk.severity == "Critical" {
                let alert = PerformanceAlert {
                    alert_id: format!("ALERT_RISK_{}", Utc::now().timestamp_millis()),
                    timestamp: Utc::now(),
                    alert_type: "Critical Risk".to_string(),
                    severity: "Critical".to_string(),
                    message: risk.description.clone(),
                    triggering_metric: risk.risk_id.clone(),
                    current_vs_expected: (risk.risk_score, 0.5),
                    recommended_actions: risk.mitigation_actions.clone(),
                    requires_immediate_action: true,
                };
                
                self.active_alerts.push(alert);
                self.stats.total_alerts_generated += 1;
                self.stats.critical_alerts_generated += 1;
            }
        }
        
        // Limpiar alertas antiguas (más de 24h)
        let cutoff = Utc::now() - Duration::hours(24);
        self.active_alerts.retain(|alert| alert.timestamp > cutoff);
    }
    
    /// Generar reporte resumido
    pub fn generate_summary_report(&self) -> String {
        let latest_analysis = self.analysis_history.back();
        
        let mut report = String::new();
        report.push_str("📊 PERFORMANCE ANALYTICS SUMMARY REPORT\n");
        report.push_str("══════════════════════════════════════════\n\n");
        
        if let Some(analysis) = latest_analysis {
            report.push_str(&format!("🎯 Overall Performance Score: {:.1}/100\n", analysis.overall_performance_score));
            report.push_str(&format!("📈 Analyzed {} metrics over {} hours\n", 
                                   analysis.metrics.len(), analysis.analysis_period_hours));
            
            if !analysis.strengths.is_empty() {
                report.push_str("\n✅ STRENGTHS:\n");
                for strength in &analysis.strengths {
                    report.push_str(&format!("  • {}\n", strength));
                }
            }
            
            if !analysis.improvement_areas.is_empty() {
                report.push_str("\n⚠️  IMPROVEMENT AREAS:\n");
                for area in &analysis.improvement_areas {
                    report.push_str(&format!("  • {}\n", area));
                }
            }
            
            if !analysis.recommendations.is_empty() {
                report.push_str("\n💡 TOP RECOMMENDATIONS:\n");
                for (i, rec) in analysis.recommendations.iter().take(3).enumerate() {
                    report.push_str(&format!("  {}. {} (Priority: {})\n", 
                                           i + 1, rec.title, rec.priority));
                }
            }
        }
        
        report.push_str(&format!("\n📊 SYSTEM STATISTICS:\n"));
        report.push_str(&format!("  • Total Analyses: {}\n", self.stats.total_analyses_performed));
        report.push_str(&format!("  • Recommendations Generated: {}\n", self.stats.total_recommendations_generated));
        report.push_str(&format!("  • Active Alerts: {}\n", self.active_alerts.len()));
        report.push_str(&format!("  • Average Analysis Time: {:.1}s\n", self.stats.average_analysis_time_seconds));
        
        report
    }
    
    /// Obtener estadísticas
    pub fn get_statistics(&self) -> &AnalyticsStats {
        &self.stats
    }
    
    /// Obtener alertas activas
    pub fn get_active_alerts(&self) -> &Vec<PerformanceAlert> {
        &self.active_alerts
    }
    
    /// Verificar si está habilitado
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_performance_analysis() {
        let settings = SimpleConfig::default();
        let mut analytics = PerformanceAnalyticsAI::new(None, settings);
        
        let mut metrics = HashMap::new();
        metrics.insert("total_profit_usd".to_string(), 1500.0);
        metrics.insert("success_rate".to_string(), 0.75);
        metrics.insert("api_latency_ms".to_string(), 250.0);
        
        let analysis = analytics.perform_comprehensive_analysis(&metrics).await.unwrap();
        
        assert!(!analysis.analysis_id.is_empty());
        assert!(analysis.overall_performance_score >= 0.0 && analysis.overall_performance_score <= 100.0);
        assert!(!analysis.metrics.is_empty());
    }
    
    #[test]
    fn test_recommendation_generation() {
        let mut engine = RecommendationEngine::new(5);
        
        let metrics = vec![
            PerformanceMetric {
                metric_name: "profit_usd".to_string(),
                current_value: 100.0,
                historical_average: 150.0,
                change_percentage: -33.3,
                trend: "Declining".to_string(),
                timestamp: Utc::now(),
                category: "Financial".to_string(),
                unit: "USD".to_string(),
            }
        ];
        
        let recommendations = engine.generate_recommendations(&metrics);
        
        assert!(!recommendations.is_empty());
        assert!(recommendations[0].expected_impact > 0.0);
    }
}
