# 📊 ANÁLISIS DETALLADO DE PROFIT REAL - 0.29 SOL
# Analiza el log para determinar profit real vs oportunidades detectadas

param(
    [string]$LogFile = "quick_sim_0.29SOL_20250729_154833.log"
)

Write-Host "🔍 ANÁLISIS DETALLADO DE PROFIT REAL PARA 0.29 SOL" -ForegroundColor Cyan
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

if (-not (Test-Path $LogFile)) {
    Write-Host "❌ Log file no encontrado: $LogFile" -ForegroundColor Red
    exit 1
}

$logContent = Get-Content $LogFile -Raw

Write-Host "📋 Analizando log: $LogFile" -ForegroundColor Yellow
Write-Host "📏 Tamaño del log: $([math]::Round((Get-Item $LogFile).Length / 1KB, 2)) KB" -ForegroundColor White

# 1. ANÁLISIS DE OPORTUNIDADES CROSS-CHAIN (POSITIVAS)
Write-Host "`n🌐 OPORTUNIDADES CROSS-CHAIN RENTABLES:" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Green

$crossChainRegex = "Cross-Chain:.*\$(\d+\.?\d*)\s+net profit.*(\d+\.?\d*)%"
$crossChainMatches = [regex]::Matches($logContent, $crossChainRegex)

$totalCrossChainProfit = 0
$crossChainCount = 0

foreach ($match in $crossChainMatches) {
    $profitUSD = [decimal]$match.Groups[1].Value
    $percentage = [decimal]$match.Groups[2].Value
    $profitSOL = [math]::Round($profitUSD / 290, 4)  # Asumiendo $290 por SOL
    
    Write-Host "   ✅ Cross-chain: +$profitUSD USD (+$profitSOL SOL) - $percentage%" -ForegroundColor Green
    
    $totalCrossChainProfit += $profitSOL
    $crossChainCount++
}

Write-Host "`n📊 RESUMEN CROSS-CHAIN:" -ForegroundColor Cyan
Write-Host "   • Oportunidades rentables: $crossChainCount" -ForegroundColor Green
Write-Host "   • Profit total estimado: $([math]::Round($totalCrossChainProfit, 4)) SOL" -ForegroundColor Green
Write-Host "   • Profit promedio por operación: $([math]::Round($totalCrossChainProfit / [math]::Max($crossChainCount, 1), 4)) SOL" -ForegroundColor Green

# 2. ANÁLISIS DE FLASH LOANS
Write-Host "`n💰 OPORTUNIDADES FLASH LOANS:" -ForegroundColor Yellow
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Yellow

$flashLoanRegex = "Flash Loan.*profit.*(\d+\.?\d+)\s*SOL"
$flashLoanMatches = [regex]::Matches($logContent, $flashLoanRegex)

$totalFlashLoanProfit = 0
$flashLoanCount = 0

foreach ($match in $flashLoanMatches) {
    $profitSOL = [decimal]$match.Groups[1].Value
    Write-Host "   ✅ Flash loan: +$profitSOL SOL" -ForegroundColor Yellow
    
    $totalFlashLoanProfit += $profitSOL
    $flashLoanCount++
}

if ($flashLoanCount -eq 0) {
    # Buscar otro patrón de flash loans
    $flashLoanRegex2 = "(\d+\.?\d+)\s*SOL.*(\d+\.?\d+)\s*SOL profit.*Flash"
    $flashLoanMatches2 = [regex]::Matches($logContent, $flashLoanRegex2)
    
    foreach ($match in $flashLoanMatches2) {
        $loanAmount = [decimal]$match.Groups[1].Value
        $profitSOL = [decimal]$match.Groups[2].Value
        Write-Host "   ✅ Flash loan: $loanAmount SOL → +$profitSOL SOL profit" -ForegroundColor Yellow
        
        $totalFlashLoanProfit += $profitSOL
        $flashLoanCount++
    }
}

Write-Host "`n📊 RESUMEN FLASH LOANS:" -ForegroundColor Cyan
Write-Host "   • Oportunidades rentables: $flashLoanCount" -ForegroundColor Green
Write-Host "   • Profit total estimado: $([math]::Round($totalFlashLoanProfit, 4)) SOL" -ForegroundColor Green

# 3. ANÁLISIS DE ARBITRAJES TRADICIONALES (NEGATIVOS)
Write-Host "`n❌ ARBITRAJES TRADICIONALES (NO RENTABLES):" -ForegroundColor Red
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Red

$negativeRegex = "NET PROFIT:\s*-(\d+\.?\d+)\s*SOL"
$negativeMatches = [regex]::Matches($logContent, $negativeRegex)

$totalNegativeProfit = 0
$negativeCount = 0

foreach ($match in $negativeMatches) {
    $lossSOL = [decimal]$match.Groups[1].Value
    Write-Host "   ❌ Pérdida: -$lossSOL SOL" -ForegroundColor Red
    
    $totalNegativeProfit += $lossSOL
    $negativeCount++
}

Write-Host "`n📊 RESUMEN PÉRDIDAS:" -ForegroundColor Cyan
Write-Host "   • Operaciones con pérdida: $negativeCount" -ForegroundColor Red
Write-Host "   • Pérdida total: -$([math]::Round($totalNegativeProfit, 4)) SOL" -ForegroundColor Red
Write-Host "   • Pérdida promedio: -$([math]::Round($totalNegativeProfit / [math]::Max($negativeCount, 1), 4)) SOL" -ForegroundColor Red

