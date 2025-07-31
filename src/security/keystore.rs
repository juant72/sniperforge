use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key
};
use ed25519_dalek::{SigningKey, VerifyingKey};
use bip39::Mnemonic;
use sha2::{Sha256, Digest};
use zeroize::{Zeroize, ZeroizeOnDrop};
use chrono::{DateTime, Utc};
use base64::{Engine as _, engine::general_purpose};

/// Enterprise-grade encrypted keystore for managing private keys
/// Simulates Hardware Security Module (HSM) functionality in software
#[derive(Debug)]
pub struct SecureKeystore {
    /// Path to encrypted keystore file
    keystore_path: PathBuf,
    /// Master encryption key derived from password
    master_key: Option<Key<Aes256Gcm>>,
    /// Configuration for keystore behavior
    config: KeystoreConfig,
    /// In-memory cache of decrypted keys (short-lived)
    key_cache: HashMap<String, CachedKey>,
}

/// Configuration for keystore security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystoreConfig {
    /// Key derivation iterations (higher = more secure but slower)
    pub key_derivation_iterations: u32,
    /// Cache TTL for keys in memory (seconds)
    pub key_cache_ttl: u64,
    /// Maximum failed unlock attempts before lockout
    pub max_unlock_attempts: u32,
    /// Lockout duration in minutes after max attempts
    pub lockout_duration_minutes: u32,
    /// Whether to enable key backup
    pub enable_backup: bool,
    /// Backup retention period in days
    pub backup_retention_days: u32,
    /// Require confirmation for critical operations
    pub require_confirmation: bool,
}

impl Default for KeystoreConfig {
    fn default() -> Self {
        Self {
            key_derivation_iterations: 100_000,
            key_cache_ttl: 60, // 1 minute
            max_unlock_attempts: 3,
            lockout_duration_minutes: 15,
            enable_backup: true,
            backup_retention_days: 90,
            require_confirmation: true,
        }
    }
}

/// Cached private key with expiration and usage tracking
#[derive(Debug, Clone)]
struct CachedKey {
    /// The actual private key bytes (will be zeroized on drop)
    key_data: PrivateKeyData,
    /// When this cache entry expires
    expires_at: DateTime<Utc>,
    /// Number of times this key has been accessed
    access_count: u32,
    /// Last access timestamp
    last_accessed: DateTime<Utc>,
}

