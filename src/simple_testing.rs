// Simplified testing for current API compatibility

use anyhow::Result;

pub async fn test_websocket_basic() {
    test_websocket_with_network("devnet").await;
}

pub async fn test_websocket_with_network(network: &str) {
    println!("🔌 Basic WebSocket connectivity test");
    
    // Load config based on network
    let config_file = match network {
        "mainnet" => "config/mainnet.toml",
        "devnet" => "config/devnet.toml",
        _ => "config/devnet.toml", // Default fallback
    };
    
    let config = match crate::Config::load(config_file) {
        Ok(c) => c,
        Err(_) => match crate::Config::load("config/platform.toml") {
            Ok(c) => c,
            Err(e) => {
                println!("❌ Could not load config: {}", e);
                return;
            }
        }
    };
    
    // Test WebSocket manager creation
    match crate::shared::websocket_manager::WebSocketManager::new(&config).await {
        Ok(manager) => {
            println!("✅ WebSocket manager created successfully");
            
            // Give WebSocket time to connect (short timeout)
            println!("🔄 Waiting for WebSocket connection...");
            for i in 1..=3 {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                if manager.is_connected().await {
                    println!("✅ WebSocket connected successfully");
                    break;
                } else if i == 3 {
                    println!("⚠️  WebSocket manager created but connection still in progress");
                    println!("   (This is normal for quick tests - connection happens in background)");
                }
            }
        }
        Err(e) => {
            println!("❌ WebSocket manager creation failed: {}", e);
        }
    }
}

pub async fn test_basic_integration() {
    println!("🧪 Basic Integration Test");
    println!("=========================");
    
    // Test config loading
    print!("📋 Testing config loading... ");
    match crate::Config::load("config/devnet.toml").or_else(|_| crate::Config::load("config/platform.toml")) {
        Ok(_) => println!("✅ OK"),
        Err(e) => {
            println!("❌ FAILED: {}", e);
            return;
        }
    }
      // Test Solana connectivity
    print!("🌐 Testing Solana connectivity... ");
    let config = match crate::Config::load("config/devnet.toml") {
        Ok(c) => c,
        Err(_) => match crate::Config::load("config/platform.toml") {
            Ok(c) => c,
            Err(_) => {
                println!("❌ FAILED: Could not load config");
                return;
            }
        }
    };
    
    match crate::solana_testing::test_solana_connectivity(&config).await {
        Ok(_) => println!("✅ OK"),
        Err(e) => println!("❌ FAILED: {}", e),
    }
      // Test Jupiter client
    print!("🪐 Testing Jupiter client... ");
    let jupiter_config = crate::shared::jupiter::JupiterConfig::default();
    match crate::shared::jupiter::JupiterClient::new(&jupiter_config).await {
        Ok(jupiter_client) => {
            // Test SOL price
            match jupiter_client.get_price("So11111111111111111111111111111111111111112").await {
                Ok(Some(price)) => {
                    println!("✅ OK");
                    println!("   SOL price: ${:.2}", price);
                    
                    // Test multiple tokens
                    print!("   🧪 Testing multiple tokens... ");
                    let test_tokens = vec![
                        ("USDC", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
                        ("RAY", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"),
                        ("USDT", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
                    ];
                    
                    let mut success_count = 0;
                    for (symbol, mint) in test_tokens {
                        match jupiter_client.get_price(mint).await {
                            Ok(Some(token_price)) => {
                                println!("   📊 {} price: ${:.6}", symbol, token_price);
                                success_count += 1;
                            }
                            Ok(None) => {
                                println!("   ⚠️  {} price: No data", symbol);
                            }
                            Err(e) => {
                                println!("   ❌ {} price failed: {}", symbol, e);
                            }
                        }
                    }
                    
                    if success_count >= 2 {
                        println!("   ✅ Multi-token test passed ({}/3 tokens)", success_count);
                    } else {
                        println!("   ⚠️  Multi-token test partial ({}/3 tokens)", success_count);
                    }
                }
                Ok(None) => {
                    println!("⚠️  OK but no price data available");
                }
                Err(e) => println!("❌ FAILED: {}", e),
            }
        }
        Err(e) => println!("❌ Jupiter client creation failed: {}", e),
    }
    
    // Test WebSocket
    print!("🔌 Testing WebSocket... ");
    test_websocket_basic().await;
    
    println!("🎉 Basic integration test completed!");
}

pub async fn run_simple_tests() {
    println!("🧪 Running Simple Test Suite");
    println!("============================");
    
    // Initialize crypto provider first to prevent rustls panics
    init_crypto_provider_early();
    
    test_basic_integration().await;
}

fn init_crypto_provider_early() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    
    INIT.call_once(|| {
        println!("🔐 Initializing crypto provider for tests...");
        
        // Try to install the ring crypto provider
        let result = rustls::crypto::ring::default_provider().install_default();
        
        match result {
            Ok(()) => {
                println!("✅ Ring crypto provider installed successfully");
            }
            Err(_) => {
                // Provider was already installed, which is fine
                println!("ℹ️  Crypto provider was already installed");
            }
        }
    });
}
