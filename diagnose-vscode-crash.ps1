# VS Code Crash Diagnosis and Mitigation Script
# Ejecutar cuando VS Code se crashea o se resetea inesperadamente

Write-Host "🔍 VS Code Crash Diagnosis Tool" -ForegroundColor Red
Write-Host "================================" -ForegroundColor Red

$ErrorActionPreference = "SilentlyContinue"

# 1. Verificar uso de memoria actual
Write-Host ""
Write-Host "📊 1. MEMORY USAGE ANALYSIS" -ForegroundColor Cyan
Write-Host "===========================" -ForegroundColor Cyan

$vscodeProcesses = Get-Process -Name "Code*" -ErrorAction SilentlyContinue
if ($vscodeProcesses) {
    $totalMemory = ($vscodeProcesses | Measure-Object -Property WorkingSet64 -Sum).Sum
    $memoryMB = [math]::Round($totalMemory/1MB, 2)
    $processCount = $vscodeProcesses.Count
    
    Write-Host "🔢 VS Code Processes: $processCount" -ForegroundColor Yellow
    Write-Host "💾 Total Memory Usage: $memoryMB MB" -ForegroundColor Yellow
    
    foreach ($proc in $vscodeProcesses) {
        $procMemory = [math]::Round($proc.WorkingSet64/1MB, 2)
        Write-Host "   📝 $($proc.ProcessName) (PID: $($proc.Id)): $procMemory MB" -ForegroundColor Gray
    }
    
    if ($memoryMB -gt 3000) {
        Write-Host "❌ CRITICAL: Memory usage exceeds 3GB!" -ForegroundColor Red
        Write-Host "   This can cause VS Code crashes on Windows" -ForegroundColor Red
    } elseif ($memoryMB -gt 2000) {
        Write-Host "⚠️  WARNING: High memory usage detected" -ForegroundColor Yellow
    } else {
        Write-Host "✅ Memory usage is normal" -ForegroundColor Green
    }
} else {
    Write-Host "✅ VS Code is not currently running" -ForegroundColor Green
}

# 2. Verificar el tamaño del workspace
Write-Host ""
Write-Host "📁 2. WORKSPACE SIZE ANALYSIS" -ForegroundColor Cyan
Write-Host "==============================" -ForegroundColor Cyan

$targetDir = ".\target"
if (Test-Path $targetDir) {
    $targetSize = (Get-ChildItem $targetDir -Recurse -File | Measure-Object -Property Length -Sum).Sum
    $targetSizeMB = [math]::Round($targetSize/1MB, 2)
    $targetSizeGB = [math]::Round($targetSize/1GB, 2)
    
    Write-Host "🦀 Rust target directory: $targetSizeMB MB ($targetSizeGB GB)" -ForegroundColor Yellow
    
    if ($targetSizeGB -gt 5) {
        Write-Host "❌ CRITICAL: Target directory is very large!" -ForegroundColor Red
        Write-Host "   This can cause file system performance issues" -ForegroundColor Red
        Write-Host "   Recommendation: Run 'cargo clean' immediately" -ForegroundColor Red
    } elseif ($targetSizeGB -gt 2) {
        Write-Host "⚠️  WARNING: Target directory is large" -ForegroundColor Yellow
        Write-Host "   Consider running 'cargo clean' periodically" -ForegroundColor Yellow
    }
}

# 3. Verificar logs recientes del sistema
Write-Host ""
Write-Host "📝 3. SYSTEM EVENT LOGS" -ForegroundColor Cyan
Write-Host "=======================" -ForegroundColor Cyan

# Buscar eventos relacionados con crashes en las últimas 24 horas
$events = Get-WinEvent -FilterHashtable @{LogName='Application'; StartTime=(Get-Date).AddHours(-24)} -MaxEvents 10 | 
    Where-Object { $_.LevelDisplayName -eq 'Error' -and ($_.Message -like '*Code*' -or $_.Message -like '*Electron*' -or $_.Message -like '*memory*') }

