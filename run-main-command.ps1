#!/usr/bin/env pwsh

# 🎯 SniperForge CLI - Comando Principal
# Este script muestra el comando CLI principal para ejecutar arbitraje real

Write-Host "🚀 SniperForge CLI - Comando Principal" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "⚠️  IMPORTANTE: El parámetro --network es OBLIGATORIO" -ForegroundColor Yellow -BackgroundColor Red
Write-Host ""

Write-Host "🎯 COMANDO PRINCIPAL:" -ForegroundColor Green
Write-Host "cargo run --bin sniperforge test cache-free-trading --network devnet" -ForegroundColor White -BackgroundColor DarkBlue
Write-Host ""

Write-Host "🔥 ESTE COMANDO EJECUTA:" -ForegroundColor Magenta
Write-Host "✅ Cache-Free Trading Engine" -ForegroundColor Green
Write-Host "✅ Arbitraje real con transacciones on-chain" -ForegroundColor Green
Write-Host "✅ Ganancias reales en tokens" -ForegroundColor Green
Write-Host "✅ Fees reales pagados en SOL" -ForegroundColor Green
Write-Host ""

Write-Host "📋 WORKFLOW COMPLETO:" -ForegroundColor Yellow
Write-Host "1. cargo build --release" -ForegroundColor Gray
Write-Host "2. cargo run --bin sniperforge wallet balance --network devnet" -ForegroundColor Gray
Write-Host "3. cargo run --bin sniperforge test cache-free-trading --network devnet" -ForegroundColor Gray
Write-Host "4. cargo run --bin sniperforge wallet balance --network devnet" -ForegroundColor Gray
Write-Host ""

Write-Host "🚀 PARA MAINNET (DINERO REAL):" -ForegroundColor Red
Write-Host "cargo run --bin sniperforge test cache-free-trading --network mainnet" -ForegroundColor White -BackgroundColor DarkRed
Write-Host ""

Write-Host "❌ COMANDO INCORRECTO (FALLA):" -ForegroundColor Red
Write-Host "cargo run --bin sniperforge test cache-free-trading" -ForegroundColor Red
Write-Host "Error: 'Network selection is required. Use --network devnet or --network mainnet'" -ForegroundColor Red
Write-Host ""

Write-Host "✅ COMANDO CORRECTO:" -ForegroundColor Green
Write-Host "cargo run --bin sniperforge test cache-free-trading --network devnet" -ForegroundColor Green
Write-Host ""

Write-Host "📖 VER AYUDA:" -ForegroundColor Cyan
Write-Host "cargo run --bin sniperforge --help" -ForegroundColor Gray
Write-Host "cargo run --bin sniperforge test cache-free-trading --help" -ForegroundColor Gray
Write-Host ""

Write-Host "🎯 ¿QUIERES EJECUTAR EL COMANDO PRINCIPAL AHORA? (y/N): " -ForegroundColor Cyan -NoNewline
$response = Read-Host

if ($response -eq "y" -or $response -eq "Y") {
    Write-Host ""
    Write-Host "🚀 Ejecutando Cache-Free Trading Engine..." -ForegroundColor Green
    Write-Host "Comando: cargo run --bin sniperforge test cache-free-trading --network devnet" -ForegroundColor Gray
    Write-Host ""
    
    # Execute the main command
    cargo run --bin sniperforge test cache-free-trading --network devnet
    
    Write-Host ""
    Write-Host "✅ Comando ejecutado. Verifica los resultados arriba!" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "📋 Para ejecutar manualmente:" -ForegroundColor Yellow
    Write-Host "cargo run --bin sniperforge test cache-free-trading --network devnet" -ForegroundColor White
}

Write-Host ""
Write-Host "🎉 ¡SniperForge CLI listo para arbitraje real!" -ForegroundColor Green
