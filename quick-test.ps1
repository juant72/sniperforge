#!/usr/bin/env pwsh
# Script para testing rápido de SniperForge

Write-Host "🚀 SniperForge Ultra-Fast Testing Suite" -ForegroundColor Cyan
Write-Host "=======================================" -ForegroundColor Cyan

Write-Host "`n⚡ Available Tests:" -ForegroundColor Yellow
Write-Host "1. Jupiter Speed Test      - cargo run -- test jupiter-speed" -ForegroundColor White
Write-Host "2. WebSocket RPC Test      - cargo run -- test websocket-rpc" -ForegroundColor White  
Write-Host "3. Syndica Ultra-Fast      - cargo run -- test syndica" -ForegroundColor White
Write-Host "4. Ultimate RPC Compare    - cargo run -- test ultimate-rpc" -ForegroundColor White
Write-Host "5. All Tests               - cargo run -- test all" -ForegroundColor White

Write-Host "`n🎯 Quick Performance Check:" -ForegroundColor Green
$start = Get-Date
cargo check --quiet
$end = Get-Date
$duration = ($end - $start).TotalSeconds
Write-Host "✅ Compilation check: $($duration)s" -ForegroundColor Green

Write-Host "`n🚀 Ready for testing! Choose your command above." -ForegroundColor Cyan
