# 🚀 SniperForge CLI - Quick Reference

## Essential Commands (Copy & Paste Ready)

### 🔧 Setup (First Time)
```powershell
# Generate DevNet wallet
cargo run --bin sniperforge -- wallet generate --output test-wallet.json

# Fund with test SOL
cargo run --bin sniperforge -- wallet airdrop
```

### 💰 Real Arbitrage Trading
```powershell
# 1. Simulation (safe test)
cargo run --bin sniperforge -- test swap-real --network devnet

# 2. REAL DevNet trading (test SOL)
cargo run --bin sniperforge -- test swap-real --network devnet --confirm

# 3. REAL Mainnet trading (REAL MONEY!)
cargo run --bin sniperforge -- test swap-real --network mainnet --confirm
```

### 📊 Monitoring
```powershell
# Check wallet balance
cargo run --bin sniperforge -- wallet balance test-wallet.json

# System status
cargo run --bin sniperforge -- status

# Full test suite
cargo run --bin sniperforge -- test all
```

## 🎯 Key Parameters

| Parameter | Description |
|-----------|-------------|
| `--network devnet` | Safe testing with test SOL |
| `--network mainnet` | Real money trading |
| `--confirm` | Execute real transaction |
| `--amount 0.001` | Custom SOL amount |
| `--wallet my-wallet.json` | Custom wallet file |

## ⚡ One-Liner Commands

```powershell
# Quick DevNet test
cargo run --bin sniperforge -- test swap-real --network devnet --confirm

# Quick balance check
cargo run --bin sniperforge -- wallet balance test-wallet.json

# Get help for any command
cargo run --bin sniperforge -- test swap-real --help
```

## 🛡️ Safety Levels

| Command | Risk Level | Description |
|---------|------------|-------------|
| Without `--confirm` | ✅ **None** | Simulation only |
| `--network devnet --confirm` | ⚠️ **Low** | Test SOL only |
| `--network mainnet --confirm` | 🚨 **High** | Real money! |

---

**Status**: ✅ Production Ready  
**Guide**: See `CLI_PRODUCTION_GUIDE.md` for complete documentation
