//! Trading Engine Module
//! 
//! This module contains the advanced trading systems and execution optimization
//! developed as part of DEV2 workstream.

pub mod strategy_executor;
pub mod order_manager;
pub mod execution_optimizer;

pub use strategy_executor::*;
pub use order_manager::*;
pub use execution_optimizer::*;
