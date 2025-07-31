# VS Code Crash Prevention Guide

## Overview

Este documento describe las causas comunes de crashes de VS Code en el proyecto SniperForge y las herramientas disponibles para prevenirlos y diagnosticarlos.

## Causas Comunes de Crashes

### 1. **Uso Excesivo de Memoria**
- **Umbral crítico**: >3GB de uso de memoria
- **Causas**: Rust-analyzer, múltiples extensiones, workspace grande
- **Síntomas**: VS Code se congela, cierre inesperado, "Out of memory" errors

### 2. **Workspace Grande**
- **Umbral crítico**: >5GB en directorio `target/`
- **Causas**: Múltiples builds de Rust, cache acumulado
- **Síntomas**: Lentitud general, operaciones de archivo lentas

### 3. **Loops Infinitos en Desarrollo**
- **Causas**: Código con loops sin break conditions, procesos background
- **Síntomas**: CPU al 100%, aplicación no responde

### 4. **Configuraciones Problemáticas**
- **RUST_LOG=debug/trace**: Genera logs excesivos
- **Sin límites de memoria para rust-analyzer**
- **Múltiples instancias de VS Code**

## Herramientas de Diagnóstico

### 1. **Script de Diagnóstico** (`diagnose-vscode-crash.ps1`)

```powershell
.\diagnose-vscode-crash.ps1
```

**Funciones:**
- Analiza uso de memoria actual
- Verifica tamaño del workspace
- Revisa logs del sistema por errores
- Identifica configuraciones problemáticas
- Cuenta patrones de código problemáticos
- Ofrece opciones de quick fix

### 2. **Script de Prevención** (`prevent-vscode-crash.ps1`)

```powershell
# Verificación básica
.\prevent-vscode-crash.ps1

# Con auto-fix activado
.\prevent-vscode-crash.ps1 -AutoFix

# Modo silencioso
.\prevent-vscode-crash.ps1 -AutoFix -Quiet
```

**Funciones:**
- Monitoreo proactivo de memoria
- Limpieza automática de cache
- Optimización de configuraciones
- Cleanup de procesos zombies
- Gestión de logs

### 3. **Monitor Continuo** (`monitor-vscode-health.ps1`)

```powershell
# Monitor básico (cada 30 minutos)
.\monitor-vscode-health.ps1

# Con auto-fix y notificaciones
.\monitor-vscode-health.ps1 -EnableAutoFix -EnableNotifications

# Monitor cada 15 minutos con log personalizado
.\monitor-vscode-health.ps1 -IntervalMinutes 15 -LogFile "custom-monitor.log"
```

**Funciones:**
- Monitoreo continuo en background
- Notificaciones Windows automáticas
- Auto-fix de problemas críticos
- Logging detallado de eventos
- Estadísticas de salud a largo plazo

## Estrategias de Prevención

### Preventivas Diarias

1. **Ejecutar prevención automática**:
```powershell
.\prevent-vscode-crash.ps1 -AutoFix
```

2. **Limpieza manual**:
```powershell
cargo clean
.\optimize-vscode.ps1
```

3. **Verificar memoria**:
```powershell
Get-Process Code* | Measure-Object WorkingSet64 -Sum
```

### Preventivas Semanales

1. **Limpieza profunda**:
```powershell
# Limpiar todo el cache de Rust
cargo clean
Remove-Item target -Recurse -Force -ErrorAction SilentlyContinue

# Limpiar logs viejos
Get-ChildItem logs -File | Where-Object { $_.LastWriteTime -lt (Get-Date).AddDays(-7) } | Remove-Item

# Optimizar Git
git gc --aggressive
```

2. **Verificar configuraciones**:
```powershell
# Verificar variables de entorno
[Environment]::GetEnvironmentVariable("RUST_LOG", "User")
[Environment]::GetEnvironmentVariable("RUST_ANALYZER_MEMORY_USAGE_THRESHOLD", "User")
```

### Configuraciones Recomendadas

#### Variables de Entorno
```powershell
[Environment]::SetEnvironmentVariable("RUST_LOG", "warn", "User")
[Environment]::SetEnvironmentVariable("RUST_ANALYZER_MEMORY_USAGE_THRESHOLD", "1024", "User")
[Environment]::SetEnvironmentVariable("CARGO_INCREMENTAL", "0", "User")
```

#### VS Code Settings
Crear `.vscode/settings.json`:
```json
{
  "rust-analyzer.server.extraEnv": {
    "RUST_LOG": "warn"
  },
  "rust-analyzer.checkOnSave.enable": true,
  "rust-analyzer.checkOnSave.allTargets": false,
  "rust-analyzer.cargo.buildScripts.enable": false,
  "rust-analyzer.procMacro.enable": false,
  "files.watcherExclude": {
    "**/target/**": true,
    "**/logs/**": true
  }
}
```

## Workflows de Emergencia

### Cuando VS Code se Congela

1. **Diagnóstico rápido**:
```powershell
.\diagnose-vscode-crash.ps1
# Seleccionar opción 4: Full cleanup
```

