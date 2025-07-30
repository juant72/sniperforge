// ================================================================================
// UTILS - Utilidades Comunes
// ================================================================================

use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use anyhow::Result;
use tracing::{debug, info, warn, error};
use serde::{Serialize, Deserialize};

use crate::{CoreResult, CoreError};

/// Utilidades para formateo y display
pub struct DisplayUtils;

/// Utilidades para cálculos matemáticos
pub struct MathUtils;

/// Utilidades para timing y delays
pub struct TimingUtils;

/// Utilidades para validación
pub struct ValidationUtils;

/// Utilidades para logging estructurado
pub struct LogUtils;

/// Configuración de rate limiting
#[derive(Debug, Clone)]
pub struct RateLimiter {
    max_requests: u32,
    window_duration: Duration,
    requests: Vec<Instant>,
}

/// Utilidades para cache simple
#[derive(Debug, Clone)]
pub struct SimpleCache<T> {
    data: HashMap<String, CacheEntry<T>>,
    max_size: usize,
    default_ttl: Duration,
}

#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    created_at: Instant,
    ttl: Duration,
    access_count: u32,
}

/// Métricas de rendimiento
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub start_time: Option<u64>,
    pub operations_count: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub avg_response_time_ms: f64,
    pub min_response_time_ms: f64,
    pub max_response_time_ms: f64,
    pub total_processing_time_ms: f64,
}

/// Configuración de retry
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub jitter_enabled: bool,
}

impl DisplayUtils {
    /// Formatear cantidad de SOL
    pub fn format_sol(amount: f64) -> String {
        if amount >= 1_000_000.0 {
            format!("{:.2}M SOL", amount / 1_000_000.0)
        } else if amount >= 1_000.0 {
            format!("{:.2}K SOL", amount / 1_000.0)
        } else if amount >= 1.0 {
            format!("{:.4} SOL", amount)
        } else {
            format!("{:.6} SOL", amount)
        }
    }
    
    /// Formatear precio USD
    pub fn format_usd(amount: f64) -> String {
        if amount >= 1_000_000.0 {
            format!("${:.2}M", amount / 1_000_000.0)
        } else if amount >= 1_000.0 {
            format!("${:.2}K", amount / 1_000.0)
        } else if amount >= 1.0 {
            format!("${:.2}", amount)
        } else {
            format!("${:.4}", amount)
        }
    }
    
    /// Formatear porcentaje
    pub fn format_percentage(value: f64) -> String {
        if value.abs() >= 100.0 {
            format!("{:+.0}%", value)
        } else if value.abs() >= 10.0 {
            format!("{:+.1}%", value)
        } else {
            format!("{:+.2}%", value)
        }
    }
    
    /// Formatear duración
    pub fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        let millis = duration.subsec_millis();
        
        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else if seconds > 0 {
            format!("{}.{:03}s", seconds, millis)
        } else {
            format!("{}ms", millis)
        }
    }
    
    /// Formatear número compacto
    pub fn format_compact_number(num: f64) -> String {
        let abs_num = num.abs();
        
        if abs_num >= 1_000_000_000.0 {
            format!("{:.1}B", num / 1_000_000_000.0)
        } else if abs_num >= 1_000_000.0 {
            format!("{:.1}M", num / 1_000_000.0)
        } else if abs_num >= 1_000.0 {
            format!("{:.1}K", num / 1_000.0)
        } else if abs_num >= 1.0 {
            format!("{:.2}", num)
        } else {
            format!("{:.4}", num)
        }
    }
    
    /// Formatear tabla de datos
    pub fn format_table(headers: &[&str], rows: &[Vec<String>]) -> String {
        if headers.is_empty() || rows.is_empty() {
            return String::new();
        }
        
        // Calcular anchos de columna
        let mut col_widths = headers.iter().map(|h| h.len()).collect::<Vec<_>>();
        
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    col_widths[i] = col_widths[i].max(cell.len());
                }
            }
        }
        
        let mut result = String::new();
        
        // Header
        for (i, header) in headers.iter().enumerate() {
            if i > 0 { result.push_str(" | "); }
            result.push_str(&format!("{:width$}", header, width = col_widths[i]));
        }
        result.push('\n');
        
        // Separator
        for (i, &width) in col_widths.iter().enumerate() {
            if i > 0 { result.push_str("-+-"); }
            result.push_str(&"-".repeat(width));
        }
        result.push('\n');
        
        // Rows
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i > 0 { result.push_str(" | "); }
                let width = col_widths.get(i).unwrap_or(&10);
                result.push_str(&format!("{:width$}", cell, width = width));
            }
            result.push('\n');
        }
        
        result
    }
}

