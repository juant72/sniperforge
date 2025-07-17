# Real Arbitrage System Launcher

Write-Host "🚀 === REAL ARBITRAGE SYSTEM ===" -ForegroundColor Green
Write-Host "   💰 EJECUTA ARBITRAJE REAL CON DINERO REAL" -ForegroundColor Yellow
Write-Host "   ⚡ INTEGRACIÓN DIRECTA CON JUPITER API" -ForegroundColor Cyan
Write-Host "   🎯 SWAPS DE TOKENS REALES, NO SIMULACIONES" -ForegroundColor Magenta
Write-Host ""

# Verificar balance antes de empezar
Write-Host "🔍 Verificando balance de wallet..." -ForegroundColor Cyan
try {
    $balance_output = cargo run --release --bin check_balance_simple 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Balance verificado" -ForegroundColor Green
        Write-Host $balance_output
    } else {
        Write-Host "❌ Error verificando balance: $balance_output" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "❌ Error ejecutando check de balance: $_" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "⚠️  ADVERTENCIA IMPORTANTE:" -ForegroundColor Red
Write-Host "   Este sistema ejecuta transacciones REALES en mainnet" -ForegroundColor Yellow
Write-Host "   Usa dinero REAL de tu wallet" -ForegroundColor Yellow
Write-Host "   Las pérdidas son REALES e irreversibles" -ForegroundColor Yellow
Write-Host ""

# Confirmación de usuario
$confirmation = Read-Host "¿Estás seguro de que quieres ejecutar arbitraje REAL? (escribir 'SI ESTOY SEGURO' para confirmar)"

if ($confirmation -ne "SI ESTOY SEGURO") {
    Write-Host "❌ Ejecución cancelada por el usuario" -ForegroundColor Red
    exit 0
}

Write-Host ""
Write-Host "🚀 Iniciando sistema de arbitraje REAL..." -ForegroundColor Green

# Ejecutar el sistema de arbitraje real
try {
    cargo run --release --bin real_arbitrage_system
} catch {
    Write-Host "❌ Error ejecutando arbitraje real: $_" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Sistema de arbitraje completado" -ForegroundColor Green
