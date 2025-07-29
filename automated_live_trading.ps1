# 🚀 TRADING REAL AUTOMATIZADO - SOLANA ONLY
# Ejecuta el sistema real con selección automática de Live Trading

param(
    [int]$DurationMinutes = 5,
    [string]$ConfigFile = "arbitrage_settings_solana_only.json"
)

Write-Host "🚀 TRADING REAL AUTOMATIZADO - SOLO SOLANA" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "💰 CAPITAL REAL: 0.29 SOL" -ForegroundColor Yellow
Write-Host "⚡ SELECCIÓN AUTOMÁTICA: Opción 1 (LIVE TRADING)" -ForegroundColor Cyan
Write-Host "📅 Inicio: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor White

# Verificaciones
if (-not (Test-Path $ConfigFile)) {
    Write-Host "❌ Configuración no encontrada: $ConfigFile" -ForegroundColor Red
    exit 1
}

$executablePath = ".\winning_bot_standalone.exe"
if (-not (Test-Path $executablePath)) {
    Write-Host "❌ Ejecutable no encontrado: $executablePath" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Todo listo para trading real automatizado" -ForegroundColor Green

# Crear log
$logFile = "AUTOMATED_TRADING_$(Get-Date -Format 'yyyyMMdd_HHmmss').log"

function Write-TradingLog {
    param($Message, $Color = "White")
    $timestamp = Get-Date -Format "yyyy-MM-ddTHH:mm:ss.fffZ"
    $logEntry = "[$timestamp] $Message"
    Write-Host $logEntry -ForegroundColor $Color
    Add-Content -Path $logFile -Value $logEntry
}

Write-TradingLog "🚀 INICIANDO TRADING REAL AUTOMATIZADO" "Green"
Write-TradingLog "💰 Capital: 0.29 SOL | Configuración: $ConfigFile" "Yellow"

# Confirmación final
Write-Host "`n⚠️ ÚLTIMA CONFIRMACIÓN:" -ForegroundColor Red
Write-Host "🔴 TRADING REAL CON DINERO REAL - 0.29 SOL EN RIESGO" -ForegroundColor Red
Write-Host "✅ Configuración Solana-only validada" -ForegroundColor Green
Write-Host "⚡ Sistema seleccionará automáticamente LIVE TRADING (opción 1)" -ForegroundColor Cyan
Write-Host "`nPresiona ENTER para iniciar trading real o CTRL+C para cancelar..." -ForegroundColor Yellow
Read-Host

Write-TradingLog "✅ Usuario confirmó - Iniciando trading real..." "Green"

try {
    Write-Host "`n🔥 INICIANDO SISTEMA CON AUTO-SELECCIÓN..." -ForegroundColor Red
    
    # Crear un archivo temporal con la selección
    "1" | Out-File -FilePath "auto_input.txt" -Encoding ASCII
    
    # Ejecutar con input automático
    $process = Start-Process -FilePath $executablePath -RedirectStandardInput "auto_input.txt" -RedirectStandardOutput "live_output.txt" -RedirectStandardError "live_errors.txt" -PassThru -NoNewWindow
    
    Write-TradingLog "✅ Sistema iniciado con PID: $($process.Id)" "Green"
    Write-Host "💎 TRADING EN VIVO EJECUTÁNDOSE..." -ForegroundColor Green
    
    # Dar tiempo al sistema para procesar la selección
    Start-Sleep -Seconds 3
    
    # Monitorear por tiempo especificado
    $startTime = Get-Date
    $endTime = $startTime.AddMinutes($DurationMinutes)
    $lastOutputSize = 0
    
    while ((Get-Date) -lt $endTime -and !$process.HasExited) {
        $elapsed = ((Get-Date) - $startTime).TotalSeconds
        $remaining = ($endTime - (Get-Date)).TotalSeconds
        
        Write-Host "`r💎 Trading Real: $([math]::Floor($elapsed))s | Restante: $([math]::Floor($remaining))s | Status: Activo" -NoNewline -ForegroundColor Green
        
        # Verificar output nuevo
        if (Test-Path "live_output.txt") {
            $currentContent = Get-Content "live_output.txt"
            $currentSize = $currentContent.Count
            
            if ($currentSize -gt $lastOutputSize) {
                $newLines = $currentContent[($lastOutputSize)..($currentSize-1)]
                foreach ($line in $newLines) {
                    if ($line -and $line.Trim() -ne "") {
                        Write-TradingLog "📊 $line" "White"
                        
                        # Detectar trades exitosos
                        if ($line -match "profit|gain|SUCCESS|✅") {
                            Write-Host "`n💰 PROFIT DETECTADO: $line" -ForegroundColor Green
                        }
                        
                        # Detectar errores críticos
                        if ($line -match "error|fail|❌|insufficient") {
                            Write-Host "`n🚨 ERROR: $line" -ForegroundColor Red
                        }
                    }
                }
                $lastOutputSize = $currentSize
            }
        }
        
        # Verificar errores
        if (Test-Path "live_errors.txt") {
            $errorContent = Get-Content "live_errors.txt"
            if ($errorContent.Count -gt 0) {
                $errorContent | ForEach-Object {
                    if ($_ -and $_.Trim() -ne "") {
                        Write-TradingLog "❌ ERROR: $_" "Red"
                    }
                }
            }
        }
        
        Start-Sleep -Seconds 2
    }
    
    Write-Host "`n"
    Write-TradingLog "⏹️ Tiempo completado - Deteniendo trading..." "Yellow"
    
    if (!$process.HasExited) {
        $process.Kill()
        Write-TradingLog "🛑 Sistema detenido exitosamente" "Green"
    } else {
        Write-TradingLog "ℹ️ Sistema terminó naturalmente" "Yellow"
    }
    
} catch {
    Write-TradingLog "❌ ERROR CRÍTICO: $($_.Exception.Message)" "Red"
    Write-Host "🚨 ERROR EN TRADING REAL" -ForegroundColor Red
} finally {
    # Limpiar archivo temporal
    if (Test-Path "auto_input.txt") {
        Remove-Item "auto_input.txt" -Force
    }
}

# Análisis final
Write-Host "`n🔍 ANÁLISIS DE RESULTADOS:" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

if (Test-Path "live_output.txt") {
    $finalOutput = Get-Content "live_output.txt"
    $totalLines = $finalOutput.Count
    
    Write-TradingLog "📊 Total líneas generadas: $totalLines" "White"
    
    # Buscar indicadores de trading
    $profitIndicators = $finalOutput | Where-Object { $_ -match "profit|gain|SUCCESS|NET.*\+|ROI" }
    $lossIndicators = $finalOutput | Where-Object { $_ -match "loss|fail|ERROR|NET.*-" }
    $tradeIndicators = $finalOutput | Where-Object { $_ -match "trade|swap|arbitrage|executed" }
    
    Write-TradingLog "💰 Indicadores de profit: $($profitIndicators.Count)" "Green"
    Write-TradingLog "📉 Indicadores de pérdida: $($lossIndicators.Count)" "Red"
    Write-TradingLog "⚡ Trades detectados: $($tradeIndicators.Count)" "Yellow"
    
    Write-Host "`n📋 ÚLTIMAS 10 LÍNEAS DEL SISTEMA:" -ForegroundColor Yellow
    $finalOutput | Select-Object -Last 10 | ForEach-Object {
        if ($_ -and $_.Trim() -ne "") {
            Write-Host "   $_" -ForegroundColor Gray
        }
    }
}

Write-TradingLog "🏁 TRADING REAL COMPLETADO - $(Get-Date -Format 'HH:mm:ss')" "Green"

Write-Host "`n🎯 TRADING REAL FINALIZADO" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "📁 Log principal: $logFile" -ForegroundColor Cyan
Write-Host "📁 Output completo: live_output.txt" -ForegroundColor Cyan
Write-Host "📁 Errores: live_errors.txt" -ForegroundColor Cyan

Write-Host "`n💡 PRÓXIMOS PASOS:" -ForegroundColor Yellow
Write-Host "1. Verificar balance de wallet después del trading" -ForegroundColor White
Write-Host "2. Analizar logs para calcular profit/loss real" -ForegroundColor White
Write-Host "3. Optimizar configuración basada en resultados" -ForegroundColor White
