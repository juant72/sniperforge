# 📁 ESTRUCTURA ORGANIZACIONAL - SniperForge Enterprise v3.0

**Documento:** Organización y Arquitectura de Archivos  
**Fecha:** Agosto 4, 2025  
**Versión:** 1.0  
**Objetivo:** Estructura limpia y profesional del proyecto  

## 🗂️ ESTRUCTURA ACTUAL DEL PROYECTO

### **Directorio Raíz**
```
sniperforge/
├── 📁 src/                     # Código fuente principal
├── 📁 tests/                   # Suite de tests
├── 📁 docs/                    # Documentación técnica
├── 📁 config/                  # Archivos de configuración
├── 📁 tools/                   # Herramientas y scripts
├── 📁 data/                    # Datos y rutas optimizadas
├── 📁 logs/                    # Logs del sistema
├── 📁 benchmarks/              # Benchmarks de performance
├── 📁 old-root-archive/        # Archivo histórico (mantenido para referencia)
├── 📄 Cargo.toml               # Configuración Rust
├── 📄 Cargo.lock               # Lock file de dependencias
├── 📄 README.md                # Documentación principal
├── 📄 clippy.toml              # Configuración de linting
├── 📄 rustfmt.toml             # Configuración de formato
└── 🔧 Scripts de Build/Deploy  # Automatización
```

## 🏗️ ARQUITECTURA DE CÓDIGO FUENTE

### **src/ - Código Principal**
```
src/
├── 📄 lib.rs                   # Entry point y exports públicos
├── 📄 main.rs                  # Aplicación principal
│
├── 📁 api/                     # API Layer - Containerized Bot System
│   ├── 📄 mod.rs               # API module exports
│   ├── 📄 gateway.rs           # Actix-web API Gateway
│   ├── 📄 bot_interface.rs     # Core BotInterface trait
│   ├── 📄 config_management.rs # Dynamic configuration
│   ├── 📄 health_monitoring.rs # Health checks
│   └── 📄 metrics_collector.rs # Performance metrics
│
├── 📁 bots/                    # Bot Implementations
│   ├── 📄 mod.rs               # Bot factory and exports
│   ├── 📄 enhanced_arbitrage_bot.rs      # Advanced arbitrage
│   ├── 📄 triangular_arbitrage_bot.rs    # Triangular arbitrage
│   ├── 📄 ml_analytics_bot.rs            # ML-powered analytics
│   ├── 📄 portfolio_manager_bot.rs       # Portfolio management
│   ├── 📄 dashboard_bot.rs               # Dashboard interface
│   └── 📁 liquidity_sniper/              # Nuevo: Liquidity Sniper Bot
│       ├── 📄 mod.rs
│       ├── 📄 pool_detector.rs
│       ├── 📄 liquidity_analyzer.rs
│       ├── 📄 sniper_engine.rs
│       ├── 📄 mev_protection.rs
│       └── 📄 position_manager.rs
│
├── 📁 trading/                 # Trading Engine Core
│   ├── 📄 mod.rs               # Trading exports
│   ├── 📄 unified_routing.rs   # Unified routing system
│   ├── 📄 route_optimizer.rs   # Route optimization
│   ├── 📄 hft_engine.rs        # High-frequency trading
│   ├── 📄 execution_engine.rs  # Trade execution
│   └── 📄 real_time_monitor.rs # Real-time monitoring
│
├── 📁 apis/                    # External API Integrations
│   ├── 📄 mod.rs               # API integrations exports
│   ├── 📄 jupiter.rs           # Jupiter API v6
│   ├── 📄 dexscreener.rs      # DexScreener integration
│   ├── 📄 birdeye.rs          # Birdeye API
│   ├── 📄 helius.rs           # Helius RPC
│   └── 📄 price_feeds.rs      # Price feed aggregator
│
├── 📁 security/                # Security & Wallet Management
│   ├── 📄 mod.rs               # Security exports
│   ├── 📄 wallet_manager.rs    # Wallet operations
│   ├── 📄 encryption.rs        # Data encryption
│   ├── 📄 key_management.rs    # Key storage
│   └── 📄 transaction_signer.rs # Transaction signing
│
├── 📁 analytics/               # Performance Analytics
│   ├── 📄 mod.rs               # Analytics exports
│   ├── 📄 performance_tracker.rs # Performance monitoring
│   ├── 📄 profit_calculator.rs   # Profit calculations
│   ├── 📄 risk_analyzer.rs       # Risk assessment
│   └── 📄 reporting.rs           # Report generation
│
├── 📁 intelligence/            # AI & ML Systems
│   ├── 📄 mod.rs               # Intelligence exports
│   ├── 📄 market_predictor.rs  # Market prediction
│   ├── 📄 sentiment_analyzer.rs # Sentiment analysis
│   ├── 📄 pattern_detector.rs   # Pattern recognition
│   └── 📄 decision_engine.rs    # AI decision making
│
├── 📁 ml/                      # Machine Learning
│   ├── 📄 mod.rs               # ML exports
│   ├── 📄 models.rs            # ML model definitions
│   ├── 📄 training.rs          # Model training
│   ├── 📄 inference.rs         # Model inference
│   └── 📄 data_processor.rs    # Data preprocessing
│
├── 📁 monitoring/              # System Monitoring
│   ├── 📄 mod.rs               # Monitoring exports
│   ├── 📄 enterprise_monitor.rs # Enterprise monitoring
│   ├── 📄 alert_system.rs       # Alert management
│   ├── 📄 dashboard.rs          # Monitoring dashboard
│   └── 📄 metrics.rs            # Metrics collection
│
├── 📁 utils/                   # Utilities & Helpers
│   ├── 📄 mod.rs               # Utils exports
│   ├── 📄 config_loader.rs     # Configuration management
│   ├── 📄 logging.rs           # Enterprise logging
│   ├── 📄 validation.rs        # Input validation
│   ├── 📄 tatum_client.rs      # Tatum RPC client
│   └── 📄 helpers.rs           # Common utilities
│
├── 📁 types/                   # Type Definitions
│   ├── 📄 mod.rs               # Type exports
│   ├── 📄 trading_types.rs     # Trading-related types
│   ├── 📄 api_types.rs         # API response types
│   └── 📄 config_types.rs      # Configuration types
│
├── 📁 errors/                  # Error Handling
│   ├── 📄 mod.rs               # Error exports
│   ├── 📄 trading_errors.rs    # Trading-specific errors
│   ├── 📄 api_errors.rs        # API error handling
│   └── 📄 system_errors.rs     # System error types
│
└── 📁 config/                  # Configuration Management
    ├── 📄 mod.rs               # Config exports
    ├── 📄 app_config.rs        # Application configuration
    ├── 📄 trading_config.rs    # Trading parameters
    └── 📄 network_config.rs    # Network settings
```

