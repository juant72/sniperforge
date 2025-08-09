# Script de desarrollo rápido para SniperForge
Write-Host "🔄 Iniciando desarrollo en modo watch..." -ForegroundColor Cyan
cargo watch -x "check --all-targets" -x "test" -x "run --bin arbitrage-basic"