2. **Kill force y restart**:
```powershell
Get-Process Code* | Stop-Process -Force
Start-Sleep 3
code .
```

### Cuando el Sistema está Lento

1. **Verificar recursos**:
```powershell
Get-Process Code* | Select-Object Name, WorkingSet64, CPU | Format-Table
```

2. **Cleanup automático**:
```powershell
.\prevent-vscode-crash.ps1 -AutoFix
```

### Después de un Crash

1. **Investigar causa**:
```powershell
.\diagnose-vscode-crash.ps1
```

2. **Revisar logs del sistema**:
```powershell
Get-WinEvent -FilterHashtable @{LogName='Application'; StartTime=(Get-Date).AddHours(-2)} | 
  Where-Object { $_.LevelDisplayName -eq 'Error' -and $_.Message -like '*Code*' }
```

## Monitoreo de Producción

### Setup de Monitor Automático

1. **Crear tarea programada**:
```powershell
# Ejecutar cada hora durante desarrollo
$trigger = New-ScheduledTaskTrigger -Once -At (Get-Date) -RepetitionInterval (New-TimeSpan -Hours 1)
$action = New-ScheduledTaskAction -Execute "PowerShell" -Argument "-File `"$PWD\prevent-vscode-crash.ps1`" -AutoFix -Quiet"
Register-ScheduledTask -TaskName "VSCode-Health-Check" -Trigger $trigger -Action $action
```

2. **Monitor continuo en desarrollo**:
```powershell
# En una terminal separada
.\monitor-vscode-health.ps1 -EnableAutoFix -EnableNotifications -IntervalMinutes 20
```

### Métricas Clave

- **Memoria VS Code**: <2GB normal, >3GB crítico
- **Tamaño workspace**: <2GB normal, >5GB crítico
- **Procesos VS Code**: <5 normal, >10 problemático
- **Tamaño logs**: <50MB normal, >200MB problemático

## Troubleshooting

### VS Code no Inicia
```powershell
# Verificar procesos zombies
Get-Process Code* | Stop-Process -Force

# Limpiar workspace storage
Remove-Item "$env:USERPROFILE\AppData\Roaming\Code\workspaceStorage\*" -Recurse -Force

# Restart con modo safe
code --disable-extensions .
```

### Rust-analyzer Consumiendo Memoria
```powershell
# Kill rust-analyzer específicamente
Get-Process rust-analyzer* | Stop-Process -Force

# Configurar límite de memoria
[Environment]::SetEnvironmentVariable("RUST_ANALYZER_MEMORY_USAGE_THRESHOLD", "512", "User")
```

### Performance Degradado
```powershell
# Verificar extensiones problemáticas
code --list-extensions | ForEach-Object { 
  Write-Host "Extension: $_"
}

# Deshabilitar extensiones temporalmente
code --disable-extensions .
```

## Scripts de Automatización

### Daily Health Check
```powershell
# daily-health-check.ps1
.\prevent-vscode-crash.ps1 -AutoFix -Quiet
if ($LASTEXITCODE -eq 2) {
    Write-Host "Critical issues found and fixed" -ForegroundColor Red
} elseif ($LASTEXITCODE -eq 1) {
    Write-Host "Minor issues found and fixed" -ForegroundColor Yellow
} else {
    Write-Host "System healthy" -ForegroundColor Green
}
```

### Pre-Commit Hook
```powershell
# .git/hooks/pre-commit.ps1
.\prevent-vscode-crash.ps1 -Quiet
if ($LASTEXITCODE -gt 1) {
    Write-Host "Health check failed - consider running optimization" -ForegroundColor Yellow
}
```

## Best Practices

1. **Rutina Diaria**:
   - Ejecutar `prevent-vscode-crash.ps1 -AutoFix` cada mañana
   - Cerrar VS Code completamente al final del día
   - Usar "Reload Window" en lugar de reiniciar

2. **Durante Desarrollo Intensivo**:
   - Monitor continuo cada 20-30 minutos
   - `cargo clean` después de cambios mayores en dependencies
   - Cerrar tabs no utilizados regularmente

3. **Antes de Commits Grandes**:
   - Verificar salud del sistema
   - Limpiar cache si es necesario
   - Asegurar que el workspace esté optimizado

4. **Mantenimiento Semanal**:
   - Ejecutar `diagnose-vscode-crash.ps1` y revisar recomendaciones
   - Limpiar logs viejos y archivos temporales
   - Verificar y optimizar configuraciones

## Contacto y Soporte

Si experimentas crashes frecuentes después de seguir esta guía:

1. Ejecuta `diagnose-vscode-crash.ps1` y guarda el output
2. Revisa el log del monitor: `vscode-monitor.log`
3. Documenta los pasos que llevaron al crash
4. Considera usar VS Code Insiders para mejor estabilidad

---

**Nota**: Esta guía está en constante evolución. Los scripts se actualizan regularmente para abordar nuevas causas de crashes identificadas durante el desarrollo.
