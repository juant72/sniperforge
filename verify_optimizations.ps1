#!/usr/bin/env pwsh

Write-Host "🚀 VERIFICANDO OPTIMIZACIONES DE FEES" -ForegroundColor Green
Write-Host "════════════════════════════════════════" -ForegroundColor Yellow

# Verificar cambios en fee_calculator.rs
Write-Host "`n🔍 Verificando fee_calculator.rs..." -ForegroundColor Cyan
$feeChanges = Select-String -Path "src/fee_calculator.rs" -Pattern "bps.*8|bps.*12|bps.*15|bps.*18" | Select-Object -First 5
if ($feeChanges) {
    Write-Host "✅ Fees optimizados detectados:" -ForegroundColor Green
    $feeChanges | ForEach-Object { Write-Host "   $($_.Line.Trim())" -ForegroundColor White }
} else {
    Write-Host "❌ Fees no optimizados" -ForegroundColor Red
}

# Verificar configuración JSON
Write-Host "`n🔍 Verificando arbitrage_settings.json..." -ForegroundColor Cyan
$jsonContent = Get-Content "arbitrage_settings.json" -Raw | ConvertFrom-Json
$tradingConfig = $jsonContent.trading

Write-Host "📊 Configuración actual:" -ForegroundColor Yellow
Write-Host "   • Max trade SOL: $($tradingConfig.max_trade_sol)" -ForegroundColor White
Write-Host "   • Min profit BPS: $($tradingConfig.military_min_profit_bps)" -ForegroundColor White
Write-Host "   • Max slippage BPS: $($tradingConfig.max_slippage_bps)" -ForegroundColor White
Write-Host "   • Min confidence: $($tradingConfig.min_confidence_threshold)" -ForegroundColor White

# Verificar si los valores están optimizados
$optimized = $true
if ($tradingConfig.max_trade_sol -lt 0.4) { $optimized = $false }
if ($tradingConfig.military_min_profit_bps -gt 12) { $optimized = $false }
if ($tradingConfig.max_slippage_bps -gt 20) { $optimized = $false }

if ($optimized) {
    Write-Host "`n✅ Configuración JSON OPTIMIZADA" -ForegroundColor Green
} else {
    Write-Host "`n⚠️ Configuración JSON necesita más optimización" -ForegroundColor Yellow
}

# Compilar y verificar
Write-Host "`n🔧 Compilando optimizaciones..." -ForegroundColor Cyan
$compileResult = cargo build --release --bin arbitrage_phase45_clean 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Compilación exitosa" -ForegroundColor Green
    
    # Mostrar tamaño del binario
    $binarySize = (Get-Item "target/release/arbitrage_phase45_clean.exe").Length / 1MB
    Write-Host "📦 Tamaño del binario: $([math]::Round($binarySize, 2)) MB" -ForegroundColor White
    
    Write-Host "`n🎯 SISTEMA LISTO PARA TESTING OPTIMIZADO" -ForegroundColor Green
    Write-Host "▶️ Ejecutar: .\target\release\arbitrage_phase45_clean.exe" -ForegroundColor Cyan
} else {
    Write-Host "❌ Error en compilación:" -ForegroundColor Red
    Write-Host $compileResult -ForegroundColor Red
}

Write-Host "`n════════════════════════════════════════" -ForegroundColor Yellow
Write-Host "🚀 VERIFICACIÓN COMPLETA" -ForegroundColor Green
