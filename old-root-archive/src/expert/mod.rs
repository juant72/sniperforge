// ===== EXPERT MATHEMATICAL SYSTEM MODULES =====
// Módulos especializados para el sistema experto de arbitraje

pub mod calculations;     // Cálculos matemáticos AMM y profit
pub mod pool_validation; // Validación de pools y verificación on-chain  
pub mod price_feeds;     // Sistema de precios y feeds de datos
pub mod constants;       // Constantes militares optimizadas

// Re-export de funciones principales
pub use calculations::*;
pub use constants::*;
// TODO: Fix module exports later
// pub use pool_validation::{PoolData, PoolType, PoolValidator};
// pub use price_feeds::ExpertPriceFeeds;
