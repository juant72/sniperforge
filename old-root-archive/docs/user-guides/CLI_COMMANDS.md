# 🚀 SniperForge CLI Commands Reference

> **📅 Última actualización: July 3, 2025**
> **🎯 Estado: Comandos core implementados, comandos avanzados en desarrollo**

Esta guía completa documenta todos los comandos CLI disponibles en SniperForge, desde básicos hasta avanzados.

---

## 📋 COMANDOS IMPLEMENTADOS ✅

### 🔧 Core Platform Commands

#### `sniperforge start`
Inicia la plataforma SniperForge con configuración por defecto.
```bash
sniperforge start
sniperforge start --network devnet
sniperforge start --config config/custom.toml
```

#### `sniperforge status`
Muestra el estado actual de la plataforma y conexiones.
```bash
sniperforge status
sniperforge status --detailed
```

#### `sniperforge config`
Gestiona la configuración del sistema.
```bash
sniperforge config --show
sniperforge config --validate
sniperforge config --network devnet
```

### 💰 Wallet Management Commands

#### `sniperforge wallet balance`
Consulta balances de wallet con soporte multi-token.
```bash
sniperforge wallet balance wallet.json
sniperforge wallet balance wallet.json --token SOL
sniperforge wallet balance wallet.json --network mainnet
```

**Ejemplo de salida:**
```
🏦 Wallet Balance Summary
═══════════════════════
📍 Network: devnet
📋 Address: 7xKXt...9mPvQ

💰 Balances:
  • SOL: 5.2457 SOL ($944.23)
  • USDC: 1,250.00 USDC
  • Total Value: $2,194.23
```

#### `sniperforge wallet export` 
Exporta información de wallet (sin claves privadas).
```bash
sniperforge wallet export wallet.json
sniperforge wallet export wallet.json --format json
```

### 🧪 Testing & Validation Commands

#### `sniperforge test all`
Ejecuta la suite completa de tests (68 tests).
```bash
sniperforge test all
sniperforge test all --network devnet
sniperforge test all --verbose
```

#### `sniperforge test jupiter`
Tests específicos de integración Jupiter.
```bash
sniperforge test jupiter
sniperforge test jupiter --real-api
```

#### `sniperforge validate`
Valida configuración y conectividad.
```bash
sniperforge validate
sniperforge validate --network mainnet
sniperforge validate --wallet wallet.json
```

---

## 🚀 ADVANCED TRADING COMMANDS ✅

### 📈 Multi-Strategy Trading

#### `sniperforge multi-strategy-trading`
Ejecuta múltiples estrategias de trading simultáneamente.
```bash
sniperforge multi-strategy-trading --strategies dca,momentum --network devnet
sniperforge multi-strategy-trading --strategies all --wallet wallet.json
sniperforge multi-strategy-trading --config strategies.json --dry-run
```

**Opciones disponibles:**
- `--strategies`: dca, momentum, grid, arbitrage, trend, all
- `--network`: devnet, mainnet  
- `--wallet`: archivo de wallet
- `--dry-run`: modo simulación
- `--max-exposure`: límite de exposición total

### 📊 Strategy Backtesting

#### `sniperforge strategy-backtest`
Realiza backtesting de estrategias con datos históricos.
```bash
sniperforge strategy-backtest --strategy trend --period 7 --network devnet
sniperforge strategy-backtest --strategy dca --period 30 --token SOL
sniperforge strategy-backtest --config backtest.json --output results.json
```

**Parámetros:**
- `--strategy`: Estrategia a testear (dca, momentum, trend, grid)
- `--period`: Período en días (1-365)
- `--token`: Token específico para backtest
- `--output`: Archivo de resultados

### 🔍 Pattern Analysis

#### `sniperforge pattern-analysis`
Análisis de patrones técnicos con ML.
```bash
sniperforge pattern-analysis --pattern all --timeframe 5m --network devnet
sniperforge pattern-analysis --pattern breakout --token SOL --output patterns.json
sniperforge pattern-analysis --ai-mode --confidence 0.85
```

**Patrones disponibles:**
- `breakout`: Patrones de ruptura
- `reversal`: Patrones de reversión
- `trend`: Análisis de tendencias
- `support_resistance`: Niveles de soporte/resistencia
- `all`: Todos los patrones

### 🔄 Arbitrage Scanning

#### `sniperforge arbitrage-scan`
Escaneo de oportunidades de arbitraje en tiempo real.
```bash
sniperforge arbitrage-scan --network devnet
sniperforge arbitrage-scan --min-profit 0.5 --max-slippage 2.0
sniperforge arbitrage-scan --tokens SOL,USDC,RAY --output opportunities.json
```

---

## 🤖 MACHINE LEARNING COMMANDS ✅

### 🧠 ML Model Training & Execution

#### `sniperforge ml`
Comandos de machine learning para análisis predictivo.
```bash
sniperforge ml --strategy pattern-recognition --config ml.json
sniperforge ml --mode training --dataset historical_data.json
sniperforge ml --predict --model trend_predictor --token SOL
```

**Estrategias ML disponibles:**
- `pattern-recognition`: Reconocimiento de patrones
- `trend-prediction`: Predicción de tendencias
- `timing-optimization`: Optimización de timing
- `risk-assessment`: Evaluación de riesgo

---

## 🏦 PORTFOLIO MANAGEMENT COMMANDS ✅

### 📊 Professional Portfolio Management

#### `sniperforge portfolio`
Gestión profesional de carteras con análisis avanzado.
```bash
sniperforge portfolio --mode professional --wallet wallet.json
sniperforge portfolio --show-analytics --timeframe 30d
sniperforge portfolio --rebalance --target-allocation balanced.json
```

