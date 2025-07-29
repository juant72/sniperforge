#!/usr/bin/env pwsh
# SIMULACIÓN DIAGNÓSTICA COMPLETA - IDENTIFICAR MEJORAS PARA ARBITRAJE GANADOR
# Verifica funcionamiento del swap y detecta oportunidades de optimización

param(
    [int]$DurationMinutes = 5,  # Duración de la simulación
    [switch]$VerboseOutput = $true,  # Output detallado
    [switch]$AnalyzeSwaps = $true   # Análisis específico de swaps
)

Write-Host "🔬 SIMULACIÓN DIAGNÓSTICA ARBITRAJE - ANÁLISIS PROFUNDO" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Yellow

# Configuración de la simulación
$ExePath = ".\target\release\arbitrage_phase45_clean.exe"
$LogFile = "diagnostic_simulation_$(Get-Date -Format 'yyyyMMdd_HHmmss').log"
$MetricsFile = "simulation_metrics_$(Get-Date -Format 'yyyyMMdd_HHmmss').json"

Write-Host "`n📋 CONFIGURACIÓN DE SIMULACIÓN:" -ForegroundColor Cyan
Write-Host "   • Duración: $DurationMinutes minutos" -ForegroundColor White
Write-Host "   • Executable: $ExePath" -ForegroundColor White
Write-Host "   • Log File: $LogFile" -ForegroundColor White
Write-Host "   • Análisis de Swaps: $(if($AnalyzeSwaps) { '✅ ACTIVADO' } else { '❌ DESACTIVADO' })" -ForegroundColor White

# Verificar que el executable existe
if (-not (Test-Path $ExePath)) {
    Write-Host "❌ ERROR: No se encuentra $ExePath" -ForegroundColor Red
    Write-Host "   Ejecuta primero: cargo build --release --bin arbitrage_phase45_clean" -ForegroundColor Yellow
    exit 1
}

# Backup de configuración actual
Write-Host "`n💾 CREANDO BACKUP DE CONFIGURACIÓN..." -ForegroundColor Green
Copy-Item "arbitrage_settings.json" "arbitrage_settings_backup_$(Get-Date -Format 'yyyyMMdd_HHmmss').json"

# Verificar configuración de simulación
Write-Host "`n🔧 VERIFICANDO CONFIGURACIÓN DE SIMULACIÓN..." -ForegroundColor Green
$config = Get-Content "arbitrage_settings.json" | ConvertFrom-Json

if ($config.trading.mode -ne "simulation") {
    Write-Host "⚠️ ADVERTENCIA: Configuración no está en modo simulación" -ForegroundColor Yellow
    Write-Host "   Cambiando a modo simulación..." -ForegroundColor Yellow
    $config.trading.mode = "simulation"
    $config.trading.force_real_transactions = $false
    $config | ConvertTo-Json -Depth 10 | Set-Content "arbitrage_settings.json"
}

Write-Host "   ✅ Modo: $($config.trading.mode)" -ForegroundColor Green
Write-Host "   ✅ Transacciones reales: $($config.trading.force_real_transactions)" -ForegroundColor Green
Write-Host "   ✅ Max trade SOL: $($config.trading.max_trade_sol)" -ForegroundColor Green

# Estructuras para métricas
$Metrics = @{
    start_time = Get-Date
    opportunities_found = 0
    swaps_attempted = 0
    swaps_successful = 0
    swaps_failed = 0
    profit_simulated = 0.0
    errors_encountered = @()
    performance_data = @()
    api_calls = @{
        jupiter = 0
        dexscreener = 0
        rpc = 0
    }
    cycle_times = @()
    latencies = @()
}

Write-Host "`n🚀 INICIANDO SIMULACIÓN DIAGNÓSTICA..." -ForegroundColor Green
Write-Host "   Tiempo de inicio: $($Metrics.start_time.ToString('HH:mm:ss'))" -ForegroundColor White

