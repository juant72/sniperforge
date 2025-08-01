//! # Enterprise Security Framework
//! 
//! This module provides a comprehensive enterprise-grade security framework for SniperForge,
//! including secrets management, keystore operations, input validation, authentication,
//! authorization, and encryption services.
//! 
//! ## Components
//! 
//! - **Secrets Management**: Secure storage and retrieval of API keys, private keys, and other sensitive data
//! - **Keystore**: Hardware Security Module (HSM) simulation for cryptographic key management
//! - **Input Validation**: Comprehensive validation and sanitization of all external inputs
//! - **Authentication & Authorization**: JWT-based authentication with role-based access control
//! - **Encryption**: Military-grade encryption for data at rest and in transit
//! - **Risk Management**: Trading risk assessment and management
//! 
//! ## Security Features
//! 
//! - AES-256-GCM and ChaCha20-Poly1305 encryption
//! - Argon2 password hashing
//! - JWT tokens with configurable expiration
//! - Rate limiting for authentication attempts
//! - Comprehensive audit logging
//! - Memory protection and secure deletion
//! - Input validation and sanitization
//! - Multi-factor authentication support

pub mod secrets;
pub mod keystore;
pub mod validation;
pub mod auth;
pub mod encryption;
pub mod risk_manager;
pub mod wallet;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

// Re-export commonly used types
pub use secrets::{SecretsManager, SecretsConfig};
pub use keystore::{SecureKeystore, KeystoreConfig, KeyMetadata, PrivateKeyData};
pub use validation::{InputValidator, ValidationConfig, ValidationResult, ValidationError};
pub use auth::{AuthenticationSystem, AuthConfig, AuthResult, UserInfo, AccountStatus};
pub use encryption::{
    EncryptionSystem, EncryptionConfig, EncryptedData, EncryptionAlgorithm, 
    EncryptionMetrics, SecureData
};
pub use risk_manager::*;
pub use wallet::{WalletManager, WalletConfig, WalletType, WalletInfo, ManagedWallet, RiskManagement};

/// Enterprise Security Framework
/// 
/// This is the main entry point for all security operations in SniperForge.
/// It provides a unified interface to all security components and ensures
/// they work together securely.
pub struct SecurityFramework {
    /// Secrets management system
    pub secrets: SecretsManager,
    /// Secure keystore for private keys
    pub keystore: SecureKeystore,
    /// Input validation system
    pub validator: InputValidator,
    /// Authentication and authorization system
    pub auth: AuthenticationSystem,
    /// Encryption and data protection system
    pub encryption: EncryptionSystem,
    /// Security configuration
    config: SecurityFrameworkConfig,
    /// Security audit log
    audit_log: Vec<SecurityAuditEntry>,
    /// Security metrics
    metrics: SecurityMetrics,
}

/// Configuration for the security framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFrameworkConfig {
    /// Path to secrets file
    pub secrets_path: String,
    /// Path to keystore file
    pub keystore_path: String,
    /// Whether to enable comprehensive audit logging
    pub enable_audit_logging: bool,
    /// Whether to enable strict security mode
    pub strict_mode: bool,
    /// Security event retention period in days
    pub audit_retention_days: u32,
    /// Whether to enable automatic security monitoring
    pub enable_monitoring: bool,
    /// Secrets management configuration
    pub secrets_config: SecretsConfig,
    /// Keystore configuration
    pub keystore_config: KeystoreConfig,
    /// Validation configuration
    pub validation_config: ValidationConfig,
    /// Authentication configuration
    pub auth_config: AuthConfig,
    /// Encryption configuration
    pub encryption_config: EncryptionConfig,
}

impl Default for SecurityFrameworkConfig {
    fn default() -> Self {
        Self {
            secrets_path: "./secure/secrets.json".to_string(),
            keystore_path: "./secure/keystore.json".to_string(),
            enable_audit_logging: true,
            strict_mode: true,
            audit_retention_days: 90,
            enable_monitoring: true,
            secrets_config: SecretsConfig::default(),
            keystore_config: KeystoreConfig::default(),
            validation_config: ValidationConfig::default(),
            auth_config: AuthConfig::default(),
            encryption_config: EncryptionConfig::default(),
        }
    }
}