impl MathUtils {
    /// Calcular diferencia porcentual
    pub fn percentage_diff(old_value: f64, new_value: f64) -> f64 {
        if old_value == 0.0 {
            return if new_value == 0.0 { 0.0 } else { 100.0 };
        }
        ((new_value - old_value) / old_value) * 100.0
    }
    
    /// Calcular promedio ponderado
    pub fn weighted_average(values: &[(f64, f64)]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        
        let mut sum_weighted = 0.0;
        let mut sum_weights = 0.0;
        
        for &(value, weight) in values {
            sum_weighted += value * weight;
            sum_weights += weight;
        }
        
        if sum_weights == 0.0 {
            values.iter().map(|(v, _)| v).sum::<f64>() / values.len() as f64
        } else {
            sum_weighted / sum_weights
        }
    }
    
    /// Calcular volatilidad (desviación estándar)
    pub fn calculate_volatility(prices: &[f64]) -> f64 {
        if prices.len() < 2 {
            return 0.0;
        }
        
        let mean = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance = prices.iter()
            .map(|price| (price - mean).powi(2))
            .sum::<f64>() / (prices.len() - 1) as f64;
        
        variance.sqrt()
    }
    
    /// Calcular EMA (Exponential Moving Average)
    pub fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
        if prices.is_empty() || period == 0 {
            return Vec::new();
        }
        
        let alpha = 2.0 / (period + 1) as f64;
        let mut ema_values = Vec::with_capacity(prices.len());
        
        // Primer valor es el precio mismo
        ema_values.push(prices[0]);
        
        // Calcular EMA para el resto
        for &price in &prices[1..] {
            let last_ema = ema_values.last().unwrap();
            let ema = alpha * price + (1.0 - alpha) * last_ema;
            ema_values.push(ema);
        }
        
        ema_values
    }
    
    /// Normalizar valor a rango 0-1
    pub fn normalize(value: f64, min: f64, max: f64) -> f64 {
        if max == min {
            return 0.5;
        }
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    }
    
    /// Redondear a decimales específicos
    pub fn round_to_decimals(value: f64, decimals: u32) -> f64 {
        let multiplier = 10_f64.powi(decimals as i32);
        (value * multiplier).round() / multiplier
    }
    
    /// Calcular slippage
    pub fn calculate_slippage(expected_price: f64, actual_price: f64) -> f64 {
        if expected_price == 0.0 {
            return 0.0;
        }
        ((actual_price - expected_price) / expected_price).abs() * 100.0
    }
    
    /// Interpolar linealmente
    pub fn linear_interpolate(x: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        if x2 == x1 {
            return y1;
        }
        y1 + (x - x1) * (y2 - y1) / (x2 - x1)
    }
}

impl TimingUtils {
    /// Crear delay adaptativo basado en el éxito
    pub fn adaptive_delay(base_ms: u64, success_rate: f64) -> Duration {
        let multiplier = if success_rate > 0.9 {
            0.5  // Reducir delay si alta tasa de éxito
        } else if success_rate > 0.7 {
            1.0  // Mantener delay normal
        } else if success_rate > 0.5 {
            1.5  // Aumentar ligeramente
        } else {
            2.0  // Duplicar delay si baja tasa de éxito
        };
        
        Duration::from_millis((base_ms as f64 * multiplier) as u64)
    }
    
    /// Crear delay con jitter
    pub fn jittered_delay(base_ms: u64, jitter_percent: f64) -> Duration {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(-jitter_percent..jitter_percent);
        let adjusted_ms = base_ms as f64 * (1.0 + jitter / 100.0);
        Duration::from_millis(adjusted_ms.max(0.0) as u64)
    }
    
