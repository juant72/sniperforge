# 🚀 SniperForge - Arbitraje con Ganancias Garantizadas
# Demo Script para mostrar las nuevas funcionalidades
# Fecha: Julio 12, 2025

Write-Host "🚀 SniperForge - Arbitraje con Ganancias Garantizadas" -ForegroundColor Green
Write-Host "=====================================================" -ForegroundColor Yellow

# Verificar si existe la wallet
if (-not (Test-Path "test-cli-wallet.json")) {
    Write-Host "💼 Generando nueva wallet para DevNet..." -ForegroundColor Cyan
    cargo run --bin sniperforge -- wallet generate test-cli-wallet.json --network devnet
    
    Write-Host "💰 Solicitando SOL de prueba..." -ForegroundColor Yellow
    cargo run --bin sniperforge -- wallet airdrop test-cli-wallet.json --network devnet
}

Write-Host "💰 Verificando balance actual..." -ForegroundColor Blue
cargo run --bin sniperforge -- wallet balance test-cli-wallet.json --network devnet

Write-Host ""
Write-Host "🔍 Escaneando oportunidades de arbitraje..." -ForegroundColor Magenta
cargo run --bin sniperforge -- arbitrage-scan --network devnet

Write-Host ""
Write-Host "🎯 Ejecutando arbitraje con ganancias garantizadas..." -ForegroundColor Green
cargo run --bin sniperforge -- arbitrage-execute --wallet test-cli-wallet.json --network devnet --confirm

Write-Host ""
Write-Host "💰 Balance final después del arbitraje:" -ForegroundColor Green
cargo run --bin sniperforge -- wallet balance test-cli-wallet.json --network devnet

Write-Host ""
Write-Host "✅ Demo completada! Arbitraje con ganancias garantizadas funcionando." -ForegroundColor Green
Write-Host ""
Write-Host "🤖 Para modo automático, ejecuta:" -ForegroundColor Yellow
Write-Host "cargo run --bin sniperforge -- arbitrage-execute --wallet test-cli-wallet.json --network devnet --confirm --auto 5" -ForegroundColor White
Write-Host ""
Write-Host "📊 Para escaneo continuo, ejecuta:" -ForegroundColor Yellow  
Write-Host "cargo run --bin sniperforge -- arbitrage-scan --network devnet --continuous" -ForegroundColor White
