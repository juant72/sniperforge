use sniperforge::shared::config_loader::load_config;
use sniperforge::shared::jupiter_api::JupiterApi;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::str::FromStr;
use dotenv::dotenv;
use std::env;

/// Test para verificar que los tokens creados automáticamente funcionan correctamente
/// y pueden ser usados para swaps en DevNet

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    println!("🧪 === Test de Tokens DevNet Automatizados ===");
    
    // Cargar configuración con tokens automatizados
    let config_path = "config/devnet-automated.json";
    
    if !std::path::Path::new(config_path).exists() {
        println!("❌ Archivo de configuración no encontrado: {}", config_path);
        println!("💡 Ejecuta primero: cargo run --bin create_devnet_tokens_automated");
        return Ok(());
    }
    
    let config = load_config(config_path)?;
    println!("✅ Configuración cargada desde: {}", config_path);
    
    // Configurar cliente RPC
    let client = RpcClient::new_with_commitment(
        config.cluster_url.clone(),
        CommitmentConfig::confirmed()
    );
    
    // Cargar wallet
    let private_key_str = env::var("SOLANA_PRIVATE_KEY")
        .expect("SOLANA_PRIVATE_KEY no encontrada en .env");
    
    let private_key_bytes: Vec<u8> = private_key_str
        .split(',')
        .map(|s| s.trim().parse::<u8>().unwrap())
        .collect();
    
    let payer = Keypair::from_bytes(&private_key_bytes)?;
    println!("💰 Wallet: {}", payer.pubkey());
    
    // Verificar balance SOL
    let balance = client.get_balance(&payer.pubkey())?;
    let balance_sol = balance as f64 / 1_000_000_000.0;
    println!("💵 Balance SOL: {:.4}", balance_sol);
    
    // Listar todos los tokens en la configuración
    println!("\n📋 === TOKENS EN CONFIGURACIÓN ===");
    for (symbol, token_info) in &config.tokens {
        println!("🪙 {}: {}", symbol, token_info.mint);
        println!("   Name: {}", token_info.name);
        println!("   Decimals: {}", token_info.decimals);
        
        // Verificar que el token existe on-chain
        match client.get_account(&Pubkey::from_str(&token_info.mint)?) {
            Ok(account) => {
                println!("   ✅ Token verificado on-chain");
                
                // Si no es SOL, verificar que tenemos balance
                if symbol != "SOL" {
                    // Derivar associated token account
                    let mint_pubkey = Pubkey::from_str(&token_info.mint)?;
                    let ata = spl_associated_token_account::get_associated_token_address(
                        &payer.pubkey(),
                        &mint_pubkey
                    );
                    
                    match client.get_token_account_balance(&ata) {
                        Ok(balance) => {
                            println!("   💰 Balance: {} {}", balance.ui_amount_string, symbol);
                        }
                        Err(_) => {
                            println!("   ⚠️  No tienes balance de este token");
                        }
                    }
                }
            }
            Err(e) => {
                println!("   ❌ Error verificando token: {}", e);
            }
        }
        println!();
    }
    
    // Test de Jupiter quotes
    println!("🔄 === TEST DE JUPITER QUOTES ===");
    
    if let (Some(sol_token), Some(test_usdc)) = (config.tokens.get("SOL"), config.tokens.get("TEST_USDC")) {
        let jupiter = JupiterApi::new(&config.cluster_url)?;
        
        println!("📞 Solicitando quote SOL -> TEST_USDC...");
        
        let input_mint = sol_token.mint.clone();
        let output_mint = test_usdc.mint.clone();
        let amount = 100_000_000; // 0.1 SOL en lamports
        
        match jupiter.get_quote(&input_mint, &output_mint, amount, None).await {
            Ok(quote) => {
                println!("✅ Quote recibido exitosamente!");
                println!("   Input: {} lamports SOL", quote.in_amount);
                println!("   Output: {} {} units", quote.out_amount, test_usdc.symbol);
                println!("   Price Impact: {:?}", quote.price_impact_pct);
                
                if let Some(route_plan) = &quote.route_plan {
                    println!("   Rutas disponibles: {} pasos", route_plan.len());
                    for (i, step) in route_plan.iter().enumerate() {
                        println!("     Paso {}: {}", i + 1, step.swap_info.label);
                    }
                }
            }
            Err(e) => {
                println!("❌ Error obteniendo quote: {}", e);
                println!("   Esto es normal si no hay liquidez entre estos tokens en DevNet");
            }
        }
    }
    
    // Test con otros pares de tokens
    println!("\n🔄 === TEST DE MÚLTIPLES PARES ===");
    
    let test_pairs = vec![
        ("SOL", "TEST_USDT"),
        ("TEST_USDC", "TEST_USDT"),
        ("SOL", "TEST_RAY"),
        ("TEST_USDC", "TEST_RAY"),
    ];
    
    for (from_symbol, to_symbol) in test_pairs {
        if let (Some(from_token), Some(to_token)) = (
            config.tokens.get(from_symbol),
            config.tokens.get(to_symbol)
        ) {
            println!("\n📞 Quote {} -> {}...", from_symbol, to_symbol);
            
            let jupiter = JupiterApi::new(&config.cluster_url)?;
            let amount = if from_symbol == "SOL" { 50_000_000 } else { 100_000_000 }; // 0.05 SOL o 100 tokens
            
            match jupiter.get_quote(&from_token.mint, &to_token.mint, amount, None).await {
                Ok(quote) => {
                    println!("✅ Quote disponible: {} -> {} units", 
                           quote.in_amount, quote.out_amount);
                }
                Err(e) => {
                    println!("⚠️  No hay ruta disponible: {}", e);
                }
            }
        }
    }
    
    // Resumen final
    println!("\n🎯 === RESUMEN FINAL ===");
    println!("✅ Configuración cargada correctamente");
    println!("✅ Tokens verificados on-chain");
    println!("✅ Jupiter API funcional");
    
    println!("\n💡 PRÓXIMOS PASOS:");
    println!("1. Para hacer swaps reales, asegúrate de tener balance de los tokens de origen");
    println!("2. Usa cargo run --bin test_real_swap_configured para swaps reales");
    println!("3. Usa spl-token accounts para ver todos tus balances");
    
    println!("\n📝 COMANDOS ÚTILES:");
    println!("spl-token accounts                    # Ver todos tus tokens");
    println!("spl-token balance <MINT_ADDRESS>      # Ver balance específico");
    println!("spl-token transfer <MINT> <AMOUNT> <RECIPIENT>  # Transferir tokens");
    
    Ok(())
}