    /// Timeout escalado
    pub fn scaled_timeout(base_timeout: Duration, attempt: u32, max_attempts: u32) -> Duration {
        let scale_factor = 1.0 + (attempt as f64 / max_attempts as f64);
        Duration::from_millis((base_timeout.as_millis() as f64 * scale_factor) as u64)
    }
    
    /// Obtener timestamp Unix
    pub fn unix_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
    
    /// Obtener timestamp Unix en milisegundos
    pub fn unix_timestamp_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

impl ValidationUtils {
    /// Validar dirección de wallet Solana
    pub fn is_valid_solana_address(address: &str) -> bool {
        // Verificación básica de formato
        if address.len() < 32 || address.len() > 44 {
            return false;
        }
        
        // Verificar que solo contenga caracteres base58
        address.chars().all(|c| {
            matches!(c, '1'..='9' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | 'a'..='k' | 'm'..='z')
        })
    }
    
    /// Validar cantidad de tokens
    pub fn is_valid_token_amount(amount: f64, min_amount: f64, max_amount: f64) -> bool {
        amount >= min_amount && amount <= max_amount && amount.is_finite()
    }
    
    /// Validar porcentaje de slippage
    pub fn is_valid_slippage(slippage: f64) -> bool {
        slippage >= 0.0 && slippage <= 100.0 && slippage.is_finite()
    }
    
    /// Validar URL
    pub fn is_valid_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }
    
    /// Validar que un número esté en rango
    pub fn is_in_range(value: f64, min: f64, max: f64) -> bool {
        value >= min && value <= max && value.is_finite()
    }
    
    /// Validar formato de email básico
    pub fn is_valid_email(email: &str) -> bool {
        email.contains('@') && email.contains('.') && email.len() > 5
    }
}

impl LogUtils {
    /// Log de oportunidad de arbitraje
    pub fn log_arbitrage_opportunity(
        symbol: &str,
        buy_dex: &str,
        sell_dex: &str,
        profit_percent: f64,
        profit_usd: f64,
    ) {
        info!(
            "🎯 OPORTUNIDAD: {} | {}→{} | Profit: {}% (${:.2})",
            symbol,
            buy_dex,
            sell_dex,
            DisplayUtils::format_percentage(profit_percent),
            profit_usd
        );
    }
    
    /// Log de trade ejecutado
    pub fn log_trade_execution(
        symbol: &str,
        side: &str,
        amount: f64,
        price: f64,
        dex: &str,
        status: &str,
    ) {
        match status {
            "success" => {
                info!(
                    "✅ TRADE: {} {} {:.4} @ {:.6} on {} - SUCCESS",
                    side.to_uppercase(),
                    symbol,
                    amount,
                    price,
                    dex
                );
            },
            "failed" => {
                error!(
                    "❌ TRADE: {} {} {:.4} @ {:.6} on {} - FAILED",
                    side.to_uppercase(),
                    symbol,
                    amount,
                    price,
                    dex
                );
            },
            _ => {
                warn!(
                    "⏳ TRADE: {} {} {:.4} @ {:.6} on {} - {}",
                    side.to_uppercase(),
                    symbol,
                    amount,
                    price,
                    dex,
                    status.to_uppercase()
                );
            }
        }
    }
    
    /// Log de performance metrics
    pub fn log_performance_summary(metrics: &PerformanceMetrics) {
        let success_rate = if metrics.operations_count > 0 {
            (metrics.success_count as f64 / metrics.operations_count as f64) * 100.0
        } else {
            0.0
        };
        
        info!(
            "📊 PERFORMANCE: {} ops | {:.1}% success | Avg: {:.1}ms | Min: {:.1}ms | Max: {:.1}ms",
            metrics.operations_count,
            success_rate,
            metrics.avg_response_time_ms,
            metrics.min_response_time_ms,
            metrics.max_response_time_ms
        );
    }
    
    /// Log de balance
    pub fn log_balance_update(token: &str, old_balance: f64, new_balance: f64) {
        let diff = new_balance - old_balance;
        let symbol = if diff > 0.0 { "📈" } else if diff < 0.0 { "📉" } else { "➡️" };
        
        info!(
            "{} BALANCE: {} | {:.6} → {:.6} ({:+.6})",
            symbol,
            token,
            old_balance,
            new_balance,
            diff
        );
    }
}

