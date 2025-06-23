//! Model Manager Module
//! 
//! Handles ML model loading, saving, versioning, and lifecycle management.
//! Provides a centralized interface for all ML models in SniperForge.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn, error};
use tokio::fs;

/// Configuration for model management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManagerConfig {
    pub models_directory: PathBuf,
    pub auto_backup: bool,
    pub max_model_versions: usize,
    pub performance_threshold: f64,
    pub retrain_interval_days: u32,
    pub model_cache_size: usize,
}

impl Default for ModelManagerConfig {
    fn default() -> Self {
        Self {
            models_directory: PathBuf::from("models/"),
            auto_backup: true,
            max_model_versions: 5,
            performance_threshold: 0.8,
            retrain_interval_days: 7,
            model_cache_size: 10,
        }
    }
}

/// Types of ML models supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelType {
    PatternRecognition,
    StrategyOptimizer,
    RiskAssessment,
    TimingPredictor,
    VolatilityPredictor,
    TrendClassifier,
}

/// Model metadata and information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_id: String,
    pub model_type: ModelType,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub trained_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub performance_metrics: PerformanceMetrics,
    pub training_config: TrainingConfig,
    pub feature_schema: FeatureSchema,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub checksum: String,
}

/// Performance metrics for model evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub accuracy: Option<f64>,
    pub precision: Option<f64>,
    pub recall: Option<f64>,
    pub f1_score: Option<f64>,
    pub auc_roc: Option<f64>,
    pub mse: Option<f64>,
    pub mae: Option<f64>,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub win_rate: Option<f64>,
    pub avg_return: Option<f64>,
    pub evaluation_date: DateTime<Utc>,
    pub sample_size: usize,
}

/// Training configuration used for the model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub training_period_start: DateTime<Utc>,
    pub training_period_end: DateTime<Utc>,
    pub validation_split: f64,
    pub hyperparameters: HashMap<String, serde_json::Value>,
    pub features_used: Vec<String>,
    pub target_variable: String,
    pub preprocessing_steps: Vec<String>,
}

/// Schema for input features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSchema {
    pub feature_names: Vec<String>,
    pub feature_types: Vec<FeatureType>,
    pub feature_ranges: Vec<(f64, f64)>,
    pub required_features: Vec<String>,
    pub optional_features: Vec<String>,
}

/// Types of features supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureType {
    Numeric,
    Categorical,
    Binary,
    Temporal,
}

/// Model state and loaded instance
#[derive(Debug)]
pub struct LoadedModel {
    pub metadata: ModelMetadata,
    pub model_data: Vec<u8>,
    pub is_loaded: bool,
    pub last_prediction: Option<DateTime<Utc>>,
    pub prediction_count: u64,
}

/// Centralized model manager
pub struct ModelManager {
    config: ModelManagerConfig,
    models: HashMap<String, ModelMetadata>,
    loaded_models: HashMap<String, LoadedModel>,
    model_registry: ModelRegistry,
}

/// Registry for tracking all models
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelRegistry {
    models: HashMap<String, ModelMetadata>,
    last_updated: DateTime<Utc>,
    registry_version: String,
}

impl ModelManager {
    /// Create a new model manager
    pub async fn new(config: ModelManagerConfig) -> Result<Self> {
        info!("Initializing Model Manager with config: {:?}", config);

        // Ensure models directory exists
        if !config.models_directory.exists() {
            fs::create_dir_all(&config.models_directory).await?;
            info!("Created models directory: {:?}", config.models_directory);
        }

        let registry_path = config.models_directory.join("registry.json");
        let model_registry = if registry_path.exists() {
            let content = fs::read_to_string(&registry_path).await?;
            serde_json::from_str(&content)?
        } else {
            ModelRegistry::new()
        };

        let mut manager = Self {
            config,
            models: HashMap::new(),
            loaded_models: HashMap::new(),
            model_registry,
        };

        // Load existing models from registry
        manager.refresh_model_registry().await?;

        Ok(manager)
    }

    /// Register a new model
    pub async fn register_model(
        &mut self,
        model_type: ModelType,
        model_data: Vec<u8>,
        metadata: ModelMetadata,
    ) -> Result<String> {
        let model_id = format!(
            "{:?}_{}_{}", 
            model_type, 
            metadata.version,
            metadata.created_at.timestamp()
        );

        info!("Registering new model: {}", model_id);

        // Save model file
        let model_filename = format!("{}.model", model_id);
        let model_path = self.config.models_directory.join(&model_filename);
        
        fs::write(&model_path, &model_data).await?;

        // Update metadata with file info
        let mut updated_metadata = metadata;
        updated_metadata.model_id = model_id.clone();
        updated_metadata.file_path = model_path;
        updated_metadata.file_size = model_data.len() as u64;
        updated_metadata.checksum = self.calculate_checksum(&model_data);

        // Add to registry
        self.models.insert(model_id.clone(), updated_metadata.clone());
        self.model_registry.models.insert(model_id.clone(), updated_metadata);
        self.model_registry.last_updated = Utc::now();

        // Save registry
        self.save_registry().await?;

        // Cleanup old versions if needed
        self.cleanup_old_versions(&model_type).await?;

        info!("Successfully registered model: {}", model_id);
        Ok(model_id)
    }