# Función para analizar línea de log
function Analyze-LogLine {
    param($Line)
    
    # Detectar oportunidades encontradas
    if ($Line -match "Found arbitrage opportunity") {
        $script:Metrics.opportunities_found++
    }
    
    # Detectar intentos de swap
    if ($Line -match "Attempting swap|Executing swap") {
        $script:Metrics.swaps_attempted++
    }
    
    # Detectar swaps exitosos
    if ($Line -match "Swap successful|Transaction confirmed") {
        $script:Metrics.swaps_successful++
    }
    
    # Detectar swaps fallidos
    if ($Line -match "Swap failed|Transaction failed|Error executing") {
        $script:Metrics.swaps_failed++
        if ($Line -match "Error: (.+)") {
            $script:Metrics.errors_encountered += $matches[1]
        }
    }
    
    # Detectar profit simulado
    if ($Line -match "Simulated profit: ([\d.]+) SOL") {
        $script:Metrics.profit_simulated += [float]$matches[1]
    }
    
    # Detectar llamadas a APIs
    if ($Line -match "Jupiter API call") { $script:Metrics.api_calls.jupiter++ }
    if ($Line -match "DexScreener API call") { $script:Metrics.api_calls.dexscreener++ }
    if ($Line -match "RPC call") { $script:Metrics.api_calls.rpc++ }
    
    # Detectar tiempos de ciclo
    if ($Line -match "Cycle completed in ([\d.]+)ms") {
        $script:Metrics.cycle_times += [float]$matches[1]
    }
    
    # Detectar latencias
    if ($Line -match "API latency: ([\d.]+)ms") {
        $script:Metrics.latencies += [float]$matches[1]
    }
}

# Iniciar el proceso de arbitraje
$StartTime = Get-Date
$Process = Start-Process -FilePath $ExePath -ArgumentList "--config", "arbitrage_settings.json" -RedirectStandardOutput $LogFile -RedirectStandardError "${LogFile}.err" -PassThru -NoNewWindow

Write-Host "   🔄 Proceso iniciado (PID: $($Process.Id))" -ForegroundColor Green
Write-Host "   📊 Monitoreando por $DurationMinutes minutos..." -ForegroundColor Yellow

# Monitoreo en tiempo real
$MonitoringEnd = $StartTime.AddMinutes($DurationMinutes)
$LastLogSize = 0

while ((Get-Date) -lt $MonitoringEnd -and -not $Process.HasExited) {
    Start-Sleep -Seconds 2
    
    # Leer nuevas líneas del log
    if (Test-Path $LogFile) {
        $LogContent = Get-Content $LogFile -Raw
        if ($LogContent.Length -gt $LastLogSize) {
            $NewContent = $LogContent.Substring($LastLogSize)
            $NewLines = $NewContent -split "`n" | Where-Object { $_.Trim() -ne "" }
            
            foreach ($Line in $NewLines) {
                Analyze-LogLine $Line
                
                if ($VerboseOutput -and $Line -match "(Found|Executing|Error|Profit)") {
                    $timestamp = Get-Date -Format "HH:mm:ss"
                    Write-Host "   [$timestamp] $Line" -ForegroundColor Gray
                }
            }
            
            $LastLogSize = $LogContent.Length
        }
    }
    
    # Update en tiempo real cada 30 segundos
    if (((Get-Date) - $StartTime).TotalSeconds % 30 -lt 2) {
        $elapsed = ((Get-Date) - $StartTime).TotalMinutes
        Write-Host "`n   📊 PROGRESO ($([math]::Round($elapsed, 1))m): Oportunidades: $($Metrics.opportunities_found) | Swaps: $($Metrics.swaps_attempted) | Exitosos: $($Metrics.swaps_successful)" -ForegroundColor Cyan
    }
}

