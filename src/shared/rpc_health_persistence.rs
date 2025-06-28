//! RPC Health Persistence System
//! 
//! Persiste el estado de salud de los endpoints RPC para recordar
//! qué endpoints han fallado históricamente y evitar usarlos
//! inmediatamente en futuras ejecuciones.

use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{info, warn, debug};

/// Estado persistente de salud de un endpoint RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedRpcHealth {
    pub url: String,
    pub last_failure_time: Option<DateTime<Utc>>,
    pub last_success_time: Option<DateTime<Utc>>,
    pub total_failures: u64,
    pub total_successes: u64,
    pub consecutive_failures: u32,
    pub average_response_time_ms: u64,
    pub failure_types: HashMap<String, u32>, // Tipos de errores: "410 Gone", "timeout", etc.
    pub reliability_score: f64, // 0.0 (malo) a 1.0 (perfecto)
}

impl PersistedRpcHealth {
    pub fn new(url: String) -> Self {
        Self {
            url,
            last_failure_time: None,
            last_success_time: None,
            total_failures: 0,
            total_successes: 0,
            consecutive_failures: 0,
            average_response_time_ms: 0,
            failure_types: HashMap::new(),
            reliability_score: 1.0, // Comienza con score perfecto
        }
    }

    /// Registra un fallo con el tipo de error específico
    pub fn record_failure(&mut self, error_type: &str) {
        self.last_failure_time = Some(Utc::now());
        self.total_failures += 1;
        self.consecutive_failures += 1;
        
        // Registra el tipo de error
        *self.failure_types.entry(error_type.to_string()).or_insert(0) += 1;
        
        // Recalcula el score de confiabilidad
        self.update_reliability_score();
        
        debug!("📉 RPC {} failure recorded: {} (consecutive: {})", 
               self.url, error_type, self.consecutive_failures);
    }

    /// Registra un éxito
    pub fn record_success(&mut self, response_time_ms: u64) {
        self.last_success_time = Some(Utc::now());
        self.total_successes += 1;
        self.consecutive_failures = 0; // Reset contador de fallos consecutivos
        
        // Actualiza tiempo promedio de respuesta
        if self.total_successes == 1 {
            self.average_response_time_ms = response_time_ms;
        } else {
            let total_time = self.average_response_time_ms * (self.total_successes - 1);
            self.average_response_time_ms = (total_time + response_time_ms) / self.total_successes;
        }
        
        // Recalcula el score de confiabilidad
        self.update_reliability_score();
        
        debug!("📈 RPC {} success recorded: {}ms (score: {:.2})", 
               self.url, response_time_ms, self.reliability_score);
    }

    /// Calcula el score de confiabilidad basado en histórico
    fn update_reliability_score(&mut self) {
        let total_requests = self.total_successes + self.total_failures;
        if total_requests == 0 {
            self.reliability_score = 1.0;
            return;
        }

        // Score base: ratio de éxito
        let success_ratio = self.total_successes as f64 / total_requests as f64;
        
        // Penaliza fallos consecutivos recientes
        let consecutive_penalty = if self.consecutive_failures > 0 {
            1.0 - (self.consecutive_failures as f64 * 0.1).min(0.5)
        } else {
            1.0
        };
        
        // Penaliza ciertos tipos de errores más que otros
        let error_type_penalty = self.calculate_error_type_penalty();
        
        // Penaliza respuestas lentas
        let speed_bonus = if self.average_response_time_ms < 500 {
            1.0
        } else if self.average_response_time_ms < 1000 {
            0.9
        } else {
            0.8
        };

        self.reliability_score = (success_ratio * consecutive_penalty * error_type_penalty * speed_bonus)
            .max(0.0)
            .min(1.0);
    }

    /// Calcula penalidad basada en tipos de errores
    fn calculate_error_type_penalty(&self) -> f64 {
        let mut penalty = 1.0;
        
        for (error_type, count) in &self.failure_types {
            let error_penalty = match error_type.as_str() {
                "410 Gone" => 0.7,      // Severo: rate limiting
                "timeout" => 0.8,       // Moderado: puede ser temporal
                "401 Unauthorized" => 0.6, // Severo: necesita API key
                "403 Forbidden" => 0.6,  // Severo: acceso denegado
                "dns error" => 0.5,     // Muy severo: endpoint no existe
                "connection refused" => 0.5, // Muy severo: endpoint caído
                _ => 0.9,               // Error genérico
            };
            
            // Aplica penalidad proporcional a la frecuencia
            let frequency_factor = (*count as f64 / (self.total_failures + 1) as f64).min(1.0);
            penalty *= 1.0 - (1.0 - error_penalty) * frequency_factor;
        }
        
        penalty.max(0.1) // Mínimo 10% de score
    }

    /// Verifica si el endpoint debería evitarse basado en historial
    pub fn should_avoid_endpoint(&self, hours_to_consider: u64) -> bool {
        // Si el score es muy bajo, evitarlo
        if self.reliability_score < 0.3 {
            return true;
        }

        // Si hay muchos fallos consecutivos recientes, evitarlo
        if self.consecutive_failures >= 5 {
            return true;
        }

        // Si ha fallado recientemente con errores severos, evitarlo temporalmente
        if let Some(last_failure) = self.last_failure_time {
            let hours_since_failure = Utc::now().signed_duration_since(last_failure).num_hours();
            if hours_since_failure < hours_to_consider as i64 {
                // Verifica si el último fallo fue severo
                if self.has_severe_recent_errors() {
                    return true;
                }
            }
        }

        false
    }

    /// Verifica si hay errores severos recientes
    fn has_severe_recent_errors(&self) -> bool {
        for error_type in self.failure_types.keys() {
            match error_type.as_str() {
                "410 Gone" | "401 Unauthorized" | "403 Forbidden" | 
                "dns error" | "connection refused" => return true,
                _ => {}
            }
        }
        false
    }
}

