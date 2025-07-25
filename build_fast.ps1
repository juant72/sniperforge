# COMPILACIÓN RÁPIDA - SNIPERFORGE ARBITRAGE
# Uso: .\build_fast.ps1

Write-Host "🚀 SNIPERFORGE - COMPILACIÓN RÁPIDA" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════" -ForegroundColor Cyan

# Limpiar builds anteriores para forzar recompilación incremental
Write-Host "🧹 Limpiando build cache..." -ForegroundColor Yellow
cargo clean --quiet

# Compilación con optimizaciones para desarrollo rápido
Write-Host "⚡ Compilando con optimizaciones de velocidad..." -ForegroundColor Green

$env:CARGO_INCREMENTAL = "1"
$env:RUSTFLAGS = "-C target-cpu=native"

# Compilar solo lo necesario
cargo build --bin arbitrage_bot_phase45_final --features basic --release

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ COMPILACIÓN EXITOSA!" -ForegroundColor Green
    Write-Host "🎯 Ejecutar con: cargo run --bin arbitrage_bot_phase45_final --release" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "📊 TIPS PARA BUILDS MÁS RÁPIDOS:" -ForegroundColor Yellow
    Write-Host "• Use 'cargo check' para verificación rápida sin compilar" -ForegroundColor White
    Write-Host "• Use 'cargo build' solo para debug, 'cargo run' compila y ejecuta" -ForegroundColor White
    Write-Host "• Evite 'cargo clean' a menos que sea necesario" -ForegroundColor White
    Write-Host "• Use '--release' solo para versión final" -ForegroundColor White
} else {
    Write-Host "❌ ERROR EN COMPILACIÓN" -ForegroundColor Red
    Write-Host "💡 Revisar errores arriba" -ForegroundColor Yellow
}
