use std::fs;
use solana_sdk::signer::{keypair::Keypair, Signer};
use bs58;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Leer el archivo de wallet
    let wallet_content = fs::read_to_string("mainnet-validation-wallet.json")?;
    let keypair_data: Vec<u8> = serde_json::from_str(&wallet_content)?;
    
    // Crear keypair desde los datos
    let keypair = Keypair::from_bytes(&keypair_data)?;
    
    // Obtener la clave privada como bytes
    let private_key_bytes = keypair.to_bytes();
    
    // Convertir a Base58
    let base58_private_key = bs58::encode(&private_key_bytes).into_string();
    
    println!("=== WALLET EXPORT INFORMATION ===");
    println!("Public Key: {}", keypair.pubkey());
    println!("Private Key (Base58): {}", base58_private_key);
    println!("Private Key (JSON Array): {:?}", private_key_bytes.to_vec());
    
    // Crear archivo de exportación actualizado
    let export_content = format!(r#"# 📱 WALLET EXPORT FOR MOBILE - MAINNET VALIDATION

## ⚠️ SECURITY WARNING
This file contains your PRIVATE KEY. Keep it SECURE and DELETE after import.
Never share this information with anyone.

## 📋 WALLET INFORMATION
Public Key: {}
Network: Solana Mainnet
Purpose: SniperForge Sprint 1 Validation

## 🔑 IMPORT FORMATS

### FORMAT 1: JSON ARRAY (Solflare, Phantom)
{:?}

### FORMAT 2: BASE58 PRIVATE KEY (most wallets)
{}

### FORMAT 3: MNEMONIC SEED PHRASE (if supported)
This format requires conversion. Most mobile wallets prefer Base58 format above.

## 📱 IMPORT INSTRUCTIONS

### PHANTOM WALLET (iOS/Android)
1. Open Phantom app
2. Tap "+" → "Import Wallet"  
3. Select "Private Key"
4. Paste the Base58 key from FORMAT 2
5. Set wallet name: "SniperForge Mainnet"

### SOLFLARE WALLET (iOS/Android)
1. Open Solflare app
2. Tap "Create/Import" → "Import Wallet"
3. Select "Private Key"
4. Paste the JSON array from FORMAT 1
5. Set wallet name: "SniperForge Mainnet"

### BACKPACK WALLET (iOS/Android)
1. Open Backpack app
2. Tap "Import Wallet"
3. Select "Private Key"
4. Paste the Base58 key from FORMAT 2
5. Confirm import

## 🔒 SECURITY CHECKLIST
- [ ] Delete this file after successful import
- [ ] Test small transaction first (0.001 SOL)
- [ ] Keep backup copy in secure location
- [ ] Never screenshot private keys
- [ ] Use secure messaging to transfer to phone

## 📊 VERIFICATION
After import, verify the public key matches:
{}

If public key doesn't match, DO NOT USE THIS WALLET.

---
Generated: June 27, 2025
Project: SniperForge Sprint 1 Validation
"#, 
        keypair.pubkey(),
        private_key_bytes.to_vec(),
        base58_private_key,
        keypair.pubkey()
    );
    
    fs::write("wallet-export-MOBILE.txt", export_content)?;
    println!("\n✅ Export file updated: wallet-export-MOBILE.txt");
    
    Ok(())
}
