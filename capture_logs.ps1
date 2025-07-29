# Script de analisis de logs para arbitrage_phase45_clean
Write-Host "=== INICIANDO ANALISIS DE LOGS ===" -ForegroundColor Cyan

# Ejecutar el sistema por un tiempo corto para capturar logs
Write-Host "Ejecutando sistema para capturar logs..." -ForegroundColor Yellow
$job = Start-Job -ScriptBlock {
    Set-Location "c:\work\encrypia\labs\sniperforge"
    $env:RUST_LOG = "info"
    cargo run --bin arbitrage_phase45_clean 2>&1
}

# Esperar 10 segundos
Start-Sleep -Seconds 10

# Detener el job
Stop-Job $job
$output = Receive-Job $job
Remove-Job $job

# Guardar output en archivo
$output | Out-File -FilePath "logs_analysis_current.txt" -Encoding UTF8

Write-Host "Logs capturados en logs_analysis_current.txt" -ForegroundColor Green
Write-Host "Primeras 20 lineas de logs:" -ForegroundColor Yellow
$output | Select-Object -First 20 | ForEach-Object { Write-Host $_ }

Write-Host "`n=== ANALISIS COMPLETO ===" -ForegroundColor Cyan
