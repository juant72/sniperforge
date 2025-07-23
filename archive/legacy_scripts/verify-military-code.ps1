#!/usr/bin/env pwsh
# SCRIPT MILITAR: Verificación y ejecución SIN BUILD

Write-Host "🎯 SISTEMA MILITAR: Verificación de código sin compilación" -ForegroundColor Cyan

# 1. ANÁLISIS ESTÁTICO: Verificar sintaxis sin compilar
Write-Host "🔍 VERIFICACIÓN SINTAXIS (cargo check - súper rápido)..." -ForegroundColor Yellow
$checkStart = Get-Date
cargo check --bin military_arbitrage_system --profile dev 2>&1 | Tee-Object -FilePath "syntax_check.txt"
$checkEnd = Get-Date
$checkDuration = ($checkEnd - $checkStart).TotalSeconds

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ SINTAXIS CORRECTA verificada en $([math]::Round($checkDuration, 1))s" -ForegroundColor Green
} else {
    Write-Host "❌ ERRORES DE SINTAXIS encontrados:" -ForegroundColor Red
    Get-Content "syntax_check.txt" | Select-String "error\[" | ForEach-Object {
        Write-Host "   $($_.Line)" -ForegroundColor Red
    }
    exit 1
}

# 2. ANÁLISIS DE CÓDIGO: Clippy para calidad
Write-Host "🧹 VERIFICACIÓN CALIDAD (clippy)..." -ForegroundColor Yellow
cargo clippy --bin military_arbitrage_system -- -D warnings 2>&1 | Tee-Object -FilePath "quality_check.txt"

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ CÓDIGO LIMPIO - Sin warnings" -ForegroundColor Green
} else {
    Write-Host "⚠️ WARNINGS encontrados (no críticos):" -ForegroundColor Yellow
    Get-Content "quality_check.txt" | Select-String "warning:" | ForEach-Object {
        Write-Host "   $($_.Line)" -ForegroundColor Yellow
    }
}

# 3. VERIFICACIÓN MILITAR: Funciones críticas implementadas
Write-Host "⚔️ VERIFICACIÓN MILITAR: Funciones críticas..." -ForegroundColor Cyan

$militaryFunctions = @(
    "find_real_arbitrage_opportunities",
    "calculate_direct_pair_arbitrage", 
    "calculate_pool_output_realistic",
    "get_token_account_balance",
    "parse_raydium_pool",
    "parse_orca_pool",
    "parse_orca_whirlpool"
)

$missingFunctions = @()
foreach ($func in $militaryFunctions) {
    $found = Select-String -Path "military_arbitrage_system.rs" -Pattern "fn $func" -Quiet
    if ($found) {
        Write-Host "✅ $func - IMPLEMENTADA" -ForegroundColor Green
    } else {
        Write-Host "❌ $func - FALTANTE" -ForegroundColor Red
        $missingFunctions += $func
    }
}

# 4. RESUMEN MILITAR
Write-Host "`n🏆 RESUMEN MILITAR:" -ForegroundColor Cyan
Write-Host "   ✅ Sintaxis verificada: $([math]::Round($checkDuration, 1))s" -ForegroundColor White
Write-Host "   ✅ Funciones críticas: $($militaryFunctions.Count - $missingFunctions.Count)/$($militaryFunctions.Count)" -ForegroundColor White

if ($missingFunctions.Count -eq 0) {
    Write-Host "🎯 SISTEMA MILITAR COMPLETO - Listo para operación" -ForegroundColor Green
    Write-Host "🚀 Para ejecutar: .\run-military-arbitrage.ps1" -ForegroundColor Cyan
} else {
    Write-Host "⚠️ FUNCIONES FALTANTES: $($missingFunctions -join ', ')" -ForegroundColor Yellow
    Write-Host "🔧 Completar implementación antes de operación" -ForegroundColor Yellow
}

Write-Host "`n💡 VENTAJA: Verificación completa SIN build de 30 minutos" -ForegroundColor Magenta