/// Gestor de persistencia de salud RPC
#[derive(Debug)]
pub struct RpcHealthPersistence {
    file_path: String,
    endpoints: HashMap<String, PersistedRpcHealth>,
}

impl RpcHealthPersistence {
    /// Crea una nueva instancia del gestor de persistencia
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
            endpoints: HashMap::new(),
        }
    }

    /// Carga el estado persistido desde disco
    pub async fn load(&mut self) -> Result<()> {
        if !Path::new(&self.file_path).exists() {
            info!("📂 RPC health file not found, starting with clean state");
            return Ok(());
        }

        let content = fs::read_to_string(&self.file_path).await?;
        self.endpoints = serde_json::from_str(&content)?;
        
        info!("📂 Loaded RPC health data for {} endpoints", self.endpoints.len());
        
        // Log endpoints que deberían evitarse
        let problematic_endpoints: Vec<_> = self.endpoints.iter()
            .filter(|(_, health)| health.should_avoid_endpoint(24))
            .collect();
            
        if !problematic_endpoints.is_empty() {
            warn!("⚠️ Found {} problematic RPC endpoints:", problematic_endpoints.len());
            for (url, health) in problematic_endpoints {
                warn!("   ❌ {} (score: {:.2}, consecutive failures: {})", 
                      url, health.reliability_score, health.consecutive_failures);
            }
        }

        Ok(())
    }

    /// Guarda el estado actual en disco
    pub async fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.endpoints)?;
        
        // Crea el directorio si no existe
        if let Some(parent) = Path::new(&self.file_path).parent() {
            fs::create_dir_all(parent).await?;
        }
        
        fs::write(&self.file_path, content).await?;
        
        debug!("💾 Saved RPC health data for {} endpoints", self.endpoints.len());
        Ok(())
    }

    /// Registra un fallo de endpoint
    pub async fn record_endpoint_failure(&mut self, url: &str, error_type: &str) -> Result<()> {
        let health = self.endpoints.entry(url.to_string())
            .or_insert_with(|| PersistedRpcHealth::new(url.to_string()));
        
        health.record_failure(error_type);
        
        // Guarda inmediatamente para no perder datos
        self.save().await?;
        
        Ok(())
    }

    /// Registra un éxito de endpoint
    pub async fn record_endpoint_success(&mut self, url: &str, response_time_ms: u64) -> Result<()> {
        let health = self.endpoints.entry(url.to_string())
            .or_insert_with(|| PersistedRpcHealth::new(url.to_string()));
        
        health.record_success(response_time_ms);
        
        // Guarda cada cierto número de éxitos para no saturar el disco
        if health.total_successes % 10 == 0 {
            self.save().await?;
        }
        
        Ok(())
    }

    /// Obtiene el estado de salud de un endpoint
    pub fn get_endpoint_health(&self, url: &str) -> Option<&PersistedRpcHealth> {
        self.endpoints.get(url)
    }

    /// Obtiene endpoints que deberían evitarse
    pub fn get_problematic_endpoints(&self, hours_to_consider: u64) -> Vec<String> {
        self.endpoints.iter()
            .filter(|(_, health)| health.should_avoid_endpoint(hours_to_consider))
            .map(|(url, _)| url.clone())
            .collect()
    }

    /// Obtiene endpoints recomendados ordenados por confiabilidad
    pub fn get_recommended_endpoints(&self) -> Vec<(String, f64)> {
        let mut endpoints: Vec<_> = self.endpoints.iter()
            .filter(|(_, health)| !health.should_avoid_endpoint(24))
            .map(|(url, health)| (url.clone(), health.reliability_score))
            .collect();
        
        // Ordena por score de confiabilidad (mayor primero)
        endpoints.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        endpoints
    }

    /// Genera un reporte de salud de todos los endpoints
    pub fn generate_health_report(&self) -> String {
        let mut report = String::from("🏥 RPC Endpoints Health Report\n");
        report.push_str("=====================================\n\n");

        if self.endpoints.is_empty() {
            report.push_str("No RPC health data available.\n");
            return report;
        }

        // Estadísticas generales
        let total_endpoints = self.endpoints.len();
        let healthy_endpoints = self.endpoints.iter()
            .filter(|(_, health)| !health.should_avoid_endpoint(24))
            .count();
        
        report.push_str(&format!("📊 Overview: {}/{} endpoints healthy\n\n", 
                                healthy_endpoints, total_endpoints));

        // Detalles por endpoint
        let mut sorted_endpoints: Vec<_> = self.endpoints.iter().collect();
        sorted_endpoints.sort_by(|a, b| b.1.reliability_score.partial_cmp(&a.1.reliability_score)
            .unwrap_or(std::cmp::Ordering::Equal));

        for (url, health) in sorted_endpoints {
            let status = if health.should_avoid_endpoint(24) { "❌ AVOID" } else { "✅ OK" };
            let total_requests = health.total_successes + health.total_failures;
            
            report.push_str(&format!(
                "{} {} (Score: {:.2})\n   Requests: {} | Success Rate: {:.1}% | Avg: {}ms | Consecutive Fails: {}\n",
                status,
                url,
                health.reliability_score,
                total_requests,
                if total_requests > 0 { health.total_successes as f64 / total_requests as f64 * 100.0 } else { 0.0 },
                health.average_response_time_ms,
                health.consecutive_failures
            ));

            if !health.failure_types.is_empty() {
                report.push_str("   Error Types: ");
                for (error_type, count) in &health.failure_types {
                    report.push_str(&format!("{}({}), ", error_type, count));
                }
                report.push_str("\n");
            }
            report.push_str("\n");
        }

        report
    }
}
