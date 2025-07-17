#!/usr/bin/env pwsh

Write-Host "🚀 === REAL ARBITRAGE MEGA TOKEN HUNTER ===" -ForegroundColor Green
Write-Host "   ⚠️  WARNING: REAL MONEY EXECUTION" -ForegroundColor Yellow
Write-Host "   💰 Will execute actual trades on MainNet" -ForegroundColor Yellow
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
    Write-Host "⚠️  Using default RPC. Consider setting SOLANA_RPC_URL" -ForegroundColor Yellow
} else {
    Write-Host "✅ RPC configured: $rpcUrl" -ForegroundColor Green
}

Write-Host ""
Write-Host "🔥 FINAL WARNING: This will execute REAL transactions!" -ForegroundColor Red
Write-Host "   Press Ctrl+C within 10 seconds to cancel..." -ForegroundColor Yellow
Write-Host ""

# 10 second countdown
for ($i = 10; $i -gt 0; $i--) {
    Write-Host "   Starting in $i seconds..." -ForegroundColor Yellow
    Start-Sleep 1
}

Write-Host ""
Write-Host "🚀 Starting REAL arbitrage execution..." -ForegroundColor Green
Write-Host ""

# Build and run
cargo build --release --bin mega_token_hunter
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Build successful, starting hunter..." -ForegroundColor Green
    cargo run --release --bin mega_token_hunter
} else {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}
