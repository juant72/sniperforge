# VS Code Emergency Recovery Script
# Para usar si VS Code sigue teniendo problemas

Write-Host "=== VS Code Emergency Recovery ===" -ForegroundColor Green

# 1. Matar todos los procesos relacionados con rust-analyzer
Write-Host "1. Cerrando rust-analyzer..." -ForegroundColor Yellow
Get-Process | Where-Object {$_.ProcessName -like "*rust-analyzer*"} | Stop-Process -Force -ErrorAction SilentlyContinue

# 2. Limpiar cache de Cargo
Write-Host "2. Limpiando cache de Cargo..." -ForegroundColor Yellow
cargo clean 2>$null

# 3. Limpiar target de rust-analyzer
Write-Host "3. Limpiando target de rust-analyzer..." -ForegroundColor Yellow
if (Test-Path "target/ra-check") {
    Remove-Item "target/ra-check" -Recurse -Force -ErrorAction SilentlyContinue
}

# 4. Verificar compilación
Write-Host "4. Verificando compilación..." -ForegroundColor Yellow
$compileResult = cargo check --message-format=short 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Código compila correctamente" -ForegroundColor Green
} else {
    Write-Host "❌ ERROR DE COMPILACIÓN DETECTADO:" -ForegroundColor Red
    Write-Host $compileResult -ForegroundColor Red
    Write-Host "SOLUCIÓN: Corregir errores de compilación antes de abrir VS Code" -ForegroundColor Yellow
    exit 1
}

# 5. Reiniciar VS Code si está abierto
Write-Host "5. Verificando VS Code..." -ForegroundColor Yellow
$vscodeProcesses = Get-Process | Where-Object {$_.ProcessName -eq "Code"}
if ($vscodeProcesses) {
    Write-Host "VS Code está corriendo. Recomendación: Reiniciar manualmente" -ForegroundColor Yellow
    Write-Host "Usa: Ctrl+Shift+P > 'Developer: Reload Window'" -ForegroundColor Cyan
} else {
    Write-Host "✅ VS Code no está corriendo" -ForegroundColor Green
}

Write-Host "`n=== RECOVERY COMPLETE ===" -ForegroundColor Green
Write-Host "Próximos pasos:" -ForegroundColor Cyan
Write-Host "1. Abre VS Code" -ForegroundColor White
Write-Host "2. Si rust-analyzer sigue fallando, usa: Ctrl+Shift+P > 'Rust Analyzer: Restart Server'" -ForegroundColor White
Write-Host "3. Evita abrir muchos archivos grandes simultáneamente" -ForegroundColor White
Write-Host "4. Si el problema persiste, ejecuta este script nuevamente" -ForegroundColor White