impl RateLimiter {
    /// Crear nuevo rate limiter
    pub fn new(max_requests: u32, window_duration: Duration) -> Self {
        Self {
            max_requests,
            window_duration,
            requests: Vec::new(),
        }
    }
    
    /// Verificar si se puede hacer una nueva request
    pub fn can_proceed(&mut self) -> bool {
        let now = Instant::now();
        
        // Limpiar requests antiguas
        self.requests.retain(|&request_time| {
            now.duration_since(request_time) < self.window_duration
        });
        
        // Verificar límite
        if self.requests.len() < self.max_requests as usize {
            self.requests.push(now);
            true
        } else {
            false
        }
    }
    
    /// Obtener tiempo hasta que se pueda hacer la siguiente request
    pub fn time_until_next_request(&self) -> Option<Duration> {
        if self.requests.len() < self.max_requests as usize {
            return None;
        }
        
        let oldest_request = self.requests.first()?;
        let elapsed = oldest_request.elapsed();
        
        if elapsed < self.window_duration {
            Some(self.window_duration - elapsed)
        } else {
            None
        }
    }
}

impl<T: Clone> SimpleCache<T> {
    /// Crear nuevo cache
    pub fn new(max_size: usize, default_ttl: Duration) -> Self {
        Self {
            data: HashMap::new(),
            max_size,
            default_ttl,
        }
    }
    
    /// Insertar en cache
    pub fn insert(&mut self, key: String, value: T) {
        self.insert_with_ttl(key, value, self.default_ttl);
    }
    
    /// Insertar con TTL específico
    pub fn insert_with_ttl(&mut self, key: String, value: T, ttl: Duration) {
        // Si está lleno, remover el más antiguo
        if self.data.len() >= self.max_size {
            self.evict_oldest();
        }
        
        self.data.insert(key, CacheEntry {
            value,
            created_at: Instant::now(),
            ttl,
            access_count: 0,
        });
    }
    
    /// Obtener del cache
    pub fn get(&mut self, key: &str) -> Option<T> {
        if let Some(entry) = self.data.get_mut(key) {
            if entry.created_at.elapsed() < entry.ttl {
                entry.access_count += 1;
                Some(entry.value.clone())
            } else {
                self.data.remove(key);
                None
            }
        } else {
            None
        }
    }
    
    /// Limpiar entradas expiradas
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        self.data.retain(|_, entry| {
            now.duration_since(entry.created_at) < entry.ttl
        });
    }
    
    /// Remover el más antiguo
    fn evict_oldest(&mut self) {
        if let Some((oldest_key, _)) = self.data.iter()
            .min_by_key(|(_, entry)| entry.created_at)
            .map(|(k, v)| (k.clone(), v.created_at)) {
            self.data.remove(&oldest_key);
        }
    }
    
    /// Obtener estadísticas del cache
    pub fn stats(&self) -> (usize, usize, u32) {
        let total_accesses = self.data.values().map(|e| e.access_count).sum();
        (self.data.len(), self.max_size, total_accesses)
    }
}

impl PerformanceMetrics {
    /// Crear nuevas métricas
    pub fn new() -> Self {
        Self {
            start_time: Some(TimingUtils::unix_timestamp()),
            ..Default::default()
        }
    }
    
    /// Registrar operación exitosa
    pub fn record_success(&mut self, response_time_ms: f64) {
        self.operations_count += 1;
        self.success_count += 1;
        self.update_response_times(response_time_ms);
    }
    
    /// Registrar operación fallida
    pub fn record_error(&mut self, response_time_ms: f64) {
        self.operations_count += 1;
        self.error_count += 1;
        self.update_response_times(response_time_ms);
    }
    
    /// Actualizar tiempos de respuesta
    fn update_response_times(&mut self, response_time_ms: f64) {
        if self.operations_count == 1 {
            self.min_response_time_ms = response_time_ms;
            self.max_response_time_ms = response_time_ms;
            self.avg_response_time_ms = response_time_ms;
        } else {
            self.min_response_time_ms = self.min_response_time_ms.min(response_time_ms);
            self.max_response_time_ms = self.max_response_time_ms.max(response_time_ms);
            
            // Actualizar promedio
            let total_ops = self.operations_count as f64;
            self.avg_response_time_ms = 
                (self.avg_response_time_ms * (total_ops - 1.0) + response_time_ms) / total_ops;
        }
        
        self.total_processing_time_ms += response_time_ms;
    }
    
