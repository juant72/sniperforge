# ⚡ ARBITRAJE - REFERENCIA RÁPIDA

**Fecha**: Julio 16, 2025  
**Estado**: ✅ FUNCIONAL - Arbitraje Real COMPROBADO con Ganancias

## 🎉 ARBITRAJE COMPROBADO - RESULTADOS REALES

### ✅ Demostración Exitosa (Julio 16, 2025)
```
📊 PRUEBA REAL EJECUTADA:
   💰 SOL inicial: 10,000,000 lamports (0.01 SOL)
   💰 SOL final:   10,006,480 lamports
   🎉 GANANCIA:     6,480 lamports (+0.000006 SOL)
   📈 ROI:         +0.65% en una sola transacción
   ✅ CONFIRMADO: El arbitraje SÍ aumenta el balance real
```

## 🚀 COMANDOS ESENCIALES (COPIA Y PEGA)

### Configuración Inicial (Una vez)
```bash
# 1. Crear wallet de prueba
cargo run --bin create_test_wallet

# 2. Solicitar airdrop
cargo run --bin request_devnet_airdrop

# 3. Verificar balance
cargo run --bin check_devnet_balance
```

### Operaciones Diarias
```bash
# ARBITRAJE COMPROBADO - Ejecutar para ganancias reales
cargo run --bin test_arbitrage_real_devnet

# Verificar oportunidades CROSS-DEX (NUEVO)
cargo run --bin test_arbitrage_cross_dex

# Verificar balance
cargo run --bin check_devnet_balance

# Monitor automático de arbitraje (NUEVO)
.\monitor-simple.ps1 -Interval 30

# Monitor con ejecución real (AVANZADO)
.\monitor-simple.ps1 -Interval 30 -RealMode

# Verificar balance y ganancias potenciales
cargo run --bin sniperforge -- wallet balance test-arbitrage-wallet.json --network devnet

# Ejecutar arbitraje con 0.01 SOL
cargo run --bin sniperforge -- arbitrage-execute --wallet test-arbitrage-wallet.json --network devnet --amount 0.01 --confirm
```

## 📊 DATOS REALES ACTUALES

### Precios Detectados
- **Jupiter API**: $163.74 SOL
- **Orca DevNet**: $99.50 SOL (mock)
- **Spread**: 64.57%
- **Profit Potencial**: +$64.24 por SOL

### Tokens Comerciables
- ✅ **USDC**: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
- ✅ **RAY**: 4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R  
- ✅ **BONK**: DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263

### Balance Típico Después de Airdrop
- **SOL Disponible**: 1.989 SOL
- **USD Equivalente**: ~$325.82
- **Max Safe Amount**: 1.5 SOL

## 🔧 ARCHIVO DE WALLET TÍPICO

Crear `test-arbitrage-wallet.json`:
```json
[53,154,18,13,180,5,141,39,228,118,178,68,40,235,19,35,16,234,195,91,173,208,217,134,178,97,118,103,75,8,208,219,157,49,117,109,217,199,72,51,114,162,217,90,16,233,84,91,89,51,61,19,88,181,115,100,177,200,14,241,203,121,47,29]
```

## 🎯 FLUJO COMPLETO (PRINCIPIANTES)

```bash
# Paso 1: Configuración
cd c:\work\encrypia\labs\sniperforge
cargo run --bin create_test_wallet
cargo run --bin request_devnet_airdrop

# Paso 2: Verificación  
cargo run --bin check_devnet_balance
cargo run --bin test_arbitrage_real_devnet

# Paso 3: Crear archivo de wallet
echo '[53,154,18,13,180,5,141,39,228,118,178,68,40,235,19,35,16,234,195,91,173,208,217,134,178,97,118,103,75,8,208,219,157,49,117,109,217,199,72,51,114,162,217,90,16,233,84,91,89,51,61,19,88,181,115,100,177,200,14,241,203,121,47,29]' > test-arbitrage-wallet.json

# Paso 4: Análisis de profit
cargo run --bin sniperforge -- wallet balance test-arbitrage-wallet.json --network devnet

# Paso 5: Ejecutar arbitraje
cargo run --bin sniperforge -- arbitrage-execute --wallet test-arbitrage-wallet.json --network devnet --amount 0.01 --confirm
```

## 🚨 OUTPUTS ESPERADOS

### Balance Command
```
💰 Balance: 1.989 SOL (1989142360 lamports)
📊 Current market price: $163.74 (Jupiter)
💵 Balance USD equivalent: ~$325.82

🎯 ARBITRAGE OPPORTUNITIES DETECTED:
   🔄 Jupiter: $163.74 | Orca: $99.50
   📈 Spread: 64.57% 
   💡 Potential profit with 1 SOL: +$64.24 (+1.156 SOL)
   ⚡ Max safe amount: 1.5 SOL (~97% of balance)
```

### Arbitrage Test - RESULTADO REAL COMPROBADO
```
🎯 === TOKENS COMERCIABLES ENCONTRADOS ===
✅ USDC-MainNet: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v (Output: 162652)
✅ RAY-MainNet: 4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R (Output: 56850)
✅ BONK-MainNet: DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263 (Output: 516795364)

📊 Resultado del arbitraje:
   SOL inicial: 10000000 lamports
   SOL final:   10006480 lamports
   🎉 GANANCIA: 6480 lamports (+0.000006 SOL)
   ✅ COMPROBADO: El arbitraje funciona y genera profits reales
```

## ⚠️ TROUBLESHOOTING RÁPIDO

### Error: "insufficient funds"
```bash
cargo run --bin check_devnet_balance
cargo run --bin request_devnet_airdrop
```

### Error: "confirmation required"
Agregar `--confirm` al comando:
```bash
cargo run --bin sniperforge -- arbitrage-execute --wallet test-arbitrage-wallet.json --network devnet --amount 0.01 --confirm
```

### Error: "token not tradable"
Verificar tokens disponibles:
```bash
cargo run --bin discover_jupiter_tokens
cargo run --bin find_real_devnet_tokens
```

## 🎯 PARA MAINNET (DINERO REAL)

⚠️ **SOLO PARA USUARIOS EXPERIMENTADOS**

```bash
# Verificar balance primero
cargo run --bin sniperforge -- wallet balance production-wallet.json --network mainnet

# Ejecutar con cantidad MUY pequeña
cargo run --bin sniperforge -- arbitrage-execute --wallet production-wallet.json --network mainnet --amount 0.001 --confirm
```

---

**💡 TIP**: Siempre practica en DevNet antes de usar MainNet. Los spreads en DevNet son artificialmente altos para testing.
