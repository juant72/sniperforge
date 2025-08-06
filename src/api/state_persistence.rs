//! State Persistence Module
//! 
//! This module provides enterprise-grade persistence for bot states, configurations,
//! metrics, and operational history. It ensures system recovery after restarts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;
use tracing::{info, warn, error};

use crate::api::bot_interface::{BotConfig, BotType, BotStatus, BotMetrics};

/// System state persistence errors
#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("State file not found: {0}")]
    StateFileNotFound(String),
    
    #[error("Invalid state format: {0}")]
    InvalidStateFormat(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Bot runtime state for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedBotState {
    pub bot_id: Uuid,
    pub bot_type: BotType,
    pub status: BotStatus,
    pub config: Option<BotConfig>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub last_started_at: Option<DateTime<Utc>>,
    pub last_stopped_at: Option<DateTime<Utc>>,
    pub total_uptime_seconds: u64,
    pub restart_count: u32,
    pub last_known_metrics: Option<BotMetrics>,
}

/// System metrics history for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub total_bots: usize,
    pub running_bots: usize,
    pub total_profit: f64,
    pub total_trades: u64,
    pub system_uptime_seconds: u64,
    pub memory_usage_mb: f64,
}

/// Complete system state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStateSnapshot {
    pub version: String,
    pub snapshot_timestamp: DateTime<Utc>,
    pub server_start_count: u64,
    pub total_system_uptime_seconds: u64,
    pub bots: HashMap<Uuid, PersistedBotState>,
    pub default_arbitrage_bot: Option<Uuid>,
    pub metrics_history: Vec<PersistedSystemMetrics>,
    pub configuration_hashes: HashMap<String, String>, // Para detectar cambios
}

/// Enterprise-grade state persistence manager
pub struct StatePersistenceManager {
    persistence_path: PathBuf,
    state_file: PathBuf,
    backup_directory: PathBuf,
    current_state: RwLock<SystemStateSnapshot>,
    auto_save_enabled: bool,
    backup_retention_days: u32,
}

impl StatePersistenceManager {
    /// Create new persistence manager
    pub fn new<P: AsRef<Path>>(persistence_path: P) -> Self {
        let persistence_path = persistence_path.as_ref().to_path_buf();
        let state_file = persistence_path.join("system_state.json");
        let backup_directory = persistence_path.join("backups");
        
        Self {
            persistence_path,
            state_file,
            backup_directory,
            current_state: RwLock::new(SystemStateSnapshot::new()),
            auto_save_enabled: true,
            backup_retention_days: 30,
        }
    }
    
    /// Initialize persistence system
    pub async fn initialize(&self) -> Result<(), PersistenceError> {
        // Create directories
        fs::create_dir_all(&self.persistence_path).await?;
        fs::create_dir_all(&self.backup_directory).await?;
        
        // Load existing state or create new
        if self.state_file.exists() {
            info!("📂 Loading existing system state from disk...");
            self.load_system_state().await?;
        } else {
            info!("🆕 Initializing new system state...");
            self.create_initial_state().await?;
        }
        
        // Clean old backups
        self.cleanup_old_backups().await?;
        
        info!("✅ State persistence system initialized");
        Ok(())
    }
    
    /// Save bot state
    pub async fn save_bot_state(&self, bot_state: PersistedBotState) -> Result<(), PersistenceError> {
        let mut state = self.current_state.write().await;
        state.bots.insert(bot_state.bot_id, bot_state);
        
        if self.auto_save_enabled {
            drop(state); // Release lock before async operation
            self.save_state_to_disk().await?;
        }
        
        Ok(())
    }
    
