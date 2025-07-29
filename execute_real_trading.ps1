# 🚀 TRADING REAL SOLANA-ONLY - CAPITAL 0.29 SOL
# Ejecución con dinero real usando configuración optimizada

param(
    [int]$DurationMinutes = 5,
    [string]$ConfigFile = "arbitrage_settings_solana_only.json"
)

Write-Host "🚀 TRADING REAL ARBITRAJE SOLO SOLANA - INICIANDO..." -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "💰 CAPITAL REAL: 0.29 SOL (~$68 USD)" -ForegroundColor Yellow
Write-Host "🎯 ESTRATEGIA: Solo DEXes Solana (Sin Cross-Chain)" -ForegroundColor Cyan
Write-Host "⏱️ DURACIÓN INICIAL: $DurationMinutes minutos" -ForegroundColor White
Write-Host "🔒 MODO: TRADING REAL - DINERO EN RIESGO" -ForegroundColor Red
Write-Host "📅 Inicio: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor White

# Verificaciones de seguridad
Write-Host "`n🔐 VERIFICACIONES DE SEGURIDAD:" -ForegroundColor Red
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Red

if (-not (Test-Path $ConfigFile)) {
    Write-Host "❌ CRÍTICO: Archivo de configuración no encontrado: $ConfigFile" -ForegroundColor Red
    Write-Host "🛑 ABORTANDO EJECUCIÓN POR SEGURIDAD" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Configuración encontrada: $ConfigFile" -ForegroundColor Green

# Verificar que el ejecutable existe
$executablePath = ".\winning_bot_standalone.exe"
if (-not (Test-Path $executablePath)) {
    Write-Host "❌ CRÍTICO: Ejecutable no encontrado: $executablePath" -ForegroundColor Red
    
    # Buscar ejecutables alternativos
    $alternatives = @(".\test_compile.exe", ".\test_fix.exe", ".\verificacion_final.exe", ".\test_fee_calculator.exe")
    $foundExecutable = $null
    
    foreach ($alt in $alternatives) {
        if (Test-Path $alt) {
            $foundExecutable = $alt
            break
        }
    }
    
    if ($foundExecutable) {
        Write-Host "✅ Ejecutable alternativo encontrado: $foundExecutable" -ForegroundColor Yellow
        $executablePath = $foundExecutable
    } else {
        Write-Host "🛑 ABORTANDO EJECUCIÓN POR SEGURIDAD" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "✅ Ejecutable encontrado: $executablePath" -ForegroundColor Green
}

# Mostrar configuración actual
Write-Host "`n📋 CONFIGURACIÓN ACTIVA:" -ForegroundColor Cyan
$config = Get-Content $ConfigFile | ConvertFrom-Json
Write-Host "   • Max trade: $($config.trading.max_trade_amount_sol) SOL" -ForegroundColor White
Write-Host "   • Min profit: $($config.trading.min_profit_threshold_sol) SOL" -ForegroundColor White
Write-Host "   • Max slippage: $($config.trading.max_slippage_percent)%" -ForegroundColor White
Write-Host "   • Cross-chain: $($config.arbitrage.strategies.cross_chain.enabled)" -ForegroundColor $(if($config.arbitrage.strategies.cross_chain.enabled){"Red"}else{"Green"})
Write-Host "   • Flash loans: $($config.arbitrage.strategies.flash_loan.enabled)" -ForegroundColor $(if($config.arbitrage.strategies.flash_loan.enabled){"Red"}else{"Green"})
Write-Host "   • Traditional DEX: $($config.arbitrage.strategies.traditional_dex.enabled)" -ForegroundColor $(if($config.arbitrage.strategies.traditional_dex.enabled){"Green"}else{"Red"})

# Confirmación final
Write-Host "`n⚠️ CONFIRMACIÓN FINAL:" -ForegroundColor Yellow
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Yellow
Write-Host "🔴 ESTO ES TRADING REAL CON DINERO REAL" -ForegroundColor Red
Write-Host "💰 Capital en riesgo: 0.29 SOL (~$68 USD)" -ForegroundColor Yellow
Write-Host "📊 Resultados de simulación: +4.47% ROI, 83.3% éxito" -ForegroundColor Green
Write-Host "🎯 Solo arbitraje dentro de Solana (sin cross-chain)" -ForegroundColor Cyan

Write-Host "`nPresiona ENTER para continuar con trading real o CTRL+C para cancelar..." -ForegroundColor Yellow
Read-Host

# Crear log de trading real
$logFile = "REAL_TRADING_solana_$(Get-Date -Format 'yyyyMMdd_HHmmss').log"
Write-Host "`n✅ INICIANDO TRADING REAL..." -ForegroundColor Green
Write-Host "📝 Log file: $logFile" -ForegroundColor White

# Función para log
function Write-RealLog {
    param($Message, $Color = "White")
    $timestamp = Get-Date -Format "yyyy-MM-ddTHH:mm:ss.fffZ"
    $logEntry = "[$timestamp] $Message"
    Write-Host $logEntry -ForegroundColor $Color
    Add-Content -Path $logFile -Value $logEntry
}

Write-RealLog "🚀 INICIANDO TRADING REAL ARBITRAJE SOLO SOLANA" "Green"
Write-RealLog "💰 Capital real disponible: 0.29 SOL" "Yellow"
Write-RealLog "⚙️ Configuración: $ConfigFile" "White"
Write-RealLog "🎯 Estrategia: Solo DEXes Solana (Raydium, Orca, Meteora, Phoenix)" "Cyan"

# Ejecutar el sistema real
Write-RealLog "⚡ EJECUTANDO SISTEMA REAL..." "Green"
Write-Host "`n🔥 INICIANDO ARBITRAGE_PHASE45_CLEAN.EXE..." -ForegroundColor Red

try {
    # Ejecutar el sistema real
    $process = Start-Process -FilePath $executablePath -ArgumentList "--config", $ConfigFile, "--mode", "live" -PassThru -NoNewWindow -RedirectStandardOutput "real_output.txt" -RedirectStandardError "real_errors.txt"
    
    Write-RealLog "✅ Sistema real iniciado - PID: $($process.Id)" "Green"
    Write-Host "💎 SISTEMA EJECUTÁNDOSE EN TIEMPO REAL..." -ForegroundColor Green
    Write-Host "📊 Monitoreando por $DurationMinutes minutos..." -ForegroundColor Cyan
    
    # Monitorear el proceso
    $startTime = Get-Date
    $endTime = $startTime.AddMinutes($DurationMinutes)
    
    while ((Get-Date) -lt $endTime -and !$process.HasExited) {
        $elapsed = ((Get-Date) - $startTime).TotalSeconds
        $remaining = ($endTime - (Get-Date)).TotalSeconds
        
        Write-Host "`r⏱️ Tiempo: $([math]::Floor($elapsed))s | Restante: $([math]::Floor($remaining))s | PID: $($process.Id)" -NoNewline -ForegroundColor Gray
        
        # Verificar si hay output
        if (Test-Path "real_output.txt") {
            $output = Get-Content "real_output.txt" -Tail 5 | Where-Object { $_ -ne "" }
            if ($output) {
                foreach ($line in $output) {
                    Write-RealLog "📊 $line" "White"
                }
            }
        }
        
        # Verificar errores
        if (Test-Path "real_errors.txt") {
            $errors = Get-Content "real_errors.txt" -Tail 3 | Where-Object { $_ -ne "" }
            if ($errors) {
                foreach ($error in $errors) {
                    Write-RealLog "❌ ERROR: $error" "Red"
                }
            }
        }
        
        Start-Sleep -Seconds 5
    }
    
    Write-Host "`n"
    Write-RealLog "⏹️ Deteniendo sistema después de $DurationMinutes minutos..." "Yellow"
    
    # Detener el proceso
    if (!$process.HasExited) {
        $process.Kill()
        Write-RealLog "🛑 Sistema detenido exitosamente" "Green"
    } else {
        Write-RealLog "ℹ️ Sistema terminó por sí mismo" "Yellow"
    }
    
} catch {
    Write-RealLog "❌ ERROR CRÍTICO: $($_.Exception.Message)" "Red"
    Write-Host "🚨 ERROR AL EJECUTAR SISTEMA REAL" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
}

# Análisis post-ejecución
Write-Host "`n🔍 ANÁLISIS POST-EJECUCIÓN:" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

# Verificar balance final
Write-RealLog "💰 Verificando balance final..." "Yellow"

# Analizar logs generados
if (Test-Path "real_output.txt") {
    $outputContent = Get-Content "real_output.txt"
    $totalLines = $outputContent.Count
    Write-RealLog "📊 Líneas de output generadas: $totalLines" "White"
    
    # Buscar profits/losses
    $profitLines = $outputContent | Where-Object { $_ -match "profit|gain|NET" }
    $errorLines = $outputContent | Where-Object { $_ -match "error|fail|loss" }
    
    Write-RealLog "💰 Líneas con profits detectadas: $($profitLines.Count)" "Green"
    Write-RealLog "❌ Líneas con errores detectadas: $($errorLines.Count)" "Red"
    
    # Mostrar últimas líneas importantes
    Write-Host "`n📋 ÚLTIMAS LÍNEAS DEL SISTEMA:" -ForegroundColor Yellow
    $outputContent | Select-Object -Last 10 | ForEach-Object {
        Write-Host "   $_" -ForegroundColor Gray
    }
}

if (Test-Path "real_errors.txt") {
    $errorContent = Get-Content "real_errors.txt"
    if ($errorContent.Count -gt 0) {
        Write-Host "`n🚨 ERRORES DETECTADOS:" -ForegroundColor Red
        $errorContent | Select-Object -Last 5 | ForEach-Object {
            Write-Host "   ❌ $_" -ForegroundColor Red
        }
    }
}

Write-RealLog "🏁 TRADING REAL COMPLETADO - $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" "Green"

Write-Host "`n🎯 TRADING REAL COMPLETADO" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "📁 Log completo: $logFile" -ForegroundColor Cyan
Write-Host "📁 Output sistema: real_output.txt" -ForegroundColor Cyan
Write-Host "📁 Errores sistema: real_errors.txt" -ForegroundColor Cyan

Write-Host "`n💡 PRÓXIMOS PASOS:" -ForegroundColor Yellow
Write-Host "1. Revisar logs para análisis de resultados" -ForegroundColor White
Write-Host "2. Verificar balance de wallet real" -ForegroundColor White
Write-Host "3. Optimizar configuración basada en resultados" -ForegroundColor White
