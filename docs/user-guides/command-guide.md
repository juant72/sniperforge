# 📚 SNIPERFORGE CLI - GUÍA COMPLETA DE COMANDOS

**Versión**: 0.1.0 (Sprint 1 + Fases 6A/6B/6C)  
**Fecha**: Junio 27, 2025  
**Estado**: Sprint 1 Completado ✅ + Comandos Avanzados ML/Portfolio - SELECCIÓN DE RED OBLIGATORIA

## 🔥 **CAMBIO CRÍTICO DE SEGURIDAD**

**⚠️ NUEVO REQUISITO: TODOS LOS COMANDOS PRINCIPALES REQUIEREN `--network`**

A partir de esta versión, **NO HAY VALORES POR DEFECTO** para la red. Debes especificar explícitamente:
- `--network devnet` - Para pruebas en DevNet
- `--network mainnet` - Para operaciones en Mainnet (DINERO REAL)

**Esto previene ejecuciones accidentales en la red incorrecta.**

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
sniperforge start --help
sniperforge wallet --help
sniperforge test --help
sniperforge test swap-real --help
sniperforge ml --help
```

---

## 🚀 COMANDOS PRINCIPALES

### `start` - Iniciar la Plataforma
```bash
cargo run --bin sniperforge start --network <NETWORK> [OPCIONES]
```

**Descripción**: Inicia la plataforma SniperForge o bots específicos

**Parámetros Obligatorios**:
- `--network <NETWORK>` - Red a utilizar: `devnet` o `mainnet`

**Opciones**:
- `-b, --bot <BOT_TYPE>` - Bot específico a iniciar (puede repetirse)

**Ejemplos**:
```bash
# Iniciar plataforma en DevNet
cargo run --bin sniperforge start --network devnet

# Iniciar en Mainnet (DINERO REAL)
cargo run --bin sniperforge start --network mainnet

# Iniciar bot específico en DevNet
cargo run --bin sniperforge start --network devnet -b lp-sniper
```

---

### `status` - Estado de la Plataforma
```bash
cargo run --bin sniperforge status --network <NETWORK>
```

**Descripción**: Muestra el estado actual de la plataforma en la red especificada

**Parámetros Obligatorios**:
- `--network <NETWORK>` - Red a verificar: `devnet` o `mainnet`

**Ejemplos**:
```bash
# Estado en DevNet
cargo run --bin sniperforge status --network devnet

# Estado en Mainnet
cargo run --bin sniperforge status --network mainnet
```

---

### `config` - Mostrar Configuración
```bash
cargo run --bin sniperforge config --network <NETWORK>
```

**Descripción**: Muestra la configuración para la red especificada

**Parámetros Obligatorios**:
- `--network <NETWORK>` - Red de configuración: `devnet` o `mainnet`

**Ejemplos**:
```bash
# Configuración DevNet
cargo run --bin sniperforge config --network devnet

# Configuración Mainnet
cargo run --bin sniperforge config --network mainnet
```

---

## 💰 COMANDOS WALLET

### `wallet balance` - Verificar Balance
```bash
cargo run --bin sniperforge wallet balance --network <NETWORK> [OPCIONES]
```

**Descripción**: Verifica el balance de una wallet en la red especificada

**Parámetros Obligatorios**:
- `--network <NETWORK>` - Red a verificar: `devnet` o `mainnet`

**Opciones**:
- `WALLET_FILE` - Archivo JSON de la wallet
- `-a, --address <ADDRESS>` - Dirección de wallet (alternativa al archivo)

**Ejemplos**:
```bash
# Verificar wallet en DevNet
cargo run --bin sniperforge wallet balance --network devnet test-wallet.json

# Verificar wallet en Mainnet
cargo run --bin sniperforge wallet balance --network mainnet mainnet-validation-wallet.json

# Verificar por dirección en Mainnet
cargo run --bin sniperforge wallet balance --network mainnet --address 7BgBvyjrZX8YKHGoM7BXJnK2vhABwxnVUvRSHFHHkLjr
```

### `wallet airdrop` - Solicitar Airdrop
```bash
cargo run --bin sniperforge wallet airdrop --network devnet <WALLET_FILE>
```

**Descripción**: Solicita airdrop de SOL en DevNet (SOLO DevNet)

**Parámetros Obligatorios**:
- `--network devnet` - SOLO funciona en DevNet
- `WALLET_FILE` - Archivo de wallet para recibir el airdrop

**Ejemplos**:
```bash
# Solicitar airdrop en DevNet
cargo run --bin sniperforge wallet airdrop --network devnet test-wallet.json
```

### `wallet generate` - Generar Nueva Wallet
```bash
cargo run --bin sniperforge wallet generate --network <NETWORK> [OPCIONES]
```

**Descripción**: Genera una nueva wallet para la red especificada

**Parámetros Obligatorios**:
- `--network <NETWORK>` - Red objetivo: `devnet` o `mainnet`

**Opciones**:
- `-o, --output <FILE>` - Archivo de salida (default: test-wallet-new.json)

**Ejemplos**:
```bash
# Generar wallet para DevNet
cargo run --bin sniperforge wallet generate --network devnet

