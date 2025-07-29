# Verificador de configuracion final
Write-Host "Verificando configuracion..." -ForegroundColor Cyan

# Test 1: Validar JSON
try {
    $config = Get-Content -Raw -Path "arbitrage_settings.json" | ConvertFrom-Json
    Write-Host "JSON syntax: VALIDO" -ForegroundColor Green
} catch {
    Write-Host "JSON syntax: ERROR - $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Test 2: Verificar secciones principales
$required_sections = @("trading", "wallet", "rpc", "apis", "anti_circular", 
                      "ml_analysis", "performance", "triangular_arbitrage", 
                      "mev_protection", "logging", "target_tokens", 
                      "risk_management", "dashboard", "security")

Write-Host "`nVerificando secciones principales..." -ForegroundColor Yellow
foreach ($section in $required_sections) {
    if ($config.$section) {
        Write-Host "  ✓ $section presente" -ForegroundColor Green
    } else {
        Write-Host "  ✗ $section faltante" -ForegroundColor Red
    }
}

# Test 3: Campos críticos
Write-Host "`nVerificando campos criticos de trading..." -ForegroundColor Yellow
$trading_fields = @("mode", "force_real_transactions", "max_trade_sol", 
                   "trade_timeout_seconds", "max_slippage_bps")

foreach ($field in $trading_fields) {
    if ($null -ne $config.trading.$field) {
        Write-Host "  ✓ trading.$field = $($config.trading.$field)" -ForegroundColor Green
    } else {
        Write-Host "  ✗ trading.$field faltante" -ForegroundColor Red
    }
}

# Test 4: Compilación
Write-Host "`nVerificando compilacion..." -ForegroundColor Yellow
$result = cargo check --bin arbitrage_phase45_clean 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "  ✓ Compilacion exitosa" -ForegroundColor Green
} else {
    Write-Host "  ✗ Error de compilacion" -ForegroundColor Red
}

Write-Host "`nConfiguracion lista para uso!" -ForegroundColor Green
