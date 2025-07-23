# 🎯 COMANDOS ESENCIALES - ARBITRAJE REAL EN DEVNET
# Comandos CLI FUNCIONALES de SniperForge

# ============================================================================
# 🚀 COMANDOS QUE FUNCIONAN REALMENTE
# ============================================================================

# 1. Construir proyecto
cargo build --release

# 2. Ver ayuda general
cargo run --bin sniperforge -- --help

# 3. Verificar balances de wallet (usa wallet desde .env)
cargo run --bin sniperforge -- wallet balance

# 4. EJECUTAR SWAP REAL EN DEVNET (COMANDO PRINCIPAL)
cargo run --bin sniperforge -- test swap-real --network devnet --confirm

# 5. Solicitar airdrop en DevNet
cargo run --bin sniperforge -- wallet airdrop

# ============================================================================
# 🎯 COMANDOS CLI CORRECTOS (VERIFICADOS)
# ============================================================================

# Wallet Management (FUNCIONA)
cargo run --bin sniperforge -- wallet balance              # Ver balances
cargo run --bin sniperforge -- wallet airdrop             # Solicitar SOL
cargo run --bin sniperforge -- wallet generate            # Generar wallet

# Test Suite (FUNCIONA)
cargo run --bin sniperforge -- test all                   # Todos los tests
cargo run --bin sniperforge -- test basic                 # Test básico
cargo run --bin sniperforge -- test jupiter              # Test Jupiter API
cargo run --bin sniperforge -- test solana               # Test Solana RPC

# SWAP REAL - COMANDO PRINCIPAL (FUNCIONA)
cargo run --bin sniperforge -- test swap-real --network devnet           # Simulación
cargo run --bin sniperforge -- test swap-real --network devnet --confirm # REAL!
cargo run --bin sniperforge -- test swap-real --network mainnet --confirm # MAINNET!

# ============================================================================
# 💰 TRADING REAL PASO A PASO
# ============================================================================

# Paso 1: Verificar setup
cargo run --bin sniperforge -- wallet balance

# Paso 2: Ejecutar swap real en DevNet (SEGURO)
cargo run --bin sniperforge -- test swap-real --network devnet --confirm

# Paso 3: Verificar cambios en balance
cargo run --bin sniperforge -- wallet balance

# ============================================================================
# 🚀 PARA MAINNET (DINERO REAL!)
# ============================================================================

# ⚠️ ADVERTENCIA: Estos comandos usan DINERO REAL
cargo run --bin sniperforge -- test swap-real --network mainnet --confirm --amount 0.001
cargo run --bin sniperforge -- wallet balance  # Verificar en MainNet

# ============================================================================
# 🔧 COMANDOS ADICIONALES FUNCIONALES
# ============================================================================

# Platform Status
cargo run --bin sniperforge -- status

# Configuration
cargo run --bin sniperforge -- config

# Interactive Mode
cargo run --bin sniperforge -- interactive

# ============================================================================
# ⚠️ COMANDOS QUE NO FUNCIONAN TODAVÍA
# ============================================================================

# ❌ ESTOS NO ESTÁN IMPLEMENTADOS:
# cargo run --bin sniperforge -- arbitrage-scan --network devnet
# cargo run --bin sniperforge -- test cache-free-trading --network devnet
# cargo run --bin sniperforge -- multi-strategy-trading --network devnet

# ============================================================================
# 🎯 WORKFLOW RECOMENDADO (COMANDOS REALES)
# ============================================================================

# 1. Build
cargo build --release

# 2. Verificar wallet
cargo run --bin sniperforge -- wallet balance

# 3. Solicitar SOL si es necesario
cargo run --bin sniperforge -- wallet airdrop

# 4. EJECUTAR TRADING REAL (COMANDO PRINCIPAL QUE FUNCIONA)
cargo run --bin sniperforge -- test swap-real --network devnet --confirm

# 5. Verificar ganancias
cargo run --bin sniperforge -- wallet balance

# ============================================================================
# 🔍 RESULTADOS ESPERADOS
# ============================================================================

# Con --confirm verás:
# ✅ Transaction signatures reales
# ✅ Cambios en balances de tokens
# ✅ Fees reales pagados en SOL
# ✅ "Transaction completed successfully"

# Sin --confirm verás:
# ✅ Simulación del swap
# ✅ Precios y quotes de Jupiter
# ✅ "Simulation completed"

# ============================================================================
# 📖 AYUDA DISPONIBLE
# ============================================================================

# Ver todos los comandos
cargo run --bin sniperforge -- --help

# Ayuda específica
cargo run --bin sniperforge -- wallet --help
cargo run --bin sniperforge -- test --help
cargo run --bin sniperforge -- test swap-real --help

Write-Host "📋 Comandos CLI FUNCIONALES cargados." -ForegroundColor Green
Write-Host "🎯 COMANDO PRINCIPAL REAL: cargo run --bin sniperforge -- test swap-real --network devnet --confirm" -ForegroundColor Cyan
Write-Host "⚠️  Con --confirm ejecuta transacciones REALES!" -ForegroundColor Yellow

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
