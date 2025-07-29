#!/usr/bin/env pwsh
# APLICAR OPTIMIZACIONES DEVNET - Configuración optimizada para detectar más oportunidades

Write-Host "🔧 APLICANDO OPTIMIZACIONES PARA DEVNET" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════" -ForegroundColor Yellow

Write-Host "`n📊 ANÁLISIS DE SIMULACIÓN ANTERIOR:" -ForegroundColor Cyan
Write-Host "   ✅ Cross-chain: 182+ SOL profit (FUNCIONA PERFECTAMENTE)" -ForegroundColor Green
Write-Host "   ✅ Flash loans: 47+ SOL profit (FUNCIONA)" -ForegroundColor Green
Write-Host "   ❌ Swaps tradicionales: 0 oportunidades (NECESITA OPTIMIZACIÓN)" -ForegroundColor Red
Write-Host "   ❌ Triangular: 0 oportunidades (NECESITA OPTIMIZACIÓN)" -ForegroundColor Red

# Backup actual
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
Write-Host "`n💾 Creando backup de configuración actual..." -ForegroundColor Green
Copy-Item "arbitrage_settings.json" "arbitrage_settings_before_optimization_$timestamp.json"

# Cargar configuración actual
$config = Get-Content "arbitrage_settings.json" | ConvertFrom-Json

Write-Host "`n🎯 APLICANDO OPTIMIZACIONES ESPECÍFICAS:" -ForegroundColor Green

# 1. REDUCIR UMBRALES PARA DEVNET
Write-Host "   1️⃣ Ajustando umbrales para devnet..." -ForegroundColor Yellow
$config.trading.min_profit_threshold_sol = 0.0001      # ✅ 10x más sensible
$config.trading.military_min_profit_bps = 5            # ✅ 0.05% más realista  
$config.trading.min_confidence_threshold = 0.50        # ✅ 50% para devnet
$config.trading.max_trade_sol = 0.5                    # ✅ Más capital
$config.trading.max_slippage_bps = 300                 # ✅ Más tolerancia
$config.trading.base_profit_percentage = 0.005         # ✅ 0.5% base más bajo

Write-Host "      • Min profit: 0.001 → 0.0001 SOL (10x más sensible)" -ForegroundColor Green
Write-Host "      • Min profit bps: 20 → 5 bps (4x más tolerante)" -ForegroundColor Green
Write-Host "      • Confidence: 75% → 50% (más oportunidades)" -ForegroundColor Green
Write-Host "      • Max trade: 0.08 → 0.5 SOL (6x más capital)" -ForegroundColor Green

# 2. OPTIMIZAR PERFORMANCE DISCOVERY
Write-Host "   2️⃣ Optimizando velocidad de discovery..." -ForegroundColor Yellow
$config.performance.discovery_cycle_delay_seconds = 1   # ✅ Ciclos más rápidos
$config.performance.latency_target_ms = 500            # ✅ Target más realista
$config.performance.cache_ttl_seconds = 5              # ✅ Cache más agresivo
$config.performance.max_concurrent_discoveries = 15    # ✅ Más paralelismo

Write-Host "      • Cycle delay: 3s → 1s (3x más rápido)" -ForegroundColor Green
Write-Host "      • Latency target: 250ms → 500ms (más realista)" -ForegroundColor Green
Write-Host "      • Cache TTL: 15s → 5s (más fresco)" -ForegroundColor Green

# 3. HABILITAR MÁS APIs
Write-Host "   3️⃣ Habilitando más fuentes de datos..." -ForegroundColor Yellow
$config.apis.birdeye.enabled = $true
$config.apis.birdeye.timeout_seconds = 5
$config.apis.dexscreener.rate_limit_per_sec = 15       # ✅ Más requests
$config.apis.jupiter.timeout_seconds = 3               # ✅ Más tiempo para devnet

Write-Host "      • Birdeye API: Habilitada" -ForegroundColor Green
Write-Host "      • DexScreener rate limit: 10 → 15 req/s" -ForegroundColor Green
Write-Host "      • Jupiter timeout: 2s → 3s" -ForegroundColor Green

# 4. OPTIMIZAR TRIANGULAR ARBITRAGE
Write-Host "   4️⃣ Optimizando arbitraje triangular..." -ForegroundColor Yellow
$config.triangular_arbitrage.min_profit_percentage = 0.5   # ✅ 0.5% más bajo
$config.triangular_arbitrage.max_amount_sol = 1.0         # ✅ Más capital
$config.triangular_arbitrage.adaptive_min_profit = 0.3     # ✅ Adaptativo más agresivo

