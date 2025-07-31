# SniperForge - Ejecución Concurrente y Multi-Terminal

## Resumen

SniperForge está diseñado con una arquitectura **stateless** que permite la ejecución de múltiples instancias del comando `sniperforge` desde diferentes terminales de forma segura y concurrente.

## ¿Se puede ejecutar desde varias terminales?

**✅ SÍ** - Cada ejecución de `sniperforge` es un proceso independiente que puede ejecutarse simultáneamente desde múltiples terminales.

## Arquitectura de Procesos

### 1. CLI Stateless
- Cada proceso `sniperforge` es completamente independiente
- No hay estado compartido entre instancias
- Cada proceso carga su propia configuración y maneja sus propios recursos
- Los logs se escriben de forma segura con `tracing_appender::rolling`

### 2. Gestión de Recursos

#### Recursos NO Compartidos (seguros para concurrencia):
- **Configuración**: Cada proceso lee los archivos TOML independientemente
- **Logs**: Escritura concurrent-safe con rolling logs
- **RPC Connections**: Cada proceso crea sus propias conexiones
- **Wallets**: Auto-generación independiente por proceso

#### Recursos que Requieren Coordinación:
- **Archivos de wallet persistentes** (cuando se implementen)
- **Estado de trading activo** (para evitar órdenes duplicadas)
- **Limites de API rate** (compartidos entre procesos)

## Casos de Uso Recomendados

### ✅ Comandos Totalmente Seguros (Multi-Terminal)

```powershell
# Terminal 1: Monitoreo continuo
sniperforge interactive

# Terminal 2: Consultas de estado
sniperforge status
sniperforge config
sniperforge wallet balance

# Terminal 3: Testing y validaciones
sniperforge test solana
sniperforge test pools
sniperforge test wallets

# Terminal 4: Operaciones específicas
sniperforge wallet airdrop
```

### ⚠️ Comandos que Requieren Cuidado

```powershell
# ⚠️ EVITAR: Múltiples instancias de bots de trading
# Terminal 1
sniperforge start --bot lp-sniper --devnet

# Terminal 2 (CONFLICTO POTENCIAL)
sniperforge start --bot lp-sniper --devnet
```

**Problema**: Ambos procesos podrían:
- Competir por las mismas oportunidades de trading
- Generar órdenes duplicadas
- Agotar límites de RPC/API

### 🎯 Patrones Recomendados

#### Patrón 1: Monitoreo + Comandos
```powershell
# Terminal Principal: Dashboard en tiempo real
sniperforge interactive

# Terminales Auxiliares: Comandos específicos
sniperforge wallet balance
sniperforge test pools
```

#### Patrón 2: Testing Multi-Componente
```powershell
# Terminal 1: Test de conectividad
sniperforge test solana

# Terminal 2: Test de wallets  
sniperforge test wallets

# Terminal 3: Test de pools
sniperforge test pools
```

#### Patrón 3: Desarrollo y Debug
```powershell
# Terminal 1: Bot en modo desarrollo
sniperforge start --devnet --bot lp-sniper

# Terminal 2: Monitoreo de logs
tail -f logs/sniperforge.log.*

# Terminal 3: Tests y validaciones
sniperforge status
sniperforge wallet balance
```

## Consideraciones Técnicas

### Logging Concurrente
- ✅ **Seguro**: Utiliza `tracing_appender::rolling::daily`
- ✅ **No-blocking**: Implementa escritura asíncrona
- ✅ **Thread-safe**: Múltiples procesos pueden escribir simultáneamente

### Gestión de Archivos
- ✅ **Lectura de configuración**: Múltiples procesos pueden leer TOML files
- ⚠️ **Persistencia de wallets**: Aún no implementada (sin conflictos actuales)
- ⚠️ **Estado de trading**: Futuro requisito de coordinación

### Conexiones RPC
- ✅ **Independientes**: Cada proceso maneja sus propias conexiones
- ⚠️ **Rate limiting**: Compartido entre procesos (límite de RPC provider)
- 💡 **Pool de conexiones**: Futuro mejoramiento para eficiencia

## Estado Actual vs. Futuro

### ✅ Actualmente Soportado
- Múltiples comandos CLI simultáneos
- Comandos de consulta (status, config, balance)
- Tests independientes
- Modo interactivo en múltiples terminales
- Generación independiente de wallets

### 🔧 Mejoras Futuras Necesarias
- **Persistencia de wallets**: Coordinación para reutilizar wallets
- **Trading coordination**: Prevenir órdenes duplicadas
- **Shared state management**: Para bots que requieren coordinación
- **Resource pooling**: Optimización de conexiones RPC

## Recomendaciones de Uso

### Para Desarrollo
```powershell
# Setup recomendado para desarrollo
Terminal 1: sniperforge interactive       # Dashboard principal
Terminal 2: sniperforge test wallets      # Testing continuo  
Terminal 3: código/logs                   # VS Code + logs
```

### Para Testing
```powershell
# Testing integral
Terminal 1: sniperforge start --devnet    # Bot en devnet
Terminal 2: sniperforge wallet balance    # Monitoreo de balances
Terminal 3: sniperforge test pools        # Validación de pools
```

### Para Producción
```powershell
# Producción (futuro)
Terminal 1: sniperforge start --mainnet   # Bot principal
Terminal 2: sniperforge interactive       # Monitoreo
Terminal 3: Reserved for emergency commands
```

## Conclusión

La arquitectura actual de SniperForge **permite y soporta** la ejecución concurrente desde múltiples terminales de forma segura para la mayoría de casos de uso. Los comandos de consulta, testing y monitoreo son completamente seguros para uso concurrente.

Las únicas consideraciones especiales son para comandos de trading activo, donde la coordinación será necesaria en futuras versiones para prevenir conflictos de órdenes duplicadas.
