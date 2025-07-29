#!/usr/bin/env pwsh
# CONFIGURACIÓN MAINNET - Preparar sistema para trading real con datos reales

Write-Host "🌐 CONFIGURANDO SISTEMA PARA MAINNET" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════" -ForegroundColor Yellow

Write-Host "`n💡 VENTAJAS DE MAINNET:" -ForegroundColor Cyan
Write-Host "   ✅ Datos de precios REALES y actualizados" -ForegroundColor Green
Write-Host "   ✅ Liquidez genuina en pools" -ForegroundColor Green
Write-Host "   ✅ Oportunidades de arbitraje auténticas" -ForegroundColor Green
Write-Host "   ✅ Spread y slippage realistas" -ForegroundColor Green
Write-Host "   ✅ Volumen de trading real" -ForegroundColor Green

Write-Host "`n⚠️ CONFIGURACIÓN SEGURA PARA 0.29 SOL:" -ForegroundColor Yellow
Write-Host "   🛡️ Modo simulación para pruebas iniciales" -ForegroundColor White
Write-Host "   🛡️ Umbrales conservadores" -ForegroundColor White
Write-Host "   🛡️ Trades pequeños para minimizar riesgo" -ForegroundColor White

# Backup configuración actual
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
Write-Host "`n💾 Creando backup..." -ForegroundColor Green
Copy-Item "arbitrage_settings.json" "arbitrage_settings_before_mainnet_$timestamp.json"

# Cargar configuración actual
$config = Get-Content "arbitrage_settings.json" | ConvertFrom-Json

Write-Host "`n🔧 APLICANDO CONFIGURACIÓN MAINNET:" -ForegroundColor Green

# 1. CAMBIAR A MAINNET RPCs
Write-Host "   1️⃣ Configurando RPCs mainnet..." -ForegroundColor Yellow
$config.rpc.primary_url = "https://api.mainnet-beta.solana.com"
$config.rpc.backup_urls = @(
    "https://api.mainnet-beta.solana.com",
    "https://rpc.ankr.com/solana",
    "https://solana-api.projectserum.com",
    "https://mainnet.helius-rpc.com/?api-key=public"
)

Write-Host "      • RPC primario: mainnet-beta" -ForegroundColor Green
Write-Host "      • Backups: Ankr, Serum, Helius" -ForegroundColor Green

# 2. CONFIGURACIÓN TRADING CONSERVADORA PARA 0.29 SOL
Write-Host "   2️⃣ Configuración conservadora para capital limitado..." -ForegroundColor Yellow
$config.trading.mode = "simulation"                    # ✅ Simulación inicial
$config.trading.force_real_transactions = $false      # ✅ Seguridad
$config.trading.max_trade_sol = 0.05                  # ✅ Máximo 5% del capital
$config.trading.min_trade_size_sol = 0.01             # ✅ Trades pequeños
$config.trading.min_profit_threshold_sol = 0.002      # ✅ Profit mínimo realista
$config.trading.max_slippage_bps = 100                # ✅ Slippage conservador 1%
$config.trading.military_min_profit_bps = 15          # ✅ 0.15% mínimo

Write-Host "      • Modo: simulación (seguro)" -ForegroundColor Green
Write-Host "      • Max trade: 0.05 SOL (17% del capital)" -ForegroundColor Green
Write-Host "      • Min profit: 0.002 SOL" -ForegroundColor Green
Write-Host "      • Slippage: 1% (conservador)" -ForegroundColor Green

# 3. AJUSTAR PERFORMANCE PARA MAINNET
Write-Host "   3️⃣ Optimizando para mainnet..." -ForegroundColor Yellow
$config.performance.discovery_cycle_delay_seconds = 2  # ✅ Ciclos moderados
$config.performance.latency_target_ms = 400           # ✅ Más estricto
$config.performance.cache_ttl_seconds = 10            # ✅ Cache balanceado
$config.performance.max_concurrent_discoveries = 8    # ✅ Conservador

Write-Host "      • Cycle delay: 2s (balanceado)" -ForegroundColor Green
Write-Host "      • Latency target: 400ms" -ForegroundColor Green
Write-Host "      • Cache TTL: 10s" -ForegroundColor Green

# 4. CONFIGURAR APIs PARA MAINNET
Write-Host "   4️⃣ Configurando APIs mainnet..." -ForegroundColor Yellow
$config.apis.jupiter.enabled = $true
$config.apis.jupiter.timeout_seconds = 5              # ✅ Más tiempo en mainnet
$config.apis.dexscreener.enabled = $true
$config.apis.dexscreener.timeout_seconds = 4
$config.apis.coinbase.enabled = $true
$config.apis.coinbase.timeout_seconds = 6
$config.apis.birdeye.enabled = $true                  # ✅ Más fuentes

Write-Host "      • Jupiter: ✅ 5s timeout" -ForegroundColor Green
Write-Host "      • DexScreener: ✅ 4s timeout" -ForegroundColor Green
Write-Host "      • Coinbase: ✅ 6s timeout" -ForegroundColor Green
Write-Host "      • Birdeye: ✅ Habilitado" -ForegroundColor Green

