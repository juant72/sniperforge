# 🎖️ ANÁLISIS MILITAR COMPLETO - PREPARACIÓN PARA TRADING REAL
# Evaluación exhaustiva del sistema antes de activar trading con 0.29 SOL

param(
    [string]$LogFile = "quick_sim_0.29SOL_20250729_154833.log",
    [decimal]$CapitalSOL = 0.29
)

Write-Host "🎖️ ANÁLISIS DE GRADO MILITAR - PREPARACIÓN TRADING REAL" -ForegroundColor Red
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Red
Write-Host "🔍 EVALUACIÓN EXHAUSTIVA PARA CAPITAL: $CapitalSOL SOL" -ForegroundColor Yellow
Write-Host "📅 Fecha análisis: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor White

# ═══════════════════════════════════════════════════════════════════════
# 🎯 SECCIÓN 1: ANÁLISIS DE INTELIGENCIA OPERACIONAL
# ═══════════════════════════════════════════════════════════════════════

Write-Host "`n🎯 SECCIÓN 1: INTELIGENCIA OPERACIONAL" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

# 1.1 Verificación de assets críticos
Write-Host "`n📋 1.1 VERIFICACIÓN DE ASSETS CRÍTICOS:" -ForegroundColor Yellow

$executablePath = ".\target\release\arbitrage_phase45_clean.exe"
$configPath = "arbitrage_settings.json"
$walletPath = "keypair.json"

$assetsStatus = @()
$assetsStatus += [PSCustomObject]@{
    Asset = "Executable Principal"
    Path = $executablePath
    Status = if (Test-Path $executablePath) { "✅ OPERACIONAL" } else { "❌ FALTANTE" }
    Critical = $true
}

$assetsStatus += [PSCustomObject]@{
    Asset = "Configuración JSON"
    Path = $configPath
    Status = if (Test-Path $configPath) { "✅ OPERACIONAL" } else { "❌ FALTANTE" }
    Critical = $true
}

$assetsStatus += [PSCustomObject]@{
    Asset = "Wallet/Keypair"
    Path = $walletPath
    Status = if (Test-Path $walletPath) { "✅ OPERACIONAL" } else { "⚠️ NO VERIFICADO" }
    Critical = $true
}

$assetsStatus += [PSCustomObject]@{
    Asset = "Log de Simulación"
    Path = $LogFile
    Status = if (Test-Path $LogFile) { "✅ DISPONIBLE" } else { "❌ FALTANTE" }
    Critical = $true
}

foreach ($asset in $assetsStatus) {
    $color = if ($asset.Status.StartsWith("✅")) { "Green" } elseif ($asset.Status.StartsWith("⚠️")) { "Yellow" } else { "Red" }
    Write-Host "   • $($asset.Asset): $($asset.Status)" -ForegroundColor $color
}

# Determinar estado general de assets
$criticalAssetsFailed = ($assetsStatus | Where-Object { $_.Critical -and $_.Status.StartsWith("❌") }).Count
$assetReadiness = if ($criticalAssetsFailed -eq 0) { "✅ LISTO" } else { "❌ NO LISTO" }
Write-Host "`n📊 ESTADO ASSETS CRÍTICOS: $assetReadiness" -ForegroundColor $(if($assetReadiness.StartsWith("✅")){"Green"}else{"Red"})

# 1.2 Análisis de configuración tactical
Write-Host "`n⚙️ 1.2 ANÁLISIS CONFIGURACIÓN TACTICAL:" -ForegroundColor Yellow

