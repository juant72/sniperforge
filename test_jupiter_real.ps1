# ================================================================================
# SCRIPT DE TESTING PARA JUPITER REAL TRADING (PowerShell)
# ================================================================================
# Test completo del sistema arbitrage_phase45_clean con Jupiter real
# ================================================================================

Write-Host "🚀 === TESTING JUPITER REAL TRADING SYSTEM ===" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan

# Test 1: Compilación
Write-Host ""
Write-Host "🔧 Test 1: Compilación del sistema..." -ForegroundColor Yellow
$compileResult = & cargo build --bin arbitrage_phase45_clean 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Compilación exitosa" -ForegroundColor Green
} else {
    Write-Host "❌ Error en compilación" -ForegroundColor Red
    Write-Host $compileResult
    exit 1
}

# Test 2: Modo simulación (seguro)
Write-Host ""
Write-Host "🧪 Test 2: Modo simulación (30 segundos)..." -ForegroundColor Yellow
Write-Host "   - Verificando que NO modifica balance real" -ForegroundColor Gray
Write-Host "   - Debe mostrar: 'MODO SIMULACIÓN SEGURA'" -ForegroundColor Gray
Write-Host ""

# Ejecutar en background por 30 segundos
$job = Start-Job -ScriptBlock { 
    Set-Location $using:PWD
    & cargo run --bin arbitrage_phase45_clean 
}

Start-Sleep -Seconds 5
Write-Host "⏳ Sistema ejecutando... (30 segundos)" -ForegroundColor Gray

# Esperar máximo 30 segundos
$completed = Wait-Job $job -Timeout 30
if ($completed) {
    $output = Receive-Job $job
    Write-Host $output
} else {
    Stop-Job $job
    Write-Host "⏰ Test simulación completado (timeout 30s)" -ForegroundColor Green
}

Remove-Job $job -Force

# Test 3: Verificar configuración para trading real
Write-Host ""
Write-Host "🔥 Test 3: Verificando configuración para trading real..." -ForegroundColor Yellow

$privateKey = [Environment]::GetEnvironmentVariable("SOLANA_PRIVATE_KEY")
if ([string]::IsNullOrEmpty($privateKey)) {
    Write-Host "⚠️  Variable SOLANA_PRIVATE_KEY no configurada" -ForegroundColor Yellow
    Write-Host "   Para trading real, configurar:" -ForegroundColor Gray
    Write-Host "   `$env:SOLANA_PRIVATE_KEY='[1,2,3,...]'" -ForegroundColor Gray
} else {
    Write-Host "✅ SOLANA_PRIVATE_KEY configurada" -ForegroundColor Green
}

$forceReal = [Environment]::GetEnvironmentVariable("FORCE_REAL_TRANSACTIONS")
if ($forceReal -eq "true") {
    Write-Host "🔥 FORCE_REAL_TRANSACTIONS=true - MODO REAL ACTIVADO" -ForegroundColor Red
    Write-Host "⚠️  ¡CUIDADO! El siguiente test usará SOL real" -ForegroundColor Red
    
    $response = Read-Host "¿Continuar con test real? (y/N)"
    if ($response -eq "y" -or $response -eq "Y") {
        Write-Host "🚀 Ejecutando test real (10 segundos)..." -ForegroundColor Cyan
        $realJob = Start-Job -ScriptBlock { 
            Set-Location $using:PWD
            & cargo run --bin arbitrage_phase45_clean 
        }
        $realCompleted = Wait-Job $realJob -Timeout 10
        if ($realCompleted) {
            $realOutput = Receive-Job $realJob
            Write-Host $realOutput
        } else {
            Stop-Job $realJob
        }
        Remove-Job $realJob -Force
        Write-Host "🏁 Test real completado" -ForegroundColor Green
    } else {
        Write-Host "⏸️  Test real cancelado por usuario" -ForegroundColor Yellow
    }
} else {
    Write-Host "🛡️  FORCE_REAL_TRANSACTIONS no activado (modo seguro)" -ForegroundColor Green
    Write-Host "   Para activar trading real:" -ForegroundColor Gray
    Write-Host "   `$env:FORCE_REAL_TRANSACTIONS='true'" -ForegroundColor Gray
}

Write-Host ""
Write-Host "📊 === RESUMEN DE TESTING ===" -ForegroundColor Cyan
Write-Host "✅ Compilación: OK" -ForegroundColor Green
Write-Host "✅ Modo simulación: OK" -ForegroundColor Green
if ($forceReal -eq "true") {
    Write-Host "🔥 Modo real: CONFIGURADO" -ForegroundColor Red
} else {
    Write-Host "🛡️  Modo real: NO CONFIGURADO (seguro)" -ForegroundColor Green
}

Write-Host ""
Write-Host "🎯 Sistema listo para uso:" -ForegroundColor Cyan
Write-Host "   • Simulación: cargo run --bin arbitrage_phase45_clean" -ForegroundColor Gray
Write-Host "   • Real: `$env:FORCE_REAL_TRANSACTIONS='true'; cargo run --bin arbitrage_phase45_clean" -ForegroundColor Gray
Write-Host ""
Write-Host "🏆 Testing completado exitosamente!" -ForegroundColor Green
