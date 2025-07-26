# 🔍 DIAGNÓSTICO EXHAUSTIVO - DETECCIÓN DE OPORTUNIDADES
Write-Host "🔍 DIAGNÓSTICO EXHAUSTIVO - DETECCIÓN DE OPORTUNIDADES" -ForegroundColor Cyan

# Paso 1: Verificar precios obtenidos de APIs
Write-Host "`n📡 PASO 1: Verificación de APIs de precios" -ForegroundColor Yellow

# Test DexScreener API directamente
Write-Host "🔍 Testing DexScreener API..." -ForegroundColor Gray
try {
    $sol_mint = "So11111111111111111111111111111111111111112"
    $dexscreener_url = "https://api.dexscreener.com/latest/dex/tokens/$sol_mint"
    $dex_response = Invoke-RestMethod -Uri $dexscreener_url -TimeoutSec 10
    
    if ($dex_response.pairs -and $dex_response.pairs.Count -gt 0) {
        Write-Host "✅ DexScreener: $($dex_response.pairs.Count) pares encontrados" -ForegroundColor Green
        
        # Mostrar primeros 3 pares para análisis
        $dex_response.pairs | Select-Object -First 3 | ForEach-Object {
            $price_diff = if ($_.priceChange.h24) { [math]::Abs([double]$_.priceChange.h24) } else { 0 }
            Write-Host "    📊 $($_.dexId): Precio $($_.priceUsd), Liquidez $($_.liquidity.usd), Cambio 24h: $($price_diff)%" -ForegroundColor Gray
        }
    } else {
        Write-Host "❌ DexScreener: No pairs found" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ DexScreener API Error: $($_.Exception.Message)" -ForegroundColor Red
}

# Test Jupiter API directamente
Write-Host "`n🔍 Testing Jupiter API..." -ForegroundColor Gray
try {
    $jupiter_url = "https://price.jup.ag/v4/price?ids=$sol_mint"
    $jupiter_response = Invoke-RestMethod -Uri $jupiter_url -TimeoutSec 10
    
    if ($jupiter_response.data) {
        Write-Host "✅ Jupiter: Precio obtenido" -ForegroundColor Green
        Write-Host "    📊 SOL: $($jupiter_response.data.$sol_mint.price)" -ForegroundColor Gray
    } else {
        Write-Host "❌ Jupiter: No price data" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ Jupiter API Error: $($_.Exception.Message)" -ForegroundColor Red
}

# Paso 2: Análisis de tokens configurados
Write-Host "`n📋 PASO 2: Verificación de tokens configurados" -ForegroundColor Yellow

# Buscar configuración de tokens
$token_configs = Get-ChildItem -Path "src" -Recurse -Include "*.rs" | Select-String -Pattern "token.*mint|USDC|BONK|JUP" | Select-Object -First 10

if ($token_configs) {
    Write-Host "✅ Configuraciones de tokens encontradas:" -ForegroundColor Green
    $token_configs | ForEach-Object {
        Write-Host "    📄 $($_.Filename):$($_.LineNumber) - $($_.Line.Trim())" -ForegroundColor Gray
    }
} else {
    Write-Host "❌ No se encontraron configuraciones de tokens" -ForegroundColor Red
}

# Paso 3: Verificar thresholds y filtros
Write-Host "`n⚙️ PASO 3: Verificación de thresholds y filtros" -ForegroundColor Yellow

$threshold_configs = Get-ChildItem -Path "src" -Recurse -Include "*.rs" | Select-String -Pattern "threshold|min.*profit|confidence.*score|price.*diff" | Select-Object -First 10

if ($threshold_configs) {
    Write-Host "✅ Configuraciones de thresholds encontradas:" -ForegroundColor Green
    $threshold_configs | ForEach-Object {
        Write-Host "    📄 $($_.Filename):$($_.LineNumber) - $($_.Line.Trim())" -ForegroundColor Gray
    }
} else {
    Write-Host "❌ No se encontraron configuraciones de thresholds" -ForegroundColor Red
}

# Paso 4: Ejecutar sistema con logs detallados
Write-Host "`n🚀 PASO 4: Ejecución con logs detallados" -ForegroundColor Yellow

$env:RUST_LOG = "debug"
$env:REAL_TRADING_MODE = "false"  # Modo seguro

Write-Host "🔍 Ejecutando sistema con logs DEBUG..." -ForegroundColor Gray

# Capturar output detallado y analizarlo
$output = & "./target/debug/arbitrage_phase45_clean.exe" 2>&1 | Select-Object -First 50

# Analizar output por categorías
$price_logs = $output | Select-String -Pattern "precio|price|DexScreener|Jupiter|Coinbase"
$opportunity_logs = $output | Select-String -Pattern "oportunidad|opportunity|detected|found"
$error_logs = $output | Select-String -Pattern "error|Error|ERROR|failed|Failed"

Write-Host "`n📊 ANÁLISIS DE LOGS:" -ForegroundColor Cyan

Write-Host "  🔢 Logs de precios: $($price_logs.Count)" -ForegroundColor $(if ($price_logs.Count -gt 0) { "Green" } else { "Red" })
Write-Host "  💎 Logs de oportunidades: $($opportunity_logs.Count)" -ForegroundColor $(if ($opportunity_logs.Count -gt 0) { "Green" } else { "Red" })
Write-Host "  ❌ Logs de errores: $($error_logs.Count)" -ForegroundColor $(if ($error_logs.Count -eq 0) { "Green" } else { "Red" })

if ($error_logs.Count -gt 0) {
    Write-Host "`n🚨 ERRORES DETECTADOS:" -ForegroundColor Red
    $error_logs | Select-Object -First 5 | ForEach-Object {
        Write-Host "    $($_.Line)" -ForegroundColor Red
    }
}

# Paso 5: Recomendaciones específicas
Write-Host "`n💡 PASO 5: Diagnóstico y recomendaciones" -ForegroundColor Yellow

$recommendations = @()

if ($price_logs.Count -eq 0) {
    $recommendations += "❌ CRÍTICO: No se están obteniendo precios de APIs"
}

if ($opportunity_logs.Count -eq 0) {
    $recommendations += "❌ CRÍTICO: No se están detectando oportunidades"
}

if ($error_logs.Count -gt 3) {
    $recommendations += "⚠️ ADVERTENCIA: Múltiples errores en APIs"
}

if ($recommendations.Count -eq 0) {
    $recommendations += "✅ APIs funcionando, problema en lógica de detección"
}

Write-Host "`n📋 RECOMENDACIONES:" -ForegroundColor Cyan
$recommendations | ForEach-Object {
    Write-Host "  $_" -ForegroundColor Yellow
}

Write-Host "`n🎯 SIGUIENTE ACCIÓN RECOMENDADA:" -ForegroundColor Green
if ($price_logs.Count -eq 0) {
    Write-Host "  1. Reparar conectividad con APIs de precios" -ForegroundColor Cyan
} elseif ($opportunity_logs.Count -eq 0) {
    Write-Host "  1. Reducir thresholds de detección de oportunidades" -ForegroundColor Cyan
    Write-Host "  2. Aumentar número de tokens analizados" -ForegroundColor Cyan
    Write-Host "  3. Revisar lógica de comparación de precios" -ForegroundColor Cyan
} else {
    Write-Host "  1. Investigar lógica de filtros en detección" -ForegroundColor Cyan
}

Write-Host "`n🔍 DIAGNÓSTICO COMPLETO - Revisar resultados arriba" -ForegroundColor Green