    /// Load a model into memory
    pub async fn load_model(&mut self, model_id: &str) -> Result<()> {
        info!("Loading model: {}", model_id);

        if self.loaded_models.contains_key(model_id) {
            debug!("Model already loaded: {}", model_id);
            return Ok(());
        }

        let metadata = self.models.get(model_id)
            .ok_or_else(|| anyhow!("Model not found: {}", model_id))?
            .clone();

        // Read model data from file
        let model_data = fs::read(&metadata.file_path).await?;

        // Verify checksum
        let actual_checksum = self.calculate_checksum(&model_data);
        if actual_checksum != metadata.checksum {
            return Err(anyhow!("Model file corrupted: checksum mismatch for {}", model_id));
        }

        // Create loaded model instance
        let loaded_model = LoadedModel {
            metadata: metadata.clone(),
            model_data,
            is_loaded: true,
            last_prediction: None,
            prediction_count: 0,
        };

        self.loaded_models.insert(model_id.to_string(), loaded_model);

        // Update last used timestamp
        if let Some(metadata) = self.models.get_mut(model_id) {
            metadata.last_used = Some(Utc::now());
        }

        // Manage cache size
        self.manage_cache_size().await?;

        info!("Successfully loaded model: {}", model_id);
        Ok(())
    }

    /// Unload a model from memory
    pub async fn unload_model(&mut self, model_id: &str) -> Result<()> {
        info!("Unloading model: {}", model_id);

        if let Some(mut loaded_model) = self.loaded_models.remove(model_id) {
            loaded_model.is_loaded = false;
            debug!("Model unloaded: {} (predictions: {})", 
                   model_id, loaded_model.prediction_count);
        }

        Ok(())
    }

    /// Get model for prediction
    pub async fn get_model(&mut self, model_id: &str) -> Result<&LoadedModel> {
        if !self.loaded_models.contains_key(model_id) {
            self.load_model(model_id).await?;
        }

        self.loaded_models.get(model_id)
            .ok_or_else(|| anyhow!("Failed to load model: {}", model_id))
    }

    /// Find best model for a given type
    pub fn find_best_model(&self, model_type: &ModelType) -> Option<String> {
        let mut best_model = None;
        let mut best_score = 0.0;

        for (model_id, metadata) in &self.models {
            if metadata.model_type == *model_type {
                let score = self.calculate_model_score(&metadata.performance_metrics);
                if score > best_score {
                    best_score = score;
                    best_model = Some(model_id.clone());
                }
            }
        }

        best_model
    }

    /// Update model performance metrics
    pub async fn update_performance(
        &mut self,
        model_id: &str,
        metrics: PerformanceMetrics,
    ) -> Result<()> {
        info!("Updating performance metrics for model: {}", model_id);

        if let Some(metadata) = self.models.get_mut(model_id) {
            metadata.performance_metrics = metrics;
            
            // Update registry
            if let Some(registry_metadata) = self.model_registry.models.get_mut(model_id) {
                registry_metadata.performance_metrics = metadata.performance_metrics.clone();
            }
            
            self.save_registry().await?;
            info!("Performance metrics updated for model: {}", model_id);
        } else {
            warn!("Model not found for performance update: {}", model_id);
        }

        Ok(())
    }

    /// List all models of a specific type
    pub fn list_models(&self, model_type: Option<ModelType>) -> Vec<&ModelMetadata> {
        self.models
            .values()
            .filter(|metadata| {
                model_type.is_none() || Some(&metadata.model_type) == model_type.as_ref()
            })
            .collect()
    }

    /// Get model metadata
    pub fn get_metadata(&self, model_id: &str) -> Option<&ModelMetadata> {
        self.models.get(model_id)
    }

    /// Check if models need retraining
    pub fn models_needing_retrain(&self) -> Vec<String> {
        let threshold_date = Utc::now() - chrono::Duration::days(
            self.config.retrain_interval_days as i64
        );

        self.models
            .iter()
            .filter(|(_, metadata)| {
                metadata.trained_at
                    .map(|trained| trained < threshold_date)
                    .unwrap_or(true)
            })
            .map(|(model_id, _)| model_id.clone())
            .collect()
    }

