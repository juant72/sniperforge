# Script de build de producción para SniperForge
Write-Host "🏗️ Construyendo versión de producción..." -ForegroundColor Cyan
cargo clean
cargo build --release
Write-Host "✅ Build de producción completado" -ForegroundColor Green
Write-Host "📦 Binario disponible en: target\release\arbitrage-basic.exe" -ForegroundColor Cyan
