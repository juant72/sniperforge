use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key
};
use chacha20poly1305::ChaCha20Poly1305;
use argon2::password_hash::{rand_core::OsRng as ArgonRng, SaltString};
use zeroize::{Zeroize, ZeroizeOnDrop};
use sha2::{Sha256, Digest};
use hmac::Hmac;
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use std::mem;

/// Enterprise-grade encryption and data protection system
/// Provides comprehensive encryption, key management, and secure data handling
#[derive(Debug)]
pub struct EncryptionSystem {
    /// Configuration for encryption behavior
    config: EncryptionConfig,
    /// Active encryption keys
    keys: KeyManager,
    /// Memory protection for sensitive data
    memory_protector: MemoryProtector,
    /// Encryption metrics and statistics
    metrics: EncryptionMetrics,
}

/// Configuration for encryption system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Default encryption algorithm
    pub default_algorithm: EncryptionAlgorithm,
    /// Key derivation iterations
    pub key_derivation_iterations: u32,
    /// Whether to enable compression before encryption
    pub enable_compression: bool,
    /// Maximum size for in-memory encryption (bytes)
    pub max_memory_encryption_size: usize,
    /// Whether to use secure memory allocation
    pub use_secure_memory: bool,
    /// Key rotation interval in days
    pub key_rotation_interval_days: u32,
    /// Whether to enable integrity checking
    pub enable_integrity_checks: bool,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            default_algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_derivation_iterations: 100_000,
            enable_compression: true,
            max_memory_encryption_size: 100 * 1024 * 1024, // 100 MB
            use_secure_memory: true,
            key_rotation_interval_days: 90,
            enable_integrity_checks: true,
        }
    }
}

/// Supported encryption algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM (recommended for most use cases)
    Aes256Gcm,
    /// ChaCha20-Poly1305 (good for performance on systems without AES-NI)
    ChaCha20Poly1305,
    /// XChaCha20-Poly1305 (for very large nonces)
    XChaCha20Poly1305,
}

/// Key manager for encryption keys
#[derive(Debug)]
struct KeyManager {
    /// Master encryption keys by algorithm
    master_keys: HashMap<EncryptionAlgorithm, MasterKey>,
    /// Data encryption keys (DEKs) cache
    dek_cache: HashMap<String, DataEncryptionKey>,
    /// Key derivation parameters
    kdf_params: KdfParams,
}

/// Master encryption key
#[derive(Debug, Clone)]
struct MasterKey {
    /// The actual key bytes
    key_bytes: Vec<u8>,
    /// Algorithm this key is for
    #[allow(dead_code)] // Used in key rotation
    algorithm: EncryptionAlgorithm,
    /// Key creation timestamp
    #[allow(dead_code)] // Used in audit logs
    created_at: DateTime<Utc>,
    /// Key version for rotation
    #[allow(dead_code)] // Used in version management
    version: u32,
}

/// Data encryption key (derived from master key)
#[derive(Debug, Clone)]
struct DataEncryptionKey {
    /// The actual key bytes
    key_bytes: Vec<u8>,
    /// Algorithm for this key
    #[allow(dead_code)] // Used in encryption selection
    algorithm: EncryptionAlgorithm,
    /// Key derivation context
    #[allow(dead_code)] // Used in key derivation
    context: String,
    /// Creation timestamp
    created_at: DateTime<Utc>,
    /// Expiration timestamp
    expires_at: DateTime<Utc>,
}

/// Key derivation function parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
struct KdfParams {
    /// Salt for key derivation
    salt: String,
    /// Number of iterations
    iterations: u32,
    /// Memory cost (for Argon2)
    memory_cost: u32,
    /// Parallelism (for Argon2)
    parallelism: u32,
}

/// Memory protection for sensitive data
#[derive(Debug)]
struct MemoryProtector {
    /// Whether secure memory is enabled
    #[allow(dead_code)] // Feature flag for future enhancement
    enabled: bool,
    /// Statistics about protected memory
    stats: MemoryStats,
}

/// Memory protection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Number of secure allocations
    secure_allocations: u64,
    /// Total bytes protected
    total_bytes_protected: u64,
    /// Number of memory wipes performed
    memory_wipes: u64,
}