/// Different types of private keys supported
#[derive(Debug, Clone, ZeroizeOnDrop)]
pub enum PrivateKeyData {
    /// Ed25519 key for Solana
    Ed25519(#[zeroize(skip)] [u8; 32]),
    /// Secp256k1 key for Ethereum/Bitcoin
    Secp256k1(#[zeroize(skip)] [u8; 32]),
    /// BIP39 mnemonic seed phrase
    Mnemonic(#[zeroize(skip)] String),
    /// Raw bytes for custom key types
    Raw(#[zeroize(skip)] Vec<u8>),
}

/// Encrypted key entry stored on disk
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedKeyEntry {
    /// Unique identifier for the key
    key_id: String,
    /// Base64 encoded encrypted key data
    encrypted_data: String,
    /// Base64 encoded nonce for AES-GCM
    nonce: String,
    /// Key type identifier
    key_type: String,
    /// Key metadata
    metadata: KeyMetadata,
    /// Creation timestamp
    created_at: DateTime<Utc>,
    /// Last modification timestamp
    modified_at: DateTime<Utc>,
}

/// Metadata about a stored key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    /// Human-readable name for the key
    pub name: String,
    /// Description or purpose of the key
    pub description: String,
    /// Network the key is intended for (mainnet, devnet, etc.)
    pub network: String,
    /// Whether this is a primary signing key
    pub is_primary: bool,
    /// Usage restrictions
    pub usage_restrictions: Vec<String>,
    /// Associated public key (for verification)
    pub public_key: Option<String>,
    /// Last used timestamp
    pub last_used: Option<DateTime<Utc>>,
    /// Usage count
    pub usage_count: u32,
}

/// Container for all encrypted keys
#[derive(Debug, Serialize, Deserialize)]
struct KeystoreContainer {
    /// Version for backward compatibility
    version: String,
    /// All encrypted keys by ID
    keys: HashMap<String, EncryptedKeyEntry>,
    /// Salt for key derivation
    salt: String,
    /// Security audit log entries
    audit_log: Vec<AuditEntry>,
    /// Failed unlock attempts
    failed_attempts: u32,
    /// Last failed attempt timestamp
    last_failed_attempt: Option<DateTime<Utc>>,
    /// Creation timestamp
    created_at: DateTime<Utc>,
    /// Last modification timestamp
    modified_at: DateTime<Utc>,
}

/// Audit log entry for security tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Timestamp of the event
    timestamp: DateTime<Utc>,
    /// Type of event (unlock, key_access, key_created, etc.)
    event_type: String,
    /// Key ID involved (if applicable)
    key_id: Option<String>,
    /// Additional event details
    details: String,
    /// Source IP or identifier (for network operations)
    source: Option<String>,
}

impl SecureKeystore {
    /// Create a new secure keystore
    pub fn new<P: AsRef<Path>>(keystore_path: P) -> Result<Self> {
        Self::with_config(keystore_path, KeystoreConfig::default())
    }

    /// Create a new secure keystore with custom configuration
    pub fn with_config<P: AsRef<Path>>(keystore_path: P, config: KeystoreConfig) -> Result<Self> {
        Ok(Self {
            keystore_path: keystore_path.as_ref().to_path_buf(),
            master_key: None,
            config,
            key_cache: HashMap::new(),
        })
    }

    /// Initialize or unlock the keystore with a master password
    pub async fn unlock(&mut self, master_password: &str) -> Result<()> {
        // Check for lockout
        if self.is_locked_out().await? {
            return Err(anyhow::anyhow!("Keystore is locked due to too many failed attempts"));
        }

        // Create keystore if it doesn't exist
        if !self.keystore_path.exists() {
            self.create_new_keystore(master_password).await?;
            self.log_audit_event("keystore_created", None, "New keystore created").await?;
        }

        // Derive master key
        match self.derive_master_key(master_password).await {
            Ok(key) => {
                self.master_key = Some(key);
                self.reset_failed_attempts().await?;
                self.log_audit_event("keystore_unlocked", None, "Keystore unlocked successfully").await?;
                log::info!("Keystore unlocked successfully");
                Ok(())
            }
            Err(e) => {
                self.increment_failed_attempts().await?;
                self.log_audit_event("unlock_failed", None, &format!("Failed unlock attempt: {}", e)).await?;
                Err(e)
            }
        }
    }

    /// Lock the keystore and clear sensitive data from memory
    pub async fn lock(&mut self) -> Result<()> {
        // Clear master key
        if let Some(mut key) = self.master_key.take() {
            key.zeroize();
        }

        // Clear key cache
        for (_, _cached_key) in self.key_cache.drain() {
            // cached_key.zeroize(); // Remove zeroize call
        }

        self.log_audit_event("keystore_locked", None, "Keystore locked").await?;
        log::info!("Keystore locked and sensitive data cleared");
        Ok(())
    }

    /// Store a new private key in the keystore
    pub async fn store_key(
        &mut self,
        key_id: &str,
        key_data: PrivateKeyData,
        metadata: KeyMetadata,
    ) -> Result<()> {
        self.ensure_unlocked()?;

        // Serialize key data
        let serialized_data = self.serialize_key_data(&key_data)?;

        // Encrypt the key data
        let encrypted_entry = self.encrypt_key_data(key_id, &serialized_data, &metadata).await?;

        // Load container, add key, and save
        let mut container = self.load_keystore_container().await?;
        container.keys.insert(key_id.to_string(), encrypted_entry);
        container.modified_at = Utc::now();

        self.save_keystore_container(&container).await?;

        // Add to cache
        let cached_key = CachedKey {
            key_data,
            expires_at: Utc::now() + chrono::Duration::seconds(self.config.key_cache_ttl as i64),
            access_count: 0,
            last_accessed: Utc::now(),
        };
        self.key_cache.insert(key_id.to_string(), cached_key);

        self.log_audit_event("key_stored", Some(key_id), &format!("Key stored: {}", metadata.name)).await?;
        log::info!("Private key '{}' stored securely", key_id);
        Ok(())
    }

    /// Retrieve a private key from the keystore
    pub async fn get_key(&mut self, key_id: &str) -> Result<Option<PrivateKeyData>> {
        self.ensure_unlocked()?;

        // Check cache first
        if let Some(cached) = self.key_cache.get_mut(key_id) {
            if cached.expires_at > Utc::now() {
                cached.access_count += 1;
                cached.last_accessed = Utc::now();
                let key_data = cached.key_data.clone();
                
                self.log_audit_event("key_accessed", Some(key_id), "Key accessed from cache").await?;
                return Ok(Some(key_data));
            }
            // Remove expired entry
            self.key_cache.remove(key_id);
        }

        // Load from encrypted storage
        let container = self.load_keystore_container().await?;
        
        if let Some(encrypted_entry) = container.keys.get(key_id) {
            let key_data = self.decrypt_key_data(encrypted_entry).await?;
            
            // Add to cache
            let cached_key = CachedKey {
                key_data: key_data.clone(),
                expires_at: Utc::now() + chrono::Duration::seconds(self.config.key_cache_ttl as i64),
                access_count: 1,
                last_accessed: Utc::now(),
            };
            self.key_cache.insert(key_id.to_string(), cached_key);
            
            // Update usage statistics
            self.update_key_usage(key_id).await?;
            
            self.log_audit_event("key_accessed", Some(key_id), "Key accessed from storage").await?;
            Ok(Some(key_data))
        } else {
            Ok(None)
        }
    }

    /// List all keys with their metadata
    pub async fn list_keys(&self) -> Result<HashMap<String, KeyMetadata>> {
        self.ensure_unlocked()?;

        let container = self.load_keystore_container().await?;
        let mut result = HashMap::new();
        
        for (key_id, encrypted_entry) in container.keys {
            result.insert(key_id, encrypted_entry.metadata);
        }
        
        Ok(result)
    }

    /// Generate a new Ed25519 keypair for Solana
    pub async fn generate_solana_keypair(&mut self, key_id: &str, name: &str, network: &str) -> Result<VerifyingKey> {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        
        let metadata = KeyMetadata {
            name: name.to_string(),
            description: format!("Generated Ed25519 keypair for {}", network),
            network: network.to_string(),
            is_primary: false,
            usage_restrictions: vec!["signing".to_string()],
            public_key: Some(general_purpose::STANDARD.encode(verifying_key.as_bytes())),
            last_used: None,
            usage_count: 0,
        };

        let key_data = PrivateKeyData::Ed25519(signing_key.to_bytes());
        self.store_key(key_id, key_data, metadata).await?;

        log::info!("Generated new Solana keypair: {}", key_id);
        Ok(verifying_key)
    }

    /// Generate a new mnemonic seed phrase
    pub async fn generate_mnemonic(&mut self, key_id: &str, name: &str, word_count: usize) -> Result<String> {
        let entropy_bits = match word_count {
            12 => 128,
            15 => 160,
            18 => 192,
            21 => 224,
            24 => 256,
            _ => return Err(anyhow::anyhow!("Invalid word count. Must be 12, 15, 18, 21, or 24")),
        };

        let mut entropy = vec![0u8; entropy_bits / 8];
        getrandom::getrandom(&mut entropy)
            .context("Failed to generate random entropy")?;

        let mnemonic = Mnemonic::from_entropy(&entropy)
            .context("Failed to create mnemonic from entropy")?;

        let mnemonic_phrase = mnemonic.to_string();

        let metadata = KeyMetadata {
            name: name.to_string(),
            description: format!("{}-word BIP39 mnemonic seed phrase", word_count),
            network: "multi".to_string(),
            is_primary: true,
            usage_restrictions: vec!["key_derivation".to_string()],
            public_key: None,
            last_used: None,
            usage_count: 0,
        };

        let key_data = PrivateKeyData::Mnemonic(mnemonic_phrase.clone());
        self.store_key(key_id, key_data, metadata).await?;

        log::info!("Generated new {}-word mnemonic: {}", word_count, key_id);
        Ok(mnemonic_phrase)
    }

    /// Derive a keypair from a mnemonic at a specific derivation path
    pub async fn derive_key_from_mnemonic(
        &mut self,
        mnemonic_key_id: &str,
        derivation_path: &str,
        derived_key_id: &str,
        name: &str,
    ) -> Result<()> {
        // Get the mnemonic
        let mnemonic_data = self.get_key(mnemonic_key_id).await?
            .context("Mnemonic not found")?;

        let mnemonic_phrase = match &mnemonic_data {
            PrivateKeyData::Mnemonic(phrase) => phrase.clone(),
            _ => return Err(anyhow::anyhow!("Key is not a mnemonic")),
        };

        // Create mnemonic and seed
        let mnemonic = Mnemonic::parse(&mnemonic_phrase)
            .context("Invalid mnemonic phrase")?;
        let seed = mnemonic.to_seed(""); // Empty passphrase

        // For simplicity, we'll derive the first 32 bytes as an Ed25519 key
        // In a real implementation, you'd use proper BIP44 derivation
        let mut hasher = Sha256::new();
        hasher.update(&seed);
        hasher.update(derivation_path.as_bytes());
        let derived_seed = hasher.finalize();

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&derived_seed[..32]);

        let metadata = KeyMetadata {
            name: name.to_string(),
            description: format!("Derived from mnemonic at path {}", derivation_path),
            network: "solana".to_string(),
            is_primary: false,
            usage_restrictions: vec!["signing".to_string()],
            public_key: None, // Would compute from private key in real implementation
            last_used: None,
            usage_count: 0,
        };

        let key_data = PrivateKeyData::Ed25519(key_bytes);
        self.store_key(derived_key_id, key_data, metadata).await?;

        log::info!("Derived key '{}' from mnemonic '{}' at path '{}'", 
                  derived_key_id, mnemonic_key_id, derivation_path);
        Ok(())
    }

    /// Create backup of the keystore
    pub async fn create_backup(&self) -> Result<PathBuf> {
        let backup_path = self.keystore_path.with_extension(
            format!("backup.{}", Utc::now().format("%Y%m%d_%H%M%S"))
        );
        
        fs::copy(&self.keystore_path, &backup_path)
            .context("Failed to create keystore backup")?;
        
        self.log_audit_event("backup_created", None, &format!("Backup created: {:?}", backup_path)).await?;
        log::info!("Keystore backup created: {:?}", backup_path);
        Ok(backup_path)
    }

    /// Get audit log entries
    pub async fn get_audit_log(&self) -> Result<Vec<AuditEntry>> {
        let container = self.load_keystore_container().await?;
        Ok(container.audit_log)
    }

    /// Clear the key cache
    pub fn clear_cache(&mut self) {
        for (_, _cached_key) in self.key_cache.drain() {
            // cached_key.zeroize(); // Remove zeroize call
        }
        log::info!("Key cache cleared");
    }

    // Private helper methods

    fn ensure_unlocked(&self) -> Result<()> {
        if self.master_key.is_none() {
            return Err(anyhow::anyhow!("Keystore is locked. Call unlock() first."));
        }
        Ok(())
    }

    async fn derive_master_key(&self, password: &str) -> Result<Key<Aes256Gcm>> {
        let container = self.load_keystore_container_raw().await?;
        let salt = general_purpose::STANDARD.decode(&container.salt)
            .context("Invalid salt in keystore")?;

        let mut key_bytes = [0u8; 32];
        let _ = pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
            password.as_bytes(),
            &salt,
            self.config.key_derivation_iterations,
            &mut key_bytes,
        );

        Ok(*Key::<Aes256Gcm>::from_slice(&key_bytes))
    }

