# VS Code Crash Prevention Script
# Ejecutar proactivamente para evitar crashes durante desarrollo

param(
    [switch]$AutoFix,
    [switch]$Quiet
)

if (!$Quiet) {
    Write-Host "🛡️  VS Code Crash Prevention" -ForegroundColor Green
    Write-Host "============================" -ForegroundColor Green
}

$ErrorActionPreference = "SilentlyContinue"

# Función para logging condicional
function Write-ConditionalHost {
    param($Message, $Color = "White")
    if (!$Quiet) {
        Write-Host $Message -ForegroundColor $Color
    }
}

# 1. Verificar y limpiar memoria si es necesario
$vscodeProcesses = Get-Process -Name "Code*" -ErrorAction SilentlyContinue
if ($vscodeProcesses) {
    $totalMemory = ($vscodeProcesses | Measure-Object -Property WorkingSet64 -Sum).Sum
    $memoryMB = [math]::Round($totalMemory/1MB, 2)
    
    if ($memoryMB -gt 2500) {
        Write-ConditionalHost "⚠️  High memory usage detected: $memoryMB MB" "Yellow"
        
        if ($AutoFix) {
            Write-ConditionalHost "🔧 Auto-fixing: Applying memory optimizations..." "Yellow"
            
            # Configurar variables de entorno para reducir uso de memoria
            [Environment]::SetEnvironmentVariable("RUST_ANALYZER_MEMORY_USAGE_THRESHOLD", "1024", "Process")
            [Environment]::SetEnvironmentVariable("RUST_LOG", "warn", "Process")
            [Environment]::SetEnvironmentVariable("CARGO_INCREMENTAL", "0", "Process")
            
            Write-ConditionalHost "✅ Memory optimizations applied" "Green"
        } else {
            Write-ConditionalHost "💡 Recommendation: Run with -AutoFix flag or manually optimize" "Cyan"
        }
    } else {
        Write-ConditionalHost "✅ Memory usage is healthy: $memoryMB MB" "Green"
    }
}

# 2. Verificar tamaño del workspace y limpiar si es necesario
$targetDir = ".\target"
if (Test-Path $targetDir) {
    $targetSize = (Get-ChildItem $targetDir -Recurse -File | Measure-Object -Property Length -Sum).Sum
    $targetSizeGB = [math]::Round($targetSize/1GB, 2)
    
    if ($targetSizeGB -gt 3) {
        Write-ConditionalHost "⚠️  Large target directory detected: $targetSizeGB GB" "Yellow"
        
        if ($AutoFix) {
            Write-ConditionalHost "🧹 Auto-fixing: Cleaning Rust cache..." "Yellow"
            cargo clean | Out-Null
            Write-ConditionalHost "✅ Rust cache cleaned" "Green"
        } else {
            Write-ConditionalHost "💡 Recommendation: Run 'cargo clean' to free up space" "Cyan"
        }
    } else {
        Write-ConditionalHost "✅ Target directory size is reasonable: $targetSizeGB GB" "Green"
    }
}

# 3. Verificar configuraciones problemáticas
$rustLog = [Environment]::GetEnvironmentVariable("RUST_LOG", "User")
if ($rustLog -eq "debug" -or $rustLog -eq "trace") {
    Write-ConditionalHost "⚠️  Verbose logging detected: RUST_LOG=$rustLog" "Yellow"
    
    if ($AutoFix) {
        Write-ConditionalHost "🔧 Auto-fixing: Setting RUST_LOG to warn level..." "Yellow"
        [Environment]::SetEnvironmentVariable("RUST_LOG", "warn", "User")
        Write-ConditionalHost "✅ RUST_LOG optimized" "Green"
    }
}

# 4. Verificar procesos zombies de VS Code
$codeProcesses = Get-Process -Name "Code*" -ErrorAction SilentlyContinue
$suspiciousProcesses = $codeProcesses | Where-Object { 
    $_.WorkingSet64 -lt 50MB -or $_.CPU -eq 0 
}

if ($suspiciousProcesses.Count -gt 0) {
    Write-ConditionalHost "⚠️  Found $($suspiciousProcesses.Count) potentially stuck VS Code processes" "Yellow"
    
    if ($AutoFix) {
        Write-ConditionalHost "🔧 Auto-fixing: Cleaning up stuck processes..." "Yellow"
        $suspiciousProcesses | Stop-Process -Force
        Write-ConditionalHost "✅ Stuck processes cleaned up" "Green"
    }
}