# Generar wallet para Mainnet
cargo run --bin sniperforge wallet generate --network mainnet --output mainnet-wallet.json
```

### `wallet export` - Exportar para Móvil
```bash
cargo run --bin sniperforge wallet export <WALLET_FILE> [OPCIONES]
```

**Descripción**: Exporta wallet para importar en apps móviles (Phantom, Solflare, etc.)

**Parámetros**:
- `WALLET_FILE` - Archivo de wallet a exportar

**Opciones**:
- `-o, --output <FILE>` - Archivo de exportación (default: wallet-export-MOBILE.txt)

**Ejemplos**:
```bash
# Exportar wallet
cargo run --bin sniperforge wallet export mainnet-validation-wallet.json

# Exportar con nombre específico
cargo run --bin sniperforge wallet export test-wallet.json --output mi-export.txt
```

---

## 🧪 COMANDOS TEST

### `test all` - Ejecutar Todos los Tests
```bash
cargo run --bin sniperforge test all --network <NETWORK>
```

**Descripción**: Ejecuta todos los tests en la red especificada

**Parámetros Obligatorios**:
- `--network <NETWORK>` - Red para testing: `devnet` o `mainnet`

**Ejemplos**:
```bash
# Tests en DevNet
cargo run --bin sniperforge test all --network devnet

# Tests en Mainnet (cuidado)
cargo run --bin sniperforge test all --network mainnet
```

### `test basic` - Tests Básicos
```bash
cargo run --bin sniperforge test basic --network <NETWORK>
```

**Descripción**: Tests básicos de conectividad en la red especificada

**Ejemplos**:
```bash
cargo run --bin sniperforge test basic --network devnet
cargo run --bin sniperforge test basic --network mainnet
```

### `test solana` - Tests Solana RPC
```bash
cargo run --bin sniperforge test solana --network <NETWORK>
```

**Descripción**: Tests de conectividad RPC de Solana

**Ejemplos**:
```bash
cargo run --bin sniperforge test solana --network devnet
cargo run --bin sniperforge test solana --network mainnet
```

### `test websocket` - Tests WebSocket
```bash
cargo run --bin sniperforge test websocket --network <NETWORK>
```

**Descripción**: Tests de conectividad WebSocket

**Ejemplos**:
```bash
cargo run --bin sniperforge test websocket --network devnet
cargo run --bin sniperforge test websocket --network mainnet
```

### `test swap-real` - 🚀 SWAP REAL (SPRINT 1)
```bash
cargo run --bin sniperforge test swap-real --network <NETWORK> [OPCIONES]
```

**Descripción**: **COMANDO PRINCIPAL SPRINT 1** - Ejecuta swaps reales en blockchain

**Parámetros Obligatorios**:
- `--network <NETWORK>` - Red a usar: `devnet` o `mainnet`

**Opciones**:
- `-a, --amount <SOL>` - Cantidad de SOL a intercambiar (default: 0.00001)
- `-w, --wallet <FILE>` - Archivo de wallet para ejecución real
- `--confirm` - Confirmar envío de transacción REAL

**⚠️ MEDIDAS DE SEGURIDAD IMPLEMENTADAS**:
- **Límite máximo DevNet**: 1.0 SOL por transacción
- **Límite máximo Mainnet**: 0.1 SOL por transacción
- **Margen de seguridad**: 0.01 SOL se mantiene para fees
- **Verificación de balance** antes y después de transacciones
- **Validación de cantidades** para prevenir drenado de wallets

**Ejemplos**:
```bash
# Simulación DevNet (sin --confirm, muestra qué haría)
cargo run --bin sniperforge test swap-real --network devnet --wallet test-wallet.json

# Swap real DevNet
cargo run --bin sniperforge test swap-real --network devnet --wallet test-wallet.json --confirm

# Swap real Mainnet (¡DINERO REAL!)
cargo run --bin sniperforge test swap-real --network mainnet --wallet mainnet-validation-wallet.json --amount 0.001 --confirm
```

### `test integration` - Tests de Integración
```bash
cargo run --bin sniperforge test integration --network <NETWORK>
```

**Descripción**: Tests del flujo completo de integración

### `test performance` - Tests de Performance
```bash
cargo run --bin sniperforge test performance --network <NETWORK>
```

**Descripción**: Tests de performance y latencia

---

## � COMANDO DE INVESTIGACIÓN

### `check-balance` - Verificar Cualquier Dirección
```bash
cargo run --bin sniperforge check-balance --network <NETWORK> --address <ADDRESS>
```

**Descripción**: Verifica el balance de cualquier dirección de wallet para investigación

**Parámetros Obligatorios**:
- `--network <NETWORK>` - Red a verificar: `devnet` o `mainnet`
- `--address <ADDRESS>` - Dirección de wallet a verificar

**Ejemplos**:
```bash
# Verificar dirección en Mainnet
cargo run --bin sniperforge check-balance --network mainnet --address 7BgBvyjrZX8YKHGoM7BXJnK2vhABwxnVUvRSHFHHkLjr

# Verificar dirección en DevNet
cargo run --bin sniperforge check-balance --network devnet --address ABC123...
```

---

## �🖥️ MODO INTERACTIVO

### `interactive` - Modo Monitoring Interactivo
```bash
cargo run --bin sniperforge interactive --network <NETWORK>
```

**Descripción**: Inicia el modo de monitoreo interactivo para la red especificada

**Parámetros Obligatorios**:
- `--network <NETWORK>` - Red a monitorear: `devnet` o `mainnet`

**Ejemplos**:
```bash
# Modo interactivo DevNet
cargo run --bin sniperforge interactive --network devnet

