# 📚 SNIPERFORGE CLI - GUÍA COMPLETA DE COMANDOS

**Versión**: 0.1.0  
**Fecha**: Junio 27, 2025  
**Estado**: Sprint 1 Completado ✅

## ℹ️ **AYUDA DISPONIBLE**

**¡Todas las opciones `--help` ahora funcionan correctamente!**

```bash
# Ayuda principal
sniperforge --help

# Ayuda de comandos
sniperforge [COMANDO] --help

# Ayuda de subcomandos  
sniperforge [COMANDO] [SUBCOMANDO] --help

# Ejemplos
sniperforge wallet --help
sniperforge test --help
sniperforge test swap-real --help
sniperforge ml --help
```

---

## 🚀 COMANDOS PRINCIPALES

### `start` - Iniciar la Plataforma
```bash
cargo run --bin sniperforge start [OPCIONES]
```

**Descripción**: Inicia la plataforma SniperForge o bots específicos

**Opciones**:
- `-b, --bot <BOT_TYPE>` - Bot específico a iniciar (puede repetirse)
- `--devnet` - Usar configuración DevNet para testing

**Ejemplos**:
```bash
# Iniciar plataforma completa
cargo run --bin sniperforge start

# Iniciar con DevNet
cargo run --bin sniperforge start --devnet

# Iniciar bot específico
cargo run --bin sniperforge start -b lp-sniper
```

---

### `status` - Estado de la Plataforma
```bash
cargo run --bin sniperforge status
```

**Descripción**: Muestra el estado actual de la plataforma y todos los componentes

---

### `config` - Mostrar Configuración
```bash
cargo run --bin sniperforge config
```

**Descripción**: Muestra la configuración actual del sistema

---

## 💰 COMANDOS WALLET

### `wallet balance` - Verificar Balance
```bash
cargo run --bin sniperforge wallet balance [WALLET_FILE]
```

**Descripción**: Verifica el balance de una wallet específica o la default

**Parámetros**:
- `WALLET_FILE` - Archivo JSON de la wallet (opcional)

**Ejemplos**:
```bash
# Verificar wallet específica
cargo run --bin sniperforge wallet balance test-wallet.json

# Verificar wallet Mainnet
cargo run --bin sniperforge wallet balance mainnet-validation-wallet.json
```

### `wallet airdrop` - Solicitar Airdrop
```bash
cargo run --bin sniperforge wallet airdrop
```

**Descripción**: Solicita airdrop de SOL en DevNet

### `wallet generate` - Generar Nueva Wallet
```bash
cargo run --bin sniperforge wallet generate [OPCIONES]
```

**Descripción**: Genera una nueva wallet para DevNet

**Opciones**:
- `-o, --output <FILE>` - Archivo de salida (default: test-wallet-new.json)

**Ejemplos**:
```bash
# Generar con nombre default
cargo run --bin sniperforge wallet generate

# Generar con nombre específico
cargo run --bin sniperforge wallet generate --output mi-wallet.json
```

---

## 🧪 COMANDOS TEST

### `test all` - Ejecutar Todos los Tests
```bash
cargo run --bin sniperforge test all
```

**Descripción**: Ejecuta todos los tests de la suite completa

### `test basic` - Tests Básicos
```bash
cargo run --bin sniperforge test basic
```

**Descripción**: Tests básicos de conectividad

### `test solana` - Tests Solana RPC
```bash
cargo run --bin sniperforge test solana
```

**Descripción**: Tests de conectividad y llamadas RPC de Solana

### `test jupiter` - Tests Jupiter API
```bash
cargo run --bin sniperforge test jupiter
```

**Descripción**: Tests de integración con Jupiter API

### `test wallet` - Tests de Wallet
```bash
cargo run --bin sniperforge test wallet
```

**Descripción**: Tests de funcionalidad de wallets

### `test websocket` - Tests WebSocket
```bash
cargo run --bin sniperforge test websocket
```

**Descripción**: Tests de conectividad y suscripciones WebSocket

### `test trade` - Tests de Trading (Simulación)
```bash
cargo run --bin sniperforge test trade
```

**Descripción**: Tests de ejecución de trades en modo simulación

### `test swap-real` - 🚀 SWAP REAL (SPRINT 1)
```bash
cargo run --bin sniperforge test swap-real [OPCIONES]
```

**Descripción**: **COMANDO PRINCIPAL SPRINT 1** - Ejecuta swaps reales en blockchain

**Opciones**:
- `-a, --amount <SOL>` - Cantidad de SOL a intercambiar (default: 0.00001)
- `-w, --wallet <FILE>` - Archivo de wallet para ejecución real
- `--network <NET>` - Red a usar: devnet o mainnet (default: devnet)
- `--confirm` - Confirmar envío de transacción REAL

