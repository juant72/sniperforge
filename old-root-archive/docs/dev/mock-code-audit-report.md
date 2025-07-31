# REPORTE DE CÓDIGO MOCK/SIMULADO ENCONTRADO

## ARCHIVOS CON CÓDIGO PROBLEMÁTICO IDENTIFICADO:

### 1. `src/shared/pool_detector.rs` - CRÍTICO
**Funciones que generan datos falsos:**
- `generate_realistic_new_pool()` - Línea 525
- `generate_realistic_token_pair()` - Línea 1614  
- `fetch_real_raydium_pools()` - Usa generación falsa en lugar de API real

**Problema:** En lugar de usar APIs reales de Raydium/Orca, está generando pools falsos con datos simulados.

### 2. `src/shared/syndica_websocket.rs` - CRÍTICO
**Funciones problemáticas:**
- `start_price_simulation()` - Línea 123
- Genera precios sintéticos en lugar de usar WebSocket real
- `SyndicaSynthetic` enum usado para generar datos falsos

**Problema:** Simula precios en lugar de usar datos reales del WebSocket de Syndica.

### 3. `src/shared/cache_free_trader_simple.rs` - MENOR
**Funciones problemáticas:**
- `get_fresh_price_no_cache()` - Línea 66
- Retorna precio placeholder de 180.0 en lugar de llamar APIs reales

### 4. `src/shared/wallet_manager.rs` - PAPEL TRADING
**Funciones problemáticas:**
- Código de "virtual wallet" para paper trading
- Comentarios sobre "virtual/paper trading"

### 5. Configuración con campos `virtual_balance_sol`
**Archivos:**
- `src/config.rs` - Líneas 175, 419, 427

## ACCIONES REQUERIDAS:

### ALTA PRIORIDAD:
1. **Eliminar completamente** `generate_realistic_new_pool()` y funciones relacionadas
2. **Eliminar** `start_price_simulation()` y forzar uso de WebSocket real
3. **Reemplazar** placeholders en `cache_free_trader_simple.rs` con APIs reales

### MEDIA PRIORIDAD:
4. **Limpiar** configuración virtual en `wallet_manager.rs`
5. **Evaluar** si `virtual_balance_sol` es necesario o se puede eliminar

### VALIDACIÓN:
6. **Verificar** que no queden referencias a datos sintéticos/falsos
7. **Asegurar** que todas las funciones usen únicamente APIs reales

## RIESGO:
- **ALTO**: Los pools generados falsos podrían causar trades en direcciones inexistentes
- **ALTO**: Los precios simulados no reflejan mercado real
- **MEDIO**: Configuración virtual podría confundir modos de operación