if (Test-Path $configPath) {
    try {
        $config = Get-Content $configPath -Raw | ConvertFrom-Json
        
        Write-Host "   💰 PARÁMETROS TRADING:" -ForegroundColor White
        Write-Host "      • Modo: $($config.trading.mode)" -ForegroundColor $(if($config.trading.mode -eq "simulation"){"Green"}else{"Red"})
        Write-Host "      • Max trade: $($config.trading.max_trade_sol) SOL ($(([math]::Round(($config.trading.max_trade_sol / $CapitalSOL) * 100, 1)))% del capital)" -ForegroundColor White
        Write-Host "      • Min profit: $($config.trading.min_profit_threshold_sol) SOL" -ForegroundColor White
        Write-Host "      • Max slippage: $($config.trading.max_slippage_bps) bps" -ForegroundColor White
        
        Write-Host "`n   🛡️ RISK MANAGEMENT:" -ForegroundColor White
        Write-Host "      • Max exposure: $($config.risk_management.max_total_exposure_sol) SOL" -ForegroundColor White
        Write-Host "      • Max loss/trade: $($config.risk_management.max_loss_per_trade_sol) SOL" -ForegroundColor White
        Write-Host "      • Circuit breaker: $($config.risk_management.circuit_breaker_loss_threshold_sol) SOL" -ForegroundColor White
        
        # Evaluación de configuración
        $configScore = 0
        if ($config.trading.mode -eq "simulation") { $configScore += 25 }
        if ($config.trading.max_trade_sol -le ($CapitalSOL * 0.2)) { $configScore += 25 }
        if ($config.risk_management.enabled) { $configScore += 25 }
        if ($config.trading.max_slippage_bps -le 200) { $configScore += 25 }
        
        $configReadiness = if ($configScore -ge 75) { "✅ EXCELENTE" } elseif ($configScore -ge 50) { "⚠️ ACEPTABLE" } else { "❌ RIESGOSO" }
        Write-Host "`n📊 SCORE CONFIGURACIÓN: $configScore/100 - $configReadiness" -ForegroundColor $(if($configScore -ge 75){"Green"}elseif($configScore -ge 50){"Yellow"}else{"Red"})
        
    } catch {
        Write-Host "   ❌ Error parseando configuración: $($_.Exception.Message)" -ForegroundColor Red
        $configReadiness = "❌ ERROR"
    }
} else {
    Write-Host "   ❌ Archivo de configuración no encontrado" -ForegroundColor Red
    $configReadiness = "❌ FALTANTE"
}

# ═══════════════════════════════════════════════════════════════════════
# 🎯 SECCIÓN 2: ANÁLISIS DE PERFORMANCE EN COMBATE
# ═══════════════════════════════════════════════════════════════════════

Write-Host "`n🎯 SECCIÓN 2: ANÁLISIS DE PERFORMANCE EN COMBATE" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

# 2.1 Análisis detallado del log de combate
Write-Host "`n⚔️ 2.1 ANÁLISIS DETALLADO LOG DE COMBATE:" -ForegroundColor Yellow

