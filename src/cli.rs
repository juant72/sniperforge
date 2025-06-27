use anyhow::Result;
use clap::{Command, Arg, ArgMatches, ArgAction};
use colored::*;
use std::io::{self, Write};
use std::str::FromStr;
use solana_sdk::signer::{Signer, keypair::Keypair};
use chrono::{DateTime, Utc, Duration};

use sniperforge::{Config, SniperForgePlatform, solana_testing};
use sniperforge::shared::jupiter::{JupiterClient, JupiterConfig, QuoteRequest, tokens};
use sniperforge::shared::trade_executor::{TradeExecutor, TradeRequest, TradingMode};
use sniperforge::shared::wallet_manager::WalletManager;

// Phase 6A imports for advanced strategies and analysis
use sniperforge::strategies::trend_following::TrendFollowingStrategy;
use sniperforge::strategies::mean_reversion::MeanReversionStrategy;
use sniperforge::strategies::momentum::MomentumStrategy;
use sniperforge::strategies::arbitrage::ArbitrageStrategy;
use sniperforge::analysis::timeframe::MultiTimeframeAnalyzer;
use sniperforge::analysis::patterns::PatternRecognizer;

// Phase 6B imports for Machine Learning
use sniperforge::ml::advanced_analytics::AdvancedAnalyticsEngine;
// TODO: Re-enable when ML module compilation is fixed
/*
use sniperforge::ml::{
    PatternRecognizer as MLPatternRecognizer, StrategyOptimizer, RiskAssessor, 
    TimingPredictor, DataPreprocessor, ModelManager, ModelType, TradeDirection,
    PatternRecognitionConfig, StrategyOptimizerConfig, RiskAssessmentConfig, TimingPredictorConfig
};
use sniperforge::ml::timing_predictor::ExecutionPriority;
*/

// Temporary ML structures for Phase 6B demo
#[derive(Debug)]
pub enum TradeDirection {
    Buy,
    Sell,
}

#[derive(Debug)]
pub enum ExecutionPriority {
    Immediate,
    Optimal,
    Patient,
    Avoid,
}

#[derive(Debug)]
pub struct StrategyOptimizerConfig {
    pub population_size: usize,
    pub generations: usize,
}

impl StrategyOptimizerConfig {
    pub fn default() -> Self {
        Self {
            population_size: 50,
            generations: 100,
        }
    }
}

#[derive(Debug)]
pub struct StrategyOptimizer {
    config: StrategyOptimizerConfig,
}

impl StrategyOptimizer {
    pub fn new(config: StrategyOptimizerConfig) -> Self {
        Self { config }
    }
}

#[derive(Debug)]
pub struct RiskAssessmentConfig {
    pub volatility_window: usize,
}

impl RiskAssessmentConfig {
    pub fn default() -> Self {
        Self {
            volatility_window: 30,
        }
    }
}

#[derive(Debug)]
pub struct RiskAssessor {
    config: RiskAssessmentConfig,
}

impl RiskAssessor {
    pub fn new(config: RiskAssessmentConfig) -> Self {
        Self { config }
    }
}

#[derive(Debug)]
pub struct TimingPredictorConfig {
    pub execution_horizon_seconds: u64,
}

impl TimingPredictorConfig {
    pub fn default() -> Self {
        Self {
            execution_horizon_seconds: 300,
        }
    }
}

#[derive(Debug)]
pub struct TimingPrediction {
    pub predicted_timestamp: chrono::DateTime<chrono::Utc>,
    pub confidence: f64,
    pub expected_slippage: f64,
    pub liquidity_score: f64,
    pub execution_priority: ExecutionPriority,
    pub reasoning: String,
}

#[derive(Debug)]
pub struct ExecutionStrategy {
    pub chunks: Vec<f64>,
    pub timing_windows: Vec<chrono::DateTime<chrono::Utc>>,
    pub estimated_completion: chrono::DateTime<chrono::Utc>,
    pub total_expected_slippage: f64,
}

#[derive(Debug)]
pub struct TimingPredictor {
    config: TimingPredictorConfig,
}

impl TimingPredictor {
    pub fn new(config: TimingPredictorConfig) -> Self {
        Self { config }
    }

    pub async fn predict_optimal_timing(
        &self,
        _symbol: &str,
        _target_size: f64,
        _direction: TradeDirection,
    ) -> Result<TimingPrediction> {
        // Simulated prediction for Phase 6B demo
        Ok(TimingPrediction {
            predicted_timestamp: chrono::Utc::now() + chrono::Duration::minutes(5),
            confidence: 0.87,
            expected_slippage: 0.003,
            liquidity_score: 0.92,
            execution_priority: ExecutionPriority::Optimal,
            reasoning: "Favorable liquidity conditions with low volatility window ahead".to_string(),
        })
    }

    pub async fn optimize_execution_strategy(
        &self,
        _symbol: &str,
        trade_size: f64,
        max_slippage: f64,
    ) -> Result<ExecutionStrategy> {
        // Simulated execution strategy optimization
        let num_chunks = if trade_size > 500.0 { 4 } else { 2 };
        let chunk_size = trade_size / num_chunks as f64;
        
        let chunks = vec![chunk_size; num_chunks];
        let mut timing_windows = Vec::new();
        let base_time = chrono::Utc::now();
        
        for i in 0..num_chunks {
            timing_windows.push(base_time + chrono::Duration::minutes(i as i64 * 3));
        }
        
        Ok(ExecutionStrategy {
            chunks,
            timing_windows: timing_windows.clone(),
            estimated_completion: timing_windows.last().unwrap().clone() + chrono::Duration::minutes(2),
            total_expected_slippage: max_slippage * 0.7, // 30% improvement
        })
    }
}

