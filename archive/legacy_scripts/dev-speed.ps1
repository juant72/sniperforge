# 🚀 SniperForge Ultra-Speed Development Script
# Optimizado para iteraciones de < 200ms

param(
    [string]$Command = "jupiter-speed",
    [switch]$Clean = $false
)

Write-Host "⚡ ULTRA-SPEED DEV MODE" -ForegroundColor Magenta
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta

# Optimizaciones de entorno para velocidad máxima
$env:CARGO_BUILD_JOBS = "16"
$env:CARGO_BUILD_PIPELINING = "true"
$env:CARGO_INCREMENTAL = "1"
$env:RUSTC_WRAPPER = "sccache"

if ($Clean) {
    Write-Host "🧹 Quick clean..." -ForegroundColor Yellow
    cargo clean -q
}

Write-Host "⚡ Ultra-fast check + build + run..." -ForegroundColor Cyan
$totalTime = Measure-Command {
    # Check ultra-rápido
    $checkTime = Measure-Command { 
        cargo check -q 2>&1 | Out-Null 
    }
    
    # Build ultra-rápido (solo binarios necesarios)
    $buildTime = Measure-Command { 
        cargo build -q --bin sniperforge 2>&1 | Out-Null 
    }
    
    # Run el comando
    $runTime = Measure-Command {
        cargo run -q -- test $Command 2>&1
    }
    
    Write-Host ""
    Write-Host "📊 Performance Breakdown:" -ForegroundColor Green
    Write-Host "  ⚡ Check: $($checkTime.TotalMilliseconds.ToString('F0'))ms" -ForegroundColor Yellow
    Write-Host "  🔨 Build: $($buildTime.TotalMilliseconds.ToString('F0'))ms" -ForegroundColor Yellow  
    Write-Host "  🏃 Run: $($runTime.TotalMilliseconds.ToString('F0'))ms" -ForegroundColor Yellow
}

Write-Host "🎯 Total Dev Cycle: $($totalTime.TotalMilliseconds.ToString('F0'))ms" -ForegroundColor White

# Performance targets para trading bot
if ($totalTime.TotalMilliseconds -lt 500) {
    Write-Host "🏆 EXCELLENT: Ultra-fast dev cycle!" -ForegroundColor Green
} elseif ($totalTime.TotalMilliseconds -lt 1000) {
    Write-Host "✅ GOOD: Fast enough for rapid iteration" -ForegroundColor Yellow
} else {
    Write-Host "⚠️  SLOW: Needs more optimization" -ForegroundColor Red
}

Write-Host ""
Write-Host "📈 sccache stats:" -ForegroundColor Blue
sccache --show-stats --stats-format=json | ConvertFrom-Json | ForEach-Object {
    Write-Host "  Cache hits: $($_.stats.cache_hits)" -ForegroundColor Green
    Write-Host "  Cache misses: $($_.stats.cache_misses)" -ForegroundColor Yellow
    Write-Host "  Hit rate: $(($_.stats.cache_hits / ($_.stats.cache_hits + $_.stats.cache_misses) * 100).ToString('F1'))%" -ForegroundColor Cyan
}
