#!/usr/bin/env pwsh
# SCRIPT DE PREPARACIÓN PARA TRADING REAL - 0.29 SOL
# FASE DE TESTING CONSERVADOR

Write-Host "🛡️ PREPARACIÓN PARA TRADING REAL - CAPITAL LIMITADO 0.29 SOL" -ForegroundColor Red
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Yellow

Write-Host "`n⚠️ ADVERTENCIAS CRÍTICAS:" -ForegroundColor Red
Write-Host "   • Capital muy limitado (0.29 SOL = ~$53.7 USD)" -ForegroundColor Yellow
Write-Host "   • Solo para TESTING MUY CONSERVADOR" -ForegroundColor Yellow
Write-Host "   • Fees representarán ~1-2% por trade" -ForegroundColor Yellow
Write-Host "   • Riesgo de agotamiento rápido del capital" -ForegroundColor Yellow

Write-Host "`n📊 ANÁLISIS DE CONFIGURACIÓN:" -ForegroundColor Cyan

# Verificar balance actual
Write-Host "`n🔍 1. VERIFICANDO BALANCE ACTUAL..." -ForegroundColor Green
if (Test-Path "./keypair.json") {
    Write-Host "   ✅ Wallet keypair encontrado: ./keypair.json" -ForegroundColor Green
    # Aquí se podría agregar verificación de balance real
    Write-Host "   💰 Balance esperado: ~0.29 SOL" -ForegroundColor White
} else {
    Write-Host "   ❌ ERROR: No se encontró ./keypair.json" -ForegroundColor Red
    Write-Host "   🔧 ACCIÓN REQUERIDA: Configurar wallet antes de continuar" -ForegroundColor Yellow
    exit 1
}

# Verificar configuración segura
Write-Host "`n🔧 2. VERIFICANDO CONFIGURACIÓN SEGURA..." -ForegroundColor Green
if (Test-Path "./arbitrage_settings_real_0.29SOL.json") {
    Write-Host "   ✅ Configuración segura creada: arbitrage_settings_real_0.29SOL.json" -ForegroundColor Green
    
    $config = Get-Content "./arbitrage_settings_real_0.29SOL.json" | ConvertFrom-Json
    
    Write-Host "   📋 PARÁMETROS DE SEGURIDAD:" -ForegroundColor White
    Write-Host "      • Modo: $($config.trading.mode)" -ForegroundColor $(if ($config.trading.mode -eq "real") { "Red" } else { "Green" })
    Write-Host "      • Max trade: $($config.trading.max_trade_sol) SOL (17% del capital)" -ForegroundColor Green
    Write-Host "      • Min trade: $($config.trading.min_trade_size_sol) SOL (7% del capital)" -ForegroundColor Green
    Write-Host "      • Max slippage: $($config.trading.max_slippage_bps/100)%" -ForegroundColor Green
    Write-Host "      • Min profit: $($config.trading.min_profit_threshold_sol) SOL" -ForegroundColor Green
    Write-Host "      • Max concurrent: $($config.trading.max_concurrent_trades) trades" -ForegroundColor Green
    Write-Host "      • Max daily loss: $($config.risk_management.max_daily_loss_sol) SOL" -ForegroundColor Green
    
} else {
    Write-Host "   ❌ ERROR: Configuración segura no encontrada" -ForegroundColor Red
    exit 1
}

# Calculadora de riesgo
Write-Host "`n📊 3. CALCULADORA DE RIESGO:" -ForegroundColor Green
$totalCapital = 0.29
$maxTrade = 0.05
$maxConcurrent = 2
$feesPerTrade = 0.0003
$maxDailyLoss = 0.1

Write-Host "   💰 ANÁLISIS FINANCIERO:" -ForegroundColor White
Write-Host "      • Capital total: $totalCapital SOL" -ForegroundColor White
Write-Host "      • Capital por trade: $maxTrade SOL ($([math]::Round(($maxTrade/$totalCapital)*100, 1))% del total)" -ForegroundColor White
Write-Host "      • Capital en riesgo máximo: $($maxTrade * $maxConcurrent) SOL ($([math]::Round((($maxTrade * $maxConcurrent)/$totalCapital)*100, 1))% del total)" -ForegroundColor $(if ((($maxTrade * $maxConcurrent)/$totalCapital) -gt 0.4) { "Yellow" } else { "Green" })
Write-Host "      • Fees estimados por trade: $feesPerTrade SOL ($([math]::Round(($feesPerTrade/$maxTrade)*100, 1))% del trade)" -ForegroundColor Yellow
Write-Host "      • Profit mínimo para rentabilidad: $([math]::Round($feesPerTrade * 1.5, 6)) SOL" -ForegroundColor White
Write-Host "      • Máxima pérdida diaria: $maxDailyLoss SOL ($([math]::Round(($maxDailyLoss/$totalCapital)*100, 1))% del total)" -ForegroundColor White