if ($events) {
    Write-Host "⚠️  Found system errors in the last 24 hours:" -ForegroundColor Yellow
    foreach ($event in $events) {
        Write-Host "   📅 $($event.TimeCreated): $($event.LevelDisplayName) - $($event.Id)" -ForegroundColor Gray
        Write-Host "   📋 $($event.Message.Substring(0, [Math]::Min(100, $event.Message.Length)))..." -ForegroundColor Gray
    }
} else {
    Write-Host "✅ No VS Code related system errors found" -ForegroundColor Green
}

# 4. Verificar archivos de configuración problemáticos
Write-Host ""
Write-Host "⚙️  4. CONFIGURATION ANALYSIS" -ForegroundColor Cyan
Write-Host "==============================" -ForegroundColor Cyan

# Verificar si hay configuraciones que puedan causar problemas
$rustAnalyzerMemory = [Environment]::GetEnvironmentVariable("RUST_ANALYZER_MEMORY_USAGE_THRESHOLD", "User")
if ($rustAnalyzerMemory) {
    Write-Host "🔧 Rust Analyzer Memory Limit: $rustAnalyzerMemory MB" -ForegroundColor Yellow
} else {
    Write-Host "⚠️  No Rust Analyzer memory limit set" -ForegroundColor Yellow
    Write-Host "   Recommendation: Set RUST_ANALYZER_MEMORY_USAGE_THRESHOLD=1024" -ForegroundColor Gray
}

$rustLog = [Environment]::GetEnvironmentVariable("RUST_LOG", "User")
if ($rustLog -eq "debug" -or $rustLog -eq "trace") {
    Write-Host "❌ RUST_LOG is set to verbose level: $rustLog" -ForegroundColor Red
    Write-Host "   This can cause excessive memory usage and log generation" -ForegroundColor Red
} elseif ($rustLog) {
    Write-Host "✅ RUST_LOG level: $rustLog" -ForegroundColor Green
} else {
    Write-Host "ℹ️  RUST_LOG not set (using default)" -ForegroundColor Gray
}

# 5. Verificar patrones problemáticos en código
Write-Host ""
Write-Host "🔍 5. CODE PATTERN ANALYSIS" -ForegroundColor Cyan
Write-Host "============================" -ForegroundColor Cyan

# Contar loops y funciones recursivas que podrían causar problemas
$loopCount = (Select-String -Path "src\**\*.rs" -Pattern "loop\s*\{" -ErrorAction SilentlyContinue | Measure-Object).Count
$whileCount = (Select-String -Path "src\**\*.rs" -Pattern "while\s+.*\{" -ErrorAction SilentlyContinue | Measure-Object).Count
$unwrapCount = (Select-String -Path "src\**\*.rs" -Pattern "\.unwrap\(\)" -ErrorAction SilentlyContinue | Measure-Object).Count

Write-Host "🔄 Infinite loops found: $loopCount" -ForegroundColor Yellow
Write-Host "🔁 While loops found: $whileCount" -ForegroundColor Yellow
Write-Host "💥 .unwrap() calls found: $unwrapCount" -ForegroundColor Yellow

if ($unwrapCount -gt 50) {
    Write-Host "⚠️  High number of .unwrap() calls detected" -ForegroundColor Yellow
    Write-Host "   These can cause panics and crashes" -ForegroundColor Yellow
}

# 6. Recomendaciones de mitigación
Write-Host ""
Write-Host "🛠️  6. MITIGATION RECOMMENDATIONS" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan

$recommendations = @()

if ($targetSizeGB -gt 2) {
    $recommendations += "1. Run 'cargo clean' to reduce workspace size"
}

if ($memoryMB -gt 2000 -or !$rustAnalyzerMemory) {
    $recommendations += "2. Run '.\optimize-vscode.ps1' to apply memory optimizations"
}

if ($loopCount -gt 20) {
    $recommendations += "3. Review infinite loops for proper timeout/break conditions"
}

if ($unwrapCount -gt 50) {
    $recommendations += "4. Replace .unwrap() calls with proper error handling"
}

