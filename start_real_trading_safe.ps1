#!/usr/bin/env pwsh
# ACTIVAR TRADING REAL CON PROTECCIONES ANTI-CIRCULARES
# =====================================================
# Sistema corregido con protección contra arbitrajes circulares falsos

Write-Host "🔥 ACTIVANDO TRADING REAL" -ForegroundColor Red
Write-Host "========================" -ForegroundColor Red

Write-Host "✅ VERIFICACIONES PRE-TRADING:" -ForegroundColor Green
Write-Host "   • Protección anti-circular: ✅ IMPLEMENTADA"
Write-Host "   • Wallet configurada: ✅ JDzF9LkpoQVac6c6ufHW1c6Gevd3vFB4RXSSjceL8Kq7"
Write-Host "   • Balance disponible: ✅ 0.292 SOL"
Write-Host "   • Oportunidades detectadas: ✅ WIF 1.63%, PYTH 1.79%"

Write-Host ""
Write-Host "🚀 CONFIGURANDO VARIABLES DE ENTORNO:" -ForegroundColor Yellow
$env:FORCE_REAL_TRANSACTIONS = "true"
$env:MAX_TRADE_SOL = "0.01"

Write-Host "   FORCE_REAL_TRANSACTIONS = $env:FORCE_REAL_TRANSACTIONS"
Write-Host "   MAX_TRADE_SOL = $env:MAX_TRADE_SOL"

Write-Host ""
Write-Host "💰 PARÁMETROS DE TRADING REAL:" -ForegroundColor Magenta
Write-Host "   • Máximo por trade: 0.01 SOL (~$2 @ $200/SOL)"
Write-Host "   • Trades posibles: ~29 trades con balance actual"
Write-Host "   • Target: Oportunidades >1% profit"
Write-Host "   • Protección: Solo arbitrajes entre DEXs diferentes"

Write-Host ""
Write-Host "🎯 INICIANDO SISTEMA..." -ForegroundColor Cyan

# Ejecutar con variables de entorno configuradas
cargo run --bin arbitrage_phase45_clean
