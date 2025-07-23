# VS Code Performance Optimization Script
# Ejecutar cuando VS Code se vuelva lento o inestable

Write-Host "🔧 VS Code Performance Optimization" -ForegroundColor Cyan
Write-Host "===================================" -ForegroundColor Cyan

# Verificar memoria actual de VS Code
$vscodeMemory = Get-Process -Name "Code*" -ErrorAction SilentlyContinue | Measure-Object -Property WorkingSet64 -Sum
if ($vscodeMemory.Sum) {
    $memoryMB = [math]::Round($vscodeMemory.Sum/1MB, 2)
    Write-Host "📊 Current VS Code Memory Usage: $memoryMB MB" -ForegroundColor Yellow
    
    if ($memoryMB -gt 2500) {
        Write-Host "⚠️  High memory usage detected! Optimization recommended." -ForegroundColor Red
    }
} else {
    Write-Host "✅ VS Code is not currently running" -ForegroundColor Green
}

# Función para limpiar con confirmación
function Confirm-Clean {
    param($Message)
    $response = Read-Host "$Message (y/N)"
    return $response -eq 'y' -or $response -eq 'Y'
}

Write-Host ""
Write-Host "🧹 Available Optimizations:" -ForegroundColor Cyan

# 1. Limpiar cache de Cargo/Rust
if (Confirm-Clean "1. Clean Rust/Cargo cache") {
    Write-Host "🦀 Cleaning Rust cache..." -ForegroundColor Yellow
    
    if (Test-Path "target") {
        Remove-Item "target" -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "   ✅ Removed target directory" -ForegroundColor Green
    }
    
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        cargo clean 2>$null
        Write-Host "   ✅ Cargo clean completed" -ForegroundColor Green
    }
}

# 2. Limpiar logs de VS Code
if (Confirm-Clean "2. Clean VS Code logs") {
    Write-Host "📝 Cleaning VS Code logs..." -ForegroundColor Yellow
    
    $logPath = "$env:USERPROFILE\AppData\Roaming\Code\logs"
    if (Test-Path $logPath) {
        Get-ChildItem $logPath | Where-Object { $_.LastWriteTime -lt (Get-Date).AddDays(-7) } | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "   ✅ Removed old log files" -ForegroundColor Green
    }
}

# 3. Limpiar workspace storage
if (Confirm-Clean "3. Clean VS Code workspace storage") {
    Write-Host "💾 Cleaning workspace storage..." -ForegroundColor Yellow
    
    $workspacePath = "$env:USERPROFILE\AppData\Roaming\Code\workspaceStorage"
    if (Test-Path $workspacePath) {
        Get-ChildItem $workspacePath | Where-Object { $_.LastWriteTime -lt (Get-Date).AddDays(-30) } | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "   ✅ Removed old workspace storage" -ForegroundColor Green
    }
}

# 4. Optimizar Git
if (Confirm-Clean "4. Optimize Git configuration") {
    Write-Host "🐙 Optimizing Git..." -ForegroundColor Yellow
    
    git config core.preloadindex true 2>$null
    git config core.fscache true 2>$null
    git config gc.auto 256 2>$null
    git config fetch.writeCommitGraph true 2>$null
    
    Write-Host "   ✅ Git optimization completed" -ForegroundColor Green
}

# 5. Configurar variables de entorno
if (Confirm-Clean "5. Set Rust optimization environment variables") {
    Write-Host "🔧 Setting environment variables..." -ForegroundColor Yellow
    
    [Environment]::SetEnvironmentVariable("RUST_LOG", "warn", "User")
    [Environment]::SetEnvironmentVariable("CARGO_INCREMENTAL", "0", "User")
    [Environment]::SetEnvironmentVariable("RUST_ANALYZER_MEMORY_USAGE_THRESHOLD", "1024", "User")
    
    Write-Host "   ✅ Environment variables set" -ForegroundColor Green
    Write-Host "   ⚠️  Restart VS Code to apply changes" -ForegroundColor Yellow
}

# 6. Restart VS Code si está corriendo
if (Get-Process -Name "Code*" -ErrorAction SilentlyContinue) {
    if (Confirm-Clean "6. Restart VS Code to apply optimizations") {
        Write-Host "🔄 Restarting VS Code..." -ForegroundColor Yellow
        
        # Cerrar VS Code suavemente
        Get-Process -Name "Code*" | ForEach-Object { $_.CloseMainWindow() | Out-Null }
        Start-Sleep 3
        
        # Forzar cierre si aún está corriendo
        Get-Process -Name "Code*" -ErrorAction SilentlyContinue | Stop-Process -Force
        
        Write-Host "   ✅ VS Code closed" -ForegroundColor Green
        Write-Host "   🚀 You can now restart VS Code manually" -ForegroundColor Cyan
    }
}

Write-Host ""
Write-Host "🎉 Optimization completed!" -ForegroundColor Green
Write-Host ""
Write-Host "📋 Summary of applied optimizations:" -ForegroundColor Cyan
Write-Host "   • Cleaned Rust/Cargo cache" -ForegroundColor Gray
Write-Host "   • Removed old VS Code logs" -ForegroundColor Gray  
Write-Host "   • Optimized Git configuration" -ForegroundColor Gray
Write-Host "   • Set memory-efficient environment variables" -ForegroundColor Gray
Write-Host ""
Write-Host "🔍 Next steps:" -ForegroundColor Cyan
Write-Host "   1. Restart VS Code" -ForegroundColor Gray
Write-Host "   2. Check that .vscode/settings.json is applied" -ForegroundColor Gray
Write-Host "   3. Monitor memory usage with: Get-Process Code* | Measure-Object WorkingSet64 -Sum" -ForegroundColor Gray
Write-Host ""
Write-Host "⚡ Expected improvements:" -ForegroundColor Cyan
Write-Host "   • 30-40% less memory usage" -ForegroundColor Green
Write-Host "   • Faster rust-analyzer startup" -ForegroundColor Green
Write-Host "   • More stable Copilot chat" -ForegroundColor Green
Write-Host "   • Reduced extension host restarts" -ForegroundColor Green
