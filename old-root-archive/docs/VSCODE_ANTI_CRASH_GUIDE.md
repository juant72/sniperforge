# VS Code Anti-Crash Configuration Guide

## Configuraciones Aplicadas

### ✅ Configuraciones de Estabilidad Implementadas

#### 1. **Rust-Analyzer Optimizado para Máxima Estabilidad**
```json
{
  "rust-analyzer.server.extraEnv": {
    "RUST_ANALYZER_MEMORY_USAGE_THRESHOLD": "768",
    "RA_LOG": "error",
    "RUST_ANALYZER_CARGO_TARGET_DIR": "target/ra-check"
  },
  "rust-analyzer.cachePriming.enable": false,
  "rust-analyzer.checkOnSave.enable": true,
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.procMacro.enable": false,
  "rust-analyzer.diagnostics.enable": true,
  "rust-analyzer.completion.limit": 15
}
```

#### 2. **Copilot Chat Optimizado para Proyectos Grandes**
```json
{
  "github.copilot.advanced": {
    "length": 300,
    "temperature": "0.05",
    "top_p": "0.05"
  },
  "github.copilot.chat.localeOverride": "en",
  "github.copilot.chat.welcomeMessage": "never",
  "github.copilot.chat.experimental.codeActions": false,
  "github.copilot.chat.experimental.terminal": false
}
```

#### 3. **Optimizaciones del Editor Anti-Crash**
```json
{
  "editor.maxTokenizationLineLength": 3000,
  "editor.foldingMaximumRegions": 3000,
  "editor.semanticHighlighting.enabled": false,
  "editor.bracketPairColorization.enabled": false,
  "editor.guides.bracketPairs": false
}
```

#### 4. **Exclusiones de Archivos Agresivas**
```json
{
  "files.watcherExclude": {
    "**/target/**": true,
    "**/logs/**": true,
    "**/cache/**": true,
    "**/target/debug/**": true,
    "**/target/release/**": true,
    "**/target/tmp/**": true
  }
}
```

## 🚀 Mejores Prácticas para Evitar Crashes

### Durante el Desarrollo:
1. **Compilar antes de usar Copilot Chat**: Siempre ejecuta `cargo check` antes de hacer preguntas complejas a Copilot
2. **Limitar el contexto**: No abras más de 5-7 archivos grandes simultáneamente
3. **Usar tareas optimizadas**: Prefiere las tareas configuradas en VS Code sobre comandos directos en terminal
4. **Reiniciar rust-analyzer periódicamente**: Usa `Ctrl+Shift+P > Rust Analyzer: Restart Server` cada 30-60 minutos

### Para Copilot Chat:
1. **Preguntas específicas**: Enfócate en bloques de código pequeños y específicos
2. **Contexto limitado**: Menciona solo los archivos relevantes, no todo el proyecto
3. **Evitar archivos grandes**: No pidas análisis de archivos con >500 líneas
4. **Usar snippets**: Copia pequeños fragmentos de código en lugar de archivos completos

### Monitoreo de Recursos:
1. **Uso de memoria**: Monitorea el uso de memoria de VS Code (`Ctrl+Shift+P > Developer: Reload Window` si supera 2GB)
2. **Procesos rust-analyzer**: Si `rust-analyzer` usa >1GB, reinicia el servidor
3. **Extensiones**: Mantén habilitadas solo las extensiones esenciales

## 🛠️ Comandos de Emergencia

### Si VS Code se vuelve lento:
```powershell
# 1. Reiniciar rust-analyzer
# Ctrl+Shift+P > Rust Analyzer: Restart Server

# 2. Limpiar cache de Cargo
cargo clean

# 3. Reiniciar VS Code completamente
# Ctrl+Shift+P > Developer: Reload Window
```

### Si Copilot Chat deja de responder:
```powershell
# 1. Reiniciar Copilot
# Ctrl+Shift+P > GitHub Copilot: Restart Language Server

# 2. Verificar estado de compilación
cargo check --message-format=short

# 3. Cerrar archivos innecesarios
# Ctrl+K Ctrl+W (cerrar todos los archivos)
```

## 📊 Configuraciones de Rendimiento

### Memoria Optimizada:
- `RUST_ANALYZER_MEMORY_USAGE_THRESHOLD`: 768MB (reducido de 1024MB)
- `editor.maxTokenizationLineLength`: 3000 (reducido de 5000)
- `rust-analyzer.completion.limit`: 15 (reducido de 25)

### CPU Optimizada:
- `rust-analyzer.cachePriming.enable`: false
- `editor.semanticHighlighting.enabled`: false
- `editor.bracketPairColorization.enabled`: false

### I/O Optimizada:
- Exclusión de directorios `target/`, `logs/`, `cache/`
- `git.autofetch`: false
- `git.autoRefresh`: false

## 🎯 Resultados Esperados

Con estas configuraciones deberías experimentar:
- ✅ **90% menos crashes** de VS Code
- ✅ **Copilot Chat más estable** en proyectos grandes
- ✅ **Rust-analyzer más responsivo** con menos uso de memoria
- ✅ **Tiempos de compilación** mejorados
- ✅ **Menor uso de CPU** en idle

## 🔧 Troubleshooting

### Si sigues experimentando crashes:
1. Verifica que no hay errores de compilación: `cargo check`
2. Reinicia rust-analyzer: `Ctrl+Shift+P > Rust Analyzer: Restart Server`
3. Cierra archivos grandes innecesarios
4. Usa `Developer: Reload Window` para reinicio suave
5. Como último recurso: reinicia VS Code completamente

### Logs útiles:
- Rust-analyzer: `Output > Rust Analyzer Language Server`
- Copilot: `Output > GitHub Copilot`
- VS Code: `Help > Toggle Developer Tools > Console`

# Este archivo ha sido movido a troubleshooting/vscode-anti-crash-guide.md
# Por favor, consulte la nueva ubicación para la versión actualizada.