/// Security audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditEntry {
    /// Timestamp of the security event
    pub timestamp: DateTime<Utc>,
    /// Type of security event
    pub event_type: SecurityEventType,
    /// Component that generated the event
    pub component: String,
    /// Event severity level
    pub severity: SecuritySeverity,
    /// Detailed event description
    pub description: String,
    /// Additional event metadata
    pub metadata: HashMap<String, String>,
    /// User or system that triggered the event
    pub actor: Option<String>,
    /// IP address (if applicable)
    pub ip_address: Option<String>,
}

/// Types of security events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SecurityEventType {
    /// Authentication events
    Authentication,
    /// Authorization events
    Authorization,
    /// Encryption/decryption events
    Encryption,
    /// Key management events
    KeyManagement,
    /// Secret access events
    SecretAccess,
    /// Input validation events
    Validation,
    /// Security policy violations
    PolicyViolation,
    /// Audit events
    Audit,
    /// System security events
    System,
}

/// Security event severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SecuritySeverity {
    /// Informational events
    Info,
    /// Warning events
    Warning,
    /// Error events
    Error,
    /// Critical security events
    Critical,
}

/// Security metrics and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    /// Total number of security events
    pub total_events: u64,
    /// Events by type
    pub events_by_type: HashMap<SecurityEventType, u64>,
    /// Events by severity
    pub events_by_severity: HashMap<SecuritySeverity, u64>,
    /// Last security event timestamp
    pub last_event: Option<DateTime<Utc>>,
    /// Security framework uptime
    pub uptime_seconds: u64,
    /// Number of failed authentication attempts
    pub failed_auth_attempts: u64,
    /// Number of validation failures
    pub validation_failures: u64,
    /// Number of encryption operations
    pub encryption_operations: u64,
    /// Number of key operations
    pub key_operations: u64,
}

impl Default for SecurityMetrics {
    fn default() -> Self {
        Self {
            total_events: 0,
            events_by_type: HashMap::new(),
            events_by_severity: HashMap::new(),
            last_event: None,
            uptime_seconds: 0,
            failed_auth_attempts: 0,
            validation_failures: 0,
            encryption_operations: 0,
            key_operations: 0,
        }
    }
}

impl SecurityFramework {
    /// Create a new security framework with default configuration
    pub async fn new() -> Result<Self> {
        Self::with_config(SecurityFrameworkConfig::default()).await
    }