## 🧪 ESTRUCTURA DE TESTS

### **tests/ - Testing Suite**
```
tests/
├── 📁 unit/                    # Unit tests
│   ├── 📄 mod.rs
│   ├── 📄 bot_interface_tests.rs
│   ├── 📄 trading_engine_tests.rs
│   ├── 📄 api_integration_tests.rs
│   └── 📄 utility_tests.rs
│
├── 📁 integration/             # Integration tests
│   ├── 📄 mod.rs
│   ├── 📄 full_system_tests.rs
│   ├── 📄 bot_coordination_tests.rs
│   ├── 📄 api_gateway_tests.rs
│   └── 📄 performance_tests.rs
│
├── 📁 performance/             # Performance tests
│   ├── 📄 mod.rs
│   ├── 📄 latency_tests.rs
│   ├── 📄 throughput_tests.rs
│   ├── 📄 memory_tests.rs
│   └── 📄 stress_tests.rs
│
├── 📁 e2e/                     # End-to-end tests
│   ├── 📄 mod.rs
│   ├── 📄 trading_scenarios.rs
│   ├── 📄 bot_lifecycle_tests.rs
│   └── 📄 system_reliability_tests.rs
│
└── 📁 fixtures/                # Test data and fixtures
    ├── 📄 mock_data.rs
    ├── 📄 test_configs.rs
    └── 📄 sample_responses.rs
```

## 📚 ESTRUCTURA DE DOCUMENTACIÓN

