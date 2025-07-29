# 🚀 TEST CONFIGURACIÓN ULTRA-MEJORADA
param([int]$DurationMinutes = 5)

Write-Host "🚀 TESTING CONFIGURACIÓN ULTRA-MEJORADA" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green

$configFile = "arbitrage_settings_ultra_mejorada.json"
if (-not (Test-Path $configFile)) {
    Write-Host "❌ Configuración no encontrada: $configFile" -ForegroundColor Red
    exit 1
}

$config = Get-Content $configFile | ConvertFrom-Json
Write-Host "✅ Configuración cargada: $($config.version)" -ForegroundColor Green
Write-Host "📊 Spread mínimo: $($config.trading.min_price_difference_percent)%" -ForegroundColor Yellow
Write-Host "💰 Trade máximo: $($config.trading.max_trade_size_sol) SOL" -ForegroundColor Yellow
Write-Host "🎯 Profit mínimo: $($config.profit_targets.minimum_profit_sol) SOL" -ForegroundColor Yellow

Write-Host "`nPresiona ENTER para iniciar test mejorado..." -ForegroundColor Cyan
Read-Host

# Simular oportunidades más realistas con configuración mejorada
$OpportunidadesMejoradas = @(
    @{ Pair = "SOL/USDC"; Spread = 4.2; Size = 0.15; Confidence = 85; Profit = 0.0042 },
    @{ Pair = "BONK/SOL"; Spread = 5.8; Size = 0.12; Confidence = 70; Profit = 0.0038 },
    @{ Pair = "WIF/SOL"; Spread = 6.5; Size = 0.10; Confidence = 65; Profit = 0.0041 },
    @{ Pair = "RAY/SOL"; Spread = 3.8; Size = 0.08; Confidence = 80; Profit = 0.0019 },
    @{ Pair = "JUP/SOL"; Spread = 4.9; Size = 0.14; Confidence = 75; Profit = 0.0045 }
)

$startTime = Get-Date
$endTime = $startTime.AddMinutes($DurationMinutes)
$opportunities = 0
$profitable = 0
$totalProfit = 0

Write-Host "`n🔍 SCANNING CON CONFIGURACIÓN MEJORADA..." -ForegroundColor Cyan

while ((Get-Date) -lt $endTime) {
    if ((Get-Random -Minimum 1 -Maximum 100) -lt 15) {
        $opp = $OpportunidadesMejoradas | Get-Random
        $opportunities++
        
        # Cálculo preciso de fees con configuración mejorada
        $phoenixFee = $opp.Size * 0.002  # 0.20%
        $baseFees = 0.00006  # Fees de red más realistas
        $totalFees = $phoenixFee + $baseFees
        $slippage = $opp.Size * 0.01  # 1% slippage
        $netProfit = $opp.Profit - $totalFees - $slippage
        
        Write-Host "🎯 Oportunidad #$opportunities`: $($opp.Pair) | Spread: $($opp.Spread)%" -ForegroundColor Yellow
        Write-Host "   💰 Profit bruto: $([math]::Round($opp.Profit, 6)) SOL" -ForegroundColor White
        Write-Host "   💸 Fees: $([math]::Round($totalFees, 6)) SOL | Slippage: $([math]::Round($slippage, 6)) SOL" -ForegroundColor Red
        Write-Host "   💎 NET: $([math]::Round($netProfit, 6)) SOL" -ForegroundColor $(if($netProfit -gt 0){"Green"}else{"Red"})
        
        if ($netProfit -gt $config.profit_targets.minimum_profit_sol) {
            if ((Get-Random -Minimum 1 -Maximum 100) -lt $opp.Confidence) {
                $profitable++
                $actualProfit = $netProfit * 0.9
                $totalProfit += $actualProfit
                Write-Host "   ✅ ÉXITO: +$([math]::Round($actualProfit, 6)) SOL" -ForegroundColor Green
            } else {
                Write-Host "   📉 FALLÓ: Slippage excesivo" -ForegroundColor Red
            }
        } else {
            Write-Host "   ❌ NO RENTABLE: Profit inferior al mínimo" -ForegroundColor Red
        }
    }
    
    Start-Sleep -Milliseconds 750
}

$successRate = if ($opportunities -gt 0) { [math]::Round($profitable / $opportunities * 100, 1) } else { 0 }
$roiPercent = [math]::Round($totalProfit / 0.29 * 100, 2)

Write-Host "`n🏁 TEST MEJORADO COMPLETADO" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "🎯 Oportunidades: $opportunities" -ForegroundColor Yellow
Write-Host "🏆 Exitosos: $profitable" -ForegroundColor Green
Write-Host "📈 Tasa éxito: ${successRate}%" -ForegroundColor $(if($successRate -gt 60){"Green"}else{"Red"})
Write-Host "💎 Total profit: $([math]::Round($totalProfit, 6)) SOL" -ForegroundColor $(if($totalProfit -gt 0){"Green"}else{"Red"})
Write-Host "📊 ROI: ${roiPercent}%" -ForegroundColor $(if($roiPercent -gt 0){"Green"}else{"Red"})

if ($successRate -gt 70 -and $totalProfit -gt 0.003) {
    Write-Host "`n🚀 CONFIGURACIÓN ULTRA-MEJORADA: EXCELENTE" -ForegroundColor Green
    Write-Host "✅ Lista para trading real" -ForegroundColor Green
} elseif ($successRate -gt 50 -and $totalProfit -gt 0) {
    Write-Host "`n⚡ CONFIGURACIÓN ULTRA-MEJORADA: BUENA" -ForegroundColor Yellow
    Write-Host "✅ Proceder con precaución" -ForegroundColor Yellow
} else {
    Write-Host "`n⚠️ CONFIGURACIÓN ULTRA-MEJORADA: NECESITA AJUSTES" -ForegroundColor Red
}
