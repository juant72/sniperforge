// Performance benchmarks for SniperForge
// Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sniperforge_core::{
    config::Config,
    trading::arbitrage::ArbitrageEngine,
    apis::price_feeds::PriceFeedManager,
    types::ArbitrageOpportunity,
};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn benchmark_opportunity_scanning(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (engine, _config) = rt.block_on(async {
        let config = create_test_config();
        let price_feed_manager = Arc::new(PriceFeedManager::new(&config));
        let engine = ArbitrageEngine::new(config.clone(), price_feed_manager)
            .await
            .expect("Failed to create engine");
        (engine, config)
    });
    
    c.bench_function("opportunity_scanning", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(engine.scan_for_opportunities().await.unwrap())
        })
    });
}

fn benchmark_risk_assessment(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let (engine, opportunity) = rt.block_on(async {
        let config = create_test_config();
        let price_feed_manager = Arc::new(PriceFeedManager::new(&config));
        let engine = ArbitrageEngine::new(config, price_feed_manager)
            .await
            .expect("Failed to create engine");
            
        let opportunity = ArbitrageOpportunity {
            profit_percentage: 0.02,
            volume_required: 100.0,
            estimated_gas_cost: 0.001,
            confidence_score: 0.8,
            ..Default::default()
        };
        
        (engine, opportunity)
    });
    
    c.bench_function("risk_assessment", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(engine.assess_risk(&opportunity).await.unwrap())
        })
    });
}

fn benchmark_price_feed_updates(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let price_feed_manager = rt.block_on(async {
        let config = create_test_config();
        PriceFeedManager::new(&config)
    });
    
    c.bench_function("price_feed_updates", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(price_feed_manager.update_prices().await.unwrap())
        })
    });
}

fn benchmark_configuration_loading(c: &mut Criterion) {
    c.bench_function("config_loading", |b| {
        b.iter(|| {
            black_box(create_test_config())
        })
    });
}

// Helper function
fn create_test_config() -> Config {
    Config {
        solana_rpc_url: "https://api.devnet.solana.com".to_string(),
        solana_ws_url: "wss://api.devnet.solana.com/".to_string(),
        max_slippage: 0.005,
        min_profit_threshold: 0.001,
        max_position_size: 0.1,
        private_key_path: "./test_wallet.json".to_string(),
        enable_simulation: true,
        log_level: "info".to_string(),
        dexscreener_base_url: "https://api.dexscreener.com".to_string(),
        max_requests_per_second: 10,
        cooldown_period_ms: 100,
    }
}

criterion_group!(
    benches,
    benchmark_opportunity_scanning,
    benchmark_risk_assessment,
    benchmark_price_feed_updates,
    benchmark_configuration_loading
);
criterion_main!(benches);