    /// Create a new security framework with custom configuration
    pub async fn with_config(config: SecurityFrameworkConfig) -> Result<Self> {
        // Ensure secure directory exists
        if let Some(parent) = std::path::Path::new(&config.secrets_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        if let Some(parent) = std::path::Path::new(&config.keystore_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Initialize all security components
        let secrets = SecretsManager::with_config(&config.secrets_path, config.secrets_config.clone())
            .context("Failed to initialize secrets manager")?;

        let keystore = SecureKeystore::with_config(&config.keystore_path, config.keystore_config.clone())
            .context("Failed to initialize secure keystore")?;

        let validator = InputValidator::with_config(config.validation_config.clone())
            .context("Failed to initialize input validator")?;

        let auth = AuthenticationSystem::with_config(config.auth_config.clone())
            .context("Failed to initialize authentication system")?;

        let encryption = EncryptionSystem::with_config(config.encryption_config.clone())
            .context("Failed to initialize encryption system")?;

        let mut framework = Self {
            secrets,
            keystore,
            validator,
            auth,
            encryption,
            config,
            audit_log: Vec::new(),
            metrics: SecurityMetrics::default(),
        };

        // Log framework initialization
        framework.log_security_event(
            SecurityEventType::System,
            SecuritySeverity::Info,
            "SecurityFramework",
            "Security framework initialized successfully",
            None,
            None,
        ).await?;

        Ok(framework)
    }

    /// Initialize the security framework with master credentials
    pub async fn initialize(&mut self, master_password: &str) -> Result<()> {
        // Initialize secrets manager
        self.secrets.initialize(master_password).await
            .context("Failed to initialize secrets manager")?;

        // Initialize keystore
        self.keystore.unlock(master_password).await
            .context("Failed to unlock keystore")?;

        // Initialize encryption system
        self.encryption.initialize(master_password).await
            .context("Failed to initialize encryption system")?;

        // Log successful initialization
        self.log_security_event(
            SecurityEventType::System,
            SecuritySeverity::Info,
            "SecurityFramework",
            "Security framework initialized with master credentials",
            None,
            None,
        ).await?;

        log::info!("SecurityFramework: All components initialized successfully");
        Ok(())
    }

    /// Store an API key securely
    pub async fn store_api_key(&mut self, name: &str, api_key: &str, service: &str) -> Result<()> {
        self.secrets.store_api_key(name, api_key, service).await?;
        
        self.log_security_event(
            SecurityEventType::SecretAccess,
            SecuritySeverity::Info,
            "SecretsManager",
            &format!("API key stored for service: {}", service),
            Some(vec![("key_name".to_string(), name.to_string())].into_iter().collect()),
            None,
        ).await?;

        Ok(())
    }

    /// Retrieve an API key securely
    pub async fn get_api_key(&mut self, service: &str) -> Result<Option<String>> {
        let result = self.secrets.get_api_key(service).await?;
        
        self.log_security_event(
            SecurityEventType::SecretAccess,
            SecuritySeverity::Info,
            "SecretsManager",
            &format!("API key accessed for service: {}", service),
            Some(vec![("service".to_string(), service.to_string())].into_iter().collect()),
            None,
        ).await?;

        Ok(result)
    }

    /// Validate a Solana address
    pub async fn validate_solana_address(&mut self, address: &str) -> Result<ValidationResult> {
        let result = self.validator.validate_solana_address(address).await?;
        
        if !result.is_valid {
            self.log_security_event(
                SecurityEventType::Validation,
                SecuritySeverity::Warning,
                "InputValidator",
                &format!("Invalid Solana address validation: {}", address),
                Some(vec![("address".to_string(), address.to_string())].into_iter().collect()),
                None,
            ).await?;
            
            self.metrics.validation_failures += 1;
        }

        Ok(result)
    }

    /// Validate price data from external APIs
    pub async fn validate_price_data(&mut self, price: &str, source: &str) -> Result<ValidationResult> {
        let result = self.validator.validate_price_data(price, source).await?;
        
        if !result.is_valid {
            self.log_security_event(
                SecurityEventType::Validation,
                SecuritySeverity::Warning,
                "InputValidator",
                &format!("Invalid price data from {}: {}", source, price),
                Some(vec![
                    ("price".to_string(), price.to_string()),
                    ("source".to_string(), source.to_string()),
                ].into_iter().collect()),
                None,
            ).await?;
            
            self.metrics.validation_failures += 1;
        }

        Ok(result)
    }

    /// Get comprehensive security metrics
    pub fn get_security_metrics(&self) -> SecurityMetrics {
        let mut metrics = self.metrics.clone();
        
        // Add component-specific metrics
        let encryption_metrics = self.encryption.get_metrics();
        metrics.encryption_operations = encryption_metrics.encrypt_operations + encryption_metrics.decrypt_operations;
        
        metrics
    }

    /// Get security audit log
    pub fn get_audit_log(&self) -> &[SecurityAuditEntry] {
        &self.audit_log
    }

    // Private helper methods

    async fn log_security_event(
        &mut self,
        event_type: SecurityEventType,
        severity: SecuritySeverity,
        component: &str,
        description: &str,
        metadata: Option<HashMap<String, String>>,
        ip_address: Option<String>,
    ) -> Result<()> {
        let entry = SecurityAuditEntry {
            timestamp: Utc::now(),
            event_type: event_type.clone(),
            component: component.to_string(),
            severity: severity.clone(),
            description: description.to_string(),
            metadata: metadata.unwrap_or_default(),
            actor: None, // Would be set in a real implementation
            ip_address,
        };

        if self.config.enable_audit_logging {
            self.audit_log.push(entry);
        }

        // Update metrics
        self.metrics.total_events += 1;
        *self.metrics.events_by_type.entry(event_type).or_insert(0) += 1;
        *self.metrics.events_by_severity.entry(severity).or_insert(0) += 1;
        self.metrics.last_event = Some(Utc::now());

        Ok(())
    }
}