if (Test-Path $LogFile) {
    $logContent = Get-Content $LogFile -Raw
    $logSize = [math]::Round((Get-Item $LogFile).Length / 1KB, 2)
    
    Write-Host "   📏 Tamaño log: $logSize KB" -ForegroundColor White
    
    # Análisis Cross-Chain (Operaciones exitosas)
    $crossChainRegex = "Cross-Chain:.*\$(\d+\.?\d*)\s+net profit.*(\d+\.?\d*)%"
    $crossChainMatches = [regex]::Matches($logContent, $crossChainRegex)
    
    $crossChainData = @()
    $totalCrossChainProfitUSD = 0
    
    foreach ($match in $crossChainMatches) {
        $profitUSD = [decimal]$match.Groups[1].Value
        $percentage = [decimal]$match.Groups[2].Value
        $profitSOL = [math]::Round($profitUSD / 290, 4)
        
        $crossChainData += [PSCustomObject]@{
            ProfitUSD = $profitUSD
            ProfitSOL = $profitSOL
            Percentage = $percentage
        }
        $totalCrossChainProfitUSD += $profitUSD
    }
    
    # Análisis Flash Loans
    $flashLoanRegex = "Flash Loan.*(\d+\.?\d+)\s*SOL.*(\d+\.?\d+)\s*SOL profit"
    $flashLoanMatches = [regex]::Matches($logContent, $flashLoanRegex)
    
    $flashLoanData = @()
    $totalFlashLoanProfit = 0
    
    foreach ($match in $flashLoanMatches) {
        $loanAmount = [decimal]$match.Groups[1].Value
        $profitSOL = [decimal]$match.Groups[2].Value
        
        $flashLoanData += [PSCustomObject]@{
            LoanAmount = $loanAmount
            ProfitSOL = $profitSOL
        }
        $totalFlashLoanProfit += $profitSOL
    }
    
    # Análisis de operaciones fallidas
    $failedRegex = "NET PROFIT:\s*-(\d+\.?\d+)\s*SOL"
    $failedMatches = [regex]::Matches($logContent, $failedRegex)
    
    $totalLosses = 0
    foreach ($match in $failedMatches) {
        $lossSOL = [decimal]$match.Groups[1].Value
        $totalLosses += $lossSOL
    }
    
    # Estadísticas de combate
    Write-Host "`n   🏆 ESTADÍSTICAS DE COMBATE:" -ForegroundColor Green
    Write-Host "      • Operaciones Cross-Chain exitosas: $($crossChainMatches.Count)" -ForegroundColor White
    Write-Host "      • Operaciones Flash Loan exitosas: $($flashLoanMatches.Count)" -ForegroundColor White
    Write-Host "      • Operaciones fallidas: $($failedMatches.Count)" -ForegroundColor White
    
    $totalSuccessfulOps = $crossChainMatches.Count + $flashLoanMatches.Count
    $totalOps = $totalSuccessfulOps + $failedMatches.Count
    $successRate = if ($totalOps -gt 0) { [math]::Round(($totalSuccessfulOps / $totalOps) * 100, 2) } else { 0 }
    
    Write-Host "      • Tasa de éxito: $successRate%" -ForegroundColor $(if($successRate -gt 70){"Green"}elseif($successRate -gt 50){"Yellow"}else{"Red"})
    
    # Análisis financiero militar
    $totalProfitSOL = [math]::Round(($totalCrossChainProfitUSD / 290) + $totalFlashLoanProfit - $totalLosses, 4)
    $roi = if ($CapitalSOL -gt 0) { [math]::Round(($totalProfitSOL / $CapitalSOL) * 100, 2) } else { 0 }
    
    Write-Host "`n   💰 ANÁLISIS FINANCIERO:" -ForegroundColor Green
    Write-Host "      • Profit Cross-Chain: $([math]::Round($totalCrossChainProfitUSD / 290, 4)) SOL ($$totalCrossChainProfitUSD USD)" -ForegroundColor White
    Write-Host "      • Profit Flash Loans: $totalFlashLoanProfit SOL" -ForegroundColor White
    Write-Host "      • Pérdidas totales: -$totalLosses SOL" -ForegroundColor Red
    Write-Host "      • PROFIT NETO: $totalProfitSOL SOL" -ForegroundColor $(if($totalProfitSOL -gt 0){"Green"}else{"Red"})
    Write-Host "      • ROI (2 minutos): $roi%" -ForegroundColor $(if($roi -gt 5){"Green"}elseif($roi -gt 0){"Yellow"}else{"Red"})
    
    # Score de performance
    $performanceScore = 0
    if ($successRate -gt 70) { $performanceScore += 30 }
    elseif ($successRate -gt 50) { $performanceScore += 20 }
    elseif ($successRate -gt 30) { $performanceScore += 10 }
    
    if ($roi -gt 10) { $performanceScore += 30 }
    elseif ($roi -gt 5) { $performanceScore += 20 }
    elseif ($roi -gt 0) { $performanceScore += 10 }
    
    if ($totalSuccessfulOps -gt 50) { $performanceScore += 25 }
    elseif ($totalSuccessfulOps -gt 30) { $performanceScore += 20 }
    elseif ($totalSuccessfulOps -gt 10) { $performanceScore += 15 }
    
    if ($totalLosses / [math]::Max($CapitalSOL, 1) -lt 0.1) { $performanceScore += 15 }
    elseif ($totalLosses / [math]::Max($CapitalSOL, 1) -lt 0.2) { $performanceScore += 10 }
    
    $performanceReadiness = if ($performanceScore -ge 80) { "✅ ELITE" } elseif ($performanceScore -ge 60) { "✅ LISTO" } elseif ($performanceScore -ge 40) { "⚠️ ACEPTABLE" } else { "❌ NO LISTO" }
    Write-Host "`n📊 SCORE PERFORMANCE: $performanceScore/100 - $performanceReadiness" -ForegroundColor $(if($performanceScore -ge 60){"Green"}elseif($performanceScore -ge 40){"Yellow"}else{"Red"})
    
} else {
    Write-Host "   ❌ Log de simulación no encontrado - ANÁLISIS IMPOSIBLE" -ForegroundColor Red
    $performanceReadiness = "❌ SIN DATOS"
    $performanceScore = 0
}