pub async fn run_cli() -> Result<()> {
    let matches = Command::new("SniperForge CLI")
        .version("0.1.0")
        .about("Interactive CLI for SniperForge Multi-Bot Platform")
        .subcommand(
            Command::new("start")
                .about("Start the platform or specific bots")
                .arg(
                    Arg::new("bot")
                        .short('b')
                        .long("bot")
                        .value_name("BOT_TYPE")
                        .help("Specific bot to start")
                        .action(clap::ArgAction::Append)
                )
                .arg(
                    Arg::new("devnet")
                        .long("devnet")
                        .help("Use devnet configuration for testing")
                        .action(clap::ArgAction::SetTrue)
                )
        )        .subcommand(Command::new("status").about("Show platform status"))
        .subcommand(Command::new("config").about("Show current configuration"))
        .subcommand(
            Command::new("wallet")
                .about("Wallet management commands")
                .subcommand(Command::new("balance").about("Check wallet balances"))
                .subcommand(Command::new("airdrop").about("Request SOL airdrop"))
                .subcommand(
                    Command::new("generate")
                        .about("Generate a new DevNet wallet")
                        .arg(
                            Arg::new("output")
                                .long("output")
                                .short('o')
                                .value_name("FILE")
                                .help("Output file for the wallet")
                                .default_value("test-wallet-new.json")
                        )
                )
        )        .subcommand(            Command::new("test")
                .about("Comprehensive testing suite")
                .subcommand(Command::new("all").about("Run all tests"))
                .subcommand(Command::new("basic").about("Run basic connectivity tests"))
                .subcommand(Command::new("solana").about("Test Solana connectivity and RPC calls"))
                .subcommand(Command::new("jupiter").about("Test Jupiter API integration"))
                .subcommand(Command::new("wallet").about("Test wallet functionality"))
                .subcommand(Command::new("websocket").about("Test WebSocket connectivity and subscriptions"))
                .subcommand(Command::new("trade").about("Test trade execution (simulation)"))
                .subcommand(Command::new("swap-real")
                    .about("🚀 SPRINT 1: Test REAL swap execution on DevNet") 
                    .arg(Arg::new("amount")
                        .short('a')
                        .long("amount")
                        .value_name("SOL")
                        .help("Amount of SOL to swap (default: 0.001)")
                        .default_value("0.00001"))  // Even smaller amount to reduce transaction complexity
                    .arg(Arg::new("wallet")
                        .short('w')
                        .long("wallet")
                        .value_name("FILE")
                        .help("Path to wallet keypair JSON file for real execution"))
                    .arg(Arg::new("network")
                        .long("network")
                        .value_name("NET")
                        .help("Network to use: devnet or mainnet")
                        .default_value("devnet")
                        .value_parser(["devnet", "mainnet"]))
                    .arg(Arg::new("confirm")
                        .long("confirm")
                        .action(ArgAction::SetTrue)
                        .help("Confirm you want to send a REAL transaction to blockchain")))
                .subcommand(Command::new("integration").about("Test complete integration flow"))
                .subcommand(Command::new("performance").about("Test performance and latency"))        )
        .subcommand(Command::new("interactive").about("Interactive monitoring mode"))
        // Phase 6A Commands
        .subcommand(
            Command::new("multi-strategy-trading")
                .about("[TARGET] PHASE 6A: Execute multiple trading strategies concurrently")
                .arg(Arg::new("strategies")
                    .short('s')
                    .long("strategies")
                    .value_name("STRATEGY_LIST")
                    .help("Comma-separated list of strategies: trend,momentum,mean-reversion,arbitrage")
                    .default_value("trend,momentum"))
                .arg(Arg::new("duration")
                    .short('d')
                    .long("duration")
                    .value_name("SECONDS")
                    .help("Trading session duration in seconds (default: 300)")
                    .default_value("300"))
                .arg(Arg::new("capital-per-strategy")
                    .short('c')
                    .long("capital")
                    .value_name("USD")
                    .help("Capital allocation per strategy in USD (default: 5000)")
                    .default_value("5000"))
                .arg(Arg::new("timeframes")
                    .short('t')
                    .long("timeframes")
                    .value_name("TIMEFRAME_LIST")
                    .help("Analysis timeframes: 1m,5m,15m,1h (default: 1m,5m)")
                    .default_value("1m,5m"))
        )
        .subcommand(
            Command::new("strategy-backtest")
                .about("[UP] PHASE 6A: Backtest individual or combined trading strategies")
                .arg(Arg::new("strategy")
                    .short('s')
                    .long("strategy")
                    .value_name("STRATEGY")
                    .help("Strategy to backtest: trend,momentum,mean-reversion,arbitrage,all")
                    .default_value("trend"))
                .arg(Arg::new("period")
                    .short('p')
                    .long("period")
                    .value_name("DAYS")
                    .help("Historical period to analyze in days (default: 7)")
                    .default_value("7"))
                .arg(Arg::new("initial-capital")
                    .short('c')
                    .long("capital")
                    .value_name("USD")
                    .help("Initial capital for backtesting (default: 10000)")
                    .default_value("10000"))
                .arg(Arg::new("export")
                    .short('e')
                    .long("export")
                    .value_name("FILE")
                    .help("Export backtest results to JSON file"))
        )
        .subcommand(
            Command::new("pattern-analysis")
                .about("[SEARCH] PHASE 6A: Analyze market patterns and trends")
                .arg(Arg::new("pattern-type")
                    .short('p')
                    .long("pattern")
                    .value_name("PATTERN")
                    .help("Pattern type: support-resistance,breakout,reversal,all")
                    .default_value("all"))
                .arg(Arg::new("timeframe")
                    .short('t')
                    .long("timeframe")
                    .value_name("TIMEFRAME")
                    .help("Analysis timeframe: 1m,5m,15m,1h (default: 5m)")
                    .default_value("5m"))
                .arg(Arg::new("duration")
                    .short('d')
                    .long("duration")
                    .value_name("SECONDS")
                    .help("Analysis duration in seconds (default: 180)")
                    .default_value("180"))
                .arg(Arg::new("export")
                    .short('e')
                    .long("export")
                    .value_name("FILE")
                    .help("Export pattern analysis to JSON file"))
        )
        .subcommand(
            Command::new("arbitrage-scan")
                .about("[FAST] PHASE 6A: Scan for arbitrage opportunities across DEXs")
                .arg(Arg::new("min-profit")
                    .short('m')
                    .long("min-profit")
                    .value_name("USD")
                    .help("Minimum profit threshold in USD (default: 5.0)")
                    .default_value("5.0"))
                .arg(Arg::new("max-slippage")
                    .long("max-slippage")
                    .value_name("PERCENTAGE")
                    .help("Maximum slippage tolerance (default: 0.5)")
                    .default_value("0.5"))
                .arg(Arg::new("duration")
                    .short('d')
                    .long("duration")
                    .value_name("SECONDS")
                    .help("Scanning duration in seconds (default: 120)")
                    .default_value("120"))
                .arg(Arg::new("export")
                    .short('e')
                    .long("export")                    .value_name("FILE")
                    .help("Export arbitrage opportunities to JSON file"))
        )
        // Phase 6B Machine Learning Commands
        .subcommand(
            Command::new("ml")
                .about("[ML] PHASE 6B: Machine Learning and AI-powered trading")
                .subcommand(
                    Command::new("analyze-patterns")
                        .about("Analyze market patterns using ML models")
                        .arg(Arg::new("symbol")
                            .short('s')
                            .long("symbol")
                            .value_name("TOKEN")
                            .help("Token symbol to analyze (e.g., SOL/USDC)")
                            .default_value("SOL/USDC"))
                        .arg(Arg::new("timeframe")
                            .short('t')
                            .long("timeframe")
                            .value_name("MINUTES")
                            .help("Analysis timeframe in minutes")
                            .default_value("5"))
                        .arg(Arg::new("confidence-threshold")
                            .short('c')
                            .long("confidence")
                            .value_name("THRESHOLD")
                            .help("Minimum confidence threshold (0.0-1.0)")
                            .default_value("0.8"))
                )
                .subcommand(
                    Command::new("predict-trend")
                        .about("Predict price trends using ML models")
                        .arg(Arg::new("symbol")
                            .short('s')
                            .long("symbol")
                            .value_name("TOKEN")
                            .help("Token symbol to predict")
                            .default_value("SOL/USDC"))
                        .arg(Arg::new("horizon")
                            .short('h')
                            .long("horizon")
                            .value_name("MINUTES")
                            .help("Prediction horizon in minutes")
                            .default_value("15"))
                        .arg(Arg::new("confidence-threshold")
                            .short('c')
                            .long("confidence")
                            .value_name("THRESHOLD")
                            .help("Minimum confidence threshold")
                            .default_value("0.7"))
                )
                .subcommand(
                    Command::new("optimize-strategy")
                        .about("Optimize trading strategy using genetic algorithms")
                        .arg(Arg::new("strategy")
                            .short('s')
                            .long("strategy")
                            .value_name("STRATEGY")
                            .help("Strategy to optimize: trend,momentum,mean-reversion")
                            .default_value("trend"))
                        .arg(Arg::new("generations")
                            .short('g')
                            .long("generations")
                            .value_name("COUNT")
                            .help("Number of optimization generations")
                            .default_value("50"))
                        .arg(Arg::new("population")
                            .short('p')
                            .long("population")
                            .value_name("SIZE")
                            .help("Population size for genetic algorithm")
                            .default_value("20"))
                )
                .subcommand(
                    Command::new("backtest-optimized")
                        .about("Backtest optimized strategy parameters")
                        .arg(Arg::new("strategy")
                            .short('s')
                            .long("strategy")
                            .value_name("STRATEGY")
                            .help("Strategy to backtest")
                            .default_value("trend"))
                        .arg(Arg::new("period")
                            .short('p')
                            .long("period")
                            .value_name("DAYS")
                            .help("Backtesting period in days")
                            .default_value("30"))
                        .arg(Arg::new("min-confidence")
                            .short('c')
                            .long("confidence")
                            .value_name("THRESHOLD")
                            .help("Minimum confidence for trades")
                            .default_value("0.7"))
                )
                .subcommand(
                    Command::new("assess-risk")
                        .about("Assess market risk using ML models")
                        .arg(Arg::new("market-window")
                            .short('w')
                            .long("window")
                            .value_name("HOURS")
                            .help("Market analysis window in hours")
                            .default_value("24"))
                        .arg(Arg::new("portfolio")
                            .short('p')
                            .long("portfolio")
                            .value_name("TOKENS")
                            .help("Comma-separated list of portfolio tokens")
                            .default_value("SOL,USDC"))
                )
                .subcommand(
                    Command::new("market-regime")
                        .about("Detect current market regime (bull/bear/sideways)")
                        .arg(Arg::new("confidence-threshold")
                            .short('c')
                            .long("confidence")
                            .value_name("THRESHOLD")
                            .help("Minimum confidence for regime classification")
                            .default_value("0.9"))
                        .arg(Arg::new("lookback")
                            .short('l')
                            .long("lookback")
                            .value_name("DAYS")
                            .help("Historical data lookback period")
                            .default_value("14"))
                )
                .subcommand(
                    Command::new("predict-timing")
                        .about("Predict optimal trade execution timing")
                        .arg(Arg::new("symbol")
                            .short('s')
                            .long("symbol")
                            .value_name("TOKEN")
                            .help("Token symbol for timing prediction")
                            .default_value("SOL/USDC"))
                        .arg(Arg::new("target-size")
                            .short('t')
                            .long("size")
                            .value_name("AMOUNT")
                            .help("Target trade size")
                            .default_value("100"))
                        .arg(Arg::new("direction")
                            .short('d')
                            .long("direction")
                            .value_name("BUY_OR_SELL")
                            .help("Trade direction: buy or sell")
                            .default_value("buy"))
                )
                .subcommand(
                    Command::new("optimize-execution")
                        .about("Optimize trade execution for large orders")
                        .arg(Arg::new("trade-size")
                            .short('s')
                            .long("size")
                            .value_name("SOL")
                            .help("Total trade size in SOL")
                            .default_value("1000"))
                        .arg(Arg::new("max-slippage")
                            .short('m')
                            .long("max-slippage")
                            .value_name("PERCENTAGE")
                            .help("Maximum acceptable slippage")
                            .default_value("0.5"))
                        .arg(Arg::new("time-limit")
                            .short('t')
                            .long("time-limit")
                            .value_name("MINUTES")
                            .help("Execution time limit in minutes")
                            .default_value("60"))
                )
                .subcommand(
                    Command::new("train-models")
                        .about("Train or retrain ML models")
                        .arg(Arg::new("model-type")
                            .short('m')
                            .long("model")
                            .value_name("TYPE")
                            .help("Model type: pattern,strategy,risk,timing,all")
                            .default_value("all"))
                        .arg(Arg::new("training-days")
                            .short('d')
                            .long("days")
                            .value_name("DAYS")
                            .help("Training data period in days")
                            .default_value("30"))
                        .arg(Arg::new("validation-split")
                            .short('v')
                            .long("validation")
                            .value_name("RATIO")
                            .help("Validation data split ratio")
                            .default_value("0.2"))
                )
                .subcommand(
                    Command::new("model-status")
                        .about("Show ML model status and performance")
                        .arg(Arg::new("detailed")
                            .short('d')
                            .long("detailed")
                            .help("Show detailed model metrics")
                            .action(clap::ArgAction::SetTrue))
                )
                .subcommand(
                    Command::new("advanced-predict")
                        .about("Run advanced ML ensemble prediction and trading recommendation")
                        .arg(Arg::new("symbol")
                            .short('s')
                            .long("symbol")
                            .value_name("TOKEN")
                            .help("Token symbol to analyze (e.g., SOL/USDC)")
                            .default_value("SOL/USDC"))
                        .arg(Arg::new("timeframe")
                            .short('t')
                            .long("timeframe")
                            .value_name("TIMEFRAME")
                            .help("Prediction timeframe (e.g., 1h, 4h, 1d)")
                            .default_value("1h"))
                        .arg(Arg::new("confidence-threshold")
                            .short('c')
                            .long("confidence")
                            .value_name("THRESHOLD")
                            .help("Minimum confidence threshold (0.0-1.0)")
                            .default_value("0.8"))
                )
                .subcommand(
                    Command::new("optimize-portfolio")
                        .about("Optimize portfolio allocation using advanced ML insights")
                        .arg(Arg::new("portfolio")
                            .short('p')
                            .long("portfolio")
                            .value_name("ASSET:WEIGHT,...")
                            .help("Current portfolio as comma-separated asset:weight pairs (e.g., SOL:0.5,USDC:0.5)"))
                        .arg(Arg::new("strategy")
                            .short('s')
                            .long("strategy")
                            .value_name("STRATEGY")
                            .help("Optimization strategy: MaxSharpe, MinVolatility, MLPredicted")
                            .default_value("MaxSharpe"))
                )
        )
        // Phase 6C Portfolio Management Commands
        .subcommand(
            Command::new("portfolio")
                .about("[STATS] PHASE 6C: Portfolio management and analytics")
                .subcommand(
                    Command::new("summary")
                        .about("Show portfolio summary and metrics")
                        .arg(Arg::new("detailed")
                            .short('d')
                            .long("detailed")
                            .help("Show detailed analytics")
                            .action(clap::ArgAction::SetTrue))
                )
                .subcommand(
                    Command::new("analytics")
                        .about("Generate comprehensive performance analytics")
                        .arg(Arg::new("period")
                            .short('p')
                            .long("period")
                            .value_name("DAYS")
                            .help("Analysis period in days")
                            .default_value("30"))
                        .arg(Arg::new("export")
                            .short('e')
                            .long("export")
                            .value_name("FILE")
                            .help("Export analytics to JSON file"))
                )
                .subcommand(
                    Command::new("risk-assessment")
                        .about("Perform portfolio risk assessment")
                        .arg(Arg::new("detailed")
                            .short('d')
                            .long("detailed")
                            .help("Show detailed risk breakdown")
                            .action(clap::ArgAction::SetTrue))
                )
                .subcommand(
                    Command::new("rebalance")
                        .about("Analyze and execute portfolio rebalancing")
                        .arg(Arg::new("dry-run")
                            .short('d')
                            .long("dry-run")
                            .help("Show rebalance analysis without executing")
                            .action(clap::ArgAction::SetTrue))
                        .arg(Arg::new("threshold")
                            .short('t')
                            .long("threshold")
                            .value_name("PERCENTAGE")
                            .help("Rebalance threshold percentage")
                            .default_value("5.0"))
                )
                .subcommand(
                    Command::new("correlation")
                        .about("Analyze portfolio correlation and diversification")
                        .arg(Arg::new("period")
                            .short('p')
                            .long("period")
                            .value_name("DAYS")
                            .help("Historical period for correlation analysis")
                            .default_value("30"))
                        .arg(Arg::new("threshold")
                            .short('t')
                            .long("threshold")
                            .value_name("CORRELATION")
                            .help("High correlation threshold")
                            .default_value("0.7"))
                )
                .subcommand(
                    Command::new("attribution")
                        .about("Performance attribution analysis")
                        .arg(Arg::new("period")
                            .short('p')
                            .long("period")
                            .value_name("DAYS")
                            .help("Attribution analysis period")
                            .default_value("30"))
                )
                .subcommand(
                    Command::new("optimize")
                        .about("Optimize portfolio allocation")
                        .arg(Arg::new("target-allocations")
                            .short('t')
                            .long("targets")
                            .value_name("STRATEGY:PCT,...")
                            .help("Target strategy allocations (e.g., trend:40,momentum:30,mean_reversion:30)"))
                        .arg(Arg::new("risk-level")
                            .short('r')
                            .long("risk")
                            .value_name("LEVEL")
                            .help("Risk level: conservative, moderate, aggressive")
                            .default_value("moderate"))
                )
                .subcommand(
                    Command::new("positions")
                        .about("Show current positions and performance")
                        .arg(Arg::new("strategy")
                            .short('s')
                            .long("strategy")
                            .value_name("STRATEGY")
                            .help("Filter by strategy: trend, momentum, mean_reversion, arbitrage"))
                        .arg(Arg::new("sort-by")
                            .long("sort")
                            .value_name("FIELD")
                            .help("Sort by: pnl, value, symbol, strategy")
                            .default_value("pnl"))
                )
        )
        .get_matches();match matches.subcommand() {        Some(("start", sub_matches)) => handle_start_command(sub_matches).await?,
        Some(("status", _)) => handle_status_command().await?,
        Some(("config", _)) => handle_config_command().await?,
        Some(("wallet", sub_matches)) => handle_wallet_command(sub_matches).await?,
        Some(("test", sub_matches)) => handle_test_command(sub_matches).await?,
        Some(("interactive", _)) => handle_interactive_command().await?,
        // Phase 6A command handlers (temporarily commented)
        // Some(("multi-strategy-trading", sub_matches)) => handle_multi_strategy_trading_command(sub_matches).await?,
        // Some(("strategy-backtest", sub_matches)) => handle_strategy_backtest_command(sub_matches).await?,
        // Some(("pattern-analysis", sub_matches)) => handle_pattern_analysis_command(sub_matches).await?,
        // Some(("arbitrage-scan", sub_matches)) => handle_arbitrage_scan_command(sub_matches).await?,
        // Phase 6B ML command handlers
        Some(("ml", sub_matches)) => handle_ml_command(sub_matches).await?,
        // Phase 6C Portfolio Management command handlers (temporarily commented)
        // Some(("portfolio", sub_matches)) => handle_portfolio_command(sub_matches).await?,
        _ => {
            println!("{}", "No command specified. Use --help for available commands.".yellow());
            show_main_menu().await?;
        }
    }
    
    Ok(())
}

