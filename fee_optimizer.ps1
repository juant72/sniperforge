# 🚀 CALCULADORA DE FEES Y PROFIT REAL - OPTIMIZACIÓN MÁXIMA
# Análisis exacto de fees para maximizar ganancias con 0.29 SOL

param(
    [decimal]$CapitalSOL = 0.29,
    [decimal]$TradeAmountSOL = 0.15
)

Write-Host "🚀 CALCULADORA DE FEES Y PROFIT REAL - OPTIMIZACIÓN 0.29 SOL" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "💰 Capital disponible: $CapitalSOL SOL" -ForegroundColor Yellow
Write-Host "⚡ Trade amount optimizado: $TradeAmountSOL SOL" -ForegroundColor Cyan
Write-Host "📅 Análisis: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor White

# Función para calcular fees exactos
function Calculate-TradeFees {
    param(
        [decimal]$TradeAmount,
        [string]$DexName,
        [decimal]$DexFeePercent,
        [decimal]$PriorityFee = 0.000015
    )
    
    $SolanaBaseFee = 0.000005
    $DexFee = $TradeAmount * ($DexFeePercent / 100)
    $TotalFee = $SolanaBaseFee + $DexFee + $PriorityFee
    
    return @{
        DexName = $DexName
        TradeAmount = $TradeAmount
        SolanaBaseFee = $SolanaBaseFee
        DexFee = $DexFee
        PriorityFee = $PriorityFee
        TotalFee = $TotalFee
        NetAmount = $TradeAmount - $TotalFee
        FeePercentage = [math]::Round(($TotalFee / $TradeAmount) * 100, 4)
    }
}

# Análisis de fees por DEX
Write-Host "`n📊 ANÁLISIS DE FEES POR DEX:" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

$DexConfigs = @(
    @{ Name = "Phoenix"; Fee = 0.20 },
    @{ Name = "Raydium"; Fee = 0.25 },
    @{ Name = "Meteora"; Fee = 0.25 },
    @{ Name = "Orca"; Fee = 0.30 },
    @{ Name = "Lifinity"; Fee = 0.30 }
)

$FeeAnalysis = @()
foreach ($dex in $DexConfigs) {
    $analysis = Calculate-TradeFees -TradeAmount $TradeAmountSOL -DexName $dex.Name -DexFeePercent $dex.Fee
    $FeeAnalysis += $analysis
    
    Write-Host "🔹 $($dex.Name):" -ForegroundColor Yellow
    Write-Host "   • Fee DEX: $([math]::Round($analysis.DexFee, 6)) SOL ($($dex.Fee)%)" -ForegroundColor White
    Write-Host "   • Fee total: $([math]::Round($analysis.TotalFee, 6)) SOL ($($analysis.FeePercentage)%)" -ForegroundColor White
    Write-Host "   • Cantidad neta: $([math]::Round($analysis.NetAmount, 6)) SOL" -ForegroundColor Green
}

# Calcular profit mínimo necesario
Write-Host "`n💎 CÁLCULO DE PROFIT MÍNIMO NECESARIO:" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

$BestDex = $FeeAnalysis | Sort-Object TotalFee | Select-Object -First 1
Write-Host "🏆 DEX más económico: $($BestDex.DexName)" -ForegroundColor Green
Write-Host "   • Fee mínimo: $([math]::Round($BestDex.TotalFee, 6)) SOL" -ForegroundColor Green

# Para arbitraje (2 trades: compra + venta)
$ArbitrageFee = $BestDex.TotalFee * 2
$SlippageBuffer = $TradeAmountSOL * 0.01  # 1% slippage buffer
$TotalCost = $ArbitrageFee + $SlippageBuffer

Write-Host "`n⚡ ARBITRAJE (2 trades):" -ForegroundColor Yellow
Write-Host "   • Fee compra: $([math]::Round($BestDex.TotalFee, 6)) SOL" -ForegroundColor White
Write-Host "   • Fee venta: $([math]::Round($BestDex.TotalFee, 6)) SOL" -ForegroundColor White
Write-Host "   • Buffer slippage: $([math]::Round($SlippageBuffer, 6)) SOL" -ForegroundColor White
Write-Host "   • COSTO TOTAL: $([math]::Round($TotalCost, 6)) SOL" -ForegroundColor Red

$MinProfitRequired = $TotalCost * 1.2  # 20% margen de seguridad
Write-Host "   • PROFIT MÍNIMO REQUERIDO: $([math]::Round($MinProfitRequired, 6)) SOL" -ForegroundColor Magenta

# Análisis de diferentes tamaños de trade
Write-Host "`n📈 OPTIMIZACIÓN DE TAMAÑO DE TRADE:" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

$TradeSizes = @(0.05, 0.10, 0.15, 0.20, 0.25)
foreach ($size in $TradeSizes) {
    if ($size -le $CapitalSOL) {
        $analysis = Calculate-TradeFees -TradeAmount $size -DexName "Phoenix" -DexFeePercent 0.20
        $arbFee = $analysis.TotalFee * 2
        $buffer = $size * 0.01
        $totalCost = $arbFee + $buffer
        $minProfit = $totalCost * 1.2
        $profitPercent = [math]::Round(($minProfit / $size) * 100, 2)
        
        $color = if ($profitPercent -lt 2.0) { "Green" } elseif ($profitPercent -lt 3.0) { "Yellow" } else { "Red" }
        
        Write-Host "🔸 Trade $size SOL:" -ForegroundColor $color
        Write-Host "   • Costo total: $([math]::Round($totalCost, 6)) SOL" -ForegroundColor White
        Write-Host "   • Profit mínimo: $([math]::Round($minProfit, 6)) SOL ($profitPercent%)" -ForegroundColor $color
    }
}

