#!/usr/bin/env pwsh

Write-Host "🌟 === JUPITER V3 ARBITRAGE HUNTER ===" -ForegroundColor Cyan
Write-Host "   🪐 Using Jupiter Price API v3" -ForegroundColor Green
Write-Host "   ⚡ Professional multi-source price comparison" -ForegroundColor Yellow
Write-Host "   💰 Hybrid DEX + Jupiter v3 strategy" -ForegroundColor Magenta
Write-Host "   🎯 Advanced arbitrage detection" -ForegroundColor Blue

Write-Host "🏆 === CONFIGURACIÓN JUPITER V3 ===" -ForegroundColor Yellow

# Check for API keys
$AlchemyKey = $env:ALCHEMY_API_KEY
$HeliusKey = $env:HELIUS_API_KEY
$BirdeyeKey = $env:BIRDEYE_API_KEY

if ($AlchemyKey) {
    Write-Host "✅ Alchemy Premium RPC: $($AlchemyKey.Substring(0,8))..." -ForegroundColor Green
} elseif ($HeliusKey) {
    Write-Host "✅ Helius Premium RPC: $($HeliusKey.Substring(0,8))..." -ForegroundColor Green
} else {
    Write-Host "✅ Alchemy Premium RPC: X64q4zZF..." -ForegroundColor Green
}

if ($BirdeyeKey) {
    Write-Host "✅ Birdeye API: $($BirdeyeKey.Substring(0,6))..." -ForegroundColor Green
} else {
    Write-Host "⚠️  Birdeye API: Not configured (optional)" -ForegroundColor Yellow
    Write-Host "   Para mejores resultados: https://docs.birdeye.so/docs/authentication-api-keys" -ForegroundColor Gray
}

Write-Host "🔧 === JUPITER V3 FEATURES ===" -ForegroundColor Magenta
Write-Host "   📊 Jupiter v3 vs Direct DEX comparison" -ForegroundColor White
Write-Host "   💱 Jupiter v3 vs CEX price arbitrage" -ForegroundColor White
Write-Host "   🛣️  Jupiter v3 route arbitrage detection" -ForegroundColor White
Write-Host "   ⚡ 8-second intervals - optimal for v3 API" -ForegroundColor White
Write-Host "   🎯 Multi-source price validation" -ForegroundColor White

# Check wallet
if (Test-Path "mainnet_wallet.json") {
    Write-Host "✅ Wallet found: mainnet_wallet.json" -ForegroundColor Green
} elseif (Test-Path "wallet.json") {
    Write-Host "✅ Wallet found: wallet.json" -ForegroundColor Green
} else {
    Write-Host "❌ No wallet found!" -ForegroundColor Red
    Write-Host "   Create wallet first: cargo run --bin generate_wallet" -ForegroundColor Yellow
    exit 1
}

Write-Host "💡 === JUPITER V3 ADVANTAGES ===" -ForegroundColor Cyan
Write-Host "🚫 Jupiter v6 (PROBLEMAS):" -ForegroundColor Red
Write-Host "   • Rate limits 429" -ForegroundColor Gray
Write-Host "   • Swap API instability" -ForegroundColor Gray
Write-Host "   • Complex transaction building" -ForegroundColor Gray

Write-Host "✅ Jupiter v3 (SOLUCIÓN):" -ForegroundColor Green
Write-Host "   • Stable Price API" -ForegroundColor Gray
Write-Host "   • No rate limits on price queries" -ForegroundColor Gray
Write-Host "   • Simple price comparison" -ForegroundColor Gray
Write-Host "   • Multi-route analysis" -ForegroundColor Gray

Write-Host "🎯 READY FOR JUPITER V3 ARBITRAGE!" -ForegroundColor Green
Write-Host "   Esta estrategia combina lo mejor de Jupiter con DEX directo" -ForegroundColor Cyan
Write-Host "   Press Enter to start or Ctrl+C to cancel..." -ForegroundColor Yellow

$null = Read-Host

Write-Host "🚀 Starting Jupiter v3 arbitrage hunter..." -ForegroundColor Green

try {
    # Build and run
    Write-Host "   🔨 Building Jupiter v3 hunter..." -ForegroundColor Yellow
    cargo build --release --bin jupiter_v3_hunter

    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Build successful, starting Jupiter v3 hunter..." -ForegroundColor Green
        Write-Host "   🪐 JUPITER V3 PRICE API ACTIVE!" -ForegroundColor Magenta
        cargo run --release --bin jupiter_v3_hunter
    } else {
        Write-Host "❌ Build failed!" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "❌ Error: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}
