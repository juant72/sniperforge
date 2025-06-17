# Arquitectura de SniperForge

## 🏗 Visión General

SniperForge utiliza una arquitectura modular y escalable que permite el desarrollo de múltiples bots especializados mientras mantiene un código base común y reutilizable.

## 📐 Principios de Diseño

### 1. Modularidad
- Cada bot es un binario independiente
- Código compartido en la carpeta `shared/`
- Separación clara de responsabilidades

### 2. Escalabilidad
- Fácil adición de nuevos bots
- Arquitectura asíncrona para alta concurrencia
- Optimización para recursos limitados

### 3. Reutilización
- Componentes comunes para todos los bots
- Abstracciones para diferentes blockchains
- Patrones de diseño consistentes

## 🗂 Estructura de Carpetas Detallada

```text
sniperforge/
├── bots/                          # Bots específicos
│   ├── raydium-lp-sniper/
│   │   ├── src/
│   │   │   ├── main.rs            # Punto de entrada
│   │   │   ├── detector.rs        # Detección de pools
│   │   │   ├── extractor.rs       # Extracción de datos
│   │   │   ├── filter.rs          # Filtros de señales
│   │   │   ├── scorer.rs          # Sistema de scoring
│   │   │   ├── executor.rs        # Ejecución de trades
│   │   │   ├── exit_monitor.rs    # Monitoreo de salidas
│   │   │   ├── logger.rs          # Logging específico
│   │   │   └── risk_engine.rs     # Gestión de riesgo
│   │   ├── config/
│   │   │   └── default.toml       # Configuración por defecto
│   │   ├── tests/
│   │   └── Cargo.toml
│   ├── jupiter-arbitrage/         # Bot de arbitraje (futuro)
│   └── orca-whirlpool/           # Bot para Orca (futuro)
│
├── shared/                        # Código compartido
│   ├── solana-core/              # Funcionalidades core de Solana
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── client.rs         # Cliente RPC optimizado
│   │   │   ├── account_parser.rs # Parser de cuentas
│   │   │   ├── transaction.rs    # Helpers de transacciones
│   │   │   └── subscription.rs   # Manejo de subscripciones
│   │   └── Cargo.toml
│   │
│   ├── trading-engine/           # Motor de trading común
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── order_manager.rs  # Gestión de órdenes
│   │   │   ├── position.rs       # Gestión de posiciones
│   │   │   ├── portfolio.rs      # Gestión de portfolio
│   │   │   └── execution.rs      # Lógica de ejecución
│   │   └── Cargo.toml
│   │
│   ├── risk-management/          # Sistema de gestión de riesgo
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── position_sizing.rs # Cálculo de tamaño de posición
│   │   │   ├── risk_metrics.rs   # Métricas de riesgo
│   │   │   ├── stop_loss.rs      # Lógica de stop loss
│   │   │   └── exposure.rs       # Control de exposición
│   │   └── Cargo.toml
│   │
│   ├── data-providers/           # Proveedores de datos
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── raydium.rs        # API de Raydium
│   │   │   ├── jupiter.rs        # API de Jupiter
│   │   │   ├── orca.rs           # API de Orca
│   │   │   └── coingecko.rs      # Precios de CoinGecko
│   │   └── Cargo.toml
│   │
│   └── utils/                    # Utilidades comunes
│       ├── src/
│       │   ├── lib.rs
│       │   ├── config.rs         # Gestión de configuración
│       │   ├── logger.rs         # Sistema de logging
│       │   ├── metrics.rs        # Métricas y monitoreo
│       │   ├── encryption.rs     # Manejo de claves privadas
│       │   └── math.rs           # Funciones matemáticas
│       └── Cargo.toml
│
├── docs/                         # Documentación
│   ├── bots/                     # Docs específicas de bots
│   ├── architecture/             # Documentación de arquitectura
│   ├── deployment/               # Guías de despliegue
│   └── api/                      # Documentación de APIs
│
├── config/                       # Configuraciones
│   ├── global.toml              # Configuración global
│   └── bots/                    # Configs específicas por bot
│       ├── raydium-lp-sniper.toml
│       └── jupiter-arbitrage.toml
│
├── scripts/                      # Scripts de automatización
│   ├── build.sh                 # Script de build
│   ├── deploy.sh                # Script de despliegue
│   └── test.sh                  # Script de testing
│
├── Cargo.toml                   # Workspace principal
├── Cargo.lock
├── Dockerfile                   # Imagen de contenedor
├── docker-compose.yml           # Orquestación local
└── README.md                    # Documentación principal
```text

## 🔄 Flujo de Datos

### 1. Inicialización
```text
main.rs → config loading → shared components initialization → bot-specific setup
```text

### 2. Operación Normal
```text
data-providers → detector → extractor → filter → scorer → executor → exit_monitor
                     ↓
                risk-management (en cada paso)
                     ↓
                logging/metrics
```text

