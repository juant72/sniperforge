# 🔍 VERIFICACIÓN DE AUTENTICIDAD DE DATOS - DETECTIVE FORENSIC
# Análisis forense para detectar datos sintéticos vs reales

param(
    [string]$LogFile = "quick_sim_0.29SOL_20250729_154833.log"
)

Write-Host "🔍 VERIFICACIÓN FORENSIC DE AUTENTICIDAD DE DATOS" -ForegroundColor Red
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Red
Write-Host "🎯 INVESTIGANDO SI LOS DATOS SON REALES O SINTÉTICOS" -ForegroundColor Yellow
Write-Host "📅 Análisis iniciado: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor White

if (-not (Test-Path $LogFile)) {
    Write-Host "❌ Log file no encontrado: $LogFile" -ForegroundColor Red
    exit 1
}

$logContent = Get-Content $LogFile -Raw
$logLines = Get-Content $LogFile

Write-Host "`n📋 INFORMACIÓN BÁSICA DEL LOG:" -ForegroundColor Cyan
Write-Host "   • Archivo: $LogFile" -ForegroundColor White
Write-Host "   • Tamaño: $([math]::Round((Get-Item $LogFile).Length / 1KB, 2)) KB" -ForegroundColor White
Write-Host "   • Líneas totales: $($logLines.Count)" -ForegroundColor White

# ═══════════════════════════════════════════════════════════════════════
# 🔍 ANÁLISIS 1: TIMESTAMPS - ¿SON REALES O SINTÉTICOS?
# ═══════════════════════════════════════════════════════════════════════

Write-Host "`n🕐 ANÁLISIS 1: VERIFICACIÓN DE TIMESTAMPS" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

# Extraer timestamps del log
$timestampRegex = "(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)"
$timestampMatches = [regex]::Matches($logContent, $timestampRegex)

if ($timestampMatches.Count -gt 0) {
    $firstTimestamp = [DateTime]::Parse($timestampMatches[0].Groups[1].Value.Replace('Z', '+00:00'))
    $lastTimestamp = [DateTime]::Parse($timestampMatches[-1].Groups[1].Value.Replace('Z', '+00:00'))
    $duration = $lastTimestamp - $firstTimestamp
    
    Write-Host "   📊 ANÁLISIS TEMPORAL:" -ForegroundColor Yellow
    Write-Host "      • Primer timestamp: $($firstTimestamp.ToString('HH:mm:ss.fff'))" -ForegroundColor White
    Write-Host "      • Último timestamp: $($lastTimestamp.ToString('HH:mm:ss.fff'))" -ForegroundColor White
    Write-Host "      • Duración real: $($duration.TotalMinutes.ToString('F2')) minutos" -ForegroundColor White
    Write-Host "      • Total timestamps: $($timestampMatches.Count)" -ForegroundColor White
    
    # Verificar si la duración es consistente con lo esperado (2 minutos)
    if ($duration.TotalMinutes -lt 1.5) {
        Write-Host "   🚨 SOSPECHA: Duración muy corta para generar datos reales" -ForegroundColor Red
    } elseif ($duration.TotalMinutes -gt 3) {
        Write-Host "   ⚠️ NOTA: Duración más larga de lo esperado" -ForegroundColor Yellow
    } else {
        Write-Host "   ✅ Duración consistente con simulación de 2 minutos" -ForegroundColor Green
    }
    
    # Analizar distribución de timestamps
    $timestampIntervals = @()
    for ($i = 1; $i -lt $timestampMatches.Count; $i++) {
        $prev = [DateTime]::Parse($timestampMatches[$i-1].Groups[1].Value.Replace('Z', '+00:00'))
        $curr = [DateTime]::Parse($timestampMatches[$i].Groups[1].Value.Replace('Z', '+00:00'))
        $interval = ($curr - $prev).TotalMilliseconds
        if ($interval -gt 0) { $timestampIntervals += $interval }
    }
    
    if ($timestampIntervals.Count -gt 0) {
        $avgInterval = [math]::Round(($timestampIntervals | Measure-Object -Average).Average, 2)
        $minInterval = [math]::Round(($timestampIntervals | Measure-Object -Minimum).Minimum, 2)
        $maxInterval = [math]::Round(($timestampIntervals | Measure-Object -Maximum).Maximum, 2)
        
        Write-Host "`n   ⏱️ ANÁLISIS DE INTERVALOS:" -ForegroundColor Yellow
        Write-Host "      • Intervalo promedio: $avgInterval ms" -ForegroundColor White
        Write-Host "      • Intervalo mínimo: $minInterval ms" -ForegroundColor White
        Write-Host "      • Intervalo máximo: $maxInterval ms" -ForegroundColor White
        
        # Detectar patrones sospechosos
        if ($minInterval -eq $maxInterval) {
            Write-Host "   🚨 SOSPECHA ALTA: Intervalos perfectamente uniformes (posible generación sintética)" -ForegroundColor Red
        } elseif ($avgInterval -lt 10) {
            Write-Host "   🚨 SOSPECHA: Intervalos muy cortos (posible generación masiva sintética)" -ForegroundColor Red
        } else {
            Write-Host "   ✅ Variabilidad natural en intervalos detectada" -ForegroundColor Green
        }
    }
} else {
    Write-Host "   ❌ No se encontraron timestamps válidos en el log" -ForegroundColor Red
}

