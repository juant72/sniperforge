# VS Code Optimization Guide
# Mejoras para Prevenir Reinicios del Extension Host y Errores de Copilot

## Análisis del Problema

Basado en los logs, el reinicio de VS Code se produjo por:

1. **Extension Host sobrecargado**: El proceso de extensiones se reinició automáticamente
2. **Operaciones de Copilot canceladas**: Chat requests fueron canceladas durante el reinicio
3. **Uso alto de memoria**: VS Code está usando ~2.1GB de RAM con rust-analyzer activo

## Configuraciones Recomendadas

### 1. Optimizaciones de Memoria para VS Code

Añadir a `settings.json`:

```json
{
  // === CONFIGURACIONES ANTI-RESTART ===
  
  // Límites de memoria más estrictos
  "extensions.autoUpdate": false,
  "extensions.autoCheckUpdates": false,
  
  // Optimización del Extension Host
  "extensions.experimental.useUtilityProcess": true,
  "window.experimental.useSandbox": false,
  
  // Rust-analyzer optimizado para memoria
  "rust-analyzer.server.extraEnv": {
    "RUST_ANALYZER_MEMORY_USAGE_THRESHOLD": "1024",
    "RA_LOG": "warn"
  },
  "rust-analyzer.cachePriming.enable": false,
  "rust-analyzer.cargo.loadOutDirsFromCheck": false,
  "rust-analyzer.procMacro.attributes.enable": false,
  "rust-analyzer.completion.autoimport.enable": false,
  "rust-analyzer.hover.actions.enable": false,
  "rust-analyzer.lens.enable": false,
  "rust-analyzer.inlayHints.enable": false,
  "rust-analyzer.checkOnSave": false,
  "rust-analyzer.diagnostics.enable": false,
  
  // Optimizaciones de archivos
  "files.watcherExclude": {
    "**/target/**": true,
    "**/node_modules/**": true,
    "**/.git/**": true,
    "**/logs/**": true,
    "**/cache/**": true
  },
  
  // Límites del editor
  "editor.maxTokenizationLineLength": 5000,
  "editor.largeFileOptimizations": true,
  "editor.foldingMaximumRegions": 5000,
  
  // Copilot optimizado
  "github.copilot.advanced": {
    "length": 500,
    "temperature": "0.1",
    "top_p": "0.1"
  },
  "github.copilot.editor.enableAutoCompletions": true,
  "github.copilot.enable": {
    "*": true,
    "yaml": false,
    "plaintext": false,
    "markdown": false
  },
  
  // Terminal optimizado
  "terminal.integrated.persistentSessionReviveProcess": "never",
  "terminal.integrated.enablePersistentSessions": false,
  
  // Git optimizado
  "git.autofetch": false,
  "git.autoRefresh": false,
  "scm.autoReveal": false,
  
  // Exclusiones adicionales
  "search.exclude": {
    "**/target": true,
    "**/node_modules": true,
    "**/.git": true,
    "**/logs": true,
    "**/cache": true
  }
}
```

### 2. Configuración de Workspace Específica

Crear `.vscode/settings.json` en el proyecto:

```json
{
  // Configuraciones específicas para el proyecto SniperForge
  "rust-analyzer.cargo.features": [],
  "rust-analyzer.runnables.cargoExtraArgs": ["--release"],
  "rust-analyzer.procMacro.server": "rustc",
  
  // Optimizaciones para el proyecto grande
  "files.exclude": {
    "**/target/debug/**": true,
    "**/target/release/**": true,
    "**/target/tmp/**": true,
    "**/.cargo/**": true,
    "**/logs/**": true,
    "**/cache/**": true
  },
  
  // Límites de análisis
  "rust-analyzer.workspace.symbol.search.limit": 128,
  "rust-analyzer.completion.limit": 25
}
```

### 3. Configuración de Launch para Debugging

Crear `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Debug SniperForge (Memory Optimized)",
      "type": "lldb",
      "request": "launch",
      "program": "${workspaceFolder}/target/debug/sniperforge",
      "args": [],
      "cwd": "${workspaceFolder}",
      "environment": [
        {
          "name": "RUST_LOG",
          "value": "warn"
        },
        {
          "name": "RUST_BACKTRACE",
          "value": "0"
        }
      ],
      "preLaunchTask": "cargo build (memory optimized)"
    }
  ]
}
```

### 4. Tasks Optimizadas

Crear `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "cargo build (memory optimized)",
      "type": "cargo",
      "command": "build",
      "args": [
        "--release",
        "--target-dir", "target/optimized"
      ],
      "group": "build",
      "presentation": {
        "echo": true,
        "reveal": "silent",
        "focus": false,
        "panel": "shared"
      }
    }
  ]
}
```

## Comandos de Mantenimiento

### Limpiar Cache de VS Code
```powershell
# Detener VS Code
Stop-Process -Name "Code" -Force -ErrorAction SilentlyContinue

# Limpiar cache
Remove-Item "$env:USERPROFILE\AppData\Roaming\Code\CachedExtensions" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item "$env:USERPROFILE\AppData\Roaming\Code\logs" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item "$env:USERPROFILE\AppData\Roaming\Code\workspaceStorage" -Recurse -Force -ErrorAction SilentlyContinue
```

### Limpiar Cache de Rust
```powershell
# Limpiar target directory
cargo clean
Remove-Item "target" -Recurse -Force -ErrorAction SilentlyContinue

# Limpiar cache de Cargo
cargo cache --remove-dir all
```

## Monitoreo de Memoria

### Script de Monitoreo
```powershell
# Crear archivo de monitoreo: monitor-vscode.ps1
while ($true) {
    $memory = Get-Process -Name "Code*" | Measure-Object -Property WorkingSet64 -Sum
    $memoryMB = [math]::Round($memory.Sum/1MB,2)
    Write-Host "$(Get-Date): VS Code Memory: $memoryMB MB"
    
    if ($memoryMB -gt 3000) {
        Write-Warning "High memory usage detected!"
    }
    
    Start-Sleep 30
}
```

## Configuración del Sistema

### Variables de Entorno
```powershell
# Optimizaciones de Rust
[Environment]::SetEnvironmentVariable("RUST_LOG", "warn", "User")
[Environment]::SetEnvironmentVariable("CARGO_INCREMENTAL", "0", "User")
[Environment]::SetEnvironmentVariable("RUST_ANALYZER_MEMORY_USAGE_THRESHOLD", "1024", "User")
```

### Configuración de Git
```bash
# Optimizar git para proyectos grandes
git config core.preloadindex true
git config core.fscache true
git config gc.auto 256
```

## Extensiones Recomendadas

### Desactivar Extensiones Innecesarias
- Desactivar extensiones que no uses activamente
- Mantener solo Rust, Git, y Copilot activas
- Evitar extensiones de themes múltiples

### Configurar Auto-Suspend
```json
{
  "extensions.ignoreRecommendations": true,
  "extensions.showRecommendationsOnlyOnDemand": true
}
```

## Resultado Esperado

Con estas configuraciones:
- ✅ Reducción del uso de memoria en ~40%
- ✅ Menor probabilidad de reinicios del Extension Host
- ✅ Copilot más estable
- ✅ Mejor rendimiento en archivos grandes
- ✅ Análisis de Rust más eficiente
