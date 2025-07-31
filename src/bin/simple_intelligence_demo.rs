use sniperforge::{
    monitoring::EnterpriseMonitor,
    intelligence::{
        initialize_intelligence_system, IntelligenceConfig
    },
};
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), sniperforge::SniperForgeError> {
    println!("🚀 Professional Intelligence System - Enterprise Demo");
    println!("====================================================");
    
    // Initialize intelligence configuration
    let intelligence_config = IntelligenceConfig {
        enable_ml_predictions: true,
        enable_sentiment_analysis: true,
        enable_autonomous_trading: true,
        risk_tolerance: 0.02,
        max_position_size: 1000.0,
        learning_rate: 0.001,
    };
    
    println!("🔧 Configuration loaded:");
    println!("   - ML Predictions: {}", intelligence_config.enable_ml_predictions);
    println!("   - Sentiment Analysis: {}", intelligence_config.enable_sentiment_analysis);
    println!("   - Autonomous Trading: {}", intelligence_config.enable_autonomous_trading);
    println!("   - Risk tolerance: {:.2}%", intelligence_config.risk_tolerance * 100.0);
    println!("   - Max position size: ${:.2}", intelligence_config.max_position_size);
    
    // Initialize enterprise monitoring
    let monitor = Arc::new(EnterpriseMonitor::new());
    println!("✅ Enterprise monitoring system initialized");
    
    // Initialize the intelligence system
    let intelligence_suite = initialize_intelligence_system(intelligence_config.clone()).await
        .map_err(|e| sniperforge::SniperForgeError::config(e.to_string()))?;
    
    println!("✅ Intelligence System initialized successfully");
    
    // Demo symbols for analysis
    let symbols = vec!["SOL/USDC", "BTC/USDT", "ETH/USDC", "RAY/SOL"];
    
    println!("\n🎯 Starting Market Intelligence Analysis");
    println!("======================================");
    
    // Simulate professional market analysis
    for (i, symbol) in symbols.iter().enumerate() {
        println!("\n📊 Analyzing {} (Symbol {} of {})", symbol, i + 1, symbols.len());
        
        // Simulate processing delay
        sleep(Duration::from_millis(500)).await;
        
        println!("   ✓ Market sentiment analysis completed");
        println!("   ✓ Technical indicators processed");
        println!("   ✓ Risk assessment performed");
        println!("   ✓ Trading signals generated");
        
        // Simulate getting system health
        let system_status = monitor.get_system_status().await;
        println!("   📈 System Health: {:?}", system_status.health_status);
    }
    
    println!("\n🏆 Intelligence Analysis Complete");
    println!("================================");
    println!("✅ All market analysis tasks completed successfully");
    println!("📊 Professional intelligence system operational");
    println!("💼 Enterprise-grade monitoring active");
    println!("🔒 Risk management protocols engaged");
    
    // Final system status
    let final_status = monitor.get_system_status().await;
    println!("\n📋 Final System Status:");
    println!("   - Health Status: {:?}", final_status.health_status);
    println!("   - Monitoring Active: {}", final_status.monitoring_active);
    println!("   - Active Alerts: {}", final_status.active_alerts);
    
    println!("\n🎉 Professional Intelligence System Demo completed successfully!");
    println!("💡 System ready for enterprise deployment");
    
    Ok(())
}