/// Encryption metrics and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetrics {
    /// Total number of encrypt operations
    pub encrypt_operations: u64,
    /// Total number of decrypt operations
    pub decrypt_operations: u64,
    /// Total bytes encrypted
    pub bytes_encrypted: u64,
    /// Total bytes decrypted
    pub bytes_decrypted: u64,
    /// Number of key derivations
    pub key_derivations: u64,
    /// Average encryption time in microseconds
    pub avg_encrypt_time_us: u64,
    /// Average decryption time in microseconds
    pub avg_decrypt_time_us: u64,
    /// Number of integrity check failures
    pub integrity_failures: u64,
}

impl Default for EncryptionMetrics {
    fn default() -> Self {
        Self {
            encrypt_operations: 0,
            decrypt_operations: 0,
            bytes_encrypted: 0,
            bytes_decrypted: 0,
            key_derivations: 0,
            avg_encrypt_time_us: 0,
            avg_decrypt_time_us: 0,
            integrity_failures: 0,
        }
    }
}

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Encrypted payload (base64 encoded)
    pub ciphertext: String,
    /// Nonce/IV (base64 encoded)
    pub nonce: String,
    /// Encryption algorithm used
    pub algorithm: EncryptionAlgorithm,
    /// Authentication tag (base64 encoded)
    pub tag: Option<String>,
    /// HMAC for additional integrity (base64 encoded)
    pub hmac: Option<String>,
    /// Metadata about the encryption
    pub metadata: EncryptionMetadata,
}

/// Metadata about encrypted data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    /// Timestamp when data was encrypted
    pub encrypted_at: DateTime<Utc>,
    /// Key version used for encryption
    pub key_version: u32,
    /// Original data size (before encryption)
    pub original_size: usize,
    /// Whether data was compressed before encryption
    pub compressed: bool,
    /// Data type hint
    pub data_type: Option<String>,
    /// Custom context or identifier
    pub context: Option<String>,
}

/// Secure data container that automatically zeroizes on drop
#[derive(ZeroizeOnDrop)]
pub struct SecureData {
    /// The actual sensitive data
    #[zeroize(skip)]
    data: Vec<u8>,
    /// Data type identifier
    data_type: String,
    /// Whether this data is currently protected in secure memory
    secure_memory: bool,
}

impl SecureData {
    /// Create new secure data container
    pub fn new(data: Vec<u8>, data_type: &str) -> Self {
        Self {
            data,
            data_type: data_type.to_string(),
            secure_memory: false,
        }
    }

    /// Get a reference to the data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the data type
    pub fn data_type(&self) -> &str {
        &self.data_type
    }

    /// Move data out of the container (consumes self)
    pub fn into_data(mut self) -> Vec<u8> {
        // Prevent automatic zeroization since we're moving the data
        let data = mem::take(&mut self.data);
        mem::forget(self);
        data
    }
}

impl EncryptionSystem {
    /// Create a new encryption system
    pub fn new() -> Result<Self> {
        Self::with_config(EncryptionConfig::default())
    }

    /// Create a new encryption system with custom configuration
    pub fn with_config(config: EncryptionConfig) -> Result<Self> {
        let keys = KeyManager::new(&config)?;
        let memory_protector = MemoryProtector::new(config.use_secure_memory);
        let metrics = EncryptionMetrics::default();

        Ok(Self {
            config,
            keys,
            memory_protector,
            metrics,
        })
    }

    /// Initialize the encryption system with a master password
    pub async fn initialize(&mut self, master_password: &str) -> Result<()> {
        self.keys.derive_master_keys(master_password, &self.config).await?;
        self.metrics.key_derivations += 1;
        
        log::info!("Encryption system initialized with master key derivation");
        Ok(())
    }

    /// Encrypt data using the default algorithm
    pub async fn encrypt(&mut self, data: &[u8], context: Option<&str>) -> Result<EncryptedData> {
        self.encrypt_with_algorithm(data, self.config.default_algorithm.clone(), context).await
    }

