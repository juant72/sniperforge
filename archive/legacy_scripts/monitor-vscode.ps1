# VS Code Memory Monitor
# Monitorea el uso de memoria y detecta problemas potenciales

param(
    [int]$IntervalSeconds = 30,
    [int]$WarningThresholdMB = 2500,
    [int]$CriticalThresholdMB = 4000,
    [switch]$ContinuousMode
)

function Write-ColoredOutput {
    param($Message, $Color = "White")
    Write-Host $Message -ForegroundColor $Color
}

function Get-VSCodeMemoryInfo {
    $processes = Get-Process -Name "Code*" -ErrorAction SilentlyContinue
    if (-not $processes) {
        return $null
    }
    
    $totalMemory = ($processes | Measure-Object -Property WorkingSet64 -Sum).Sum
    $processCount = $processes.Count
    
    return @{
        TotalMemoryMB = [math]::Round($totalMemory / 1MB, 2)
        ProcessCount = $processCount
        Processes = $processes | Select-Object ProcessName, Id, WorkingSet64, StartTime
    }
}

function Show-MemoryStatus {
    param($MemoryInfo)
    
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    
    if (-not $MemoryInfo) {
        Write-ColoredOutput "[$timestamp] VS Code is not running" "Gray"
        return
    }
    
    $memoryMB = $MemoryInfo.TotalMemoryMB
    $processCount = $MemoryInfo.ProcessCount
    
    $status = "NORMAL"
    $color = "Green"
    
    if ($memoryMB -gt $CriticalThresholdMB) {
        $status = "CRITICAL"
        $color = "Red"
    } elseif ($memoryMB -gt $WarningThresholdMB) {
        $status = "WARNING"
        $color = "Yellow"
    }
    
    Write-ColoredOutput "[$timestamp] VS Code Memory: $memoryMB MB ($processCount processes) - $status" $color
    
    # Mostrar procesos individuales si hay problemas
    if ($status -ne "NORMAL") {
        Write-ColoredOutput "  Individual processes:" "Gray"
        foreach ($proc in $MemoryInfo.Processes) {
            $procMemoryMB = [math]::Round($proc.WorkingSet64 / 1MB, 2)
            Write-ColoredOutput "    $($proc.ProcessName) (PID: $($proc.Id)): $procMemoryMB MB" "Gray"
        }
    }
    
    return $status
}

function Show-Recommendations {
    param($Status)
    
    switch ($Status) {
        "WARNING" {
            Write-ColoredOutput "⚠️  Recommendations:" "Yellow"
            Write-ColoredOutput "   • Close unused tabs and files" "Gray"
            Write-ColoredOutput "   • Restart language servers (Ctrl+Shift+P -> 'Restart Extension Host')" "Gray"
            Write-ColoredOutput "   • Check for runaway extensions" "Gray"
        }
        "CRITICAL" {
            Write-ColoredOutput "🚨 Critical Recommendations:" "Red"
            Write-ColoredOutput "   • RESTART VS Code immediately" "White"
            Write-ColoredOutput "   • Run ./optimize-vscode.ps1 after restart" "White"
            Write-ColoredOutput "   • Check .vscode/settings.json is applied" "White"
            Write-ColoredOutput "   • Consider disabling unused extensions" "White"
        }
    }
}

# Main monitoring loop
Write-ColoredOutput "🖥️  VS Code Memory Monitor Started" "Cyan"
Write-ColoredOutput "===================================" "Cyan"
Write-ColoredOutput "Warning Threshold: $WarningThresholdMB MB" "Yellow"
Write-ColoredOutput "Critical Threshold: $CriticalThresholdMB MB" "Red"
Write-ColoredOutput "Interval: $IntervalSeconds seconds" "Gray"
Write-ColoredOutput "Press Ctrl+C to stop monitoring" "Gray"
Write-ColoredOutput ""

$iteration = 0
do {
    $iteration++
    
    try {
        $memoryInfo = Get-VSCodeMemoryInfo
        $status = Show-MemoryStatus -MemoryInfo $memoryInfo
        
        # Mostrar recomendaciones cada 5 iteraciones si hay problemas
        if ($status -ne "NORMAL" -and ($iteration % 5 -eq 0)) {
            Show-Recommendations -Status $status
            Write-ColoredOutput ""
        }
        
        # Log crítico a archivo
        if ($status -eq "CRITICAL") {
            $logFile = "vscode-memory-critical.log"
            $logEntry = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss'): CRITICAL - $($memoryInfo.TotalMemoryMB) MB"
            Add-Content -Path $logFile -Value $logEntry
        }
        
    } catch {
        Write-ColoredOutput "Error monitoring VS Code: $_" "Red"
    }
    
    if ($ContinuousMode) {
        Start-Sleep $IntervalSeconds
    }
    
} while ($ContinuousMode)

if (-not $ContinuousMode) {
    Write-ColoredOutput ""
    Write-ColoredOutput "Single check completed. Use -ContinuousMode for ongoing monitoring." "Cyan"
}