# Modo interactivo Mainnet
cargo run --bin sniperforge interactive --network mainnet
```

---

## 📊 COMANDOS AVANZADOS (FASE 6A)

### `multi-strategy-trading` - Trading Multi-Estrategia
```bash
cargo run --bin sniperforge multi-strategy-trading --network <NETWORK> [OPCIONES]
```

**Descripción**: Ejecuta múltiples estrategias de trading concurrentemente

**Parámetros Obligatorios**:
- `--network <NETWORK>` - Red para trading: `devnet` o `mainnet`

**Opciones**:
- `-s, --strategies <LIST>` - Estrategias separadas por comas (default: trend,momentum)
- `-d, --duration <SECONDS>` - Duración de sesión (default: 300)
- `-c, --capital <USD>` - Capital por estrategia (default: 5000)
- `-t, --timeframes <LIST>` - Timeframes de análisis (default: 1m,5m)

**Ejemplos**:
```bash
# Trading multi-estrategia DevNet
cargo run --bin sniperforge multi-strategy-trading --network devnet

# Estrategias específicas en Mainnet
cargo run --bin sniperforge multi-strategy-trading --network mainnet --strategies trend,arbitrage --duration 600
```

### `strategy-backtest` - Backtesting de Estrategias
```bash
cargo run --bin sniperforge strategy-backtest --network <NETWORK> [OPCIONES]
```

**Descripción**: Backtesting de estrategias individuales o combinadas

**Parámetros Obligatorios**:
- `--network <NETWORK>` - Red para datos históricos: `devnet` o `mainnet`

**Opciones**:
- `-s, --strategy <STRATEGY>` - Estrategia: trend,momentum,mean-reversion,arbitrage,all (default: trend)
- `-p, --period <DAYS>` - Período histórico en días (default: 7)
- `-c, --capital <USD>` - Capital inicial (default: 10000)
- `-e, --export <FILE>` - Exportar resultados a JSON

**Ejemplos**:
```bash
# Backtest básico DevNet
cargo run --bin sniperforge strategy-backtest --network devnet

# Backtest completo con exportación
cargo run --bin sniperforge strategy-backtest --network mainnet --strategy all --period 30 --export backtest-results.json
```

### `pattern-analysis` - Análisis de Patrones
```bash
cargo run --bin sniperforge pattern-analysis --network <NETWORK> [OPCIONES]
```

**Descripción**: Analiza patrones de mercado y tendencias

**Parámetros Obligatorios**:
- `--network <NETWORK>` - Red para análisis: `devnet` o `mainnet`

**Opciones**:
- `-p, --pattern <PATTERN>` - Tipo: support-resistance,breakout,reversal,all (default: all)
- `-t, --timeframe <TIMEFRAME>` - Timeframe: 1m,5m,15m,1h (default: 5m)
- `-d, --duration <SECONDS>` - Duración de análisis (default: 180)
- `-e, --export <FILE>` - Exportar análisis a JSON

**Ejemplos**:
```bash
# Análisis de patrones DevNet
cargo run --bin sniperforge pattern-analysis --network devnet

# Análisis específico de breakouts
cargo run --bin sniperforge pattern-analysis --network mainnet --pattern breakout --timeframe 15m
```

### `arbitrage-scan` - Escaneo de Arbitraje
```bash
cargo run --bin sniperforge arbitrage-scan --network <NETWORK> [OPCIONES]
```

**Descripción**: Escanea oportunidades de arbitraje entre DEXs

**Parámetros Obligatorios**:
- `--network <NETWORK>` - Red para escaneo: `devnet` o `mainnet`

**Opciones**:
- `-m, --min-profit <USD>` - Ganancia mínima en USD (default: 5.0)
- `--max-slippage <PCT>` - Slippage máximo (default: 0.5)
- `-d, --duration <SECONDS>` - Duración de escaneo (default: 120)
- `-e, --export <FILE>` - Exportar oportunidades a JSON

**Ejemplos**:
```bash
# Escaneo básico DevNet
cargo run --bin sniperforge arbitrage-scan --network devnet

# Escaneo con parámetros específicos Mainnet
cargo run --bin sniperforge arbitrage-scan --network mainnet --min-profit 10.0 --max-slippage 0.3
```

---

## 🤖 COMANDOS MACHINE LEARNING (FASE 6B)

### `ml analyze-patterns` - Análisis ML de Patrones
```bash
cargo run --bin sniperforge ml analyze-patterns [OPCIONES]
```

**Descripción**: Analiza patrones de mercado usando modelos de Machine Learning

**Opciones**:
- `-s, --symbol <TOKEN>` - Símbolo a analizar (default: SOL/USDC)
- `-t, --timeframe <MINUTES>` - Timeframe en minutos (default: 5)
- `-c, --confidence <THRESHOLD>` - Umbral de confianza 0.0-1.0 (default: 0.8)

**Ejemplos**:
```bash
# Análisis ML básico
cargo run --bin sniperforge ml analyze-patterns

