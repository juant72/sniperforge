# 📚 SNIPERFORGE CLI - GUÍA COMPLETA DE COMANDOS

**Versión**: 0.1.0 (Sprint 1 + Fases 6A/6B/6C + Tatum Integration + Cache-Free Trading)  
**Fecha**: Junio 29, 2025  
**Estado**: Sprint 1 Completado ✅ + Comandos Avanzados ML/Portfolio + RPC Premium 100% ✅ + Cache-Free Trading 🛡️

## 🔥 **CAMBIO CRÍTICO DE SEGURIDAD**

**⚠️ NUEVO REQUISITO: TODOS LOS COMANDOS PRINCIPALES REQUIEREN `--network`**

A partir de esta versión, **NO HAY VALORES POR DEFECTO** para la red. Debes especificar explícitamente:
- `--network devnet` - Para pruebas en DevNet
- `--network mainnet` - Para operaciones en Mainnet (DINERO REAL)

**Esto previene ejecuciones accidentales en la red incorrecta.**

## 🌟 **NUEVOS COMANDOS - INTEGRACIÓN TATUM (Junio 29, 2025)**

### RPC Testing Comprehensivo
```bash
# Test completo de todos los métodos RPC
cargo run --bin test_all_rpc_methods

# Test específico de Tatum
cargo run --bin sniperforge -- test tatum

# Test básico con todos los endpoints
cargo run --bin sniperforge -- test basic --network devnet
cargo run --bin sniperforge -- test basic --network mainnet
```

**Estado**: ✅ **100% FUNCIONAL** - Todos los endpoints RPC verificados

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

### 🌟 **NUEVOS TESTS RPC - TATUM INTEGRATION (June 29, 2025)**

#### `test_all_rpc_methods` - Test Comprehensivo de RPC
```bash
cargo run --bin test_all_rpc_methods
```

**Descripción**: Test exhaustivo de todos los métodos RPC en ambas redes (devnet y mainnet)
**Estado**: ✅ **100% Funcional** - Todos los endpoints verificados

**Métodos Testados**:
- `getSlot` - Obtener slot actual
- `getLatestBlockhash` - Obtener último blockhash
- `getAccountInfo` - Información de cuentas
- `getBalance` - Balance de cuentas

**Resultados Esperados**:
- **Devnet**: 4/4 tests passed (100% success rate)
- **Mainnet**: 4/4 tests passed (100% success rate)

#### `test tatum` - Test Específico de Tatum
```bash
cargo run --bin sniperforge -- test tatum
```

**Descripción**: Test dedicado para endpoints Tatum con autenticación de header
**Estado**: ✅ **100% Funcional** - Header authentication working

**Características**:
- Autenticación con `x-api-key` header
- Test de mainnet y devnet por separado
- Verificación de conectividad específica de Tatum

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
**Estado**: ✅ **Actualizado** - Incluye Tatum endpoints

**Incluye**:
- Conectividad Solana RPC
- Jupiter API integration
- WebSocket connectivity
- DexScreener API integration
- **Tatum RPC integration** (Nuevo)

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
**Estado**: ✅ **Mejorado** - Sin errores falsos

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

### `test cache-free-trading` - 🛡️ CACHE-FREE TRADING ENGINE
```bash
cargo run --bin sniperforge test cache-free-trading --network <NETWORK> [OPCIONES]
```

**Descripción**: **TRADING ENGINE ULTRA-SEGURO** - Sistema de trading sin caché para máxima precisión de precios

**Parámetros Obligatorios**:
- `--network <NETWORK>` - Red a usar: `devnet` o `mainnet`

**Opciones**:
- `-w, --wallet <FILE>` - **NUEVO** - Archivo de wallet para integración real (opcional)

**🔥 MODOS DE OPERACIÓN**:

**1. Modo Simulación (Por defecto)**:
```bash
# Testing básico sin wallet (ultra-seguro)
cargo run --bin sniperforge test cache-free-trading --network devnet
cargo run --bin sniperforge test cache-free-trading --network mainnet
```

