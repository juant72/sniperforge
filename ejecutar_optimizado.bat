@echo off
echo 🚀 EJECUTANDO SISTEMA OPTIMIZADO...
echo ═══════════════════════════════════════
echo.
echo 📊 Optimizaciones aplicadas:
echo   • Jupiter fees: 25bps → 8bps (-68%% reducción)
echo   • Raydium fees: 25bps → 12bps (-52%% reducción)
echo   • Orca fees: 30bps → 15bps (-50%% reducción)
echo   • Slippage: 0.1%% → 0.05%% (-50%% reducción)
echo.
echo 🎯 Verificar en logs los nuevos fees optimizados:
echo   ANTES: Jupiter Fee: 0.000154 SOL (25 bps) ❌
echo   DESPUÉS: Jupiter Fee: ~0.000050 SOL (8 bps) ✅
echo.
echo ▶️ Ejecutando...
cargo run --release --bin arbitrage_phase45_clean
