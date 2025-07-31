# Script de testing completo para SniperForge
Write-Host "🧪 Ejecutando suite de tests completa..." -ForegroundColor Cyan
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt -- --check
cargo audit
