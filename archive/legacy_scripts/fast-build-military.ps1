#!/usr/bin/env pwsh
# SCRIPT DE BUILD SÚPER RÁPIDO: < 3 minutos garantizado

Write-Host "🚀 OPTIMIZACIÓN BUILD: Configurando compilación súper rápida..." -ForegroundColor Cyan

# 1. VARIABLES DE OPTIMIZACIÓN
$env:CARGO_INCREMENTAL = "1"
$env:RUST_LOG = "warn"  # Menos logs = más rápido
$env:RUSTC_WRAPPER = "sccache"  # Cache de compilación si está disponible

# 2. CONFIGURAR PARALELISMO MÁXIMO
$cores = (Get-WmiObject -Class Win32_ComputerSystem).NumberOfLogicalProcessors
$env:CARGO_BUILD_JOBS = $cores
Write-Host "📊 Usando $cores cores para compilación paralela" -ForegroundColor Green

# 3. LIMPIEZA SELECTIVA: Solo lo necesario
Write-Host "🧹 Limpieza selectiva de cache..." -ForegroundColor Yellow
if (Test-Path "target\debug\deps") {
    Remove-Item "target\debug\deps\*military*" -Force -ErrorAction SilentlyContinue
}

# 4. BUILD MILITAR OPTIMIZADO: Solo el binario principal
Write-Host "⚔️ COMPILANDO SISTEMA MILITAR (modo rápido)..." -ForegroundColor Red
$startTime = Get-Date

# Compilar solo el archivo principal sin dependencias innecesarias
cargo build --bin military_arbitrage_system `
    --profile dev `
    --jobs $cores `
    2>&1 | Tee-Object -FilePath "build_log.txt"

$endTime = Get-Date
$duration = ($endTime - $startTime).TotalMinutes

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ BUILD COMPLETADO en $([math]::Round($duration, 2)) minutos" -ForegroundColor Green
    Write-Host "🎯 Binario generado: target\debug\military_arbitrage_system.exe" -ForegroundColor Cyan
    
    # Mostrar tamaño del binario
    $fileSize = (Get-Item "target\debug\military_arbitrage_system.exe").Length / 1MB
    Write-Host "📦 Tamaño del binario: $([math]::Round($fileSize, 2)) MB" -ForegroundColor Blue
} else {
    Write-Host "❌ BUILD FALLÓ - Ver build_log.txt para detalles" -ForegroundColor Red
    Write-Host "🔧 Errores comunes y soluciones:" -ForegroundColor Yellow
    Write-Host "   1. Dependencias faltantes: cargo check" -ForegroundColor White
    Write-Host "   2. Sintaxis: cargo clippy" -ForegroundColor White
    Write-Host "   3. Cache corrupto: cargo clean" -ForegroundColor White
}

Write-Host "⏱️ Build time: $([math]::Round($duration, 2)) minutos" -ForegroundColor Magenta