# Análisis específico con alta confianza
cargo run --bin sniperforge ml analyze-patterns --symbol BTC/USDC --confidence 0.9
```

### `ml predict-trend` - Predicción de Tendencias
```bash
cargo run --bin sniperforge ml predict-trend [OPCIONES]
```

**Descripción**: Predice tendencias de precios usando modelos ML

**Opciones**:
- `-s, --symbol <TOKEN>` - Símbolo a predecir (default: SOL/USDC)
- `-h, --horizon <MINUTES>` - Horizonte de predicción (default: 15)
- `-c, --confidence <THRESHOLD>` - Umbral de confianza (default: 0.7)

### `ml optimize-strategy` - Optimización de Estrategias
```bash
cargo run --bin sniperforge ml optimize-strategy [OPCIONES]
```

**Descripción**: Optimiza estrategias usando algoritmos genéticos

**Opciones**:
- `-s, --strategy <STRATEGY>` - Estrategia: trend,momentum,mean-reversion (default: trend)
- `-g, --generations <COUNT>` - Generaciones de optimización (default: 50)
- `-p, --population <SIZE>` - Tamaño de población (default: 20)

### `ml backtest-optimized` - Backtest Optimizado
```bash
cargo run --bin sniperforge ml backtest-optimized [OPCIONES]
```

**Descripción**: Backtest de estrategias optimizadas con ML

**Opciones**:
- `-s, --strategy <STRATEGY>` - Estrategia a probar (default: trend)
- `-p, --period <DAYS>` - Período en días (default: 30)
- `-c, --confidence <THRESHOLD>` - Confianza mínima (default: 0.7)

### `ml assess-risk` - Evaluación de Riesgo
```bash
cargo run --bin sniperforge ml assess-risk [OPCIONES]
```

**Descripción**: Evalúa riesgo de mercado usando modelos ML

**Opciones**:
- `-w, --window <HOURS>` - Ventana de análisis en horas (default: 24)
- `-p, --portfolio <TOKENS>` - Tokens del portfolio separados por coma (default: SOL,USDC)

### `ml market-regime` - Detección de Régimen de Mercado
```bash
cargo run --bin sniperforge ml market-regime [OPCIONES]
```

**Descripción**: Detecta el régimen actual del mercado (bull/bear/sideways)

**Opciones**:
- `-c, --confidence <THRESHOLD>` - Confianza mínima (default: 0.9)
- `-l, --lookback <DAYS>` - Período histórico (default: 14)

### `ml predict-timing` - Predicción de Timing
```bash
cargo run --bin sniperforge ml predict-timing [OPCIONES]
```

**Descripción**: Predice timing óptimo para ejecución de trades

**Opciones**:
- `-s, --symbol <TOKEN>` - Símbolo para predicción (default: SOL/USDC)
- `-t, --size <AMOUNT>` - Tamaño objetivo del trade (default: 100)
- `-d, --direction <BUY_OR_SELL>` - Dirección: buy o sell (default: buy)

### `ml optimize-execution` - Optimización de Ejecución
```bash
cargo run --bin sniperforge ml optimize-execution [OPCIONES]
```

**Descripción**: Optimiza ejecución de órdenes grandes

**Opciones**:
- `-s, --size <SOL>` - Tamaño total en SOL (default: 1000)
- `-m, --max-slippage <PCT>` - Slippage máximo (default: 0.5)
- `-t, --time-limit <MINUTES>` - Límite de tiempo (default: 60)

### `ml train-models` - Entrenamiento de Modelos
```bash
cargo run --bin sniperforge ml train-models [OPCIONES]
```

**Descripción**: Entrena o reentrena modelos ML

**Opciones**:
- `-m, --model <TYPE>` - Tipo: pattern,strategy,risk,timing,all (default: all)
- `-d, --days <DAYS>` - Días de datos de entrenamiento (default: 30)
- `-v, --validation <RATIO>` - Ratio de validación (default: 0.2)

### `ml model-status` - Estado de Modelos
```bash
cargo run --bin sniperforge ml model-status [OPCIONES]
```

**Descripción**: Muestra estado y rendimiento de modelos ML

**Opciones**:
- `-d, --detailed` - Mostrar métricas detalladas

### `ml advanced-predict` - Predicción Avanzada
```bash
cargo run --bin sniperforge ml advanced-predict [OPCIONES]
```

**Descripción**: Predicción ML ensemble y recomendaciones de trading

**Opciones**:
- `-s, --symbol <TOKEN>` - Símbolo a analizar (default: SOL/USDC)
- `-t, --timeframe <TIMEFRAME>` - Timeframe (default: 1h)
- `-c, --confidence <THRESHOLD>` - Umbral de confianza (default: 0.8)

### `ml optimize-portfolio` - Optimización de Portfolio
```bash
cargo run --bin sniperforge ml optimize-portfolio [OPCIONES]
```

**Descripción**: Optimiza asignación de portfolio usando ML

**Opciones**:
- `-p, --portfolio <ASSET:WEIGHT,...>` - Portfolio actual
- `-s, --strategy <STRATEGY>` - Estrategia: MaxSharpe,MinVolatility,MLPredicted (default: MaxSharpe)

---

## 📈 COMANDOS PORTFOLIO (FASE 6C)

### `portfolio summary` - Resumen de Portfolio
```bash
cargo run --bin sniperforge portfolio summary [OPCIONES]
```

**Descripción**: Muestra resumen y métricas del portfolio

**Opciones**:
- `-d, --detailed` - Mostrar análisis detallado

### `portfolio analytics` - Análisis de Performance
```bash
cargo run --bin sniperforge portfolio analytics [OPCIONES]
```

**Descripción**: Genera análisis completo de performance

**Opciones**:
- `-p, --period <DAYS>` - Período de análisis (default: 30)
- `-e, --export <FILE>` - Exportar a JSON

### `portfolio risk-assessment` - Evaluación de Riesgo
```bash
cargo run --bin sniperforge portfolio risk-assessment [OPCIONES]
```

**Descripción**: Realiza evaluación de riesgo del portfolio

**Opciones**:
- `-d, --detailed` - Mostrar desglose detallado de riesgo

### `portfolio rebalance` - Rebalanceo
```bash
cargo run --bin sniperforge portfolio rebalance [OPCIONES]
```

**Descripción**: Analiza y ejecuta rebalanceo del portfolio

**Opciones**:
- `-d, --dry-run` - Mostrar análisis sin ejecutar
- `-t, --threshold <PCT>` - Umbral de rebalanceo (default: 5.0)

### `portfolio correlation` - Análisis de Correlación
```bash
cargo run --bin sniperforge portfolio correlation [OPCIONES]
```

**Descripción**: Analiza correlación y diversificación

**Opciones**:
- `-p, --period <DAYS>` - Período histórico (default: 30)
- `-t, --threshold <CORRELATION>` - Umbral de alta correlación (default: 0.7)

### `portfolio attribution` - Atribución de Performance
```bash
cargo run --bin sniperforge portfolio attribution [OPCIONES]
```

**Descripción**: Análisis de atribución de performance

**Opciones**:
- `-p, --period <DAYS>` - Período de análisis (default: 30)

### `portfolio optimize` - Optimización de Asignación
```bash
cargo run --bin sniperforge portfolio optimize [OPCIONES]
```

**Descripción**: Escanea oportunidades de arbitraje entre DEXs

**Parámetros Obligatorios**:
- `--network <NETWORK>` - Red para escaneo: `devnet` o `mainnet`

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
- `-c, --confidence <THRESHOLD>` - Confianza mínima para backtests (default: 0.7)

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
- `-p, --period <DAYS>` - Período histórico para correlrelación (default: 30)
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

**Descripción**: Optimiza asignación de portfolio

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

**Ejemplos**:
```bash
# Resumen completo del portfolio
cargo run --bin sniperforge portfolio summary --detailed