# ═══════════════════════════════════════════════════════════════════════
# 🎯 SECCIÓN 3: EVALUACIÓN DE RIESGOS OPERACIONALES
# ═══════════════════════════════════════════════════════════════════════

Write-Host "`n🎯 SECCIÓN 3: EVALUACIÓN DE RIESGOS OPERACIONALES" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

# 3.1 Análisis de riesgos financieros
Write-Host "`n⚠️ 3.1 ANÁLISIS DE RIESGOS FINANCIEROS:" -ForegroundColor Yellow

$risks = @()

# Riesgo de capital
$maxTradeRisk = if (Test-Path $configPath) {
    $cfg = Get-Content $configPath -Raw | ConvertFrom-Json
    $cfg.trading.max_trade_sol / $CapitalSOL * 100
} else { 100 }

$risks += [PSCustomObject]@{
    Categoria = "Capital"
    Riesgo = "Max trade por operación"
    Nivel = if ($maxTradeRisk -le 20) { "✅ BAJO" } elseif ($maxTradeRisk -le 30) { "⚠️ MEDIO" } else { "❌ ALTO" }
    Valor = "$([math]::Round($maxTradeRisk, 1))% del capital"
    Recomendacion = if ($maxTradeRisk -gt 20) { "Reducir max_trade_sol" } else { "Aceptable" }
}

# Riesgo de liquidez
$risks += [PSCustomObject]@{
    Categoria = "Liquidez"
    Riesgo = "Capital total disponible"
    Nivel = if ($CapitalSOL -ge 0.5) { "✅ BAJO" } elseif ($CapitalSOL -ge 0.25) { "⚠️ MEDIO" } else { "❌ ALTO" }
    Valor = "$CapitalSOL SOL"
    Recomendacion = if ($CapitalSOL -lt 0.25) { "Considerar aumentar capital" } else { "Suficiente para trading conservador" }
}

# Riesgo de fees
$estimatedFeesPerTrade = 0.003  # ~0.003 SOL por trade estimado
$feesRisk = ($estimatedFeesPerTrade / $CapitalSOL) * 100

$risks += [PSCustomObject]@{
    Categoria = "Fees"
    Riesgo = "Impacto de fees por trade"
    Nivel = if ($feesRisk -le 2) { "✅ BAJO" } elseif ($feesRisk -le 5) { "⚠️ MEDIO" } else { "❌ ALTO" }
    Valor = "$([math]::Round($feesRisk, 2))% del capital"
    Recomendacion = if ($feesRisk -gt 5) { "Trades más grandes para diluir fees" } else { "Impacto aceptable" }
}

foreach ($risk in $risks) {
    $color = if ($risk.Nivel.StartsWith("✅")) { "Green" } elseif ($risk.Nivel.StartsWith("⚠️")) { "Yellow" } else { "Red" }
    Write-Host "   • $($risk.Categoria) - $($risk.Riesgo):" -ForegroundColor White
    Write-Host "     ├─ Nivel: $($risk.Nivel)" -ForegroundColor $color
    Write-Host "     ├─ Valor: $($risk.Valor)" -ForegroundColor White
    Write-Host "     └─ Recomendación: $($risk.Recomendacion)" -ForegroundColor Cyan
}

# Score de riesgos
$lowRisks = ($risks | Where-Object { $_.Nivel.StartsWith("✅") }).Count
$mediumRisks = ($risks | Where-Object { $_.Nivel.StartsWith("⚠️") }).Count
$highRisks = ($risks | Where-Object { $_.Nivel.StartsWith("❌") }).Count

$riskScore = (($lowRisks * 100) + ($mediumRisks * 50) + ($highRisks * 0)) / $risks.Count
$riskReadiness = if ($riskScore -ge 80) { "✅ BAJO RIESGO" } elseif ($riskScore -ge 60) { "⚠️ RIESGO MEDIO" } else { "❌ ALTO RIESGO" }

Write-Host "`n📊 SCORE RIESGOS: $([math]::Round($riskScore, 0))/100 - $riskReadiness" -ForegroundColor $(if($riskScore -ge 60){"Green"}elseif($riskScore -ge 40){"Yellow"}else{"Red"})