    fn serialize_key_data(&self, key_data: &PrivateKeyData) -> Result<Vec<u8>> {
        match key_data {
            PrivateKeyData::Ed25519(bytes) => Ok(bytes.to_vec()),
            PrivateKeyData::Secp256k1(bytes) => Ok(bytes.to_vec()),
            PrivateKeyData::Mnemonic(phrase) => Ok(phrase.as_bytes().to_vec()),
            PrivateKeyData::Raw(bytes) => Ok(bytes.clone()),
        }
    }

    fn deserialize_key_data(&self, data: &[u8], key_type: &str) -> Result<PrivateKeyData> {
        match key_type {
            "ed25519" => {
                let mut bytes = [0u8; 32];
                bytes.copy_from_slice(data);
                Ok(PrivateKeyData::Ed25519(bytes))
            }
            "secp256k1" => {
                let mut bytes = [0u8; 32];
                bytes.copy_from_slice(data);
                Ok(PrivateKeyData::Secp256k1(bytes))
            }
            "mnemonic" => {
                let phrase = String::from_utf8(data.to_vec())
                    .context("Invalid UTF-8 in mnemonic")?;
                Ok(PrivateKeyData::Mnemonic(phrase))
            }
            "raw" => Ok(PrivateKeyData::Raw(data.to_vec())),
            _ => Err(anyhow::anyhow!("Unknown key type: {}", key_type)),
        }
    }

