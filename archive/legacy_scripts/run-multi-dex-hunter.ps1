#!/usr/bin/env pwsh

Write-Host "🌟 === MULTI-DEX ARBITRAGE HUNTER ===" -ForegroundColor Cyan
Write-Host "   🚀 NO JUPITER DEPENDENCY" -ForegroundColor Green
Write-Host "   ⚡ Direct DEX access (Raydium + Orca + Birdeye)" -ForegroundColor Yellow
Write-Host "   💰 Professional Multi-DEX Strategy" -ForegroundColor Magenta
Write-Host "   🎯 No rate limits with direct API access" -ForegroundColor Blue

# Configurar las API keys que ya tienes
$env:ALCHEMY_API_KEY = "X64q4zZFEMz_RYzthxUMg"

# Verificar si hay otras API keys
if (Test-Path ".env") {
    $envContent = Get-Content ".env" -Raw
    if ($envContent -match "HELIUS_API_KEY=([^\s]+)") {
        $env:HELIUS_API_KEY = $matches[1]
        Write-Host "✅ Helius API key found!" -ForegroundColor Green
    }
    if ($envContent -match "BIRDEYE_API_KEY=([^\s]+)") {
        $env:BIRDEYE_API_KEY = $matches[1]
        Write-Host "✅ Birdeye API key found!" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "🏆 === CONFIGURACIÓN MULTI-DEX ===" -ForegroundColor Green
Write-Host "✅ Alchemy Premium RPC: X64q4zZFEMz_RYzthxUMg" -ForegroundColor Cyan

if ($env:BIRDEYE_API_KEY) {
    Write-Host "✅ Birdeye API: Enhanced price feeds" -ForegroundColor Cyan
} else {
    Write-Host "⚠️  Birdeye API: Not configured (optional)" -ForegroundColor Yellow
    Write-Host "   Para mejores resultados: https://docs.birdeye.so/docs/authentication-api-keys" -ForegroundColor White
}

Write-Host ""
Write-Host "🔧 === ESTRATEGIAS PROFESIONALES ===" -ForegroundColor Magenta
Write-Host "   📊 Raydium vs Orca price differences" -ForegroundColor White
Write-Host "   🐦 Birdeye price feed arbitrage" -ForegroundColor White
Write-Host "   📚 Serum orderbook vs AMM" -ForegroundColor White
Write-Host "   🦎 CoinGecko vs on-chain prices" -ForegroundColor White
Write-Host "   ⚡ 10-second intervals - NO RATE LIMITS" -ForegroundColor White
Write-Host "   🎯 Direct DEX API access" -ForegroundColor White

# Check wallet
if (Test-Path "mainnet_wallet.json") {
    Write-Host "✅ Wallet found: mainnet_wallet.json" -ForegroundColor Green
} else {
    Write-Host "❌ Wallet not found: mainnet_wallet.json" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "💡 === VENTAJAS MULTI-DEX ===" -ForegroundColor Yellow
Write-Host "🚫 Jupiter (PROBLEMAS):" -ForegroundColor Red
Write-Host "   • Rate limits 429" -ForegroundColor White
Write-Host "   • Dependencia de un solo aggregator" -ForegroundColor White
Write-Host "   • Timeouts frecuentes" -ForegroundColor White
Write-Host ""
Write-Host "✅ Multi-DEX (SOLUCIÓN):" -ForegroundColor Green
Write-Host "   • Sin rate limits" -ForegroundColor White
Write-Host "   • Múltiples fuentes de precios" -ForegroundColor White
Write-Host "   • Estrategias diversificadas" -ForegroundColor White
Write-Host "   • Acceso directo a DEXs" -ForegroundColor White

Write-Host ""
Write-Host "🎯 READY FOR MULTI-DEX ARBITRAGE!" -ForegroundColor Green
Write-Host "   Esta es la estrategia que usan los profesionales" -ForegroundColor Yellow
Write-Host "   Press Enter to start or Ctrl+C to cancel..." -ForegroundColor Cyan
Read-Host

Write-Host "🚀 Starting MULTI-DEX arbitrage hunter..." -ForegroundColor Green

# Build and run Multi-DEX hunter
cargo build --release --bin multi_dex_hunter
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Build successful, starting MULTI-DEX hunter..." -ForegroundColor Green
    Write-Host "   🏆 NO MORE JUPITER RATE LIMITS!" -ForegroundColor Yellow
    cargo run --release --bin multi_dex_hunter
} else {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}