# Análisis de performance mensual
cargo run --bin sniperforge portfolio analytics --period 30 --export performance-report.json

# Evaluación de riesgo detallada
cargo run --bin sniperforge portfolio risk-assessment --detailed

# Análisis de correlación
cargo run --bin sniperforge portfolio correlation --period 60

# Rebalanceo automático (simulación)
cargo run --bin sniperforge portfolio rebalance --dry-run

# Optimización conservadora
cargo run --bin sniperforge portfolio optimize --risk conservative

# Ver posiciones por estrategia
cargo run --bin sniperforge portfolio positions --strategy trend --sort pnl
```

---

## 🔧 COMANDOS ÚTILES RÁPIDOS

### Verificar Sistema Completo
```bash
# Ejecutar todos los tests en DevNet
cargo run --bin sniperforge test all --network devnet

# Verificar estado en DevNet
cargo run --bin sniperforge status --network devnet

# Verificar configuración Mainnet
cargo run --bin sniperforge config --network mainnet
```

### Workflow Sprint 1 (Datos Reales)
```bash
# 1. Verificar wallet DevNet
cargo run --bin sniperforge wallet balance --network devnet test-wallet.json

# 2. Test completo DevNet
cargo run --bin sniperforge test swap-real --network devnet --wallet test-wallet.json --confirm

# 3. Preparar Mainnet (si aplica)
cargo run --bin sniperforge test swap-real --network mainnet --wallet mainnet-validation-wallet.json --amount 0.001
```

### Desarrollo y Debug
```bash
# Verificar conectividad DevNet
cargo run --bin sniperforge test basic --network devnet

# Tests específicos DevNet
cargo run --bin sniperforge test jupiter --network devnet
cargo run --bin sniperforge test websocket --network devnet

# Modo interactivo para monitoreo
cargo run --bin sniperforge interactive --network devnet
```

### Comandos Avanzados Rápidos
```bash
# Análisis ML básico
cargo run --bin sniperforge ml analyze-patterns --symbol SOL/USDC

# Predicción de tendencias
cargo run --bin sniperforge ml predict-trend --horizon 30

# Escaneo de arbitraje DevNet
cargo run --bin sniperforge arbitrage-scan --network devnet --min-profit 10.0

# Análisis de patrones DevNet
cargo run --bin sniperforge pattern-analysis --network devnet --pattern breakout

# Resumen del portfolio
cargo run --bin sniperforge portfolio summary --detailed

# Estado de modelos ML
cargo run --bin sniperforge ml model-status --detailed
```

### Verificación de Balances
```bash
# Verificar balance propio DevNet
cargo run --bin sniperforge wallet balance --network devnet test-wallet.json

# Verificar cualquier dirección Mainnet
cargo run --bin sniperforge check-balance --network mainnet --address 7BgBvyjrZX8YKHGoM7BXJnK2vhABwxnVUvRSHFHHkLjr

