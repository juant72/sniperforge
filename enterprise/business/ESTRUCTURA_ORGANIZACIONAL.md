# SniperForge Enterprise - Estructura Organizacional

## 📁 Estructura de Directorios

Esta es la estructura organizacional empresarial de SniperForge, diseñada para mantener un directorio raíz limpio y una organización profesional de todos los componentes.

```
sniperforge/
├── 📋 ARCHIVOS PRINCIPALES (solo esenciales)
│   ├── README.md                    # Este archivo
│   ├── Cargo.toml                   # Configuración Rust
│   ├── Cargo.lock                   # Lock de dependencias
│   ├── wallet.json                  # Wallet principal activo
│   ├── config.json                  # Configuración principal
│   ├── docker-compose.yml           # Docker para desarrollo
│   ├── rustfmt.toml                 # Formato de código
│   ├── .env*                        # Variables de entorno
│   └── .gitignore                   # Git ignore
│
├── 🏢 enterprise/                   # COMPONENTES EMPRESARIALES
│   ├── business/                    # Documentos de negocio
│   ├── deployment/                  # Archivos de deployment
│   ├── reports/                     # Reportes empresariales
│   ├── security/                    # Documentos de seguridad
│   └── phases/                      # Fases de desarrollo
│
├── 🛠️ development/                  # HERRAMIENTAS DE DESARROLLO
│   ├── scripts/                     # Scripts PowerShell/Bash
│   ├── testing/                     # Archivos de testing
│   ├── analysis/                    # Análisis de código
│   ├── logs/                        # Logs de desarrollo
│   └── [archivos de desarrollo]     # Planes, migraciones, etc.
│
├── 📚 docs/                         # DOCUMENTACIÓN COMPLETA
│   └── enterprise/                  # Documentación API empresarial
│       ├── api/                     # 11 módulos documentados
│       ├── guides/                  # Guías de uso
│       └── examples/                # Ejemplos de código
│
├── 💾 legacy/                       # ARCHIVOS LEGACY
│   └── wallets/                     # Wallets antiguos/comprometidos
│
├── 🔧 src/                          # CÓDIGO FUENTE
│   ├── apis/                        # Integración APIs externas
│   ├── bots/                        # Bots de trading
│   ├── config/                      # Gestión configuración
│   ├── control/                     # Control de bots
│   ├── intelligence/                # Sistema IA
│   ├── ml/                          # Machine Learning
│   ├── monitoring/                  # Monitoreo empresarial
│   ├── security/                    # Seguridad
│   ├── trading/                     # Motores de trading
│   └── [otros módulos]
│
└── 🗂️ [OTROS DIRECTORIOS ESTÁNDAR]
    ├── target/                      # Compilación Rust
    ├── tests/                       # Tests automatizados
    ├── examples/                    # Ejemplos de uso
    ├── benchmarks/                  # Benchmarks
    ├── tools/                       # Herramientas auxiliares
    ├── k8s/                         # Kubernetes
    ├── docker/                      # Docker configs
    └── [otros]
```

## 🎯 Filosofía Organizacional

### ✅ **Directorio Raíz Limpio**
Solo contiene archivos esenciales para la operación:
- Configuración principal del proyecto
- Archivos de build (Cargo.toml, etc.)
- Wallet activo principal
- Docker compose básico
- Variables de entorno

### 🏢 **Enterprise Structure**
Separación clara entre:
- **Negocio**: Documentos empresariales y ROI
- **Deployment**: Archivos de producción
- **Reportes**: Informes de progreso y certificaciones
- **Seguridad**: Documentación de seguridad

### 🛠️ **Development Tools**
Herramientas de desarrollo organizadas:
- **Scripts**: Automatización PowerShell/Bash
- **Testing**: Estados de bots y pruebas
- **Analysis**: Análisis de código y performance
- **Logs**: Archivos de log y debug

### 💾 **Legacy Management**
Archivos históricos o comprometidos:
- Wallets antiguos
- Configuraciones obsoletas
- Archivos de migración completados

## 📋 **Archivos Reubicados**

### Desde Raíz → Enterprise
- `CERTIFICACION_FINAL_COMPLETA.md` → `enterprise/reports/`
- `DASHBOARD_EMPRESARIAL_COMPLETADO.md` → `enterprise/reports/`
- `SISTEMA_100_OPERATIVO_FINAL.md` → `enterprise/reports/`
- `docker-compose.enterprise.yml` → `enterprise/deployment/`
- `Dockerfile.enterprise` → `enterprise/deployment/`

### Desde Raíz → Development
- `CONTEXTO_COMPLETO_DESARROLLO.md` → `development/`
- `COPILOT_GUIDELINES.md` → `development/`
- `PLAN_*.md` → `development/`
- `test_real_sentiment.rs` → `development/testing/`
- `BOT_SNIPER_*.md` → `development/testing/`

### Desde Raíz → Legacy
- `test_wallet.json` → `legacy/wallets/`
- `wallet_secure_*.json` → `legacy/wallets/`
- Wallets comprometidos → `legacy/wallets/`

## 🚀 **Beneficios de la Nueva Estructura**

1. **✅ Directorio raíz limpio** - Solo archivos esenciales
2. **🏢 Organización empresarial** - Separación clara de concerns
3. **🔍 Fácil navegación** - Estructura lógica y predecible
4. **📚 Documentación centralizada** - Docs enterprise completa
5. **🛠️ Herramientas organizadas** - Dev tools en su lugar
6. **🔒 Seguridad mejorada** - Legacy segregado
7. **📈 Escalabilidad** - Preparado para crecimiento

## 📞 **Uso de la Estructura**

### Para Desarrolladores
```bash
# Código fuente
cd src/

# Herramientas de desarrollo
cd development/

# Documentación
cd docs/enterprise/
```

### Para DevOps
```bash
# Deployment empresarial
cd enterprise/deployment/

# Monitoreo
cd src/monitoring/

# Configuración
cd config/
```

### Para Business
```bash
# Documentos empresariales
cd enterprise/business/

# Reportes
cd enterprise/reports/

# Documentación API
cd docs/enterprise/
```

---

**Versión**: 2.0.0  
**Fecha Reorganización**: 2025-01-08  
**Status**: ✅ ESTRUCTURA EMPRESARIAL COMPLETA
