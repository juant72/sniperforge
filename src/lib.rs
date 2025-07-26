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
pub mod ml_pattern_recognition; // ACCIÓN 8.1: Pattern Recognition Engine
pub mod multi_dex_scanner; // PROPOSAL-002: Multi-DEX scanner system
pub mod multi_token_manager; // PROPOSAL-003: Multi-token management system
pub mod multi_token_config; // PROPOSAL-003: Multi-token configuration system
pub mod phase1; // Phase 1: Jupiter API Optimization
pub mod phase2; // Phase 2: MEV Protection
pub mod phase3; // Phase 3: DEX Specialization  
pub mod phase4; // Phase 4: Advanced Features (Event-driven, Parallel, Monitoring, Benchmarking)
pub mod phase4_demo; // Phase 4 demonstration and testing
pub mod real_price_feeds; // REAL PRICE FEEDS: Obtener precios reales de múltiples DEXs
pub mod real_arbitrage_engine; // REAL ARBITRAGE ENGINE: Motor de arbitraje verdadero
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

// ===== PHASE 4.5 INTEGRATION MODULES (SIMPLIFICADOS) =====
// Sistema integrado que unifica todas las fases con evolución inteligente
pub mod unified_config; // Configuración unificada para todas las fases
pub mod jupiter_integration_simple; // Integración Phase 1: Jupiter Advanced (simplificado)
pub mod mev_integration_simple; // Integración Phase 2: MEV Protection (simplificado)
pub mod dex_integration_simple; // Integración Phase 3: DEX Specialization (simplificado)
pub mod event_driven_integration_simple; // Integración Phase 4: Event-driven + Parallel (simplificado)
pub mod arbitrage_bot_phase45_integrated; // Sistema principal integrado
pub mod jupiter_real_client; // Cliente real de Jupiter para swaps reales
pub mod jupiter_integration_real; // NUEVO: Jupiter Integration Real - Sistema completo de Jupiter trading
pub mod wallet_manager; // Gestión segura de wallets para trading real
// pub mod ml_integration_advanced; // ACCIÓN 7: Advanced Machine Learning Integration - Temporalmente comentado
pub mod advanced_real_trading; // ACCIÓN 7.2: Advanced Real Trading System

// ACCIÓN 7: ADVANCED OPTIMIZATIONS MODULES
pub mod advanced_performance_optimizer; // ACCIÓN 7.1: Performance optimization engine
pub mod advanced_profit_tracker; // ACCIÓN 7.2: Advanced profit tracking and analytics  
pub mod real_time_trading_dashboard; // ACCIÓN 7.3: Real-time trading dashboard

// Re-export del sistema integrado
pub use arbitrage_bot_phase45_integrated::ArbitrageBotPhase45Integrated;

pub use crate::config::Config;
pub use platform::SniperForgePlatform;