# ═══════════════════════════════════════════════════════════════════════
# 🎯 SECCIÓN 4: EVALUACIÓN FINAL Y RECOMENDACIONES MILITARES
# ═══════════════════════════════════════════════════════════════════════

Write-Host "`n🎯 SECCIÓN 4: EVALUACIÓN FINAL Y RECOMENDACIONES MILITARES" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

# 4.1 Cálculo del Score Final Militar
Write-Host "`n🎖️ 4.1 CÁLCULO SCORE FINAL MILITAR:" -ForegroundColor Yellow

$assetWeight = 25
$configWeight = 25  
$performanceWeight = 35
$riskWeight = 15

$assetScoreNum = if ($assetReadiness.StartsWith("✅")) { 100 } else { 0 }
$configScoreNum = if ($configReadiness -eq "✅ EXCELENTE") { 100 } elseif ($configReadiness -eq "⚠️ ACEPTABLE") { 70 } else { 0 }

$finalScore = [math]::Round(
    ($assetScoreNum * $assetWeight/100) + 
    ($configScoreNum * $configWeight/100) + 
    ($performanceScore * $performanceWeight/100) + 
    ($riskScore * $riskWeight/100), 0)

Write-Host "   📊 COMPONENTES DEL SCORE:" -ForegroundColor White
Write-Host "      • Assets críticos ($assetWeight%): $assetScoreNum/100" -ForegroundColor White
Write-Host "      • Configuración ($configWeight%): $configScoreNum/100" -ForegroundColor White  
Write-Host "      • Performance ($performanceWeight%): $performanceScore/100" -ForegroundColor White
Write-Host "      • Gestión riesgos ($riskWeight%): $([math]::Round($riskScore, 0))/100" -ForegroundColor White

# 4.2 Determinación de readiness militar
Write-Host "`n🎖️ 4.2 DETERMINACIÓN DE READINESS MILITAR:" -ForegroundColor Yellow

$militaryReadiness = if ($finalScore -ge 85) { "🏆 ELITE - LISTO PARA COMBATE" } 
elseif ($finalScore -ge 75) { "✅ OPERACIONAL - AUTORIZADO PARA TRADING" }
elseif ($finalScore -ge 65) { "⚠️ CONDICIONAL - TRADING CON SUPERVISIÓN" }
elseif ($finalScore -ge 50) { "⚠️ ENTRENAMIENTO - NECESITA MEJORAS" }
else { "❌ NO LISTO - ABORTAR MISIÓN" }

$readinessColor = if ($finalScore -ge 75) { "Green" } 
elseif ($finalScore -ge 50) { "Yellow" } 
else { "Red" }

Write-Host "`n🎯 SCORE FINAL MILITAR: $finalScore/100" -ForegroundColor Cyan
Write-Host "🎖️ READINESS STATUS: $militaryReadiness" -ForegroundColor $readinessColor

# 4.3 Recomendaciones operacionales específicas
Write-Host "`n⚔️ 4.3 RECOMENDACIONES OPERACIONALES:" -ForegroundColor Yellow

