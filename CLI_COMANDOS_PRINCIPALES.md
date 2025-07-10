# 🚀 COMANDOS CLI PRINCIPALES - SNIPERFORGE

## 🎯 COMANDO PRINCIPAL OBLIGATORIO

```powershell
# COMANDO PRINCIPAL para arbitraje real en DevNet
cargo run --bin sniperforge test cache-free-trading --network devnet

# COMANDO PRINCIPAL para arbitraje real en MainNet (DINERO REAL)
cargo run --bin sniperforge test cache-free-trading --network mainnet
```

## ⚠️ IMPORTANTE: PARÁMETRO --network OBLIGATORIO

```powershell
# ❌ FALLA: cargo run --bin sniperforge test cache-free-trading
# Error: "Network selection is required. Use --network devnet or --network mainnet"

# ✅ CORRECTO: 
cargo run --bin sniperforge test cache-free-trading --network devnet
cargo run --bin sniperforge test cache-free-trading --network mainnet
```

## 💰 COMANDOS CLI PRINCIPALES

### Wallet y Balances
```powershell
# Verificar balances de wallet
cargo run --bin sniperforge wallet balance --network devnet

# Solicitar airdrop en DevNet
cargo run --bin sniperforge wallet airdrop --network devnet

# Generar nueva wallet
cargo run --bin sniperforge wallet generate

# Exportar wallet actual
cargo run --bin sniperforge wallet export
```

### Trading y Arbitraje
```powershell
# Cache-Free Trading Engine (PRINCIPAL)
cargo run --bin sniperforge test cache-free-trading --network devnet

# Arbitrage scan en tiempo real
cargo run --bin sniperforge arbitrage-scan --network devnet

# Multi-Strategy Trading
cargo run --bin sniperforge multi-strategy-trading --network devnet

# Interactive Trading Mode
cargo run --bin sniperforge interactive --network devnet
```

### Testing y Validación
```powershell
# Test completo del sistema
cargo run --bin sniperforge test all --network devnet

# Test básico de conectividad
cargo run --bin sniperforge test basic --network devnet

# Test de Jupiter API
cargo run --bin sniperforge test jupiter --network devnet

# Test de trading real
cargo run --bin sniperforge test trade --network devnet

# Test de swap real
cargo run --bin sniperforge test swap-real --network devnet
```

## 📊 WORKFLOW PASO A PASO

```powershell
# 1. Construir proyecto
cargo build --release

# 2. Verificar balances iniciales
cargo run --bin sniperforge wallet balance --network devnet

# 3. Ejecutar cache-free trading (COMANDO PRINCIPAL)
cargo run --bin sniperforge test cache-free-trading --network devnet

# 4. Verificar ganancias reales
cargo run --bin sniperforge wallet balance --network devnet
```

## 🎯 COMANDOS AVANZADOS

### Pattern Analysis y ML
```powershell
# Pattern Analysis
cargo run --bin sniperforge pattern-analysis --network devnet

# Strategy Backtest
cargo run --bin sniperforge strategy-backtest --strategy arbitrage --network devnet

# ML Pattern Recognition
cargo run --bin sniperforge ml analyze-patterns --network devnet

# ML Trend Prediction
cargo run --bin sniperforge ml predict-trend --network devnet
```

### Arbitrage con Parámetros
```powershell
# Arbitrage scan con profit mínimo ($10)
cargo run --bin sniperforge arbitrage-scan --network devnet --min-profit 10.0

# Arbitrage scan con duración específica (5 minutos)
cargo run --bin sniperforge arbitrage-scan --network devnet --duration 300

# Arbitrage scan con exportación de resultados
cargo run --bin sniperforge arbitrage-scan --network devnet --export arbitrage_results.json
```

## 🚀 MAINNET (DINERO REAL)

```powershell
# Cache-Free Trading en MainNet
cargo run --bin sniperforge test cache-free-trading --network mainnet

# Arbitrage scan en MainNet
cargo run --bin sniperforge arbitrage-scan --network mainnet

# Verificar balances en MainNet
cargo run --bin sniperforge wallet balance --network mainnet
```

## 📋 AYUDA Y DOCUMENTACIÓN

```powershell
# Ver todos los comandos disponibles
cargo run --bin sniperforge --help

# Ayuda para subcomandos
cargo run --bin sniperforge test --help
cargo run --bin sniperforge wallet --help
cargo run --bin sniperforge arbitrage-scan --help

# Ayuda específica para cache-free trading
cargo run --bin sniperforge test cache-free-trading --help
```

## ✅ RESULTADOS ESPERADOS

Después de ejecutar los comandos CLI, deberías ver:
- ✅ "Network selection is required" si olvidas --network
- ✅ Transaction signatures reales en DevNet
- ✅ Balances de tokens incrementados (ganancias reales)
- ✅ Balance SOL reducido por fees reales
- ✅ Mensajes "Trading completed successfully"

## 🌐 VERIFICACIÓN ON-CHAIN

Todas las transacciones son verificables en:
- DevNet: https://explorer.solana.com/?cluster=devnet
- MainNet: https://explorer.solana.com/
- SolanaFM: https://solana.fm/

---

**🎯 COMANDO PRINCIPAL RECOMENDADO:**
```powershell
cargo run --bin sniperforge test cache-free-trading --network devnet
```

**✨ Este comando ejecuta el Cache-Free Trading Engine que genera ganancias reales!**