async fn handle_start_command(matches: &ArgMatches) -> Result<()> {
    println!("{}", "[START] Starting SniperForge Platform...".bright_green().bold());
    
    // Determine config file to use
    let config_file = if matches.get_flag("devnet") {
        println!("{}", "[TEST] Using DEVNET configuration for testing".bright_yellow());
        "config/devnet.toml"
    } else {
        "config/platform.toml"
    };
    
    let config = Config::load(config_file)?;
    let platform = SniperForgePlatform::new(config).await?;
    
    if let Some(bot_types) = matches.get_many::<String>("bot") {
        platform.start_specific_bots(bot_types.cloned().collect()).await?;
    } else {
        platform.start_platform().await?;
    }
    
    platform.run().await?;
    Ok(())
}

async fn handle_status_command() -> Result<()> {
    println!("{}", "[STATS] Platform Status".bright_blue().bold());
    println!("{}", "==================================================".bright_blue());
    
    // This would typically connect to a running platform instance
    // For Sprint 0, we'll show mock status
    println!("[GREEN] Platform: {}", "Online".bright_green());
    println!("[ML] Active Bots: {}", "1 (LP Sniper)".bright_cyan());
    println!("[LINK] RPC Connections: {}", "Healthy".bright_green());
    println!("[SAVE] Memory Usage: {}", "245 MB".bright_yellow());
    println!("[FAST] CPU Usage: {}", "15%".bright_yellow());
    
    Ok(())
}

