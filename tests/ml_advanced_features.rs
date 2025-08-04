//! FASE 6: ADVANCED ML FEATURES TESTS
//! Tests para validar la integración de características ML avanzadas

use sniperforge::{
    SimpleConfig,
    types::{TradingOpportunity, MarketData, OpportunityType},
    trading::strategies::StrategyManager,
    ml::{MLEngineFactory, MLConfig},
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use chrono::Utc;
use anyhow::Result;

/// Test de inicialización del ML Engine
#[tokio::test]
async fn test_ml_engine_initialization() -> Result<()> {
    println!("🤖 ML Engine Test: Initialization and Configuration");
    
    // Test 1: Default ML Engine
    let _default_engine = MLEngineFactory::create_default();
    println!("✅ Default ML Engine created successfully");
    
    // Test 2: Production ML Engine
    let _production_engine = MLEngineFactory::create_production();
    println!("✅ Production ML Engine created successfully");
    
    // Test 3: HFT ML Engine
    let _hft_engine = MLEngineFactory::create_hft();
    println!("✅ HFT ML Engine created successfully");
    
    // Test 4: Custom Configuration
    let custom_config = MLConfig {
        sentiment_threshold: 0.8,
        prediction_horizon: 20,
        risk_tolerance: 0.2,
        portfolio_rebalance_frequency: 120,
        pattern_confidence_threshold: 0.9,
        model_update_interval: 300,
        enable_real_time_learning: true,
    };
    let _custom_engine = MLEngineFactory::create_with_config(custom_config);
    println!("✅ Custom ML Engine created successfully");
    
    println!("✅ ML Engine initialization test: PASSED");
    Ok(())
}

/// Test de análisis ML completo
#[tokio::test]
async fn test_ml_comprehensive_analysis() -> Result<()> {
    println!("🧠 ML Analysis Test: Comprehensive Opportunity Analysis");
    
    let ml_engine = MLEngineFactory::create_default();
    
    let opportunity = create_test_opportunity();
    let market_data = create_test_market_data();
    
    // Realizar análisis ML completo
    let start_time = Instant::now();
    let ml_result = ml_engine.analyze_opportunity(&opportunity, &market_data).await?;
    let analysis_time = start_time.elapsed();
    
    println!("📊 ML Analysis Results:");
    println!("   ML Score: {:.3}", ml_result.ml_score);
    println!("   Overall Confidence: {:.3}", ml_result.confidence);
    println!("   Analysis Time: {:.2}ms", analysis_time.as_millis());
    
    // Validar componentes del análisis
    println!("🔍 Sentiment Analysis:");
    println!("   Token: {}", ml_result.sentiment_analysis.token);
    println!("   Score: {:.3}", ml_result.sentiment_analysis.score);
    println!("   Trend: {:?}", ml_result.sentiment_analysis.trend);
    println!("   Confidence: {:.3}", ml_result.sentiment_analysis.confidence);
    
    println!("📈 Market Prediction:");
    println!("   Token: {}", ml_result.market_prediction.token);
    println!("   Predicted Price: ${:.2}", ml_result.market_prediction.predicted_price);
    println!("   Predicted Volatility: {:.3}", ml_result.market_prediction.predicted_volatility);
    println!("   Trend: {:?}", ml_result.market_prediction.trend);
    println!("   Confidence: {:.3}", ml_result.market_prediction.confidence);
    
    println!("🛡️ Risk Assessment:");
    println!("   Token: {}", ml_result.risk_assessment.token);
    println!("   Overall Risk: {:.3}", ml_result.risk_assessment.overall_risk_score);
    println!("   Risk Category: {:?}", ml_result.risk_assessment.risk_category);
    println!("   Market Risk: {:.3}", ml_result.risk_assessment.market_risk);
    println!("   Liquidity Risk: {:.3}", ml_result.risk_assessment.liquidity_risk);
    
    println!("🎯 Pattern Recognition:");
    println!("   Patterns Found: {}", ml_result.pattern_matches.len());
    for pattern in &ml_result.pattern_matches {
        println!("     - {}: {:.3} confidence", pattern.pattern_name, pattern.confidence);
    }
    
    // Validaciones
    assert!(ml_result.ml_score >= 0.0 && ml_result.ml_score <= 1.0, "ML score should be between 0 and 1");
    assert!(ml_result.confidence >= 0.0 && ml_result.confidence <= 1.0, "Confidence should be between 0 and 1");
    assert!(analysis_time.as_millis() < 5000, "Analysis should complete in < 5 seconds");
    
    println!("✅ ML comprehensive analysis test: PASSED");
    Ok(())
}

/// Test de integración ML con estrategia de arbitraje
#[tokio::test]
async fn test_ml_arbitrage_integration() -> Result<()> {
    println!("🔗 ML Integration Test: Arbitrage Strategy Enhancement");
    
    let config = SimpleConfig::default();
    let mut strategy_manager = StrategyManager::new(config);
    strategy_manager.initialize_strategies().await?;
    
    // Crear múltiples oportunidades para test
    let opportunities = vec![
        create_test_opportunity_with_params("SOL/USDC", 2.5, 0.85),
        create_test_opportunity_with_params("BTC/USDC", 1.8, 0.90),
        create_test_opportunity_with_params("ETH/USDC", 3.2, 0.75),
    ];
    
    let market_data = create_test_market_data();
    
    // Analizar cada oportunidad con ML enhancement
    for (i, opportunity) in opportunities.iter().enumerate() {
        println!("🔍 Analyzing opportunity {}: {}", i + 1, opportunity.token_pair);
        
        let analysis_start = Instant::now();
        let signals = strategy_manager.analyze_opportunity(opportunity, &market_data).await?;
        let analysis_time = analysis_start.elapsed();
        
        println!("   Signals generated: {}", signals.len());
        println!("   Analysis time: {:.2}ms", analysis_time.as_millis());
        
        if !signals.is_empty() {
            let signal = &signals[0];
            println!("   Signal type: {:?}", signal.signal_type);
            println!("   Expected profit: {:.2}%", signal.expected_profit);
            println!("   Risk score: {:.3}", signal.risk_score);
            if let Some(ref reasoning) = signal.reasoning {
                println!("   Reasoning: {}", reasoning);
            }
        }
        
        // Validar que el análisis no tarde demasiado
        assert!(analysis_time.as_millis() < 2000, "ML-enhanced analysis should complete in < 2 seconds");
    }
    
    println!("✅ ML arbitrage integration test: PASSED");
    Ok(())
}

/// Test de performance ML bajo carga
#[tokio::test]
async fn test_ml_performance_under_load() -> Result<()> {
    println!("⚡ ML Performance Test: Load Testing");
    
    let _ml_engine = MLEngineFactory::create_production();
    let concurrent_analyses = 10;
    let analyses_per_task = 5;
    
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    // Ejecutar análisis concurrentes
    for task_id in 0..concurrent_analyses {
        let engine_clone = MLEngineFactory::create_production(); // Create separate engine for each task
        
        let handle = tokio::spawn(async move {
            let mut task_results = Vec::new();
            
            for i in 0..analyses_per_task {
                let opportunity = create_test_opportunity_with_params(
                    &format!("TASK_{}_{}/USDC", task_id, i),
                    1.0 + (i as f64 * 0.5),
                    0.7 + (i as f64 * 0.05),
                );
                let market_data = create_test_market_data();
                
                let analysis_start = Instant::now();
                match engine_clone.analyze_opportunity(&opportunity, &market_data).await {
                    Ok(result) => {
                        let analysis_time = analysis_start.elapsed();
                        task_results.push((task_id, i, result.ml_score, analysis_time));
                    },
                    Err(e) => {
                        eprintln!("Analysis failed for task {} analysis {}: {}", task_id, i, e);
                    }
                }
            }
            
            task_results
        });
        
        handles.push(handle);
    }
    
    // Recopilar resultados
    let mut total_analyses = 0;
    let mut total_time_ms = 0u128;
    let mut ml_scores = Vec::new();
    
    for handle in handles {
        if let Ok(task_results) = handle.await {
            for (task_id, analysis_id, ml_score, analysis_time) in task_results {
                total_analyses += 1;
                total_time_ms += analysis_time.as_millis();
                ml_scores.push(ml_score);
                
                println!("   Task {} Analysis {}: ML Score {:.3}, Time {}ms", 
                         task_id, analysis_id, ml_score, analysis_time.as_millis());
            }
        }
    }
    
    let total_elapsed = start_time.elapsed();
    let avg_analysis_time = if total_analyses > 0 { total_time_ms / total_analyses as u128 } else { 0 };
    let throughput = total_analyses as f64 / total_elapsed.as_secs_f64();
    let avg_ml_score = ml_scores.iter().sum::<f64>() / ml_scores.len() as f64;
    
    println!("📊 ML Performance Results:");
    println!("   Total Analyses: {}", total_analyses);
    println!("   Total Time: {:.2}s", total_elapsed.as_secs_f64());
    println!("   Average Analysis Time: {}ms", avg_analysis_time);
    println!("   Throughput: {:.1} analyses/second", throughput);
    println!("   Average ML Score: {:.3}", avg_ml_score);
    
    // Validaciones de performance
    assert!(avg_analysis_time < 1000, "Average ML analysis time should be < 1 second");
    assert!(throughput > 5.0, "ML throughput should be > 5 analyses/second");
    assert!(avg_ml_score > 0.3, "Average ML score should be > 0.3");
    
    println!("✅ ML performance under load test: PASSED");
    Ok(())
}

/// Test de ML utils y funciones auxiliares
#[tokio::test]
async fn test_ml_utilities() -> Result<()> {
    println!("🔧 ML Utils Test: Helper Functions Validation");
    
    // Test Sharpe ratio calculation
    let returns = vec![0.1, 0.15, 0.08, 0.12, 0.05, -0.02, 0.18, 0.09];
    let risk_free_rate = 0.03;
    let sharpe = sniperforge::ml::utils::calculate_sharpe_ratio(&returns, risk_free_rate);
    println!("📈 Sharpe Ratio: {:.3}", sharpe);
    assert!(sharpe > 0.0, "Sharpe ratio should be positive for profitable returns");
    
    // Test max drawdown calculation
    let equity_curve = vec![100.0, 110.0, 105.0, 120.0, 90.0, 95.0, 130.0, 125.0];
    let max_drawdown = sniperforge::ml::utils::calculate_max_drawdown(&equity_curve);
    println!("📉 Max Drawdown: {:.3}", max_drawdown);
    assert!(max_drawdown > 0.0 && max_drawdown < 1.0, "Max drawdown should be between 0 and 1");
    
    // Test normalization
    let values = vec![10.0, 25.0, 5.0, 40.0, 15.0];
    let normalized = sniperforge::ml::utils::normalize(&values);
    println!("🔢 Normalized values: {:?}", normalized);
    assert!(normalized.iter().all(|&v| v >= 0.0 && v <= 1.0), "All normalized values should be between 0 and 1");
    assert!((normalized[0] - 0.142857).abs() < 0.001, "First normalized value should be ~0.143");
    assert!((normalized[3] - 1.0).abs() < 0.001, "Max value should normalize to 1.0");
    
    // Test correlation calculation
    let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
    let correlation = sniperforge::ml::utils::correlation(&x, &y);
    println!("🔗 Correlation: {:.3}", correlation);
    assert!((correlation - 1.0).abs() < 0.001, "Perfect positive correlation should be ~1.0");
    
    println!("✅ ML utilities test: PASSED");
    Ok(())
}

/// Test de ML performance monitoring
#[tokio::test]
async fn test_ml_performance_monitoring() -> Result<()> {
    println!("📊 ML Monitoring Test: Performance Tracking");
    
    let mut monitor = sniperforge::ml::monitoring::MLPerformanceMonitor::new(100);
    
    // Simular predicciones y resultados reales
    let predictions = vec![
        (10.0, 0.8, 9.5),   // Close prediction
        (15.0, 0.9, 16.0),  // Close prediction  
        (12.0, 0.7, 20.0),  // Poor prediction
        (8.0, 0.85, 7.8),   // Very close prediction
        (25.0, 0.6, 22.0),  // Moderate prediction
    ];
    
    // Agregar predicciones
    for (i, (predicted, confidence, actual)) in predictions.iter().enumerate() {
        monitor.add_prediction(*predicted, *confidence);
        monitor.update_actual(i, *actual);
        println!("   Prediction {}: {:.1} -> {:.1} (confidence: {:.2})", 
                 i + 1, predicted, actual, confidence);
    }
    
    // Calcular métricas
    let accuracy = monitor.calculate_accuracy(2.0); // 2.0 tolerance
    let mae = monitor.calculate_mae();
    let rmse = monitor.calculate_rmse();
    
    println!("📈 Performance Metrics:");
    println!("   Accuracy (±2.0): {:.3}", accuracy);
    println!("   Mean Absolute Error: {:.3}", mae);
    println!("   Root Mean Square Error: {:.3}", rmse);
    
    // Validaciones
    assert!(accuracy >= 0.0 && accuracy <= 1.0, "Accuracy should be between 0 and 1");
    assert!(mae >= 0.0, "MAE should be non-negative");
    assert!(rmse >= 0.0, "RMSE should be non-negative");
    assert!(rmse >= mae, "RMSE should be >= MAE");
    
    println!("✅ ML performance monitoring test: PASSED");
    Ok(())
}

// Helper functions

fn create_test_opportunity() -> TradingOpportunity {
    create_test_opportunity_with_params("SOL/USDC", 2.5, 0.85)
}

fn create_test_opportunity_with_params(token_pair: &str, profit_percentage: f64, confidence: f64) -> TradingOpportunity {
    TradingOpportunity {
        opportunity_type: OpportunityType::Arbitrage,
        token_pair: token_pair.to_string(),
        profit_percentage,
        volume_24h: 1_000_000.0,
        liquidity: 100_000.0,
        source_exchange: "Jupiter".to_string(),
        target_exchange: "Raydium".to_string(),
        entry_price: 150.0,
        exit_price: 150.0 * (1.0 + profit_percentage / 100.0),
        risk_score: 1.0 - confidence,
        confidence,
        timestamp: Utc::now(),
        execution_window: Duration::from_secs(30),
        metadata: HashMap::new(),
    }
}

fn create_test_market_data() -> MarketData {
    let mut prices = HashMap::new();
    prices.insert("SOL".to_string(), 150.0);
    prices.insert("USDC".to_string(), 1.0);
    prices.insert("BTC".to_string(), 45000.0);
    prices.insert("ETH".to_string(), 3000.0);
    
    let mut volumes = HashMap::new();
    volumes.insert("SOL".to_string(), 1_000_000.0);
    volumes.insert("USDC".to_string(), 500_000.0);
    volumes.insert("BTC".to_string(), 2_000_000.0);
    volumes.insert("ETH".to_string(), 1_500_000.0);
    
    let mut liquidity = HashMap::new();
    liquidity.insert("SOL".to_string(), 100_000.0);
    liquidity.insert("USDC".to_string(), 500_000.0);
    liquidity.insert("BTC".to_string(), 200_000.0);
    liquidity.insert("ETH".to_string(), 150_000.0);
    
    MarketData {
        prices,
        volumes,
        liquidity,
        current_price: 150.0,
        volume_24h: 1_000_000.0,
        last_updated: Some(Instant::now()),
        bid_ask_spread: 0.001,
    }
}