**⚠️ IMPORTANTE**: 
- **DevNet**: Usa SOL de prueba
- **Mainnet**: Usa SOL REAL con valor monetario

**Ejemplos**:
```bash
# Simulación DevNet (sin --confirm)
cargo run --bin sniperforge test swap-real --wallet test-wallet.json

# Swap real DevNet
cargo run --bin sniperforge test swap-real --wallet test-wallet.json --confirm

# Swap real Mainnet (¡DINERO REAL!)
cargo run --bin sniperforge test swap-real --wallet mainnet-validation-wallet.json --network mainnet --amount 0.001 --confirm
```

### `test integration` - Tests de Integración
```bash
cargo run --bin sniperforge test integration
```

**Descripción**: Tests del flujo completo de integración

### `test performance` - Tests de Performance
```bash
cargo run --bin sniperforge test performance
```

**Descripción**: Tests de performance y latencia

---

## 🖥️ MODO INTERACTIVO

### `interactive` - Modo Monitoring Interactivo
```bash
cargo run --bin sniperforge interactive
```

**Descripción**: Inicia el modo de monitoreo interactivo de la plataforma

---

## 📊 COMANDOS AVANZADOS (FASE 6A)

### `multi-strategy-trading` - Trading Multi-Estrategia
```bash
cargo run --bin sniperforge multi-strategy-trading [OPCIONES]
```

**Descripción**: Ejecuta múltiples estrategias de trading concurrentemente

**Opciones**:
- `-s, --strategies <LIST>` - Lista de estrategias: trend,momentum,mean-reversion,arbitrage (default: trend,momentum)
- `-d, --duration <SECONDS>` - Duración de sesión en segundos (default: 300)
- `-c, --capital <USD>` - Capital por estrategia en USD (default: 5000)
- `-t, --timeframes <LIST>` - Timeframes de análisis: 1m,5m,15m,1h (default: 1m,5m)

**Ejemplos**:
```bash
# Trading básico multi-estrategia
cargo run --bin sniperforge multi-strategy-trading

# Trading personalizado
cargo run --bin sniperforge multi-strategy-trading -s trend,arbitrage -d 600 -c 10000
```

### `strategy-backtest` - Backtest de Estrategias
```bash
cargo run --bin sniperforge strategy-backtest [OPCIONES]
```

**Descripción**: Realiza backtest de estrategias individuales o combinadas

**Opciones**:
- `-s, --strategy <STRATEGY>` - Estrategia: trend,momentum,mean-reversion,arbitrage,all (default: trend)
- `-p, --period <DAYS>` - Período histórico en días (default: 7)
- `-c, --capital <USD>` - Capital inicial para backtest (default: 10000)
- `-e, --export <FILE>` - Exportar resultados a archivo JSON

**Ejemplos**:
```bash
# Backtest básico
cargo run --bin sniperforge strategy-backtest

# Backtest personalizado
cargo run --bin sniperforge strategy-backtest -s momentum -p 30 -c 50000 -e results.json
```

### `pattern-analysis` - Análisis de Patrones
```bash
cargo run --bin sniperforge pattern-analysis [OPCIONES]
```

**Descripción**: Analiza patrones y tendencias del mercado

**Opciones**:
- `-p, --pattern <PATTERN>` - Tipo de patrón: support-resistance,breakout,reversal,all (default: all)
- `-t, --timeframe <TIMEFRAME>` - Timeframe: 1m,5m,15m,1h (default: 5m)
- `-d, --duration <SECONDS>` - Duración de análisis en segundos (default: 180)
- `-e, --export <FILE>` - Exportar análisis a archivo JSON

### `arbitrage-scan` - Escaneo de Arbitraje
```bash
cargo run --bin sniperforge arbitrage-scan [OPCIONES]
```

**Descripción**: Escanea oportunidades de arbitraje entre DEXs

**Opciones**:
- `-m, --min-profit <USD>` - Umbral mínimo de ganancia en USD (default: 5.0)
- `--max-slippage <PCT>` - Tolerancia máxima de slippage (default: 0.5)
- `-d, --duration <SECONDS>` - Duración de escaneo en segundos (default: 120)
- `-e, --export <FILE>` - Exportar oportunidades a archivo JSON

---

## 🤖 COMANDOS MACHINE LEARNING (FASE 6B)

### `ml analyze-patterns` - Análisis de Patrones ML
```bash
cargo run --bin sniperforge ml analyze-patterns [OPCIONES]
```

**Opciones**:
- `-s, --symbol <TOKEN>` - Símbolo de token (default: SOL/USDC)
- `-t, --timeframe <MINUTES>` - Timeframe en minutos (default: 5)
- `-c, --confidence <THRESHOLD>` - Umbral de confianza 0.0-1.0 (default: 0.8)

