#!/usr/bin/env pwsh

Write-Host "🚀 === REAL ARBITRAGE HUNTER OFFLINE ===" -ForegroundColor Cyan
Write-Host "   💰 SIN RATE LIMITS - 100% AUTONOMO" -ForegroundColor Green
Write-Host "   ⚡ SOLO BLOCKCHAIN - CERO DEPENDENCIAS" -ForegroundColor Yellow
Write-Host "   🎯 MAXIMA ESTABILIDAD Y CONFIABILIDAD" -ForegroundColor Magenta
Write-Host ""

Write-Host "🔄 Compilando Real Arbitrage Hunter Offline..." -ForegroundColor Blue
cargo build --bin real_arbitrage_hunter_offline

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Compilación exitosa!" -ForegroundColor Green
    Write-Host ""
    Write-Host "🚀 Iniciando ARBITRAJE OFFLINE (sin APIs externas)..." -ForegroundColor Cyan
    Write-Host "   🌟 Este sistema NO depende de CoinGecko ni otros APIs" -ForegroundColor Green
    Write-Host "   ⚡ Usa SOLO datos de la blockchain de Solana" -ForegroundColor Yellow
    Write-Host "   💰 NUNCA MAS RATE LIMITS - MAXIMA AUTONOMIA" -ForegroundColor Magenta
    Write-Host ""
    
    cargo run --bin real_arbitrage_hunter_offline
} else {
    Write-Host "❌ Error en la compilación" -ForegroundColor Red
    exit 1
}