# ═══════════════════════════════════════════════════════════════════════
# 🔍 ANÁLISIS 2: PATRONES DE OPORTUNIDADES - ¿REALES O GENERADAS?
# ═══════════════════════════════════════════════════════════════════════

Write-Host "`n💰 ANÁLISIS 2: PATRONES DE OPORTUNIDADES" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

# Buscar oportunidades cross-chain reportadas
$crossChainLines = $logLines | Where-Object { $_ -match "Cross-Chain.*profit" }
$flashLoanLines = $logLines | Where-Object { $_ -match "Flash Loan.*profit" }
$traditionalLines = $logLines | Where-Object { $_ -match "Traditional.*profit" }

Write-Host "   📊 CONTEO DE OPORTUNIDADES REPORTADAS:" -ForegroundColor Yellow
Write-Host "      • Cross-chain mencionadas: $($crossChainLines.Count)" -ForegroundColor White
Write-Host "      • Flash loans mencionadas: $($flashLoanLines.Count)" -ForegroundColor White
Write-Host "      • Tradicionales mencionadas: $($traditionalLines.Count)" -ForegroundColor White

# Analizar las primeras líneas del log para detectar patrones
Write-Host "`n   🔍 MUESTRA DE OPORTUNIDADES CROSS-CHAIN:" -ForegroundColor Yellow
$sampleCrossChain = $crossChainLines | Select-Object -First 5
foreach ($line in $sampleCrossChain) {
    if ($line -match "Cross-Chain.*(\$\d+\.?\d*)\s+.*(\d+\.?\d*)%") {
        $profit = $matches[1]
        $percentage = $matches[2]
        Write-Host "      • $profit profit ($percentage%)" -ForegroundColor White
    } else {
        Write-Host "      • $($line.Substring(0, [math]::Min(80, $line.Length)))..." -ForegroundColor Gray
    }
}

# ═══════════════════════════════════════════════════════════════════════
# 🔍 ANÁLISIS 3: VERIFICACIÓN DE DATOS DE MERCADO REALES
# ═══════════════════════════════════════════════════════════════════════

Write-Host "`n🌐 ANÁLISIS 3: VERIFICACIÓN DE DATOS DE MERCADO" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

# Buscar llamadas a APIs reales
$jupiterCalls = ($logContent | Select-String "Jupiter.*API" -AllMatches).Matches.Count
$dexScreenerCalls = ($logContent | Select-String "DexScreener.*API" -AllMatches).Matches.Count
$rpcCalls = ($logContent | Select-String "RPC.*call" -AllMatches).Matches.Count

Write-Host "   📡 ACTIVIDAD DE APIs:" -ForegroundColor Yellow
Write-Host "      • Llamadas Jupiter API detectadas: $jupiterCalls" -ForegroundColor White
Write-Host "      • Llamadas DexScreener detectadas: $dexScreenerCalls" -ForegroundColor White
Write-Host "      • Llamadas RPC detectadas: $rpcCalls" -ForegroundColor White