    /// Update bot status
    pub async fn update_bot_status(&self, bot_id: Uuid, status: BotStatus) -> Result<(), PersistenceError> {
        let mut state = self.current_state.write().await;
        
        if let Some(bot_state) = state.bots.get_mut(&bot_id) {
            let old_status = bot_state.status.clone();
            bot_state.status = status.clone();
            
            // Update timestamps based on status transition
            match (&old_status, &status) {
                (BotStatus::Stopped, BotStatus::Running) => {
                    bot_state.last_started_at = Some(Utc::now());
                    bot_state.restart_count += 1;
                }
                (BotStatus::Running, BotStatus::Stopped) => {
                    bot_state.last_stopped_at = Some(Utc::now());
                    // Calculate and add uptime
                    if let Some(started_at) = bot_state.last_started_at {
                        let uptime = Utc::now().signed_duration_since(started_at).num_seconds() as u64;
                        bot_state.total_uptime_seconds += uptime;
                    }
                }
                _ => {}
            }
            
            if self.auto_save_enabled {
                drop(state); // Release lock
                self.save_state_to_disk().await?;
            }
        }
        
        Ok(())
    }
    
    /// Save system metrics snapshot
    pub async fn save_system_metrics(&self, metrics: PersistedSystemMetrics) -> Result<(), PersistenceError> {
        let mut state = self.current_state.write().await;
        
        // Add to history
        state.metrics_history.push(metrics);
        
        // Keep only last 1000 entries to prevent unbounded growth
        if state.metrics_history.len() > 1000 {
            let current_len = state.metrics_history.len();
            state.metrics_history.drain(..current_len - 1000);
        }
        
        if self.auto_save_enabled {
            drop(state);
            self.save_state_to_disk().await?;
        }
        
        Ok(())
    }
    
    /// Load system state after restart
    pub async fn load_system_state(&self) -> Result<SystemStateSnapshot, PersistenceError> {
        let content = fs::read_to_string(&self.state_file).await?;
        let mut state: SystemStateSnapshot = serde_json::from_str(&content)?;
        
        // Increment server start count
        state.server_start_count += 1;
        
        // Reset running bots to stopped (they need to be restarted manually)
        for bot_state in state.bots.values_mut() {
            if bot_state.status == BotStatus::Running {
                info!("🔄 Bot {} was running before restart, marking as stopped", bot_state.bot_id);
                bot_state.status = BotStatus::Stopped;
                bot_state.last_stopped_at = Some(Utc::now());
            }
        }
        
        *self.current_state.write().await = state.clone();
        
        info!("📋 Loaded {} persisted bot states", state.bots.len());
        info!("🔢 Server restart count: {}", state.server_start_count);
        
        Ok(state)
    }
    
    /// Get current system state
    pub async fn get_current_state(&self) -> SystemStateSnapshot {
        self.current_state.read().await.clone()
    }
    
    /// Get bot state
    pub async fn get_bot_state(&self, bot_id: Uuid) -> Option<PersistedBotState> {
        self.current_state.read().await.bots.get(&bot_id).cloned()
    }
    
    /// Get all bot states
    pub async fn get_all_bot_states(&self) -> HashMap<Uuid, PersistedBotState> {
        self.current_state.read().await.bots.clone()
    }
    
    /// Remove bot state
    pub async fn remove_bot_state(&self, bot_id: Uuid) -> Result<(), PersistenceError> {
        let mut state = self.current_state.write().await;
        state.bots.remove(&bot_id);
        
        if self.auto_save_enabled {
            drop(state);
            self.save_state_to_disk().await?;
        }
        
        Ok(())
    }
    
    /// Create backup of current state
    pub async fn create_backup(&self) -> Result<PathBuf, PersistenceError> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_file = self.backup_directory.join(format!("system_state_backup_{}.json", timestamp));
        
        let state = self.current_state.read().await;
        let content = serde_json::to_string_pretty(&*state)?;
        
        fs::write(&backup_file, content).await?;
        
