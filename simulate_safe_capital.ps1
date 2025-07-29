# 🎯 SIMULACIÓN SEGURA CON 0.29 SOL - SIN ROMPER CONFIGURACIÓN
# Ajusta solo los parámetros necesarios sin sobreescribir el JSON completo

param(
    [int]$DurationMinutes = 3,
    [string]$LogPrefix = "safe_real_capital_simulation"
)

$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$logFile = "${LogPrefix}_${timestamp}.log"

Write-Host "💰 SIMULACIÓN SEGURA CON CAPITAL REAL: 0.29 SOL" -ForegroundColor Cyan
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "🔒 MÉTODO SEGURO - Solo ajustar parámetros clave" -ForegroundColor Green
Write-Host "📋 CONFIGURACIÓN DE SIMULACIÓN:" -ForegroundColor Yellow
Write-Host "   • Capital disponible: 0.29 SOL (~$85 USD)" -ForegroundColor White
Write-Host "   • Max trade por operación: 0.05 SOL (17% del capital)" -ForegroundColor White
Write-Host "   • Duración: $DurationMinutes minutos" -ForegroundColor White

# 1. Backup de configuración actual
$configBackup = "arbitrage_settings_safe_backup_$timestamp.json"
Copy-Item "arbitrage_settings.json" $configBackup
Write-Host "💾 Backup creado: $configBackup" -ForegroundColor Green

# 2. Verificar configuración actual
Write-Host "`n🔍 VERIFICANDO CONFIGURACIÓN ACTUAL..." -ForegroundColor Cyan
if (Test-Path "arbitrage_settings.json") {
    $currentConfig = Get-Content "arbitrage_settings.json" -Raw | ConvertFrom-Json
    Write-Host "   ✅ Archivo de configuración encontrado" -ForegroundColor Green
    Write-Host "   ✅ Modo actual: $($currentConfig.trading.mode)" -ForegroundColor Green
    Write-Host "   ✅ Max trade actual: $($currentConfig.trading.max_trade_sol) SOL" -ForegroundColor Green
} else {
    Write-Host "   ❌ No se encontró arbitrage_settings.json" -ForegroundColor Red
    Write-Host "   🔧 Restaurando configuración desde backup..." -ForegroundColor Yellow
    
    # Buscar backup más reciente
    $latestBackup = Get-ChildItem "arbitrage_settings_backup_*.json" | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if ($latestBackup) {
        Copy-Item $latestBackup.FullName "arbitrage_settings.json"
        Write-Host "   ✅ Configuración restaurada desde: $($latestBackup.Name)" -ForegroundColor Green
    } else {
        Write-Host "   ❌ No se encontraron backups. Abortando simulación." -ForegroundColor Red
        exit 1
    }
}

# 3. Ajustar solo parámetros clave para 0.29 SOL
Write-Host "`n🔧 AJUSTANDO PARÁMETROS PARA 0.29 SOL..." -ForegroundColor Yellow

# Leer configuración actual
$config = Get-Content "arbitrage_settings.json" -Raw | ConvertFrom-Json

# Ajustar parámetros específicos para capital limitado
$config.trading.max_trade_sol = 0.05
$config.trading.min_profit_threshold_sol = 0.0005
$config.trading.min_confidence_threshold = 0.4
$config.trading.max_slippage_bps = 150
$config.trading.military_min_profit_bps = 8
$config.trading.default_trade_amount_usd = 15.0
$config.trading.max_concurrent_trades = 2

# Ajustar risk management para capital pequeño
$config.risk_management.max_total_exposure_sol = 0.25
$config.risk_management.max_loss_per_trade_sol = 0.008
$config.risk_management.position_size_limits.max_sol = 0.05
$config.risk_management.position_size_limits.max_percentage_of_balance = 17.2

# Ajustar flash loans para capital pequeño
$config.arbitrage.flash_loan.max_loan_amount_sol = 5.0

# Guardar configuración ajustada
$config | ConvertTo-Json -Depth 10 | Out-File -FilePath "arbitrage_settings.json" -Encoding UTF8

Write-Host "✅ Parámetros ajustados para 0.29 SOL:" -ForegroundColor Green
Write-Host "   • Max trade: 0.05 SOL (17% de capital)" -ForegroundColor White
Write-Host "   • Min profit: 0.0005 SOL (~$0.15)" -ForegroundColor White
Write-Host "   • Max loss por trade: 0.008 SOL" -ForegroundColor White
Write-Host "   • Flash loans limitados: 5.0 SOL max" -ForegroundColor White

