# Configuración de SniperForge Enterprise

## 📁 Archivo de Configuración: config.json

**IMPORTANTE**: Todas las credenciales y parámetros se configuran en `config.json` - NO hay hardcoding en el código fuente.

### 🔧 Estructura de config.json

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
  }
}
```

### 🔑 Credenciales de Helius

Para usar las credenciales reales de Helius proporcionadas:

1. **API Key**: `062bf3dd-23d4-4ffd-99fd-6e397ee59d6c`
2. **Mainnet URL**: `https://mainnet.helius-rpc.com/?api-key={API_KEY}`
3. **WebSocket URL**: `wss://mainnet.helius-rpc.com/?api-key={API_KEY}`
4. **Eclipse URL**: `https://eclipse.helius-rpc.com/`

### ⚙️ Parámetros Configurables

#### Rate Limits (ms entre requests)
- `helius`: 100ms (rápido, Helius es generoso)
- `jupiter`: 2000ms (ultra-conservativo para evitar rate limiting)
- `dexscreener`: 500ms
- `pyth`: 300ms

#### Timeouts (segundos)
- `helius`: 15s
- `jupiter`: 10s
- `dexscreener`: 12s
- `pyth`: 10s

#### Precios de Fallback
Precios usados cuando todas las APIs fallan:
- `SOL`: $180.0
- `ETH`: $3200.0
- `USDC`: $1.0
- `USDT`: $1.0
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
