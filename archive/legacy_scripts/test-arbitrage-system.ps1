# Script para probar el sistema de arbitraje real
# Verifica cálculos, prevención de pérdidas y funcionamiento correcto

Write-Host "🧪 === PRUEBAS DEL SISTEMA DE ARBITRAJE REAL ===" -ForegroundColor Cyan
Write-Host "   🔍 Verificando cálculos matemáticos" -ForegroundColor Yellow
Write-Host "   🛡️ Validando prevención de pérdidas" -ForegroundColor Yellow
Write-Host "   ⚡ Probando integración con Jupiter API" -ForegroundColor Yellow
Write-Host ""

# Paso 1: Verificar que el sistema compila
Write-Host "📋 Paso 1: Verificando compilación..." -ForegroundColor Green
cargo check --bin real_arbitrage_system
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ ERROR: El sistema principal no compila correctamente" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Sistema principal compila correctamente" -ForegroundColor Green
Write-Host ""

# Paso 2: Compilar el sistema de pruebas
Write-Host "📋 Paso 2: Compilando sistema de pruebas..." -ForegroundColor Green
cargo build --bin test_arbitrage_calculations
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ ERROR: El sistema de pruebas no compila" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Sistema de pruebas compilado correctamente" -ForegroundColor Green
Write-Host ""

# Paso 3: Ejecutar pruebas de cálculos (modo seguro, sin transacciones reales)
Write-Host "📋 Paso 3: Ejecutando pruebas de cálculos..." -ForegroundColor Green
Write-Host "⚠️  NOTA: Estas pruebas NO ejecutan transacciones reales" -ForegroundColor Yellow
Write-Host "   Solo verifican cálculos y lógica de prevención de pérdidas" -ForegroundColor Yellow
Write-Host ""

cargo run --bin test_arbitrage_calculations
$test_result = $LASTEXITCODE

Write-Host ""
if ($test_result -eq 0) {
    Write-Host "✅ === TODAS LAS PRUEBAS PASARON EXITOSAMENTE ===" -ForegroundColor Green
    Write-Host ""
    Write-Host "🎯 RESULTADOS DE LAS PRUEBAS:" -ForegroundColor Cyan
    Write-Host "   ✅ Cálculos matemáticos: CORRECTOS" -ForegroundColor Green
    Write-Host "   ✅ Prevención de pérdidas: FUNCIONAL" -ForegroundColor Green
    Write-Host "   ✅ Cálculos de slippage: SEGUROS" -ForegroundColor Green
    Write-Host "   ✅ Integración Jupiter API: OPERATIVA" -ForegroundColor Green
    Write-Host ""
    Write-Host "🚀 CONCLUSIÓN: El sistema está LISTO para uso con dinero real" -ForegroundColor Green
    Write-Host ""
    Write-Host "⚠️  RECORDATORIOS IMPORTANTES:" -ForegroundColor Yellow
    Write-Host "   • Siempre probar primero con cantidades pequeñas" -ForegroundColor Yellow
    Write-Host "   • Verificar que tienes suficiente SOL para fees" -ForegroundColor Yellow
    Write-Host "   • Configurar SOLANA_RPC_URL para un RPC premium" -ForegroundColor Yellow
    Write-Host "   • Tener archivo mainnet_wallet.json en el directorio" -ForegroundColor Yellow
} else {
    Write-Host "❌ === ALGUNAS PRUEBAS FALLARON ===" -ForegroundColor Red
    Write-Host ""
    Write-Host "🔍 REVISAR LOS LOGS ANTERIORES PARA VER DETALLES" -ForegroundColor Yellow
    Write-Host "   Las pruebas que fallaron pueden indicar:" -ForegroundColor Yellow
    Write-Host "   • Problemas de conectividad con Jupiter API" -ForegroundColor Yellow
    Write-Host "   • Rate limits de la API" -ForegroundColor Yellow
    Write-Host "   • Errores en cálculos matemáticos" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "⚠️  NO EJECUTAR CON DINERO REAL HASTA RESOLVER LOS ERRORES" -ForegroundColor Red
}

Write-Host ""
Write-Host "📚 SIGUIENTE PASO:" -ForegroundColor Cyan
if ($test_result -eq 0) {
    Write-Host "   Si quieres ejecutar el sistema real:" -ForegroundColor White
    Write-Host "   cargo run --bin real_arbitrage_system" -ForegroundColor White
    Write-Host ""
    Write-Host "   ⚠️  IMPORTANTE: Esto ejecutará transacciones REALES en mainnet" -ForegroundColor Red
} else {
    Write-Host "   Revisar y corregir los errores encontrados" -ForegroundColor White
    Write-Host "   Luego volver a ejecutar este script de pruebas" -ForegroundColor White
}

Write-Host ""
