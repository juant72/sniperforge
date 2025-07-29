#!/usr/bin/env pwsh
# MONITOR DE RENDIMIENTO EN TIEMPO REAL - SISTEMA ARBITRAJE PHASE 4.5
# Monitorea el sistema en ejecución y proporciona análisis de rendimiento

Write-Host "🚀 MONITOR DE RENDIMIENTO - ARBITRAGE PHASE 4.5 CLEAN" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Yellow

# Crear directorio de logs si no existe
$logDir = "monitoring_logs"
if (-not (Test-Path $logDir)) {
    New-Item -ItemType Directory -Path $logDir | Out-Null
}

$timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
$performanceLog = "$logDir\performance_$timestamp.log"
$metricsLog = "$logDir\metrics_$timestamp.log"

Write-Host "📊 Iniciando monitoreo de rendimiento..." -ForegroundColor Cyan
Write-Host "📋 Performance Log: $performanceLog" -ForegroundColor Gray
Write-Host "📈 Metrics Log: $metricsLog" -ForegroundColor Gray

# Contadores de rendimiento
$cycleCount = 0
$errorCount = 0
$opportunityCount = 0
$simulationCount = 0

# Análisis de logs en tiempo real
Write-Host "`n🔍 ANÁLISIS DE LOGS EN TIEMPO REAL:" -ForegroundColor Yellow

# Función para analizar línea de log
function Analyze-LogLine {
    param($line)
    
    $timestamp = Get-Date -Format "HH:mm:ss"
    
    # Detectar oportunidades
    if ($line -match "🎯 OPTIMAL TRADE CALCULATION" -or $line -match "✅ OPTIMAL AMOUNT") {
        $global:opportunityCount++
        Write-Host "[$timestamp] 💰 OPORTUNIDAD DETECTADA (#$global:opportunityCount)" -ForegroundColor Green
        $line | Out-File -Append $global:metricsLog
    }
    
    # Detectar simulaciones ML
    if ($line -match "ML Simulations:" -and $line -match "(\d+)") {
        $simCount = [regex]::Match($line, "ML Simulations: (\d+)").Groups[1].Value
        if ($simCount -ne $global:simulationCount) {
            $global:simulationCount = $simCount
            Write-Host "[$timestamp] 🧠 SIMULACIÓN ML COMPLETADA (#$simCount)" -ForegroundColor Blue
        }
    }
    
    # Detectar errores críticos
    if ($line -match "ERROR" -or $line -match "❌") {
        $global:errorCount++
        Write-Host "[$timestamp] ⚠️ ERROR DETECTADO (#$global:errorCount)" -ForegroundColor Red
        $line | Out-File -Append $global:performanceLog
    }
    
    # Detectar timeouts de performance
    if ($line -match "Timeout en discovery" -or $line -match "exceeds.*target") {
        Write-Host "[$timestamp] ⏱️ TIMEOUT DE PERFORMANCE DETECTADO" -ForegroundColor Yellow
        $line | Out-File -Append $global:performanceLog
    }
    
    # Detectar optimizaciones automáticas
    if ($line -match "Reduciendo concurrencia" -or $line -match "Performance score") {
        Write-Host "[$timestamp] 🔧 OPTIMIZACIÓN AUTOMÁTICA APLICADA" -ForegroundColor Magenta
        $line | Out-File -Append $global:performanceLog
    }
    
    # Detectar ciclos completados
    if ($line -match "Total Cycles: (\d+)") {
        $newCycleCount = [regex]::Match($line, "Total Cycles: (\d+)").Groups[1].Value
        if ($newCycleCount -ne $global:cycleCount) {
            $global:cycleCount = $newCycleCount
            Write-Host "[$timestamp] 🔄 CICLO #$newCycleCount COMPLETADO" -ForegroundColor Cyan
        }
    }
}

# Monitoreo de proceso activo
$startTime = Get-Date

Write-Host "`n⏰ Iniciando monitoreo activo (Ctrl+C para salir)..." -ForegroundColor Yellow

$monitorDuration = 120 # 2 minutos de monitoreo
$endTime = $startTime.AddSeconds($monitorDuration)