# 4. CÁLCULO DE PROFIT NETO REAL
Write-Host "`n💎 ANÁLISIS DE PROFIT NETO PARA 0.29 SOL:" -ForegroundColor Magenta
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Magenta

$netProfitSOL = $totalCrossChainProfit + $totalFlashLoanProfit - $totalNegativeProfit
$netProfitUSD = $netProfitSOL * 290
$roi = ($netProfitSOL / 0.29) * 100

Write-Host "📊 PROFIT BRUTO:" -ForegroundColor Green
Write-Host "   • Cross-chain: +$([math]::Round($totalCrossChainProfit, 4)) SOL" -ForegroundColor Green
Write-Host "   • Flash loans: +$([math]::Round($totalFlashLoanProfit, 4)) SOL" -ForegroundColor Green
Write-Host "   • Total bruto: +$([math]::Round($totalCrossChainProfit + $totalFlashLoanProfit, 4)) SOL" -ForegroundColor Green

Write-Host "`n📊 COSTOS:" -ForegroundColor Red
Write-Host "   • Pérdidas/fees: -$([math]::Round($totalNegativeProfit, 4)) SOL" -ForegroundColor Red

Write-Host "`n💰 PROFIT NETO FINAL:" -ForegroundColor Cyan
Write-Host "   • Profit neto: $([math]::Round($netProfitSOL, 4)) SOL (~$$([math]::Round($netProfitUSD, 2)) USD)" -ForegroundColor $(if($netProfitSOL -gt 0){"Green"}else{"Red"})
Write-Host "   • ROI en 2 minutos: $([math]::Round($roi, 2))%" -ForegroundColor $(if($roi -gt 0){"Green"}else{"Red"})

# 5. PROYECCIÓN Y RECOMENDACIONES
Write-Host "`n🚀 PROYECCIONES PARA TRADING REAL:" -ForegroundColor Magenta
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Magenta

if ($netProfitSOL -gt 0) {
    $hourlyROI = $roi * 30  # 2 minutos → 1 hora
    $dailyROI = $hourlyROI * 24
    
    Write-Host "⏰ PROYECCIONES OPTIMISTAS:" -ForegroundColor Yellow
    Write-Host "   • ROI por hora: $([math]::Round($hourlyROI, 2))%" -ForegroundColor Yellow
    Write-Host "   • ROI por día: $([math]::Round($dailyROI, 2))%" -ForegroundColor Yellow
    Write-Host "   • Profit diario estimado: $([math]::Round($netProfitSOL * 30 * 24, 4)) SOL" -ForegroundColor Yellow
    
    if ($roi -gt 1) {
        Write-Host "`n🎯 RECOMENDACIÓN: ¡EXCELENTE RENTABILIDAD!" -ForegroundColor Green
        Write-Host "   ✅ El sistema genera profit neto positivo" -ForegroundColor Green
        Write-Host "   ✅ Cross-chain arbitrage es altamente rentable" -ForegroundColor Green
        Write-Host "   ✅ Listo para trading real con 0.29 SOL" -ForegroundColor Green
        Write-Host "`n🚀 ACTIVAR TRADING REAL:" -ForegroundColor Magenta
        Write-Host "   .\prepare_real_trading_0.29SOL.ps1" -ForegroundColor White
    } else {
        Write-Host "`n⚠️  RECOMENDACIÓN: Rentabilidad baja pero positiva" -ForegroundColor Yellow
        Write-Host "   • Considerar esperar mejores condiciones de mercado" -ForegroundColor Yellow
        Write-Host "   • O proceder con trading muy conservador" -ForegroundColor Yellow
    }
} else {
    Write-Host "`n❌ RECOMENDACIÓN: NO PROCEDER AÚN" -ForegroundColor Red
    Write-Host "   • Profit neto negativo en esta simulación" -ForegroundColor Red
    Write-Host "   • Esperar mejores condiciones de mercado" -ForegroundColor Red
    Write-Host "   • O ajustar configuración para ser más agresivo" -ForegroundColor Red
}

# 6. ANÁLISIS DE EFICIENCIA
Write-Host "`n📈 ANÁLISIS DE EFICIENCIA:" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

$totalOportunidades = $crossChainCount + $flashLoanCount + $negativeCount
$efectividad = if ($totalOportunidades -gt 0) { (($crossChainCount + $flashLoanCount) / $totalOportunidades) * 100 } else { 0 }

Write-Host "📊 ESTADÍSTICAS:" -ForegroundColor White
Write-Host "   • Total oportunidades analizadas: $totalOportunidades" -ForegroundColor White
Write-Host "   • Oportunidades rentables: $($crossChainCount + $flashLoanCount)" -ForegroundColor Green
Write-Host "   • Oportunidades no rentables: $negativeCount" -ForegroundColor Red
Write-Host "   • Efectividad: $([math]::Round($efectividad, 2))%" -ForegroundColor $(if($efectividad -gt 50){"Green"}else{"Red"})

Write-Host "`n✨ ANÁLISIS COMPLETADO - DATOS REALES DE TU CAPITAL" -ForegroundColor Magenta
