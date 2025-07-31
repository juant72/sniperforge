// ================================================================================
// WALLET MANAGER FOR REAL TRADING
// ================================================================================
// Gestión segura de claves privadas para trading real
// ================================================================================

use anyhow::{Result, anyhow};
use solana_sdk::{
    signature::{Keypair, Signer},
    pubkey::Pubkey,
};
use std::fs;
use std::path::Path;
use tracing::{info, warn, error};

/// Manager para manejar wallets de forma segura
pub struct WalletManager {
    pub keypair: Keypair,
    pub public_key: Pubkey,
}

impl WalletManager {
    /// Crear wallet manager desde archivo de clave privada
    pub fn from_file<P: AsRef<Path>>(keypair_path: P) -> Result<Self> {
        info!("📁 Cargando keypair desde archivo...");
        
        let keypair_bytes = fs::read(&keypair_path)
            .map_err(|e| anyhow!("Failed to read keypair file: {}", e))?;
        
        let keypair = if keypair_bytes.len() == 64 {
            // Archivo de bytes raw (64 bytes)
            Keypair::from_bytes(&keypair_bytes)
                .map_err(|e| anyhow!("Invalid keypair format: {}", e))?
        } else {
            // Intentar como JSON array
            let json_str = String::from_utf8(keypair_bytes)
                .map_err(|e| anyhow!("Keypair file is not valid UTF-8: {}", e))?;
            
            let bytes_array: Vec<u8> = serde_json::from_str(&json_str)
                .map_err(|e| anyhow!("Failed to parse keypair JSON: {}", e))?;
            
            if bytes_array.len() != 64 {
                return Err(anyhow!("Keypair must be 64 bytes, got {}", bytes_array.len()));
            }
            
            let mut bytes = [0u8; 64];
            bytes.copy_from_slice(&bytes_array);
            
            Keypair::from_bytes(&bytes)
                .map_err(|e| anyhow!("Invalid keypair bytes: {}", e))?
        };
        
        let public_key = keypair.pubkey();
        
        info!("✅ Wallet cargado exitosamente");
        info!("   🔑 Public key: {}", public_key);
        
        Ok(Self {
            keypair,
            public_key,
        })
    }
    
    /// Crear wallet manager desde variable de entorno
    pub fn from_env(env_var: &str) -> Result<Self> {
        info!("🌍 Cargando keypair desde variable de entorno: {}", env_var);
        
        let keypair_str = std::env::var(env_var)
            .map_err(|_| anyhow!("Environment variable {} not found", env_var))?;
        
        // Intentar como JSON array
        let bytes_array: Vec<u8> = serde_json::from_str(&keypair_str)
            .map_err(|e| anyhow!("Failed to parse keypair from env: {}", e))?;
        
        if bytes_array.len() != 64 {
            return Err(anyhow!("Keypair must be 64 bytes, got {}", bytes_array.len()));
        }
        
        let mut bytes = [0u8; 64];
        bytes.copy_from_slice(&bytes_array);
        
        let keypair = Keypair::from_bytes(&bytes)
            .map_err(|e| anyhow!("Invalid keypair bytes: {}", e))?;
        
        let public_key = keypair.pubkey();
        
        info!("✅ Wallet cargado desde environment");
        info!("   🔑 Public key: {}", public_key);
        
        Ok(Self {
            keypair,
            public_key,
        })
    }
    
    /// Crear wallet manager desde base58 private key
    pub fn from_base58(private_key_base58: &str) -> Result<Self> {
        info!("🔤 Cargando keypair desde base58...");
        
        let private_key_bytes = bs58::decode(private_key_base58)
            .into_vec()
            .map_err(|e| anyhow!("Invalid base58 private key: {}", e))?;
        
        if private_key_bytes.len() != 64 {
            return Err(anyhow!("Private key must be 64 bytes, got {}", private_key_bytes.len()));
        }
        
        let keypair = Keypair::from_bytes(&private_key_bytes)
            .map_err(|e| anyhow!("Invalid keypair bytes: {}", e))?;
        
        let public_key = keypair.pubkey();
        
        info!("✅ Wallet cargado desde base58");
        info!("   🔑 Public key: {}", public_key);
        
        Ok(Self {
            keypair,
            public_key,
        })
    }
    
    /// Crear wallet desde la clave privada conocida del sistema
    pub fn from_known_system_key() -> Result<Self> {
        warn!("⚠️ Usando clave privada del sistema conocida");
        warn!("   🔒 Solo para testing/demo - NO usar en producción");
        
        // Esta es la clave privada correspondiente a JDzF9LkpoQVac6c6ufHW1c6Gevd3vFB4RXSSjceL8Kq7
        // SOLO para testing - en producción usar método seguro
        let _known_private_key = [
            // Esto debería venir de un método seguro en producción
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        
        // NOTA: Esta implementación es solo para demostración
        // En producción, la clave privada debe venir de:
        // 1. Archivo seguro cifrado
        // 2. Hardware wallet
        // 3. Secure key management service
        
        error!("🚨 IMPLEMENTACIÓN DE DEMO - CLAVE PRIVADA FALTANTE");
        error!("   Para trading real, configurar clave privada segura");
        
        Err(anyhow!("Sistema clave real no implementado - usar from_file() o from_env()"))
    }
    
    /// Verificar si el wallet tiene suficiente balance para trading
    pub async fn check_balance(&self, rpc_client: &solana_client::rpc_client::RpcClient, min_sol: f64) -> Result<f64> {
        let balance_lamports = rpc_client.get_balance(&self.public_key)
            .map_err(|e| anyhow!("Failed to get wallet balance: {}", e))?;
        
        let balance_sol = balance_lamports as f64 / 1_000_000_000.0;
        
        info!("💰 Balance actual: {:.9} SOL", balance_sol);
        
        if balance_sol < min_sol {
            return Err(anyhow!(
                "Insufficient balance: {:.9} SOL < {:.9} SOL required", 
                balance_sol, min_sol
            ));
        }
        
        Ok(balance_sol)
    }
    
    /// Obtener referencia al keypair (para firmar transacciones)
    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }
    
    /// Obtener public key
    pub fn pubkey(&self) -> &Pubkey {
        &self.public_key
    }
}