# 5. Verificar logs de errores recientes y limpiar si hay muchos
$logDir = ".\logs"
if (Test-Path $logDir) {
    $logFiles = Get-ChildItem $logDir -File | Where-Object { $_.LastWriteTime -gt (Get-Date).AddDays(-7) }
    $totalLogSize = ($logFiles | Measure-Object -Property Length -Sum).Sum
    $logSizeMB = [math]::Round($totalLogSize/1MB, 2)
    
    if ($logSizeMB -gt 100) {
        Write-ConditionalHost "⚠️  Large log files detected: $logSizeMB MB" "Yellow"
        
        if ($AutoFix) {
            Write-ConditionalHost "🧹 Auto-fixing: Cleaning old log files..." "Yellow"
            $oldLogs = Get-ChildItem $logDir -File | Where-Object { $_.LastWriteTime -lt (Get-Date).AddDays(-3) }
            $oldLogs | Remove-Item -Force
            Write-ConditionalHost "✅ Old log files cleaned" "Green"
        }
    }
}

# 6. Verificar extensiones problemáticas
if ($AutoFix) {
    # Verificar si rust-analyzer está consumiendo demasiados recursos
    $rustAnalyzerProc = Get-Process -Name "rust-analyzer*" -ErrorAction SilentlyContinue
    if ($rustAnalyzerProc) {
        $rustAnalyzerMemory = [math]::Round($rustAnalyzerProc.WorkingSet64/1MB, 2)
        if ($rustAnalyzerMemory -gt 1500) {
            Write-ConditionalHost "⚠️  Rust-analyzer using $rustAnalyzerMemory MB, restarting..." "Yellow"
            $rustAnalyzerProc | Stop-Process -Force
            Write-ConditionalHost "✅ Rust-analyzer restarted" "Green"
        }
    }
}

# 7. Crear o actualizar archivo de monitoreo
$statusFile = ".\vscode-health-status.json"
$healthData = @{
    timestamp = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss")
    memory_usage_mb = if ($vscodeProcesses) { [math]::Round(($vscodeProcesses | Measure-Object -Property WorkingSet64 -Sum).Sum/1MB, 2) } else { 0 }
    workspace_size_gb = if (Test-Path $targetDir) { [math]::Round((Get-ChildItem $targetDir -Recurse -File | Measure-Object -Property Length -Sum).Sum/1GB, 2) } else { 0 }
    process_count = if ($vscodeProcesses) { $vscodeProcesses.Count } else { 0 }
    auto_fix_applied = $AutoFix
    health_score = "good"
}

# Calcular score de salud
if ($healthData.memory_usage_mb -gt 3000 -or $healthData.workspace_size_gb -gt 5) {
    $healthData.health_score = "critical"
} elseif ($healthData.memory_usage_mb -gt 2000 -or $healthData.workspace_size_gb -gt 3) {
    $healthData.health_score = "warning"
}

$healthData | ConvertTo-Json | Out-File $statusFile -Encoding UTF8

if (!$Quiet) {
    Write-Host ""
    Write-Host "📊 HEALTH SUMMARY" -ForegroundColor Cyan
    Write-Host "=================" -ForegroundColor Cyan
    Write-Host "Health Score: $($healthData.health_score.ToUpper())" -ForegroundColor $(if ($healthData.health_score -eq "good") { "Green" } elseif ($healthData.health_score -eq "warning") { "Yellow" } else { "Red" })
    Write-Host "Memory Usage: $($healthData.memory_usage_mb) MB" -ForegroundColor Gray
    Write-Host "Workspace Size: $($healthData.workspace_size_gb) GB" -ForegroundColor Gray
    Write-Host "VS Code Processes: $($healthData.process_count)" -ForegroundColor Gray
    
    if ($AutoFix) {
        Write-Host "✅ Auto-fixes applied where needed" -ForegroundColor Green
    } else {
        Write-Host "ℹ️  Run with -AutoFix to automatically resolve issues" -ForegroundColor Cyan
    }
    
    Write-Host ""
    Write-Host "💡 PREVENTION TIPS:" -ForegroundColor Yellow
    Write-Host "   • Run this script every 30 minutes during development" -ForegroundColor Gray
    Write-Host "   • Use 'Ctrl+Shift+P > Developer: Reload Window' instead of restarting" -ForegroundColor Gray
    Write-Host "   • Close unused editor tabs and extensions regularly" -ForegroundColor Gray
    Write-Host "   • Consider using VS Code Insiders for better stability" -ForegroundColor Gray
}

# Devolver código de salida basado en health score
switch ($healthData.health_score) {
    "critical" { exit 2 }
    "warning" { exit 1 }
    default { exit 0 }
}