# Terminar el proceso si aún está corriendo
if (-not $Process.HasExited) {
    Write-Host "`n🛑 Terminando simulación..." -ForegroundColor Yellow
    $Process | Stop-Process -Force
    Start-Sleep -Seconds 2
}

Write-Host "`n✅ SIMULACIÓN COMPLETADA" -ForegroundColor Green

# Análisis final de métricas
$Metrics.end_time = Get-Date
$Metrics.duration_minutes = ($Metrics.end_time - $Metrics.start_time).TotalMinutes

Write-Host "`n📊 RESULTADOS DE LA SIMULACIÓN:" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════" -ForegroundColor Yellow

Write-Host "`n🎯 MÉTRICAS GENERALES:" -ForegroundColor Cyan
Write-Host "   • Duración: $([math]::Round($Metrics.duration_minutes, 2)) minutos" -ForegroundColor White
Write-Host "   • Oportunidades encontradas: $($Metrics.opportunities_found)" -ForegroundColor White
Write-Host "   • Tasa de descubrimiento: $([math]::Round($Metrics.opportunities_found / $Metrics.duration_minutes, 1)) oportunidades/min" -ForegroundColor White

Write-Host "`n🔄 ANÁLISIS DE SWAPS:" -ForegroundColor Cyan
$swap_success_rate = if ($Metrics.swaps_attempted -gt 0) { ($Metrics.swaps_successful / $Metrics.swaps_attempted) * 100 } else { 0 }
Write-Host "   • Swaps intentados: $($Metrics.swaps_attempted)" -ForegroundColor White
Write-Host "   • Swaps exitosos: $($Metrics.swaps_successful)" -ForegroundColor $(if($Metrics.swaps_successful -gt 0) { "Green" } else { "Yellow" })
Write-Host "   • Swaps fallidos: $($Metrics.swaps_failed)" -ForegroundColor $(if($Metrics.swaps_failed -eq 0) { "Green" } else { "Red" })
Write-Host "   • Tasa de éxito: $([math]::Round($swap_success_rate, 1))%" -ForegroundColor $(if($swap_success_rate -gt 80) { "Green" } elseif($swap_success_rate -gt 50) { "Yellow" } else { "Red" })

Write-Host "`n💰 ANÁLISIS DE RENTABILIDAD:" -ForegroundColor Cyan
Write-Host "   • Profit simulado total: $([math]::Round($Metrics.profit_simulated, 4)) SOL" -ForegroundColor $(if($Metrics.profit_simulated -gt 0) { "Green" } else { "Red" })
Write-Host "   • Profit promedio/oportunidad: $([math]::Round($Metrics.profit_simulated / [Math]::Max($Metrics.opportunities_found, 1), 6)) SOL" -ForegroundColor White

Write-Host "`n🌐 ACTIVIDAD DE APIs:" -ForegroundColor Cyan
Write-Host "   • Llamadas Jupiter: $($Metrics.api_calls.jupiter)" -ForegroundColor White
Write-Host "   • Llamadas DexScreener: $($Metrics.api_calls.dexscreener)" -ForegroundColor White
Write-Host "   • Llamadas RPC: $($Metrics.api_calls.rpc)" -ForegroundColor White

if ($Metrics.cycle_times.Count -gt 0) {
    $avg_cycle = ($Metrics.cycle_times | Measure-Object -Average).Average
    $max_cycle = ($Metrics.cycle_times | Measure-Object -Maximum).Maximum
    Write-Host "`n⚡ PERFORMANCE:" -ForegroundColor Cyan
    Write-Host "   • Tiempo ciclo promedio: $([math]::Round($avg_cycle, 1))ms" -ForegroundColor White
    Write-Host "   • Tiempo ciclo máximo: $([math]::Round($max_cycle, 1))ms" -ForegroundColor White
}

if ($Metrics.latencies.Count -gt 0) {
    $avg_latency = ($Metrics.latencies | Measure-Object -Average).Average
    Write-Host "   • Latencia API promedio: $([math]::Round($avg_latency, 1))ms" -ForegroundColor White
}

