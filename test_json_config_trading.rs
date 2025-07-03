// Test de swap real usando configuración JSON paramétrica
// Sistema completamente configurable sin hardcoding

use dotenv::dotenv;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::Keypair,
    signer::Signer,
    commitment_config::CommitmentConfig,
};
use std::env;
use std::error::Error;
use sniperforge::shared::config_loader::NetworkConfigFile;
use sniperforge::shared::jupiter_api::Jupiter;
use sniperforge::shared::jupiter_config::JupiterConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    
    println!("🎯 TESTING REAL SWAP WITH JSON CONFIGURATION");
    println!("==============================================");
    
    // Cargar configuración desde archivo JSON
    println!("📂 Loading network configuration from JSON...");
    let network = env::var("NETWORK").unwrap_or_else(|_| "devnet".to_string());
    println!("   Network: {}", network);
    
    let config_file = NetworkConfigFile::load_by_network(&network)?;
    println!("✅ Configuration loaded: {}", config_file.display_name);
    
    // Validar configuración
    config_file.validate()?;
    println!("✅ Configuration validated");
    
    // Parsear configuración
    let config = config_file.parse()?;
    println!("✅ Configuration parsed");
    
    // Verificar si las transacciones reales están habilitadas
    if !config.transactions_enabled() {
        println!("⚠️  Real transactions are DISABLED in configuration");
        println!("   To enable, set 'enable_real_transactions': true in config/{}.json", network);
        println!("   Simulation mode only.");
    } else {
        println!("🔓 Real transactions are ENABLED");
    }
    
    // Configurar RPC client
    let rpc_client = RpcClient::new_with_commitment(
        &config.rpc_endpoint, 
        CommitmentConfig::confirmed()
    );
    
    // Cargar wallet desde environment
    println!("\n📂 Loading wallet from environment...");
    let private_key = env::var("SOLANA_PRIVATE_KEY")
        .expect("SOLANA_PRIVATE_KEY not found in environment");
    
    let private_key_bytes = bs58::decode(private_key)
        .into_vec()
        .expect("Failed to decode private key");
    
    let wallet = Keypair::from_bytes(&private_key_bytes)
        .expect("Failed to create wallet from private key");
    
    println!("✅ Wallet loaded successfully");
    println!("🔑 Public key: {}", wallet.pubkey());
    
    // Verificar balance
    println!("\n💰 Checking wallet balance...");
    let balance = rpc_client.get_balance(&wallet.pubkey())?;
    let sol_balance = balance as f64 / 1_000_000_000.0;
    println!("   Balance: {} SOL", sol_balance);
    
    let min_balance = config.safety_limits.min_balance_reserve_sol;
    if sol_balance < min_balance {
        println!("❌ Insufficient balance for swap test");
        println!("   Required minimum: {} SOL", min_balance);
        return Ok(());
    }
    
    // Obtener el primer par de trading habilitado
    let trading_pairs = config.get_enabled_trading_pairs();
    if trading_pairs.is_empty() {
        println!("❌ No enabled trading pairs found in configuration");
        return Ok(());
    }
    
    let trading_pair = &trading_pairs[0];
    println!("\n📊 Selected trading pair: {} -> {}", 
             trading_pair.base.to_uppercase(), 
             trading_pair.quote.to_uppercase());
    
    // Obtener tokens de la configuración
    let base_token = config.get_token(&trading_pair.base)
        .ok_or_else(|| format!("Base token '{}' not found in configuration", trading_pair.base))?;
    let quote_token = config.get_token(&trading_pair.quote)
        .ok_or_else(|| format!("Quote token '{}' not found in configuration", trading_pair.quote))?;
    
    println!("   Base:  {} ({}) - {}", base_token.symbol, base_token.name, base_token.address);
    println!("   Quote: {} ({}) - {}", quote_token.symbol, quote_token.name, quote_token.address);
    
    // Verificar que los tokens sean tradeables
    if !base_token.tradeable || !quote_token.tradeable {
        println!("❌ One or both tokens are marked as non-tradeable in configuration");
        return Ok(());
    }
    
    // Configurar Jupiter
    let jupiter_config = JupiterConfig {
        base_url: config.dex_config.jupiter.base_url.clone()
            .unwrap_or_else(|| "https://lite-api.jup.ag".to_string()),
        api_key: None,
        timeout_seconds: config.dex_config.jupiter.timeout_seconds.unwrap_or(30),
        max_retries: config.dex_config.jupiter.max_retries.unwrap_or(3),
        rpc_endpoint: config.rpc_endpoint.clone(),
        network_name: config.network.clone(),
    };
    
    println!("\n🌍 Initializing Jupiter API...");
    let jupiter = Jupiter::new(&jupiter_config).await?;
    println!("✅ Jupiter API connected");
    
    // Determinar cantidad de swap usando límites de configuración
    let swap_amount = trading_pair.min_trade_amount; // Usar exactamente el mínimo configurado
    let max_allowed = config.max_swap_amount();
    
    if swap_amount > max_allowed {
        println!("❌ Swap amount ({}) exceeds safety limit ({})", swap_amount, max_allowed);
        return Ok(());
    }
    
    println!("\n📊 Getting quote from Jupiter...");
    println!("   Input: {} {}", swap_amount, base_token.symbol.to_uppercase());
    println!("   From: {} ({})", base_token.address, base_token.symbol);
    println!("   To: {} ({})", quote_token.address, quote_token.symbol);
    println!("   Slippage: {}bps", trading_pair.default_slippage_bps);
    
    // Obtener quote de Jupiter
    let quote = jupiter.get_quote(
        &base_token.address.to_string(),
        &quote_token.address.to_string(),
        swap_amount,
        trading_pair.default_slippage_bps,
    ).await?;
    
    println!("✅ Quote received from Jupiter");
    println!("   Output: {} {} tokens", quote.out_amount(), quote_token.symbol.to_uppercase());
    println!("   Price Impact: {}%", quote.price_impact_pct());
    println!("   Route: {} steps", quote.routePlan.len());
    
    // Mostrar detalles de la ruta
    if config.feature_flags.verbose_logging {
        println!("\n📍 Route Details:");
        for (i, step) in quote.routePlan.iter().enumerate() {
            let swap_info = &step.swapInfo;
            println!("   Step {}: {} -> {} via {}", 
                     i + 1,
                     swap_info.inputMint,
                     swap_info.outputMint,
                     swap_info.label);
        }
    }
    
    // Ejecutar swap o simulación
    println!("\n🚀 EXECUTING SWAP...");
    
    if config.transactions_enabled() {
        println!("💳 Executing REAL transaction on {}", config.display_name);
        
        match jupiter.execute_swap_with_wallet(&quote, &wallet.pubkey().to_string(), Some(&wallet)).await {
            Ok(result) => {
                if result.success {
                    println!("✅ SWAP EXECUTED SUCCESSFULLY!");
                    println!("📝 Transaction signature: {}", result.transaction_signature);
                    println!("🔗 View on Solana Explorer:");
                    println!("   {}", config.get_explorer_url(&result.transaction_signature));
                    
                    // Verificar nuevo balance
                    println!("\n💰 Checking new balance...");
                    let new_balance = rpc_client.get_balance(&wallet.pubkey())?;
                    let new_sol_balance = new_balance as f64 / 1_000_000_000.0;
                    println!("   Previous balance: {} SOL", sol_balance);
                    println!("   New balance: {} SOL", new_sol_balance);
                    println!("   Difference: {} SOL", new_sol_balance - sol_balance);
                } else {
                    println!("❌ SWAP EXECUTION FAILED");
                    println!("   Transaction signature: {}", result.transaction_signature);
                    println!("   Logs:");
                    for log in result.logs {
                        println!("     {}", log);
                    }
                }
            }
            Err(e) => {
                println!("❌ SWAP EXECUTION FAILED");
                println!("   Reason: {}", e);
                
                // Analizar el error usando configuración
                let error_str = e.to_string();
                if error_str.contains("Route not found") {
                    println!("\n🔍 Analysis: No trading route found");
                    println!("   • The token pair may not be available on {}", config.display_name);
                    println!("   • Try a different trading pair from the configuration");
                    println!("   • Check if the tokens are actually deployed on this network");
                } else if error_str.contains("IncorrectProgramId") {
                    println!("\n🔍 Analysis: Program ID mismatch detected");
                    println!("   • Token accounts may not exist for this wallet");
                    println!("   • Token mint may not be properly initialized on {}", config.display_name);
                    println!("   • Check program_ids configuration in config/{}.json", network);
                }
            }
        }
    } else {
        println!("🔧 SIMULATION MODE (Real transactions disabled)");
        println!("   The swap quote was successful, but execution is disabled");
        println!("   To enable real transactions, update config/{}.json:", network);
        println!("   Set 'enable_real_transactions': true");
        println!("✅ Simulation completed successfully");
    }
    
    Ok(())
}