# Buscar precios reales
$priceLines = $logLines | Where-Object { $_ -match "precio|price.*\$" }
Write-Host "      • Líneas con datos de precios: $($priceLines.Count)" -ForegroundColor White

# Verificar si hay datos hardcodeados sospechosos
$suspiciousPatterns = @(
    "mock",
    "fake",
    "test.*data",
    "synthetic",
    "generated",
    "dummy"
)

$suspiciousFindings = @()
foreach ($pattern in $suspiciousPatterns) {
    $matches = ($logContent | Select-String $pattern -AllMatches).Matches.Count
    if ($matches -gt 0) {
        $suspiciousFindings += "$pattern: $matches coincidencias"
    }
}

if ($suspiciousFindings.Count -gt 0) {
    Write-Host "`n   🚨 PATRONES SOSPECHOSOS DETECTADOS:" -ForegroundColor Red
    foreach ($finding in $suspiciousFindings) {
        Write-Host "      • $finding" -ForegroundColor Red
    }
} else {
    Write-Host "`n   ✅ No se detectaron patrones obvios de datos sintéticos" -ForegroundColor Green
}

# ═══════════════════════════════════════════════════════════════════════
# 🔍 ANÁLISIS 4: VERIFICACIÓN DE PROFIT REAL VS REPORTADO
# ═══════════════════════════════════════════════════════════════════════

Write-Host "`n💎 ANÁLISIS 4: PROFIT REAL VS REPORTADO" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

# Buscar líneas de NET PROFIT (datos reales de ejecución)
$netProfitLines = $logLines | Where-Object { $_ -match "NET PROFIT" }
$negativeProfit = $netProfitLines | Where-Object { $_ -match "NET PROFIT.*-" }
$positiveProfit = $netProfitLines | Where-Object { $_ -match "NET PROFIT.*[^-]\d" }

Write-Host "   💰 ANÁLISIS DE PROFIT REAL:" -ForegroundColor Yellow
Write-Host "      • Líneas NET PROFIT total: $($netProfitLines.Count)" -ForegroundColor White
Write-Host "      • Profits NEGATIVOS: $($negativeProfit.Count)" -ForegroundColor Red
Write-Host "      • Profits POSITIVOS: $($positiveProfit.Count)" -ForegroundColor Green

# Buscar líneas de "NO rentable"
$nonProfitableLines = $logLines | Where-Object { $_ -match "NO rentable|not profitable" }
Write-Host "      • Operaciones NO rentables: $($nonProfitableLines.Count)" -ForegroundColor Red

# Muestra de profits negativos
if ($negativeProfit.Count -gt 0) {
    Write-Host "`n   📉 MUESTRA DE PÉRDIDAS REALES:" -ForegroundColor Red
    $negativeProfit | Select-Object -First 3 | ForEach-Object {
        if ($_ -match "NET PROFIT:\s*(-\d+\.?\d*)\s*SOL") {
            Write-Host "      • Pérdida: $($matches[1]) SOL" -ForegroundColor Red
        }
    }
}

# ═══════════════════════════════════════════════════════════════════════
# 🔍 ANÁLISIS 5: DETECCIÓN DE SIMULACIÓN vs EJECUCIÓN REAL
# ═══════════════════════════════════════════════════════════════════════

Write-Host "`n🎭 ANÁLISIS 5: SIMULACIÓN vs EJECUCIÓN REAL" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

# Buscar indicadores de modo simulación
$simulationIndicators = @(
    "simulation mode",
    "modo simulación",
    "simulated",
    "preview",
    "dry run"
)

$simulationFindings = @()
foreach ($indicator in $simulationIndicators) {
    $matches = ($logContent | Select-String $indicator -AllMatches).Matches.Count
    if ($matches -gt 0) {
        $simulationFindings += "$indicator: $matches menciones"
    }
}

Write-Host "   🎭 INDICADORES DE SIMULACIÓN:" -ForegroundColor Yellow
if ($simulationFindings.Count -gt 0) {
    foreach ($finding in $simulationFindings) {
        Write-Host "      • $finding" -ForegroundColor Yellow
    }
} else {
    Write-Host "      • No se encontraron indicadores explícitos de simulación" -ForegroundColor White
}