**2. Modo Wallet Real (Nuevo)**:
```bash
# Testing con wallet real DevNet
cargo run --bin sniperforge test cache-free-trading --network devnet --wallet test-wallet-new.json

# Testing con wallet real Mainnet (¡CUIDADO!)
cargo run --bin sniperforge test cache-free-trading --network mainnet --wallet mainnet-wallet.json
```

**🛡️ CARACTERÍSTICAS ÚNICAS**:
- **Cache COMPLETAMENTE deshabilitado** - datos ultra-frescos (< 50ms)
- **Validación de precio en tiempo real** - múltiples fuentes
- **Límites ultra-conservadores en DevNet** - máximo $0.10 USD
- **Detección de staleness** - rechaza datos antiguos
- **Integración de wallet opcional** - sin crear comandos duplicados

**⚠️ MEDIDAS DE SEGURIDAD ESPECÍFICAS**:
- **DevNet**: Máximo $0.10 USD por trade, mínimo $0.01 USD profit
- **MainNet**: Configuración de producción con límites apropiados
- **Price Staleness**: Rechaza precios > 50ms de antigüedad
- **Real Balance Check**: Verificación de fondos reales si se usa wallet
- **Error Handling**: Manejo robusto de fallos de API

**Ejemplos**:
```bash
# Testing básico cache-free (modo demo)
cargo run --bin sniperforge test cache-free-trading --network devnet

# Integración real de wallet DevNet
cargo run --bin sniperforge test cache-free-trading --network devnet --wallet test-wallet-new.json

# Verificación Mainnet con wallet real
cargo run --bin sniperforge test cache-free-trading --network mainnet --wallet mainnet-validation-wallet.json
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

### `arbitrage-execute` - Ejecución de Arbitraje Real
```bash
cargo run --bin sniperforge arbitrage-execute --wallet <WALLET_FILE> --network <NETWORK> --amount <SOL_AMOUNT> [OPCIONES]
```

**Descripción**: Ejecuta arbitraje real usando precios en vivo de Jupiter y Orca

**⚠️ IMPORTANTE**: Este comando ejecuta transacciones REALES en blockchain y puede resultar en pérdida de fondos

**Parámetros Obligatorios**:
- `--wallet <WALLET_FILE>` - Archivo de wallet JSON (ej: test-wallet.json)
- `--network <NETWORK>` - Red para ejecución: `devnet` o `mainnet`
- `--amount <SOL_AMOUNT>` - Cantidad de SOL a utilizar (ej: 0.01)
- `--confirm` - Confirmación requerida para ejecutar transacciones reales

**Opciones**:
- `--max-slippage <PCT>` - Slippage máximo permitido (default: 1.0)
- `--min-profit <SOL>` - Ganancia mínima esperada en SOL (default: 0.001)

**Ejemplos**:
```bash
# Ejecución de prueba en DevNet (RECOMENDADO PARA EMPEZAR)
cargo run --bin sniperforge arbitrage-execute --wallet test-wallet.json --network devnet --amount 0.01 --confirm

# Ejecución con parámetros específicos
cargo run --bin sniperforge arbitrage-execute --wallet my-wallet.json --network devnet --amount 0.1 --max-slippage 0.5 --min-profit 0.005 --confirm

