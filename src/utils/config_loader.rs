//! Advanced Configuration Loading Utilities
//! 
//! Migrated from old-root-archive with enterprise support

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::Path,
};
use crate::types::Result;

/// Advanced configuration loader for enterprise environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigLoader {
    /// Configuration file paths by environment
    pub config_paths: HashMap<String, String>,
    /// Environment variables mapping
    pub env_mappings: HashMap<String, String>,
    /// Fallback configurations
    pub fallbacks: HashMap<String, serde_json::Value>,
    /// Validation rules
    pub validation_enabled: bool,
}

impl Default for ConfigLoader {
    fn default() -> Self {
        let mut config_paths = HashMap::new();
        config_paths.insert("development".to_string(), "config/dev.json".to_string());
        config_paths.insert("production".to_string(), "config/prod.json".to_string());
        config_paths.insert("enterprise".to_string(), "config/enterprise.json".to_string());

        let mut env_mappings = HashMap::new();
        env_mappings.insert("SOLANA_RPC_URL".to_string(), "solana.rpc_url".to_string());
        env_mappings.insert("JUPITER_API_KEY".to_string(), "jupiter.api_key".to_string());
        env_mappings.insert("LOG_LEVEL".to_string(), "logging.level".to_string());

        Self {
            config_paths,
            env_mappings,
            fallbacks: HashMap::new(),
            validation_enabled: true,
        }
    }
}

impl ConfigLoader {
    /// Create a new ConfigLoader with custom paths
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from file with environment overrides
    pub fn load_config<T>(&self, environment: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let config_path = self.config_paths
            .get(environment)
            .ok_or_else(|| format!("Environment '{}' not configured", environment))?;

        if !Path::new(config_path).exists() {
            return Err(format!("Configuration file not found: {}", config_path).into());
        }

        let content = fs::read_to_string(config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

        // Apply environment variable overrides
        self.apply_env_overrides(&mut config)?;

        // Apply fallbacks if needed
        self.apply_fallbacks(&mut config);

        // Convert to target type
        serde_json::from_value(config)
            .map_err(|e| format!("Failed to deserialize config: {}", e).into())
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&self, config: &mut serde_json::Value) -> Result<()> {
        for (env_var, config_path) in &self.env_mappings {
            if let Ok(env_value) = std::env::var(env_var) {
                self.set_nested_value(config, config_path, env_value)?;
            }
        }
        Ok(())
    }

    /// Set nested configuration value
    fn set_nested_value(
        &self,
        config: &mut serde_json::Value,
        path: &str,
        value: String,
    ) -> Result<()> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = config;

        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part - set the value
                if let Some(obj) = current.as_object_mut() {
                    obj.insert(part.to_string(), serde_json::Value::String(value));
                    return Ok(());
                } else {
                    return Err("Cannot set value on non-object".to_string().into());
                }
            } else {
                // Intermediate part - navigate deeper
                if let Some(obj) = current.as_object_mut() {
                    if !obj.contains_key(*part) {
                        obj.insert(part.to_string(), serde_json::Value::Object(Default::default()));
                    }
                    // Get the next level - this resolves the borrow checker issue
                    let next_key = part.to_string();
                    current = obj.get_mut(&next_key).unwrap();
                } else {
                    return Err("Cannot navigate through non-object".to_string().into());
                }
            }
        }
        Ok(())
    }

    /// Apply fallback values
    fn apply_fallbacks(&self, config: &mut serde_json::Value) {
        for (path, fallback_value) in &self.fallbacks {
            if self.get_nested_value(config, path).is_none() {
                let _ = self.set_nested_value(config, path, fallback_value.to_string());
            }
        }
    }

    /// Get nested configuration value
    fn get_nested_value<'a>(&self, config: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = config;

        for part in parts {
            current = current.get(part)?;
        }
        Some(current)
    }

    /// Add fallback configuration
    pub fn add_fallback<T: Serialize>(&mut self, path: &str, value: T) -> Result<()> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| format!("Failed to serialize fallback value: {}", e))?;
        self.fallbacks.insert(path.to_string(), json_value);
        Ok(())
    }

    /// Validate configuration structure
    pub fn validate_config(&self, config: &serde_json::Value) -> Result<()> {
        if !self.validation_enabled {
            return Ok(());
        }

        // Basic validation - ensure it's an object
        if !config.is_object() {
            return Err("Configuration must be a JSON object".to_string().into());
        }

        // Additional validation can be added here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_config_loader_creation() {
        let loader = ConfigLoader::new();
        assert!(loader.config_paths.contains_key("development"));
        assert!(loader.config_paths.contains_key("production"));
        assert!(loader.config_paths.contains_key("enterprise"));
    }

    #[test]
    fn test_nested_value_operations() {
        let loader = ConfigLoader::new();
        let mut config = json!({
            "solana": {
                "rpc_url": "https://api.devnet.solana.com"
            }
        });

        // Test getting nested value
        let rpc_url = loader.get_nested_value(&config, "solana.rpc_url");
        assert!(rpc_url.is_some());

        // Test setting nested value
        let result = loader.set_nested_value(&mut config, "jupiter.api_key", "test_key".to_string());
        assert!(result.is_ok());
        
        let api_key = loader.get_nested_value(&config, "jupiter.api_key");
        assert!(api_key.is_some());
    }

    #[test]
    fn test_fallback_configuration() {
        let mut loader = ConfigLoader::new();
        let result = loader.add_fallback("logging.level", "info");
        assert!(result.is_ok());
        assert!(loader.fallbacks.contains_key("logging.level"));
    }
}
