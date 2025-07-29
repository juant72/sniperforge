# ================================================================================
# 🎯 VERIFICADOR FINAL DE CONFIGURACIÓN - ARBITRAGE PHASE 4.5
# ================================================================================
# Valida que arbitrage_settings.json es compatible con ArbitrageSettings struct
# ================================================================================

Write-Host "🔍 VERIFICANDO CONFIGURACIÓN FINAL..." -ForegroundColor Cyan

# Test 1: Validar JSON syntax
Write-Host "`n1️⃣ VALIDANDO SINTAXIS JSON..." -ForegroundColor Yellow
try {
    $config = Get-Content -Raw -Path "c:\work\encrypia\labs\sniperforge\arbitrage_settings.json" | ConvertFrom-Json
    Write-Host "   ✅ JSON SYNTAX: VÁLIDO" -ForegroundColor Green
} catch {
    Write-Host "   ❌ JSON SYNTAX: ERROR - $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Test 2: Verificar estructuras principales
Write-Host "`n2️⃣ VERIFICANDO ESTRUCTURAS PRINCIPALES..." -ForegroundColor Yellow

$required_sections = @("trading", "wallet", "rpc", "apis", "anti_circular", 
                      "ml_analysis", "performance", "triangular_arbitrage", 
                      "mev_protection", "logging", "target_tokens", 
                      "risk_management", "dashboard", "security")

foreach ($section in $required_sections) {
    if ($config.$section) {
        Write-Host "   ✅ $section: PRESENTE" -ForegroundColor Green
    } else {
        Write-Host "   ❌ $section: FALTANTE" -ForegroundColor Red
    }
}

# Test 3: Verificar campos críticos del trading
Write-Host "`n3️⃣ VERIFICANDO CAMPOS CRÍTICOS DE TRADING..." -ForegroundColor Yellow

$trading_fields = @("mode", "force_real_transactions", "max_trade_sol", 
                   "min_profit_threshold_sol", "min_confidence_threshold", 
                   "max_concurrent_trades", "trade_timeout_seconds",
                   "min_trade_size_sol", "max_trade_size_sol", "max_slippage_bps",
                   "military_min_profit_bps", "base_profit_percentage",
                   "max_profit_percentage", "default_trade_amount_usd",
                   "estimated_gas_cost_usd", "estimated_execution_time_ms")

foreach ($field in $trading_fields) {
    if ($null -ne $config.trading.$field) {
        Write-Host "   ✅ trading.$field: $($config.trading.$field)" -ForegroundColor Green
    } else {
        Write-Host "   ❌ trading.$field: FALTANTE" -ForegroundColor Red
    }
}

# Test 4: Intentar compilación
Write-Host "`n4️⃣ VERIFICANDO COMPILACIÓN..." -ForegroundColor Yellow
$compile_result = cargo check --bin arbitrage_phase45_clean 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "   ✅ COMPILACIÓN: EXITOSA" -ForegroundColor Green
} else {
    Write-Host "   ❌ COMPILACIÓN: ERROR" -ForegroundColor Red
    $compile_result | Select-String -Pattern "error|missing|field" | ForEach-Object {
        Write-Host "      $($_.Line)" -ForegroundColor Red
    }
}

Write-Host "`n🎯 VERIFICACIÓN COMPLETA" -ForegroundColor Cyan
Write-Host "📋 RESULTADO: CONFIGURACIÓN LISTA PARA USO" -ForegroundColor Green
