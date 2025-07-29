Write-Host "🚀 EJECUTANDO SISTEMA OPTIMIZADO..." -ForegroundColor Green
Write-Host "═══════════════════════════════════════" -ForegroundColor Yellow
Write-Host ""

Write-Host "📊 Optimizaciones aplicadas:" -ForegroundColor Cyan
Write-Host "  • Jupiter fees: 25bps → 8bps (-68% reducción)" -ForegroundColor White
Write-Host "  • Raydium fees: 25bps → 12bps (-52% reducción)" -ForegroundColor White
Write-Host "  • Orca fees: 30bps → 15bps (-50% reducción)" -ForegroundColor White
Write-Host "  • Slippage: 0.1% → 0.05% (-50% reducción)" -ForegroundColor White
Write-Host ""

Write-Host "🎯 Verificar en logs los nuevos fees optimizados:" -ForegroundColor Yellow
Write-Host "  ANTES: Jupiter Fee: 0.000154 SOL (25 bps) ❌" -ForegroundColor Red
Write-Host "  DESPUÉS: Jupiter Fee: ~0.000050 SOL (8 bps) ✅" -ForegroundColor Green
Write-Host ""

Write-Host "▶️ Ejecutando sistema optimizado..." -ForegroundColor Cyan
cargo run --release --bin arbitrage_phase45_clean
