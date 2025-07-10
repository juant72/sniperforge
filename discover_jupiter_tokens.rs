use anyhow::Result;
use sniperforge::shared::jupiter_api::Jupiter;
use sniperforge::shared::jupiter_config::JupiterConfig;
use std::env;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    info!("🚀 === DESCUBRIENDO TOKENS VÁLIDOS EN JUPITER DEVNET ===");
    info!("======================================================");

    // Create Jupiter client
    let rpc_url = env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://solana-devnet.g.alchemy.com/v2/X64q4zZFEMz_RYzthxUMg".to_string());
    
    let jupiter_config = JupiterConfig {
        base_url: "https://quote-api.jup.ag".to_string(),
        api_key: None,
        timeout_seconds: 60,
        max_retries: 3,
        rpc_endpoint: rpc_url,
        network_name: "devnet".to_string(),
    };

    let jupiter = Jupiter::new(&jupiter_config).await?;
    info!("✅ Jupiter client inicializado");

    // Get token list from Jupiter
    info!("\n🔍 Obteniendo lista de tokens de Jupiter...");
    
    // Try to get token list (this is a hypothetical endpoint)
    // Since we don't have direct access to token list, let's test known tokens
    
    let test_tokens = vec![
        ("SOL", "So11111111111111111111111111111111111111112"),
        ("wSOL", "So11111111111111111111111111111111111111112"),
        ("USDC", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        ("USDT", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
        ("RAY", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"),
        ("BONK", "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"),
        ("JUP", "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN"),
        ("ORCA", "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE"),
        ("SERUM", "SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt"),
        ("MNGO", "MangoCzJ36AjZyKwVj3VnYU4GTonjfVEnJmvvWaxLac"),
        ("STEPN", "StepAscQoEioFxxWGnh2sLBDFp9d8rvKz2Yp39iDpyT"),
        ("ATLAS", "ATLASXmbPQxBUYbxPsV97usA3fPQYEqzQBUHgiFCUsXx"),
        ("POLIS", "poLisWXnNRwC6oBu1vHiuKQzFjGL4XDSu4g9qjz9qVk"),
        ("SAMO", "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"),
        ("COPE", "8HGyAAB1yoM1ttS7pXjHMa3dukTFGQggnFFH3hJZgzQh"),
        ("FIDA", "EchesyfXePKdLtoiZSL8pBe8Myagyy8ZRqsACNCFGnvp"),
        ("ROPE", "8PMHT4swUMtBzgHnh5U564N5sjPSiUz2cjEQzFnnP1Fo"),
        ("MEDIA", "ETAtLmCmsoiEEKfNrHKJ2kYy3MoABhU6NQvpSfij5tDs"),
        ("STAR", "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"),
        ("SBR", "Saber2gLauYim4Mvftnrasomsv6NvAuncvMEZwcLpD1"),
        ("PORT", "PoRTjZMPXb9T7dyU7tpLEZRQj7e6ssfAE62j2oQuc6y"),
        ("TULIP", "TuLipcqtGVXP9XR62wM8WWCm6a9vhLs7T1uoWBk6FDs"),
        ("SUNNY", "SUNNYWgPQmFxe9wTZzNK7iPnJ3vYDrkgnxJRJm1s3ag"),
        ("SLRS", "SLRSSpSLUTP7okbCUBYStWCo1vUgyt775faPqz8HUMr"),
        ("SLIM", "xxxxa1sKNGwFtw2kFn8XauW9xq8hBZ5kVtcSesTT9fW"),
        ("GRAPE", "8upjSpvjcdpuzhfR1zriwg5NXkwDruejqNE9WNbPRtyA"),
        ("LARIX", "Lrxqnh6ZHKbGy3dcrCqnPjF5xXEUhJHNYoqQBF2oQJzB"),
        ("PRTC", "PRTCNhbyAJNmS9ddXXEJKCQsYuVpZbKcqAHh9cJmVYs"),
        ("CRWNY", "CRWNYkqdgvhGGae9CKfNka58j6QQjgCG5tzu3sAVrJe"),
        ("SONAR", "sonarX4VtVkQemriJeLm6CKeW3GDMyiBnnAEMw1MRAE"),
        ("WOOF", "9nEqaUcb16sQ3Tn1psbkWqyhPdLmfHWjKGymREjsAgTE"),
        ("CHEEMS", "3FoUAsGDbvTD6YZ4wVKJgTB76onJUKz7GPEBNiR5b8wc"),
        ("DEGEN", "DGNNSTMVs4BPWCvhhTJGWFuqWiTLJUJSGVVwFKNT8S6S"),
        ("SILLY", "SiLLY7NmCjGrwLTBPxoZkL7SZzV6wQELcJWmBzEPRWn"),
        ("ROOST", "RoOST1FTfRdJVnGCnqx5HJXGQgE2jKQWqAzfvEUhSNt"),
        ("FOXY", "FoXyMu5xwXre7zEoSvzViRk3nGawHUp9kUh97y2NDhcq"),
        ("FROG", "FRoGjEZVgE5kAZoQNQrjKEzaYQZvkjpbE5kGkmWvDJg"),
        ("GOAT", "GoATqsWjkYxDxbFaP3qjTEaJQmQ7qRMnXuT9qjRxNT5"),
        ("SHDW", "SHDWyBxihqiCj6YekG2GUr7wqKLeLAMK1gHZck9pL6y"),
        ("DUST", "DUSTawucrTsGU8hcqRdHDCbuYhCPADMLM2VcCb8VnFnQ"),
        ("PUFF", "PuffyngtQmKbL3JNWCpZi6yGwHCJUpGdKpKhCFgJSgK"),
        ("WAIFU", "WaifuLzEprQYUAfQgMdFqLZKFNrpjCVB7kTBXqQF9on"),
        ("MOONLANA", "MoonLanaBDXjQ6mBuNKXPJW2oYVtXgzgBGDhVWfFfXj"),
        ("MAFIA", "MafiaDBgQB9bZFXJq4mKwKCjP3CjUYqZCVgtGLRNnBV"),
        ("PEPE", "PepePWjmGWkSLKBL5QzWJGVjdJMnpvkSKFLNroBLHJe"),
        ("SHIB", "SHIBLJLxjY7nq8pxbKh9FfbwJNkTHrFRfQKrVvRzWvY"),
        ("DOGE", "DogeCPwFgMHqZiQDfvRoZANgVMFzZqYpx9kNEQRrZzz"),
        ("FLOKI", "FLOKiJqYQmWxNnCKCMGqRzjGrxRJGnxPFxQKFR2n5sA"),
        ("ELON", "ELONJqYQmWxNnCKCMGqRzjGrxRJGnxPFxQKFR2n5sA"),
    ];

    info!("📊 Probando {} tokens potenciales...", test_tokens.len());

    let mut working_tokens = Vec::new();
    let mut failed_tokens = Vec::new();

    for (symbol, mint) in test_tokens {
        info!("🔍 Probando {}: {}", symbol, mint);
        
        // Try to get a quote from SOL to this token
        match jupiter.get_quote(
            "So11111111111111111111111111111111111111112", // SOL
            mint,
            0.001, // 0.001 SOL
            100 // 1% slippage
        ).await {
            Ok(quote) => {
                let output_amount = quote.outAmount.parse::<u64>().unwrap_or(0);
                info!("  ✅ {} FUNCIONA! Output: {}", symbol, output_amount);
                working_tokens.push((symbol, mint));
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("TOKEN_NOT_TRADABLE") {
                    info!("  ❌ {} no es tradeable", symbol);
                } else if error_msg.contains("CIRCULAR_ARBITRAGE_IS_DISABLED") {
                    info!("  ⚠️ {} es el mismo token (circular)", symbol);
                } else {
                    info!("  ❌ {} error: {}", symbol, error_msg);
                }
                failed_tokens.push((symbol, mint, error_msg));
            }
        }
        
        // Small delay to avoid rate limiting
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    // Summary
    info!("\n📊 === RESUMEN ===");
    info!("✅ Tokens que funcionan: {}", working_tokens.len());
    for (symbol, mint) in &working_tokens {
        info!("  {} - {}", symbol, mint);
    }
    
    info!("❌ Tokens que no funcionan: {}", failed_tokens.len());
    for (symbol, _mint, error) in &failed_tokens {
        if error.contains("TOKEN_NOT_TRADABLE") {
            info!("  {} - No tradeable", symbol);
        } else if error.contains("CIRCULAR_ARBITRAGE_IS_DISABLED") {
            info!("  {} - Circular", symbol);
        } else {
            info!("  {} - Otro error", symbol);
        }
    }

    if !working_tokens.is_empty() {
        info!("\n🎯 === RECOMENDACIONES ===");
        info!("✅ Usa estos tokens para pruebas reales:");
        for (symbol, mint) in working_tokens {
            info!("  {} - {}", symbol, mint);
        }
    } else {
        info!("\n⚠️ === PROBLEMA ===");
        info!("❌ No se encontraron tokens funcionales en Jupiter DevNet");
        info!("💡 Considera:");
        info!("  1. Usar AMM directo (Orca, Raydium)");
        info!("  2. Crear pools de liquidez");
        info!("  3. Migrar a MainNet para pruebas");
    }

    Ok(())
}