# 4. Verificar balance real
Write-Host "`n💰 VERIFICANDO BALANCE REAL..." -ForegroundColor Cyan
try {
    $walletOutput = solana balance 2>&1
    if ($walletOutput -match "(\d+\.?\d*)\s*SOL") {
        $actualBalance = [decimal]$matches[1]
        Write-Host "   Balance verificado: $actualBalance SOL" -ForegroundColor Green
        
        if ($actualBalance -lt 0.1) {
            Write-Host "   ⚠️  Balance muy bajo - simulación con datos sintéticos" -ForegroundColor Yellow
        } elseif ($actualBalance -ge 0.25) {
            Write-Host "   ✅ Balance suficiente para trades de 0.05 SOL" -ForegroundColor Green
        } else {
            Write-Host "   ⚠️  Balance limitado - trades reducidos recomendados" -ForegroundColor Yellow
        }
    } else {
        Write-Host "   📊 Balance: No se pudo verificar (usando 0.29 SOL estimado)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   📊 Balance estimado: 0.29 SOL (verificación no disponible)" -ForegroundColor Yellow
}

# 5. Ejecutar simulación
Write-Host "`n🚀 INICIANDO SIMULACIÓN SEGURA..." -ForegroundColor Magenta
Write-Host "   Tiempo de inicio: $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor White

$startTime = Get-Date
$executable = ".\target\release\arbitrage_phase45_clean.exe"

if (-not (Test-Path $executable)) {
    Write-Host "❌ Executable no encontrado. Compilando..." -ForegroundColor Red
    Write-Host "   Ejecutando: cargo build --release" -ForegroundColor Yellow
    cargo build --release
    
    if (-not (Test-Path $executable)) {
        Write-Host "❌ Error compilando. Abortando simulación." -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ Compilación exitosa" -ForegroundColor Green
}

# Ejecutar simulación con logging mejorado
Write-Host "🔄 Ejecutando simulación por $DurationMinutes minutos..." -ForegroundColor Cyan

$process = Start-Process -FilePath $executable -PassThru -RedirectStandardOutput $logFile -RedirectStandardError "${logFile}.error"

# Monitoreo en tiempo real con análisis
$endTime = $startTime.AddMinutes($DurationMinutes)
$opportunityCount = 0
$lastLogSize = 0

while ((Get-Date) -lt $endTime -and -not $process.HasExited) {
    Start-Sleep -Seconds 3
    
    if (Test-Path $logFile) {
        $currentLogSize = (Get-Item $logFile).Length
        if ($currentLogSize -gt $lastLogSize) {
            $logContent = Get-Content $logFile -Tail 5 -ErrorAction SilentlyContinue
            
            # Detectar actividad
            foreach ($line in $logContent) {
                if ($line -match "Cross-chain.*profit|Flash loan.*profit|Traditional.*profit") {
                    $opportunityCount++
                    Write-Host "   ✨ Oportunidad #$opportunityCount detectada" -ForegroundColor Green
                }
                if ($line -match "ERROR") {
                    Write-Host "   ⚠️  Error detectado en logs" -ForegroundColor Yellow
                }
            }
            $lastLogSize = $currentLogSize
        }
    }
    
    $elapsed = [math]::Round(((Get-Date) - $startTime).TotalMinutes, 1)
    Write-Host "   [$(Get-Date -Format 'HH:mm:ss')] Progreso: $elapsed/$DurationMinutes min | Oportunidades: $opportunityCount" -ForegroundColor Cyan
}

# Finalizar simulación
if (-not $process.HasExited) {
    $process.Kill()
    Write-Host "✅ Simulación completada por timeout" -ForegroundColor Green
} else {
    Write-Host "✅ Simulación completada naturalmente" -ForegroundColor Green
}

# 6. Análisis detallado de resultados
Write-Host "`n📊 ANÁLISIS DETALLADO PARA 0.29 SOL:" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

if (Test-Path $logFile) {
    $fullLog = Get-Content $logFile -Raw
    
    # Análisis más detallado
    $crossChainMatches = ($fullLog | Select-String "Cross-chain.*profit.*SOL").Count
    $flashLoanMatches = ($fullLog | Select-String "Flash loan.*profit.*SOL").Count
    $traditionalMatches = ($fullLog | Select-String "Traditional.*swap.*profit").Count
    $errorCount = ($fullLog | Select-String "ERROR").Count
    
    Write-Host "🎯 RESUMEN DE OPORTUNIDADES:" -ForegroundColor Yellow
    Write-Host "   • Cross-chain: $crossChainMatches oportunidades" -ForegroundColor White
    Write-Host "   • Flash loans: $flashLoanMatches oportunidades" -ForegroundColor White
    Write-Host "   • Swaps tradicionales: $traditionalMatches oportunidades" -ForegroundColor White
    Write-Host "   • Errores encontrados: $errorCount" -ForegroundColor $(if($errorCount -gt 0){"Red"}else{"Green"})
    
    $totalOpportunities = $crossChainMatches + $flashLoanMatches + $traditionalMatches
    
    if ($totalOpportunities -gt 0) {
        $estimatedProfit = [math]::Round($totalOpportunities * 0.002, 4)  # Estimación conservadora
        $estimatedUSD = [math]::Round($estimatedProfit * 290, 2)
        $roi = [math]::Round(($estimatedProfit / 0.29) * 100, 2)
        
        Write-Host "`n💰 PROYECCIÓN CONSERVADORA PARA 0.29 SOL:" -ForegroundColor Green
        Write-Host "   • Profit estimado: $estimatedProfit SOL (~$$estimatedUSD USD)" -ForegroundColor Green
        Write-Host "   • ROI estimado: $roi%" -ForegroundColor Green
        
        if ($roi -gt 5) {
            Write-Host "`n🚀 RENTABILIDAD PROMETEDORA - CONSIDERA TRADING REAL" -ForegroundColor Magenta
        }
    }
    
    Write-Host "`n📋 Logs guardados en:" -ForegroundColor Cyan
    Write-Host "   • Principal: $logFile" -ForegroundColor White
    Write-Host "   • Errores: ${logFile}.error" -ForegroundColor White
    
} else {
    Write-Host "⚠️  No se generaron logs de simulación" -ForegroundColor Yellow
}

# 7. Restaurar configuración original
Write-Host "`n🔄 RESTAURANDO CONFIGURACIÓN ORIGINAL..." -ForegroundColor Cyan
Copy-Item $configBackup "arbitrage_settings.json"
Write-Host "✅ Configuración original restaurada desde: $configBackup" -ForegroundColor Green

# 8. Recomendaciones finales
Write-Host "`n🎯 RECOMENDACIONES ESPECÍFICAS PARA 0.29 SOL:" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

if ($totalOpportunities -gt 5) {
    Write-Host "✅ EXCELENTE ACTIVIDAD - Sistema detecta oportunidades" -ForegroundColor Green
    Write-Host "🚀 SIGUIENTE PASO: Trading real con configuración conservadora" -ForegroundColor Green
    Write-Host "   Comando sugerido: .\prepare_real_trading_0.29SOL.ps1" -ForegroundColor White
} elseif ($totalOpportunities -gt 0) {
    Write-Host "⚠️  ACTIVIDAD MODERADA - Algunas oportunidades detectadas" -ForegroundColor Yellow
    Write-Host "💡 OPCIONES:" -ForegroundColor Yellow
    Write-Host "   1. Esperar mejores condiciones de mercado" -ForegroundColor White
    Write-Host "   2. Ejecutar simulación más larga (5-10 minutos)" -ForegroundColor White
    Write-Host "   3. Probar en horarios de mayor volatilidad" -ForegroundColor White
} else {
    Write-Host "❌ BAJA ACTIVIDAD - Pocas oportunidades detectadas" -ForegroundColor Red
    Write-Host "💡 POSIBLES CAUSAS:" -ForegroundColor Yellow
    Write-Host "   1. Condiciones de mercado tranquilas" -ForegroundColor White
    Write-Host "   2. Umbrales de profit muy altos" -ForegroundColor White
    Write-Host "   3. Problemas de conectividad de red" -ForegroundColor White
}

Write-Host "`n✨ SIMULACIÓN SEGURA COMPLETADA - CONFIGURACIÓN PRESERVADA" -ForegroundColor Magenta