### `ml predict-trend` - Predicción de Tendencias
```bash
cargo run --bin sniperforge ml predict-trend [OPCIONES]
```

**Opciones**:
- `-s, --symbol <TOKEN>` - Símbolo a predecir (default: SOL/USDC)
- `-h, --horizon <MINUTES>` - Horizonte de predicción en minutos (default: 15)
- `-c, --confidence <THRESHOLD>` - Umbral mínimo de confianza (default: 0.7)

### `ml optimize-strategy` - Optimización de Estrategias
```bash
cargo run --bin sniperforge ml optimize-strategy [OPCIONES]
```

**Opciones**:
- `-s, --strategy <STRATEGY>` - Estrategia: trend,momentum,mean-reversion (default: trend)
- `-g, --generations <COUNT>` - Número de generaciones de optimización (default: 50)
- `-p, --population <SIZE>` - Tamaño de población para algoritmo genético (default: 20)

### `ml backtest-optimized` - Backtest Optimizado
```bash
cargo run --bin sniperforge ml backtest-optimized [OPCIONES]
```

**Opciones**:
- `-s, --strategy <STRATEGY>` - Estrategia a probar (default: trend)
- `-p, --period <DAYS>` - Período de backtest en días (default: 30)
- `-c, --confidence <THRESHOLD>` - Confianza mínima para trades (default: 0.7)

### `ml assess-risk` - Evaluación de Riesgo
```bash
cargo run --bin sniperforge ml assess-risk [OPCIONES]
```

**Opciones**:
- `-w, --window <HOURS>` - Ventana de análisis en horas (default: 24)
- `-p, --portfolio <TOKENS>` - Lista de tokens del portfolio (default: SOL,USDC)

### `ml market-regime` - Detección de Régimen de Mercado
```bash
cargo run --bin sniperforge ml market-regime [OPCIONES]
```

**Opciones**:
- `-c, --confidence <THRESHOLD>` - Confianza mínima para clasificación (default: 0.9)
- `-l, --lookback <DAYS>` - Período de datos históricos (default: 14)

### `ml predict-timing` - Predicción de Timing
```bash
cargo run --bin sniperforge ml predict-timing [OPCIONES]
```

**Opciones**:
- `-s, --symbol <TOKEN>` - Símbolo para predicción (default: SOL/USDC)
- `-t, --size <AMOUNT>` - Tamaño objetivo del trade (default: 100)
- `-d, --direction <BUY_OR_SELL>` - Dirección del trade: buy o sell (default: buy)

### `ml optimize-execution` - Optimización de Ejecución
```bash
cargo run --bin sniperforge ml optimize-execution [OPCIONES]
```

**Opciones**:
- `-s, --size <SOL>` - Tamaño total del trade en SOL (default: 1000)
- `-m, --max-slippage <PCT>` - Slippage máximo aceptable (default: 0.5)
- `-t, --time-limit <MINUTES>` - Límite de tiempo de ejecución (default: 60)

### `ml train-models` - Entrenar Modelos
```bash
cargo run --bin sniperforge ml train-models [OPCIONES]
```

**Opciones**:
- `-m, --model <TYPE>` - Tipo de modelo: pattern,strategy,risk,timing,all (default: all)
- `-d, --days <DAYS>` - Período de datos de entrenamiento (default: 30)
- `-v, --validation <RATIO>` - Ratio de datos de validación (default: 0.2)

### `ml model-status` - Estado de Modelos
```bash
cargo run --bin sniperforge ml model-status [OPCIONES]
```

**Opciones**:
- `-d, --detailed` - Mostrar métricas detalladas

### `ml advanced-predict` - Predicción Avanzada Ensemble
```bash
cargo run --bin sniperforge ml advanced-predict [OPCIONES]
```

**Opciones**:
- `-s, --symbol <TOKEN>` - Símbolo a analizar (default: SOL/USDC)
- `-t, --timeframe <TIMEFRAME>` - Timeframe de predicción (default: 1h)
- `-c, --confidence <THRESHOLD>` - Umbral de confianza (default: 0.8)

### `ml optimize-portfolio` - Optimización de Portfolio
```bash
cargo run --bin sniperforge ml optimize-portfolio [OPCIONES]
```

**Opciones**:
- `-p, --portfolio <ASSET:WEIGHT,...>` - Portfolio actual como pares asset:peso
- `-s, --strategy <STRATEGY>` - Estrategia: MaxSharpe,MinVolatility,MLPredicted (default: MaxSharpe)

---

## 📈 COMANDOS PORTFOLIO (FASE 6C)

