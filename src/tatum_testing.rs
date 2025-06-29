use anyhow::Result;
use crate::shared::tatum_rpc_client::TatumRpcClient;

/// Test Tatum RPC connectivity with proper authentication
pub async fn test_tatum_connectivity() -> Result<()> {
    println!("🔌 Testing Tatum RPC Integration");
    println!("================================");
    
    // Test mainnet endpoint
    if let Ok(api_key) = std::env::var("TATUM_API_KEY_MAINNET") {
        println!("📡 Testing Tatum Mainnet...");
        
        match TatumRpcClient::new(
            "https://solana-mainnet.gateway.tatum.io".to_string(), 
            api_key
        ) {
            Ok(client) => {
                match client.test_connection().await {
                    Ok(_) => {
                        println!("✅ Tatum Mainnet: Connection successful");
                        
                        // Test getting current slot
                        match client.get_slot().await {
                            Ok(slot) => println!("✅ Tatum Mainnet: Current slot: {}", slot),
                            Err(e) => println!("❌ Tatum Mainnet: Failed to get slot: {}", e),
                        }
                    }
                    Err(e) => println!("❌ Tatum Mainnet: Connection failed: {}", e),
                }
            }
            Err(e) => println!("❌ Tatum Mainnet: Client creation failed: {}", e),
        }
    } else {
        println!("⚠️ No TATUM_API_KEY_MAINNET found");
    }
    
    // Test devnet endpoint
    if let Ok(api_key) = std::env::var("TATUM_API_KEY_DEVNET") {
        println!("📡 Testing Tatum Devnet...");
        
        match TatumRpcClient::new(
            "https://solana-devnet.gateway.tatum.io".to_string(), 
            api_key
        ) {
            Ok(client) => {
                match client.test_connection().await {
                    Ok(_) => {
                        println!("✅ Tatum Devnet: Connection successful");
                        
                        // Test getting current slot
                        match client.get_slot().await {
                            Ok(slot) => println!("✅ Tatum Devnet: Current slot: {}", slot),
                            Err(e) => println!("❌ Tatum Devnet: Failed to get slot: {}", e),
                        }
                    }
                    Err(e) => println!("❌ Tatum Devnet: Connection failed: {}", e),
                }
            }
            Err(e) => println!("❌ Tatum Devnet: Client creation failed: {}", e),
        }
    } else {
        println!("⚠️ No TATUM_API_KEY_DEVNET found");
    }
    
    println!("🎉 Tatum connectivity test completed!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tatum_mainnet() {
        if std::env::var("TATUM_API_KEY_MAINNET").is_ok() {
            let result = test_tatum_connectivity().await;
            assert!(result.is_ok(), "Tatum connectivity test should pass");
        }
    }
}