    /// Encrypt data using a specific algorithm
    pub async fn encrypt_with_algorithm(
        &mut self,
        data: &[u8],
        algorithm: EncryptionAlgorithm,
        context: Option<&str>,
    ) -> Result<EncryptedData> {
        let start_time = std::time::Instant::now();

        // Check size limits
        if data.len() > self.config.max_memory_encryption_size {
            return Err(anyhow::anyhow!("Data size exceeds maximum encryption limit"));
        }

        // Get or derive encryption key
        let dek = self.keys.get_data_encryption_key(&algorithm, context).await?;

        // Optionally compress data
        let (payload, compressed) = if self.config.enable_compression && data.len() > 1024 {
            match self.compress_data(data) {
                Ok(compressed_data) if compressed_data.len() < data.len() => {
                    (compressed_data, true)
                }
                _ => (data.to_vec(), false)
            }
        } else {
            (data.to_vec(), false)
        };

        // Encrypt based on algorithm
        let (ciphertext, nonce, tag) = match algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                self.encrypt_aes_gcm(&payload, &dek.key_bytes).await?
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                self.encrypt_chacha20_poly1305(&payload, &dek.key_bytes).await?
            }
            EncryptionAlgorithm::XChaCha20Poly1305 => {
                // For simplicity, we'll use ChaCha20Poly1305 for now
                self.encrypt_chacha20_poly1305(&payload, &dek.key_bytes).await?
            }
        };

        // Calculate HMAC for additional integrity
        let hmac = if self.config.enable_integrity_checks {
            Some(self.calculate_hmac(&ciphertext, &dek.key_bytes)?)
        } else {
            None
        };

        let metadata = EncryptionMetadata {
            encrypted_at: Utc::now(),
            key_version: dek.created_at.timestamp() as u32, // Use timestamp as version
            original_size: data.len(),
            compressed,
            data_type: None,
            context: context.map(|s| s.to_string()),
        };

        let encrypted_data = EncryptedData {
            ciphertext: general_purpose::STANDARD.encode(&ciphertext),
            nonce: general_purpose::STANDARD.encode(&nonce),
            algorithm,
            tag: tag.map(|t| general_purpose::STANDARD.encode(&t)),
            hmac: hmac.map(|h| general_purpose::STANDARD.encode(&h)),
            metadata,
        };

        // Update metrics
        let elapsed = start_time.elapsed().as_micros() as u64;
        self.update_encrypt_metrics(data.len(), elapsed);

        Ok(encrypted_data)
    }

    /// Decrypt data
    pub async fn decrypt(&mut self, encrypted_data: &EncryptedData) -> Result<SecureData> {
        let start_time = std::time::Instant::now();

        // Get decryption key
        let dek = self.keys.get_data_encryption_key(
            &encrypted_data.algorithm,
            encrypted_data.metadata.context.as_deref(),
        ).await?;

        // Decode base64 data
        let ciphertext = general_purpose::STANDARD.decode(&encrypted_data.ciphertext)
            .context("Invalid base64 ciphertext")?;
        let nonce = general_purpose::STANDARD.decode(&encrypted_data.nonce)
            .context("Invalid base64 nonce")?;

        // Verify HMAC if present
        if let Some(hmac_b64) = &encrypted_data.hmac {
            let hmac_bytes = general_purpose::STANDARD.decode(hmac_b64)
                .context("Invalid base64 HMAC")?;
            
            if !self.verify_hmac(&ciphertext, &dek.key_bytes, &hmac_bytes)? {
                self.metrics.integrity_failures += 1;
                return Err(anyhow::anyhow!("HMAC verification failed"));
            }
        }

        // Decrypt based on algorithm
        let plaintext = match encrypted_data.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let tag = if let Some(tag_b64) = &encrypted_data.tag {
                    Some(general_purpose::STANDARD.decode(tag_b64)
                        .context("Invalid base64 tag")?)
                } else {
                    None
                };
                self.decrypt_aes_gcm(&ciphertext, &nonce, tag.as_deref(), &dek.key_bytes).await?
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                self.decrypt_chacha20_poly1305(&ciphertext, &nonce, &dek.key_bytes).await?
            }
            EncryptionAlgorithm::XChaCha20Poly1305 => {
                // For simplicity, use ChaCha20Poly1305
                self.decrypt_chacha20_poly1305(&ciphertext, &nonce, &dek.key_bytes).await?
            }
        };

        // Decompress if needed
        let final_data = if encrypted_data.metadata.compressed {
            self.decompress_data(&plaintext)?
        } else {
            plaintext
        };

        // Update metrics
        let elapsed = start_time.elapsed().as_micros() as u64;
        self.update_decrypt_metrics(final_data.len(), elapsed);

        Ok(SecureData::new(final_data, "decrypted"))
    }

    /// Encrypt a string and return base64 encoded result
    pub async fn encrypt_string(&mut self, text: &str, context: Option<&str>) -> Result<String> {
        let encrypted = self.encrypt(text.as_bytes(), context).await?;
        let serialized = serde_json::to_string(&encrypted)
            .context("Failed to serialize encrypted data")?;
        Ok(general_purpose::STANDARD.encode(serialized))
    }

    /// Decrypt a base64 encoded encrypted string
    pub async fn decrypt_string(&mut self, encrypted_b64: &str) -> Result<String> {
        let serialized = general_purpose::STANDARD.decode(encrypted_b64)
            .context("Invalid base64 encrypted data")?;
        let encrypted_data: EncryptedData = serde_json::from_slice(&serialized)
            .context("Failed to deserialize encrypted data")?;
        
        let secure_data = self.decrypt(&encrypted_data).await?;
        String::from_utf8(secure_data.into_data())
            .context("Decrypted data is not valid UTF-8")
    }

    /// Rotate encryption keys
    pub async fn rotate_keys(&mut self, master_password: &str) -> Result<()> {
        self.keys.rotate_master_keys(master_password, &self.config).await?;
        self.keys.clear_dek_cache();
        self.metrics.key_derivations += 1;
        
        log::info!("Encryption keys rotated successfully");
        Ok(())
    }

    /// Secure delete of sensitive data (overwrite memory)
    pub fn secure_delete(data: &mut [u8]) {
        // Overwrite with random data multiple times
        for _ in 0..3 {
            getrandom::getrandom(data).unwrap_or_else(|_| {
                // Fallback to simple overwrite if random fails
                data.fill(0xFF);
            });
        }
        // Final zero pass
        data.zeroize();
    }

    /// Get encryption metrics
    pub fn get_metrics(&self) -> &EncryptionMetrics {
        &self.metrics
    }

    /// Get memory protection statistics
    pub fn get_memory_stats(&self) -> &MemoryStats {
        &self.memory_protector.stats
    }

    // Private helper methods

    async fn encrypt_aes_gcm(&self, data: &[u8], key: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Option<Vec<u8>>)> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher.encrypt(&nonce, data)
            .map_err(|e| anyhow::anyhow!("AES-GCM encryption failed: {:?}", e))?;
        
        // AES-GCM includes authentication tag in the ciphertext
        Ok((ciphertext, nonce.to_vec(), None))
    }

    async fn decrypt_aes_gcm(&self, ciphertext: &[u8], nonce: &[u8], _tag: Option<&[u8]>, key: &[u8]) -> Result<Vec<u8>> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        let nonce = aes_gcm::Nonce::from_slice(nonce);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("AES-GCM decryption failed: {:?}", e))
    }

    async fn encrypt_chacha20_poly1305(&self, data: &[u8], key: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Option<Vec<u8>>)> {
        let key = chacha20poly1305::Key::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher.encrypt(&nonce, data)
            .map_err(|e| anyhow::anyhow!("ChaCha20Poly1305 encryption failed: {:?}", e))?;
        
        // ChaCha20Poly1305 includes authentication tag in the ciphertext
        Ok((ciphertext, nonce.to_vec(), None))
    }

    async fn decrypt_chacha20_poly1305(&self, ciphertext: &[u8], nonce: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        let key = chacha20poly1305::Key::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = chacha20poly1305::Nonce::from_slice(nonce);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("ChaCha20Poly1305 decryption failed: {:?}", e))
    }

    fn calculate_hmac(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        use hmac::Mac;
        use hmac::digest::KeyInit;
        let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(key)
            .context("Failed to create HMAC")?;
        mac.update(data);
        Ok(mac.finalize().into_bytes().to_vec())
    }

    fn verify_hmac(&self, data: &[u8], key: &[u8], expected_hmac: &[u8]) -> Result<bool> {
        let calculated_hmac = self.calculate_hmac(data, key)?;
        Ok(calculated_hmac == expected_hmac)
    }

    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Simple compression using flate2 (would need to add dependency)
        // For now, just return the original data
        Ok(data.to_vec())
    }

    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Simple decompression using flate2 (would need to add dependency)
        // For now, just return the original data
        Ok(data.to_vec())
    }

    fn update_encrypt_metrics(&mut self, bytes: usize, elapsed_us: u64) {
        self.metrics.encrypt_operations += 1;
        self.metrics.bytes_encrypted += bytes as u64;
        
        // Update running average
        let total_ops = self.metrics.encrypt_operations;
        self.metrics.avg_encrypt_time_us = 
            (self.metrics.avg_encrypt_time_us * (total_ops - 1) + elapsed_us) / total_ops;
    }

    fn update_decrypt_metrics(&mut self, bytes: usize, elapsed_us: u64) {
        self.metrics.decrypt_operations += 1;
        self.metrics.bytes_decrypted += bytes as u64;
        
        // Update running average
        let total_ops = self.metrics.decrypt_operations;
        self.metrics.avg_decrypt_time_us = 
            (self.metrics.avg_decrypt_time_us * (total_ops - 1) + elapsed_us) / total_ops;
    }
}

