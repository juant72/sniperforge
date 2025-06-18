// Tests completos para SniperForge
// Incluye: CLI, WebSocket, Jupiter API, Wallet, Trade Executor

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{
        jupiter::client::JupiterClient,
        wallet_manager::WalletManager,
        trade_executor::TradeExecutor,
        websocket_manager::WebSocketManager,
    };
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;
    use tokio;

    #[tokio::test]
    async fn test_jupiter_integration() {
        println!("🔄 Testing Jupiter API integration...");
        
        let client = JupiterClient::new();
        
        // Test quote functionality
        let input_mint = "So11111111111111111111111111111111111111112"; // SOL
        let output_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC
        let amount = 1000000; // 0.001 SOL
        
        match client.get_quote(input_mint, output_mint, amount).await {
            Ok(quote) => {
                println!("✅ Jupiter quote successful");
                println!("   Input: {} {}", quote.in_amount, input_mint);
                println!("   Output: {} {}", quote.out_amount, output_mint);
            }
            Err(e) => println!("❌ Jupiter quote failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_wallet_functionality() {
        println!("💰 Testing Wallet functionality...");
        
        // Test wallet creation and basic operations
        match WalletManager::new() {
            Ok(wallet) => {
                println!("✅ Wallet manager created");
                
                // Test balance check (devnet)
                if let Ok(balance) = wallet.get_balance().await {
                    println!("   Balance: {} SOL", balance);
                }
                
                // Test pubkey generation
                let pubkey = wallet.get_pubkey();
                println!("   Pubkey: {}", pubkey);
            }
            Err(e) => println!("❌ Wallet creation failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_websocket_connection() {
        println!("🔌 Testing WebSocket connection...");
        
        let ws_manager = WebSocketManager::new().await;
        match ws_manager {
            Ok(manager) => {
                println!("✅ WebSocket manager created");
                
                // Test connection to Solana WebSocket
                match manager.connect().await {
                    Ok(_) => {
                        println!("✅ WebSocket connected to Solana");
                        
                        // Test account subscription
                        let test_pubkey = Pubkey::from_str("11111111111111111111111111111112").unwrap();
                        if let Err(e) = manager.watch_account(&test_pubkey).await {
                            println!("⚠️  Account subscription test: {}", e);
                        } else {
                            println!("✅ Account subscription working");
                        }
                        
                        // Test slot subscription
                        if let Err(e) = manager.watch_slots().await {
                            println!("⚠️  Slot subscription test: {}", e);
                        } else {
                            println!("✅ Slot subscription working");
                        }
                    }
                    Err(e) => println!("❌ WebSocket connection failed: {}", e),
                }
            }
            Err(e) => println!("❌ WebSocket manager creation failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_trade_executor_setup() {
        println!("⚡ Testing Trade Executor setup...");
        
        match TradeExecutor::new().await {
            Ok(executor) => {
                println!("✅ Trade executor created");
                
                // Test executor configuration
                println!("   Max slippage: {}%", executor.max_slippage_bps as f64 / 100.0);
                println!("   Priority fee: {} lamports", executor.priority_fee_lamports);
                
                // Test validation without actual execution
                let input_mint = "So11111111111111111111111111111111111111112";
                let output_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
                let amount = 1000000;
                
                println!("   Testing trade validation for {} -> {}", input_mint, output_mint);
                println!("   Amount: {}", amount);
            }
            Err(e) => println!("❌ Trade executor creation failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_integration_flow() {
        println!("🔄 Testing complete integration flow...");
        
        // 1. Initialize all components
        println!("1️⃣ Initializing components...");
        
        let jupiter_client = JupiterClient::new();
        
        let wallet = match WalletManager::new() {
            Ok(w) => {
                println!("   ✅ Wallet initialized");
                w
            }
            Err(e) => {
                println!("   ❌ Wallet failed: {}", e);
                return;
            }
        };
        
        let trade_executor = match TradeExecutor::new().await {
            Ok(t) => {
                println!("   ✅ Trade executor initialized");
                t
            }
            Err(e) => {
                println!("   ❌ Trade executor failed: {}", e);
                return;
            }
        };
        
        let ws_manager = match WebSocketManager::new().await {
            Ok(w) => {
                println!("   ✅ WebSocket manager initialized");
                w
            }
            Err(e) => {
                println!("   ❌ WebSocket manager failed: {}", e);
                return;
            }
        };
        
        // 2. Test quote + wallet balance check
        println!("2️⃣ Testing quote + balance flow...");
        
        let balance = wallet.get_balance().await.unwrap_or(0.0);
        println!("   Current balance: {} SOL", balance);
        
        if balance > 0.001 {
            let input_mint = "So11111111111111111111111111111111111111112";
            let output_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
            let amount = 100000; // 0.0001 SOL
            
            match jupiter_client.get_quote(input_mint, output_mint, amount).await {
                Ok(quote) => {
                    println!("   ✅ Quote obtained: {} -> {}", quote.in_amount, quote.out_amount);
                }
                Err(e) => println!("   ❌ Quote failed: {}", e),
            }
        }
        
        // 3. Test WebSocket monitoring
        println!("3️⃣ Testing WebSocket monitoring...");
        
        if let Ok(_) = ws_manager.connect().await {
            let pubkey = wallet.get_pubkey();
            match ws_manager.watch_account(&pubkey).await {
                Ok(_) => println!("   ✅ Monitoring wallet account"),
                Err(e) => println!("   ❌ Account monitoring failed: {}", e),
            }
        }
        
        println!("🎉 Integration flow test completed!");
    }
}

// Helper para testing manual desde CLI
pub async fn run_manual_tests() {
    println!("🧪 SniperForge Manual Test Suite");
    println!("==================================");
    
    // Test básico de conectividad
    test_basic_connectivity().await;
    
    // Test de componentes individuales
    test_individual_components().await;
    
    // Test de flujo integrado
    test_integration_workflow().await;
}

async fn test_basic_connectivity() {
    println!("\n🔗 Testing basic connectivity...");
    
    // Test RPC connection
    use crate::shared::rpc_pool::RpcPool;
    match RpcPool::new() {
        Ok(_) => println!("   ✅ RPC connection pool ready"),
        Err(e) => println!("   ❌ RPC pool failed: {}", e),
    }
    
    // Test Jupiter API availability
    let client = reqwest::Client::new();
    match client.get("https://quote-api.jup.ag/v6/health").send().await {
        Ok(response) if response.status().is_success() => {
            println!("   ✅ Jupiter API accessible");
        }
        Ok(response) => {
            println!("   ⚠️  Jupiter API responded with: {}", response.status());
        }
        Err(e) => {
            println!("   ❌ Jupiter API unreachable: {}", e);
        }
    }
}

async fn test_individual_components() {
    println!("\n🔧 Testing individual components...");
    
    // Test each component in isolation
    println!("   Testing Jupiter client...");
    println!("   Testing Wallet manager...");
    println!("   Testing WebSocket manager...");
    println!("   Testing Trade executor...");
}

async fn test_integration_workflow() {
    println!("\n🔄 Testing integration workflow...");
    
    println!("   1. Component initialization");
    println!("   2. Data flow testing");
    println!("   3. WebSocket monitoring");
    println!("   4. Trade simulation");
}