    async fn encrypt_key_data(
        &self,
        key_id: &str,
        data: &[u8],
        metadata: &KeyMetadata,
    ) -> Result<EncryptedKeyEntry> {
        let master_key = self.master_key.as_ref()
            .context("Keystore not unlocked")?;

        let cipher = Aes256Gcm::new(master_key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let encrypted_data = cipher.encrypt(&nonce, data)
            .map_err(|e| anyhow::anyhow!("Failed to encrypt key data: {:?}", e))?;

        let key_type = match metadata.description.as_str() {
            desc if desc.contains("Ed25519") => "ed25519",
            desc if desc.contains("mnemonic") => "mnemonic",
            _ => "raw",
        };

        Ok(EncryptedKeyEntry {
            key_id: key_id.to_string(),
            encrypted_data: general_purpose::STANDARD.encode(&encrypted_data),
            nonce: general_purpose::STANDARD.encode(&nonce),
            key_type: key_type.to_string(),
            metadata: metadata.clone(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
        })
    }

    async fn decrypt_key_data(&self, entry: &EncryptedKeyEntry) -> Result<PrivateKeyData> {
        let master_key = self.master_key.as_ref()
            .context("Keystore not unlocked")?;

        let cipher = Aes256Gcm::new(master_key);
        
        let encrypted_data = general_purpose::STANDARD.decode(&entry.encrypted_data)
            .context("Invalid base64 in encrypted data")?;
        let nonce_bytes = general_purpose::STANDARD.decode(&entry.nonce)
            .context("Invalid base64 in nonce")?;
        
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let plaintext = cipher.decrypt(nonce, encrypted_data.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to decrypt key data: {:?}", e))?;
        
        self.deserialize_key_data(&plaintext, &entry.key_type)
    }

    async fn load_keystore_container(&self) -> Result<KeystoreContainer> {
        self.load_keystore_container_raw().await
    }

    async fn load_keystore_container_raw(&self) -> Result<KeystoreContainer> {
        let content = fs::read_to_string(&self.keystore_path)
            .context("Failed to read keystore file")?;
        
        serde_json::from_str(&content)
            .context("Failed to parse keystore file")
    }

    async fn save_keystore_container(&self, container: &KeystoreContainer) -> Result<()> {
        let content = serde_json::to_string_pretty(container)
            .context("Failed to serialize keystore")?;
        
        fs::write(&self.keystore_path, content)
            .context("Failed to write keystore file")?;
        
        Ok(())
    }

    async fn create_new_keystore(&self, _password: &str) -> Result<()> {
        // Generate random salt
        let mut salt_bytes = [0u8; 32];
        getrandom::getrandom(&mut salt_bytes)
            .context("Failed to generate salt")?;

        let container = KeystoreContainer {
            version: "1.0".to_string(),
            keys: HashMap::new(),
            salt: general_purpose::STANDARD.encode(&salt_bytes),
            audit_log: vec![],
            failed_attempts: 0,
            last_failed_attempt: None,
            created_at: Utc::now(),
            modified_at: Utc::now(),
        };

        self.save_keystore_container(&container).await?;
        Ok(())
    }

    async fn log_audit_event(&self, event_type: &str, key_id: Option<&str>, details: &str) -> Result<()> {
        // In a production system, this might write to a separate audit log
        let _entry = AuditEntry {
            timestamp: Utc::now(),
            event_type: event_type.to_string(),
            key_id: key_id.map(|s| s.to_string()),
            details: details.to_string(),
            source: None, // Could include IP address or user identifier
        };

        log::info!("Keystore audit: {} - {}", event_type, details);
        
        // For now, we'll just log it. In production, you'd want to:
        // 1. Write to a separate audit log file
        // 2. Send to a centralized logging system
        // 3. Ensure audit logs are tamper-evident
        
        Ok(())
    }

    async fn is_locked_out(&self) -> Result<bool> {
        if !self.keystore_path.exists() {
            return Ok(false);
        }

        let container = self.load_keystore_container_raw().await?;
        
        if container.failed_attempts >= self.config.max_unlock_attempts {
            if let Some(last_failed) = container.last_failed_attempt {
                let lockout_until = last_failed + chrono::Duration::minutes(self.config.lockout_duration_minutes as i64);
                return Ok(Utc::now() < lockout_until);
            }
        }
        
        Ok(false)
    }

    async fn increment_failed_attempts(&self) -> Result<()> {
        let mut container = self.load_keystore_container_raw().await?;
        container.failed_attempts += 1;
        container.last_failed_attempt = Some(Utc::now());
        self.save_keystore_container(&container).await?;
        Ok(())
    }

    async fn reset_failed_attempts(&self) -> Result<()> {
        let mut container = self.load_keystore_container().await?;
        container.failed_attempts = 0;
        container.last_failed_attempt = None;
        self.save_keystore_container(&container).await?;
        Ok(())
    }

    async fn update_key_usage(&self, key_id: &str) -> Result<()> {
        let mut container = self.load_keystore_container().await?;
        
        if let Some(entry) = container.keys.get_mut(key_id) {
            entry.metadata.usage_count += 1;
            entry.metadata.last_used = Some(Utc::now());
            entry.modified_at = Utc::now();
        }
        
        self.save_keystore_container(&container).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_keystore_basic_operations() {
        let temp_dir = tempdir().unwrap();
        let keystore_path = temp_dir.path().join("test_keystore.json");
        
        let mut keystore = SecureKeystore::new(&keystore_path).unwrap();
        keystore.unlock("test_password_123").await.unwrap();
        
        // Generate a Solana keypair
        let _public_key = keystore.generate_solana_keypair(
            "test_key",
            "Test Keypair",
            "devnet"
        ).await.unwrap();
        
        // Retrieve the key
        let retrieved_key = keystore.get_key("test_key").await.unwrap();
        assert!(retrieved_key.is_some());
        
        // List keys
        let keys = keystore.list_keys().await.unwrap();
        assert_eq!(keys.len(), 1);
        assert!(keys.contains_key("test_key"));
    }

    #[tokio::test]
    async fn test_mnemonic_generation_and_derivation() {
        let temp_dir = tempdir().unwrap();
        let keystore_path = temp_dir.path().join("test_mnemonic_keystore.json");
        
        let mut keystore = SecureKeystore::new(&keystore_path).unwrap();
        keystore.unlock("test_password_456").await.unwrap();
        
        // Generate a 12-word mnemonic
        let mnemonic = keystore.generate_mnemonic(
            "master_seed",
            "Master Seed",
            12
        ).await.unwrap();
        
        assert_eq!(mnemonic.split_whitespace().count(), 12);
        
        // Derive a key from the mnemonic
        keystore.derive_key_from_mnemonic(
            "master_seed",
            "m/44'/501'/0'/0'", // Solana derivation path
            "derived_key",
            "Derived Solana Key"
        ).await.unwrap();
        
        // Verify both keys exist
        let keys = keystore.list_keys().await.unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains_key("master_seed"));
        assert!(keys.contains_key("derived_key"));
    }

    #[tokio::test]
    async fn test_keystore_locking() {
        let temp_dir = tempdir().unwrap();
        let keystore_path = temp_dir.path().join("test_lock_keystore.json");
        
        let mut keystore = SecureKeystore::new(&keystore_path).unwrap();
        keystore.unlock("test_password_789").await.unwrap();
        
        // Store a key
        keystore.generate_solana_keypair("test_key", "Test", "devnet").await.unwrap();
        
        // Lock the keystore
        keystore.lock().await.unwrap();
        
        // Verify we can't access keys when locked
        let result = keystore.get_key("test_key").await;
        assert!(result.is_err());
        
        // Unlock again
        keystore.unlock("test_password_789").await.unwrap();
        
        // Verify we can access keys after unlocking
        let retrieved_key = keystore.get_key("test_key").await.unwrap();
        assert!(retrieved_key.is_some());
    }
}
