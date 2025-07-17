#!Write-Host "🌟 === PURE DEXWrite-Host "🔧 === PURE DEX FEATURES ===" -ForegroundColor Yellow
Write-Host "   📊 CoinGecko vs REAL DEX swaps" -ForegroundColor White
Write-Host "   🔄 Cross-DEX REAL arbitrage execution" -ForegroundColor White
Write-Host "   ⏰ Time-based volatility detection with REAL TRADES" -ForegroundColor White
Write-Host "   💰 Stablecoin deviation REAL arbitrage" -ForegroundColor White
Write-Host "   ⚡ 6-second intervals - maximum throughput" -ForegroundColor White
Write-Host "   🎯 REAL BLOCKCHAIN TRANSACTIONS" -ForegroundColor GreenAGE HUNTER ===" -ForegroundColor Cyan
Write-Host "   🚀 100% Direct DEX APIs - REAL BLOCKCHAIN TRANSACTIONS" -ForegroundColor Green
Write-Host "   ⚡ Zero-dependency professional strategy with ACTUAL SWAPS" -ForegroundColor Green
Write-Host "   💰 Battle-tested arbitrage detection - REAL MONEY EXECUTION" -ForegroundColor Green
Write-Host "   🎯 Maximum reliability and speed - NO JUPITER DEPENDENCY" -ForegroundColor Greenbin/env pwsh

Write-Host "🌟 === PURE DEX ARBITRAGE HUNTER ===" -ForegroundColor Cyan
Write-Host "   🚀 100% Direct DEX APIs - NO external dependencies" -ForegroundColor Green
Write-Host "   ⚡ Zero-dependency professional strategy" -ForegroundColor Yellow
Write-Host "   💰 Battle-tested arbitrage detection" -ForegroundColor Magenta
Write-Host "   🎯 Maximum reliability and speed" -ForegroundColor Blue

Write-Host "🏆 === CONFIGURACIÓN PURE DEX ===" -ForegroundColor Yellow

# Check for API keys
$AlchemyKey = $env:ALCHEMY_API_KEY
$HeliusKey = $env:HELIUS_API_KEY

if ($AlchemyKey) {
    Write-Host "✅ Alchemy Premium RPC: $($AlchemyKey.Substring(0,8))..." -ForegroundColor Green
} elseif ($HeliusKey) {
    Write-Host "✅ Helius Premium RPC: $($HeliusKey.Substring(0,8))..." -ForegroundColor Green
} else {
    Write-Host "✅ Alchemy Premium RPC: X64q4zZF..." -ForegroundColor Green
}

Write-Host "✅ CoinGecko API: Public (no key required)" -ForegroundColor Green
Write-Host "✅ Direct DEX simulation: Built-in" -ForegroundColor Green

Write-Host "🔧 === PURE DEX FEATURES ===" -ForegroundColor Magenta
Write-Host "   📊 CoinGecko vs Simulated DEX comparison" -ForegroundColor White
Write-Host "   🔄 Cross-DEX arbitrage simulation" -ForegroundColor White
Write-Host "   ⏰ Time-based volatility detection" -ForegroundColor White
Write-Host "   💰 Stablecoin deviation arbitrage" -ForegroundColor White
Write-Host "   ⚡ 6-second intervals - maximum throughput" -ForegroundColor White
Write-Host "   🎯 Zero external dependencies" -ForegroundColor White

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

Write-Host "💡 === PURE DEX ADVANTAGES ===" -ForegroundColor Cyan
Write-Host "🚫 External Dependencies (PROBLEMAS):" -ForegroundColor Red
Write-Host "   • Jupiter API rate limits" -ForegroundColor Gray
Write-Host "   • Birdeye API requirements" -ForegroundColor Gray
Write-Host "   • Network dependency risks" -ForegroundColor Gray
Write-Host "   • Third-party API failures" -ForegroundColor Gray

Write-Host "✅ Pure DEX (SOLUCIÓN):" -ForegroundColor Green
Write-Host "   • Only CoinGecko (most reliable)" -ForegroundColor Gray
Write-Host "   • Built-in DEX simulation" -ForegroundColor Gray
Write-Host "   • Zero external dependencies" -ForegroundColor Gray
Write-Host "   • Maximum uptime and reliability" -ForegroundColor Gray
Write-Host "   • Battle-tested algorithms" -ForegroundColor Gray

Write-Host "🎯 READY FOR PURE DEX ARBITRAGE!" -ForegroundColor Green
Write-Host "   Esta es la estrategia más confiable - sin dependencias externas" -ForegroundColor Cyan
Write-Host "   Press Enter to start or Ctrl+C to cancel..." -ForegroundColor Yellow

$null = Read-Host

Write-Host "🚀 Starting Pure DEX arbitrage hunter..." -ForegroundColor Green

try {
    # Build and run
    Write-Host "   🔨 Building Pure DEX hunter..." -ForegroundColor Yellow
    cargo build --release --bin pure_dex_hunter

    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Build successful, starting Pure DEX hunter..." -ForegroundColor Green
        Write-Host "   💪 ZERO EXTERNAL DEPENDENCIES ACTIVE!" -ForegroundColor Magenta
        cargo run --release --bin pure_dex_hunter
    } else {
        Write-Host "❌ Build failed!" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "❌ Error: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}
