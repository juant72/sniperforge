# SniperForge Enterprise Configuration v3.0

## 🏢 Enterprise System Configuration

**Professional Trading Platform Setup & Configuration Management**

### 🏗️ System Architecture

SniperForge Enterprise utilizes a modern client-server architecture with TCP communication:

- **Server Component**: `sniperforge.exe` (Port 8888)
- **Interactive Client**: `sniperforge_interactive.exe` 
- **CLI Interface**: `sniperforge_cli.exe`
- **Communication Protocol**: TCP JSON messaging
- **Configuration**: Centralized `config.json` management

## 📁 Enterprise Configuration: config.json

**SECURITY NOTE**: All credentials and sensitive parameters are externalized in `config.json` - NO hardcoding in source code for enterprise compliance.

### 🔧 Complete Configuration Structure

```json
{
  "api_credentials": {
    "helius": {
      "api_key": "TU_API_KEY_AQUI",
      "mainnet_url": "https://mainnet.helius-rpc.com",
      "websocket_url": "wss://mainnet.helius-rpc.com",
      "eclipse_url": "https://eclipse.helius-rpc.com"
    },
    "jupiter": {
      "api_url": "https://quote-api.jup.ag/v6"
    },
    "dexscreener": {
      "api_url": "https://api.dexscreener.com/latest"
    },
    "pyth": {
      "api_url": "https://hermes.pyth.network/api"
    }
  },
  "rate_limits": {
    "helius": 100,
    "jupiter": 2000,
    "dexscreener": 500,
    "pyth": 300
  },
  "timeouts": {
    "helius": 15,
  "enterprise_communication": {
    "tcp_port": 8888,
    "connection_timeout": 3,
    "localhost_only": true,
    "max_concurrent_clients": 10
  },
  "api_credentials": {
    "helius": {
      "api_key": "062bf3dd-23d4-4ffd-99fd-6e397ee59d6c",
      "mainnet_url": "https://mainnet.helius-rpc.com",
      "websocket_url": "wss://mainnet.helius-rpc.com",
      "eclipse_url": "https://eclipse.helius-rpc.com"
    },
    "jupiter": {
      "api_url": "https://quote-api.jup.ag/v6"
    },
    "dexscreener": {
      "api_url": "https://api.dexscreener.com/latest"
    },
    "pyth": {
      "api_url": "https://hermes.pyth.network/api"
    }
  },
  "rate_limits": {
    "helius": 100,
    "jupiter": 2000,
    "dexscreener": 500,
    "pyth": 300
  },
  "timeouts": {
    "helius": 15,
    "jupiter": 10,
    "dexscreener": 12,
    "pyth": 10
  },
  "fallback_prices": {
    "SOL": 180.0,
    "ETH": 3200.0,
    "USDC": 1.0,
    "USDT": 1.0,
    "WBTC": 65000.0
  },
  "trading": {
    "max_history_size": 1000,
    "bridge_fee_percentage": 0.003,
    "min_confidence_score": 0.6,
    "max_risk_score": 0.8,
    "optimal_trade_percentage": 0.25,
    "base_market_volatility": 0.15
  },
  "enterprise_interface": {
    "session_tracking": true,
    "professional_messaging": true,
    "white_label_ready": true,
    "utc_timestamps": true
  }
}
```

## 🏢 Enterprise Communication Settings

### **TCP Server Configuration**
- **Port**: 8888 (localhost binding for security)
- **Connection Timeout**: 3 seconds for responsive user experience
- **Security**: Localhost-only binding for network isolation
- **Concurrency**: Support for up to 10 concurrent client connections

### **Professional Interface Settings**
- **Session Tracking**: UTC timestamp tracking for all enterprise sessions
- **Professional Messaging**: Corporate-grade status and error reporting
- **White-Label Ready**: Enterprise branding suitable for partner deployment
- **Graceful Error Handling**: Non-alarming professional status indicators

## 🔑 API Credentials & Integration

### **Helius RPC Configuration** (Primary Blockchain Provider)
- **API Key**: `062bf3dd-23d4-4ffd-99fd-6e397ee59d6c`
- **Mainnet URL**: `https://mainnet.helius-rpc.com/?api-key={API_KEY}`
- **WebSocket URL**: `wss://mainnet.helius-rpc.com/?api-key={API_KEY}`
- **Eclipse URL**: `https://eclipse.helius-rpc.com/`

### **Trading API Endpoints**
- **Jupiter v6**: DEX aggregation and routing optimization
- **DexScreener**: Real-time market data and price discovery
- **Pyth Network**: High-frequency price feeds and oracle data

## ⚙️ Enterprise Performance Parameters

### **Rate Limiting (Enterprise-Grade)**
- **Helius**: 100ms intervals (optimized for high-frequency operations)
- **Jupiter**: 2000ms intervals (conservative for rate limit compliance)
- **DexScreener**: 500ms intervals (balanced performance/compliance)
- **Pyth Network**: 300ms intervals (real-time data optimization)

### **Connection Timeouts (Professional)**
- **Helius**: 15 seconds (blockchain operations)
- **Jupiter**: 10 seconds (DEX routing)
- **DexScreener**: 12 seconds (market data)
- **Pyth Network**: 10 seconds (price feeds)

### **Fallback Price Management**
Professional fallback pricing for enterprise continuity:
- **SOL**: $180.0 (Solana native token)
- **ETH**: $3200.0 (Ethereum bridge operations)
- **USDC**: $1.0 (USD Coin stablecoin)
- **USDT**: $1.0 (Tether stablecoin)
- etc.

#### Configuración de Trading
- `bridge_fee_percentage`: 0.3% fees de bridge
- `min_confidence_score`: 60% confianza mínima
- `max_risk_score`: 80% riesgo máximo
- `optimal_trade_percentage`: 25% del capital por trade
- `base_market_volatility`: 15% volatilidad base

### 🔄 Recarga Sin Reiniciar

El sistema puede recargar la configuración sin reiniciar:

```rust
// Recargar configuración
api_credentials.reload_from_file("config.json")?;
```

### 🛡️ Seguridad

- ✅ **NO hardcoding** de credenciales
- ✅ **Configuración externa** para fácil mantenimiento
- ✅ **Fallbacks** en caso de error de configuración
- ✅ **Validación** de credenciales al cargar

### 📊 Monitoreo

El sistema reporta el estado de configuración:

```
✅ Configuración cargada exitosamente desde: config.json
🔧 Inicializando MultiPriceFeeds con credenciales: Helius: ✅ CONFIGURADO | Jupiter: ✅ | DexScreener: ✅ | Pyth: ✅
```

### 🚨 Manejo de Errores

Si el archivo `config.json` falla:
1. El sistema usa configuración por defecto (SIN credenciales reales)
2. Se muestra advertencia en logs
3. Funcionalidad limitada hasta corregir configuración

### 📝 Ejemplo de Uso

```bash
# 1. Crear/editar config.json con tus credenciales
# 2. Ejecutar el sistema
cargo run --release

# El sistema automáticamente:
# - Carga config.json
# - Valida credenciales
# - Usa rate limits configurados
# - Aplica precios de fallback según necesidad
```

### 🔧 Personalización

Puedes modificar `config.json` para:
- Cambiar rate limits según tu plan de API
- Ajustar precios de fallback a precios actuales
- Modificar parámetros de trading
- Actualizar URLs de APIs
- Cambiar timeouts según tu conexión

**Todo sin recompilar el código!** 🎉
