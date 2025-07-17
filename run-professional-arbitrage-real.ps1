#!/usr/bin/env pwsh

Write-Host "🏆 === SISTEMA PROFESIONAL DE ARBITRAJE ===" -ForegroundColor Cyan
Write-Host "   💎 COMO HACEN LOS PROFESIONALES REALES" -ForegroundColor Green
Write-Host "   ⚡ DATOS ON-CHAIN DIRECTOS - NO APIs BASURA" -ForegroundColor Yellow
Write-Host "   🎯 PARSING DE CUENTAS AMM NATIVO" -ForegroundColor Magenta
Write-Host "   🔥 INSTRUCCIONES DE SWAP REALES" -ForegroundColor Red
Write-Host ""

Write-Host "🔄 Compilando sistema profesional..." -ForegroundColor Blue
cargo build --bin professional_arbitrage

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Compilación exitosa!" -ForegroundColor Green
    Write-Host ""
    Write-Host "🚀 Iniciando ARBITRAJE PROFESIONAL..." -ForegroundColor Cyan
    Write-Host "   🏆 Este es el sistema que usan los traders REALES" -ForegroundColor Green
    Write-Host "   💎 Lee pools directamente de la blockchain" -ForegroundColor Yellow
    Write-Host "   ⚡ Ejecuta swaps nativos en Raydium y Orca" -ForegroundColor Magenta
    Write-Host "   🎯 NADA de APIs externas de mierda" -ForegroundColor Red
    Write-Host ""
    
    cargo run --bin professional_arbitrage
} else {
    Write-Host "❌ Error en la compilación" -ForegroundColor Red
    exit 1
}
