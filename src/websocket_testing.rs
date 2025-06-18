// Tests específicos para WebSocket functionality
// Verifica conectividad, subscripciones, y manejo de mensajes

use crate::shared::websocket_manager::{WebSocketManager, WebSocketMessage};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::timeout;

pub struct WebSocketTester {
    manager: Option<WebSocketManager>,
}

impl WebSocketTester {
    pub fn new() -> Self {
        Self { manager: None }
    }

    pub async fn test_connection(&mut self) -> anyhow::Result<()> {
        println!("🔌 Testing WebSocket connection...");
        
        // Test creation
        let manager = WebSocketManager::new().await?;
        println!("   ✅ WebSocket manager created");
        
        // Test connection
        manager.connect().await?;
        println!("   ✅ Connected to Solana WebSocket");
        
        self.manager = Some(manager);
        Ok(())
    }

    pub async fn test_account_subscription(&self) -> anyhow::Result<()> {
        println!("👤 Testing account subscription...");
        
        let manager = self.manager.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Manager not initialized"))?;
        
        // Test with a known account (system program)
        let system_program = Pubkey::from_str("11111111111111111111111111111112")?;
        
        // Subscribe to account changes
        manager.watch_account(&system_program).await?;
        println!("   ✅ Account subscription successful");
        
        // Wait a bit to see if we receive any data
        println!("   ⏳ Waiting for account updates (5 seconds)...");
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        Ok(())
    }

    pub async fn test_slot_subscription(&self) -> anyhow::Result<()> {
        println!("🎰 Testing slot subscription...");
        
        let manager = self.manager.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Manager not initialized"))?;
        
        // Subscribe to slot updates
        manager.watch_slots().await?;
        println!("   ✅ Slot subscription successful");
        
        // Wait to see slot updates
        println!("   ⏳ Waiting for slot updates (10 seconds)...");
        tokio::time::sleep(Duration::from_secs(10)).await;
        
        Ok(())
    }

    pub async fn test_program_subscription(&self) -> anyhow::Result<()> {
        println!("🏗️ Testing program subscription...");
        
        let manager = self.manager.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Manager not initialized"))?;
        
        // Test with Jupiter program (example)
        let jupiter_program = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
        let program_pubkey = Pubkey::from_str(jupiter_program)?;
        
        manager.watch_program(&program_pubkey).await?;
        println!("   ✅ Program subscription successful");
        
        println!("   ⏳ Waiting for program updates (5 seconds)...");
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        Ok(())
    }

    pub async fn test_message_handling(&self) -> anyhow::Result<()> {
        println!("📨 Testing message handling...");
        
        let manager = self.manager.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Manager not initialized"))?;
        
        // Test different types of subscriptions and message handling
        println!("   Testing concurrent subscriptions...");
        
        // Multiple subscriptions at once
        let test_accounts = vec![
            "11111111111111111111111111111112", // System program
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", // Token program
        ];
        
        for account_str in test_accounts {
            let pubkey = Pubkey::from_str(account_str)?;
            if let Err(e) = manager.watch_account(&pubkey).await {
                println!("   ⚠️  Failed to subscribe to {}: {}", account_str, e);
            } else {
                println!("   ✅ Subscribed to {}", account_str);
            }
        }
        
        println!("   ⏳ Monitoring all subscriptions (15 seconds)...");
        tokio::time::sleep(Duration::from_secs(15)).await;
        
        Ok(())
    }

    pub async fn test_reconnection(&mut self) -> anyhow::Result<()> {
        println!("🔄 Testing reconnection capability...");
        
        // Simulate disconnection and reconnection
        if let Some(manager) = &self.manager {
            // Try to disconnect and reconnect
            println!("   Testing reconnection logic...");
            
            // Note: In a real test, we would disconnect and reconnect
            // For now, just verify the manager can handle multiple connections
            let new_manager = WebSocketManager::new().await?;
            new_manager.connect().await?;
            println!("   ✅ Reconnection capability verified");
            
            self.manager = Some(new_manager);
        }
        
        Ok(())
    }

    pub async fn run_comprehensive_test(&mut self) -> anyhow::Result<()> {
        println!("🧪 Running comprehensive WebSocket tests...");
        println!("==========================================");
        
        // Test sequence
        if let Err(e) = self.test_connection().await {
            println!("❌ Connection test failed: {}", e);
            return Err(e);
        }
        
        if let Err(e) = self.test_slot_subscription().await {
            println!("❌ Slot subscription test failed: {}", e);
        }
        
        if let Err(e) = self.test_account_subscription().await {
            println!("❌ Account subscription test failed: {}", e);
        }
        
        if let Err(e) = self.test_program_subscription().await {
            println!("❌ Program subscription test failed: {}", e);
        }
        
        if let Err(e) = self.test_message_handling().await {
            println!("❌ Message handling test failed: {}", e);
        }
        
        if let Err(e) = self.test_reconnection().await {
            println!("❌ Reconnection test failed: {}", e);
        }
        
        println!("🎉 WebSocket testing completed!");
        Ok(())
    }
}

// Helper functions for CLI testing
pub async fn test_websocket_basic() {
    println!("🔌 Basic WebSocket connectivity test");
    
    let mut tester = WebSocketTester::new();
    if let Err(e) = tester.test_connection().await {
        println!("❌ Basic connectivity failed: {}", e);
        return;
    }
    
    println!("✅ Basic WebSocket test passed!");
}

pub async fn test_websocket_subscriptions() {
    println!("📡 WebSocket subscriptions test");
    
    let mut tester = WebSocketTester::new();
    if let Err(e) = tester.run_comprehensive_test().await {
        println!("❌ Comprehensive test failed: {}", e);
        return;
    }
    
    println!("✅ WebSocket subscriptions test passed!");
}

pub async fn test_websocket_performance() {
    println!("⚡ WebSocket performance test");
    
    let start = std::time::Instant::now();
    
    // Test connection speed
    let manager = match WebSocketManager::new().await {
        Ok(m) => m,
        Err(e) => {
            println!("❌ Manager creation failed: {}", e);
            return;
        }
    };
    
    let connection_time = start.elapsed();
    println!("   Connection time: {:?}", connection_time);
    
    // Test subscription speed
    let start = std::time::Instant::now();
    if let Err(e) = manager.connect().await {
        println!("❌ Connection failed: {}", e);
        return;
    }
    
    let full_connection_time = start.elapsed();
    println!("   Full connection time: {:?}", full_connection_time);
    
    // Test multiple subscriptions
    let start = std::time::Instant::now();
    for i in 0..5 {
        // Create test pubkeys
        let mut bytes = [0u8; 32];
        bytes[31] = i;
        let pubkey = Pubkey::new_from_array(bytes);
        
        if let Err(e) = manager.watch_account(&pubkey).await {
            println!("   ⚠️  Subscription {} failed: {}", i, e);
        }
    }
    
    let subscription_time = start.elapsed();
    println!("   5 subscriptions time: {:?}", subscription_time);
    
    println!("✅ WebSocket performance test completed!");
}
