#!/usr/bin/env pwsh
# MONITOR DE RENDIMIENTO AVANZADO - ARBITRAGE PHASE 4.5
# Monitorea métricas específicas del sistema en tiempo real

Write-Host "🚀 MONITOR AVANZADO - ARBITRAGE PHASE 4.5 CLEAN" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Yellow

# Variables de monitoreo
$monitoringStartTime = Get-Date
$performanceMetrics = @()
$opportunityCount = 0
$flashLoanCount = 0
$crossChainCount = 0
$totalProfit = 0.0
$cycleCount = 0
$errorCount = 0

Write-Host "`n📊 MÉTRICAS CLAVE DETECTADAS:" -ForegroundColor Cyan

# Función para extraer métricas de las líneas de output
function Extract-Metrics {
    param($outputLine)
    
    $timestamp = Get-Date -Format "HH:mm:ss"
    
    # Flash Loan Detections
    if ($outputLine -match "🏦 PHASE 6.*Found (\d+) (?:enterprise )?flash loan") {
        $flCount = [int]$matches[1]
        if ($flCount -gt 0) {
            $global:flashLoanCount += $flCount
            Write-Host "[$timestamp] 🏦 FLASH LOAN: +$flCount oportunidades (Total: $global:flashLoanCount)" -ForegroundColor Yellow
        }
    }
    
    # Flash Loan Profit Detection
    if ($outputLine -match "(\d+\.?\d*) SOL → (\d+\.?\d*) SOL (?:net )?profit") {
        $profit = [decimal]$matches[2]
        $global:totalProfit += $profit
        Write-Host "[$timestamp] 💰 PROFIT SIMULADO: +$profit SOL (Total: $([math]::Round($global:totalProfit, 6)) SOL)" -ForegroundColor Green
    }
    
    # Cross-Chain Detections
    if ($outputLine -match "🌐 PHASE 7.*Found (\d+) cross-chain") {
        $ccCount = [int]$matches[1]
        if ($ccCount -gt $global:crossChainCount) {
            $newOpportunities = $ccCount - $global:crossChainCount
            $global:crossChainCount = $ccCount
            Write-Host "[$timestamp] 🌐 CROSS-CHAIN: +$newOpportunities oportunidades (Total: $global:crossChainCount)" -ForegroundColor Cyan
        }
    }
    
    # Optimal Trade Calculations
    if ($outputLine -match "🎯 OPTIMAL TRADE CALCULATION" -or $outputLine -match "✅ OPTIMAL AMOUNT") {
        $global:opportunityCount++
        Write-Host "[$timestamp] 🎯 CÁLCULO ÓPTIMO: Oportunidad #$global:opportunityCount analizada" -ForegroundColor Blue
    }
    
    # Total Cycles Detection
    if ($outputLine -match "Total Cycles: (\d+)") {
        $newCycleCount = [int]$matches[1]
        if ($newCycleCount -gt $global:cycleCount) {
            $global:cycleCount = $newCycleCount
            Write-Host "[$timestamp] 🔄 CICLO COMPLETADO: #$global:cycleCount" -ForegroundColor White
        }
    }
    
    # Discovery Time Performance
    if ($outputLine -match "Discovery Time: (\d+)ms") {
        $discoveryTime = [int]$matches[1]
        if ($discoveryTime -gt 1000) {
            Write-Host "[$timestamp] ⚠️ LATENCIA ALTA: Discovery Time $discoveryTime ms" -ForegroundColor Red
        } elseif ($discoveryTime -lt 500) {
            Write-Host "[$timestamp] ⚡ EXCELENTE LATENCIA: Discovery Time $discoveryTime ms" -ForegroundColor Green
        }
    }
    
    # ML Simulations
    if ($outputLine -match "ML Simulations: (\d+)") {
        $mlSims = [int]$matches[1]
        Write-Host "[$timestamp] 🧠 ML SIMULACIONES: $mlSims ejecutadas" -ForegroundColor Magenta
    }
    
    # Error Detection
    if ($outputLine -match "ERROR" -or $outputLine -match "❌" -or $outputLine -match "WARN.*❌") {
        $global:errorCount++
        Write-Host "[$timestamp] ⚠️ ERROR/WARNING: #$global:errorCount detectado" -ForegroundColor Red
    }
    
    # Performance Score Detection
    if ($outputLine -match "Performance Score: (\d+\.?\d*) ops/sec") {
        $perfScore = [decimal]$matches[1]
        if ($perfScore -gt 5) {
            Write-Host "[$timestamp] 📈 EXCELENTE PERFORMANCE: $perfScore ops/sec" -ForegroundColor Green
        } elseif ($perfScore -lt 2) {
            Write-Host "[$timestamp] 📉 BAJA PERFORMANCE: $perfScore ops/sec" -ForegroundColor Yellow
        }
    }
}

