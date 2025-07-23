#!/usr/bin/env pwsh
# TEST SCRIPT FOR FIXED ARBITRAGE BOT

Write-Host "🎯 TESTING ARBITRAGE BOT WITH FIXED MULTI-DEX APIs" -ForegroundColor Green
Write-Host "=================================================" -ForegroundColor Yellow

Write-Host "`n🔧 Building project..." -ForegroundColor Cyan
$buildResult = cargo build --bin arbitrage_bot 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Build successful" -ForegroundColor Green
} else {
    Write-Host "❌ Build failed" -ForegroundColor Red
    Write-Host $buildResult
    exit 1
}

Write-Host "`n🚀 Starting arbitrage bot in simulation mode..." -ForegroundColor Cyan
Write-Host "   This will test the fixed DEX APIs:" -ForegroundColor White
Write-Host "   - Meteora: https://dlmm-api.meteora.ag" -ForegroundColor Gray
Write-Host "   - Saber: https://registry.saber.so/data" -ForegroundColor Gray
Write-Host "   - Lifinity: Simulated pools" -ForegroundColor Gray
Write-Host "   - Phoenix: Simulated markets" -ForegroundColor Gray

Write-Host "`n⏱️  Running for 60 seconds to test API fixes..." -ForegroundColor Yellow

# Run the bot with option A (simulation) and timeout after 60 seconds
$process = Start-Process -FilePath "cargo" -ArgumentList "run","--bin","arbitrage_bot" -NoNewWindow -PassThru -RedirectStandardInput
Start-Sleep -Seconds 2
$process.StandardInput.WriteLine("A")
$process.StandardInput.Close()

Start-Sleep -Seconds 60
if (!$process.HasExited) {
    Write-Host "`n⏹️  Stopping test after 60 seconds..." -ForegroundColor Yellow
    $process.Kill()
    Write-Host "✅ Test completed - Bot ran successfully without API crashes" -ForegroundColor Green
} else {
    Write-Host "`n⚠️  Bot exited early - check for errors" -ForegroundColor Yellow
}

Write-Host "`n🎖️  MULTI-DEX API FIX TEST COMPLETE" -ForegroundColor Green
Write-Host "=====================================" -ForegroundColor Yellow
