#!/usr/bin/env pwsh
# SISTEMA MILITAR: Ejecución directa del arbitraje sin build lento

Write-Host "⚔️ SISTEMA MILITAR DE ARBITRAJE: Iniciando operación..." -ForegroundColor Red

# 1. VERIFICACIÓN PREVIA RÁPIDA
Write-Host "🔍 Verificación rápida del sistema..." -ForegroundColor Yellow
if (!(Test-Path "military_arbitrage_system.rs")) {
    Write-Host "❌ Archivo militar no encontrado" -ForegroundColor Red
    exit 1
}

# 2. VARIABLES DE ENTORNO MILITARES
$env:RUST_LOG = "info"
$env:RUST_BACKTRACE = "1"

# 3. CONFIGURACIÓN HELIUS (si está disponible)
if ($env:HELIUS_API_KEY) {
    Write-Host "🔥 Helius Premium API detectada - Modo profesional" -ForegroundColor Green
} else {
    Write-Host "⚠️ Helius API no configurada - Modo fallback" -ForegroundColor Yellow
    Write-Host "   Para mejor rendimiento: set HELIUS_API_KEY=tu_clave" -ForegroundColor White
}

# 4. VERIFICAR WALLET (opcional para modo dry-run)
if ($env:SOLANA_PRIVATE_KEY) {
    Write-Host "🔑 Wallet configurada - Modo real" -ForegroundColor Green
} else {
    Write-Host "🧪 Modo simulación - Sin wallet" -ForegroundColor Cyan
}

Write-Host "`n🚀 INICIANDO ARBITRAJE MILITAR..." -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor White

# 5. EJECUCIÓN: Usar cargo run para evitar build completo
$startTime = Get-Date

try {
    # Usar cargo run con optimizaciones de dev
    cargo run --bin military_arbitrage_system --profile dev 2>&1 | Tee-Object -FilePath "military_execution.log"
    
    $endTime = Get-Date
    $duration = ($endTime - $startTime).TotalMinutes
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "`n✅ OPERACIÓN MILITAR COMPLETADA" -ForegroundColor Green
        Write-Host "⏱️ Tiempo de ejecución: $([math]::Round($duration, 2)) minutos" -ForegroundColor Blue
        Write-Host "📋 Log completo: military_execution.log" -ForegroundColor White
    } else {
        Write-Host "`n❌ OPERACIÓN FALLÓ" -ForegroundColor Red
        Write-Host "📋 Ver military_execution.log para detalles" -ForegroundColor Yellow
    }
} catch {
    Write-Host "`n💥 ERROR CRÍTICO: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`n🎯 VENTAJA: Ejecución directa sin build de 30min" -ForegroundColor Magenta
