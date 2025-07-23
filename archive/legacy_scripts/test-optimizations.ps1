# Test optimizaciones del arbitrage bot
Write-Host "🚀 TESTING ARBITRAGE BOT CON OPTIMIZACIONES CRITICAS"
Write-Host "═══════════════════════════════════════════════════"
Write-Host ""
Write-Host "📊 Filtros aplicados:"
Write-Host "   - TVL minimo: `$50,000 (reducido desde `$200,000)"
Write-Host "   - Volumen minimo: `$10,000 (reducido desde `$50,000)"
Write-Host "   - Phoenix timeout: 10 segundos"
Write-Host ""
Write-Host "🎯 Resultados esperados:"
Write-Host "   - Pools qualified: 15-50 (vs 2 anterior)"
Write-Host "   - Discovery time: 10-15s (vs 24s anterior)"
Write-Host "   - Phoenix RPC: Sin errores"
Write-Host "   - Opportunities: 3-10 (vs 0 anterior)"
Write-Host ""
Write-Host "🔥 Ejecutando arbitrage bot..."
Write-Host ""

# Ejecutar el bot con input T para testing
"T" | cargo run --bin arbitrage_bot --release