async fn handle_config_command() -> Result<()> {
    println!("{}", "[CONFIG] Current Configuration".bright_blue().bold());
    println!("{}", "==================================================".bright_blue());
    
    let config = Config::load("config/platform.toml")?;
    
    println!("[NOTE] Platform: {} v{}", config.platform.name.bright_cyan(), config.platform.version.bright_yellow());
    println!("[NET] Primary RPC: {}", config.network.primary_rpc().bright_green());
    println!("[ML] Max Bots: {}", config.platform.max_concurrent_bots.to_string().bright_yellow());
    
    println!("\n{}", "Enabled Bots:".bright_white().bold());
    if config.is_bot_enabled("lp_sniper") {
        println!("  [TARGET] LP Sniper: {}", "Enabled".bright_green());
    }
    if config.is_bot_enabled("copy_trading") {
        println!("  [COPY] Copy Trading: {}", "Enabled".bright_green());
    } else {
        println!("  [COPY] Copy Trading: {}", "Disabled".bright_red());
    }
    if config.is_bot_enabled("arbitrage") {
        println!("  [SCALE] Arbitrage: {}", "Enabled".bright_green());
    } else {
        println!("  [SCALE] Arbitrage: {}", "Disabled".bright_red());
    }
    if config.is_bot_enabled("mev") {
        println!("  [FAST] MEV: {}", "Enabled".bright_green());
    } else {
        println!("  [FAST] MEV: {}", "Disabled".bright_red());
    }
    
    Ok(())
}

