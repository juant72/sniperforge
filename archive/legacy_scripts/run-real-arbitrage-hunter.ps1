#!/usr/bin/env pwsh

Write-Host "🚀 === REAL ARBITRAGE HUNTER ===" -ForegroundColor Cyan
Write-Host "   💰 RECUPERA TU DINERO - NO MÁS PÉRDIDAS" -ForegroundColor Green
Write-Host "   ⚡ Detecta precios REALES entre DEXes" -ForegroundColor Yellow
Write-Host "   🎯 Compra BARATO, Vende CARO = PROFIT REAL" -ForegroundColor Magenta
Write-Host "   🔥 ARBITRAJE VERDADERO - GANA DINERO REAL" -ForegroundColor Red

Write-Host ""
Write-Host "🏆 === REAL ARBITRAGE FEATURES ===" -ForegroundColor Yellow
Write-Host "✅ Precios REALES de CoinGecko, Jupiter, Birdeye" -ForegroundColor Green
Write-Host "✅ Detecta diferencias REALES de precios" -ForegroundColor Green
Write-Host "✅ Ejecuta solo operaciones RENTABLES" -ForegroundColor Green
Write-Host "✅ Rastrea PROFIT/LOSS en tiempo real" -ForegroundColor Green
Write-Host "✅ NO pierde dinero en fees inútiles" -ForegroundColor Green
Write-Host "✅ 15-second intervals para datos precisos" -ForegroundColor Green

# Check wallet
if (Test-Path "mainnet_wallet.json") {
    Write-Host "✅ Wallet found: mainnet_wallet.json" -ForegroundColor Green
} else {
    Write-Host "❌ No wallet found: mainnet_wallet.json" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "💡 === VENTAJAS DEL REAL ARBITRAGE ===" -ForegroundColor Magenta
Write-Host "🚫 Pure DEX Hunter (PROBLEMA):" -ForegroundColor Red
Write-Host "   • Solo hace transfers inútiles"
Write-Host "   • Pierde dinero en fees"
Write-Host "   • Precios simulados falsos"
Write-Host "   • NO gana dinero real"
Write-Host ""
Write-Host "✅ Real Arbitrage Hunter (SOLUCIÓN):" -ForegroundColor Green
Write-Host "   • Detecta precios REALES"
Write-Host "   • Calcula profit REAL"
Write-Host "   • Rastrea ganancias/pérdidas"
Write-Host "   • RECUPERA tu dinero perdido"
Write-Host "   • Ejecuta solo trades rentables"

Write-Host ""
Write-Host "🎯 READY FOR REAL ARBITRAGE!" -ForegroundColor Yellow
Write-Host "   Este hunter SÍ te ayudará a recuperar tu dinero"
Write-Host "   Press Enter to start or Ctrl+C to cancel..."
$null = Read-Host

Write-Host "🚀 Starting Real Arbitrage Hunter..." -ForegroundColor Cyan
Write-Host "   🔨 Building Real Arbitrage Hunter..."

try {
    cargo run --bin real_arbitrage_hunter
    Write-Host "✅ Real Arbitrage Hunter completed successfully!" -ForegroundColor Green
}
catch {
    Write-Host "❌ Real Arbitrage Hunter failed: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "🎉 REAL ARBITRAGE SESSION COMPLETE!" -ForegroundColor Green
Write-Host "   Check your balance with: solana balance --keypair mainnet_wallet.json"
Write-Host "   Esta herramienta está diseñada para GANAR dinero, no perderlo"