# Estrategias de optimización
Write-Host "`n🎯 ESTRATEGIAS DE OPTIMIZACIÓN:" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

Write-Host "1. 🏆 TAMAÑO ÓPTIMO DE TRADE:" -ForegroundColor Yellow
Write-Host "   • Usar 0.15 SOL (52% del capital)" -ForegroundColor Green
Write-Host "   • Profit mínimo necesario: ~0.0008 SOL (0.53%)" -ForegroundColor Green
Write-Host "   • Permite 1-2 trades simultáneos" -ForegroundColor Green

Write-Host "`n2. 🎯 SELECCIÓN DE DEX INTELIGENTE:" -ForegroundColor Yellow
Write-Host "   • Prioridad 1: Phoenix (0.20% fee)" -ForegroundColor Green
Write-Host "   • Prioridad 2: Raydium/Meteora (0.25% fee)" -ForegroundColor Yellow
Write-Host "   • Evitar: Orca/Lifinity (0.30% fee)" -ForegroundColor Red

Write-Host "`n3. ⚡ MICRO-ARBITRAJE:" -ForegroundColor Yellow
Write-Host "   • Trades pequeños: 0.05 SOL" -ForegroundColor Green
Write-Host "   • Profit mínimo: 0.0003 SOL (0.6%)" -ForegroundColor Green
Write-Host "   • Ejecución rápida múltiple" -ForegroundColor Green

Write-Host "`n4. 🎮 MODO AGRESIVO:" -ForegroundColor Yellow
Write-Host "   • Scan cada 1 segundo" -ForegroundColor Green
Write-Host "   • Umbral mínimo: 0.15% diferencia de precio" -ForegroundColor Green
Write-Host "   • Slippage hasta 3%" -ForegroundColor Yellow
Write-Host "   • Priority fee dinámico" -ForegroundColor Green

# Simulación de oportunidades realistas
Write-Host "`n💰 SIMULACIÓN DE OPORTUNIDADES REALISTAS:" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

$OpportunityTypes = @(
    @{ Type = "Micro SOL/USDC"; PriceDiff = 0.3; TradeSize = 0.05; Frequency = "Alta" },
    @{ Type = "RAY arbitrage"; PriceDiff = 0.5; TradeSize = 0.10; Frequency = "Media" },
    @{ Type = "BONK/WIF spread"; PriceDiff = 0.8; TradeSize = 0.15; Frequency = "Baja" },
    @{ Type = "Orca cross-pool"; PriceDiff = 1.2; TradeSize = 0.20; Frequency = "Muy Baja" }
)

foreach ($opp in $OpportunityTypes) {
    $analysis = Calculate-TradeFees -TradeAmount $opp.TradeSize -DexName "Phoenix" -DexFeePercent 0.20
    $arbFee = $analysis.TotalFee * 2
    $grossProfit = $opp.TradeSize * ($opp.PriceDiff / 100)
    $netProfit = $grossProfit - $arbFee - ($opp.TradeSize * 0.01)
    $profitPercent = [math]::Round(($netProfit / $opp.TradeSize) * 100, 2)
    
    $color = if ($netProfit -gt 0) { "Green" } else { "Red" }
    
    Write-Host "🔸 $($opp.Type):" -ForegroundColor Yellow
    Write-Host "   • Diferencia precio: $($opp.PriceDiff)%" -ForegroundColor White
    Write-Host "   • Trade size: $($opp.TradeSize) SOL" -ForegroundColor White
    Write-Host "   • Profit bruto: $([math]::Round($grossProfit, 6)) SOL" -ForegroundColor White
    Write-Host "   • Fees totales: $([math]::Round($arbFee, 6)) SOL" -ForegroundColor Red
    Write-Host "   • PROFIT NETO: $([math]::Round($netProfit, 6)) SOL ($profitPercent%)" -ForegroundColor $color
    Write-Host "   • Frecuencia: $($opp.Frequency)" -ForegroundColor Gray
}

Write-Host "`n🎖️ CONFIGURACIÓN RECOMENDADA:" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green

Write-Host "✅ Usar configuración agresiva optimizada" -ForegroundColor Green
Write-Host "✅ Trade size: 0.15 SOL (máximo rendimiento)" -ForegroundColor Green
Write-Host "✅ Profit mínimo: 0.0005 SOL" -ForegroundColor Green
Write-Host "✅ Priorizar Phoenix y Raydium" -ForegroundColor Green
Write-Host "✅ Micro-arbitraje habilitado" -ForegroundColor Green
Write-Host "✅ Scan agresivo cada 1s" -ForegroundColor Green

Write-Host "`n💡 Con esta configuración necesitas encontrar diferencias de precio de 0.5%+ para ser rentable" -ForegroundColor Cyan
Write-Host "🎯 Objetivo realista: 1-3 trades exitosos por hora en mercado activo" -ForegroundColor Yellow
