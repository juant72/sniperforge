# SniperForge

Un ecosistema de bots de trading automatizado construido en Rust para múltiples blockchains, con enfoque inicial en Solana.

## 🎯 Visión General

SniperForge es una plataforma modular que permite desarrollar y ejecutar múltiples bots de trading especializados, compartiendo código común y manteniendo una arquitectura escalable.

## 📁 Estructura del Proyecto

```
sniperforge/
├── bots/
│   ├── raydium-lp-sniper/     # Bot para detectar nuevos pools en Raydium
│   ├── jupiter-arbitrage/     # Bot de arbitraje (futuro)
│   └── orca-whirlpool/        # Bot para Orca pools (futuro)
├── shared/
│   ├── solana-core/           # Funcionalidades core de Solana
│   ├── trading-engine/        # Motor de trading común
│   ├── risk-management/       # Sistema de gestión de riesgo
│   ├── data-providers/        # Proveedores de datos
│   └── utils/                 # Utilidades comunes
├── docs/
│   ├── bots/                  # Documentación específica de cada bot
│   ├── architecture/          # Documentación de arquitectura
│   └── deployment/            # Guías de despliegue
├── config/
│   ├── global.toml           # Configuración global
│   └── bots/                 # Configuraciones específicas por bot
└── scripts/                  # Scripts de automatización
```

## 🚀 Bots Disponibles

### 1. Raydium LP Sniper
- **Estado**: En desarrollo
- **Objetivo**: Detectar y comprar tokens en nuevos pools de liquidez de Raydium
- **Documentación**: [Ver docs](./bots/raydium-lp-sniper/)

### 2. Jupiter Arbitrage (Planificado)
- **Estado**: Planificado
- **Objetivo**: Bot de arbitraje entre diferentes DEXs usando Jupiter

### 3. Orca Whirlpool (Planificado)
- **Estado**: Planificado
- **Objetivo**: Trading en pools concentrados de Orca

## 🛠 Tecnologías

- **Lenguaje**: Rust (última versión estable)
- **Blockchain**: Solana
- **Dependencias principales**:
  - `solana-client`
  - `solana-sdk`
  - `anchor-client`
  - `tokio` (async runtime)
  - `serde` (serialización)
  - `clap` (CLI)

## 🏗 Arquitectura

La arquitectura de SniperForge está diseñada para:

1. **Modularidad**: Cada bot es independiente pero puede usar código compartido
2. **Escalabilidad**: Fácil adición de nuevos bots y funcionalidades
3. **Reutilización**: Código común en la carpeta `shared/`
4. **Mantenibilidad**: Separación clara de responsabilidades

## 📚 Documentación

- [Arquitectura General](./architecture/)
- [Bots Específicos](./bots/)
- [Guías de Despliegue](./deployment/)
- [Configuración](./configuration.md)

## 🚀 Inicio Rápido

```bash
# Clonar el repositorio
git clone <repository-url>
cd sniperforge

# Compilar todos los bots
cargo build --release

# Ejecutar el Raydium LP Sniper
cargo run --bin raydium-lp-sniper
```

## 📄 Licencia

[Especificar licencia]

---

**Nota**: Este proyecto está en desarrollo activo. La documentación se actualiza constantemente.