# Buscar transacciones reales en blockchain
$txIndicators = @(
    "transaction.*confirmed",
    "signature.*",
    "blockhash",
    "confirmed.*tx"
)

$realTxCount = 0
foreach ($indicator in $txIndicators) {
    $realTxCount += ($logContent | Select-String $indicator -AllMatches).Matches.Count
}

Write-Host "      • Indicadores de transacciones reales: $realTxCount" -ForegroundColor $(if($realTxCount -gt 0){"Green"}else{"Red"})

# ═══════════════════════════════════════════════════════════════════════
# 🎯 VEREDICTO FINAL FORENSIC
# ═══════════════════════════════════════════════════════════════════════

Write-Host "`n🎯 VEREDICTO FINAL FORENSIC" -ForegroundColor Red
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Red

# Calcular score de autenticidad
$authenticityScore = 0
$maxScore = 100

# Criterios de autenticidad
if ($timestampMatches.Count -gt 100) { $authenticityScore += 20 }
if ($avgInterval -gt 10 -and $avgInterval -lt 1000) { $authenticityScore += 15 }
if ($negativeProfit.Count -gt $positiveProfit.Count) { $authenticityScore += 25 }  # Más pérdidas que ganancias = más real
if ($nonProfitableLines.Count -gt 10) { $authenticityScore += 15 }  # Operaciones fallidas = más real
if ($suspiciousFindings.Count -eq 0) { $authenticityScore += 15 }
if ($realTxCount -eq 0 -and $simulationFindings.Count -gt 0) { $authenticityScore += 10 }  # Simulación claramente marcada

Write-Host "📊 SCORE DE AUTENTICIDAD: $authenticityScore/$maxScore" -ForegroundColor Cyan

if ($authenticityScore -ge 80) {
    $verdict = "✅ DATOS AUTÉNTICOS"
    $verdictColor = "Green"
    $explanation = "Los datos muestran patrones consistentes con ejecución real"
} elseif ($authenticityScore -ge 60) {
    $verdict = "⚠️ DATOS MIXTOS"
    $verdictColor = "Yellow"
    $explanation = "Mezcla de datos reales y simulados detectada"
} elseif ($authenticityScore -ge 40) {
    $verdict = "🚨 DATOS SOSPECHOSOS"
    $verdictColor = "Yellow"
    $explanation = "Patrones inconsistentes detectados, posible generación sintética"
} else {
    $verdict = "❌ DATOS SINTÉTICOS"
    $verdictColor = "Red"
    $explanation = "Evidencia fuerte de datos generados artificialmente"
}

Write-Host "`n🏆 VEREDICTO: $verdict" -ForegroundColor $verdictColor
Write-Host "💡 EXPLICACIÓN: $explanation" -ForegroundColor White

# Conclusiones específicas
Write-Host "`n📋 CONCLUSIONES ESPECÍFICAS:" -ForegroundColor Cyan

if ($negativeProfit.Count -gt 50 -and $positiveProfit.Count -eq 0) {
    Write-Host "   ✅ REAL: $($negativeProfit.Count) operaciones con pérdidas reales detectadas" -ForegroundColor Green
    Write-Host "   ✅ REAL: Sistema intentó ejecutar trades pero fueron no rentables" -ForegroundColor Green
    Write-Host "   ✅ REAL: Fees y slippage causaron pérdidas (comportamiento esperado)" -ForegroundColor Green
}

if ($crossChainLines.Count -gt 50 -and $netProfitLines.Count -gt 50) {
    Write-Host "   🤔 DISCREPANCIA: $($crossChainLines.Count) oportunidades reportadas vs $($negativeProfit.Count) pérdidas reales" -ForegroundColor Yellow
    Write-Host "   💡 EXPLICACIÓN: Oportunidades teóricas no se tradujeron en profits ejecutables" -ForegroundColor Yellow
}

Write-Host "`n🔍 ANÁLISIS FORENSIC COMPLETADO - $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor Red
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Red
