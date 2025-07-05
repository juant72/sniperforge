use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_token::{
    instruction as token_instruction,
    state::Mint,
    solana_program::program_pack::Pack,
};
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account,
};
use dotenv::dotenv;
use std::env;

/// Script para crear tokens SPL de prueba en DevNet de forma programática
/// Este script automatiza la creación de tokens que necesitas para hacer swaps reales

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    println!("🚀 === SniperForge: Creador Automático de Tokens DevNet ===");
    
    // Configurar cliente RPC para DevNet - usar RPC premium de Alchemy
    let rpc_urls = vec![
        "https://solana-devnet.g.alchemy.com/v2/X64q4zZFEMz_RYzthxUMg",
        "https://api.devnet.solana.com",
        "https://devnet.genesysgo.net",
    ];
    
    let mut client = None;
    let mut working_rpc = "";
    
    for rpc_url in &rpc_urls {
        println!("🔍 Probando RPC: {}", rpc_url);
        let test_client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
        
        // Test de conectividad
        if let Ok(latest_blockhash) = test_client.get_latest_blockhash() {
            println!("✅ RPC funcionando: {}", rpc_url);
            client = Some(test_client);
            working_rpc = rpc_url;
            break;
        } else {
            println!("❌ RPC no disponible: {}", rpc_url);
        }
    }
    
    let client = client.expect("No se pudo conectar a ningún RPC de DevNet");
    
    println!("🌐 Conectado a DevNet: {}", working_rpc);
    
    // Cargar wallet desde .env
    let private_key_str = env::var("SOLANA_PRIVATE_KEY")
        .expect("SOLANA_PRIVATE_KEY no encontrada en .env");
    
    // Decodificar desde base58 (formato de Phantom/Solflare)
    let private_key_bytes = bs58::decode(&private_key_str)
        .into_vec()
        .expect("Error decodificando clave privada desde base58");
    
    let payer = Keypair::from_bytes(&private_key_bytes)?;
    println!("💰 Wallet cargada: {}", payer.pubkey());
    
    // Verificar balance
    let balance = client.get_balance(&payer.pubkey())?;
    let balance_sol = balance as f64 / 1_000_000_000.0;
    println!("💵 Balance: {} SOL", balance_sol);
    
    if balance_sol < 0.5 {
        println!("⚠️  Balance bajo. Necesitas al menos 0.5 SOL para crear tokens.");
        println!("🚰 Solicita airdrop: solana airdrop 2");
        return Ok(());
    }
    
    // Definir tokens a crear - empezar con menos para testing
    let tokens_to_create = vec![
        ("TEST_USDC", 6, 1000),   // Reducir supply inicial
        ("TEST_USDT", 6, 1000),
        ("TEST_RAY", 6, 500),
    ];
    
    let mut created_tokens = Vec::new();
    
    for (name, decimals, supply) in tokens_to_create {
        println!("\n🪙 === Creando {} ===", name);
        
        match create_token_with_supply(&client, &payer, decimals, supply as u64).await {
            Ok((mint_pubkey, token_account)) => {
                println!("✅ {} creado exitosamente!", name);
                println!("   Mint: {}", mint_pubkey);
                println!("   Account: {}", token_account);
                println!("   Supply: {} tokens", supply);
                
                created_tokens.push((name.to_string(), mint_pubkey, decimals, supply));
            }
            Err(e) => {
                println!("❌ Error creando {}: {}", name, e);
            }
        }
        
        // Pausa más larga para evitar rate limiting
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
    
    // Generar archivo de configuración JSON
    generate_config_file(&created_tokens, working_rpc)?;
    
    // Mostrar resumen
    println!("\n🎉 === RESUMEN FINAL ===");
    println!("✅ Tokens creados: {}", created_tokens.len());
    println!("📄 Configuración guardada en: config/devnet-automated.json");
    
    println!("\n📋 MINT ADDRESSES:");
    println!("SOL (nativo): So11111111111111111111111111111111111111112");
    for (name, mint, _, supply) in &created_tokens {
        println!("{}: {} ({} tokens)", name, mint, supply);
    }
    
    println!("\n🧪 COMANDOS DE PRUEBA:");
    println!("cargo run --bin test_real_swap_configured");
    println!("cargo run --bin find_working_routes");
    
    println!("\n💡 TIP: Estos tokens ahora están disponibles para swaps reales en DevNet!");
    
    Ok(())
}