if ($finalScore -ge 75) {
    Write-Host "`n🚀 OPERACIONES AUTORIZADAS:" -ForegroundColor Green
    Write-Host "   ✅ Sistema listo para trading real" -ForegroundColor Green
    Write-Host "   ✅ Configuración validada para $CapitalSOL SOL" -ForegroundColor Green
    Write-Host "   ✅ Performance demostrada en simulación" -ForegroundColor Green
    Write-Host "   ✅ Riesgos bajo control" -ForegroundColor Green
    
    Write-Host "`n📋 PLAN DE EJECUCIÓN RECOMENDADO:" -ForegroundColor Cyan
    Write-Host "   1. ✅ Ejecutar: .\prepare_real_trading_0.29SOL.ps1" -ForegroundColor White
    Write-Host "   2. ✅ Iniciar trading con Cross-Chain (estrategia probada)" -ForegroundColor White
    Write-Host "   3. ✅ Monitorear primeros 5 trades de cerca" -ForegroundColor White
    Write-Host "   4. ✅ Aplicar stop-loss automático si pérdidas > 0.03 SOL" -ForegroundColor White
    
} elseif ($finalScore -ge 50) {
    Write-Host "`n⚠️ OPERACIONES CONDICIONALES:" -ForegroundColor Yellow
    Write-Host "   ⚠️ Sistema funcional pero requiere mejoras" -ForegroundColor Yellow
    
    Write-Host "`n📋 MEJORAS REQUERIDAS:" -ForegroundColor Cyan
    if ($performanceScore -lt 60) {
        Write-Host "   • Mejorar performance: Ejecutar simulación más larga" -ForegroundColor White
    }
    if ($riskScore -lt 70) {
        Write-Host "   • Reducir riesgos: Ajustar max_trade_sol" -ForegroundColor White
    }
    if ($configScoreNum -lt 75) {
        Write-Host "   • Optimizar configuración: Revisar parámetros" -ForegroundColor White
    }
    
    Write-Host "`n📋 TRADING CONDICIONAL:" -ForegroundColor Cyan
    Write-Host "   1. ⚠️ Proceder SOLO con trades muy pequeños (0.01 SOL max)" -ForegroundColor Yellow
    Write-Host "   2. ⚠️ Monitorear cada operación manualmente" -ForegroundColor Yellow
    Write-Host "   3. ⚠️ Stop total si pérdidas > 0.02 SOL" -ForegroundColor Yellow
    
} else {
    Write-Host "`n❌ OPERACIONES NO AUTORIZADAS:" -ForegroundColor Red
    Write-Host "   ❌ Sistema NO listo para trading real" -ForegroundColor Red
    Write-Host "   ❌ Riesgos demasiado altos" -ForegroundColor Red
    Write-Host "   ❌ Performance insuficiente" -ForegroundColor Red
    
    Write-Host "`n📋 ACCIONES REQUERIDAS:" -ForegroundColor Cyan
    Write-Host "   1. ❌ NO activar trading real" -ForegroundColor Red
    Write-Host "   2. 🔧 Ejecutar diagnósticos adicionales" -ForegroundColor Yellow
    Write-Host "   3. 🔧 Optimizar configuración" -ForegroundColor Yellow
    Write-Host "   4. 🔧 Aumentar capital si es posible" -ForegroundColor Yellow
}

# 4.4 Proyecciones financieras militares
if ($finalScore -ge 75 -and $totalProfitSOL -gt 0) {
    Write-Host "`n💰 4.4 PROYECCIONES FINANCIERAS MILITARES:" -ForegroundColor Yellow
    
    $projHourly = $totalProfitSOL * 30  # 2 min → 1 hora
    $projDaily = $projHourly * 24
    $projWeekly = $projDaily * 7
    
    Write-Host "   📈 PROYECCIONES OPTIMISTAS:" -ForegroundColor Green
    Write-Host "      • Profit por hora: $([math]::Round($projHourly, 4)) SOL" -ForegroundColor White
    Write-Host "      • Profit por día: $([math]::Round($projDaily, 4)) SOL" -ForegroundColor White
    Write-Host "      • Profit por semana: $([math]::Round($projWeekly, 4)) SOL" -ForegroundColor White
    Write-Host "      • ROI semanal: $([math]::Round(($projWeekly / $CapitalSOL) * 100, 2))%" -ForegroundColor Green
    
    Write-Host "`n   ⚠️ DISCLAIMER MILITAR:" -ForegroundColor Yellow
    Write-Host "      Las proyecciones son estimadas y pueden variar según condiciones de mercado" -ForegroundColor Gray
}

# 4.5 Resumen ejecutivo final
Write-Host "`n📋 4.5 RESUMEN EJECUTIVO FINAL:" -ForegroundColor Yellow
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

Write-Host "Capital analizado: $CapitalSOL SOL" -ForegroundColor White
Write-Host "Score militar final: $finalScore/100" -ForegroundColor Cyan
Write-Host "Status operacional: $militaryReadiness" -ForegroundColor $readinessColor

if ($finalScore -ge 75) {
    Write-Host "`n🎖️ AUTORIZACIÓN MILITAR: CONCEDIDA" -ForegroundColor Green
    Write-Host "🚀 Proceder con trading real autorizado" -ForegroundColor Green
} else {
    Write-Host "`n❌ AUTORIZACIÓN MILITAR: DENEGADA" -ForegroundColor Red
    Write-Host "⚠️ Completar mejoras antes de proceder" -ForegroundColor Red
}

Write-Host "`n🎖️ ANÁLISIS MILITAR COMPLETADO - $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor Red
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Red