Write-Host "`n⏰ Iniciando monitoreo avanzado (Ctrl+C para salir)..." -ForegroundColor Yellow

try {
    # Monitoreo durante 3 minutos
    $endTime = (Get-Date).AddMinutes(3)
    
    while ((Get-Date) -lt $endTime) {
        # Verificar si el proceso sigue corriendo
        $arbitrageProcess = Get-Process -Name "arbitrage_phase45_clean" -ErrorAction SilentlyContinue
        
        if (-not $arbitrageProcess) {
            Write-Host "`n⚠️ Proceso terminado - finalizando monitoreo" -ForegroundColor Red
            break
        }
        
        Start-Sleep -Seconds 2
        
        # Mostrar resumen cada 30 segundos
        $elapsed = (Get-Date) - $monitoringStartTime
        if ($elapsed.TotalSeconds % 30 -eq 0 -and $elapsed.TotalSeconds -gt 0) {
            Write-Host "`n📊 RESUMEN INTERMEDIO (T+$([math]::Round($elapsed.TotalMinutes, 1))min):" -ForegroundColor Green
            Write-Host "   🔄 Ciclos: $cycleCount | 🎯 Oportunidades: $opportunityCount | 🏦 Flash Loans: $flashLoanCount" -ForegroundColor White
            Write-Host "   🌐 Cross-Chain: $crossChainCount | 💰 Profit Total: $([math]::Round($totalProfit, 6)) SOL | ⚠️ Errores: $errorCount" -ForegroundColor White
            
            if ($arbitrageProcess) {
                $memoryMB = [math]::Round(($arbitrageProcess.WorkingSet64 / 1MB), 2)
                Write-Host "   💾 Memoria: ${memoryMB}MB | 🆔 PID: $($arbitrageProcess.Id)" -ForegroundColor Gray
            }
            Write-Host ""
        }
    }
    
} catch {
    Write-Host "`n⚠️ Monitoreo interrumpido por el usuario" -ForegroundColor Yellow
}

# Resumen final
$finalDuration = (Get-Date) - $monitoringStartTime

Write-Host "`n" + "="*80 -ForegroundColor Yellow
Write-Host "📊 RESUMEN FINAL DEL MONITOREO" -ForegroundColor Green
Write-Host "="*80 -ForegroundColor Yellow

Write-Host "⏱️ DURACIÓN: $([math]::Round($finalDuration.TotalMinutes, 2)) minutos" -ForegroundColor White

Write-Host "`n🎯 OPORTUNIDADES DETECTADAS:" -ForegroundColor Yellow
Write-Host "   • Cálculos óptimos: $opportunityCount" -ForegroundColor White
Write-Host "   • Flash Loans: $flashLoanCount" -ForegroundColor White
Write-Host "   • Cross-Chain: $crossChainCount" -ForegroundColor White

Write-Host "`n📈 RENDIMIENTO:" -ForegroundColor Yellow
Write-Host "   • Ciclos completados: $cycleCount" -ForegroundColor White
Write-Host "   • Ciclos por minuto: $([math]::Round($cycleCount / [math]::Max($finalDuration.TotalMinutes, 0.1), 2))" -ForegroundColor White
Write-Host "   • Oportunidades por minuto: $([math]::Round($opportunityCount / [math]::Max($finalDuration.TotalMinutes, 0.1), 2))" -ForegroundColor White