try {
    while ((Get-Date) -lt $endTime) {
        # Verificar si el proceso sigue corriendo
        $arbitrageProcess = Get-Process -Name "arbitrage_phase45_clean" -ErrorAction SilentlyContinue
        
        if ($arbitrageProcess) {
            # Obtener métricas del sistema
            $cpuUsage = (Get-Counter "\Process(arbitrage_phase45_clean)\% Processor Time" -ErrorAction SilentlyContinue).CounterSamples.CookedValue
            $memoryUsage = [math]::Round(($arbitrageProcess.WorkingSet64 / 1MB), 2)
            
            $timestamp = Get-Date -Format "HH:mm:ss"
            Write-Host "[$timestamp] 📊 CPU: $([math]::Round($cpuUsage, 2))% | RAM: ${memoryUsage}MB | Ciclos: $cycleCount | Oportunidades: $opportunityCount | Errores: $errorCount" -ForegroundColor White
            
            # Log métricas
            "$timestamp,CPU,$cpuUsage,RAM,$memoryUsage,Cycles,$cycleCount,Opportunities,$opportunityCount,Errors,$errorCount" | Out-File -Append $metricsLog
        } else {
            Write-Host "⚠️ Proceso arbitrage_phase45_clean no encontrado - puede haber terminado" -ForegroundColor Red
            break
        }
        
        Start-Sleep -Seconds 5
    }
} catch {
    Write-Host "`n⚠️ Monitoreo interrumpido por el usuario" -ForegroundColor Yellow
}

# Resumen final
$endTime = Get-Date
$duration = $endTime - $startTime

Write-Host "`n📊 RESUMEN DE MONITOREO:" -ForegroundColor Green
Write-Host "═══════════════════════════════════════" -ForegroundColor Yellow
Write-Host "⏱️ Duración: $([math]::Round($duration.TotalMinutes, 2)) minutos" -ForegroundColor White
Write-Host "🔄 Ciclos monitoreados: $cycleCount" -ForegroundColor White
Write-Host "💰 Oportunidades detectadas: $opportunityCount" -ForegroundColor White
Write-Host "🧠 Simulaciones ML: $simulationCount" -ForegroundColor White
Write-Host "⚠️ Errores detectados: $errorCount" -ForegroundColor White

# Análisis de performance
$cyclesPerMinute = if ($duration.TotalMinutes -gt 0) { [math]::Round($cycleCount / $duration.TotalMinutes, 2) } else { 0 }
$opportunitiesPerMinute = if ($duration.TotalMinutes -gt 0) { [math]::Round($opportunityCount / $duration.TotalMinutes, 2) } else { 0 }

Write-Host "`n🚀 MÉTRICAS DE RENDIMIENTO:" -ForegroundColor Green
Write-Host "📈 Ciclos por minuto: $cyclesPerMinute" -ForegroundColor Cyan
Write-Host "💎 Oportunidades por minuto: $opportunitiesPerMinute" -ForegroundColor Cyan
Write-Host "✅ Tasa de éxito: $([math]::Round((($cycleCount - $errorCount) / [math]::Max($cycleCount, 1)) * 100, 2))%" -ForegroundColor Cyan

Write-Host "`n💡 RECOMENDACIONES DE MEJORA:" -ForegroundColor Yellow

if ($opportunitiesPerMinute -lt 2) {
    Write-Host "• 🔍 Oportunidades detectadas bajas - considerar reducir filtros de rentabilidad" -ForegroundColor Orange
}

if ($errorCount -gt 0) {
    Write-Host "• ⚠️ $errorCount errores detectados - revisar logs para optimización" -ForegroundColor Red
}

if ($cyclesPerMinute -lt 10) {
    Write-Host "• ⚡ Velocidad de ciclos baja - considerar optimizar timeouts de red" -ForegroundColor Orange
}

if ($cyclesPerMinute -gt 15) {
    Write-Host "• ✅ Excelente velocidad de ciclos - sistema optimizado" -ForegroundColor Green
}

Write-Host "`n📁 Logs guardados en:" -ForegroundColor Gray
Write-Host "   📊 Performance: $performanceLog" -ForegroundColor Gray
Write-Host "   📈 Metrics: $metricsLog" -ForegroundColor Gray

Write-Host "`n🎯 Monitor completado exitosamente!" -ForegroundColor Green
