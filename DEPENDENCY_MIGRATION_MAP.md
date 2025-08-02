# 🗺️ MAPA DE DEPENDENCIAS - MIGRACIÓN OLD-ROOT-ARCHIVE

**Fecha**: Agosto 2, 2025  
**Objetivo**: Determinar orden correcto de migración basado en dependencias profundas

---

## 🔍 **ANÁLISIS DE DEPENDENCIAS CRÍTICAS**

### **PROBLEMA IDENTIFICADO:**
El strategy framework está fallando porque necesita dependencias fundamentales que aún no están migradas:

```
ArbitrageStrategy 
├── NECESITA → ArbitrageEngine (✅ EXISTE)
├── NECESITA → TradingOpportunity (✅ AGREGADO) 
├── NECESITA → MarketData (✅ MEJORADO)
├── FALLA → create_dummy_for_testing() (❌ NO IMPLEMENTADO)
│
ArbitrageEngine
├── NECESITA → SimpleConfig (❌ REVISAR)
├── NECESITA → PriceFeedManager (❌ REVISAR) 
├── NECESITA → RiskManager (❌ REVISAR)
├── NECESITA → Wallet/Keypair (❌ REVISAR)
└── NECESITA → RpcClient (❌ REVISAR)
```

---

## 📊 **MAPA DE DEPENDENCIAS LAYER BY LAYER**

### **LAYER 0: TIPOS FUNDAMENTALES** ⭐⭐⭐⭐
```
PRIORIDAD CRÍTICA - Base de todo el sistema

old-root-archive/src/types.rs
├── BasicConfig, SimpleConfig
├── Token, TradingPair
├── ArbitrageOpportunity  
├── MarketData (básico)
└── Result types

STATUS: 🔄 PARCIALMENTE MIGRADO (faltan configs)
DESTINO: src/types/mod.rs (AMPLIAR)
```

### **LAYER 1: CONFIGURACIÓN** ⭐⭐⭐⭐
```
PRIORIDAD CRÍTICA - Sin config no arranca nada

old-root-archive/src/shared/config_loader.rs ✅ MIGRADO
old-root-archive/src/shared/network_config.rs ✅ MIGRADO

FALTANTES CRÍTICOS:
├── SimpleConfig structure
├── ArbitrageConfig 
├── TradingConfig
└── PerformanceConfig

STATUS: 🔄 PARCIAL - Loaders migrados, structs faltan
ACCIÓN: Migrar config structs a src/types/
```

### **LAYER 2: RPC & CONEXIONES** ⭐⭐⭐
```
PRIORIDAD ALTA - Conexión con blockchain

old-root-archive/src/shared/rpc_pool.rs (❌ NO MIGRADO)
old-root-archive/src/shared/premium_rpc_manager.rs (❌ NO MIGRADO)  
old-root-archive/src/shared/rpc_health_persistence.rs (❌ NO MIGRADO)

DEPENDENCIAS:
├── Solana RPC Client wrappers
├── Connection pooling
├── Health monitoring
└── Failover logic

STATUS: ❌ NO MIGRADO
DESTINO: src/apis/rpc/
```

### **LAYER 3: WALLET MANAGEMENT** ⭐⭐⭐
```
PRIORIDAD ALTA - Sin wallet no hay trading

old-root-archive/src/shared/wallet_manager.rs (❌ NO MIGRADO)
old-root-archive/src/shared/test_wallet_integration.rs (❌ NO MIGRADO)

DEPENDENCIAS:
├── Keypair management
├── Balance checking  
├── Transaction signing
└── Security features

STATUS: ❌ NO MIGRADO
DESTINO: src/security/wallet/
```

### **LAYER 4: PRICE FEEDS** ⭐⭐⭐
```
PRIORIDAD ALTA - Datos de mercado en tiempo real

old-root-archive/src/shared/data_feeds.rs (❌ NO MIGRADO)
old-root-archive/src/shared/websocket_price_feed.rs (❌ NO MIGRADO)
old-root-archive/src/shared/real_data_manager.rs (❌ NO MIGRADO)

COMPONENTE CLAVE:
PriceFeedManager - Required by ArbitrageEngine

STATUS: ❌ NO MIGRADO  
DESTINO: src/trading/data_feeds/
```

### **LAYER 5: RISK MANAGEMENT** ⭐⭐⭐
```
PRIORIDAD ALTA - Control de riesgo

old-root-archive/src/shared/risk_manager.rs (❌ NO MIGRADO)

DEPENDENCIAS:
├── Position size calculation
├── Stop loss logic
├── Risk metrics
└── Portfolio limits

STATUS: ❌ NO MIGRADO
DESTINO: src/trading/risk/
```

### **LAYER 6: TRADING ENGINE CORE** ⭐⭐⭐
```
PRIORIDAD ALTA - Motor de trading base

old-root-archive/src/shared/real_trading_engine.rs (❌ NO MIGRADO)
old-root-archive/src/shared/trade_executor.rs (❌ NO MIGRADO)
old-root-archive/src/shared/real_trade_executor.rs (❌ NO MIGRADO)

DEPENDENCIAS: Layers 0-5 COMPLETOS

STATUS: ❌ NO MIGRADO
DESTINO: src/trading/execution/
```

### **LAYER 7: ARBITRAGE ENGINE** ⭐⭐
```
PRIORIDAD MEDIA - Motor arbitraje específico

ESTADO ACTUAL: ✅ EXISTE en src/trading/arbitrage.rs
PROBLEMA: Necesita dependencias de Layers 1-6

DEPENDENCIAS FALTANTES:
├── SimpleConfig (Layer 1)
├── PriceFeedManager (Layer 4)  
├── RiskManager (Layer 5)
├── RpcClient pools (Layer 2)
└── Wallet management (Layer 3)

STATUS: ✅ CÓDIGO MIGRADO, ❌ DEPENDENCIAS FALTANTES
```