# Análisis de errores
if ($Metrics.errors_encountered.Count -gt 0) {
    Write-Host "`n❌ ERRORES DETECTADOS:" -ForegroundColor Red
    $error_groups = $Metrics.errors_encountered | Group-Object | Sort-Object Count -Descending
    foreach ($error_group in $error_groups) {
        Write-Host "   • $($error_group.Name) (x$($error_group.Count))" -ForegroundColor Yellow
    }
}

# Recomendaciones basadas en resultados
Write-Host "`n🎯 RECOMENDACIONES DE MEJORA:" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════" -ForegroundColor Yellow

if ($Metrics.opportunities_found -eq 0) {
    Write-Host "   🔍 SIN OPORTUNIDADES DETECTADAS:" -ForegroundColor Red
    Write-Host "      • Verificar conectividad con DEXs" -ForegroundColor White
    Write-Host "      • Revisar configuración de APIs" -ForegroundColor White
    Write-Host "      • Considerar ajustar umbrales de profit" -ForegroundColor White
} elseif ($swap_success_rate -lt 50) {
    Write-Host "   ⚠️ BAJA TASA DE ÉXITO EN SWAPS:" -ForegroundColor Yellow
    Write-Host "      • Revisar configuración de slippage" -ForegroundColor White
    Write-Host "      • Verificar liquidez en pools" -ForegroundColor White
    Write-Host "      • Optimizar timeouts de transacciones" -ForegroundColor White
} elseif ($Metrics.profit_simulated -le 0) {
    Write-Host "   💸 PROFIT NEGATIVO O NULO:" -ForegroundColor Red
    Write-Host "      • Reducir umbrales de fees" -ForegroundColor White
    Write-Host "      • Optimizar cálculos de rentabilidad" -ForegroundColor White
    Write-Host "      • Considerar arbitraje triangular" -ForegroundColor White
} else {
    Write-Host "   ✅ SISTEMA FUNCIONANDO CORRECTAMENTE:" -ForegroundColor Green
    Write-Host "      • Oportunidades detectadas: $($Metrics.opportunities_found)" -ForegroundColor White
    Write-Host "      • Swaps funcionales: $([math]::Round($swap_success_rate, 1))% éxito" -ForegroundColor White
    Write-Host "      • Profit positivo: $([math]::Round($Metrics.profit_simulated, 4)) SOL" -ForegroundColor White
    Write-Host "      • Listo para trading real con configuración conservadora" -ForegroundColor Green
}

# Guardar métricas detalladas
$Metrics | ConvertTo-Json -Depth 3 | Set-Content $MetricsFile
Write-Host "`n💾 Métricas guardadas en: $MetricsFile" -ForegroundColor Green
Write-Host "📋 Log completo en: $LogFile" -ForegroundColor Green

Write-Host "`n🚀 PRÓXIMOS PASOS SUGERIDOS:" -ForegroundColor Yellow
if ($Metrics.profit_simulated -gt 0 -and $swap_success_rate -gt 70) {
    Write-Host "   1. ✅ Sistema listo - proceder con trading real conservador" -ForegroundColor Green
    Write-Host "   2. 🔧 Usar configuración real con 0.29 SOL" -ForegroundColor Green
    Write-Host "   3. 📊 Monitorear primeras operaciones reales" -ForegroundColor Green
} else {
    Write-Host "   1. 🔧 Aplicar optimizaciones sugeridas" -ForegroundColor Yellow
    Write-Host "   2. 🔄 Repetir simulación diagnóstica" -ForegroundColor Yellow
    Write-Host "   3. 📈 Iterar hasta obtener resultados positivos" -ForegroundColor Yellow
}

Write-Host "`n✨ SIMULACIÓN DIAGNÓSTICA COMPLETADA" -ForegroundColor Green
