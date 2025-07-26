#!/usr/bin/env pwsh
# =========================================================================
# SCRIPT DE EJECUCIÓN PARA TRADING REAL - SISTEMA CORPORATIVO
# =========================================================================
# Este script activa el modo trading real con las correcciones aplicadas
# =========================================================================

Write-Host "🚀 ═══════════════════════════════════════════════════════════════"
Write-Host "    ACTIVANDO SISTEMA DE TRADING REAL CORPORATIVO"
Write-Host "    Sistema con ML Enhancement y filtros corregidos"
Write-Host "🚀 ═══════════════════════════════════════════════════════════════"

# CONFIGURACIÓN DE TRADING REAL - VARIABLES DE ENTORNO
$env:FORCE_REAL_TRANSACTIONS = "true"
$env:MAX_TRADE_SOL = "0.005"  # 5 mSOL máximo por trade
$env:RUST_LOG = "info"

Write-Host "✅ Variables de entorno configuradas:"
Write-Host "   FORCE_REAL_TRANSACTIONS = $($env:FORCE_REAL_TRANSACTIONS)"
Write-Host "   MAX_TRADE_SOL = $($env:MAX_TRADE_SOL)"

# Verificar compilación
Write-Host "🔧 Verificando compilación del sistema..."
if (Test-Path ".\target\release\arbitrage_phase45_clean.exe") {
    Write-Host "✅ Ejecutable encontrado: arbitrage_phase45_clean.exe"
} else {
    Write-Host "🔨 Compilando sistema..."
    cargo build --release --bin arbitrage_phase45_clean
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Compilación exitosa"
    } else {
        Write-Host "❌ Error en compilación. Abortando."
        exit 1
    }
}

Write-Host ""
Write-Host "🎯 CONFIGURACIÓN FINAL:"
Write-Host "   • Modo: TRADING REAL ACTIVADO"
Write-Host "   • Filtros: Optimizados para detectar más oportunidades"
Write-Host "   • Profit mínimo: 0.05% (reducido de 0.1%)"
Write-Host "   • Confidence mínimo: 0.3 (reducido de 0.6)"
Write-Host "   • ML Thresholds: 0.2+ para ejecución (reducido de 0.4+)"
Write-Host ""

$confirmation = Read-Host "¿Continuar con TRADING REAL? (escriba 'SI' para confirmar)"

if ($confirmation -eq "SI") {
    Write-Host "🔥 INICIANDO TRADING REAL..."
    Write-Host "💰 Sistema configurado para trades reales con SOL de wallet"
    Write-Host "🧠 ML Enhancement activo para protección"
    Write-Host "📊 Dashboard mostrará: REAL TRADING MODE ACTIVE"
    Write-Host ""
    
    # EJECUTAR EL SISTEMA REAL
    Write-Host "🚀 EJECUTANDO SISTEMA CORREGIDO..."
    & ".\target\release\arbitrage_phase45_clean.exe"
    
} else {
    Write-Host "❌ Trading real cancelado por el usuario"
    Write-Host "💡 Para activar, ejecute nuevamente y confirme con 'SI'"
}

Write-Host ""
Write-Host "🚀 ═══════════════════════════════════════════════════════════════"
Write-Host "    SCRIPT DE TRADING REAL COMPLETADO"
Write-Host "🚀 ═══════════════════════════════════════════════════════════════"