Write-Host "`n💰 RESULTADOS FINANCIEROS (SIMULACIÓN):" -ForegroundColor Yellow
Write-Host "   • Profit total simulado: $([math]::Round($totalProfit, 6)) SOL" -ForegroundColor Green
Write-Host "   • Profit promedio por trade: $([math]::Round($totalProfit / [math]::Max($flashLoanCount + $crossChainCount, 1), 6)) SOL" -ForegroundColor Green
Write-Host "   • Valor USD estimado: `$$([math]::Round($totalProfit * 185, 2))" -ForegroundColor Green

Write-Host "`n⚠️ CALIDAD DEL SISTEMA:" -ForegroundColor Yellow
Write-Host "   • Errores detectados: $errorCount" -ForegroundColor $(if ($errorCount -eq 0) { "Green" } else { "Red" })
Write-Host "   • Tasa de éxito: $([math]::Round((($cycleCount - $errorCount) / [math]::Max($cycleCount, 1)) * 100, 2))%" -ForegroundColor $(if ($errorCount -eq 0) { "Green" } else { "Yellow" })

Write-Host "`n💡 EVALUACIÓN DEL RENDIMIENTO:" -ForegroundColor Yellow

# Evaluaciones específicas
if ($cycleCount -gt 0) {
    $cyclesPerMinute = $cycleCount / [math]::Max($finalDuration.TotalMinutes, 0.1)
    if ($cyclesPerMinute -gt 15) {
        Write-Host "   ✅ EXCELENTE: Velocidad de ciclos superior ($([math]::Round($cyclesPerMinute, 1))/min)" -ForegroundColor Green
    } elseif ($cyclesPerMinute -gt 8) {
        Write-Host "   ✅ BUENO: Velocidad de ciclos adecuada ($([math]::Round($cyclesPerMinute, 1))/min)" -ForegroundColor Green
    } else {
        Write-Host "   ⚠️ MEJORABLE: Velocidad de ciclos baja ($([math]::Round($cyclesPerMinute, 1))/min)" -ForegroundColor Yellow
    }
}

if ($totalProfit -gt 5) {
    Write-Host "   ✅ EXCELENTE: Generación de profit simulado muy alta" -ForegroundColor Green
} elseif ($totalProfit -gt 1) {
    Write-Host "   ✅ BUENO: Generación de profit simulado sólida" -ForegroundColor Green
} else {
    Write-Host "   📈 NORMAL: Sistema en fase de aprendizaje" -ForegroundColor Cyan
}

if ($errorCount -eq 0) {
    Write-Host "   ✅ PERFECTO: Cero errores detectados" -ForegroundColor Green
} elseif ($errorCount -lt 3) {
    Write-Host "   ✅ BUENO: Errores mínimos ($errorCount)" -ForegroundColor Green
} else {
    Write-Host "   ⚠️ REVISAR: Varios errores detectados ($errorCount)" -ForegroundColor Yellow
}

Write-Host "`n🔧 RECOMENDACIONES DE OPTIMIZACIÓN:" -ForegroundColor Yellow

if ($opportunityCount / [math]::Max($finalDuration.TotalMinutes, 0.1) -lt 5) {
    Write-Host "   • Considerar reducir filtros de rentabilidad para más oportunidades" -ForegroundColor Cyan
}

if ($errorCount -gt 2) {
    Write-Host "   • Revisar logs para identificar patrones de error" -ForegroundColor Orange
}

if ($flashLoanCount -lt $crossChainCount) {
    Write-Host "   • Flash loans detectando menos oportunidades - revisar configuración" -ForegroundColor Cyan
}

if ($totalProfit -gt 0) {
    Write-Host "   • Sistema funcionando bien - mantener configuración actual" -ForegroundColor Green
}

Write-Host "`n🎯 ESTADO FINAL: " -NoNewline -ForegroundColor Yellow
if ($errorCount -eq 0 -and $totalProfit -gt 1 -and $cycleCount -gt 5) {
    Write-Host "SISTEMA ÓPTIMO 🚀" -ForegroundColor Green
} elseif ($errorCount -lt 3 -and $totalProfit -gt 0) {
    Write-Host "SISTEMA FUNCIONAL ✅" -ForegroundColor Green
} else {
    Write-Host "SISTEMA EN OPTIMIZACIÓN 🔧" -ForegroundColor Yellow
}

Write-Host "`n🎉 Monitor completado exitosamente!" -ForegroundColor Green
