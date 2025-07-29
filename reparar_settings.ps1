# SCRIPT DE REPARACION DE ARBITRAGE SETTINGS
# Repara y restaura la configuracion del sistema de arbitraje

Write-Host "🔧 REPARANDO ARBITRAGE SETTINGS..." -ForegroundColor Yellow

# Crear backup del archivo actual si existe
if (Test-Path "arbitrage_settings.json") {
    $timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
    Copy-Item "arbitrage_settings.json" "arbitrage_settings_BACKUP_$timestamp.json"
    Write-Host "✅ Backup creado: arbitrage_settings_BACKUP_$timestamp.json" -ForegroundColor Green
}

# Aplicar archivo reparado
Write-Host "🔄 Aplicando configuracion reparada..." -ForegroundColor Green
Copy-Item "arbitrage_settings_REPAIRED.json" "arbitrage_settings.json" -Force

# Verificar integridad del JSON
Write-Host "🔍 Verificando integridad del JSON..." -ForegroundColor Cyan
try {
    $config = Get-Content "arbitrage_settings.json" | ConvertFrom-Json
    Write-Host "✅ JSON valido y parseado correctamente" -ForegroundColor Green
    
    # Verificar campos criticos
    if ($config.trading -and $config.trading.max_trade_sol) {
        Write-Host "✅ Seccion trading: OK" -ForegroundColor Green
    }
    if ($config.mev_protection -and $config.mev_protection.jito_tip_lamports) {
        Write-Host "✅ Seccion mev_protection: OK" -ForegroundColor Green
    }
    if ($config.target_tokens -and $config.target_tokens.Count -gt 0) {
        Write-Host "✅ Target tokens: $($config.target_tokens.Count) configurados" -ForegroundColor Green
    }
    
} catch {
    Write-Host "❌ Error en JSON: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Mostrar configuracion clave
Write-Host ""
Write-Host "📊 CONFIGURACION APLICADA:" -ForegroundColor Cyan
Write-Host "• Modo: $($config.trading.mode)" -ForegroundColor White
Write-Host "• Max Trade: $($config.trading.max_trade_sol) SOL" -ForegroundColor White
Write-Host "• Min Profit: $($config.trading.military_min_profit_bps) bps" -ForegroundColor White
Write-Host "• MEV Tips: $($config.mev_protection.jito_tip_lamports) lamports" -ForegroundColor White
Write-Host "• Tokens objetivo: $($config.target_tokens.Count)" -ForegroundColor White

Write-Host ""
Write-Host "✅ ARBITRAGE SETTINGS REPARADO EXITOSAMENTE" -ForegroundColor Green
Write-Host "🚀 Listo para ejecutar el sistema" -ForegroundColor Green