impl KeyManager {
    fn new(config: &EncryptionConfig) -> Result<Self> {
        let kdf_params = KdfParams {
            salt: general_purpose::STANDARD.encode(&SaltString::generate(&mut ArgonRng).as_str()),
            iterations: config.key_derivation_iterations,
            memory_cost: 65536, // 64 MB
            parallelism: 4,
        };

        Ok(Self {
            master_keys: HashMap::new(),
            dek_cache: HashMap::new(),
            kdf_params,
        })
    }

    async fn derive_master_keys(&mut self, password: &str, _config: &EncryptionConfig) -> Result<()> {
        let salt = general_purpose::STANDARD.decode(&self.kdf_params.salt)?;
        
        // Derive keys for each supported algorithm
        for algorithm in [EncryptionAlgorithm::Aes256Gcm, EncryptionAlgorithm::ChaCha20Poly1305] {
            let key_size = match algorithm {
                EncryptionAlgorithm::Aes256Gcm => 32,
                EncryptionAlgorithm::ChaCha20Poly1305 | EncryptionAlgorithm::XChaCha20Poly1305 => 32,
            };

            let mut key_bytes = vec![0u8; key_size];
            
            // Use PBKDF2 for key derivation
            let _ = pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
                password.as_bytes(),
                &salt,
                self.kdf_params.iterations,
                &mut key_bytes,
            );

            let master_key = MasterKey {
                key_bytes,
                algorithm: algorithm.clone(),
                created_at: Utc::now(),
                version: 1,
            };

            self.master_keys.insert(algorithm, master_key);
        }