### **docs/ - Documentation**
```
docs/
├── 📄 README.md                # Documentación principal
├── 📄 PLAN_MEJORA_ARQUITECTURAL_2025.md    # Plan maestro de mejoras
├── 📄 PLAN_LIQUIDITY_SNIPER_BOT.md         # Plan específico del sniper bot
├── 📄 ORGANIZACION_ARCHIVOS.md             # Este documento
│
├── 📁 architecture/            # Documentación arquitectural
│   ├── 📄 system_overview.md   # Visión general del sistema
│   ├── 📄 bot_interface_spec.md # Especificación de interfaces
│   ├── 📄 api_documentation.md  # Documentación de APIs
│   ├── 📄 database_schema.md    # Esquemas de base de datos
│   └── 📄 security_model.md     # Modelo de seguridad
│
├── 📁 user_guides/             # Guías de usuario
│   ├── 📄 quick_start.md       # Guía de inicio rápido
│   ├── 📄 configuration.md     # Configuración del sistema
│   ├── 📄 bot_management.md    # Gestión de bots
│   ├── 📄 monitoring.md        # Monitoreo y alertas
│   └── 📄 troubleshooting.md   # Solución de problemas
│
├── 📁 development/             # Documentación de desarrollo
│   ├── 📄 contributing.md      # Guía de contribución
│   ├── 📄 coding_standards.md  # Estándares de código
│   ├── 📄 testing_guidelines.md # Guías de testing
│   ├── 📄 deployment.md        # Procedimientos de deployment
│   └── 📄 performance_tuning.md # Optimización de performance
│
├── 📁 api_reference/           # Referencia de APIs
│   ├── 📄 rest_api.md          # REST API endpoints
│   ├── 📄 websocket_api.md     # WebSocket API
│   ├── 📄 bot_api.md           # Bot Interface API
│   └── 📄 configuration_api.md # Configuration API
│
└── 📁 reports/                 # Reportes y análisis
    ├── 📄 performance_reports.md # Reportes de performance
    ├── 📄 security_audits.md     # Auditorías de seguridad
    ├── 📄 testing_reports.md     # Reportes de testing
    └── 📄 deployment_logs.md     # Logs de deployment
```

## ⚙️ CONFIGURACIÓN Y DATOS

### **config/ - Configuration Files**
```
config/
├── 📄 development.toml         # Configuración de desarrollo
├── 📄 production.toml          # Configuración de producción
├── 📄 testing.toml             # Configuración de testing
├── 📄 docker-compose.yml       # Docker composition
├── 📄 Dockerfile               # Docker container definition
└── 📁 templates/               # Templates de configuración
    ├── 📄 bot_config_template.json
    ├── 📄 trading_config_template.json
    └── 📄 network_config_template.json
```

### **data/ - Data Files**
```
data/
├── 📁 routes/                  # Rutas de trading optimizadas
│   ├── 📄 arbitrage_routes_optimized.json
│   ├── 📄 real_time_routes.json
│   └── 📄 historical_routes.json
│
├── 📁 market_data/             # Datos de mercado
│   ├── 📄 token_metadata.json
│   ├── 📄 pool_statistics.json
│   └── 📄 price_history.json
│
└── 📁 ml_models/               # Modelos de ML
    ├── 📄 sentiment_model.bin
    ├── 📄 price_prediction_model.bin
    └── 📄 risk_assessment_model.bin
```

## 🛠️ HERRAMIENTAS Y SCRIPTS

### **tools/ - Tools & Scripts**
```
tools/
├── 📄 dev.ps1                 # Script de desarrollo
├── 📄 test.ps1                # Script de testing
├── 📄 build-production.ps1    # Build de producción
├── 📄 deploy-enterprise.ps1   # Deployment empresarial
├── 📄 run_system.bat          # Ejecución del sistema
├── 📄 apply-best-practices.ps1 # Aplicación de mejores prácticas
└── 📁 monitoring/              # Herramientas de monitoreo
    ├── 📄 health_check.ps1
    ├── 📄 performance_monitor.ps1
    └── 📄 log_analyzer.ps1
```

## 📊 BENCHMARKS Y LOGS

### **benchmarks/ - Performance Benchmarks**
```
benchmarks/
├── 📄 performance_new.rs       # Nuevos benchmarks
├── 📄 performance.rs           # Benchmarks base
├── 📁 results/                 # Resultados de benchmarks
│   ├── 📄 latency_results.json
│   ├── 📄 throughput_results.json
│   └── 📄 memory_usage_results.json
└── 📁 scripts/                 # Scripts de benchmarking
    ├── 📄 run_benchmarks.ps1
    └── 📄 analyze_results.ps1
```

