//! Configuración de credenciales de APIs para el sistema SniperForge
//! Centraliza todas las credenciales y URLs de servicios externos
//! TODAS LAS CREDENCIALES SE CARGAN DESDE config.json - NO HAY HARDCODING

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

/// Estructura para deserializar el archivo config.json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigFile {
    pub api_credentials: ApiCredentialsConfig,
    pub rate_limits: HashMap<String, u64>,
    pub timeouts: HashMap<String, u64>,
    pub fallback_prices: HashMap<String, f64>,
    pub websocket: WebSocketConfigData,
    pub trading: TradingConfigData,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiCredentialsConfig {
    pub helius: HeliusConfig,
    pub jupiter: JupiterConfig,
    pub dexscreener: DexScreenerConfig,
    pub pyth: PythConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HeliusConfig {
    pub api_key: String,
    pub mainnet_url: String,
    pub websocket_url: String,
    pub eclipse_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JupiterConfig {
    pub api_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DexScreenerConfig {
    pub api_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PythConfig {
    pub api_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebSocketConfigData {
    pub reconnect_interval_ms: u64,
    pub max_reconnect_attempts: u32,
    pub message_buffer_size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradingConfigData {
    pub max_history_size: usize,
    pub bridge_fee_percentage: f64,
    pub min_confidence_score: f64,
    pub max_risk_score: f64,
    pub optimal_trade_percentage: f64,
    pub base_market_volatility: f64,
}

/// Credenciales y configuración de APIs - CARGA DESDE config.json
#[derive(Debug, Clone)]
pub struct ApiCredentials {
    /// Credenciales de Helius RPC
    pub helius_api_key: String,
    pub helius_mainnet_url: String,
    pub helius_websocket_url: String,
    pub helius_eclipse_url: String,
    
    /// URLs de otros proveedores
    pub jupiter_api_url: String,
    pub dexscreener_api_url: String,
    pub pyth_api_url: String,
    
    /// Rate limiting configuraciones
    pub rate_limits: HashMap<String, u64>, // ms entre requests
    
    /// Timeouts por proveedor
    pub timeouts: HashMap<String, u64>, // segundos
    
    /// Precios de fallback
    pub fallback_prices: HashMap<String, f64>,
    
    /// Configuración de trading
    pub trading_config: TradingConfigData,
}

impl Default for ApiCredentials {
    fn default() -> Self {
        // Intentar cargar desde config.json, usar valores por defecto si falla
        Self::load_from_file("config.json").unwrap_or_else(|e| {
            eprintln!("⚠️ Error cargando config.json: {}", e);
            eprintln!("📁 Usando configuración por defecto (NO RECOMENDADO para producción)");
            Self::create_default_fallback()
        })
    }
}

impl ApiCredentials {
    /// Cargar configuración desde archivo JSON
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_content = fs::read_to_string(&path)
            .with_context(|| format!("No se pudo leer el archivo de configuración: {:?}", path.as_ref()))?;
        
        let config: ConfigFile = serde_json::from_str(&config_content)
            .with_context(|| "Error parseando config.json - verificar sintaxis JSON")?;
        
        println!("✅ Configuración cargada exitosamente desde: {:?}", path.as_ref());
        
        Ok(Self {
            helius_api_key: config.api_credentials.helius.api_key,
            helius_mainnet_url: config.api_credentials.helius.mainnet_url,
            helius_websocket_url: config.api_credentials.helius.websocket_url,
            helius_eclipse_url: config.api_credentials.helius.eclipse_url,
            
            jupiter_api_url: config.api_credentials.jupiter.api_url,
            dexscreener_api_url: config.api_credentials.dexscreener.api_url,
            pyth_api_url: config.api_credentials.pyth.api_url,
            
            rate_limits: config.rate_limits,
            timeouts: config.timeouts,
            fallback_prices: config.fallback_prices,
            trading_config: config.trading,
        })
    }
    
    /// Crear configuración por defecto como fallback (SIN CREDENCIALES REALES)
    fn create_default_fallback() -> Self {
        let mut rate_limits = HashMap::new();
        rate_limits.insert("helius".to_string(), 100);
        rate_limits.insert("jupiter".to_string(), 2000);
        rate_limits.insert("dexscreener".to_string(), 500);
        rate_limits.insert("pyth".to_string(), 300);

        let mut timeouts = HashMap::new();
        timeouts.insert("helius".to_string(), 15);
        timeouts.insert("jupiter".to_string(), 10);
        timeouts.insert("dexscreener".to_string(), 12);
        timeouts.insert("pyth".to_string(), 10);
        
        let mut fallback_prices = HashMap::new();
        fallback_prices.insert("SOL".to_string(), 180.0);
        fallback_prices.insert("ETH".to_string(), 3200.0);
        fallback_prices.insert("USDC".to_string(), 1.0);
        fallback_prices.insert("USDT".to_string(), 1.0);

        Self {
            helius_api_key: "CONFIGURAR_EN_CONFIG_JSON".to_string(),
            helius_mainnet_url: "https://mainnet.helius-rpc.com".to_string(),
            helius_websocket_url: "wss://mainnet.helius-rpc.com".to_string(),
            helius_eclipse_url: "https://eclipse.helius-rpc.com".to_string(),
            
            jupiter_api_url: "https://quote-api.jup.ag/v6".to_string(),
            dexscreener_api_url: "https://api.dexscreener.com/latest".to_string(),
            pyth_api_url: "https://hermes.pyth.network/api".to_string(),
            
            rate_limits,
            timeouts,
            fallback_prices,
            trading_config: TradingConfigData {
                max_history_size: 1000,
                bridge_fee_percentage: 0.003,
                min_confidence_score: 0.6,
                max_risk_score: 0.8,
                optimal_trade_percentage: 0.25,
                base_market_volatility: 0.15,
            },
        }
    }
    /// Obtener precio de fallback desde configuración
    pub fn get_fallback_price(&self, token: &str) -> f64 {
        self.fallback_prices.get(token).copied().unwrap_or(1.0)
    }
    
    /// Obtener configuración de trading
    pub fn get_trading_config(&self) -> &TradingConfigData {
        &self.trading_config
    }
    
    /// Obtener URL completa de Helius con API key
    pub fn get_helius_url(&self) -> String {
        format!("{}/?api-key={}", self.helius_mainnet_url, self.helius_api_key)
    }
    
    /// Obtener URL completa de WebSocket de Helius con API key
    pub fn get_helius_websocket_url(&self) -> String {
        format!("{}/?api-key={}", self.helius_websocket_url, self.helius_api_key)
    }
    
    /// Obtener rate limit para un proveedor específico
    pub fn get_rate_limit(&self, provider: &str) -> u64 {
        self.rate_limits.get(provider).copied().unwrap_or(1000)
    }
    
    /// Obtener timeout para un proveedor específico
    pub fn get_timeout(&self, provider: &str) -> u64 {
        self.timeouts.get(provider).copied().unwrap_or(10)
    }
    
    /// Verificar si las credenciales de Helius están configuradas
    pub fn has_helius_credentials(&self) -> bool {
        !self.helius_api_key.is_empty() && 
        self.helius_api_key != "YOUR_API_KEY" && 
        self.helius_api_key != "CONFIGURAR_EN_CONFIG_JSON"
    }
    
    /// Obtener información de configuración para logging
    pub fn get_config_summary(&self) -> String {
        format!(
            "Helius: {} | Jupiter: {} | DexScreener: {} | Pyth: {} | Fallback Prices: {} | Rate Limits: {:?}",
            if self.has_helius_credentials() { "✅ CONFIGURADO" } else { "❌ FALTA API KEY" },
            if !self.jupiter_api_url.is_empty() { "✅" } else { "❌" },
            if !self.dexscreener_api_url.is_empty() { "✅" } else { "❌" },
            if !self.pyth_api_url.is_empty() { "✅" } else { "❌" },
            self.fallback_prices.len(),
            self.rate_limits
        )
    }
    
    /// Recargar configuración desde archivo (útil para actualizar sin reiniciar)
    pub fn reload_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let new_config = Self::load_from_file(path)?;
        *self = new_config;
        println!("🔄 Configuración recargada exitosamente");
        Ok(())
    }
}

/// Configuración de WebSocket para streaming de datos en tiempo real
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// URL del WebSocket de Helius
    pub helius_ws_url: String,
    /// Configuración de reconexión
    pub reconnect_interval_ms: u64,
    pub max_reconnect_attempts: u32,
    /// Buffer de mensajes
    pub message_buffer_size: usize,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        let credentials = ApiCredentials::default();
        
        Self {
            helius_ws_url: credentials.get_helius_websocket_url(),
            reconnect_interval_ms: 5000, // 5 segundos por defecto
            max_reconnect_attempts: 10,
            message_buffer_size: 1000,
        }
    }
}

impl WebSocketConfig {
    /// Crear desde archivo de configuración
    pub fn from_config_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_content = fs::read_to_string(&path)?;
        let config: ConfigFile = serde_json::from_str(&config_content)?;
        let credentials = ApiCredentials::load_from_file(&path)?;
        
        Ok(Self {
            helius_ws_url: credentials.get_helius_websocket_url(),
            reconnect_interval_ms: config.websocket.reconnect_interval_ms,
            max_reconnect_attempts: config.websocket.max_reconnect_attempts,
            message_buffer_size: config.websocket.message_buffer_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_credentials_default() {
        let creds = ApiCredentials::default();
        
        // Verificar que las credenciales están configuradas
        assert!(creds.has_helius_credentials());
        assert!(!creds.helius_api_key.is_empty());
        assert!(creds.helius_mainnet_url.starts_with("https://"));
        
        // Verificar URLs
        let helius_url = creds.get_helius_url();
        assert!(helius_url.contains("api-key="));
        assert!(helius_url.contains(&creds.helius_api_key));
        
        // Verificar rate limits
        assert!(creds.get_rate_limit("helius") > 0);
        assert!(creds.get_rate_limit("jupiter") > 0);
        
        println!("✅ Configuración de APIs: {}", creds.get_config_summary());
    }
    
    #[test]
    fn test_websocket_config() {
        let ws_config = WebSocketConfig::default();
        
        assert!(ws_config.helius_ws_url.starts_with("wss://"));
        assert!(ws_config.helius_ws_url.contains("api-key="));
        assert!(ws_config.reconnect_interval_ms > 0);
        assert!(ws_config.max_reconnect_attempts > 0);
        
        println!("✅ WebSocket configurado: {}", ws_config.helius_ws_url);
    }
}
