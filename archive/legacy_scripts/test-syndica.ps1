#!/usr/bin/env pwsh
# Test Syndica Ultra-Fast WebSocket Performance

Write-Host "⚡ Setting up Syndica Ultra-Fast WebSocket Test" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan

# Set Syndica token
$env:SYNDICA_TOKEN = "4gJVJtRPS6J2MMWPasUfQHitRZCzQShiJUtKFBTZgXgqmcyCnyVdRVZ1wcjYKkCF83MNSVyP12EDeYJgFMr3zqQjdArFmPXRwmT"

Write-Host "✅ Syndica token configured" -ForegroundColor Green
Write-Host "🎯 Target: 5-15ms latency (vs 50ms HTTP current)" -ForegroundColor Yellow

Write-Host "`n🚀 Running Syndica test..." -ForegroundColor Cyan
cargo run -- test syndica

Write-Host "`n📊 Expected Results:" -ForegroundColor Yellow
Write-Host "   • Connection: < 100ms" -ForegroundColor White
Write-Host "   • Price updates: 5-15ms" -ForegroundColor White  
Write-Host "   • Improvement: 3-10x faster than HTTP" -ForegroundColor White
