#!/usr/bin/env pwsh

Write-Host "🔥 === AGGRESSIVE ARBITRAGE HUNTER ===" -ForegroundColor Red
Write-Host "   🚀 NEW OPTIMIZED VERSION FOR MAXIMUM OPPORTUNITIES" -ForegroundColor Yellow
Write-Host "   💰 70+ token pairs with multiple amounts" -ForegroundColor Green
Write-Host "   ⚡ 10-second intervals + 3x fee threshold" -ForegroundColor Cyan
Write-Host "   🎯 Higher slippage tolerance for more trades" -ForegroundColor Magenta
Write-Host ""

# Check if wallet exists
$walletPaths = @(
    $env:SOLANA_WALLET_PATH,
    "mainnet_wallet.json",
    "wallet.json"
)

$foundWallet = $false
foreach ($path in $walletPaths) {
    if ($path -and (Test-Path $path)) {
        Write-Host "✅ Wallet found: $path" -ForegroundColor Green
        $foundWallet = $true
        break
    }
}

if (-not $foundWallet) {
    Write-Host "❌ No wallet found! Please create one first." -ForegroundColor Red
    exit 1
}

# Check RPC configuration
$rpcUrl = $env:SOLANA_RPC_URL
if (-not $rpcUrl) {
    Write-Host "⚠️  Using default RPC. Consider premium RPC for better performance" -ForegroundColor Yellow
} else {
    Write-Host "✅ RPC configured: $rpcUrl" -ForegroundColor Green
}

Write-Host ""
Write-Host "🔥 OPTIMIZATION CHANGES:" -ForegroundColor Red
Write-Host "   📊 70+ token pairs (vs 46 before)" -ForegroundColor Yellow
Write-Host "   ⏰ 10 seconds intervals (vs 15 before)" -ForegroundColor Yellow
Write-Host "   💎 3x fee threshold (vs 8x before)" -ForegroundColor Yellow
Write-Host "   🌊 Higher slippage tolerance for more routes" -ForegroundColor Yellow
Write-Host "   🚀 3 simultaneous executions (vs 2 before)" -ForegroundColor Yellow
Write-Host ""

Write-Host "🎯 READY FOR AGGRESSIVE HUNTING!" -ForegroundColor Green
Write-Host "   Press Enter to start or Ctrl+C to cancel..." -ForegroundColor Cyan
$null = Read-Host

Write-Host ""
Write-Host "🚀 Starting aggressive arbitrage hunter..." -ForegroundColor Green
Write-Host ""

# Build and run
cargo build --release --bin mega_token_hunter
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Build successful, starting aggressive hunter..." -ForegroundColor Green
    cargo run --release --bin mega_token_hunter
} else {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}
