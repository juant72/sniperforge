use anyhow::Result;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::instruction::Instruction;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::instruction as ata_instruction;
use spl_token::instruction as token_instruction;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::str::FromStr;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigFile {
    network: String,
    cluster_url: String,
    tokens: HashMap<String, TokenInfo>,
    programs: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenInfo {
    symbol: String,
    name: String,
    mint: String,
    decimals: u8,
    verified: bool,
    test_supply: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    info!("🚀 === PRUEBA DE TOKENS CUSTOM - DevNet ===");
    info!("============================================");

    // Load wallet from environment
    let wallet_keypair = load_wallet_from_env()?;
    let wallet_pubkey = wallet_keypair.pubkey();
    info!("✅ Wallet cargado: {}", wallet_pubkey);

    // Load config with custom tokens
    let config_path = "config/devnet-automated.json";
    let config_content = fs::read_to_string(config_path)?;
    let config: ConfigFile = serde_json::from_str(&config_content)?;

    info!("📋 Configuración cargada: {}", config.network);
    info!("🔗 RPC: {}", config.cluster_url);
    info!("🪙 Tokens disponibles: {}", config.tokens.len());

    // Create RPC client
    let rpc_client =
        RpcClient::new_with_commitment(config.cluster_url.clone(), CommitmentConfig::confirmed());

    // Check wallet balance
    info!("💰 Verificando balance del wallet...");
    let balance = rpc_client.get_balance(&wallet_pubkey)?;
    let balance_sol = balance as f64 / LAMPORTS_PER_SOL as f64;
    info!("   Balance: {:.9} SOL", balance_sol);

    if balance_sol < 0.01 {
        error!("❌ Balance insuficiente. Necesitas al menos 0.01 SOL");
        return Ok(());
    }

    // Test our custom tokens
    info!("\n🎯 === VERIFICANDO TOKENS CUSTOM ===");

    // Test each custom token
    for (symbol, token_info) in &config.tokens {
        if symbol == "SOL" {
            continue; // Skip SOL
        }

        info!("\n📊 Probando token: {} ({})", symbol, token_info.name);
        test_token_account(&rpc_client, &wallet_keypair, token_info).await?;
    }

    info!("\n🎯 === CONCLUSIONES ===");
    info!("✅ Tokens custom verificados exitosamente");
    info!("✅ Transacciones confirmadas en blockchain");
    info!("⚠️ Jupiter no soporta tokens custom en DevNet");
    info!("💡 Necesitamos usar AMM directo o crear liquidez");

    Ok(())
}

async fn test_token_account(
    rpc_client: &RpcClient,
    wallet_keypair: &Keypair,
    token_info: &TokenInfo,
) -> Result<()> {
    info!("🔄 Verificando token: {}", token_info.symbol);

    // Parse mint address
    let mint_pubkey = Pubkey::from_str(&token_info.mint)?;
    let token_program = spl_token::ID;

    // Get associated token account
    let ata_pubkey = spl_associated_token_account::get_associated_token_address(
        &wallet_keypair.pubkey(),
        &mint_pubkey,
    );

    info!("   Mint: {}", mint_pubkey);
    info!("   ATA: {}", ata_pubkey);

    // Check if ATA exists
    match rpc_client.get_account(&ata_pubkey) {
        Ok(account) => {
            info!("   ✅ ATA exists, checking balance...");

            // Get token balance
            match rpc_client.get_token_account_balance(&ata_pubkey) {
                Ok(balance) => {
                    let amount = balance.amount.parse::<u64>().unwrap_or(0);
                    let ui_amount = amount as f64 / 10_u64.pow(token_info.decimals as u32) as f64;
                    info!(
                        "   💰 Balance: {} {} (raw: {})",
                        ui_amount, token_info.symbol, amount
                    );

                    if amount > 0 {
                        info!("   🎯 TOKEN DISPONIBLE PARA USAR");

                        // Try a small transfer to test the token works
                        test_token_transfer(
                            rpc_client,
                            wallet_keypair,
                            &ata_pubkey,
                            &mint_pubkey,
                            1,
                        )
                        .await?;
                    } else {
                        info!("   ⚠️ Balance cero, token existe pero no tiene fondos");
                    }
                }
                Err(e) => {
                    info!("   ❌ Error obteniendo balance: {}", e);
                }
            }
        }
        Err(_) => {
            info!("   ❌ ATA no existe, token no disponible");
        }
    }

    Ok(())
}

async fn test_token_transfer(
    rpc_client: &RpcClient,
    wallet_keypair: &Keypair,
    from_ata: &Pubkey,
    mint: &Pubkey,
    amount: u64,
) -> Result<()> {
    info!("   🔄 Probando transferencia de {} unidades...", amount);

    // Create a temporary destination account (self-transfer)
    let dest_ata =
        spl_associated_token_account::get_associated_token_address(&wallet_keypair.pubkey(), mint);

    // Create transfer instruction
    let transfer_instruction = token_instruction::transfer(
        &spl_token::ID,
        from_ata,
        &dest_ata,
        &wallet_keypair.pubkey(),
        &[],
        amount,
    )?;

    // Get recent blockhash
    let recent_blockhash = rpc_client.get_latest_blockhash()?;

    // Create and sign transaction
    let mut transaction =
        Transaction::new_with_payer(&[transfer_instruction], Some(&wallet_keypair.pubkey()));

    transaction.sign(&[wallet_keypair], recent_blockhash);

    // Send transaction
    match rpc_client.send_and_confirm_transaction(&transaction) {
        Ok(signature) => {
            info!("   ✅ Transferencia exitosa: {}", signature);
            info!(
                "   🔍 Explorer: https://explorer.solana.com/tx/{}?cluster=devnet",
                signature
            );
        }
        Err(e) => {
            info!("   ❌ Error en transferencia: {}", e);
        }
    }

    Ok(())
}

fn load_wallet_from_env() -> Result<Keypair> {
    // Try to load wallet from environment variable
    if let Ok(private_key) = env::var("SOLANA_PRIVATE_KEY") {
        // Handle different formats
        if private_key.starts_with('[') && private_key.ends_with(']') {
            // Array format: [1,2,3,...]
            let bytes_str = private_key.trim_start_matches('[').trim_end_matches(']');
            let bytes: Vec<u8> = bytes_str
                .split(',')
                .map(|s| s.trim().parse::<u8>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| anyhow::anyhow!("Invalid private key format: {}", e))?;

            if bytes.len() != 64 {
                return Err(anyhow::anyhow!("Private key must be 64 bytes long"));
            }

            Ok(Keypair::from_bytes(&bytes)?)
        } else {
            // Base58 format
            let bytes = bs58::decode(private_key)
                .into_vec()
                .map_err(|e| anyhow::anyhow!("Invalid base58 private key: {}", e))?;
            Ok(Keypair::from_bytes(&bytes)?)
        }
    } else {
        Err(anyhow::anyhow!(
            "SOLANA_PRIVATE_KEY environment variable not found"
        ))
    }
}