# Verificar balance propio Mainnet
cargo run --bin sniperforge wallet balance --network mainnet mainnet-validation-wallet.json
```

---

---

## 🛡️ CAMBIOS CRÍTICOS DE SEGURIDAD

### **NUEVO REQUISITO: SELECCIÓN EXPLÍCITA DE RED**

**⚠️ CAMBIO IMPORTANTE**: A partir de esta versión, **TODOS** los comandos principales requieren el parámetro `--network`.

**Antes (INSEGURO)**:
```bash
# ❌ Esto ya NO funciona - fallará con error
cargo run --bin sniperforge start
cargo run --bin sniperforge wallet balance test-wallet.json
cargo run --bin sniperforge test swap-real --confirm
```

**Ahora (SEGURO)**:
```bash
# ✅ Esto es obligatorio - especificación explícita
cargo run --bin sniperforge start --network devnet
cargo run --bin sniperforge wallet balance --network devnet test-wallet.json
cargo run --bin sniperforge test swap-real --network devnet --confirm
```

### **MEDIDAS DE PROTECCIÓN IMPLEMENTADAS**

1. **No hay valores por defecto de red** - Previene ejecuciones accidentales
2. **Límites de swap obligatorios**:
   - DevNet: Máximo 1.0 SOL por transacción
   - Mainnet: Máximo 0.1 SOL por transacción
3. **Margen de seguridad**: 0.01 SOL siempre reservado para fees
4. **Validación de balances** antes y después de transacciones
5. **Verificación de cantidades** para prevenir drenado de wallets

### **COMANDOS QUE REQUIEREN --network**

**Todos estos comandos ahora requieren `--network devnet` o `--network mainnet`**:

- `start` - Iniciar plataforma
- `status` - Ver estado
- `config` - Ver configuración
- `wallet balance` - Verificar balance
- `wallet airdrop` - Solicitar airdrop (solo devnet)
- `wallet generate` - Generar wallet
- `test all` - Ejecutar todos los tests
- `test basic` - Tests básicos
- `test solana` - Tests Solana RPC
- `test websocket` - Tests WebSocket
- `test swap-real` - **SWAP REAL** (comando crítico)
- `test integration` - Tests integración
- `test performance` - Tests performance
- `interactive` - Modo interactivo
- `check-balance` - Verificar cualquier dirección
- Todos los comandos avanzados (Phase 6A/6B)

### **¿POR QUÉ ESTE CAMBIO?**

**Incidente crítico previo**: Un usuario perdió todos los fondos en Mainnet porque:
1. Jupiter API devolvió una quote que usaba todo el balance disponible
2. No había validaciones de cantidad máxima
3. El comando no requería selección explícita de red

**Protecciones implementadas**:
- **Explícita selección de red** previene confusión DevNet/Mainnet
- **Límites máximos** previenen pérdidas accidentales grandes
- **Validaciones múltiples** verifican cada transacción

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
- Generar nuevas: `wallet generate --network <NETWORK>`

### **Estado Sprint 1**
- ✅ **Código 100% datos reales** (0% mock)
- ✅ **Todos los tests pasando**
- ✅ **DevNet completamente funcional**
- ✅ **Medidas de seguridad implementadas**
- ✅ **Selección explícita de red obligatoria**
- ⏳ **Mainnet listo para validación segura**

---

## 🚀 QUICK START ACTUALIZADO

```bash
# 1. Verificar sistema en DevNet
cargo run --bin sniperforge test all --network devnet

# 2. Generar wallet para DevNet
cargo run --bin sniperforge wallet generate --network devnet

# 3. Solicitar airdrop (solo DevNet)
cargo run --bin sniperforge wallet airdrop --network devnet test-wallet-new.json

# 4. Verificar balance
cargo run --bin sniperforge wallet balance --network devnet test-wallet-new.json

# 5. Ejecutar swap real DevNet (SEGURO)
cargo run --bin sniperforge test swap-real --network devnet --wallet test-wallet-new.json --confirm

# 6. Para Mainnet (DINERO REAL) - Generar wallet separada
cargo run --bin sniperforge wallet generate --network mainnet --output mainnet-wallet.json

# 7. Verificar balance Mainnet
cargo run --bin sniperforge wallet balance --network mainnet mainnet-wallet.json

# 8. ¡Listo para usar de forma segura! 🛡️
```

**¡La plataforma está completamente operativa con datos reales y medidas de seguridad robustas!** 🚀🛡️

---

## 🎯 FLUJOS DE TRABAJO COMPLETOS

### **🔰 Flujo de Trabajo Principiante (DevNet)**
```bash
# 1. Verificar que todo funciona
cargo run --bin sniperforge test all --network devnet

# 2. Generar wallet nueva
cargo run --bin sniperforge wallet generate --network devnet --output mi-wallet-devnet.json

# 3. Obtener SOL de prueba
cargo run --bin sniperforge wallet airdrop --network devnet mi-wallet-devnet.json

# 4. Verificar balance
cargo run --bin sniperforge wallet balance --network devnet mi-wallet-devnet.json

# 5. Primer swap real (seguro en DevNet)
cargo run --bin sniperforge test swap-real --network devnet --wallet mi-wallet-devnet.json --amount 0.001 --confirm

# 6. Verificar balance después del swap
cargo run --bin sniperforge wallet balance --network devnet mi-wallet-devnet.json
```

### **💼 Flujo de Trabajo Intermedio (Análisis y Estrategias)**
```bash
# 1. Análisis de mercado con ML
cargo run --bin sniperforge ml analyze-patterns --symbol SOL/USDC --confidence 0.8