# 5. MAINNET PROGRAM IDS
Write-Host "   5️⃣ Actualizando program IDs mainnet..." -ForegroundColor Yellow
$mainnetPrograms = @(
    "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",  # Jupiter V6
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",   # Token Program
    "11111111111111111111111111111111",               # System Program
    "ComputeBudget111111111111111111111111111111",    # Compute Budget
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",   # Associated Token
    "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP",   # Orca
    "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc",   # Orca Whirlpool
    "DjVE6JNiYqPL2QXyCUUh8rNjHrbz9hXHNYt99MQ59qw1",   # Orca V2
    "EhpbDdNX2ya45KAR8p9FydBE9MJSJgNnuJNsNc7fzhg7",   # Serum DEX
    "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",   # Serum V3
    "CAMMCzo5YL8w4VFF8KVHrK22GGUQpMNRoyz8rqE5g9Qy",   # Raydium V4
    "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",   # Raydium V4 AMM
    "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1",   # Raydium Stable
    "27haf8L6oxUeXrHrgEgsexjSY5hbVUWEmvv9Nyxg8vQv"    # Phoenix
)

$config.trading.program_id_whitelist = $mainnetPrograms

Write-Host "      • Program whitelist: Actualizada con DEXs mainnet" -ForegroundColor Green

# 6. CONFIGURACIÓN ML PARA MAINNET
Write-Host "   6️⃣ Ajustando ML para datos reales..." -ForegroundColor Yellow
$config.ml_analysis.enabled = $true
$config.ml_analysis.min_score_threshold = 0.3         # ✅ Más estricto para mainnet
$config.ml_analysis.ml_confidence_threshold = 0.6     # ✅ Mayor confidence
$config.ml_analysis.pattern_recognition_enabled = $true

Write-Host "      • ML score threshold: 0.3 (más estricto)" -ForegroundColor Green
Write-Host "      • Confidence: 0.6 (mayor calidad)" -ForegroundColor Green

# 7. CONFIGURACIÓN DE SEGURIDAD
Write-Host "   7️⃣ Configuración de seguridad adicional..." -ForegroundColor Yellow
$config.trading.pre_execution_validation = $true      # ✅ Validación obligatoria
$config.trading.simulation_before_execution = $true   # ✅ Simular antes de ejecutar
$config.trading.max_simulation_retries = 3            # ✅ Más intentos
$config.trading.max_concurrent_trades = 2             # ✅ Conservador

Write-Host "      • Pre-validation: ✅ Habilitada" -ForegroundColor Green
Write-Host "      • Simulation first: ✅ Habilitada" -ForegroundColor Green
Write-Host "      • Max concurrent: 2 (conservador)" -ForegroundColor Green

# Guardar configuración mainnet
$mainnetConfig = $config | ConvertTo-Json -Depth 10
Set-Content "arbitrage_settings.json" $mainnetConfig

Write-Host "`n✅ CONFIGURACIÓN MAINNET APLICADA" -ForegroundColor Green

Write-Host "`n📊 CONFIGURACIÓN MAINNET RESUMEN:" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════" -ForegroundColor Yellow

Write-Host "`n🌐 CONEXIÓN MAINNET:" -ForegroundColor Green
Write-Host "   • RPC: mainnet-beta.solana.com" -ForegroundColor White
Write-Host "   • Backups: Ankr, Serum, Helius" -ForegroundColor White

Write-Host "`n🛡️ CONFIGURACIÓN SEGURA (0.29 SOL):" -ForegroundColor Green
Write-Host "   • Modo: simulación (inicialmente)" -ForegroundColor White
Write-Host "   • Max trade: 0.05 SOL (17% del capital)" -ForegroundColor White
Write-Host "   • Min profit: 0.002 SOL" -ForegroundColor White
Write-Host "   • Slippage máximo: 1%" -ForegroundColor White

Write-Host "`n🎯 PERFORMANCE OPTIMIZADA:" -ForegroundColor Green
Write-Host "   • Cycle delay: 2s (balanceado)" -ForegroundColor White
Write-Host "   • Latency target: 400ms" -ForegroundColor White
Write-Host "   • Cache TTL: 10s" -ForegroundColor White

Write-Host "`n🔗 APIs MAINNET:" -ForegroundColor Green
Write-Host "   • Jupiter V6: ✅ 5s timeout" -ForegroundColor White
Write-Host "   • DexScreener: ✅ 4s timeout" -ForegroundColor White
Write-Host "   • Coinbase: ✅ 6s timeout" -ForegroundColor White
Write-Host "   • Birdeye: ✅ Habilitado" -ForegroundColor White

Write-Host "`n🤖 ML CONFIGURADO:" -ForegroundColor Green
Write-Host "   • Score threshold: 0.3 (mainnet quality)" -ForegroundColor White
Write-Host "   • Confidence: 0.6 (alta calidad)" -ForegroundColor White

Write-Host "`n🚀 PRÓXIMOS PASOS:" -ForegroundColor Yellow
Write-Host "   1. Verificar wallet mainnet:" -ForegroundColor White
Write-Host "      solana config set --url mainnet-beta" -ForegroundColor Gray
Write-Host "      solana balance" -ForegroundColor Gray
Write-Host ""
Write-Host "   2. Ejecutar simulación mainnet:" -ForegroundColor White
Write-Host "      .\diagnostic_simulation.ps1 -DurationMinutes 2" -ForegroundColor Gray
Write-Host ""
Write-Host "   3. Si resultados son buenos, trading real:" -ForegroundColor White
Write-Host "      .\prepare_real_trading_0.29SOL.ps1" -ForegroundColor Gray

Write-Host "`n💾 Backup guardado en:" -ForegroundColor Green
Write-Host "   arbitrage_settings_before_mainnet_$timestamp.json" -ForegroundColor Gray

Write-Host "`n🎉 SISTEMA LISTO PARA MAINNET - DATOS REALES" -ForegroundColor Green
Write-Host "   💰 Capital: 0.29 SOL | Max trade: 0.05 SOL | Modo: Seguro" -ForegroundColor Cyan
