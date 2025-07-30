// ================================================================================
// CORE CONFIG - SISTEMA DE CONFIGURACIÓN MODULAR
// ================================================================================
// Manejo centralizado de configuraciones para todos los bots
// Soporte para JSON, TOML, y variables de entorno
// ================================================================================

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fs;
use std::path::Path;
use std::collections::HashMap;
use tracing::{info, warn, error, debug};

use crate::{CoreResult, CoreError, Environment};

/// Error específico de configuración
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Environment variable error: {0}")]
    EnvError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Manager de configuración central
pub struct ConfigManager {
    configs: HashMap<String, serde_json::Value>,
    environment: Environment,
}

impl ConfigManager {
    /// Crear nuevo manager de configuración
    pub fn new(environment: Environment) -> Self {
        Self {
            configs: HashMap::new(),
            environment,
        }
    }
    
    /// Cargar configuración desde archivo JSON
    pub fn load_json<T>(&mut self, key: &str, path: &str) -> CoreResult<T>
    where
        T: DeserializeOwned,
    {
        info!("📋 Cargando configuración desde: {}", path);
        
        if !Path::new(path).exists() {
            return Err(CoreError::Config(ConfigError::FileNotFound(path.to_string())));
        }
        
        let content = fs::read_to_string(path)
            .map_err(|e| CoreError::Config(ConfigError::IoError(e)))?;
            
        let config: T = serde_json::from_str(&content)
            .map_err(|e| CoreError::Config(ConfigError::ParseError(e.to_string())))?;
            
        // Guardar el valor raw para futuro uso
        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| CoreError::Config(ConfigError::ParseError(e.to_string())))?;
        self.configs.insert(key.to_string(), value);
        
        info!("✅ Configuración '{}' cargada exitosamente", key);
        Ok(config)
    }
    
    /// Guardar configuración a archivo JSON
    pub fn save_json<T>(&mut self, key: &str, config: &T, path: &str) -> CoreResult<()>
    where
        T: Serialize,
    {
        debug!("💾 Guardando configuración '{}' en: {}", key, path);
        
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| CoreError::Config(ConfigError::ParseError(e.to_string())))?;
            
        fs::write(path, content)
            .map_err(|e| CoreError::Config(ConfigError::IoError(e)))?;
            
        // Actualizar cache
        let value: serde_json::Value = serde_json::to_value(config)
            .map_err(|e| CoreError::Config(ConfigError::ParseError(e.to_string())))?;
        self.configs.insert(key.to_string(), value);
        
        info!("✅ Configuración '{}' guardada exitosamente", key);
        Ok(())
    }
    
    /// Obtener configuración del cache
    pub fn get_config<T>(&self, key: &str) -> CoreResult<T>
    where
        T: DeserializeOwned,
    {
        let value = self.configs.get(key)
            .ok_or_else(|| CoreError::Config(ConfigError::ValidationError(format!("Config '{}' not found", key))))?;
            
        let config: T = serde_json::from_value(value.clone())
            .map_err(|e| CoreError::Config(ConfigError::ParseError(e.to_string())))?;
            
        Ok(config)
    }
    
    /// Validar configuración con schema personalizado
    pub fn validate_config<T, F>(&self, key: &str, validator: F) -> CoreResult<()>
    where
        T: DeserializeOwned,
        F: Fn(&T) -> Result<(), String>,
    {
        let config: T = self.get_config(key)?;
        
        if let Err(msg) = validator(&config) {
            return Err(CoreError::Config(ConfigError::ValidationError(msg)));
        }
        
        info!("✅ Configuración '{}' validada correctamente", key);
        Ok(())
    }
    
    /// Obtener variable de entorno con fallback
    pub fn get_env_or_default(&self, key: &str, default: &str) -> String {
        std::env::var(key).unwrap_or_else(|_| {
            debug!("🔧 Usando valor por defecto para {}: {}", key, default);
            default.to_string()
        })
    }
    
    /// Crear configuración por defecto para un bot específico
    pub fn create_default_config<T>(&self) -> T
    where
        T: Default,
    {
        T::default()
    }
    
    /// Merge de configuraciones (útil para overrides)
    pub fn merge_configs(&mut self, base_key: &str, override_key: &str, result_key: &str) -> CoreResult<()> {
        let base = self.configs.get(base_key)
            .ok_or_else(|| CoreError::Config(ConfigError::ValidationError(format!("Base config '{}' not found", base_key))))?
            .clone();
            
        let override_val = self.configs.get(override_key)
            .ok_or_else(|| CoreError::Config(ConfigError::ValidationError(format!("Override config '{}' not found", override_key))))?
            .clone();
            
        let merged = self.merge_json_values(base, override_val)?;
        self.configs.insert(result_key.to_string(), merged);
        
        info!("✅ Configuraciones '{}' + '{}' → '{}' merged", base_key, override_key, result_key);
        Ok(())
    }
    
    /// Helper para merge de JSON values
    fn merge_json_values(&self, base: serde_json::Value, override_val: serde_json::Value) -> CoreResult<serde_json::Value> {
        use serde_json::Value;
        
        match (base, override_val) {
            (Value::Object(mut base_obj), Value::Object(override_obj)) => {
                for (key, value) in override_obj {
                    if let Some(base_value) = base_obj.get(&key) {
                        base_obj.insert(key, self.merge_json_values(base_value.clone(), value)?);
                    } else {
                        base_obj.insert(key, value);
                    }
                }
                Ok(Value::Object(base_obj))
            }
            (_, override_val) => Ok(override_val),
        }
    }
    
    /// Listar todas las configuraciones cargadas
    pub fn list_configs(&self) -> Vec<String> {
        self.configs.keys().cloned().collect()
    }
    
    /// Hot reload de configuración
    pub fn reload_config<T>(&mut self, key: &str, path: &str) -> CoreResult<T>
    where
        T: DeserializeOwned,
    {
        warn!("🔄 Hot reload de configuración '{}'", key);
        self.load_json(key, path)
    }
}

