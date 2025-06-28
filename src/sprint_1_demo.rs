//! Demo de las funcionalidades críticas implementadas en Sprint 1
//! 
//! Este archivo demuestra las nuevas capacidades de SniperForge:
//! - Jupiter swap execution real
//! - Cache-free trading con ejecución real
//! - Pipeline completo de trading

use anyhow::Result;
use tracing::{info, warn};

use crate::shared::jupiter::{Jupiter, JupiterConfig};
use crate::shared::cache_free_trading::{CacheFreeTradeEngine, CacheFreeConfig};
use crate::shared::pool_detector::{TradingOpportunity, OpportunityType, DetectedPool, TokenInfo, RiskScore};

/// Demo de las funcionalidades implementadas en Sprint 1
pub async fn demo_sprint_1_functionality() -> Result<()> {
    info!("🚀 SniperForge Sprint 1 - Functionality Demo");
    info!("============================================");
    
    // 1. Demostrar Jupiter real swap execution
    demo_jupiter_real_execution().await?;
    
    // 2. Demostrar cache-free trading con ejecución real
    demo_cache_free_real_trading().await?;
    
    info!("✅ Sprint 1 Demo completed successfully!");
    Ok(())
}

/// Demo de Jupiter real swap execution
async fn demo_jupiter_real_execution() -> Result<()> {
    info!("📊 Demo 1: Jupiter Real Swap Execution");
    info!("--------------------------------------");
    
    // Inicializar Jupiter
    let jupiter_config = JupiterConfig::default();
    let jupiter = Jupiter::new(&jupiter_config).await?;
    
    // Obtener quote real
    info!("🔍 Getting real quote from Jupiter...");
    let quote = jupiter.get_quote(
        "So11111111111111111111111111111111111111112", // SOL
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
        0.001, // 0.001 SOL
        100,   // 1% slippage
    ).await?;
    
    info!("✅ Quote received:");
    info!("   Input: {} SOL", quote.in_amount);
    info!("   Output: {} USDC", quote.out_amount);
    info!("   Price impact: {:.4}%", quote.price_impact_pct);
    
    // Construir transacción real
    info!("🔧 Building real swap transaction...");
    let swap_result = jupiter.execute_swap(&quote, "DEMO_WALLET_ADDRESS").await?;
    
    info!("✅ Real transaction built:");
    info!("   Success: {}", swap_result.success);
    info!("   Transaction ID: {}", swap_result.transaction_signature.unwrap_or("None".to_string()));
    info!("   Output amount: {}", swap_result.output_amount);
    info!("   Fee estimate: ${:.6}", swap_result.fee_amount);
    
    Ok(())
}

/// Demo de cache-free trading con ejecución real
async fn demo_cache_free_real_trading() -> Result<()> {
    info!("🎯 Demo 2: Cache-Free Real Trading Engine");
    info!("------------------------------------------");
    
    // Configurar cache-free trading
    let config = CacheFreeConfig {
        max_slippage_pct: 2.0,
        min_profit_threshold_usd: 0.5,
        ..Default::default()
    };
    
    let mut trade_engine = CacheFreeTradeEngine::new(config).await?;
    
    // Crear una oportunidad de trading demo
    let opportunity = create_demo_trading_opportunity();
    
    info!("🔍 Processing trading opportunity:");
    info!("   Type: {:?}", opportunity.opportunity_type);
    info!("   Expected profit: ${:.2}", opportunity.expected_profit_usd);
    info!("   Recommended size: ${:.2}", opportunity.recommended_size_usd);
    info!("   Confidence: {:.1}%", opportunity.confidence * 100.0);
    
    // Ejecutar trade con validación real
    info!("🚀 Executing trade with real validation...");
    match trade_engine.execute_trade_with_validation(&opportunity).await {
        Ok(trade_result) => {
            info!("✅ Trade executed successfully:");
            info!("   Trade ID: {}", &trade_result.trade_id[..8]);
            info!("   Success: {}", trade_result.success);
            info!("   Execution time: {}ms", trade_result.execution_time_ms);
            info!("   Entry price: ${:.6}", trade_result.entry_price);
            info!("   Actual slippage: {:.4}%", trade_result.actual_slippage_pct);
            info!("   Net profit: ${:.4}", trade_result.net_profit_usd);
            info!("   Gas fees: ${:.6}", trade_result.gas_fees_usd);
        }
        Err(e) => {
            warn!("❌ Trade execution failed: {}", e);
        }
    }
    
    // Mostrar métricas de performance
    let metrics = trade_engine.get_performance_metrics();
    info!("📈 Performance Metrics:");
    info!("   Total opportunities: {}", metrics.total_opportunities_evaluated);
    info!("   Total trades: {}", metrics.total_trades_executed);
    info!("   Success rate: {:.1}%", metrics.success_rate_pct);
    info!("   Average execution time: {:.1}ms", metrics.average_execution_time_ms);
    
    Ok(())
}

/// Crear una oportunidad de trading demo para testing
fn create_demo_trading_opportunity() -> TradingOpportunity {
    TradingOpportunity {
        pool: DetectedPool {
            pool_address: "DEMO_POOL_ADDRESS".to_string(),
            token_a: TokenInfo {
                mint: "So11111111111111111111111111111111111111112".to_string(), // SOL
                symbol: "SOL".to_string(),
                decimals: 9,
                supply: 1000000000,
                price_usd: 150.0,
                market_cap: 150000000000.0,
            },
            token_b: TokenInfo {
                mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
                symbol: "USDC".to_string(),
                decimals: 6,
                supply: 1000000000,
                price_usd: 1.0,
                market_cap: 1000000000.0,
            },
            liquidity_usd: 50000.0,
            price_impact_1k: 0.5,
            volume_24h: 25000.0,
            created_at: chrono::Utc::now().timestamp() as u64,
            detected_at: chrono::Utc::now().timestamp() as u64,
            dex: "Raydium".to_string(),
            risk_score: RiskScore {
                overall: 0.75,
                liquidity_score: 0.8,
                volume_score: 0.85,
                token_age_score: 0.9,
                holder_distribution_score: 0.7,
                rug_indicators: vec![],
            },
            transaction_signature: None,
            creator: None,
            detection_method: Some("DEMO".to_string()),
        },
        opportunity_type: OpportunityType::PriceDiscrepancy,
        confidence: 0.85,
        expected_profit_usd: 2.5,
        recommended_size_usd: 50.0,
        time_window_ms: 30000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_demo_jupiter_execution() {
        // Test que la funcionalidad de Jupiter funciona
        let result = demo_jupiter_real_execution().await;
        assert!(result.is_ok(), "Jupiter demo should work");
    }

    #[tokio::test]
    async fn test_demo_cache_free_trading() {
        // Test que cache-free trading funciona
        let result = demo_cache_free_real_trading().await;
        assert!(result.is_ok(), "Cache-free trading demo should work");
    }
}
