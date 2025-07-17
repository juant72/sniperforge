# 🛡️ SISTEMA SEGURO VS SISTEMA CON RIESGOS

Write-Host "🔍 === COMPARACIÓN DE SISTEMAS DE ARBITRAJE ===" -ForegroundColor Cyan
Write-Host ""

Write-Host "📊 ANÁLISIS COMPLETADO:" -ForegroundColor Yellow
Write-Host ""

Write-Host "❌ SISTEMA CON RIESGOS DETECTADOS:" -ForegroundColor Red
Write-Host "   📁 Archivo: real_arbitrage_system.rs" -ForegroundColor White
Write-Host "   🚩 Problemas críticos encontrados:" -ForegroundColor Red
Write-Host "      • Línea 343: Estimación ciega del 99.7%" -ForegroundColor Red
Write-Host "      • No verifica token accounts reales" -ForegroundColor Red
Write-Host "      • Slippage hardcodeado al 0.5%" -ForegroundColor Red
Write-Host "      • Fees subestimados (no incluye Jupiter)" -ForegroundColor Red
Write-Host "      • Falta verificación de balances intermedios" -ForegroundColor Red
Write-Host ""

Write-Host "✅ SISTEMA SEGURO CREADO:" -ForegroundColor Green
Write-Host "   📁 Archivo: secure_arbitrage_system.rs" -ForegroundColor White
Write-Host "   🛡️ Correcciones implementadas:" -ForegroundColor Green
Write-Host "      • Verificación de token accounts" -ForegroundColor Green
Write-Host "      • Slippage dinámico según tamaño de trade" -ForegroundColor Green
Write-Host "      • Cálculo preciso de fees (incluye Jupiter)" -ForegroundColor Green
Write-Host "      • Verificación de balances reales" -ForegroundColor Green
Write-Host "      • Múltiples checks de seguridad" -ForegroundColor Green
Write-Host "      • Abort automático en caso de pérdida" -ForegroundColor Green
Write-Host ""

Write-Host "⚠️ RIESGOS DEL SISTEMA ORIGINAL:" -ForegroundColor Yellow
Write-Host "   💰 Pérdida por cantidad incorrecta en segundo swap" -ForegroundColor White
Write-Host "   💰 Fees no calculados correctamente" -ForegroundColor White
Write-Host "   💰 Slippage inadecuado en mercados volátiles" -ForegroundColor White
Write-Host "   💰 No maneja tokens accounts inexistentes" -ForegroundColor White
Write-Host ""

Write-Host "🎯 RECOMENDACIÓN:" -ForegroundColor Cyan
Write-Host ""

$choice = Read-Host "¿Qué sistema desea usar? (1=Seguro, 2=Con Riesgos, 3=Ver Reporte)"

switch ($choice) {
    "1" {
        Write-Host ""
        Write-Host "🛡️ Ejecutando sistema SEGURO..." -ForegroundColor Green
        Write-Host "✅ Con todas las validaciones y verificaciones" -ForegroundColor Green
        Write-Host ""
        
        try {
            cargo run --release --bin secure_arbitrage_system
        } catch {
            Write-Host "❌ Error ejecutando sistema seguro: $_" -ForegroundColor Red
        }
    }
    "2" {
        Write-Host ""
        Write-Host "⚠️ ADVERTENCIA: Sistema con riesgos detectados" -ForegroundColor Red
        Write-Host "❌ Puede causar pérdidas de dinero" -ForegroundColor Red
        Write-Host ""
        
        $confirm = Read-Host "¿Está seguro de usar el sistema CON RIESGOS? (escribir 'ACEPTO RIESGOS')"
        
        if ($confirm -eq "ACEPTO RIESGOS") {
            Write-Host "⚠️ Ejecutando sistema CON RIESGOS..." -ForegroundColor Yellow
            try {
                cargo run --release --bin real_arbitrage_system
            } catch {
                Write-Host "❌ Error ejecutando sistema con riesgos: $_" -ForegroundColor Red
            }
        } else {
            Write-Host "✅ Ejecución cancelada - Decisión inteligente" -ForegroundColor Green
        }
    }
    "3" {
        Write-Host ""
        Write-Host "📋 Abriendo reporte detallado..." -ForegroundColor Cyan
        try {
            Get-Content "AUDITORIA_REAL_ARBITRAGE_REPORT.md" | Write-Host
        } catch {
            Write-Host "❌ Error leyendo reporte: $_" -ForegroundColor Red
        }
    }
    default {
        Write-Host ""
        Write-Host "❌ Opción inválida" -ForegroundColor Red
        Write-Host "📋 Comandos disponibles:" -ForegroundColor White
        Write-Host "   Seguro: cargo run --release --bin secure_arbitrage_system" -ForegroundColor Green
        Write-Host "   Riesgoso: cargo run --release --bin real_arbitrage_system" -ForegroundColor Red
        Write-Host "   Reporte: Get-Content AUDITORIA_REAL_ARBITRAGE_REPORT.md" -ForegroundColor Cyan
    }
}

Write-Host ""
Write-Host "📊 RESUMEN:" -ForegroundColor Cyan
Write-Host "✅ Sistema seguro disponible y recomendado" -ForegroundColor Green
Write-Host "⚠️ Sistema original tiene riesgos documentados" -ForegroundColor Yellow
Write-Host "🛑 Sistema fraudulento completamente deshabilitado" -ForegroundColor Red