        info!("💾 Created system state backup: {}", backup_file.display());
        Ok(backup_file)
    }
    
    /// Restore from backup
    pub async fn restore_from_backup<P: AsRef<Path>>(&self, backup_path: P) -> Result<(), PersistenceError> {
        let content = fs::read_to_string(backup_path).await?;
        let state: SystemStateSnapshot = serde_json::from_str(&content)?;
        
        // Create backup of current state before restore
        self.create_backup().await?;
        
        *self.current_state.write().await = state;
        self.save_state_to_disk().await?;
        
        info!("🔄 System state restored from backup");
        Ok(())
    }
    
    /// Get metrics history
    pub async fn get_metrics_history(&self, hours: u32) -> Vec<PersistedSystemMetrics> {
        let state = self.current_state.read().await;
        let cutoff_time = Utc::now() - chrono::Duration::hours(hours as i64);
        
        state.metrics_history
            .iter()
            .filter(|m| m.timestamp > cutoff_time)
            .cloned()
            .collect()
    }
    
    /// Force save current state to disk
    pub async fn force_save(&self) -> Result<(), PersistenceError> {
        self.save_state_to_disk().await
    }
    
    /// Enable/disable auto-save
    pub fn set_auto_save(&mut self, enabled: bool) {
        self.auto_save_enabled = enabled;
    }
    
    // Private methods
    
    async fn create_initial_state(&self) -> Result<(), PersistenceError> {
        let initial_state = SystemStateSnapshot::new();
        *self.current_state.write().await = initial_state;
        self.save_state_to_disk().await?;
        Ok(())
    }
    
    async fn save_state_to_disk(&self) -> Result<(), PersistenceError> {
        let state = self.current_state.read().await;
        
        // Create temporary file for atomic write
        let temp_file = self.state_file.with_extension("tmp");
        let content = serde_json::to_string_pretty(&*state)?;
        
        // Write to temp file then rename (atomic operation)
        fs::write(&temp_file, content).await?;
        fs::rename(&temp_file, &self.state_file).await?;
        
        Ok(())
    }
    
    async fn cleanup_old_backups(&self) -> Result<(), PersistenceError> {
        let cutoff_time = Utc::now() - chrono::Duration::days(self.backup_retention_days as i64);
        
        if !self.backup_directory.exists() {
            return Ok(());
        }
        
        let mut dir = fs::read_dir(&self.backup_directory).await?;
        let mut removed_count = 0;
        
        while let Some(entry) = dir.next_entry().await? {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.starts_with("system_state_backup_") && file_name.ends_with(".json") {
                    if let Ok(metadata) = entry.metadata().await {
                        if let Ok(modified) = metadata.modified() {
                            let modified_time = DateTime::<Utc>::from(modified);
                            if modified_time < cutoff_time {
                                if let Err(e) = fs::remove_file(entry.path()).await {
                                    warn!("Failed to remove old backup {}: {}", entry.path().display(), e);
                                } else {
                                    removed_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        if removed_count > 0 {
            info!("🧹 Cleaned up {} old backup files", removed_count);
        }
        
        Ok(())
    }
}

impl SystemStateSnapshot {
    fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            snapshot_timestamp: Utc::now(),
            server_start_count: 0,
            total_system_uptime_seconds: 0,
            bots: HashMap::new(),
            default_arbitrage_bot: None,
            metrics_history: Vec::new(),
            configuration_hashes: HashMap::new(),
        }
    }
}

impl PersistedBotState {
    pub fn from_runtime(
        bot_id: Uuid,
        bot_type: BotType,
        status: BotStatus,
        config: Option<BotConfig>,
        is_default: bool,
        metrics: Option<BotMetrics>,
    ) -> Self {
        let is_running = status == BotStatus::Running;
        Self {
            bot_id,
            bot_type,
            status,
            config,
            is_default,
            created_at: Utc::now(),
            last_started_at: if is_running { Some(Utc::now()) } else { None },
            last_stopped_at: None,
            total_uptime_seconds: 0,
            restart_count: 0,
            last_known_metrics: metrics,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_persistence_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StatePersistenceManager::new(temp_dir.path());
        
        assert!(manager.initialize().await.is_ok());
    }
    
    #[tokio::test]
    async fn test_bot_state_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StatePersistenceManager::new(temp_dir.path());
        manager.initialize().await.unwrap();
        
        let bot_id = Uuid::new_v4();
        let bot_state = PersistedBotState::from_runtime(
            bot_id,
            BotType::EnhancedArbitrage,
            BotStatus::Running,
            None,
            false,
            None,
        );
        
        manager.save_bot_state(bot_state.clone()).await.unwrap();
        
        let loaded_state = manager.get_bot_state(bot_id).await.unwrap();
        assert_eq!(loaded_state.bot_id, bot_id);
        assert_eq!(loaded_state.bot_type, BotType::EnhancedArbitrage);
    }
}
