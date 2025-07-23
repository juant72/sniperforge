# SniperForge CLI - Production Ready Guide

## 🚀 Real Arbitrage Bot for Solana DevNet/Mainnet

This guide provides **production-ready CLI commands** for running the SniperForge arbitrage bot with **real on-chain transactions**, custom tokens, and premium RPC endpoints.

> **⚠️ IMPORTANT**: All commands execute REAL blockchain transactions. DevNet uses test SOL (safe), Mainnet uses REAL SOL with monetary value.

---

## 📋 Quick Start Workflow

### 1. Check CLI Status
```powershell
cargo run --bin sniperforge -- --help
```

### 2. Generate DevNet Wallet (First Time)
```powershell
cargo run --bin sniperforge -- wallet generate --output test-wallet.json
```

### 3. Fund DevNet Wallet
```powershell
cargo run --bin sniperforge -- wallet airdrop
```

### 4. Check Wallet Balance
```powershell
cargo run --bin sniperforge -- wallet balance test-wallet.json
```

### 5. Test Arbitrage (Simulation - Safe)
```powershell
cargo run --bin sniperforge -- test swap-real --wallet test-wallet.json --network devnet
```

### 6. **Execute Real Arbitrage** 🎯
```powershell
cargo run --bin sniperforge -- test swap-real --wallet test-wallet.json --network devnet --confirm
```

---

## 🏆 Main Commands

### Core Arbitrage Execution
| Command | Description | Safety |
|---------|-------------|---------|
| `cargo run --bin sniperforge -- test swap-real --network devnet` | **Simulation** (no real transaction) | ✅ Safe |
| `cargo run --bin sniperforge -- test swap-real --network devnet --confirm` | **Real DevNet arbitrage** | ⚠️ Test SOL |
| `cargo run --bin sniperforge -- test swap-real --network mainnet --confirm` | **Real Mainnet arbitrage** | 🚨 REAL MONEY |

### Wallet Management
| Command | Description |
|---------|-------------|
| `cargo run --bin sniperforge -- wallet generate --output my-wallet.json` | Create new wallet |
| `cargo run --bin sniperforge -- wallet balance test-wallet.json` | Check wallet balance |
| `cargo run --bin sniperforge -- wallet airdrop` | Get DevNet test SOL |

### Testing & Validation
| Command | Description |
|---------|-------------|
| `cargo run --bin sniperforge -- test all` | Run comprehensive tests |
| `cargo run --bin sniperforge -- test solana` | Test Solana RPC connectivity |
| `cargo run --bin sniperforge -- test jupiter` | Test Jupiter API integration |

---

## 🔧 Advanced Options

### Custom Amounts
```powershell
# DevNet with custom amount
cargo run --bin sniperforge -- test swap-real --wallet test-wallet.json --network devnet --amount 0.001 --confirm

# Mainnet with minimal amount (REAL MONEY!)
cargo run --bin sniperforge -- test swap-real --wallet mainnet-wallet.json --network mainnet --amount 0.0001 --confirm
```

### Different Wallets
```powershell
# DevNet wallet
cargo run --bin sniperforge -- test swap-real --wallet devnet-wallet.json --network devnet --confirm

# Mainnet wallet (use with caution!)
cargo run --bin sniperforge -- test swap-real --wallet mainnet-validation-wallet.json --network mainnet --confirm
```

---

## 🌐 Network Configuration

### DevNet (Recommended for Testing)
- **Network**: `devnet`
- **Currency**: Test SOL (no real value)
- **Safety**: ✅ Safe for testing
- **RPC**: Solana DevNet RPC

### Mainnet (Production)
- **Network**: `mainnet`
- **Currency**: Real SOL (monetary value)
- **Safety**: 🚨 REAL MONEY - Use extreme caution
- **RPC**: Premium RPC endpoints

---

## 📊 Expected Results

### Successful DevNet Arbitrage
```
🚀 SPRINT 1: Real swap execution test
💰 Profit Detection: +0.000123 SOL
📈 Route: SOL -> USDC -> SOL via Jupiter
✅ Transaction: 5X7Tn...Kq9Z confirmed
🎯 Arbitrage successful: +0.000123 SOL profit
```

### Simulation Mode (Safe Testing)
```
🧪 SIMULATION MODE - No real transaction sent
💰 Detected profit: +0.000089 SOL
📊 Route would be: SOL -> USDC -> SOL
✅ Simulation successful
```

---

## 🛡️ Safety Features

### Built-in Protections
- **Simulation Mode**: Default behavior without `--confirm`
- **Network Validation**: Prevents accidental mainnet execution
- **Amount Limits**: Default micro-amounts for testing
- **Wallet Validation**: Checks wallet file existence and format

### Risk Levels
| Level | Command Pattern | Risk |
|-------|----------------|------|
| **None** | Without `--confirm` | No transaction sent |
| **Low** | DevNet + `--confirm` | Test SOL only |
| **High** | Mainnet + `--confirm` | Real money at risk |

---

## 🔗 Integration Features

### Jupiter API Integration
- ✅ Real-time pricing data
- ✅ Actual swap routes
- ✅ Live transaction execution
- ✅ Production-ready endpoints

### RPC Connectivity
- ✅ Premium RPC support
- ✅ Mainnet and DevNet
- ✅ Automatic failover
- ✅ Rate limiting compliance

### Custom Token Support
- ✅ Any SPL token
- ✅ Dynamic token discovery
- ✅ Real-time pair analysis
- ✅ Profit optimization

---

## 📚 Additional Resources

### Documentation Files
- `CLI_COMANDOS_PRINCIPALES.md` - Essential commands
- `COMANDOS_ESENCIALES.ps1` - PowerShell script shortcuts
- `VALIDACION_REAL_FINAL_REPORT.md` - Real execution validation
- `DEVNET_SUCCESS_REPORT.md` - DevNet testing results

### Quick Scripts
- `run-main-command.ps1` - Main arbitrage command
- `quick-start-arbitrage.ps1` - Full setup workflow
- `setup-devnet-tokens.ps1` - DevNet environment setup

---

## ⚡ Performance Notes

- **Compilation**: ~1-10 seconds (cached builds)
- **Execution**: ~5-15 seconds per arbitrage scan
- **Network Latency**: Depends on RPC endpoint quality
- **Success Rate**: Varies by market conditions

---

## 🚨 Important Warnings

1. **Always test on DevNet first** before using mainnet
2. **Use small amounts** when testing on mainnet
3. **Verify wallet file** before executing transactions
4. **Check network parameter** to avoid wrong network
5. **Monitor gas fees** on mainnet for profitability

---

## ✅ Production Checklist

- [ ] CLI compiles without errors
- [ ] DevNet wallet created and funded
- [ ] Simulation mode tested successfully
- [ ] Real DevNet arbitrage executed
- [ ] Wallet balance verified after execution
- [ ] Network parameters understood
- [ ] Safety features tested

**Ready for production arbitrage trading!** 🎯

---

*Last Updated: Sprint 1 - Real Transaction Capability*
*Status: ✅ Production Ready*
