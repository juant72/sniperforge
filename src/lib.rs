#![allow(dead_code)]
#![allow(unused_imports)]

pub mod analysis;
pub mod arbitrage;
pub mod bots;
pub mod config;
pub mod dexscreener_testing; // DexScreener API testing
pub mod enhanced_pool_discovery; // PROPOSAL-002: Multi-DEX pool discovery
pub mod expert; // EXPERT SYSTEM MODULES
pub mod jupiter_speed_test;
pub mod ml; // Phase 6B ML module
pub mod multi_dex_scanner; // PROPOSAL-002: Multi-DEX scanner system
pub mod multi_token_manager; // PROPOSAL-003: Multi-token management system
pub mod multi_token_config; // PROPOSAL-003: Multi-token configuration system
pub mod phase1; // Phase 1: Jupiter API Optimization
pub mod phase2; // Phase 2: MEV Protection
pub mod phase3; // Phase 3: DEX Specialization  
pub mod phase4; // Phase 4: Advanced Features (Event-driven, Parallel, Monitoring, Benchmarking)
pub mod phase4_demo; // Phase 4 demonstration and testing
pub mod platform;
pub mod portfolio; // Phase 6C Portfolio Management
pub mod shared;
pub mod simple_testing;
pub mod solana_testing;
pub mod sprint_1_demo; // Sprint 1 functionality demo
pub mod strategies;
pub mod tatum_testing; // Tatum RPC testing
pub mod trading; // DEV2 Trading Engine
pub mod types;
pub mod ultimate_rpc_test;
pub mod websocket_rpc_test; // Arbitrage detection and execution

pub use crate::config::Config;
pub use platform::SniperForgePlatform;