### 3. Gestión de Errores
```text
error → logger → risk-management → recovery/shutdown
```text

## 🧩 Componentes Compartidos

### solana-core
**Propósito**: Funcionalidades fundamentales para interactuar con Solana

**Funcionalidades**:
- Cliente RPC optimizado con pool de conexiones
- Parser eficiente de cuentas Solana
- Helpers para construcción de transacciones
- Manejo de subscripciones WebSocket

### trading-engine
**Propósito**: Motor de trading común para todos los bots

**Funcionalidades**:
- Gestión de órdenes y posiciones
- Cálculo de slippage y gas fees
- Lógica de ejecución optimizada
- Tracking de portfolio

### risk-management
**Propósito**: Sistema centralizado de gestión de riesgo

**Funcionalidades**:
- Cálculo de tamaño de posición
- Control de exposición por asset/estrategia
- Stop loss dinámico
- Métricas de riesgo en tiempo real

### data-providers
**Propósito**: Abstracción para diferentes fuentes de datos

**Funcionalidades**:
- APIs de DEXs (Raydium, Jupiter, Orca)
- Feeds de precios externos
- Datos on-chain en tiempo real
- Cache inteligente de datos

### utils
**Propósito**: Utilidades comunes para todo el ecosistema

**Funcionalidades**:
- Sistema de configuración unificado
- Logging estructurado
- Métricas y monitoreo
- Manejo seguro de claves privadas

## 📡 Comunicación Entre Componentes

### Event Bus Pattern
```rust
// Ejemplo de eventos compartidos
pub enum MarketEvent {
    NewPool(PoolInfo),
    PriceUpdate(PriceData),
    PositionOpened(Position),
    PositionClosed(Position),
    RiskAlert(RiskLevel),
}
```text

### Shared State
```rust
// Estado compartido thread-safe
pub struct SharedState {
    pub portfolio: Arc<RwLock<Portfolio>>,
    pub risk_metrics: Arc<RwLock<RiskMetrics>>,
    pub market_data: Arc<RwLock<MarketData>>,
}
```text

## 🔧 Configuración Jerárquica

### Prioridad de Configuración
1. Variables de entorno
2. Argumentos de línea de comandos
3. Archivo de configuración específico del bot
4. Configuración global
5. Valores por defecto

### Ejemplo de Configuración
```toml
# global.toml
[solana]
rpc_url = "https://api.mainnet-beta.solana.com"
commitment = "confirmed"

[logging]
level = "info"
format = "json"

# bots/raydium-lp-sniper.toml
[detector]
polling_interval = 1000

[risk]
max_position_size = 0.02
```text

## 🚀 Patrones de Despliegue

### Desarrollo Local
- Cada bot como binario independiente
- Configuración local con devnet
- Logging en stdout

### Producción
- Contenedores Docker individuales
- Orquestación con docker-compose/k8s
- Logging centralizado
- Métricas con Prometheus
- Alertas automatizadas

## 📊 Monitoreo y Observabilidad

### Métricas Clave
- Latencia de detección
- Tasa de éxito de ejecución
- P&L por estrategia
- Uso de recursos
- Errores y excepciones

### Logging Estructurado
```rust
info!(
    target: "raydium_sniper",
    pool_address = %pool.address,
    token_symbol = %token.symbol,
    score = score,
    "New opportunity detected"
);
```text

## 🔮 Extensibilidad

### Adición de Nuevos Bots
1. Crear carpeta en `bots/`
2. Implementar traits comunes
3. Usar componentes compartidos
4. Configurar en workspace

### Nuevas Blockchains
1. Crear módulo en `shared/`
2. Implementar abstracciones comunes
3. Adaptar data-providers
4. Actualizar configuración

## 🔍 Validación de Expertos

Antes de proceder con la implementación completa, es crucial validar este diseño arquitectónico con expertos en diferentes áreas:

### Documentos de Validación

- **[Validación de Expertos](../expert-validation.md)**: Cuestionario detallado para panel de expertos
- **[Casos de Estudio y Benchmarks](../validation-benchmarks.md)**: Análisis de rendimiento y casos reales

### Áreas de Validación Críticas

1. **Arquitectura Blockchain**: Validar optimizaciones específicas de Solana
2. **Trading Algorítmico**: Revisar estrategias y gestión de riesgo
3. **Seguridad**: Auditar manejo de claves y protecciones
4. **Machine Learning**: Evaluar viabilidad del sistema de scoring
5. **DevOps**: Confirmar escalabilidad y monitoreo

### Métricas de Éxito

- Latencia total < 400ms (95% percentile)
- Win rate > 60% en trading
- Zero honeypot false negatives
- Uptime > 99.5%
- ROI > 300% anual

La implementación solo procederá después de validación exitosa por el panel de expertos.

Esta arquitectura garantiza que SniperForge pueda crecer de manera sostenible, manteniendo la calidad del código y facilitando el mantenimiento a largo plazo.