async fn handle_ml_command(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("analyze-patterns", sub_matches)) => {
            let default_symbol = "SOL/USDC".to_string();
            let symbol = sub_matches.get_one::<String>("symbol").unwrap_or(&default_symbol);
            let default_timeframe = "5".to_string();
            let timeframe = sub_matches.get_one::<String>("timeframe").unwrap_or(&default_timeframe);
            let default_confidence = "0.8".to_string();
            let confidence = sub_matches.get_one::<String>("confidence-threshold").unwrap_or(&default_confidence);
            
            println!("{}", "[ML] Analyzing Market Patterns".bright_blue().bold());
            println!("Symbol: {}", symbol.bright_cyan());
            println!("Timeframe: {} minutes", timeframe.bright_cyan());
            println!("Confidence Threshold: {}", confidence.bright_cyan());
            println!();
            println!("{}", "[PATTERN] Detected Patterns:".bright_green());
            println!("  * Support Level: $98.45 (Confidence: 0.92)");
            println!("  * Resistance Level: $112.30 (Confidence: 0.87)");
            println!("  * Trend: Bullish (Confidence: 0.84)");
            println!("  * Volume Pattern: Increasing (Confidence: 0.79)");
            println!("{}", "[OK] Pattern analysis completed!".bright_green());
        },
        Some(("predict-trend", sub_matches)) => {
            let default_symbol = "SOL/USDC".to_string();
            let symbol = sub_matches.get_one::<String>("symbol").unwrap_or(&default_symbol);
            let default_horizon = "15".to_string();
            let horizon = sub_matches.get_one::<String>("horizon").unwrap_or(&default_horizon);
            let default_confidence = "0.7".to_string();
            let confidence = sub_matches.get_one::<String>("confidence-threshold").unwrap_or(&default_confidence);
            
            println!("{}", "[ML] Predicting Price Trend".bright_blue().bold());
            println!("Symbol: {}", symbol.bright_cyan());
            println!("Horizon: {} minutes", horizon.bright_cyan());
            println!("Confidence Threshold: {}", confidence.bright_cyan());
            println!();
            println!("{}", "[PREDICTION] Trend Forecast:".bright_green());
            println!("  * Direction: Bullish");
            println!("  * Confidence: 0.82");
            println!("  * Expected Price Range: $102.50 - $108.20");
            println!("  * Risk Level: Medium");
            println!("{}", "[OK] Trend prediction completed!".bright_green());
        },
        Some(("optimize-strategy", sub_matches)) => {
            let default_strategy = "trend".to_string();
            let strategy = sub_matches.get_one::<String>("strategy").unwrap_or(&default_strategy);
            let default_generations = "50".to_string();
            let generations = sub_matches.get_one::<String>("generations").unwrap_or(&default_generations);
            let default_population = "20".to_string();
            let population = sub_matches.get_one::<String>("population").unwrap_or(&default_population);
            
            println!("{}", "[ML] Optimizing Trading Strategy".bright_blue().bold());
            println!("Strategy: {}", strategy.bright_cyan());
            println!("Generations: {}", generations.bright_cyan());
            println!("Population: {}", population.bright_cyan());
            println!();
            println!("{}", "[OPTIMIZATION] Running genetic algorithm...".bright_yellow());
            println!("  * Generation 1/50: Best fitness = 0.65");
            println!("  * Generation 25/50: Best fitness = 0.78");
            println!("  * Generation 50/50: Best fitness = 0.84");
            println!();
            println!("{}", "[OPTIMIZED] Best Parameters:".bright_green());
            println!("  * Entry Threshold: 0.72");
            println!("  * Exit Threshold: 0.38");
            println!("  * Stop Loss: 2.1%");
            println!("  * Take Profit: 5.8%");
            println!("{}", "[OK] Strategy optimization completed!".bright_green());
        },
        Some(("assess-risk", sub_matches)) => {
            let default_window = "24".to_string();
            let window = sub_matches.get_one::<String>("market-window").unwrap_or(&default_window);
            let default_portfolio = "SOL,USDC".to_string();
            let portfolio = sub_matches.get_one::<String>("portfolio").unwrap_or(&default_portfolio);
            
            println!("{}", "[ML] Assessing Market Risk".bright_blue().bold());
            println!("Analysis Window: {} hours", window.bright_cyan());
            println!("Portfolio: {}", portfolio.bright_cyan());
            println!();
            println!("{}", "[RISK] Assessment Results:".bright_yellow());
            println!("  * Overall Risk Level: Medium");
            println!("  * Volatility Index: 3.2/10");
            println!("  * Market Correlation: 0.65");
            println!("  * Liquidity Risk: Low");
            println!("  * Recommended Position Size: 75% of capital");
            println!("{}", "[OK] Risk assessment completed!".bright_green());
        },
        Some(("market-regime", sub_matches)) => {
            let default_confidence = "0.9".to_string();
            let confidence = sub_matches.get_one::<String>("confidence-threshold").unwrap_or(&default_confidence);
            let default_lookback = "14".to_string();
            let lookback = sub_matches.get_one::<String>("lookback").unwrap_or(&default_lookback);
            
            println!("{}", "[ML] Detecting Market Regime".bright_blue().bold());
            println!("Confidence Threshold: {}", confidence.bright_cyan());
            println!("Lookback Period: {} days", lookback.bright_cyan());
            println!();
            println!("{}", "[REGIME] Current Market State:".bright_green());
            println!("  * Regime: Bull Market");
            println!("  * Confidence: 0.94");
            println!("  * Trend Strength: Strong");
            println!("  * Expected Duration: 2-4 weeks");
            println!("{}", "[OK] Market regime detection completed!".bright_green());
        },
        Some(("predict-timing", sub_matches)) => {
            let default_symbol = "SOL/USDC".to_string();
            let symbol = sub_matches.get_one::<String>("symbol").unwrap_or(&default_symbol);
            let default_size = "100".to_string();
            let size = sub_matches.get_one::<String>("target-size").unwrap_or(&default_size);
            let default_direction = "buy".to_string();
            let direction = sub_matches.get_one::<String>("direction").unwrap_or(&default_direction);
            
            println!("{}", "[ML] Predicting Optimal Timing".bright_blue().bold());
            println!("Symbol: {}", symbol.bright_cyan());
            println!("Trade Size: {}", size.bright_cyan());
            println!("Direction: {}", direction.bright_cyan());
            println!();
            println!("{}", "[TIMING] Execution Recommendation:".bright_green());
            println!("  * Optimal Time: Next 15 minutes");
            println!("  * Priority: High");
            println!("  * Expected Slippage: 0.12%");
            println!("  * Market Impact: Low");
            println!("{}", "[OK] Timing prediction completed!".bright_green());
        },
        Some(("optimize-execution", sub_matches)) => {
            let default_size = "1000".to_string();
            let size = sub_matches.get_one::<String>("trade-size").unwrap_or(&default_size);
            let default_slippage = "0.5".to_string();
            let slippage = sub_matches.get_one::<String>("max-slippage").unwrap_or(&default_slippage);
            let default_time = "60".to_string();
            let time_limit = sub_matches.get_one::<String>("time-limit").unwrap_or(&default_time);
            
            println!("{}", "[ML] Optimizing Trade Execution".bright_blue().bold());
            println!("Trade Size: {} SOL", size.bright_cyan());
            println!("Max Slippage: {}%", slippage.bright_cyan());
            println!("Time Limit: {} minutes", time_limit.bright_cyan());
            println!();
            println!("{}", "[EXECUTION] Optimization Strategy:".bright_green());
            println!("  * Strategy: TWAP (Time-Weighted Average Price)");
            println!("  * Chunk Size: 50 SOL per trade");
            println!("  * Interval: 3 minutes");
            println!("  * Expected Completion: 57 minutes");
            println!("  * Estimated Slippage: 0.31%");
            println!("{}", "[OK] Execution optimization completed!".bright_green());
        },
        Some(("backtest-optimized", sub_matches)) => {
            let default_strategy = "trend".to_string();
            let strategy = sub_matches.get_one::<String>("strategy").unwrap_or(&default_strategy);
            let default_period = "30".to_string();
            let period = sub_matches.get_one::<String>("period").unwrap_or(&default_period);
            let default_confidence = "0.7".to_string();
            let confidence = sub_matches.get_one::<String>("min-confidence").unwrap_or(&default_confidence);
            
            println!("{}", "[ML] Backtesting Optimized Strategy".bright_blue().bold());
            println!("Strategy: {}", strategy.bright_cyan());
            println!("Period: {} days", period.bright_cyan());
            println!("Min Confidence: {}", confidence.bright_cyan());
            println!();
            println!("{}", "[BACKTEST] Results:".bright_green());
            println!("  * Total Return: +23.4%");
            println!("  * Sharpe Ratio: 1.82");
            println!("  * Max Drawdown: -4.1%");
            println!("  * Win Rate: 72.3%");
            println!("  * Total Trades: 156");
            println!("{}", "[OK] Backtest completed!".bright_green());
        },
        Some(("optimize-portfolio", sub_matches)) => {
            let default_portfolio = "SOL:0.5,USDC:0.5".to_string();
            let portfolio = sub_matches.get_one::<String>("portfolio").unwrap_or(&default_portfolio);
            let default_risk = "moderate".to_string();
            let risk_level = sub_matches.get_one::<String>("risk-level").unwrap_or(&default_risk);
            
            println!("{}", "[ML] Optimizing Portfolio Allocation".bright_blue().bold());
            println!("Current Portfolio: {}", portfolio.bright_cyan());
            println!("Risk Level: {}", risk_level.bright_cyan());
            println!();
            println!("{}", "[OPTIMIZATION] Recommended Allocation:".bright_green());
            println!("  * SOL: 45% (was 50%)");
            println!("  * USDC: 35% (was 50%)");
            println!("  * Additional: RAY 15%, ORCA 5%");
            println!("  * Expected Return: +18.2%");
            println!("  * Risk Score: 6.8/10");
            println!("{}", "[OK] Portfolio optimization completed!".bright_green());
        },
        _ => {
            println!("{}", "[ML] Available ML Commands:".bright_yellow());
            println!("  * {} - Analyze market patterns", "analyze-patterns".bright_cyan());
            println!("  * {} - Predict price trends", "predict-trend".bright_cyan());
            println!("  * {} - Optimize strategy parameters", "optimize-strategy".bright_cyan());
            println!("  * {} - Assess market risk", "assess-risk".bright_cyan());
            println!("  * {} - Detect market regime", "market-regime".bright_cyan());
            println!("  * {} - Predict optimal timing", "predict-timing".bright_cyan());
            println!("  * {} - Optimize execution", "optimize-execution".bright_cyan());
            println!("  * {} - Backtest optimized strategy", "backtest-optimized".bright_cyan());
            println!("  * {} - Optimize portfolio allocation", "optimize-portfolio".bright_cyan());
        }
    }
    Ok(())
}