/// Configuración base compartida por todos los bots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedConfig {
    pub environment: String,
    pub log_level: String,
    pub rpc_url: String,
    pub rpc_timeout_seconds: u64,
    pub max_retries: u32,
    pub priority_fee_lamports: u64,
}

impl Default for SharedConfig {
    fn default() -> Self {
        Self {
            environment: "development".to_string(),
            log_level: "info".to_string(),
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            rpc_timeout_seconds: 30,
            max_retries: 3,
            priority_fee_lamports: 1000,
        }
    }
}

/// Configuración de wallet compartida
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub keypair_file: String,
    pub backup_keypair_file: Option<String>,
    pub use_env_private_key: bool,
    pub env_key_name: String,
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            keypair_file: "./keypair.json".to_string(),
            backup_keypair_file: Some("~/.config/solana/id.json".to_string()),
            use_env_private_key: false,
            env_key_name: "SOLANA_PRIVATE_KEY".to_string(),
        }
    }
}

/// Configuración de APIs externas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub jupiter: JupiterApiConfig,
    pub dexscreener: DexScreenerApiConfig,
    pub coinbase: CoinbaseApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterApiConfig {
    pub enabled: bool,
    pub quote_url: String,
    pub price_url: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

impl Default for JupiterApiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            quote_url: "https://quote-api.jup.ag/v6".to_string(),
            price_url: "https://api.jup.ag/price/v2".to_string(),
            timeout_seconds: 10,
            max_retries: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexScreenerApiConfig {
    pub enabled: bool,
    pub base_url: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub rate_limit_per_sec: u32,
}

impl Default for DexScreenerApiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base_url: "https://api.dexscreener.com/latest/dex".to_string(),
            timeout_seconds: 5,
            max_retries: 2,
            rate_limit_per_sec: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseApiConfig {
    pub enabled: bool,
    pub base_url: String,
    pub timeout_seconds: u64,
}

impl Default for CoinbaseApiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "https://api.coinbase.com/v2".to_string(),
            timeout_seconds: 5,
        }
    }
}

/// Trait para configuración específica de cada bot
pub trait BotConfigurable {
    type Config: Serialize + DeserializeOwned + Default;
    
    const CONFIG_KEY: &'static str;
    const DEFAULT_PATH: &'static str;
    
    fn load_config(manager: &mut ConfigManager) -> CoreResult<Self::Config> {
        manager.load_json(Self::CONFIG_KEY, Self::DEFAULT_PATH)
    }
    
    fn save_config(manager: &mut ConfigManager, config: &Self::Config) -> CoreResult<()> {
        manager.save_json(Self::CONFIG_KEY, config, Self::DEFAULT_PATH)
    }
    
    fn validate_config(config: &Self::Config) -> Result<(), String> {
        // Default validation - override in implementations
        Ok(())
    }
}

/// Helper functions para validación común
pub mod validators {
    use super::*;
    
    pub fn validate_positive_f64(value: f64, field: &str) -> Result<(), String> {
        if value <= 0.0 {
            Err(format!("{} must be positive, got: {}", field, value))
        } else {
            Ok(())
        }
    }
    
    pub fn validate_range_f64(value: f64, min: f64, max: f64, field: &str) -> Result<(), String> {
        if value < min || value > max {
            Err(format!("{} must be between {} and {}, got: {}", field, min, max, value))
        } else {
            Ok(())
        }
    }
    
    pub fn validate_percentage(value: f64, field: &str) -> Result<(), String> {
        validate_range_f64(value, 0.0, 1.0, field)
    }
    
    pub fn validate_file_exists(path: &str, field: &str) -> Result<(), String> {
        if !Path::new(path).exists() {
            Err(format!("{} file not found: {}", field, path))
        } else {
            Ok(())
        }
    }
    
    pub fn validate_url(url: &str, field: &str) -> Result<(), String> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            Err(format!("{} must be a valid URL, got: {}", field, url))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_config_manager_json() {
        let mut manager = ConfigManager::new(Environment::Testing);
        
        // Create temp file with test config
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_config = r#"{
            "test_value": 42,
            "test_string": "hello"
        }"#;
        write!(temp_file, "{}", test_config).unwrap();
        
        // Test loading
        #[derive(Deserialize)]
        struct TestConfig {
            test_value: i32,
            test_string: String,
        }
        
        let config: TestConfig = manager.load_json("test", temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.test_value, 42);
        assert_eq!(config.test_string, "hello");
    }
    
    #[test]
    fn test_validators() {
        use validators::*;
        
        assert!(validate_positive_f64(1.0, "test").is_ok());
        assert!(validate_positive_f64(-1.0, "test").is_err());
        
        assert!(validate_range_f64(0.5, 0.0, 1.0, "test").is_ok());
        assert!(validate_range_f64(1.5, 0.0, 1.0, "test").is_err());
        
        assert!(validate_percentage(0.5, "test").is_ok());
        assert!(validate_percentage(1.5, "test").is_err());
        
        assert!(validate_url("https://example.com", "test").is_ok());
        assert!(validate_url("invalid-url", "test").is_err());
    }
}