### `portfolio summary` - Resumen de Portfolio
```bash
cargo run --bin sniperforge portfolio summary [OPCIONES]
```

**Opciones**:
- `-d, --detailed` - Mostrar análisis detallado

### `portfolio analytics` - Análisis de Performance
```bash
cargo run --bin sniperforge portfolio analytics [OPCIONES]
```

**Opciones**:
- `-p, --period <DAYS>` - Período de análisis en días (default: 30)
- `-e, --export <FILE>` - Exportar análisis a archivo JSON

### `portfolio risk-assessment` - Evaluación de Riesgo
```bash
cargo run --bin sniperforge portfolio risk-assessment [OPCIONES]
```

**Opciones**:
- `-d, --detailed` - Mostrar desglose detallado de riesgo

### `portfolio rebalance` - Rebalanceo de Portfolio
```bash
cargo run --bin sniperforge portfolio rebalance [OPCIONES]
```

**Opciones**:
- `-d, --dry-run` - Mostrar análisis sin ejecutar
- `-t, --threshold <PCT>` - Umbral de rebalanceo en porcentaje (default: 5.0)

### `portfolio correlation` - Análisis de Correlación
```bash
cargo run --bin sniperforge portfolio correlation [OPCIONES]
```

**Opciones**:
- `-p, --period <DAYS>` - Período histórico para correlación (default: 30)
- `-t, --threshold <CORRELATION>` - Umbral de alta correlación (default: 0.7)

### `portfolio attribution` - Análisis de Atribución
```bash
cargo run --bin sniperforge portfolio attribution [OPCIONES]
```

**Opciones**:
- `-p, --period <DAYS>` - Período de análisis de atribución (default: 30)

### `portfolio optimize` - Optimización de Asignación
```bash
cargo run --bin sniperforge portfolio optimize [OPCIONES]
```

**Opciones**:
- `-t, --targets <STRATEGY:PCT,...>` - Asignaciones objetivo por estrategia
- `-r, --risk <LEVEL>` - Nivel de riesgo: conservative,moderate,aggressive (default: moderate)

### `portfolio positions` - Mostrar Posiciones
```bash
cargo run --bin sniperforge portfolio positions [OPCIONES]
```

**Opciones**:
- `-s, --strategy <STRATEGY>` - Filtrar por estrategia: trend,momentum,mean_reversion,arbitrage
- `--sort <FIELD>` - Ordenar por: pnl,value,symbol,strategy (default: pnl)

---

## 🔧 COMANDOS ÚTILES RÁPIDOS

### Verificar Sistema Completo
```bash
# Ejecutar todos los tests
cargo run --bin sniperforge test all

# Verificar estado
cargo run --bin sniperforge status
```

### Workflow Sprint 1 (Datos Reales)
```bash
# 1. Verificar wallet
cargo run --bin sniperforge wallet balance test-wallet.json

# 2. Test completo DevNet
cargo run --bin sniperforge test swap-real --wallet test-wallet.json --confirm

# 3. Preparar Mainnet (si aplica)
cargo run --bin sniperforge test swap-real --wallet mainnet-validation-wallet.json --network mainnet
```

### Desarrollo y Debug
```bash
# Modo devnet
cargo run --bin sniperforge start --devnet

# Tests específicos
cargo run --bin sniperforge test jupiter
cargo run --bin sniperforge test websocket
```

---

## ⚠️ NOTAS IMPORTANTES

### **Datos Reales vs Simulación**
- **Sin `--confirm`**: Modo simulación/preview
- **Con `--confirm`**: Transacciones REALES en blockchain

### **DevNet vs Mainnet**
- **DevNet**: SOL de prueba, sin valor monetario
- **Mainnet**: SOL REAL con valor monetario

### **Archivos de Wallet**
- `test-wallet.json` - Wallet DevNet
- `mainnet-validation-wallet.json` - Wallet Mainnet
- Generar nuevas: `wallet generate`

### **Estado Sprint 1**
- ✅ **Código 100% datos reales** (0% mock)
- ✅ **Todos los tests pasando**
- ✅ **DevNet completamente funcional**
- ⏳ **Mainnet listo para validación**

---

## 🚀 QUICK START

```bash
# 1. Verificar sistema
cargo run --bin sniperforge test all

# 2. Generar wallet si necesario
cargo run --bin sniperforge wallet generate

# 3. Verificar balance
cargo run --bin sniperforge wallet balance test-wallet-new.json

# 4. Ejecutar swap real DevNet
cargo run --bin sniperforge test swap-real --wallet test-wallet-new.json --confirm

# 5. ¡Listo para usar! 🎉
```

**¡La plataforma está completamente operativa con datos reales!** 🚀