### **LAYER 8: STRATEGY FRAMEWORK** ⭐
```
PRIORIDAD BAJA - Framework strategies

ESTADO ACTUAL: ✅ IMPLEMENTADO en src/trading/strategies/
PROBLEMA: Depende de ArbitrageEngine funcional (Layer 7)

STATUS: ✅ CÓDIGO COMPLETO, ❌ DEPENDENCIAS BROKEN
```

---

## 🎯 **PLAN DE MIGRACIÓN ORDENADO**

### **FASE PREPARATORIA: TYPES & CONFIG**
```bash
DURACIÓN: 1-2 horas
PRIORIDAD: ⭐⭐⭐⭐ CRÍTICA

1. Migrar config structs faltantes de old-root-archive/src/types.rs
   └── Agregar SimpleConfig, ArbitrageConfig, etc a src/types/mod.rs

2. Verificar y completar configuración básica
   └── Asegurar que config_loader funciona con nuevas structs
```

### **FASE 1: INFRAESTRUCTURA BASE** 
```bash
DURACIÓN: 2-3 horas  
PRIORIDAD: ⭐⭐⭐⭐ CRÍTICA

1. Migrar RPC Management (Layer 2)
   ├── src/apis/rpc/mod.rs ← old-root-archive/src/shared/rpc_pool.rs
   ├── src/apis/rpc/premium_manager.rs ← premium_rpc_manager.rs  
   └── src/apis/rpc/health.rs ← rpc_health_persistence.rs

2. Migrar Wallet Management (Layer 3)
   ├── src/security/wallet/mod.rs ← wallet_manager.rs
   └── src/security/wallet/integration.rs ← test_wallet_integration.rs

3. Migrar Data Feeds (Layer 4)
   ├── src/trading/data_feeds/mod.rs ← data_feeds.rs
   ├── src/trading/data_feeds/websocket.rs ← websocket_price_feed.rs
   └── src/trading/data_feeds/manager.rs ← real_data_manager.rs

4. Migrar Risk Management (Layer 5)
   └── src/trading/risk/mod.rs ← risk_manager.rs
```

### **FASE 2: TRADING CORE**
```bash
DURACIÓN: 2-3 horas
PRIORIDAD: ⭐⭐⭐ ALTA

1. Migrar Trading Execution (Layer 6)
   ├── src/trading/execution/mod.rs ← trade_executor.rs
   ├── src/trading/execution/real_executor.rs ← real_trade_executor.rs
   └── src/trading/execution/engine.rs ← real_trading_engine.rs

2. Corregir ArbitrageEngine dependencies (Layer 7)
   └── Actualizar imports y crear constructores funcionales
```

### **FASE 3: STRATEGY INTEGRATION**
```bash
DURACIÓN: 1-2 horas
PRIORIDAD: ⭐⭐ MEDIA

1. Corregir Strategy Framework (Layer 8)
   └── Hacer ArbitrageStrategy funcional con dependencias reales
```

---

## ⚡ **ACCIÓN INMEDIATA RECOMENDADA**

### **PASO 1: DIAGNÓSTICO DE TIPOS FALTANTES**
```bash
# Revisar qué configs están definidas en old-root-archive
grep -r "pub struct.*Config" old-root-archive/src/

# Revisar qué necesita ArbitrageEngine específicamente  
grep -r "SimpleConfig\|PriceFeedManager\|RiskManager" src/trading/arbitrage.rs
```

### **PASO 2: MIGRACIÓN MÍNIMA PARA COMPILAR**
```bash
# Crear stubs temporales en src/types/mod.rs para:
- SimpleConfig
- PriceFeedManager  
- RiskManager

# Esto permitirá que compile mientras migramos el resto
```

### **PASO 3: MIGRACIÓN ORDENADA**
```bash
# Ejecutar Fase Preparatoria + Fase 1 completamente
# Antes de intentar que strategies funcionen
```

---

## 🚨 **RIESGOS IDENTIFICADOS**

### **CIRCULAR DEPENDENCIES**
```
ArbitrageEngine → PriceFeedManager → WebSocketManager → ???
Trade Executor → Wallet Manager → Config → ???
Risk Manager → Trading Engine → Risk Manager (CIRCULAR!)
```

### **MISSING ABSTRACTIONS**
```
El código en old-root-archive puede tener:
- Hardcoded dependencies
- Singleton patterns
- Global state
- Non-async code that needs async conversion
```

### **VERSION CONFLICTS**
```
Dependencies en old-root-archive pueden usar:
- Versiones diferentes de solana_client
- Diferentes versions de tokio
- Incompatible WebSocket libraries
```

---

## 📋 **RECOMENDACIÓN ESTRATÉGICA**

### **ENFOQUE PRAGMÁTICO:**
1. **COMPILAR PRIMERO**: Crear stubs/dummies para que compile
2. **MIGRAR ORDENADO**: Seguir layers de dependencias estrictamente  
3. **VALIDAR INCREMENTALMENTE**: Cada layer debe funcionar antes del siguiente
4. **PRESERVAR FUNCIONALIDAD**: Mantener existing features mientras migramos

### **MÉTRICAS DE ÉXITO:**
- ✅ `cargo check` pasa en cada layer
- ✅ Tests básicos pasan en cada layer  
- ✅ No degradación de performance
- ✅ Todas las features existentes preservadas

---

**¿Procedemos con la Fase Preparatoria para crear los stubs de config necesarios?**
