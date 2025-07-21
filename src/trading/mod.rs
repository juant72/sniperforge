//! Trading Engine Module
//!
//! This module contains the advanced trading systems and execution optimization
//! developed as part of DEV2 workstream.

pub mod execution_optimizer;
pub mod order_manager;
pub mod strategy_executor;

pub use execution_optimizer::*;
pub use order_manager::*;
pub use strategy_executor::*;