Write-Host "      • Min profit triangular: 2% → 0.5% (4x más sensible)" -ForegroundColor Green
Write-Host "      • Max amount: 50 → 1.0 SOL" -ForegroundColor Green

# 5. AJUSTAR ML Y ANTI-CIRCULAR
Write-Host "   5️⃣ Ajustando sistemas ML y detección..." -ForegroundColor Yellow
$config.ml_analysis.min_score_threshold = 0.1          # ✅ Más tolerante
$config.ml_analysis.ml_confidence_threshold = 0.4      # ✅ Más oportunidades
$config.anti_circular.max_same_token_repeats = 2       # ✅ Más flexibilidad

Write-Host "      • ML score threshold: 0.2 → 0.1" -ForegroundColor Green
Write-Host "      • ML confidence: 0.6 → 0.4" -ForegroundColor Green

# 6. CONFIGURACIÓN ESPECÍFICA DEVNET
Write-Host "   6️⃣ Configuración específica para devnet..." -ForegroundColor Yellow

# Añadir más program IDs si no existen
$devnetPrograms = @(
    "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",  # Jupiter
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",   # Token Program
    "11111111111111111111111111111111",               # System Program
    "ComputeBudget111111111111111111111111111111",    # Compute Budget
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",   # Associated Token
    "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP",   # Orca
    "DjVE6JNiYqPL2QXyCUUh8rNjHrbz9hXHNYt99MQ59qw1",   # Orca V2
    "EhpbDdNX2ya45KAR8p9FydBE9MJSJgNnuJNsNc7fzhg7"    # Serum DEX
)

$config.trading.program_id_whitelist = $devnetPrograms

Write-Host "      • Program whitelist: Actualizada con DEXs devnet" -ForegroundColor Green

# Guardar configuración optimizada
$optimizedConfig = $config | ConvertTo-Json -Depth 10
Set-Content "arbitrage_settings.json" $optimizedConfig

Write-Host "`n✅ OPTIMIZACIONES APLICADAS EXITOSAMENTE" -ForegroundColor Green

Write-Host "`n📊 CONFIGURACIÓN OPTIMIZADA:" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════" -ForegroundColor Yellow

Write-Host "`n🎯 UMBRALES OPTIMIZADOS:" -ForegroundColor Green
Write-Host "   • Min profit threshold: 0.0001 SOL (10x más sensible)" -ForegroundColor White
Write-Host "   • Min profit bps: 5 bps (0.05%)" -ForegroundColor White
Write-Host "   • Confidence threshold: 50%" -ForegroundColor White
Write-Host "   • Max trade size: 0.5 SOL" -ForegroundColor White
Write-Host "   • Max slippage: 300 bps (3%)" -ForegroundColor White

Write-Host "`n⚡ PERFORMANCE OPTIMIZADO:" -ForegroundColor Green
Write-Host "   • Discovery cycle: 1 segundo" -ForegroundColor White
Write-Host "   • Latency target: 500ms" -ForegroundColor White
Write-Host "   • Cache TTL: 5 segundos" -ForegroundColor White
Write-Host "   • Concurrent discoveries: 15" -ForegroundColor White

Write-Host "`n🌐 APIs MEJORADAS:" -ForegroundColor Green
Write-Host "   • Birdeye: ✅ Habilitada" -ForegroundColor White
Write-Host "   • DexScreener: 15 req/s" -ForegroundColor White
Write-Host "   • Jupiter: 3s timeout" -ForegroundColor White

Write-Host "`n🔺 TRIANGULAR OPTIMIZADO:" -ForegroundColor Green
Write-Host "   • Min profit: 0.5% (4x más sensible)" -ForegroundColor White
Write-Host "   • Max amount: 1.0 SOL" -ForegroundColor White
Write-Host "   • Adaptive threshold: 0.3%" -ForegroundColor White

Write-Host "`n🤖 ML AJUSTADO:" -ForegroundColor Green
Write-Host "   • Score threshold: 0.1 (más tolerante)" -ForegroundColor White
Write-Host "   • Confidence: 0.4 (más oportunidades)" -ForegroundColor White

Write-Host "`n🚀 PRÓXIMO PASO:" -ForegroundColor Yellow
Write-Host "   Ejecutar nueva simulación:" -ForegroundColor White
Write-Host "   .\diagnostic_simulation.ps1 -DurationMinutes 3" -ForegroundColor Gray

Write-Host "`n💾 Backup guardado en:" -ForegroundColor Green
Write-Host "   arbitrage_settings_before_optimization_$timestamp.json" -ForegroundColor Gray

Write-Host "`n✨ SISTEMA OPTIMIZADO PARA DETECTAR MÁS OPORTUNIDADES" -ForegroundColor Green
