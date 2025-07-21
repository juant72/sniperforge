use anyhow::Result;
use reqwest;
use serde_json;
use std::time::Duration;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("🔍 DIAGNÓSTICO DEL SISTEMA DE ARBITRAJE MILITAR");
    println!("===============================================");

    // Test 1: RPC Connection
    println!("1. 🌐 Probando conexión RPC...");
    if let Err(e) = test_rpc_connection().await {
        println!("   ❌ Error en RPC: {}", e);
    }

    // Test 2: Jupiter API
    println!("2. 🪐 Probando Jupiter API...");
    if let Err(e) = test_jupiter_api().await {
        println!("   ❌ Error en Jupiter: {}", e);
    }

    // Test 3: Raydium API
    println!("3. 🌊 Probando Raydium API...");
    if let Err(e) = test_raydium_api().await {
        println!("   ❌ Error en Raydium: {}", e);
    }

    // Test 4: Orca API
    println!("4. 🐋 Probando Orca API...");
    if let Err(e) = test_orca_api().await {
        println!("   ❌ Error en Orca: {}", e);
    }

    // Test 5: DexScreener API
    println!("5. 📊 Probando DexScreener API...");
    if let Err(e) = test_dexscreener_api().await {
        println!("   ❌ Error en DexScreener: {}", e);
    }

    // Test 6: Wallet Load
    println!("6. 💳 Probando carga de wallet...");
    if let Err(e) = test_wallet_load().await {
        println!("   ❌ Error en wallet: {}", e);
    }

    println!("✅ DIAGNÓSTICO COMPLETADO");
    Ok(())
}

async fn test_rpc_connection() -> Result<()> {
    use solana_client::rpc_client::RpcClient;
    use solana_sdk::commitment_config::CommitmentConfig;

    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    println!("   📡 Conectando a: {}", rpc_url);
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::finalized());

    match client.get_version() {
        Ok(version) => println!("   ✅ RPC conectado - Versión: {}", version.solana_core),
        Err(e) => {
            println!("   ❌ RPC falló: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

async fn test_jupiter_api() -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let url = "https://quote-api.jup.ag/v6/quote";
    let params = [
        ("inputMint", "So11111111111111111111111111111111111111112"),
        ("outputMint", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        ("amount", "1000000000"),
        ("slippageBps", "50"),
    ];

    println!("   📡 Consultando Jupiter API...");

    match client.get(url).query(&params).send().await {
        Ok(response) => {
            let status = response.status();
            println!("   📊 Estado: {}", status);

            if status.is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        println!("   ✅ Jupiter API funcionando");
                        if let Some(route_plan) = data.get("routePlan") {
                            println!(
                                "   📋 Rutas encontradas: {}",
                                route_plan.as_array().unwrap_or(&vec![]).len()
                            );
                        }
                    }
                    Err(e) => println!("   ⚠️  Error parseando JSON: {}", e),
                }
            } else {
                println!("   ❌ Jupiter API falló con estado: {}", status);
            }
        }
        Err(e) => {
            println!("   ❌ Error de conexión Jupiter: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

async fn test_raydium_api() -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30)) // Timeout más largo para Raydium
        .build()?;

    let url = "https://api.raydium.io/pairs";

    println!("   📡 Consultando Raydium API...");
    println!("   ⏱️  Timeout: 30 segundos");

    match client.get(url).send().await {
        Ok(response) => {
            let status = response.status();
            println!("   📊 Estado: {}", status);

            if status.is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        if let Some(pairs) = data.as_array() {
                            println!("   ✅ Raydium API funcionando");
                            println!("   📋 Pairs encontrados: {}", pairs.len());

                            // Mostrar algunos ejemplos
                            for (i, pair) in pairs.iter().take(3).enumerate() {
                                if let (Some(amm_id), Some(official)) =
                                    (pair.get("amm_id"), pair.get("official"))
                                {
                                    println!(
                                        "   {}. Pool: {} (oficial: {})",
                                        i + 1,
                                        amm_id.as_str().unwrap_or("N/A"),
                                        official.as_bool().unwrap_or(false)
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => println!("   ⚠️  Error parseando JSON: {}", e),
                }
            } else {
                println!("   ❌ Raydium API falló con estado: {}", status);
            }
        }
        Err(e) => {
            println!("   ❌ Error de conexión Raydium: {}", e);
            // No fallar el diagnóstico completo por una API
        }
    }

    Ok(())
}

async fn test_orca_api() -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?;

    let url = "https://api.orca.so/v1/whirlpool/list";

    println!("   📡 Consultando Orca API...");

    match client.get(url).send().await {
        Ok(response) => {
            let status = response.status();
            println!("   📊 Estado: {}", status);

            if status.is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        println!("   ✅ Orca API funcionando");
                        if let Some(whirlpools) = data.get("whirlpools") {
                            if let Some(pools) = whirlpools.as_array() {
                                println!("   📋 Whirlpools encontrados: {}", pools.len());
                            }
                        }
                    }
                    Err(e) => println!("   ⚠️  Error parseando JSON: {}", e),
                }
            } else {
                println!("   ❌ Orca API falló con estado: {}", status);
            }
        }
        Err(e) => {
            println!("   ❌ Error de conexión Orca: {}", e);
        }
    }

    Ok(())
}

async fn test_dexscreener_api() -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?;

    let url =
        "https://api.dexscreener.com/latest/dex/tokens/So11111111111111111111111111111111111111112";

    println!("   📡 Consultando DexScreener API...");

    match client.get(url).send().await {
        Ok(response) => {
            let status = response.status();
            println!("   📊 Estado: {}", status);

            if status.is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        println!("   ✅ DexScreener API funcionando");
                        if let Some(pairs) = data.get("pairs") {
                            if let Some(pairs_array) = pairs.as_array() {
                                println!("   📋 Pairs encontrados: {}", pairs_array.len());
                            }
                        }
                    }
                    Err(e) => println!("   ⚠️  Error parseando JSON: {}", e),
                }
            } else {
                println!("   ❌ DexScreener API falló con estado: {}", status);
            }
        }
        Err(e) => {
            println!("   ❌ Error de conexión DexScreener: {}", e);
        }
    }

    Ok(())
}

async fn test_wallet_load() -> Result<()> {
    let wallet_path = "mainnet_wallet.json";

    println!("   📁 Cargando wallet desde: {}", wallet_path);

    match std::fs::read_to_string(wallet_path) {
        Ok(json_str) => match serde_json::from_str::<Vec<u8>>(&json_str) {
            Ok(keypair_bytes) => {
                use solana_sdk::signer::keypair::Keypair;
                use solana_sdk::signer::Signer;

                match Keypair::from_bytes(&keypair_bytes) {
                    Ok(keypair) => {
                        println!("   ✅ Wallet cargado exitosamente");
                        println!("   🔑 Dirección: {}", keypair.pubkey());
                    }
                    Err(e) => {
                        println!("   ❌ Error creando keypair: {}", e);
                        return Err(e.into());
                    }
                }
            }
            Err(e) => {
                println!("   ❌ Error parseando JSON del wallet: {}", e);
                return Err(e.into());
            }
        },
        Err(e) => {
            println!("   ❌ Error leyendo archivo wallet: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
