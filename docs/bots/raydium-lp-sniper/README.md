# Raydium LP Sniper Bot

Bot especializado en detectar y ejecutar compras automáticas en nuevos pools de liquidez de Raydium en Solana.

## 🎯 Objetivo

Detectar automáticamente nuevos pools de liquidez en Raydium, analizar los tokens, y ejecutar compras estratégicas en los primeros momentos de vida del pool para maximizar oportunidades de ganancia.

## 🏗 Arquitectura del Bot

```
raydium-lp-sniper/
├── src/
│   ├── main.rs              # Punto de entrada principal
│   ├── detector.rs          # Detecta nuevos pools en Raydium
│   ├── extractor.rs         # Extrae metadatos del token
│   ├── filter.rs            # Filtra señales según criterios
│   ├── scorer.rs            # Calcula score de oportunidad
│   ├── executor.rs          # Ejecuta las compras
│   ├── exit_monitor.rs      # Gestiona estrategias de salida
│   ├── logger.rs            # Sistema de logging
│   └── risk_engine.rs       # Gestión de riesgo y capital
├── config/
│   └── raydium-config.toml  # Configuración específica
└── tests/                   # Tests unitarios e integración
```

## 📊 Flujo de Operación

### 1. Detección (detector.rs)
- Monitorea eventos de creación de pools en Raydium
- Filtra por programas específicos de Raydium
- Extrae información básica del pool

### 2. Extracción de Datos (extractor.rs)
- Obtiene metadatos del token (nombre, símbolo, supply)
- Analiza la liquidez inicial
- Verifica la legitimidad del token

### 3. Filtrado (filter.rs)
- Aplica filtros de seguridad:
  - Liquidez mínima
  - Supply del token
  - Verificación de honeypots
  - Blacklist de tokens conocidos

### 4. Scoring (scorer.rs)
- Calcula un score de oportunidad basado en:
  - Liquidez inicial vs supply
  - Tiempo desde creación
  - Volumen proyectado
  - Análisis de riesgo

### 5. Ejecución (executor.rs)
- Ejecuta swaps automáticos
- Gestiona slippage y gas fees
- Implementa estrategias de entrada

### 6. Monitoreo de Salida (exit_monitor.rs)
- Monitorea precio y volumen
- Ejecuta estrategias de salida:
  - Take profit
  - Stop loss
  - Trailing stop

## ⚙️ Configuración

### Parámetros Principales

```toml
[detector]
# Intervalo de polling para nuevos pools (ms)
polling_interval = 1000
# Programas de Raydium a monitorear
raydium_programs = [
    "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",  # Raydium AMM
]

[filter]
# Liquidez mínima en SOL
min_liquidity_sol = 10.0
# Supply máximo del token
max_token_supply = 1000000000
# Tiempo máximo desde creación del pool (segundos)
max_pool_age = 300

[risk]
# Porcentaje máximo del capital por trade
max_position_size_pct = 2.0
# Capital máximo en riesgo simultáneo
max_concurrent_positions = 5
# Stop loss por defecto
default_stop_loss_pct = 20.0

[execution]
# Slippage máximo aceptable
max_slippage_pct = 5.0
# Gas fee máximo (lamports)
max_gas_fee = 10000
# Tiempo máximo de espera para confirmación (ms)
confirmation_timeout = 30000
```

## 🔒 Gestión de Riesgo

### Controles Implementados

1. **Control de Capital**
   - Límite por posición (% del capital total)
   - Límite de posiciones concurrentes
   - Reserva de emergencia

2. **Filtros de Seguridad**
   - Verificación de honeypots
   - Análisis de legitimidad del token
   - Blacklist de tokens conocidos como scam

3. **Gestión de Posiciones**
   - Stop loss automático
   - Take profit parcial
   - Trailing stop dinámico

## 📈 Métricas y Logging

### Métricas Rastreadas
- Número de pools detectados
- Tasa de filtrado
- Éxito de ejecuciones
- P&L por posición
- Tiempo de respuesta

### Logs Generados
- Detección de nuevos pools
- Decisiones de filtrado
- Ejecuciones de trades
- Cambios de precio
- Eventos de salida

## 🧪 Testing

### Tests Unitarios
- Lógica de filtrado
- Cálculo de scores
- Gestión de riesgo

### Tests de Integración
- Conexión con Solana RPC
- Interacción con Raydium
- Simulación de escenarios completos

## 📋 Dependencias Específicas

```toml
[dependencies]
# Core Solana
solana-client = "1.18"
solana-sdk = "1.18"

# Raydium específico
raydium-amm = "0.1"

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Configuración
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"

# Análisis de datos
ta = "0.5"  # Technical analysis
```

## 🚀 Uso

### Ejecución Local
```bash
# Desde la raíz del proyecto
cargo run --bin raydium-lp-sniper

# Con configuración específica
cargo run --bin raydium-lp-sniper -- --config config/raydium-custom.toml
```

### Ejecución en Producción
```bash
# Build optimizado
cargo build --release --bin raydium-lp-sniper

# Ejecutar con logging
RUST_LOG=info ./target/release/raydium-lp-sniper
```

## 🔮 Roadmap

### Fase 1 (Actual)
- [x] Detección básica de pools
- [x] Filtros de seguridad fundamentales
- [ ] Sistema de scoring básico
- [ ] Ejecución automática

### Fase 2
- [ ] ML para scoring avanzado
- [ ] Análisis de sentimiento
- [ ] Optimización de gas fees
- [ ] Dashboard en tiempo real

### Fase 3
- [ ] Multi-DEX support
- [ ] Arbitraje automático
- [ ] Estrategias avanzadas de salida
- [ ] API para terceros

## ⚠️ Disclaimers

- **Riesgo Financiero**: Este bot involucra trading automatizado con riesgo de pérdida total del capital
- **Uso Responsable**: Asegúrate de entender completamente los riesgos antes de usar
- **Testing**: Siempre prueba en devnet/testnet antes de usar fondos reales
- **Regulaciones**: Verifica las regulaciones locales antes de usar
