# 🚀 SniperForge - Build Ultrarrápido
# Optimizado para iteraciones de < 500ms

Write-Host "⚡ SniperForge Ultrafast Build Script" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()

# Verificar sccache
Write-Host "🔍 Checking sccache status..." -ForegroundColor Yellow
sccache --show-stats | Out-Null
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ sccache is active" -ForegroundColor Green
} else {
    Write-Host "⚠️  sccache not found, using standard cache" -ForegroundColor Yellow
}

# Limpiar builds anteriores para medición limpia
Write-Host "🧹 Cleaning previous artifacts..." -ForegroundColor Yellow
cargo clean -q

Write-Host "⚡ Starting ultrafast build..." -ForegroundColor Cyan

# Build con máximas optimizaciones de velocidad
$buildTime = Measure-Command {
    cargo build --profile=dev 2>&1 | Out-String
}

$stopwatch.Stop()

Write-Host ""
Write-Host "📊 Build Results:" -ForegroundColor Green
Write-Host "  ⏱️  Total time: $($buildTime.TotalSeconds.ToString('F2'))s" -ForegroundColor White
Write-Host "  🚀 Target: < 1.0s (goal: < 0.5s)" -ForegroundColor Yellow

if ($buildTime.TotalSeconds -lt 1.0) {
    Write-Host "🎯 SUCCESS: Ultra-fast build achieved!" -ForegroundColor Green
} elseif ($buildTime.TotalSeconds -lt 2.0) {
    Write-Host "⚡ GOOD: Fast build, room for improvement" -ForegroundColor Yellow
} else {
    Write-Host "⚠️  SLOW: Build needs optimization" -ForegroundColor Red
}

Write-Host ""
Write-Host "🧪 Testing quick run..." -ForegroundColor Cyan
$runTime = Measure-Command {
    cargo run -- --help | Out-Null
}
Write-Host "  🏃 Run time: $($runTime.TotalSeconds.ToString('F2'))s" -ForegroundColor White

Write-Host ""
Write-Host "📈 sccache final stats:" -ForegroundColor Blue
sccache --show-stats

Write-Host ""
Write-Host "✅ Ultrafast build script completed!" -ForegroundColor Green