    /// Export model for external use
    pub async fn export_model(
        &self,
        model_id: &str,
        export_path: &Path,
    ) -> Result<()> {
        info!("Exporting model {} to {:?}", model_id, export_path);

        let metadata = self.models.get(model_id)
            .ok_or_else(|| anyhow!("Model not found: {}", model_id))?;

        // Copy model file
        fs::copy(&metadata.file_path, export_path).await?;

        // Export metadata
        let metadata_path = export_path.with_extension("metadata.json");
        let metadata_json = serde_json::to_string_pretty(metadata)?;
        fs::write(metadata_path, metadata_json).await?;

        info!("Model exported successfully");
        Ok(())
    }

    /// Import model from external source
    pub async fn import_model(
        &mut self,
        model_path: &Path,
        metadata_path: &Path,
    ) -> Result<String> {
        info!("Importing model from {:?}", model_path);

        // Read metadata
        let metadata_content = fs::read_to_string(metadata_path).await?;
        let metadata: ModelMetadata = serde_json::from_str(&metadata_content)?;

        // Read model data
        let model_data = fs::read(model_path).await?;

        // Register the imported model
        let model_id = self.register_model(
            metadata.model_type.clone(),
            model_data,
            metadata,
        ).await?;

        info!("Model imported successfully: {}", model_id);
        Ok(model_id)
    }

    /// Get model usage statistics
    pub fn get_usage_statistics(&self) -> HashMap<String, ModelUsageStats> {
        let mut stats = HashMap::new();

        for (model_id, loaded_model) in &self.loaded_models {
            stats.insert(
                model_id.clone(),
                ModelUsageStats {
                    prediction_count: loaded_model.prediction_count,
                    last_used: loaded_model.last_prediction,
                    is_loaded: loaded_model.is_loaded,
                    memory_usage: loaded_model.model_data.len(),
                },
            );
        }

        stats
    }

    /// Refresh model registry from disk
    async fn refresh_model_registry(&mut self) -> Result<()> {
        debug!("Refreshing model registry");

        for (model_id, metadata) in &self.model_registry.models {
            if metadata.file_path.exists() {
                self.models.insert(model_id.clone(), metadata.clone());
            } else {
                warn!("Model file not found: {:?}", metadata.file_path);
            }
        }

        Ok(())
    }

    /// Save registry to disk
    async fn save_registry(&self) -> Result<()> {
        let registry_path = self.config.models_directory.join("registry.json");
        let registry_json = serde_json::to_string_pretty(&self.model_registry)?;
        fs::write(registry_path, registry_json).await?;
        Ok(())
    }

    /// Calculate simple checksum for model verification
    fn calculate_checksum(&self, data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Calculate composite model score for ranking
    fn calculate_model_score(&self, metrics: &PerformanceMetrics) -> f64 {
        let mut score = 0.0;
        let mut count = 0;

        if let Some(accuracy) = metrics.accuracy {
            score += accuracy;
            count += 1;
        }

        if let Some(f1) = metrics.f1_score {
            score += f1;
            count += 1;
        }

        if let Some(sharpe) = metrics.sharpe_ratio {
            score += (sharpe / 3.0).clamp(0.0, 1.0); // Normalize Sharpe ratio
            count += 1;
        }

        if let Some(win_rate) = metrics.win_rate {
            score += win_rate;
            count += 1;
        }

        if count > 0 {
            score / count as f64
        } else {
            0.0
        }
    }

    /// Cleanup old model versions
    async fn cleanup_old_versions(&mut self, model_type: &ModelType) -> Result<()> {
        let mut type_models: Vec<_> = self.models
            .iter()
            .filter(|(_, metadata)| metadata.model_type == *model_type)
            .collect();

        // Sort by creation date, newest first
        type_models.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

        if type_models.len() > self.config.max_model_versions {
            let to_remove = &type_models[self.config.max_model_versions..];
            
            for (model_id, metadata) in to_remove {
                info!("Removing old model version: {}", model_id);
                
                // Remove file
                if metadata.file_path.exists() {
                    if let Err(e) = fs::remove_file(&metadata.file_path).await {
                        warn!("Failed to remove model file: {}", e);
                    }
                }

                // Remove from memory and registry
                self.models.remove(*model_id);
                self.model_registry.models.remove(*model_id);
                self.loaded_models.remove(*model_id);
            }

            self.save_registry().await?;
        }

        Ok(())
    }

    /// Manage loaded model cache size
    async fn manage_cache_size(&mut self) -> Result<()> {
        if self.loaded_models.len() <= self.config.model_cache_size {
            return Ok(());
        }

        // Find least recently used models to unload
        let mut models_by_usage: Vec<_> = self.loaded_models
            .iter()
            .map(|(id, model)| (
                id.clone(),
                model.last_prediction.unwrap_or(model.metadata.created_at)
            ))
            .collect();

        models_by_usage.sort_by(|a, b| a.1.cmp(&b.1));

        // Unload oldest models
        let to_unload = models_by_usage.len() - self.config.model_cache_size;
        for i in 0..to_unload {
            let model_id = &models_by_usage[i].0;
            self.unload_model(model_id).await?;
        }

        Ok(())
    }
}

/// Model usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsageStats {
    pub prediction_count: u64,
    pub last_used: Option<DateTime<Utc>>,
    pub is_loaded: bool,
    pub memory_usage: usize,
}