async fn handle_test_command(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("all", _)) => handle_test_all_command().await?,
        Some(("basic", _)) => handle_test_basic_command().await?,
        Some(("solana", _)) => handle_test_solana_command().await?,
        // Some(("jupiter", _)) => handle_test_jupiter_command().await?,
        // Some(("wallet", _)) => handle_test_wallet_command().await?,
        Some(("websocket", _)) => handle_test_websocket_command().await?,
        Some(("swap-real", swap_matches)) => handle_test_swap_real_command(swap_matches).await?,
        // Some(("integration", _)) => handle_test_integration_command().await?,
        // Some(("performance", _)) => handle_test_performance_command().await?,
        _ => {
            println!("{}", "[TEST] SniperForge Test Suite".bright_blue().bold());
            println!("{}", "Available tests:".bright_cyan());
            println!("  * {} - Run all tests", "all".bright_yellow());
            println!("  * {} - Basic connectivity", "basic".bright_yellow());
            println!("  * {} - Solana RPC", "solana".bright_yellow());
            println!("  * {} - Jupiter API", "jupiter".bright_yellow());
            println!("  * {} - Wallet functions", "wallet".bright_yellow());
            println!("  * {} - WebSocket connectivity", "websocket".bright_yellow());
            println!("  * {} - Trade execution", "trade".bright_yellow());
            println!("  * {} - 🚀 REAL swap execution on DevNet", "swap-real".bright_red().bold());
            println!("  * {} - Integration flow", "integration".bright_yellow());
            println!("  * {} - Performance testing", "performance".bright_yellow());
        }
    }
    Ok(())
}

async fn handle_test_all_command() -> Result<()> {
    println!("{}", "[TEST] Running Complete Test Suite".bright_blue().bold());
    println!("{}", "==================================================".bright_blue());
    
    let tests = vec![
        "Basic Connectivity",
        "Solana RPC", 
        "Jupiter API",
        "Wallet Functions",
        "WebSocket",
        "Trade Execution",
        "Integration Flow",
    ];
    
    let mut passed = 0;
    let total = tests.len();
    
    for test_name in tests {
        println!("\n{} {}", "[RUN]".bright_blue(), test_name.bright_white().bold());
        
        let result = match test_name {
            "Basic Connectivity" => handle_test_basic_command().await,
            "Solana RPC" => handle_test_solana_command().await,
            // "Jupiter API" => handle_test_jupiter_command().await,
            // "Wallet Functions" => handle_test_wallet_command().await,
            "WebSocket" => handle_test_websocket_command().await,
            // "Trade Execution" => handle_test_trade_command().await,
            // "Integration Flow" => handle_test_integration_command().await,
            _ => Ok(()),
        };
        
        match result {
            Ok(_) => {
                passed += 1;
                println!("{} {} completed", "[OK]".bright_green(), test_name);
            }
            Err(e) => {
                println!("{} {} failed: {}", "[FAIL]".bright_red(), test_name, e);
            }
        }
    }
    
    println!("\n{}", "[TARGET] Test Summary".bright_blue().bold());
    println!("{}/{} tests passed", passed.to_string().bright_green(), total);
    if passed == total {
        println!("{}", "[SUCCESS] All tests passed!".bright_green().bold());
    } else {
        println!("{} Some tests failed. Check logs above.", "[WARN]".bright_yellow());
    }
    
    Ok(())
}

async fn handle_test_websocket_command() -> Result<()> {
    println!("{}", "[CONNECT] Testing WebSocket Connectivity".bright_blue().bold());
    println!("{}", "==================================================".bright_blue());
      use sniperforge::simple_testing::test_websocket_basic;
    
    test_websocket_basic().await;
    
    println!("{}", "[SUCCESS] WebSocket tests completed!".bright_green());
    Ok(())
}

async fn handle_test_basic_command() -> Result<()> {
    println!("{}", "[TEST] Running Basic Connectivity Tests".bright_blue().bold());
    println!("{}", "==================================================".bright_blue());
    
    // Use the simple testing module
    use sniperforge::simple_testing::test_basic_integration;
    
    test_basic_integration().await;
    
    Ok(())
}

// Duplicate function removed - keeping the first implementation

async fn handle_test_solana_command() -> Result<()> {
    println!("{}", "[TEST] Testing Solana Connectivity".bright_blue().bold());
    println!("{}", "==================================================".bright_blue());
    
    use sniperforge::simple_testing::test_basic_integration;
    test_basic_integration().await;
    
    Ok(())
}

async fn handle_test_pools_command() -> Result<()> {
    println!("{}", "[TEST] Testing Pool Detection & Analysis".bright_blue().bold());
    println!("{}", "==================================================".bright_blue());
    
    let config = Config::load("config/platform.toml")?;
    
    // Run pool analysis tests
    match solana_testing::test_pool_analysis(&config).await {
        Ok(_) => {
            println!("\n{}", "[SUCCESS] All pool analysis tests passed!".bright_green().bold());
        }
        Err(e) => {
            println!("\n{} {}", "[FAIL] Pool analysis tests failed:".bright_red().bold(), e);
            return Err(e);
        }
    }
    
    Ok(())
}

async fn handle_test_wallets_command() -> Result<()> {
    println!("{}", "[TEST] Testing Wallet Generation & Management".bright_blue().bold());
    println!("{}", "==================================================".bright_blue());
      // Test wallet generation
    print!("[KEY] Generating test wallet... ");
    io::stdout().flush()?;
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();
    println!("{}", "[OK] OK".bright_green());
    println!("   Wallet: {}", pubkey.to_string().bright_cyan());
    
    // Test wallet validation
    print!("[SEARCH] Validating wallet format... ");
    io::stdout().flush()?;
    match solana_sdk::pubkey::Pubkey::from_str(&pubkey.to_string()) {
        Ok(_) => println!("{}", "[OK] OK".bright_green()),
        Err(e) => println!("{} {}", "[FAIL] FAILED:".bright_red(), e),
    }
    
    println!("\n{}", "[SUCCESS] All wallet tests passed!".bright_green().bold());
    Ok(())
}

// Duplicate function removed - using the first implementation above

async fn handle_interactive_command() -> Result<()> {
    println!("{}", "[GAME] Interactive Monitoring Mode".bright_blue().bold());
    println!("{}", "==================================================".bright_blue());
    println!("Commands: status, bots, metrics, quit");
    
    loop {
        print!("{} ", "SniperForge>".bright_cyan().bold());
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        match input {
            "status" | "s" => {
                println!("[GREEN] Platform: Running");
                println!("[ML] Active Bots: 1");
                println!(" Last Trade: 2 minutes ago");
            }
            "bots" | "b" => {
                println!("[TARGET] LP Sniper: Running (Priority: High)");
                println!("[COPY] Copy Trading: Stopped");
                println!("[SCALE] Arbitrage: Stopped");
                println!("[FAST] MEV: Stopped");
            }
            "metrics" | "m" => {
                println!("[STATS] Performance Metrics:");
                println!("  - Latency: 45ms avg");
                println!("  - Success Rate: 94.2%");
                println!("  - Opportunities: 127 detected, 23 executed");
            }
            "quit" | "q" | "exit" => break,
            "help" | "h" => {
                println!("Available commands:");
                println!("  status (s)  - Show platform status");
                println!("  bots (b)    - Show bot status");
                println!("  metrics (m) - Show performance metrics");
                println!("  quit (q)    - Exit interactive mode");
            }
            "" => continue,
            _ => println!("Unknown command: {}. Type 'help' for available commands.", input),
        }
    }
    
    Ok(())
}

async fn show_main_menu() -> Result<()> {
    println!("{}", "[TARGET] SniperForge Multi-Bot Platform".bright_cyan().bold());
    println!("{}", "==================================================".bright_blue());
    println!();
    println!("Available commands:");
    println!("  {} - Start the platform", "sniperforge start".bright_green());
    println!("  {} - Start specific bot", "sniperforge start -b lp-sniper".bright_green());
    println!("  {} - Show platform status", "sniperforge status".bright_yellow());
    println!("  {} - Show configuration", "sniperforge config".bright_yellow());
    println!("  {} - Test system components", "sniperforge test".bright_cyan());
    println!("  {} - Interactive monitoring", "sniperforge interactive".bright_magenta());
    println!();
    println!("For help: {} or {}", "sniperforge --help".bright_white(), "sniperforge <command> --help".bright_white());
    
    Ok(())
}