**Modos disponibles:**
- `professional`: Dashboard completo con métricas avanzadas
- `simple`: Vista simplificada
- `analytics`: Solo análisis y métricas
- `rebalance`: Modo rebalanceo automático

**Ejemplo de salida (modo professional):**
```
📊 Portfolio Summary
═══════════════════
💰 Total Value: $15,847.32
📈 Total P&L: +$1,247.82 (+8.55%)
📊 Daily P&L: +$127.45 (+0.81%)
📉 Max Drawdown: -2.34%
🎯 Win Rate: 67.8%
📊 Sharpe Ratio: 1.85

🎯 Strategy Allocations:
  • DCA: 35.2% ($5,578.25)
  • Momentum: 28.7% ($4,548.16)
  • Grid: 20.1% ($3,185.23)
  • Arbitrage: 16.0% ($2,535.68)

🏆 Top Performers (24h):
  • SOL: +5.67% ($3,247.89)
  • RAY: +3.21% ($1,895.44)
  • ORCA: +2.14% ($987.33)

📈 Risk Metrics:
  • Portfolio VaR (95%): -$234.56
  • Beta: 1.23
  • Correlation Risk: Low
```

---

## 🔄 COMANDOS EN DESARROLLO 🚧

### 📋 Strategy Management Commands (Priority)

#### `sniperforge strategy-run` 🚧
Ejecuta una estrategia específica con configuración personalizada.
```bash
# PLANIFICADO
sniperforge strategy-run --type dca --config dca.json
sniperforge strategy-run --type momentum --symbol SOL --amount 100
sniperforge strategy-run --type grid --price-range 150-200 --levels 10
```

#### `sniperforge order-create` 🚧
Crea órdenes avanzadas (stop-loss, take-profit, trailing).
```bash
# PLANIFICADO
sniperforge order-create --type stop-loss --token SOL --trigger 140
sniperforge order-create --type take-profit --token USDC --target 1.01
sniperforge order-create --type trailing-stop --token RAY --distance 5%
```

#### `sniperforge execution-optimize` 🚧
Optimiza parámetros de ejecución para trades.
```bash
# PLANIFICADO
sniperforge execution-optimize --trade-size 1000 --token USDC
sniperforge execution-optimize --route-analysis --dex all
sniperforge execution-optimize --mev-protection --priority high
```

### 🔧 Infrastructure Commands (Lower Priority)

#### `sniperforge monitor` 🚧
Monitoreo del sistema y componentes.
```bash
# PLANIFICADO
sniperforge monitor --component all --duration 1h
sniperforge monitor --rpc-health --alerts-on
sniperforge monitor --trading-performance --output metrics.json
```

#### `sniperforge alerts` 🚧
Configuración de alertas y notificaciones.
```bash
# PLANIFICADO
sniperforge alerts --configure --channel discord
sniperforge alerts --set-threshold pnl 5%
sniperforge alerts --test-notification
```

#### `sniperforge backup` 🚧
Gestión de backups y restauración.
```bash
# PLANIFICADO
sniperforge backup --wallet wallet.json --encrypt
sniperforge backup --portfolio-state --compress
sniperforge restore --backup-id 2025070301 --verify
```

#### `sniperforge performance-stats` 🚧
Estadísticas y optimización de rendimiento.
```bash
# PLANIFICADO
sniperforge performance-stats --optimize
sniperforge performance-stats --benchmark --duration 5m
sniperforge performance-stats --cache-analysis
```

---

## 📚 COMMAND CATEGORIES

### ✅ **PRODUCTION READY** (16 comandos)
- Core Platform (4 comandos)
- Wallet Management (2 comandos)
- Testing & Validation (3 comandos)
- Advanced Trading (4 comandos)
- Machine Learning (1 comando)
- Portfolio Management (2 comandos)

### 🚧 **IN DEVELOPMENT** (12 comandos)
- Strategy Management (3 comandos) - **HIGH PRIORITY**
- Infrastructure Monitoring (4 comandos) - **MEDIUM PRIORITY**
- Advanced Operations (5 comandos) - **LOW PRIORITY**

---

## 🎯 ROADMAP PRÓXIMOS COMANDOS

### **Esta Semana (High Priority)**
1. `sniperforge strategy-run` - Ejecución directa de estrategias
2. `sniperforge order-create` - Creación de órdenes avanzadas
3. `sniperforge execution-optimize` - Optimización de ejecución

### **Próximas 2 Semanas (Medium Priority)**
1. `sniperforge monitor` - Monitoreo básico del sistema
2. `sniperforge performance-stats` - Métricas de rendimiento

### **Futuro (Low Priority)**
1. `sniperforge alerts` - Sistema de alertas
2. `sniperforge backup` - Gestión de backups

---

## 💡 TIPS Y MEJORES PRÁCTICAS

### 🔒 Security Best Practices
- Siempre usar `--dry-run` primero para nuevas estrategias
- Verificar configuración con `sniperforge validate` antes de trading real
- Mantener wallets de prueba separadas para DevNet

### ⚡ Performance Tips
- Usar `--network devnet` para testing sin costos
- Configurar límites de exposición apropiados
- Monitorear métricas de rendimiento regularmente

### 📊 Analysis Recommendations
- Usar backtesting antes de implementar estrategias nuevas
- Revisar análisis de patrones para timing de entrada
- Mantener diversificación en estrategias de portfolio

---

> **🏆 STATUS**: 16 comandos production-ready, 12 comandos en desarrollo
> **🚀 NEXT**: Completar comandos de gestión de estrategias (strategy-run, order-create, execution-optimize)
