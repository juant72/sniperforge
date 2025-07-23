# Script maestro para optimizar completamente la velocidad de compilación
# Ejecuta todas las optimizaciones necesarias

Write-Host "🚀 SniperForge - Optimizador de Compilación" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan

# 1. Configurar OpenSSL precompilado
Write-Host "`n🔧 Paso 1: Configurando OpenSSL precompilado..." -ForegroundColor Yellow
& ".\setup-openssl.ps1"

# 2. Configurar sccache
Write-Host "`n⚡ Paso 2: Configurando cache de compilación..." -ForegroundColor Yellow
& ".\setup-sccache.ps1"

# 3. Limpiar cache anterior
Write-Host "`n🧹 Paso 3: Limpiando cache anterior..." -ForegroundColor Yellow
cargo clean

# 4. Verificar configuración
Write-Host "`n✅ Paso 4: Verificando configuración..." -ForegroundColor Yellow

Write-Host "Variables de entorno:" -ForegroundColor Blue
Write-Host "  OPENSSL_DIR: $env:OPENSSL_DIR" -ForegroundColor Gray
Write-Host "  OPENSSL_STATIC: $env:OPENSSL_STATIC" -ForegroundColor Gray
Write-Host "  RUSTC_WRAPPER: $env:RUSTC_WRAPPER" -ForegroundColor Gray

# 5. Test de compilación rápida
Write-Host "`n🧪 Paso 5: Probando compilación optimizada..." -ForegroundColor Yellow
Write-Host "Ejecutando: cargo check (esto debería ser mucho más rápido)" -ForegroundColor Blue

$startTime = Get-Date
cargo check
$endTime = Get-Date
$duration = $endTime - $startTime

Write-Host "`n🎉 ¡Optimización completa!" -ForegroundColor Green
Write-Host "⏱️  Tiempo de compilación: $($duration.TotalSeconds) segundos" -ForegroundColor Blue
Write-Host "💡 Las próximas compilaciones serán aún más rápidas gracias al cache" -ForegroundColor Yellow

if ($duration.TotalSeconds -lt 30) {
    Write-Host "🚀 ¡Excelente! Compilación súper rápida" -ForegroundColor Green
} elseif ($duration.TotalSeconds -lt 60) {
    Write-Host "⚡ ¡Buena velocidad de compilación!" -ForegroundColor Yellow
} else {
    Write-Host "⚠️  Aún lento. Considera usar OpenSSL vendored." -ForegroundColor Red
    Write-Host "   Ejecuta: cargo add openssl --features vendored" -ForegroundColor Red
}
