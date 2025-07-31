# 🤝 COORDINATION & INTEGRATION GUIDE
**Purpose**: Ensure Dev1 and Dev2 work independently while maintaining compatibility
**Last Updated**: July 2, 2025

---

## 🎯 WORK DISTRIBUTION SUMMARY

| **Dev1: Market Intelligence** | **Dev2: Trading Engine** |
|-------------------------------|---------------------------|
| 📈 Market analysis & opportunities | ⚡ Trading execution & strategies |
| 📊 Portfolio analytics & risk | 🛡️ Production infrastructure |
| 🔍 Real-time market scanning | 🚀 Performance optimization |
| **Independent**: Uses existing APIs | **Independent**: Uses existing wallet system |

---

## 🔄 SHARED DEPENDENCIES (Minimal)

### Common Data Structures
Both developers will use existing structures without modification:

```rust
// Already implemented - no changes needed
pub struct TokenPrice { /* existing */ }
pub struct WalletBalance { /* existing */ }
pub struct ProfessionalPortfolioStatus { /* existing */ }

// Shared configuration
pub struct Config { /* existing */ }
```

### Shared Modules (Read-Only)
```rust
// Both devs use these modules as-is
src/portfolio/wallet_scanner.rs     // ✅ Complete - read-only
src/portfolio/price_feed.rs         // ✅ Complete - read-only
src/shared/jupiter.rs               // ✅ Complete - read-only
src/config.rs                       // ✅ Complete - read-only
```

---

## 🚧 WORK BOUNDARIES

### ✅ Dev1 Can Work On (No Conflicts)
```
src/market/                 (NEW - create entire directory)
├── dex_analyzer.rs
├── opportunity_scanner.rs
├── market_metrics.rs
└── price_aggregator.rs

src/portfolio/              (EXTEND existing modules)
├── performance_tracker.rs  (NEW)
├── risk_calculator.rs      (NEW)
└── rebalancer.rs           (NEW)

src/analytics/              (NEW - create entire directory)
├── correlation.rs
├── volatility.rs
└── trend_analyzer.rs
```

### ✅ Dev2 Can Work On (No Conflicts)
```
src/trading/                (NEW - create entire directory)
├── strategy_executor.rs
├── order_manager.rs
├── execution_optimizer.rs
└── slippage_optimizer.rs

src/infrastructure/         (NEW - create entire directory)
├── monitoring.rs
├── alerting.rs
├── backup_manager.rs
└── performance_optimizer.rs

configs/                    (NEW - create configuration files)
├── strategies/
├── monitoring/
└── backup/
```

---

## 🔒 CONFLICT AVOIDANCE RULES

### 1. **File-Level Separation**
- **Dev1**: Only creates/modifies files in `src/market/`, `src/analytics/`, and NEW files in `src/portfolio/`
- **Dev2**: Only creates/modifies files in `src/trading/`, `src/infrastructure/`, and `configs/`
- **Shared**: Both can read from existing modules but NOT modify them

### 2. **CLI Command Separation**
```bash
# Dev1 CLI commands (market & analytics)
sniperforge market-scan
sniperforge opportunities
sniperforge metrics
sniperforge performance
sniperforge risk-analysis
sniperforge rebalance

# Dev2 CLI commands (trading & infrastructure)
sniperforge strategy-run
sniperforge order-create
sniperforge execution-optimize
sniperforge monitor
sniperforge alerts
sniperforge backup
sniperforge performance-stats
```

### 3. **Git Branch Strategy**
```bash
# Dev1 branch
git checkout -b feature/market-intelligence
git push -u origin feature/market-intelligence

# Dev2 branch
git checkout -b feature/trading-engine
git push -u origin feature/trading-engine

# Integration branch (weekly merges)
git checkout -b integration/week-1
```

---

## 📅 COORDINATION SCHEDULE

### **Monday**: Sprint Planning (30 minutes)
- Review previous week's progress
- Identify any integration points
- Plan week's work without conflicts

### **Wednesday**: Mid-week Sync (15 minutes)
- Quick progress update
- Share any data structure needs
- Resolve any emerging conflicts

### **Friday**: Demo & Integration (45 minutes)
- Demo completed features
- Test CLI commands together
- Plan weekend integration if needed

---

## 🔧 INTEGRATION POINTS

### Week 2: First Integration
```rust
// Dev1 exposes market data
pub struct MarketData {
    pub liquidity_score: f64,
    pub arbitrage_opportunities: Vec<ArbitrageOpp>,
    pub risk_metrics: RiskMetrics,
}

// Dev2 consumes for trading decisions
impl StrategyExecutor {
    pub fn use_market_data(&self, data: &MarketData) -> TradingDecision;
}
```

### Week 4: Full Integration
```rust
// Combined CLI command
sniperforge auto-trade --strategy dca --use-market-analysis --risk-limit 0.05
```

---

## 🧪 TESTING COORDINATION

### Individual Testing
- **Dev1**: Test with market data APIs (DexScreener, CoinGecko)
- **Dev2**: Test with trading APIs (Jupiter, Solana RPC)
- **Both**: Use same test wallet for consistency

### Integration Testing
```bash
# Shared test wallet (same for both devs)
export TEST_WALLET="DzrRWVKNjGyns9cKvp3VtJr2qqwCNGcnJ9dhYF31f1YL"

# Dev1 tests
cargo test market::tests --features mainnet
cargo test analytics::tests --features mainnet

# Dev2 tests
cargo test trading::tests --features devnet
cargo test infrastructure::tests --features mainnet

# Integration tests (both together)
cargo test integration::tests --features mainnet
```

---

## 📦 DELIVERY COORDINATION

### Week 2 Deliveries
- **Dev1**: Market analysis modules + 3 CLI commands
- **Dev2**: Basic trading strategies + monitoring
- **Integration**: Combined CLI help and basic data sharing

### Week 4 Deliveries
- **Dev1**: Full analytics suite + portfolio optimization
- **Dev2**: Production infrastructure + advanced trading
- **Integration**: Full-featured trading platform with market intelligence

---

## 🚨 CONFLICT RESOLUTION

### If File Conflicts Occur:
1. **Stop work** on conflicting file
2. **Communicate immediately** via team chat
3. **Decide ownership** based on primary responsibility
4. **Alternative implementation** for the other developer

### If API Changes Are Needed:
1. **Propose change** in team chat first
2. **Get agreement** from both developers
3. **Make change together** in a brief pair session
4. **Update both branches** simultaneously

---

## 📋 SUCCESS CRITERIA

### Independent Success
- [ ] Dev1: Market analysis working without Dev2's modules
- [ ] Dev2: Trading engine working without Dev1's modules
- [ ] Both: All tests pass independently

### Integration Success
- [ ] All CLI commands work together
- [ ] No merge conflicts in main integration
- [ ] Combined system demonstrates advanced trading with market intelligence

This plan ensures maximum independence while maintaining integration compatibility.
