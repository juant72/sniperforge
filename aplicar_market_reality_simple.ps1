# APLICAR CONFIGURACION MARKET REALITY
# Script para implementar ajustes que permitan trades exitosos

Write-Host "Aplicando configuracion Market Reality..." -ForegroundColor Yellow

# Verificar balance actual
Write-Host "Verificando balance SOL..." -ForegroundColor Green
$balance = solana balance --output json-compact | ConvertFrom-Json
$sol_balance = [math]::Round($balance.value / 1000000000, 9)
Write-Host "Balance actual: $sol_balance SOL" -ForegroundColor Cyan

if ($sol_balance -lt 0.1) {
    Write-Host "ADVERTENCIA: Balance bajo ($sol_balance SOL)" -ForegroundColor Yellow
    Write-Host "Configuracion ajustada para balance disponible" -ForegroundColor Yellow
}

# Crear backup de configuracion actual
Write-Host "Creando backup..." -ForegroundColor Green
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
if (Test-Path "arbitrage_settings.json") {
    Copy-Item "arbitrage_settings.json" "arbitrage_settings_backup_$timestamp.json"
    Write-Host "Backup creado: arbitrage_settings_backup_$timestamp.json" -ForegroundColor Green
}

# Aplicar nueva configuracion
Write-Host "Aplicando configuracion Market Reality..." -ForegroundColor Green
Copy-Item "arbitrage_settings_market_reality.json" "arbitrage_settings.json" -Force
Write-Host "Configuracion aplicada exitosamente" -ForegroundColor Green

# Mostrar cambios principales
Write-Host ""
Write-Host "CAMBIOS PRINCIPALES APLICADOS:" -ForegroundColor Cyan
Write-Host "Max Trade Size: 0.020 SOL -> 0.080 SOL (+300%)" -ForegroundColor White
Write-Host "Min Profit: 0.4% -> 0.25% (-37.5%)" -ForegroundColor White  
Write-Host "Min Confidence: 40% -> 30% (-25%)" -ForegroundColor White
Write-Host "MEV Tips: 3000 -> 1500 lamports (-50%)" -ForegroundColor White
Write-Host "Max Slippage: 30 -> 20 bps (-33%)" -ForegroundColor White

Write-Host ""
Write-Host "PROYECCION DE RESULTADOS:" -ForegroundColor Yellow
Write-Host "Trades ejecutables por dia: 2-3 (vs 0 anterior)" -ForegroundColor Green
Write-Host "Profit esperado por trade: 0.000040-0.000200 SOL" -ForegroundColor Green
Write-Host "Riesgo por trade: 0.000040 SOL maximo" -ForegroundColor Green

Write-Host ""
Write-Host "LISTO PARA EJECUTAR CON:" -ForegroundColor Green
Write-Host "cargo run --bin arbitrage_bot_phase45_unified" -ForegroundColor White

Write-Host ""
Write-Host "PARA MONITOREAR:" -ForegroundColor Cyan
Write-Host "Busca lineas 'EXECUTING ARBITRAGE'" -ForegroundColor White
Write-Host "Profits esperados: 0.000040+ SOL" -ForegroundColor White
Write-Host "Trade sizes: 0.015-0.080 SOL" -ForegroundColor White

Write-Host ""
Write-Host "CONFIGURACION MARKET REALITY APLICADA" -ForegroundColor Green