### **logs/ - System Logs**
```
logs/
├── 📄 application.log          # Logs de aplicación
├── 📄 trading.log              # Logs de trading
├── 📄 api.log                  # Logs de API
├── 📄 errors.log               # Logs de errores
└── 📁 archived/                # Logs archivados
    ├── 📄 application_2025_08_01.log
    ├── 📄 trading_2025_08_01.log
    └── 📄 api_2025_08_01.log
```

## 🎯 ARCHIVOS DE CONFIGURACIÓN PRINCIPAL

### **Cargo.toml - Dependencies**
```toml
[package]
name = "sniperforge"
version = "3.0.0"
edition = "2021"
authors = ["SniperForge Development Team"]
description = "Professional Solana DeFi Trading Bot Suite - Enterprise Edition"
license = "MIT"
repository = "https://github.com/juant72/sniperforge"

[dependencies]
# Core async runtime
tokio = { version = "1.0", features = ["full"] }
# Web framework for API
actix-web = "4.0"
# Solana SDK
solana-sdk = "1.16"
# Additional dependencies...

[dev-dependencies]
# Testing framework
criterion = "0.5"
tokio-test = "0.4"

[[bin]]
name = "sniperforge"
path = "src/main.rs"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

### **clippy.toml - Linting Rules**
```toml
# Clippy configuration for enterprise-grade code quality
avoid-breaking-exported-api = true
enum-variant-names-threshold = 3
single-char-lifetime-names-threshold = 4
trivial-bounds = true
type-repetition-threshold = 5
unnecessary-wraps = true
used-underscore-binding = true
cognitive-complexity-threshold = 30
```

### **rustfmt.toml - Code Formatting**
```toml
# Rust formatting configuration
max_width = 120
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
reorder_imports = true
reorder_modules = true
remove_nested_parens = true
edition = "2021"
```

## 📝 ARCHIVOS DE ESTADO Y METADATA

### **Archivos de Estado del Sistema**
- `📄 Cargo.lock` - Lock file de dependencias (auto-generado)
- `📄 .gitignore` - Archivos ignorados por Git
- `📄 .github/workflows/` - GitHub Actions CI/CD
- `📄 LICENSE` - Licencia del proyecto
- `📄 CHANGELOG.md` - Historial de cambios

### **Archivos de Build y Deploy**
- `📄 Dockerfile.enterprise` - Container empresarial
- `📄 docker-compose.enterprise.yml` - Orquestación de containers
- `📄 kubernetes/` - Manifiestos de Kubernetes (futuro)

## 🧹 ARCHIVOS ORGANIZADOS Y ARCHIVADOS

### **old-root-archive/** - Archivo Histórico
El directorio `old-root-archive/` contiene el código legacy y documentación histórica que se mantiene para:
- **Referencia técnica** de implementaciones anteriores
- **Documentación de arquitectura** de fases pasadas
- **Benchmarks históricos** para comparación
- **Patterns de desarrollo** que pueden ser reutilizados

Este directorio **NO** afecta la compilación actual y sirve como referencia histórica.

### **target/** - Build Artifacts**
- Generado automáticamente por Cargo
- Contiene binarios compilados y artifacts de build
- Incluido en `.gitignore`
- Se limpia automáticamente con `cargo clean`

## ✅ ESTADO DE ORGANIZACIÓN

### **Archivos Organizados** ✅
- [x] Código fuente estructurado por módulos
- [x] Tests organizados por tipo y funcionalidad
- [x] Documentación categorizada por audiencia
- [x] Configuraciones separadas por environment
- [x] Scripts de automatización centralizados
- [x] Logs estructurados y rotación automática
- [x] Benchmarks organizados con resultados

### **Mejoras Implementadas** ✅
- [x] Separación clara de responsabilidades
- [x] Estructura modular escalable
- [x] Documentación completa y actualizada
- [x] Testing suite comprehensiva
- [x] Configuración por environments
- [x] Automatización de tareas comunes
- [x] Archivado de código legacy

### **Beneficios de la Organización** ✅
- **Mantenibilidad:** Fácil navegación y comprensión
- **Escalabilidad:** Estructura preparada para crecimiento
- **Colaboración:** Estándares claros para el equipo
- **Debugging:** Logs y estructura facilitan troubleshooting
- **Testing:** Suite organizada y completa
- **Deployment:** Automatización y configuración clara

---

**Estado del Documento:** ✅ Completo  
**Última Actualización:** Agosto 4, 2025  
**Mantenido por:** SniperForge Development Team  