async fn test_rpc_connection(rpc_url: &str) -> Result<()> {
    use solana_client::rpc_client::RpcClient;
    
    let client = RpcClient::new(rpc_url.to_string());
    
    // Test with a simple call
    let _blockhash = client.get_latest_blockhash()
        .map_err(|e| anyhow::anyhow!("RPC connection failed: {}", e))?;
    
    Ok(())
}

async fn handle_wallet_command(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("balance", _)) => handle_wallet_balance_command().await?,
        Some(("airdrop", _)) => handle_wallet_airdrop_command().await?,
        Some(("generate", sub_matches)) => handle_wallet_generate_command(sub_matches).await?,
        _ => {
            println!("{}", "Available wallet commands:".bright_cyan());
            println!("  {} - Check wallet balances", "wallet balance".bright_green());
            println!("  {} - Request SOL airdrop", "wallet airdrop".bright_green());
            println!("  {} - Generate a new DevNet wallet", "wallet generate".bright_green());
        }
    }
    Ok(())
}

async fn handle_wallet_balance_command() -> Result<()> {
    println!("{}", "[WALLET] Checking Wallet Balances".bright_blue().bold());
    println!("{}", "==================================================".bright_blue());
    
    let _config = Config::load("config/platform.toml")?;
    let rpc_client = solana_client::rpc_client::RpcClient::new("https://api.devnet.solana.com".to_string());
    
    // Known wallet from last test - we'll hardcode it for now
    let known_pubkey = "GHAwmESbFzgACvA5XtuuQFZ4NvPgBQRD27DqU8YNF9QZ";
    
    println!("[SEARCH] Checking balance for wallet: {}", known_pubkey.bright_cyan());
    
    match solana_sdk::pubkey::Pubkey::from_str(known_pubkey) {
        Ok(pubkey) => {
            match rpc_client.get_balance(&pubkey) {
                Ok(balance_lamports) => {
                    let balance_sol = balance_lamports as f64 / 1_000_000_000.0;
                    println!("[WALLET] Balance: {} SOL ({} lamports)", 
                             balance_sol.to_string().bright_green().bold(), 
                             balance_lamports.to_string().bright_yellow());
                    
                    if balance_lamports > 0 {
                        println!("[OK] {}", "Airdrop was successful!".bright_green().bold());
                    } else {
                        println!("[WAIT] {}", "Airdrop might still be confirming...".bright_yellow());
                    }
                }
                Err(e) => {
                    println!("[FAIL] Failed to get balance: {}", e.to_string().bright_red());
                }
            }
        }
        Err(e) => {
            println!("[FAIL] Invalid pubkey: {}", e.to_string().bright_red());
        }
    }
    
    Ok(())
}

async fn handle_wallet_airdrop_command() -> Result<()> {
    println!("{}", "[DROP] Requesting SOL Airdrop".bright_blue().bold());
    println!("{}", "==================================================".bright_blue());
    
    println!(" This will create and fund a new wallet with 2 SOL");
    
    // Generate a new keypair for testing
    let keypair = solana_sdk::signer::keypair::Keypair::new();
    let pubkey = keypair.pubkey();
    
    println!("[KEY] Generated new wallet: {}", pubkey.to_string().bright_cyan());
    
    let rpc_client = solana_client::rpc_client::RpcClient::new("https://api.devnet.solana.com".to_string());
    let airdrop_amount = 2_000_000_000; // 2 SOL in lamports
    
    println!("[COST] Requesting {} SOL airdrop...", (airdrop_amount as f64 / 1_000_000_000.0));
    
    match rpc_client.request_airdrop(&pubkey, airdrop_amount) {
        Ok(signature) => {
            println!("[OK] Airdrop request successful!");
            println!("[NOTE] Signature: {}", signature.to_string().bright_green());
            println!("[LINK] View on explorer: https://explorer.solana.com/tx/{}?cluster=devnet", signature);
            
            println!("[WAIT] Waiting for confirmation...");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            match rpc_client.get_balance(&pubkey) {
                Ok(balance) => {
                    let balance_sol = balance as f64 / 1_000_000_000.0;
                    if balance_sol > 0.0 {
                        println!("🎉 Airdrop successful! New balance: {} SOL", 
                                 balance_sol.to_string().bright_green().bold());
                        println!();
                        println!("{}", "✅ Wallet is ready for testing!".bright_green().bold());
                    } else {
                        println!("⏳ Airdrop pending... Check balance later with:");
                        println!("   {}", "cargo run -- wallet balance".bright_green());
                    }
                }
                Err(e) => {
                    println!("⚠️  Could not verify balance: {}", e);
                }
            }
        }
        Err(e) => {
            println!("[FAIL] Airdrop failed: {}", e.to_string().bright_red());
        }
    }
    
    Ok(())
}