# Ejecución en Mainnet (DINERO REAL - SÉ CUIDADOSO)
cargo run --bin sniperforge arbitrage-execute --wallet production-wallet.json --network mainnet --amount 0.01 --confirm
```

**🛡️ Características de Seguridad**:
- ✅ Usa precios reales de Jupiter API ($163+ SOL)
- ✅ Integración con Orca Whirlpool SDK 
- ✅ Validación de transacciones antes de ejecutar
- ✅ Requiere confirmación explícita con `--confirm`
- ✅ Verifica balance antes de ejecutar
- ✅ Simula transacciones primero para detectar errores

**📊 Funcionamiento**:
1. Obtiene precios en tiempo real de Jupiter y Orca
2. Detecta oportunidades de arbitraje (ej: 64.57% spread DevNet)
3. Calcula ruta óptima: SOL → Token → SOL
4. Simula la transacción para verificar viabilidad
5. Ejecuta el arbitraje real si es rentable

**💡 Consejos**:
- Empieza siempre en DevNet con cantidades pequeñas (0.01 SOL)
- Verifica tu balance primero: `sniperforge wallet balance`
- El spread alto en DevNet es normal (precios mock vs reales)
- En Mainnet, los spreads reales son típicamente 0.1-2%

---

### `test arbitrage` - 🎯 TESTS DE ARBITRAJE ESPECÍFICOS
```bash
# Binarios especializados para testing de arbitraje
```

**Descripción**: Suite completa de tests para arbitraje usando binarios especializados

**🚀 Tests de Arbitraje Real con Precios En Vivo**:

#### `test_arbitrage_real_devnet` - Test Principal de Arbitraje
```bash
cargo run --bin test_arbitrage_real_devnet
```
**Descripción**: Test completo de arbitraje real usando Jupiter + Orca en DevNet
- ✅ Detecta tokens comerciables (USDC, RAY, BONK)
- ✅ Calcula rutas de arbitraje SOL → Token → SOL  
- ✅ Usa precios reales de Jupiter API
- ✅ Reporta ganancias/pérdidas reales
- ✅ Muestra spread entre Jupiter ($163) y Orca ($99 mock)

#### `test_arbitrage_execution_ready` - Test de Ejecución
```bash
cargo run --bin test_arbitrage_execution_ready
```
**Descripción**: Verifica que el sistema esté listo para ejecutar arbitraje real

#### `test_real_arbitrage_devnet` - Test de Datos Reales
```bash
cargo run --bin test_real_arbitrage_devnet
```
**Descripción**: Validación de arbitraje usando datos reales de mercado

**🔧 Utilities de Wallet y Balance**:

#### `create_test_wallet` - Crear Wallet de Prueba
```bash
cargo run --bin create_test_wallet
```
**Descripción**: Genera una nueva wallet de prueba para DevNet con clave privada

#### `request_devnet_airdrop` - Solicitar Airdrop
```bash
cargo run --bin request_devnet_airdrop
```
**Descripción**: Solicita airdrop de SOL en DevNet para la wallet configurada

#### `check_devnet_balance` - Verificar Balance DevNet
```bash
cargo run --bin check_devnet_balance
```
**Descripción**: Verifica el balance actual de la wallet en DevNet

**🌊 Tests de DEX y Tokens**:

#### `discover_jupiter_tokens` - Descubrir Tokens Jupiter
```bash
cargo run --bin discover_jupiter_tokens
```
**Descripción**: Descubre tokens disponibles en Jupiter API

#### `find_real_devnet_tokens` - Encontrar Tokens DevNet
```bash
cargo run --bin find_real_devnet_tokens
```
**Descripción**: Encuentra tokens reales disponibles para trading en DevNet

**💡 Comandos Útiles para Arbitraje**:
```bash
# 1. Crear wallet de prueba
cargo run --bin create_test_wallet

# 2. Solicitar airdrop
cargo run --bin request_devnet_airdrop

# 3. Verificar balance
cargo run --bin check_devnet_balance

# 4. Probar arbitraje real
cargo run --bin test_arbitrage_real_devnet

# 5. Ejecutar arbitraje con CLI
cargo run --bin sniperforge -- arbitrage-execute --wallet test-arbitrage-wallet.json --network devnet --amount 0.01 --confirm
```

**📊 Ejemplo de Output de test_arbitrage_real_devnet**:
```
✅ USDC-MainNet: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v (Output: 164479)
✅ RAY-MainNet: 4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R (Output: 56204)
✅ BONK-MainNet: DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263 (Output: 550355140)

📊 Resultado del arbitraje:
   SOL inicial: 10000000 lamports
   SOL final:   9993540 lamports
   💸 PÉRDIDA: 6460 lamports (0.000006 SOL)
```

---