# 2. Detección de régimen de mercado
cargo run --bin sniperforge ml market-regime --confidence 0.9

# 3. Predicción de tendencias
cargo run --bin sniperforge ml predict-trend --symbol SOL/USDC --horizon 30

# 4. Análisis de patrones técnicos
cargo run --bin sniperforge pattern-analysis --network devnet --pattern all --export patterns.json

# 5. Escaneo de arbitraje
cargo run --bin sniperforge arbitrage-scan --network devnet --min-profit 10.0 --export arbitrage.json

# 6. Backtesting de estrategias
cargo run --bin sniperforge strategy-backtest --network devnet --strategy all --period 14 --export backtest.json

# 7. Trading multi-estrategia en DevNet
cargo run --bin sniperforge multi-strategy-trading --network devnet --strategies trend,momentum --duration 300
```

### **🚀 Flujo de Trabajo Avanzado (ML y Portfolio)**
```bash
# 1. Estado completo de modelos ML
cargo run --bin sniperforge ml model-status --detailed

# 2. Entrenar modelos si es necesario
cargo run --bin sniperforge ml train-models --model all --days 30

# 3. Optimización de estrategias
cargo run --bin sniperforge ml optimize-strategy --strategy trend --generations 100

# 4. Predicción avanzada ensemble
cargo run --bin sniperforge ml advanced-predict --symbol SOL/USDC --timeframe 1h --confidence 0.8

# 5. Análisis completo del portfolio
cargo run --bin sniperforge portfolio summary --detailed

# 6. Evaluación de riesgo
cargo run --bin sniperforge portfolio risk-assessment --detailed

# 7. Análisis de correlación
cargo run --bin sniperforge portfolio correlation --period 60

# 8. Optimización del portfolio
cargo run --bin sniperforge portfolio optimize --risk moderate

# 9. Simulación de rebalanceo
cargo run --bin sniperforge portfolio rebalance --dry-run --threshold 5.0
```

### **💰 Flujo de Trabajo Mainnet (Producción)**
```bash
# ⚠️ SOLO USAR CON DINERO QUE PUEDES PERMITIRTE PERDER

# 1. Generar wallet específica para Mainnet
cargo run --bin sniperforge wallet generate --network mainnet --output mainnet-prod-wallet.json

# 2. FONDEAR MANUALMENTE la wallet (enviar SOL desde exchange)

# 3. Verificar balance después de fondear
cargo run --bin sniperforge wallet balance --network mainnet mainnet-prod-wallet.json

# 4. Verificar estado del sistema Mainnet
cargo run --bin sniperforge status --network mainnet

# 5. Tests básicos Mainnet
cargo run --bin sniperforge test basic --network mainnet

# 6. Análisis de mercado en vivo
cargo run --bin sniperforge ml analyze-patterns --symbol SOL/USDC --confidence 0.9

# 7. Evaluación de riesgo actual
cargo run --bin sniperforge ml assess-risk --window 24

# 8. Primer swap real MUY PEQUEÑO
cargo run --bin sniperforge test swap-real --network mainnet --wallet mainnet-prod-wallet.json --amount 0.001 --confirm

# 9. Verificar resultado
cargo run --bin sniperforge wallet balance --network mainnet mainnet-prod-wallet.json

# 10. Si todo funciona, trading con montos mayores (con extrema precaución)
cargo run --bin sniperforge multi-strategy-trading --network mainnet --capital 100 --duration 300
```

### **🔍 Flujo de Trabajo de Investigación y Análisis**
```bash
# 1. Investigar direcciones específicas
cargo run --bin sniperforge check-balance --network mainnet --address <DIRECCIÓN_INTERESANTE>

# 2. Análisis de patrones específicos
cargo run --bin sniperforge pattern-analysis --network mainnet --pattern breakout --timeframe 15m

# 3. Escaneo exhaustivo de arbitraje
cargo run --bin sniperforge arbitrage-scan --network mainnet --min-profit 5.0 --duration 300

# 4. Backtesting histórico extenso
cargo run --bin sniperforge strategy-backtest --network mainnet --strategy all --period 30

# 5. Análisis de correlaciones del mercado
cargo run --bin sniperforge portfolio correlation --period 90

# 6. Exportar wallet para análisis externo
cargo run --bin sniperforge wallet export mainnet-prod-wallet.json --output research-export.txt
```

### **🛠️ Flujo de Trabajo de Mantenimiento**
```bash
# 1. Verificar estado general del sistema
cargo run --bin sniperforge status --network devnet
cargo run --bin sniperforge status --network mainnet

# 2. Ejecutar suite completa de tests
cargo run --bin sniperforge test all --network devnet

# 3. Verificar estado de modelos ML
cargo run --bin sniperforge ml model-status --detailed

# 4. Reentrenar modelos si es necesario
cargo run --bin sniperforge ml train-models --model all --days 30

# 5. Verificar configuraciones
cargo run --bin sniperforge config --network devnet
cargo run --bin sniperforge config --network mainnet

