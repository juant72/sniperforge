# Errores Principales a Corregir

## 1. BotStatus::Error pattern matching
- Cambiar `BotStatus::Error` por `BotStatus::Error(_)` en pattern matching

## 2. Campos faltantes en estructuras de simulación
- TradeResult necesita usar solo los campos definidos
- PoolInfo necesita usar solo los campos definidos
- PriceData necesita usar solo los campos definidos

## 3. Import no utilizados
- Eliminar imports no utilizados para limpiar warnings

## 4. Problemas de tipos
- NATIVE_MINT no existe en solana_sdk::native_token
- metadata variable no definida

## 5. Problemas de ownership
- BotStatus no implementa Copy, necesita clone()

## 6. Campos incorrectos en estructuras
- HealthStatus ya corregido
- resource_pools vs pools en ResourceCoordinator