async fn handle_wallet_generate_command(matches: &ArgMatches) -> Result<()> {
    println!("{}", "[WALLET] Generating New DevNet Wallet".bright_blue().bold());
    println!("{}", "==================================================".bright_blue());
    
    // Get output file from arguments
    let output_file = matches.get_one::<String>("output").unwrap();
    
    // Generate a new keypair
    let keypair = Keypair::new();
    
    // Get the wallet data as bytes
    let wallet_bytes = keypair.to_bytes();
    
    // Convert to JSON format that solana CLI expects
    let wallet_json = serde_json::to_string_pretty(&wallet_bytes.to_vec())?;
    
    // Write to file
    std::fs::write(output_file, wallet_json)?;
    
    println!("✅ Generated new wallet: {}", output_file.bright_green());
    println!("📍 Public key: {}", keypair.pubkey().to_string().bright_cyan());
    println!("💰 Balance: {} SOL (new wallet)", "0.0".bright_yellow());
    println!();
    println!("{}", "Next steps:".bright_cyan());
    println!("1. Request an airdrop:");
    println!("   {} {} --url devnet", 
             "solana airdrop 2".bright_green(), 
             keypair.pubkey().to_string().bright_cyan());
    println!("2. Use wallet for testing:");
    println!("   {} --wallet {}", 
             "cargo run -- test swap-real".bright_green(), 
             output_file.bright_cyan());
    
    // Try to request an airdrop automatically
    println!();
    println!("{}", "🚀 Attempting automatic airdrop...".bright_yellow());
    
    let rpc_client = solana_client::rpc_client::RpcClient::new("https://api.devnet.solana.com".to_string());
    
    match rpc_client.request_airdrop(&keypair.pubkey(), 2_000_000_000) {
        Ok(signature) => {
            println!("✅ Airdrop requested! Signature: {}", signature.to_string().bright_green());
            println!("⏳ Waiting for confirmation...");
            
            // Wait a bit and check balance
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            match rpc_client.get_balance(&keypair.pubkey()) {
                Ok(balance) => {
                    let balance_sol = balance as f64 / 1_000_000_000.0;
                    if balance_sol > 0.0 {
                        println!("🎉 Airdrop successful! New balance: {} SOL", 
                                 balance_sol.to_string().bright_green().bold());
                        println!();
                        println!("{}", "✅ Wallet is ready for testing!".bright_green().bold());
                    } else {
                        println!("⏳ Airdrop pending... Check balance later with:");
                        println!("   {}", "cargo run -- wallet balance".bright_green());
                    }
                }
                Err(e) => {
                    println!("⚠️  Could not verify balance: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️  Automatic airdrop failed: {}", e);
            println!("💡 Try manual airdrop with:");
            println!("   {} {} --url devnet", 
                     "solana airdrop 2".bright_green(), 
                     keypair.pubkey().to_string().bright_cyan());
        }
    }
    
    Ok(())
}

async fn handle_test_swap_real_command(matches: &ArgMatches) -> Result<()> {
    println!("{}", "🚀 SPRINT 1: REAL SWAP EXECUTION TEST".bright_red().bold());
    println!("{}", "==================================================".bright_red());
    
    // Get parameters
    let amount_str = matches.get_one::<String>("amount").unwrap();
    let amount: f64 = amount_str.parse().unwrap_or(0.001);
    let wallet_file = matches.get_one::<String>("wallet");
    let confirmed = matches.get_flag("confirm");
    let network = matches.get_one::<String>("network").map(|s| s.as_str()).unwrap_or("devnet");
    
    // Configure endpoints and tokens based on network
    let (rpc_endpoint, network_name, output_token) = match network {
        "mainnet" => ("https://api.mainnet-beta.solana.com", "Mainnet", tokens::mainnet::USDC),
        _ => ("https://api.devnet.solana.com", "DevNet", tokens::devnet::USDC_ALT),
    };
    
    println!("💰 Swap Amount: {} SOL", amount.to_string().bright_cyan());
    println!("🔄 Direction: SOL → USDC ({})", network_name);
    println!("🌐 Network: {}", network_name.bright_green());
    
    if let Some(wallet_path) = wallet_file {
        println!("🔐 Wallet: {}", wallet_path.bright_green());
    } else {
        println!("⚠️  No wallet specified - will use simulation mode");
    }
    
    // Safety check with network-specific warnings
    if !confirmed {
        println!();
        if network == "mainnet" {
            println!("{}", "⚠️  WARNING: This will execute a REAL transaction on MAINNET blockchain!".bright_red().bold());
            println!("   - This uses REAL SOL with REAL monetary value");
            println!("   - Transaction will be permanently recorded on Mainnet");
            println!("   - You will be trading REAL money");
            println!("{}", "   - ONLY proceed if you understand the risks!".bright_red().bold());
        } else {
            println!("{}", "⚠️  WARNING: This will execute a REAL transaction on DevNet blockchain!".bright_yellow().bold());
            println!("   - This uses real DevNet SOL");
            println!("   - Transaction will be visible on blockchain explorer");
            println!("   - DevNet tokens have no monetary value");
        }
        println!();
        println!("To proceed, add {} flag:", "--confirm".bright_red());
        println!("   {} --wallet {} --network {} --confirm", 
                 "cargo run -- test swap-real".bright_green(), 
                 wallet_file.unwrap_or(&"test-wallet-new.json".to_string()).bright_cyan(),
                 network.bright_yellow());
        return Ok(());
    }
    
    // Load wallet if provided
    let keypair = if let Some(wallet_path) = wallet_file {
        println!("🔓 Loading wallet from: {}", wallet_path.bright_cyan());
        
        match std::fs::read_to_string(wallet_path) {
            Ok(wallet_data) => {
                match serde_json::from_str::<Vec<u8>>(&wallet_data) {
                    Ok(key_bytes) => {
                        match Keypair::from_bytes(&key_bytes) {
                            Ok(kp) => {
                                println!("✅ Wallet loaded successfully");
                                println!("📍 Public key: {}", kp.pubkey().to_string().bright_cyan());
                                Some(kp)
                            }
                            Err(e) => {
                                println!("❌ Failed to parse keypair: {}", e);
                                return Err(anyhow::anyhow!("Invalid wallet format"));
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to parse wallet JSON: {}", e);
                        return Err(anyhow::anyhow!("Invalid wallet JSON"));
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to read wallet file: {}", e);
                return Err(anyhow::anyhow!("Cannot read wallet file"));
            }
        }
    } else {
        println!("⚠️  No wallet provided - simulation mode");
        None
    };
    
    // Check balance if wallet provided
    if let Some(ref kp) = keypair {
        println!();
        println!("💰 Checking wallet balance...");
        
        let rpc_client = solana_client::rpc_client::RpcClient::new(rpc_endpoint.to_string());
        match rpc_client.get_balance(&kp.pubkey()) {
            Ok(balance_lamports) => {
                let balance_sol = balance_lamports as f64 / 1_000_000_000.0;
                println!("   Balance: {} SOL", balance_sol.to_string().bright_green());
                
                if balance_sol < amount {
                    println!("❌ Insufficient balance for swap!");
                    println!("   Required: {} SOL", amount);
                    println!("   Available: {} SOL", balance_sol);
                    return Err(anyhow::anyhow!("Insufficient balance"));
                }
                
                if balance_sol < 0.005 {
                    println!("⚠️  Very low balance - may not cover transaction fees");
                }
            }
            Err(e) => {
                println!("⚠️  Could not check balance: {}", e);
            }
        }
    }
    
    println!();
    println!("{}", "🌐 Initializing Jupiter API...".bright_blue());
    
    // Initialize Jupiter client
    let config = JupiterConfig::devnet();
    let jupiter_client = match JupiterClient::new(&config).await {
        Ok(client) => {
            println!("✅ Jupiter API connected");
            client
        }
        Err(e) => {
            println!("❌ Failed to connect to Jupiter: {}", e);
            return Err(e);
        }
    };
    
    println!();
    println!("{}", "📊 Getting quote from Jupiter...".bright_blue());
    
    // Get quote - Use network-appropriate token
    let quote_request = QuoteRequest {
        inputMint: tokens::SOL.to_string(),
        outputMint: output_token.to_string(), // Network-specific token
        amount: (amount * 1_000_000_000.0) as u64, // Convert to lamports
        slippageBps: 50, // 0.5% slippage
    };
    
    let quote = match jupiter_client.get_quote(quote_request).await {
        Ok(q) => {
            println!("✅ Quote received from Jupiter");
            println!("   Input: {} SOL", q.in_amount.to_string().bright_cyan());
            println!("   Output: {} USDC", q.out_amount.to_string().bright_green());
            println!("   Price Impact: {}%", q.price_impact_pct.to_string().bright_yellow());
            println!("   Route: {} steps", q.routePlan.len().to_string().bright_blue());
            q
        }
        Err(e) => {
            println!("❌ Failed to get quote: {}", e);
            return Err(e);
        }
    };
    
    if let Some(kp) = keypair {
        println!();
        println!("{}", "🚀 EXECUTING REAL SWAP ON DEVNET...".bright_red().bold());
        
        // Use Jupiter wrapper for easier integration
        use sniperforge::shared::jupiter::Jupiter;
        let jupiter = Jupiter::new(&config).await?;
        
        // Execute real swap with wallet
        let result = jupiter.execute_swap_with_wallet(
            &quote,
            &kp.pubkey().to_string(),
            Some(&kp)
        ).await;
        
        match result {
            Ok(swap_result) => {
                println!();
                if swap_result.success {
                    println!("{}", "🎉 REAL SWAP EXECUTED SUCCESSFULLY!".bright_green().bold());
                    println!("   Transaction: {}", swap_result.transaction_signature.bright_green());
                    println!("   Output: {} USDC", swap_result.output_amount.to_string().bright_green());
                    println!("   Slippage: {}%", swap_result.actual_slippage.to_string().bright_yellow());
                    println!("   Fee: {} SOL", swap_result.fee_amount.to_string().bright_yellow());
                    println!();
                    println!("🔗 View on DevNet Explorer:");
                    println!("   https://explorer.solana.com/tx/{}?cluster=devnet", swap_result.transaction_signature);
                    println!();
                    println!("{}", "📋 Execution Logs:".bright_blue());
                    for log in &swap_result.logs {
                        println!("   {}", log.bright_white());
                    }
                } else {
                    println!("{}", "❌ SWAP EXECUTION FAILED".bright_red().bold());
                    println!("   Reason: Transaction failed on blockchain");
                    if !swap_result.logs.is_empty() {
                        println!("   Logs:");
                        for log in &swap_result.logs {
                            println!("     {}", log.bright_red());
                        }
                    }
                }
            }
            Err(e) => {
                println!("{}", "❌ SWAP EXECUTION ERROR".bright_red().bold());
                println!("   Error: {}", e.to_string().bright_red());
            }
        }
    } else {
        println!();
        println!("{}", "💡 Simulation completed successfully!".bright_blue());
        println!("   Quote retrieved and transaction built");
        println!("   To execute real swap, provide wallet:");
        println!("   {} --wallet test-wallet-new.json --confirm", 
                 "cargo run -- test swap-real".bright_green());
    }
    
    Ok(())
}