# 6. Tests de conectividad críticos
cargo run --bin sniperforge test solana --network devnet
cargo run --bin sniperforge test websocket --network devnet
cargo run --bin sniperforge test jupiter --network devnet
```

---

## 📚 COMANDOS DE AYUDA DISPONIBLES

**Recuerda: Todos los comandos tienen ayuda integrada**

```bash
# Ayuda principal
cargo run --bin sniperforge --help

# Ayuda de comandos principales
cargo run --bin sniperforge start --help
cargo run --bin sniperforge wallet --help
cargo run --bin sniperforge test --help
cargo run --bin sniperforge ml --help
cargo run --bin sniperforge portfolio --help

# Ayuda de subcomandos específicos
cargo run --bin sniperforge wallet balance --help
cargo run --bin sniperforge test swap-real --help
cargo run --bin sniperforge ml analyze-patterns --help
cargo run --bin sniperforge portfolio summary --help
cargo run --bin sniperforge multi-strategy-trading --help
```

---

## 📋 REFERENCIA RÁPIDA - COMANDOS MÁS COMUNES

### **🚀 Comandos Esenciales DevNet (Testing Seguro)**
```bash
# 1. Generar wallet para testing
cargo run --bin sniperforge wallet generate --network devnet

# 2. Solicitar airdrop (solo DevNet)
cargo run --bin sniperforge wallet airdrop --network devnet test-wallet-new.json

# 3. Verificar balance
cargo run --bin sniperforge wallet balance --network devnet test-wallet-new.json

# 4. Test básico de conectividad
cargo run --bin sniperforge test basic --network devnet

# 5. Ejecutar swap real en DevNet (sin riesgo monetario)
cargo run --bin sniperforge test swap-real --network devnet --wallet test-wallet-new.json --confirm
```

### **💰 Comandos Mainnet (Dinero Real)**
```bash
# 1. Generar wallet para Mainnet
cargo run --bin sniperforge wallet generate --network mainnet --output mainnet-wallet.json

# 2. Verificar balance (después de fondear manualmente)
cargo run --bin sniperforge wallet balance --network mainnet mainnet-wallet.json

# 3. Verificar estado del sistema
cargo run --bin sniperforge status --network mainnet

# 4. Ejecutar swap real pequeño (¡DINERO REAL!)
cargo run --bin sniperforge test swap-real --network mainnet --wallet mainnet-wallet.json --amount 0.001 --confirm
```

### **🤖 Comandos Machine Learning Básicos**
```bash
# Análisis de patrones con ML
cargo run --bin sniperforge ml analyze-patterns --symbol SOL/USDC --confidence 0.8

# Predicción de tendencias
cargo run --bin sniperforge ml predict-trend --symbol SOL/USDC --horizon 15

# Evaluación de riesgo del mercado
cargo run --bin sniperforge ml assess-risk --window 24

# Detección de régimen de mercado
cargo run --bin sniperforge ml market-regime --confidence 0.9

# Estado de modelos ML
cargo run --bin sniperforge ml model-status --detailed

# Optimización de estrategias
cargo run --bin sniperforge ml optimize-strategy --strategy trend --generations 50
```

### **� Comandos Portfolio y Analytics**
```bash
# Resumen completo del portfolio
cargo run --bin sniperforge portfolio summary --detailed

# Análisis de performance
cargo run --bin sniperforge portfolio analytics --period 30

# Evaluación de riesgo del portfolio
cargo run --bin sniperforge portfolio risk-assessment --detailed

# Análisis de correlación entre assets
cargo run --bin sniperforge portfolio correlation --period 60

# Simulación de rebalanceo
cargo run --bin sniperforge portfolio rebalance --dry-run

# Ver posiciones actuales
cargo run --bin sniperforge portfolio positions --sort pnl
```

### **�🔍 Comandos de Investigación**
```bash
# Verificar balance de cualquier dirección
cargo run --bin sniperforge check-balance --network mainnet --address 7BgBvyjrZX8YKHGoM7BXJnK2vhABwxnVUvRSHFHHkLjr

# Exportar wallet para móvil
cargo run --bin sniperforge wallet export mainnet-wallet.json

# Verificar configuración
cargo run --bin sniperforge config --network mainnet

# Escaneo de arbitraje
cargo run --bin sniperforge arbitrage-scan --network devnet --min-profit 5.0

# Análisis de patrones de mercado
cargo run --bin sniperforge pattern-analysis --network devnet --pattern all
```

### **⚡ Comandos Avanzados de Trading**
```bash
# Trading multi-estrategia
cargo run --bin sniperforge multi-strategy-trading --network devnet --strategies trend,momentum

# Backtesting de estrategias
cargo run --bin sniperforge strategy-backtest --network devnet --strategy all --period 14

# Predicción avanzada con ML ensemble
cargo run --bin sniperforge ml advanced-predict --symbol SOL/USDC --timeframe 1h

# Optimización de ejecución para órdenes grandes
cargo run --bin sniperforge ml optimize-execution --size 1000 --max-slippage 0.5

# Entrenamiento de modelos ML
cargo run --bin sniperforge ml train-models --model all --days 30
```

### **⚠️ RECORDATORIO CRÍTICO**
- **Siempre especificar `--network devnet` o `--network mainnet`**
- **DevNet = Testing (sin valor monetario)**
- **Mainnet = Dinero real (¡cuidado!)**
- **Límites de seguridad activos en todas las transacciones**
- **Usar `--help` en cualquier comando para ver opciones completas**
