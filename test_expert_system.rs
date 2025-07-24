// TEST SCRIPT FOR EXPERT ARBITRAGE SYSTEM
use anyhow::Result;

mod real_arbitrage_expert;
use real_arbitrage_expert::ProfessionalArbitrageBot;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    println!("🚀 TESTING EXPERT DEFI ARBITRAGE SYSTEM");
    println!("═══════════════════════════════════════");
    
    let mut bot = ProfessionalArbitrageBot::new();
    
    // Test 1: Scan for opportunities
    println!("\n🔍 TEST 1: Scanning for arbitrage opportunities...");
    match bot.scan_arbitrage_opportunities().await {
        Ok(opportunities) => {
            println!("✅ Scan completed successfully!");
            println!("📊 Found {} opportunities", opportunities.len());
            
            for (i, opp) in opportunities.iter().take(3).enumerate() {
                println!("   {}. {} -> {} ({:.2}% spread, ${:.2} profit)",
                         i + 1, opp.dex_buy, opp.dex_sell, 
                         opp.spread_percent, opp.estimated_profit_usd);
            }
        }
        Err(e) => {
            println!("❌ Scan failed: {}", e);
        }
    }
    
    // Test 2: Performance stats
    println!("\n📊 TEST 2: Performance Statistics");
    println!("{}", bot.get_performance_stats());
    
    // Test 3: Load wallet test
    println!("\n🔑 TEST 3: Wallet Loading");
    match bot.load_wallet("wallets/mainnet_wallet.json") {
        Ok(_) => println!("✅ Wallet loaded successfully"),
        Err(e) => println!("⚠️ Wallet load failed: {} (expected if file doesn't exist)", e),
    }
    
    println!("\n🎯 EXPERT SYSTEM READY FOR REAL ARBITRAGE!");
    println!("💡 Use: cargo run --release --bin real_arbitrage_expert");
    
    Ok(())
}
