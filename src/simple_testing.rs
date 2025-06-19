// Simplified testing for current API compatibility

use anyhow::Result;

pub async fn test_websocket_basic() {
    println!("🔌 Basic WebSocket connectivity test");
    
    // Load config
    let config = match crate::Config::load("config/devnet.toml") {
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
            
            // Test connection status
            if manager.is_connected().await {
                println!("✅ WebSocket is connected");
            } else {
                println!("⚠️  WebSocket manager created but not actively connected");
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
        Ok(jupiter_client) => {            match jupiter_client.get_price("So11111111111111111111111111111111111111112").await {
                Ok(Some(price)) => {
                    println!("✅ OK");
                    println!("   SOL price: ${:.2}", price);
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
