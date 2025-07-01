# Script para reiniciar Rust Analyzer y limpiar cachés
# Basado en las mejores prácticas oficiales de VS Code y Rust Analyzer

param(
    [switch]$Force = $false,
    [switch]$CleanCache = $true,
    [switch]$CheckResources = $true
)

Write-Host "� Reiniciando Rust Analyzer - Script Anti-Crash" -ForegroundColor Cyan

# Función para verificar recursos del sistema
function Check-SystemResources {
    if ($CheckResources) {
        Write-Host "📊 Verificando recursos del sistema..." -ForegroundColor Yellow

        # Memoria disponible
        $memory = Get-WmiObject Win32_OperatingSystem
        $totalMem = [math]::Round($memory.TotalVisibleMemorySize / 1024 / 1024, 2)
        $freeMem = [math]::Round($memory.FreePhysicalMemory / 1024 / 1024, 2)
        $usedMem = $totalMem - $freeMem
        $memoryUsage = [math]::Round(($usedMem / $totalMem) * 100, 2)

        Write-Host "   � Memoria: $usedMem GB / $totalMem GB ($memoryUsage% usado)" -ForegroundColor White

        if ($memoryUsage -gt 85) {
            Write-Host "   ⚠️  ADVERTENCIA: Uso de memoria muy alto ($memoryUsage%)" -ForegroundColor Red
        }

        # Procesos de VS Code
        $vscodeProcesses = Get-Process | Where-Object { $_.ProcessName -like "*Code*" -or $_.ProcessName -like "*rust-analyzer*" }
        if ($vscodeProcesses) {
            Write-Host "   🔍 Procesos VS Code/Rust Analyzer encontrados:" -ForegroundColor White
            foreach ($proc in $vscodeProcesses) {
                $memMB = [math]::Round($proc.WorkingSet64 / 1024 / 1024, 2)
                Write-Host "      - $($proc.ProcessName): $memMB MB" -ForegroundColor Gray
            }
        }
    }
}

# Función para limpiar cachés
function Clean-RustAnalyzerCache {
    if ($CleanCache) {
        Write-Host "🧹 Limpiando cachés de Rust Analyzer..." -ForegroundColor Yellow

        # Limpiar target directories
        $targetDirs = @(
            "target\debug",
            "target\release",
            "target\rust-analyzer",
            "target\ra-check",
            "target\tmp"
        )

        foreach ($dir in $targetDirs) {
            if (Test-Path $dir) {
                try {
                    Write-Host "   🗑️ Limpiando $dir..." -ForegroundColor Gray
                    Remove-Item $dir -Recurse -Force -ErrorAction SilentlyContinue
                } catch {
                    Write-Host "   ⚠️ No se pudo limpiar $dir" -ForegroundColor Yellow
                }
            }
        }

        # Limpiar caché de VS Code para Rust Analyzer
        $vscodeCache = "$env:APPDATA\Code\CachedExtensions\rust-lang.rust-analyzer-*"
        if (Test-Path $vscodeCache) {
            try {
                Write-Host "   🗑️ Limpiando caché de extensión..." -ForegroundColor Gray
                Remove-Item $vscodeCache -Recurse -Force -ErrorAction SilentlyContinue
            } catch {
                Write-Host "   ⚠️ No se pudo limpiar caché de extensión" -ForegroundColor Yellow
            }
        }
    }
}

# Función para matar procesos de Rust Analyzer
function Stop-RustAnalyzer {
    Write-Host "� Deteniendo procesos de Rust Analyzer..." -ForegroundColor Yellow

    $processes = Get-Process | Where-Object {
        $_.ProcessName -like "*rust-analyzer*" -or
        $_.ProcessName -like "*rust-language-server*"
    }

    if ($processes) {
        foreach ($proc in $processes) {
            try {
                Write-Host "   � Terminando proceso: $($proc.ProcessName) (PID: $($proc.Id))" -ForegroundColor Gray
                $proc | Stop-Process -Force
            } catch {
                Write-Host "   ⚠️ No se pudo terminar proceso $($proc.ProcessName)" -ForegroundColor Yellow
            }
        }
    } else {
        Write-Host "   ✅ No se encontraron procesos de Rust Analyzer en ejecución" -ForegroundColor Green
    }
}

# Función para verificar estado de VS Code
function Check-VSCodeStatus {
    $vscodeProcesses = Get-Process | Where-Object { $_.ProcessName -eq "Code" }

    if ($vscodeProcesses) {
        Write-Host "📋 VS Code está ejecutándose. Procesos encontrados:" -ForegroundColor White
        foreach ($proc in $vscodeProcesses) {
            $memMB = [math]::Round($proc.WorkingSet64 / 1024 / 1024, 2)
            $cpuTime = $proc.TotalProcessorTime.TotalSeconds
            Write-Host "   - PID: $($proc.Id), Memoria: $memMB MB, CPU: $cpuTime seg" -ForegroundColor Gray
        }
        return $true
    } else {
        Write-Host "   ℹ️ VS Code no está ejecutándose" -ForegroundColor Blue
        return $false
    }
}

# Ejecutar verificaciones
Write-Host ""
Check-SystemResources
Write-Host ""

$vscodeRunning = Check-VSCodeStatus
Write-Host ""

# Ejecutar limpieza
Clean-RustAnalyzerCache
Write-Host ""

# Detener Rust Analyzer
Stop-RustAnalyzer
Write-Host ""

# Esperar un momento
Write-Host "⏳ Esperando 3 segundos..." -ForegroundColor Blue
Start-Sleep -Seconds 3

# Reiniciar VS Code si estaba ejecutándose
if ($vscodeRunning -or $Force) {
    Write-Host "� Ejecutando comando de reinicio de Rust Analyzer en VS Code..." -ForegroundColor Green

    # Intentar ejecutar comando de VS Code para reiniciar Rust Analyzer
    try {
        code --command rust-analyzer.restart 2>$null
        Start-Sleep -Seconds 2
    } catch {
        Write-Host "   ⚠️ No se pudo ejecutar comando directo, VS Code necesita estar abierto" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "✅ Proceso de reinicio completado" -ForegroundColor Green
Write-Host ""
Write-Host "📝 Para mejores resultados:" -ForegroundColor Cyan
Write-Host "   1. Abre VS Code si no está abierto" -ForegroundColor White
Write-Host "   2. Presiona Ctrl+Shift+P" -ForegroundColor White
Write-Host "   3. Ejecuta: 'Rust Analyzer: Restart server'" -ForegroundColor White
Write-Host "   4. Si persisten problemas, ejecuta: 'Developer: Restart Extension Host'" -ForegroundColor White
Write-Host ""

# Verificación final de recursos
if ($CheckResources) {
    Write-Host "📊 Estado final del sistema:" -ForegroundColor Cyan
    Check-SystemResources
}
