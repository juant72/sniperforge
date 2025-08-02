// Performance benchmarks for SniperForge
// Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sniperforge::{
    config::SimpleConfig,
    trading::ArbitrageEngine,
    apis::price_feeds::PriceFeedManager,
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
        b.iter(|| {
            rt.block_on(async {
                black_box(engine.scan_for_opportunities().await.unwrap())
            })
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

fn benchmark_price_feed_creation(c: &mut Criterion) {
    c.bench_function("price_feed_creation", |b| {
        b.iter(|| {
            let config = create_test_config();
            black_box(PriceFeedManager::new(&config))
        })
    });
}

fn create_test_config() -> SimpleConfig {
    SimpleConfig {
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
        max_history_size: 1000,
        
        // 🚀 Enterprise benchmark configuration for maximum performance
        trading_amount: 0.01,
        profit_threshold: 0.5,
        max_price_age_seconds: 30,
        risk_percentage: 2.0,
        enable_ml_analysis: true,
        enable_sentiment_analysis: true,
        enable_technical_analysis: true,
        max_concurrent_trades: 10,          // High concurrency for benchmarks
        portfolio_rebalancing: true,
        stop_loss_percentage: 5.0,
        take_profit_percentage: 10.0,
    }
}

criterion_group!(
    benches,
    benchmark_opportunity_scanning,
    benchmark_configuration_loading,
    benchmark_price_feed_creation
);
criterion_main!(benches);