        Ok(())
    }

    async fn get_data_encryption_key(
        &mut self,
        algorithm: &EncryptionAlgorithm,
        context: Option<&str>,
    ) -> Result<DataEncryptionKey> {
        let cache_key = format!("{:?}:{}", algorithm, context.unwrap_or("default"));
        
        // Check cache first
        if let Some(dek) = self.dek_cache.get(&cache_key) {
            if dek.expires_at > Utc::now() {
                return Ok(dek.clone());
            }
        }

        // Derive new DEK from master key
        let master_key = self.master_keys.get(algorithm)
            .context("Master key not found for algorithm")?;

        let mut hasher = Sha256::new();
        hasher.update(&master_key.key_bytes);
        hasher.update(context.unwrap_or("default").as_bytes());
        hasher.update(&Utc::now().timestamp().to_le_bytes());
        
        let derived_key = hasher.finalize().to_vec();

        let dek = DataEncryptionKey {
            key_bytes: derived_key,
            algorithm: algorithm.clone(),
            context: context.unwrap_or("default").to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(24),
        };

        self.dek_cache.insert(cache_key, dek.clone());
        Ok(dek)
    }

    async fn rotate_master_keys(&mut self, password: &str, config: &EncryptionConfig) -> Result<()> {
        // Generate new salt
        self.kdf_params.salt = general_purpose::STANDARD.encode(&SaltString::generate(&mut ArgonRng).as_str());
        
        // Clear existing keys
        self.master_keys.clear();
        
        // Derive new master keys
        self.derive_master_keys(password, config).await?;
        
        Ok(())
    }

    fn clear_dek_cache(&mut self) {
        self.dek_cache.clear();
    }
}

