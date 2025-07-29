# Script para probar el ejecutable principal con timeout y captura de logs
Write-Host "=== PRUEBA DEL EJECUTABLE PRINCIPAL ===" -ForegroundColor Cyan

# Verificar que el ejecutable existe
if (Test-Path "target\release\arbitrage_phase45_clean.exe") {
    Write-Host "✓ Ejecutable encontrado en target\release\" -ForegroundColor Green
    $executable = "target\release\arbitrage_phase45_clean.exe"
} elseif (Test-Path "target\debug\arbitrage_phase45_clean.exe") {
    Write-Host "✓ Ejecutable encontrado en target\debug\" -ForegroundColor Green
    $executable = "target\debug\arbitrage_phase45_clean.exe"
} else {
    Write-Host "Compilando ejecutable..." -ForegroundColor Yellow
    cargo build --bin arbitrage_phase45_clean
    $executable = "target\debug\arbitrage_phase45_clean.exe"
}

# Verificar configuracion
Write-Host "Verificando configuracion..." -ForegroundColor Yellow
if (Test-Path "arbitrage_settings.json") {
    Write-Host "✓ arbitrage_settings.json encontrado" -ForegroundColor Green
    $config = Get-Content -Raw "arbitrage_settings.json" | ConvertFrom-Json
    Write-Host "  - Modo: $($config.trading.mode)" -ForegroundColor Cyan
    Write-Host "  - Max trade: $($config.trading.max_trade_sol) SOL" -ForegroundColor Cyan
    Write-Host "  - Performance target: $($config.performance.latency_target_ms)ms" -ForegroundColor Cyan
} else {
    Write-Host "❌ arbitrage_settings.json no encontrado" -ForegroundColor Red
    exit 1
}

# Ejecutar con timeout
Write-Host "`nEjecutando sistema por 15 segundos..." -ForegroundColor Yellow
Write-Host "Presiona Ctrl+C para detener antes..." -ForegroundColor Gray

$startTime = Get-Date
$job = Start-Job -ScriptBlock {
    param($exe)
    Set-Location "c:\work\encrypia\labs\sniperforge"
    $env:RUST_LOG = "info"
    & $exe 2>&1
} -ArgumentList $executable

# Esperar 15 segundos
$timeout = 15
$elapsed = 0
while ($elapsed -lt $timeout -and $job.State -eq "Running") {
    Start-Sleep -Seconds 1
    $elapsed++
    $percent = [math]::Round(($elapsed / $timeout) * 100, 0)
    Write-Progress -Activity "Ejecutando Arbitrage System" -Status "$elapsed/$timeout segundos" -PercentComplete $percent
}

# Obtener output
if ($job.State -eq "Running") {
    Stop-Job $job
    Write-Host "`n⏰ Timeout alcanzado - deteniendo proceso" -ForegroundColor Yellow
}

$output = Receive-Job $job
Remove-Job $job

# Mostrar resultados
Write-Host "`n=== RESULTADOS DE EJECUCIÓN ===" -ForegroundColor Cyan
if ($output) {
    Write-Host "📋 Output capturado ($($output.Count) lineas):" -ForegroundColor Green
    $output | Select-Object -First 20 | ForEach-Object { 
        if ($_ -match "ERROR|error") {
            Write-Host "  🚨 $_" -ForegroundColor Red
        } elseif ($_ -match "WARN|warn") {
            Write-Host "  ⚠️  $_" -ForegroundColor Yellow
        } elseif ($_ -match "INFO|info") {
            Write-Host "  ℹ️  $_" -ForegroundColor Cyan
        } else {
            Write-Host "  $_" -ForegroundColor White
        }
    }
    
    if ($output.Count -gt 20) {
        Write-Host "  ... ($($output.Count - 20) lineas adicionales)" -ForegroundColor Gray
    }
    
    # Guardar log completo
    $output | Out-File -FilePath "test_execution_log.txt" -Encoding UTF8
    Write-Host "`n📁 Log completo guardado en: test_execution_log.txt" -ForegroundColor Green
} else {
    Write-Host "❌ No se capturó output del sistema" -ForegroundColor Red
}

Write-Host "`n=== ANÁLISIS RÁPIDO ===" -ForegroundColor Cyan
$errors = $output | Where-Object { $_ -match "ERROR|error" }
$warnings = $output | Where-Object { $_ -match "WARN|warn" }
$infos = $output | Where-Object { $_ -match "INFO|info" }

Write-Host "Errores: $($errors.Count)" -ForegroundColor $(if ($errors.Count -eq 0) { "Green" } else { "Red" })
Write-Host "Warnings: $($warnings.Count)" -ForegroundColor $(if ($warnings.Count -eq 0) { "Green" } else { "Yellow" })
Write-Host "Info messages: $($infos.Count)" -ForegroundColor Cyan

if ($errors.Count -eq 0) {
    Write-Host "`n✅ Sistema ejecutado sin errores críticos!" -ForegroundColor Green
} else {
    Write-Host "`n⚠️  Errores detectados - revisar log para detalles" -ForegroundColor Yellow
}
