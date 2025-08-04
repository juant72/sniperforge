//! Bot Factory - Creates and manages bot instances for the containerized ecosystem

use crate::api::bot_interface::{BotInterface, BotType, BotConfig, BotError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use anyhow::Result;

/// Factory for creating and managing bot instances
pub struct BotFactory {
    /// Registry of bot constructors
    constructors: HashMap<BotType, BotConstructor>,
    /// Active bot instances
    active_bots: Arc<RwLock<HashMap<Uuid, Box<dyn BotInterface>>>>,
}

/// Bot constructor function type
type BotConstructor = Box<dyn Fn(BotConfig) -> Result<Box<dyn BotInterface>, BotError> + Send + Sync>;

impl BotFactory {
    /// Create new bot factory with default constructors
    pub fn new() -> Self {
        let mut factory = Self {
            constructors: HashMap::new(),
            active_bots: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Register default bot constructors
        factory.register_default_constructors();
        factory
    }
    
    /// Register a bot constructor for a specific bot type
    pub fn register_constructor<F>(&mut self, bot_type: BotType, constructor: F)
    where
        F: Fn(BotConfig) -> Result<Box<dyn BotInterface>, BotError> + Send + Sync + 'static,
    {
        self.constructors.insert(bot_type, Box::new(constructor));
    }
    
    /// Create a new bot instance
    pub async fn create_bot(&self, bot_type: BotType, config: BotConfig) -> Result<Uuid, BotError> {
        let constructor = self.constructors.get(&bot_type)
            .ok_or_else(|| BotError::Configuration(format!("No constructor found for bot type: {:?}", bot_type)))?;
        
        let bot = constructor(config)?;
        let bot_id = bot.bot_id();
        
        // Store the bot instance
        let mut active_bots = self.active_bots.write().await;
        active_bots.insert(bot_id, bot);
        
        Ok(bot_id)
    }
    
    /// Get a bot instance by ID
    pub async fn get_bot(&self, bot_id: Uuid) -> Option<Box<dyn BotInterface>> {
        let active_bots = self.active_bots.read().await;
        // Note: This is a simplified version. In practice, you'd want to return a reference
        // or implement a more sophisticated borrowing mechanism
        None // Placeholder - requires more complex lifetime management
    }
    
    /// Remove a bot instance
    pub async fn remove_bot(&self, bot_id: Uuid) -> Result<(), BotError> {
        let mut active_bots = self.active_bots.write().await;
        active_bots.remove(&bot_id)
            .ok_or_else(|| BotError::Configuration(format!("Bot not found: {}", bot_id)))?;
        Ok(())
    }
    
    /// List all active bots
    pub async fn list_active_bots(&self) -> Vec<Uuid> {
        let active_bots = self.active_bots.read().await;
        active_bots.keys().cloned().collect()
    }
    
    /// Get bot count by type
    pub async fn get_bot_count_by_type(&self) -> HashMap<BotType, usize> {
        let mut counts = HashMap::new();
        // Placeholder implementation
        counts
    }
    
    /// Register default bot constructors
    fn register_default_constructors(&mut self) {
        // Enhanced Arbitrage Bot
        self.register_constructor(BotType::EnhancedArbitrage, |config| {
            // This would create an actual EnhancedArbitrageBot instance
            Err(BotError::Configuration("EnhancedArbitrageBot constructor not implemented".to_string()))
        });
        
        // Triangular Arbitrage Bot
        self.register_constructor(BotType::TriangularArbitrage, |config| {
            Err(BotError::Configuration("TriangularArbitrageBot constructor not implemented".to_string()))
        });
        
        // ML Analytics Bot
        self.register_constructor(BotType::MLAnalytics, |config| {
            Err(BotError::Configuration("MLAnalyticsBot constructor not implemented".to_string()))
        });
        
        // Portfolio Manager Bot
        self.register_constructor(BotType::PortfolioManager, |config| {
            Err(BotError::Configuration("PortfolioManagerBot constructor not implemented".to_string()))
        });
        
        // Real-time Dashboard Bot
        self.register_constructor(BotType::RealTimeDashboard, |config| {
            Err(BotError::Configuration("DashboardBot constructor not implemented".to_string()))
        });
        
        // Flash Loan Arbitrage Bot
        self.register_constructor(BotType::FlashLoanArbitrage, |config| {
            Err(BotError::Configuration("FlashLoanArbitrageBot constructor not implemented".to_string()))
        });
        
        // Cross Chain Arbitrage Bot
        self.register_constructor(BotType::CrossChainArbitrage, |config| {
            Err(BotError::Configuration("CrossChainArbitrageBot constructor not implemented".to_string()))
        });
        
        // Performance Profiler Bot
        self.register_constructor(BotType::PerformanceProfiler, |config| {
            Err(BotError::Configuration("PerformanceProfilerBot constructor not implemented".to_string()))
        });
        
        // Pattern Analyzer Bot
        self.register_constructor(BotType::PatternAnalyzer, |config| {
            Err(BotError::Configuration("PatternAnalyzerBot constructor not implemented".to_string()))
        });
    }
}

/// Bot registry for managing bot metadata
pub struct BotRegistry {
    /// Bot type metadata
    metadata: HashMap<BotType, BotTypeMetadata>,
}

/// Metadata for a bot type
#[derive(Debug, Clone)]
pub struct BotTypeMetadata {
    /// Bot type name
    pub name: String,
    /// Bot description
    pub description: String,
    /// Bot version
    pub version: String,
    /// Supported features
    pub features: Vec<String>,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
    /// Configuration schema
    pub config_schema: serde_json::Value,
}

/// Resource requirements for a bot type
#[derive(Debug, Clone)]
pub struct ResourceRequirements {
    /// Minimum CPU cores
    pub min_cpu: f64,
    /// Minimum memory (MB)
    pub min_memory_mb: u64,
    /// Minimum disk space (MB)
    pub min_disk_mb: u64,
    /// Network requirements
    pub network_required: bool,
}

impl BotRegistry {
    /// Create new bot registry
    pub fn new() -> Self {
        let mut registry = Self {
            metadata: HashMap::new(),
        };
        
        registry.register_default_metadata();
        registry
    }
    
    /// Register bot type metadata
    pub fn register_metadata(&mut self, bot_type: BotType, metadata: BotTypeMetadata) {
        self.metadata.insert(bot_type, metadata);
    }
    
    /// Get metadata for a bot type
    pub fn get_metadata(&self, bot_type: &BotType) -> Option<&BotTypeMetadata> {
        self.metadata.get(bot_type)
    }
    
    /// List all registered bot types
    pub fn list_bot_types(&self) -> Vec<BotType> {
        self.metadata.keys().cloned().collect()
    }
    
    /// Register default bot metadata
    fn register_default_metadata(&mut self) {
        // Enhanced Arbitrage Bot metadata
        self.register_metadata(BotType::EnhancedArbitrage, BotTypeMetadata {
            name: "Enhanced Arbitrage Bot".to_string(),
            description: "Advanced arbitrage detection with ML analysis".to_string(),
            version: "3.0.0".to_string(),
            features: vec![
                "Real-time arbitrage detection".to_string(),
                "ML-powered confidence scoring".to_string(),
                "Multi-DEX support".to_string(),
                "Risk management".to_string(),
            ],
            resource_requirements: ResourceRequirements {
                min_cpu: 0.5,
                min_memory_mb: 256,
                min_disk_mb: 512,
                network_required: true,
            },
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "trading": {
                        "type": "object",
                        "properties": {
                            "min_profit_bps": {"type": "number", "minimum": 1, "maximum": 1000},
                            "max_trade_sol": {"type": "number", "minimum": 0.01, "maximum": 1000},
                            "confidence_threshold": {"type": "number", "minimum": 0, "maximum": 1}
                        }
                    }
                }
            }),
        });
        
        // Add metadata for other bot types...
        // (Similar pattern for all bot types)
    }
}

impl Default for BotFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for BotRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bot_factory_creation() {
        let factory = BotFactory::new();
        assert!(!factory.constructors.is_empty());
    }
    
    #[test]
    fn test_bot_registry_creation() {
        let registry = BotRegistry::new();
        assert!(!registry.metadata.is_empty());
    }
    
    #[test]
    fn test_bot_registry_metadata() {
        let registry = BotRegistry::new();
        let metadata = registry.get_metadata(&BotType::EnhancedArbitrage);
        assert!(metadata.is_some());
        assert_eq!(metadata.unwrap().name, "Enhanced Arbitrage Bot");
    }
}
