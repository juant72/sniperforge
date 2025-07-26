#!/usr/bin/env pwsh
# ========================================
# QUICK START - REAL TRADING
# ========================================
# Inicio rápido sin confirmaciones
# USAR SOLO EN ENTORNO CONTROLADO
# ========================================

Write-Host "🚀 QUICK START - TRADING REAL ACTIVADO" -ForegroundColor Green
Write-Host "⚡ Configuración: Modo profesional corporativo" -ForegroundColor Cyan

# Set environment for real trading
$env:FORCE_REAL_TRANSACTIONS = "true"
$env:MAX_TRADE_SOL = "0.005"
$env:MIN_PROFIT_BPS = "10"
$env:TRADING_MODE = "professional"
$env:RUST_LOG = "info"
$env:ENABLE_MEV_PROTECTION = "true"
$env:ENABLE_SANDWICH_DETECTION = "true"
$env:ENABLE_SLIPPAGE_PROTECTION = "true"
$env:MAX_SLIPPAGE_BPS = "100"
$env:ENABLE_PROFIT_TRACKING = "true"

Write-Host "💰 Max Trade: 0.005 SOL | Min Profit: 0.10% | MEV Protection: ON" -ForegroundColor Yellow

# Quick compilation check
if (!(Test-Path "target/release/arbitrage_phase45_clean.exe")) {
    Write-Host "⚙️ Compilando sistema..." -ForegroundColor Blue
    cargo build --release --bin arbitrage_phase45_clean
}

Write-Host "🔥 INICIANDO SISTEMA REAL..." -ForegroundColor Red
Write-Host ""

# Start immediately
cargo run --release --bin arbitrage_phase45_clean
