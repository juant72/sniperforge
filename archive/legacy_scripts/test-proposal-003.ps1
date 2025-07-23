#!/usr/bin/env pwsh
# PROPOSAL-003 Testing Script
# Test automatizado del sistema multi-token

Write-Host "🚀 PROPOSAL-003 AUTOMATIC TESTING SCRIPT" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green

# Navegar al directorio correcto
Set-Location "c:\work\encrypia\labs\sniperforge"

# Verificar que el proyecto compile
Write-Host "📦 Building project..." -ForegroundColor Yellow
$buildResult = cargo build --bin arbiter_clean 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    Write-Host $buildResult
    exit 1
}
Write-Host "✅ Build successful!" -ForegroundColor Green

# Crear archivo de input automatizado para testing
Write-Host "🎯 Preparing automated test input..." -ForegroundColor Yellow
$testInput = @"
M

"@

# Guardar input en un archivo temporal
$testInput | Out-File -FilePath "test_input.txt" -Encoding utf8

Write-Host "🚀 Launching PROPOSAL-003 Multi-Token Test..." -ForegroundColor Cyan
Write-Host "   - Mode: Multi-token simulation (M)" -ForegroundColor White
Write-Host "   - Duration: Will run for 60 seconds then terminate" -ForegroundColor White
Write-Host "   - Safety: Simulation only, no real money" -ForegroundColor White

# Ejecutar el programa con input automatizado y timeout
$job = Start-Job -ScriptBlock {
    param($path)
    Set-Location $path
    Get-Content "test_input.txt" | ./target/debug/arbiter_clean.exe
} -ArgumentList (Get-Location)

# Esperar 60 segundos o hasta que termine
Write-Host "⏱️  Running test (60 second timeout)..." -ForegroundColor Yellow
$timeout = 60
$elapsed = 0

while ($job.State -eq "Running" -and $elapsed -lt $timeout) {
    Start-Sleep -Seconds 5
    $elapsed += 5
    Write-Host "   ⏳ Elapsed: $elapsed seconds" -ForegroundColor Gray
}

# Obtener resultados
if ($job.State -eq "Running") {
    Write-Host "⏰ Test completed (timeout reached)" -ForegroundColor Yellow
    Stop-Job $job
} else {
    Write-Host "🏁 Test completed naturally" -ForegroundColor Green
}

# Mostrar output
Write-Host "`n📋 TEST OUTPUT:" -ForegroundColor Cyan
$output = Receive-Job $job
$output | ForEach-Object { Write-Host "   $_" -ForegroundColor White }

# Cleanup
Remove-Job $job -Force
Remove-Item "test_input.txt" -ErrorAction SilentlyContinue

Write-Host "`n🎉 PROPOSAL-003 TEST COMPLETED!" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
Write-Host "Review the output above to validate multi-token functionality" -ForegroundColor White