async fn create_token_with_supply(
    client: &RpcClient,
    payer: &Keypair,
    decimals: u8,
    supply: u64,
) -> Result<(Pubkey, Pubkey), Box<dyn std::error::Error>> {
    // Generar keypair para el mint
    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();
    
    // Calcular rent para mint account
    let mint_rent = client.get_minimum_balance_for_rent_exemption(Mint::LEN)?;
    
    // Crear mint account
    let create_mint_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &mint_pubkey,
        mint_rent,
        Mint::LEN as u64,
        &spl_token::id(),
    );
    
    // Inicializar mint
    let init_mint_ix = token_instruction::initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &payer.pubkey(), // mint authority
        Some(&payer.pubkey()), // freeze authority
        decimals,
    )?;
    
    // Crear y enviar transacción para el mint con reintentos
    let mut transaction = Transaction::new_with_payer(
        &[create_mint_account_ix, init_mint_ix],
        Some(&payer.pubkey()),
    );
    
    let recent_blockhash = client.get_latest_blockhash()?;
    transaction.sign(&[payer, &mint_keypair], recent_blockhash);
    
    // Intentar enviar con reintentos
    let signature = retry_transaction(client, &transaction).await?;
    println!("   📝 Mint creado: {}", signature);
    
    // Crear associated token account
    let associated_token_account = get_associated_token_address(&payer.pubkey(), &mint_pubkey);
    
    let create_ata_ix = create_associated_token_account(
        &payer.pubkey(),
        &payer.pubkey(),
        &mint_pubkey,
        &spl_token::id(),
    );
    
    let mut ata_transaction = Transaction::new_with_payer(
        &[create_ata_ix],
        Some(&payer.pubkey()),
    );
    
    let recent_blockhash = client.get_latest_blockhash()?;
    ata_transaction.sign(&[payer], recent_blockhash);
    
    let ata_signature = retry_transaction(client, &ata_transaction).await?;
    println!("   📦 Token account creado: {}", ata_signature);
    
    // Mintear tokens al associated token account
    let mint_to_ix = token_instruction::mint_to(
        &spl_token::id(),
        &mint_pubkey,
        &associated_token_account,
        &payer.pubkey(),
        &[],
        supply * 10_u64.pow(decimals as u32), // Ajustar por decimales
    )?;
    
    let mut mint_transaction = Transaction::new_with_payer(
        &[mint_to_ix],
        Some(&payer.pubkey()),
    );
    
    let recent_blockhash = client.get_latest_blockhash()?;
    mint_transaction.sign(&[payer], recent_blockhash);
    
    let mint_signature = retry_transaction(client, &mint_transaction).await?;
    println!("   🏭 Tokens minteados: {}", mint_signature);
    
    Ok((mint_pubkey, associated_token_account))
}

async fn retry_transaction(
    client: &RpcClient,
    transaction: &Transaction,
) -> Result<String, Box<dyn std::error::Error>> {
    let max_retries = 3;
    let mut last_error = None;
    
    for attempt in 1..=max_retries {
        match client.send_and_confirm_transaction(transaction) {
            Ok(signature) => return Ok(signature.to_string()),
            Err(e) => {
                println!("   ⚠️  Intento {}/{} falló: {}", attempt, max_retries, e);
                last_error = Some(e);
                
                if attempt < max_retries {
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
        }
    }
    
    Err(Box::new(last_error.unwrap()))
}

fn generate_config_file(tokens: &[(String, Pubkey, u8, i32)], rpc_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    use serde_json::{json, Map};
    
    let mut token_map = Map::new();
    
    // Agregar SOL nativo
    token_map.insert("SOL".to_string(), json!({
        "mint": "So11111111111111111111111111111111111111112",
        "symbol": "SOL",
        "name": "Solana",
        "decimals": 9,
        "verified": true
    }));
    
    // Agregar tokens creados
    for (name, mint, decimals, supply) in tokens {
        let symbol = name.replace("TEST_", "");
        token_map.insert(name.clone(), json!({
            "mint": mint.to_string(),
            "symbol": symbol,
            "name": format!("{} (DevNet Test)", symbol),
            "decimals": decimals,
            "verified": true,
            "test_supply": supply
        }));
    }
    
    let config = json!({
        "network": "devnet",
        "cluster_url": rpc_url,
        "enable_real_swaps": true,
        "description": "Configuración automática con tokens creados programáticamente",
        "created_at": chrono::Utc::now().to_rfc3339(),
        "programs": {
            "jupiter": "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
            "orca": "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP", 
            "raydium": "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
            "token": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        "tokens": token_map
    });
    
    std::fs::create_dir_all("config")?;
    std::fs::write(
        "config/devnet-automated.json",
        serde_json::to_string_pretty(&config)?
    )?;
    
    Ok(())
}
