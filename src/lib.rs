#![allow(dead_code)]
#![allow(unused_imports)]

pub mod config;
pub mod platform;
pub mod bots;
pub mod shared;
pub mod types;
pub mod solana_testing;
pub mod simple_testing;
pub mod jupiter_speed_test;
pub mod websocket_rpc_test;
pub mod ultimate_rpc_test;
pub mod strategies;
pub mod analysis;
pub mod ml;  // Phase 6B ML module
pub mod portfolio;  // Phase 6C Portfolio Management
pub mod sprint_1_demo;  // Sprint 1 functionality demo

// Add test module for alternative APIs
pub mod alternative_apis_test;

pub use crate::config::Config;
pub use platform::SniperForgePlatform;
