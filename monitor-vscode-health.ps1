# VS Code Health Monitor - Continuous Background Monitoring
# Ejecutar en segundo plano para monitoreo continuo

param(
    [int]$IntervalMinutes = 30,
    [switch]$EnableAutoFix,
    [switch]$EnableNotifications,
    [string]$LogFile = ".\vscode-monitor.log"
)

Write-Host "🏥 VS Code Health Monitor Started" -ForegroundColor Green
Write-Host "==================================" -ForegroundColor Green
Write-Host "Interval: $IntervalMinutes minutes" -ForegroundColor Gray
Write-Host "Auto-fix: $(if ($EnableAutoFix) { 'Enabled' } else { 'Disabled' })" -ForegroundColor Gray
Write-Host "Notifications: $(if ($EnableNotifications) { 'Enabled' } else { 'Disabled' })" -ForegroundColor Gray
Write-Host "Log file: $LogFile" -ForegroundColor Gray
Write-Host ""
Write-Host "Press Ctrl+C to stop monitoring..." -ForegroundColor Yellow
Write-Host ""

$monitoringActive = $true
$checkCount = 0

# Función para logging
function Write-MonitorLog {
    param($Message, $Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logEntry = "[$timestamp] [$Level] $Message"
    Write-Host $logEntry -ForegroundColor $(if ($Level -eq "ERROR") { "Red" } elseif ($Level -eq "WARN") { "Yellow" } else { "Gray" })
    Add-Content -Path $LogFile -Value $logEntry
}

# Función para notificaciones Windows
function Send-WindowsNotification {
    param($Title, $Message, $Type = "Info")
    
    if ($EnableNotifications) {
        try {
            Add-Type -AssemblyName System.Windows.Forms
            $icon = [System.Windows.Forms.ToolTipIcon]::$Type
            $notification = New-Object System.Windows.Forms.NotifyIcon
            $notification.Icon = [System.Drawing.SystemIcons]::Information
            $notification.BalloonTipTitle = $Title
            $notification.BalloonTipText = $Message
            $notification.BalloonTipIcon = $icon
            $notification.Visible = $true
            $notification.ShowBalloonTip(5000)
            Start-Sleep 1
            $notification.Dispose()
        } catch {
            Write-MonitorLog "Failed to send notification: $_" "WARN"
        }
    }
}

# Manejador de Ctrl+C
Register-EngineEvent PowerShell.Exiting -Action {
    Write-Host "`n🛑 Monitor stopped by user" -ForegroundColor Yellow
    $monitoringActive = $false
}

try {
    # Crear log file si no existe
    if (!(Test-Path $LogFile)) {
        New-Item -Path $LogFile -ItemType File -Force | Out-Null
    }
    
    Write-MonitorLog "VS Code Health Monitor started (PID: $PID)"
    
    while ($monitoringActive) {
        $checkCount++
        Write-MonitorLog "Performing health check #$checkCount"
        
        try {
            # Ejecutar el script de prevención de crashes
            $result = & ".\prevent-vscode-crash.ps1" -Quiet $(if ($EnableAutoFix) { "-AutoFix" })
            $exitCode = $LASTEXITCODE
            
            # Leer el estado de salud
            $statusFile = ".\vscode-health-status.json"
            if (Test-Path $statusFile) {
                $healthData = Get-Content $statusFile | ConvertFrom-Json
                
                $memoryMB = $healthData.memory_usage_mb
                $workspaceGB = $healthData.workspace_size_gb
                $healthScore = $healthData.health_score
                $processCount = $healthData.process_count
                
                Write-MonitorLog "Health check #$checkCount: Score=$healthScore, Memory=${memoryMB}MB, Workspace=${workspaceGB}GB, Processes=$processCount"
                
                # Notificaciones basadas en el estado de salud
                switch ($healthScore) {
                    "critical" {
                        $message = "VS Code health critical! Memory: ${memoryMB}MB, Workspace: ${workspaceGB}GB"
                        Write-MonitorLog $message "ERROR"
                        Send-WindowsNotification "VS Code Health Alert" $message "Error"
                        
                        if ($EnableAutoFix) {
                            Write-MonitorLog "Auto-fix enabled, applying critical fixes..."
                            & ".\prevent-vscode-crash.ps1" -AutoFix -Quiet
                            Write-MonitorLog "Critical auto-fixes applied"
                        }
                    }
                    "warning" {
                        $message = "VS Code performance warning. Memory: ${memoryMB}MB, Workspace: ${workspaceGB}GB"
                        Write-MonitorLog $message "WARN"
                        Send-WindowsNotification "VS Code Performance Warning" $message "Warning"
                    }
                    "good" {
                        Write-MonitorLog "VS Code health is good (Memory: ${memoryMB}MB, Workspace: ${workspaceGB}GB)"
                    }
                }
                
                # Alertas específicas por umbral
                if ($memoryMB -gt 3500) {
                    $alertMsg = "MEMORY ALERT: VS Code using ${memoryMB}MB (>3.5GB)"
                    Write-MonitorLog $alertMsg "ERROR"
                    Send-WindowsNotification "Memory Alert" $alertMsg "Error"
                }
                
                if ($workspaceGB -gt 8) {
                    $alertMsg = "DISK ALERT: Workspace size ${workspaceGB}GB (>8GB)"
                    Write-MonitorLog $alertMsg "WARN"
                    Send-WindowsNotification "Disk Usage Alert" $alertMsg "Warning"
                }
                
                if ($processCount -gt 10) {
                    $alertMsg = "PROCESS ALERT: $processCount VS Code processes running"
                    Write-MonitorLog $alertMsg "WARN"
                    Send-WindowsNotification "Process Count Alert" $alertMsg "Warning"
                }
                
                # Estadísticas cada 10 checks
                if ($checkCount % 10 -eq 0) {
                    Write-MonitorLog "Statistics after $checkCount checks - Recent health: $healthScore"
                    
                    # Leer historial de los últimos checks
                    $recentLogs = Get-Content $LogFile | Select-Object -Last 50 | Where-Object { $_ -match "Health check" }
                    $criticalCount = ($recentLogs | Where-Object { $_ -match "critical" }).Count
                    $warningCount = ($recentLogs | Where-Object { $_ -match "warning" }).Count
                    
                    if ($criticalCount -gt 3) {
                        Write-MonitorLog "PATTERN ALERT: $criticalCount critical health events in recent history" "ERROR"
                        Send-WindowsNotification "Health Pattern Alert" "VS Code has been in critical state $criticalCount times recently" "Error"
                    }
                }
            }
            
        } catch {
            Write-MonitorLog "Error during health check: $_" "ERROR"
        }
        
        # Esperar el intervalo especificado
        if ($monitoringActive) {
            Write-Host "⏱️  Next check in $IntervalMinutes minutes... (Check #$($checkCount + 1))" -ForegroundColor DarkGray
            
            # Esperar en chunks de 30 segundos para permitir cancelación rápida
            $remainingSeconds = $IntervalMinutes * 60
            while ($remainingSeconds -gt 0 -and $monitoringActive) {
                Start-Sleep -Seconds ([Math]::Min(30, $remainingSeconds))
                $remainingSeconds -= 30
                
                # Mostrar countdown cada 5 minutos
                if ($remainingSeconds % 300 -eq 0 -and $remainingSeconds -gt 0) {
                    $remainingMinutes = [Math]::Round($remainingSeconds / 60)
                    Write-Host "⏳ $remainingMinutes minutes until next check..." -ForegroundColor DarkGray
                }
            }
        }
    }
    
} catch {
    Write-MonitorLog "Monitor error: $_" "ERROR"
} finally {
    Write-MonitorLog "VS Code Health Monitor stopped (after $checkCount checks)"
    Write-Host ""
    Write-Host "📊 MONITORING SUMMARY" -ForegroundColor Cyan
    Write-Host "=====================" -ForegroundColor Cyan
    Write-Host "Total health checks performed: $checkCount" -ForegroundColor Gray
    Write-Host "Log file: $LogFile" -ForegroundColor Gray
    Write-Host "Monitor stopped gracefully" -ForegroundColor Green
    
    # Mostrar resumen de los últimos eventos críticos
    if (Test-Path $LogFile) {
        $recentCritical = Get-Content $LogFile | Select-Object -Last 100 | Where-Object { $_ -match "\[ERROR\]" }
        if ($recentCritical.Count -gt 0) {
            Write-Host ""
            Write-Host "⚠️  Recent critical events:" -ForegroundColor Yellow
            $recentCritical | Select-Object -Last 5 | ForEach-Object {
                Write-Host "   $_" -ForegroundColor Red
            }
        }
    }
}

Write-Host ""
Write-Host "🏥 Health monitoring session completed" -ForegroundColor Green
