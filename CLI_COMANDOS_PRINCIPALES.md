# 🚀 COMANDOS CLI PRINCIPALES - SNIPERFORGE (VERIFICADOS)

## 🎯 COMANDO PRINCIPAL QUE FUNCIONA

```powershell
# COMANDO PRINCIPAL para swap real en DevNet
cargo run --bin sniperforge -- test swap-real --network devnet --confirm

# COMANDO PRINCIPAL para swap real en MainNet (DINERO REAL!)
cargo run --bin sniperforge -- test swap-real --network mainnet --confirm
```

## ✅ COMANDOS VERIFICADOS QUE FUNCIONAN

### Wallet Management
```powershell
# Verificar balances de wallet (usa .env)
cargo run --bin sniperforge -- wallet balance

# Solicitar airdrop en DevNet
cargo run --bin sniperforge -- wallet airdrop

# Generar nueva wallet
cargo run --bin sniperforge -- wallet generate
```

### Trading y Swaps REALES
```powershell
# Simulación de swap (SEGURO)
cargo run --bin sniperforge -- test swap-real --network devnet

# SWAP REAL en DevNet (EJECUTA TRANSACCIONES REALES!)
cargo run --bin sniperforge -- test swap-real --network devnet --confirm

# SWAP REAL en MainNet (DINERO REAL!)
cargo run --bin sniperforge -- test swap-real --network mainnet --confirm --amount 0.001
```

### Testing y Validación
```powershell
# Test completo del sistema
cargo run --bin sniperforge -- test all

# Test básico de conectividad
cargo run --bin sniperforge -- test basic

# Test de Jupiter API
cargo run --bin sniperforge -- test jupiter

# Test de Solana RPC
cargo run --bin sniperforge -- test solana
```

## 📊 WORKFLOW PASO A PASO (COMANDOS REALES)

```powershell
# 1. Construir proyecto
cargo build --release

# 2. Verificar balances iniciales
cargo run --bin sniperforge -- wallet balance

# 3. Solicitar SOL si es necesario
cargo run --bin sniperforge -- wallet airdrop

# 4. EJECUTAR SWAP REAL (COMANDO PRINCIPAL)
cargo run --bin sniperforge -- test swap-real --network devnet --confirm

# 5. Verificar cambios en balances
cargo run --bin sniperforge -- wallet balance
```

## ⚠️ DIFERENCIA ENTRE SIMULACIÓN Y REAL

### Simulación (SEGURO)
```powershell
# Sin --confirm = Solo simulación
cargo run --bin sniperforge -- test swap-real --network devnet
```
**Resultado**: Muestra precios y quotes, NO ejecuta transacciones

### REAL (EJECUTA TRANSACCIONES)
```powershell
# Con --confirm = Transacciones REALES
cargo run --bin sniperforge -- test swap-real --network devnet --confirm
```
**Resultado**: Ejecuta transacciones reales, paga fees reales, cambia balances

## 🚀 MAINNET (DINERO REAL)

```powershell
# ADVERTENCIA: Estos comandos usan DINERO REAL
cargo run --bin sniperforge -- test swap-real --network mainnet --confirm --amount 0.001
```

## ❌ COMANDOS QUE NO FUNCIONAN TODAVÍA

```powershell
# Estos comandos están en desarrollo:
# cargo run --bin sniperforge -- arbitrage-scan --network devnet
# cargo run --bin sniperforge -- test cache-free-trading --network devnet
# cargo run --bin sniperforge -- multi-strategy-trading --network devnet
```

## 📋 AYUDA Y DOCUMENTACIÓN

```powershell
# Ver todos los comandos disponibles
cargo run --bin sniperforge -- --help

# Ayuda para subcomandos
cargo run --bin sniperforge -- wallet --help
cargo run --bin sniperforge -- test --help

# Ayuda específica para swap real
cargo run --bin sniperforge -- test swap-real --help
```

## ✅ RESULTADOS ESPERADOS

### Con --confirm (REAL):
- ✅ Transaction signatures reales
- ✅ Balances de tokens cambiados
- ✅ Balance SOL reducido por fees reales
- ✅ "Transaction completed successfully"

### Sin --confirm (Simulación):
- ✅ Quotes de Jupiter API
- ✅ Precios actuales mostrados
- ✅ "Simulation completed"
- ❌ NO hay transacciones reales

## 🌐 VERIFICACIÓN ON-CHAIN

Todas las transacciones reales son verificables en:
- DevNet: https://explorer.solana.com/?cluster=devnet
- MainNet: https://explorer.solana.com/

---

**🎯 COMANDO PRINCIPAL RECOMENDADO:**
```powershell
cargo run --bin sniperforge -- test swap-real --network devnet --confirm
```

**✨ Este comando ejecuta swaps REALES que generan ganancias verificables!**