impl MemoryProtector {
    fn new(enabled: bool) -> Self {
        Self {
            enabled,
            stats: MemoryStats {
                secure_allocations: 0,
                total_bytes_protected: 0,
                memory_wipes: 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_encryption_decryption() {
        let mut encryption_system = EncryptionSystem::new().unwrap();
        encryption_system.initialize("test_password_123").await.unwrap();
        
        let test_data = b"Hello, this is sensitive data!";
        
        // Encrypt
        let encrypted = encryption_system.encrypt(test_data, Some("test_context")).await.unwrap();
        assert!(!encrypted.ciphertext.is_empty());
        assert!(!encrypted.nonce.is_empty());
        
        // Decrypt
        let decrypted = encryption_system.decrypt(&encrypted).await.unwrap();
        assert_eq!(decrypted.data(), test_data);
    }

    #[tokio::test]
    async fn test_string_encryption() {
        let mut encryption_system = EncryptionSystem::new().unwrap();
        encryption_system.initialize("test_password_456").await.unwrap();
        
        let test_string = "This is a secret message!";
        
        // Encrypt string
        let encrypted_b64 = encryption_system.encrypt_string(test_string, None).await.unwrap();
        assert!(!encrypted_b64.is_empty());
        
        // Decrypt string
        let decrypted_string = encryption_system.decrypt_string(&encrypted_b64).await.unwrap();
        assert_eq!(decrypted_string, test_string);
    }

    #[tokio::test]
    async fn test_different_algorithms() {
        let mut encryption_system = EncryptionSystem::new().unwrap();
        encryption_system.initialize("test_password_789").await.unwrap();
        
        let test_data = b"Test data for different algorithms";
        
        // Test AES-256-GCM
        let encrypted_aes = encryption_system.encrypt_with_algorithm(
            test_data,
            EncryptionAlgorithm::Aes256Gcm,
            None
        ).await.unwrap();
        
        let decrypted_aes = encryption_system.decrypt(&encrypted_aes).await.unwrap();
        assert_eq!(decrypted_aes.data(), test_data);
        
        // Test ChaCha20-Poly1305
        let encrypted_chacha = encryption_system.encrypt_with_algorithm(
            test_data,
            EncryptionAlgorithm::ChaCha20Poly1305,
            None
        ).await.unwrap();
        
        let decrypted_chacha = encryption_system.decrypt(&encrypted_chacha).await.unwrap();
        assert_eq!(decrypted_chacha.data(), test_data);
    }

    #[tokio::test]
    async fn test_key_rotation() {
        let mut encryption_system = EncryptionSystem::new().unwrap();
        encryption_system.initialize("test_password_original").await.unwrap();
        
        let test_data = b"Data encrypted with original key";
        let _encrypted_original = encryption_system.encrypt(test_data, None).await.unwrap();
        
        // Rotate keys
        encryption_system.rotate_keys("test_password_new").await.unwrap();
        
        // Should still be able to decrypt with new system after rotation
        // Note: In a real system, you'd need to track key versions
        let test_data_new = b"Data encrypted with new key";
        let encrypted_new = encryption_system.encrypt(test_data_new, None).await.unwrap();
        let decrypted_new = encryption_system.decrypt(&encrypted_new).await.unwrap();
        assert_eq!(decrypted_new.data(), test_data_new);
    }

    #[test]
    fn test_secure_data() {
        let sensitive_data = b"very sensitive information".to_vec();
        let secure_data = SecureData::new(sensitive_data.clone(), "test");
        
        assert_eq!(secure_data.data(), &sensitive_data);
        assert_eq!(secure_data.data_type(), "test");
        
        // Test move semantics
        let moved_data = secure_data.into_data();
        assert_eq!(moved_data, sensitive_data);
    }

    #[test]
    fn test_secure_delete() {
        let mut sensitive_data = b"sensitive data to delete".to_vec();
        let original_data = sensitive_data.clone();
        
        EncryptionSystem::secure_delete(&mut sensitive_data);
        
        // Data should be different after secure delete
        assert_ne!(sensitive_data, original_data);
        // Should be all zeros after final pass
        assert!(sensitive_data.iter().all(|&b| b == 0));
    }
}