$recommendations += "5. Restart VS Code regularly (every 2-3 hours of heavy development)"
$recommendations += "6. Close unused VS Code windows and extensions"
$recommendations += "7. Use VS Code Insiders for better memory management"

Write-Host "📋 Immediate actions to prevent crashes:" -ForegroundColor Green
foreach ($rec in $recommendations) {
    Write-Host "   $rec" -ForegroundColor Gray
}

# 7. Quick fix options
Write-Host ""
Write-Host "⚡ 7. QUICK FIX OPTIONS" -ForegroundColor Cyan
Write-Host "======================" -ForegroundColor Cyan

Write-Host "Choose a quick fix:"
Write-Host "1. Clean Rust cache (cargo clean)"
Write-Host "2. Apply VS Code memory optimizations"
Write-Host "3. Kill and restart all VS Code processes"
Write-Host "4. Full cleanup (cache + optimizations + restart)"
Write-Host "5. Skip quick fixes"

$choice = Read-Host "Enter choice (1-5)"

switch ($choice) {
    "1" {
        Write-Host "🧹 Cleaning Rust cache..." -ForegroundColor Yellow
        cargo clean
        Write-Host "✅ Rust cache cleaned" -ForegroundColor Green
    }
    "2" {
        Write-Host "🔧 Applying VS Code optimizations..." -ForegroundColor Yellow
        .\optimize-vscode.ps1
    }
    "3" {
        Write-Host "🔄 Restarting VS Code..." -ForegroundColor Yellow
        Get-Process -Name "Code*" -ErrorAction SilentlyContinue | Stop-Process -Force
        Start-Sleep 2
        Write-Host "✅ VS Code processes terminated" -ForegroundColor Green
        Write-Host "   You can now restart VS Code manually" -ForegroundColor Cyan
    }
    "4" {
        Write-Host "🚀 Performing full cleanup..." -ForegroundColor Yellow
        cargo clean
        [Environment]::SetEnvironmentVariable("RUST_ANALYZER_MEMORY_USAGE_THRESHOLD", "1024", "User")
        [Environment]::SetEnvironmentVariable("RUST_LOG", "warn", "User")
        Get-Process -Name "Code*" -ErrorAction SilentlyContinue | Stop-Process -Force
        Write-Host "✅ Full cleanup completed" -ForegroundColor Green
    }
    default {
        Write-Host "ℹ️  No quick fixes applied" -ForegroundColor Gray
    }
}

Write-Host ""
Write-Host "🎯 SUMMARY" -ForegroundColor Green
Write-Host "==========" -ForegroundColor Green
Write-Host "Most likely crash causes based on analysis:" -ForegroundColor White

if ($memoryMB -gt 3000) {
    Write-Host "1. 🔥 HIGH MEMORY USAGE - Primary suspect" -ForegroundColor Red
} elseif ($memoryMB -gt 2000) {
    Write-Host "1. ⚠️  Moderate memory usage - Monitor closely" -ForegroundColor Yellow
}

if ($targetSizeGB -gt 5) {
    Write-Host "2. 🔥 LARGE WORKSPACE - Contributing factor" -ForegroundColor Red
} elseif ($targetSizeGB -gt 2) {
    Write-Host "2. ⚠️  Large workspace - Minor factor" -ForegroundColor Yellow
}

if ($loopCount -gt 30) {
    Write-Host "3. 🔥 MANY INFINITE LOOPS - Potential CPU issue" -ForegroundColor Red
}

Write-Host ""
Write-Host "💡 To prevent future crashes:" -ForegroundColor Cyan
Write-Host "   • Run this script daily during development" -ForegroundColor Gray
Write-Host "   • Set up automated cargo clean (weekly)" -ForegroundColor Gray
Write-Host "   • Monitor VS Code memory usage regularly" -ForegroundColor Gray
Write-Host "   • Use VS Code's Reload Window command instead of closing/reopening" -ForegroundColor Gray

Write-Host ""
Write-Host "🔧 Script completed. Check recommendations above." -ForegroundColor Green