    /// Obtener tasa de éxito
    pub fn success_rate(&self) -> f64 {
        if self.operations_count == 0 {
            0.0
        } else {
            (self.success_count as f64 / self.operations_count as f64) * 100.0
        }
    }
    
    /// Obtener uptime en segundos
    pub fn uptime_seconds(&self) -> u64 {
        if let Some(start_time) = self.start_time {
            TimingUtils::unix_timestamp() - start_time
        } else {
            0
        }
    }
}

impl RetryConfig {
    /// Configuración por defecto
    pub fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            jitter_enabled: true,
        }
    }
    
    /// Configuración agresiva (más intentos, menos delay)
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            base_delay_ms: 500,
            max_delay_ms: 10000,
            backoff_multiplier: 1.5,
            jitter_enabled: true,
        }
    }
    
    /// Configuración conservadora (menos intentos, más delay)
    pub fn conservative() -> Self {
        Self {
            max_attempts: 2,
            base_delay_ms: 2000,
            max_delay_ms: 60000,
            backoff_multiplier: 3.0,
            jitter_enabled: false,
        }
    }
    
    /// Calcular delay para intento específico
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.base_delay_ms as f64;
        let multiplier = self.backoff_multiplier.powi(attempt as i32 - 1);
        let delay_ms = (base_delay * multiplier).min(self.max_delay_ms as f64);
        
        if self.jitter_enabled {
            TimingUtils::jittered_delay(delay_ms as u64, 10.0)
        } else {
            Duration::from_millis(delay_ms as u64)
        }
    }
}

/// Función helper para retry con backoff exponencial
pub async fn retry_with_backoff<F, Fut, T, E>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut last_error = None;
    
    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                debug!("Attempt {} failed: {:?}", attempt, error);
                last_error = Some(error);
                
                if attempt < config.max_attempts {
                    let delay = config.calculate_delay(attempt);
                    debug!("Retrying in {:?}", delay);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_sol() {
        assert_eq!(DisplayUtils::format_sol(1_500_000.0), "1.50M SOL");
        assert_eq!(DisplayUtils::format_sol(1_500.0), "1.50K SOL");
        assert_eq!(DisplayUtils::format_sol(1.5), "1.5000 SOL");
        assert_eq!(DisplayUtils::format_sol(0.000123), "0.000123 SOL");
    }
    
    #[test]
    fn test_percentage_diff() {
        assert_eq!(MathUtils::percentage_diff(100.0, 110.0), 10.0);
        assert_eq!(MathUtils::percentage_diff(100.0, 90.0), -10.0);
        assert_eq!(MathUtils::percentage_diff(0.0, 10.0), 100.0);
    }
    
    #[test]
    fn test_weighted_average() {
        let values = vec![(10.0, 1.0), (20.0, 2.0), (30.0, 1.0)];
        let avg = MathUtils::weighted_average(&values);
        assert!((avg - 20.0).abs() < 0.001);
    }
    
    #[test]
    fn test_validation() {
        assert!(ValidationUtils::is_valid_slippage(5.0));
        assert!(!ValidationUtils::is_valid_slippage(-1.0));
        assert!(!ValidationUtils::is_valid_slippage(101.0));
        
        assert!(ValidationUtils::is_valid_url("https://example.com"));
        assert!(!ValidationUtils::is_valid_url("not-a-url"));
    }
    
    #[tokio::test]
    async fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2, Duration::from_secs(1));
        
        assert!(limiter.can_proceed());
        assert!(limiter.can_proceed());
        assert!(!limiter.can_proceed()); // Debe fallar en el tercero
    }
    
    #[test]
    fn test_simple_cache() {
        let mut cache = SimpleCache::new(2, Duration::from_secs(1));
        
        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());
        
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        assert_eq!(cache.get("key2"), Some("value2".to_string()));
        
        // Insertar un tercero debe evict el primero
        cache.insert("key3".to_string(), "value3".to_string());
        assert_eq!(cache.get("key1"), None);
    }
}