impl ModelRegistry {
    fn new() -> Self {
        Self {
            models: HashMap::new(),
            last_updated: Utc::now(),
            registry_version: "1.0.0".to_string(),
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            accuracy: None,
            precision: None,
            recall: None,
            f1_score: None,
            auc_roc: None,
            mse: None,
            mae: None,
            sharpe_ratio: None,
            max_drawdown: None,
            win_rate: None,
            avg_return: None,
            evaluation_date: Utc::now(),
            sample_size: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_model_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = ModelManagerConfig {
            models_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = ModelManager::new(config).await.unwrap();
        assert_eq!(manager.models.len(), 0);
    }

    #[tokio::test]
    async fn test_model_registration() {
        let temp_dir = TempDir::new().unwrap();
        let config = ModelManagerConfig {
            models_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut manager = ModelManager::new(config).await.unwrap();
        
        let metadata = ModelMetadata {
            model_id: "test_model".to_string(),
            model_type: ModelType::PatternRecognition,
            version: "1.0.0".to_string(),
            created_at: Utc::now(),
            trained_at: Some(Utc::now()),
            last_used: None,
            performance_metrics: PerformanceMetrics::default(),
            training_config: TrainingConfig {
                training_period_start: Utc::now() - chrono::Duration::days(30),
                training_period_end: Utc::now(),
                validation_split: 0.2,
                hyperparameters: HashMap::new(),
                features_used: vec!["price".to_string(), "volume".to_string()],
                target_variable: "price_direction".to_string(),
                preprocessing_steps: vec!["normalization".to_string()],
            },
            feature_schema: FeatureSchema {
                feature_names: vec!["price".to_string(), "volume".to_string()],
                feature_types: vec![FeatureType::Numeric, FeatureType::Numeric],
                feature_ranges: vec![(0.0, 1000.0), (0.0, 10000.0)],
                required_features: vec!["price".to_string()],
                optional_features: vec!["volume".to_string()],
            },
            file_path: PathBuf::new(),
            file_size: 0,
            checksum: String::new(),
        };

        let model_data = vec![1, 2, 3, 4, 5]; // Dummy model data
        let model_id = manager.register_model(
            ModelType::PatternRecognition,
            model_data,
            metadata,
        ).await.unwrap();

        assert!(!model_id.is_empty());
        assert!(manager.models.contains_key(&model_id));
    }

    #[tokio::test]
    async fn test_model_loading() {
        let temp_dir = TempDir::new().unwrap();
        let config = ModelManagerConfig {
            models_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut manager = ModelManager::new(config).await.unwrap();
        
        // First register a model
        let metadata = ModelMetadata {
            model_id: "test_model".to_string(),
            model_type: ModelType::PatternRecognition,
            version: "1.0.0".to_string(),
            created_at: Utc::now(),
            trained_at: Some(Utc::now()),
            last_used: None,
            performance_metrics: PerformanceMetrics::default(),
            training_config: TrainingConfig {
                training_period_start: Utc::now() - chrono::Duration::days(30),
                training_period_end: Utc::now(),
                validation_split: 0.2,
                hyperparameters: HashMap::new(),
                features_used: vec!["price".to_string()],
                target_variable: "price_direction".to_string(),
                preprocessing_steps: vec!["normalization".to_string()],
            },
            feature_schema: FeatureSchema {
                feature_names: vec!["price".to_string()],
                feature_types: vec![FeatureType::Numeric],
                feature_ranges: vec![(0.0, 1000.0)],
                required_features: vec!["price".to_string()],
                optional_features: vec![],
            },
            file_path: PathBuf::new(),
            file_size: 0,
            checksum: String::new(),
        };

        let model_data = vec![1, 2, 3, 4, 5];
        let model_id = manager.register_model(
            ModelType::PatternRecognition,
            model_data,
            metadata,
        ).await.unwrap();

        // Now test loading
        manager.load_model(&model_id).await.unwrap();
        assert!(manager.loaded_models.contains_key(&model_id));
        assert!(manager.loaded_models[&model_id].is_loaded);
    }
}
