# Paper Trading Test Script
# Tests paper trading with mainnet data

Write-Host "📊 PAPER TRADING WITH MAINNET DATA" -ForegroundColor Cyan
Write-Host "===================================" -ForegroundColor Cyan

# Ensure we're in the right directory
Set-Location "c:\work\encrypia\labs\sniperforge"

# Set environment variables for mainnet
$env:RUST_LOG = "info"
$env:JUPITER_BASE_URL = "https://quote-api.jup.ag"

Write-Host "🔧 Environment configured for mainnet paper trading" -ForegroundColor Yellow

# Run the paper trading test
Write-Host "🚀 Starting paper trading test..." -ForegroundColor Green
cargo run -- test paper-trading

Write-Host "✅ Paper trading test completed!" -ForegroundColor Green
