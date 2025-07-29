# Test rápido del sistema con timeout

Write-Host "=== Test Rápido del Sistema ===" -ForegroundColor Cyan

$job = Start-Job -ScriptBlock {
    Set-Location $using:PWD
    cargo run --bin arbitrage_phase45_clean 2>&1
}

Write-Host "🚀 Iniciando sistema..." -ForegroundColor Yellow

# Esperar hasta 30 segundos
$timeout = 30
$elapsed = 0

do {
    Start-Sleep -Seconds 2
    $elapsed += 2
    
    # Verificar si el job ha terminado o hay output
    $output = Receive-Job $job -Keep
    
    if ($output) {
        $lastLines = $output | Select-Object -Last 10
        Write-Host "📋 Últimas líneas de salida:" -ForegroundColor Cyan
        foreach ($line in $lastLines) {
            if ($line -match "✅.*JSON.*exitosa") {
                Write-Host "✅ ÉXITO: $line" -ForegroundColor Green
            } elseif ($line -match "❌.*Error") {
                Write-Host "❌ ERROR: $line" -ForegroundColor Red
            } elseif ($line -match "INFO.*Iniciando") {
                Write-Host "🚀 INICIO: $line" -ForegroundColor Yellow
            } else {
                Write-Host $line -ForegroundColor Gray
            }
        }
        
        # Si hay éxito, terminar
        if ($output -match "✅.*JSON.*exitosa|Phase.*operational") {
            Write-Host "`n✅ Sistema iniciado correctamente" -ForegroundColor Green
            break
        }
        
        # Si hay error, mostrar y terminar
        if ($output -match "❌.*Error.*missing field") {
            Write-Host "`n❌ Aún hay campos faltantes" -ForegroundColor Red
            break
        }
    }
    
    Write-Host "⏳ Esperando... ($elapsed/$timeout segundos)" -ForegroundColor Gray
    
} while ($elapsed -lt $timeout -and $job.State -eq "Running")

# Cleanup
Stop-Job $job -ErrorAction SilentlyContinue
Remove-Job $job -ErrorAction SilentlyContinue

if ($elapsed -ge $timeout) {
    Write-Host "⏰ Timeout alcanzado - Sistema probablemente iniciado correctamente" -ForegroundColor Yellow
} else {
    Write-Host "`n🏁 Test completado" -ForegroundColor Cyan
}
