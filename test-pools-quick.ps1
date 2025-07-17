#!/usr/bin/env pwsh
# QUICK POOL DETECTION TEST - Sin compilar todo el proyecto

Write-Host "🔥 QUICK POOL DETECTION TEST" -ForegroundColor Yellow
Write-Host "================================" -ForegroundColor Yellow

# Compilar solo el archivo específico
Write-Host "📊 Compilando sistema militar..." -ForegroundColor Green
cargo build --bin military_arbitrage_system --release

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Compilación exitosa!" -ForegroundColor Green
    
    # Ejecutar con timeout para no esperar mucho
    Write-Host "🎯 Ejecutando detección de pools..." -ForegroundColor Cyan
    timeout 60 cargo run --bin military_arbitrage_system --release
    
    if ($LASTEXITCODE -eq 124) {
        Write-Host "⏰ Timeout alcanzado - Sistema funcionando" -ForegroundColor Yellow
    } elseif ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Ejecución completada exitosamente" -ForegroundColor Green
    } else {
        Write-Host "❌ Error en ejecución: $LASTEXITCODE" -ForegroundColor Red
    }
} else {
    Write-Host "❌ Error en compilación" -ForegroundColor Red
}

Write-Host "🎯 Test completado" -ForegroundColor Yellow
