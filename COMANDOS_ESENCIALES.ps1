# 🎯 COMANDOS ESENCIALES - ARBITRAJE REAL EN DEVNET
# Resumen ejecutivo de comandos para ejecutar el bot validado con CLI principal

# ============================================================================
# 🚀 INICIO RÁPIDO (30 SEGUNDOS)
# ============================================================================

# 1. Construir proyecto
cargo build --release

# 2. Verificar wallet y balances
cargo run --bin sniperforge wallet balance --network devnet

# 3. EJECUTAR ARBITRAJE REAL (COMANDO PRINCIPAL)
cargo run --bin sniperforge arbitrage-scan --network devnet

# 4. Verificar ganancias reales
cargo run --bin sniperforge wallet balance --network devnet

# ============================================================================
# 🎯 COMANDOS CLI PRINCIPAL CON --network
# ============================================================================

# Cache-Free Trading Engine (RECOMENDADO)
cargo run --bin sniperforge test cache-free-trading --network devnet

# Arbitrage Scan en tiempo real
cargo run --bin sniperforge arbitrage-scan --network devnet

# Multi-Strategy Trading
cargo run --bin sniperforge multi-strategy-trading --network devnet

# Interactive Trading Mode
cargo run --bin sniperforge interactive --network devnet

# ============================================================================
# 💰 COMANDOS DE ARBITRAJE CON NETWORK
# ============================================================================

# Arbitrage scan básico en DevNet
cargo run --bin sniperforge arbitrage-scan --network devnet

# Arbitrage scan con profit mínimo ($10)
cargo run --bin sniperforge arbitrage-scan --network devnet --min-profit 10.0

# Arbitrage scan con duración específica (5 minutos)
cargo run --bin sniperforge arbitrage-scan --network devnet --duration 300

# Arbitrage scan con exportación de resultados
cargo run --bin sniperforge arbitrage-scan --network devnet --export arbitrage_results.json

# ============================================================================
# 📊 MONITOREO Y VERIFICACIÓN CON NETWORK
# ============================================================================

# Verificar balances de wallet
cargo run --bin sniperforge wallet balance --network devnet

# Solicitar airdrop en DevNet
cargo run --bin sniperforge wallet airdrop --network devnet

# Generar nueva wallet
cargo run --bin sniperforge wallet generate

# Exportar wallet actual
cargo run --bin sniperforge wallet export

# ============================================================================
# � TESTING Y VALIDACIÓN CON NETWORK
# ============================================================================

# Test completo del sistema
cargo run --bin sniperforge test all --network devnet

# Test básico de conectividad
cargo run --bin sniperforge test basic --network devnet

# Test de Jupiter API
cargo run --bin sniperforge test jupiter --network devnet

# Test de Solana RPC
cargo run --bin sniperforge test solana --network devnet

# Test de trading real con wallet
cargo run --bin sniperforge test trade --network devnet

# Cache-Free Trading Test (PRINCIPAL)
cargo run --bin sniperforge test cache-free-trading --network devnet

# ============================================================================
# � PARA MAINNET (REAL MONEY)
# ============================================================================

# Cache-Free Trading en MainNet (REAL MONEY!)
cargo run --bin sniperforge test cache-free-trading --network mainnet

# Arbitrage scan en MainNet
cargo run --bin sniperforge arbitrage-scan --network mainnet

# Verificar balances en MainNet
cargo run --bin sniperforge wallet balance --network mainnet

# Multi-strategy trading en MainNet
cargo run --bin sniperforge multi-strategy-trading --network mainnet

# ============================================================================
# 🎯 COMANDOS AVANZADOS
# ============================================================================

# Pattern Analysis
cargo run --bin sniperforge pattern-analysis --network devnet

# Strategy Backtest
cargo run --bin sniperforge strategy-backtest --strategy arbitrage --network devnet

# ML Pattern Recognition
cargo run --bin sniperforge ml analyze-patterns --network devnet

# ML Trend Prediction
cargo run --bin sniperforge ml predict-trend --network devnet

# ============================================================================
# 🎯 WORKFLOW PASO A PASO RECOMENDADO
# ============================================================================

# Paso 1: Build
cargo build --release

# Paso 2: Verificar setup
cargo run --bin sniperforge wallet balance --network devnet

# Paso 3: Ejecutar cache-free trading (RECOMENDADO)
cargo run --bin sniperforge test cache-free-trading --network devnet

# Paso 4: Verificar resultados
cargo run --bin sniperforge wallet balance --network devnet

# ============================================================================
# 🔍 RESULTADOS ESPERADOS
# ============================================================================

# Después de ejecutar los comandos, deberías ver:
# ✅ "Network selection is required" si olvidas --network
# ✅ Transaction signatures reales en DevNet
# ✅ Balances de tokens incrementados (ganancias reales)
# ✅ Balance SOL reducido por fees reales
# ✅ Mensajes "Trading completed successfully"

# ============================================================================
# 🌐 VERIFICACIÓN ON-CHAIN
# ============================================================================

# Todas las transacciones son verificables en:
# DevNet: https://explorer.solana.com/?cluster=devnet
# MainNet: https://explorer.solana.com/
# SolanaFM: https://solana.fm/

# ============================================================================
# ⚠️ IMPORTANTE: PARÁMETRO --network OBLIGATORIO
# ============================================================================

# ❌ FALLA: cargo run --bin sniperforge test cache-free-trading
# ✅ CORRECTO: cargo run --bin sniperforge test cache-free-trading --network devnet
# ✅ CORRECTO: cargo run --bin sniperforge test cache-free-trading --network mainnet

# ============================================================================
# 📖 AYUDA Y DOCUMENTACIÓN
# ============================================================================

# Ver ayuda general
cargo run --bin sniperforge --help

# Ver ayuda de subcomando
cargo run --bin sniperforge test --help

# Ver ayuda de comando específico
cargo run --bin sniperforge test cache-free-trading --help

Write-Host "📋 Comandos CLI principales cargados." -ForegroundColor Green
Write-Host "🎯 Comando PRINCIPAL: cargo run --bin sniperforge test cache-free-trading --network devnet" -ForegroundColor Cyan
Write-Host "⚠️  IMPORTANTE: Siempre incluir --network devnet o --network mainnet" -ForegroundColor Yellow