# Plan de testing
Write-Host "`n🎯 4. PLAN DE TESTING CONSERVADOR:" -ForegroundColor Green
Write-Host "   📅 FASE 1 - MICRO-TESTING (Capital: 0.05 SOL)" -ForegroundColor Yellow
Write-Host "      • Objetivo: Validar funcionalidad básica" -ForegroundColor White
Write-Host "      • Trades: 2-3 muy pequeños (0.02 SOL cada uno)" -ForegroundColor White
Write-Host "      • Duración: 1-2 horas" -ForegroundColor White
Write-Host "      • Criterio de éxito: Trades exitosos sin pérdidas" -ForegroundColor White

Write-Host "`n   📅 FASE 2 - TESTING LIMITADO (Capital: 0.1 SOL)" -ForegroundColor Yellow
Write-Host "      • Objetivo: Validar rentabilidad" -ForegroundColor White
Write-Host "      • Trades: 3-5 trades (0.03-0.04 SOL cada uno)" -ForegroundColor White
Write-Host "      • Duración: 2-4 horas" -ForegroundColor White
Write-Host "      • Criterio de éxito: Profit neto positivo" -ForegroundColor White

Write-Host "`n   📅 FASE 3 - EVALUACIÓN" -ForegroundColor Yellow
Write-Host "      • Si rentable: Considerar capital adicional" -ForegroundColor White
Write-Host "      • Si no rentable: Detener hasta conseguir más capital" -ForegroundColor White

# Checklist de seguridad
Write-Host "`n✅ 5. CHECKLIST DE SEGURIDAD PRE-EJECUCIÓN:" -ForegroundColor Green
$checklist = @(
    "Wallet configurado y verificado",
    "Balance confirmado (~0.29 SOL)",
    "Configuración segura aplicada",
    "RPC endpoints mainnet configurados",
    "MEV protection habilitado",
    "Stop loss configurado (0.1 SOL max daily loss)",
    "Límites de trading conservadores aplicados",
    "Monitoreo en tiempo real preparado"
)

foreach ($item in $checklist) {
    Write-Host "   ☐ $item" -ForegroundColor White
}

Write-Host "`n🚨 CONFIRMACIÓN FINAL:" -ForegroundColor Red
Write-Host "   ¿Estás seguro de que quieres proceder con trading real?" -ForegroundColor Yellow
Write-Host "   ⚠️ Capital muy limitado: 0.29 SOL" -ForegroundColor Red
Write-Host "   ⚠️ Alto riesgo relativo de fees" -ForegroundColor Red
Write-Host "   ⚠️ Pocas oportunidades viables" -ForegroundColor Red

Write-Host "`n📋 COMANDOS PARA EJECUTAR:" -ForegroundColor Green
Write-Host "   1. Backup de configuración actual:" -ForegroundColor White
Write-Host "      Copy-Item arbitrage_settings.json arbitrage_settings_backup.json" -ForegroundColor Gray
Write-Host "   2. Aplicar configuración segura:" -ForegroundColor White  
Write-Host "      Copy-Item arbitrage_settings_real_0.29SOL.json arbitrage_settings.json" -ForegroundColor Gray
Write-Host "   3. Ejecutar en modo real:" -ForegroundColor White
Write-Host "      .\target\release\arbitrage_phase45_clean.exe" -ForegroundColor Gray
Write-Host "   4. Monitorear logs:" -ForegroundColor White
Write-Host "      Get-Content arbitrage_real.log -Wait" -ForegroundColor Gray

Write-Host "`n🛡️ SCRIPT COMPLETADO - REVISA TODAS LAS ADVERTENCIAS" -ForegroundColor Red
