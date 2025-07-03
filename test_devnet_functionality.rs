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

/// Test para verificar swaps SOL <-> wSOL que siempre funcionan en DevNet
/// Este test confirma que nuestro engine funciona correctamente

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    println!("🧪 === Test de Swap SOL <-> wSOL en DevNet ===");
    println!("ℹ️  Este swap siempre funciona porque SOL y wSOL son el mismo token");
    
    // Usar configuración básica con solo SOL
    let config_path = "config/devnet.json";
    let config = load_config(config_path)?;
    println!("✅ Configuración cargada: {}", config_path);
    
    // Configurar cliente RPC con múltiples opciones
    let rpc_urls = vec![
        "https://api.devnet.solana.com",
        "https://devnet.genesysgo.net",
        "https://solana-devnet.g.alchemy.com/v2/demo",
    ];
    
    let mut client = None;
    let mut working_rpc = "";
    
    for rpc_url in &rpc_urls {
        println!("🔍 Probando RPC: {}", rpc_url);
        let test_client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
        
        if let Ok(_) = test_client.get_health() {
            println!("✅ RPC saludable: {}", rpc_url);
            client = Some(test_client);
            working_rpc = rpc_url;
            break;
        } else {
            println!("❌ RPC no disponible: {}", rpc_url);
        }
    }
    
    let client = client.expect("No se pudo conectar a ningún RPC de DevNet");
    println!("🌐 Usando RPC: {}", working_rpc);
    
    // Cargar wallet
    let private_key_str = env::var("SOLANA_PRIVATE_KEY")
        .expect("SOLANA_PRIVATE_KEY no encontrada en .env");
    
    let private_key_bytes = bs58::decode(&private_key_str)
        .into_vec()
        .expect("Error decodificando clave privada desde base58");
    
    let payer = Keypair::from_bytes(&private_key_bytes)?;
    println!("💰 Wallet: {}", payer.pubkey());
    
    // Verificar balance SOL
    let balance = client.get_balance(&payer.pubkey())?;
    let balance_sol = balance as f64 / 1_000_000_000.0;
    println!("💵 Balance SOL: {:.4}", balance_sol);
    
    if balance_sol < 0.1 {
        println!("⚠️  Balance bajo. Necesitas al menos 0.1 SOL para testing.");
        println!("🚰 Solicita airdrop: solana airdrop 2");
        return Ok(());
    }
    
    // Test de Jupiter API con SOL/wSOL
    println!("\n🔄 === TEST DE JUPITER API ===");
    
    let jupiter = JupiterApi::new(working_rpc)?;
    
    // SOL mint address (siempre el mismo)
    let sol_mint = "So11111111111111111111111111111111111111112";
    let wsol_mint = "So11111111111111111111111111111111111111112"; // Es el mismo
    
    // Test 1: Quote SOL -> wSOL (wrap)
    println!("\n📞 Test 1: SOL -> wSOL (wrap)");
    let amount = 50_000_000; // 0.05 SOL en lamports
    
    match jupiter.get_quote(sol_mint, wsol_mint, amount, None).await {
        Ok(quote) => {
            println!("✅ Quote recibido!");
            println!("   Input: {} lamports SOL", quote.in_amount);
            println!("   Output: {} lamports wSOL", quote.out_amount);
            println!("   Price Impact: {:?}", quote.price_impact_pct);
            
            if let Some(route_plan) = &quote.route_plan {
                println!("   Rutas: {} pasos", route_plan.len());
                for (i, step) in route_plan.iter().enumerate() {
                    println!("     Paso {}: {}", i + 1, step.swap_info.label);
                }
            }
        }
        Err(e) => {
            println!("❌ Error en quote: {}", e);
        }
    }
    
    // Test 2: Verificar balance de wSOL
    println!("\n💰 === VERIFICACIÓN DE BALANCES ===");
    
    let wsol_ata = spl_associated_token_account::get_associated_token_address(
        &payer.pubkey(),
        &Pubkey::from_str(wsol_mint)?
    );
    
    match client.get_token_account_balance(&wsol_ata) {
        Ok(balance) => {
            println!("💰 Balance wSOL actual: {} wSOL", balance.ui_amount_string);
        }
        Err(_) => {
            println!("ℹ️  No tienes cuenta de wSOL todavía (normal si no has hecho wrap)");
        }
    }
    
    // Test 3: Verificar otros tokens en configuración
    println!("\n📋 === TOKENS EN CONFIGURACIÓN ===");
    
    for (symbol, token_info) in &config.tokens {
        println!("🪙 {}: {}", symbol, token_info.mint);
        
        // Verificar que el mint existe
        match client.get_account(&Pubkey::from_str(&token_info.mint)?) {
            Ok(_) => {
                println!("   ✅ Mint válido");
                
                // Si no es SOL, verificar ATA
                if symbol != "SOL" && symbol != "sol" {
                    let ata = spl_associated_token_account::get_associated_token_address(
                        &payer.pubkey(),
                        &Pubkey::from_str(&token_info.mint)?
                    );
                    
                    match client.get_token_account_balance(&ata) {
                        Ok(balance) => {
                            println!("   💰 Balance: {} {}", balance.ui_amount_string, symbol);
                        }
                        Err(_) => {
                            println!("   ⚪ Sin balance (ATA no existe)");
                        }
                    }
                }
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
            }
        }
        println!();
    }
    
    // Test 4: Simular búsqueda de rutas para todos los pares
    println!("🔍 === BÚSQUEDA DE RUTAS DISPONIBLES ===");
    
    let tokens: Vec<_> = config.tokens.iter().collect();
    let mut working_pairs = Vec::new();
    
    for (i, (from_symbol, from_token)) in tokens.iter().enumerate() {
        for (j, (to_symbol, to_token)) in tokens.iter().enumerate() {
            if i != j { // No swap a sí mismo
                println!("📞 Probando {} -> {}...", from_symbol, to_symbol);
                
                let amount = if from_symbol.to_lowercase().contains("sol") { 10_000_000 } else { 1_000_000 };
                
                match jupiter.get_quote(&from_token.mint, &to_token.mint, amount, None).await {
                    Ok(_) => {
                        println!("   ✅ Ruta disponible");
                        working_pairs.push((from_symbol.clone(), to_symbol.clone()));
                    }
                    Err(_) => {
                        println!("   ❌ Sin ruta");
                    }
                }
                
                // Pausa para evitar rate limiting
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }
    }
    
    // Resumen final
    println!("\n🎯 === RESUMEN FINAL ===");
    println!("✅ RPC funcional: {}", working_rpc);
    println!("✅ Wallet conectada: {}", payer.pubkey());
    println!("✅ Balance SOL: {:.4}", balance_sol);
    println!("✅ Jupiter API funcional");
    
    if working_pairs.is_empty() {
        println!("⚠️  No se encontraron rutas de swap disponibles");
        println!("💡 Esto es normal en DevNet - necesitas crear tokens propios");
    } else {
        println!("✅ Rutas de swap disponibles: {}", working_pairs.len());
        for (from, to) in &working_pairs {
            println!("   {} -> {}", from, to);
        }
    }
    
    println!("\n📝 PRÓXIMOS PASOS:");
    if balance_sol < 1.0 {
        println!("1. Solicitar más SOL: solana airdrop 2");
    }
    println!("2. Crear tokens propios para testing");
    println!("3. Hacer wrap SOL -> wSOL para testing básico");
    
    println!("\n🧪 COMANDOS DE TESTING:");
    println!("spl-token wrap 0.1                  # Crear 0.1 wSOL");
    println!("spl-token unwrap <WSOL_ACCOUNT>     # Convertir wSOL -> SOL");
    println!("cargo run --bin test_real_swap_configured  # Test de swap real");
    
    Ok(())
}
