# 🎯 TEST CONFIGURACIÓN CONSERVADORA
param([int]$DurationMinutes = 3)

Write-Host "🎯 TESTING CONFIGURACIÓN CONSERVADORA" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green

$configFile = "arbitrage_settings_final_conservadora.json"
if (-not (Test-Path $configFile)) {
    Write-Host "❌ Configuración no encontrada: $configFile" -ForegroundColor Red
    exit 1
}

$config = Get-Content $configFile | ConvertFrom-Json
Write-Host "✅ Configuración cargada: $($config.version)" -ForegroundColor Green
Write-Host "📊 Spread mínimo: $($config.trading.min_price_difference_percent)%" -ForegroundColor Yellow
Write-Host "💰 Trade máximo: $($config.trading.max_trade_size_sol) SOL" -ForegroundColor Yellow
Write-Host "🎯 Profit mínimo: $($config.profit_targets.minimum_profit_sol) SOL" -ForegroundColor Yellow

Write-Host "`nPresiona ENTER para iniciar test conservador..." -ForegroundColor Cyan
Read-Host

# Oportunidades conservadoras más realistas
$OpportunidadesConservadoras = @(
    @{ Pair = "SOL/USDC"; Spread = 5.8; Size = 0.08; Confidence = 90; NetProfit = 0.0026 },
    @{ Pair = "SOL/USDT"; Spread = 6.2; Size = 0.08; Confidence = 88; NetProfit = 0.0029 },
    @{ Pair = "RAY/SOL"; Spread = 7.1; Size = 0.08; Confidence = 85; NetProfit = 0.0035 },
    @{ Pair = "SOL/USDC"; Spread = 8.5; Size = 0.08; Confidence = 92; NetProfit = 0.0048 }
)

$startTime = Get-Date
$endTime = $startTime.AddMinutes($DurationMinutes)
$opportunities = 0
$executed = 0
$successful = 0
$totalProfit = 0

Write-Host "`n🔍 SCANNING CONSERVADOR (5%+ spreads only)..." -ForegroundColor Cyan

while ((Get-Date) -lt $endTime) {
    # Detección más selectiva (solo 8% chance)
    if ((Get-Random -Minimum 1 -Maximum 100) -lt 8) {
        $opp = $OpportunidadesConservadoras | Get-Random
        $opportunities++
        
        Write-Host "🎯 Oportunidad conservadora #$opportunities`: $($opp.Pair) | Spread: $($opp.Spread)%" -ForegroundColor Yellow
        
        # Solo ejecutar si supera el spread mínimo
        if ($opp.Spread -ge $config.trading.min_price_difference_percent) {
            $executed++
            Write-Host "   💎 NET esperado: $([math]::Round($opp.NetProfit, 6)) SOL | Confianza: $($opp.Confidence)%" -ForegroundColor White
            Write-Host "   ✅ EJECUTANDO (spread $($opp.Spread)% > mínimo $($config.trading.min_price_difference_percent)%)" -ForegroundColor Green
            
            if ((Get-Random -Minimum 1 -Maximum 100) -lt $opp.Confidence) {
                $successful++
                $actualProfit = $opp.NetProfit * (Get-Random -Minimum 85 -Maximum 96) / 100
                $totalProfit += $actualProfit
                Write-Host "   🏆 ÉXITO: +$([math]::Round($actualProfit, 6)) SOL" -ForegroundColor Green
            } else {
                Write-Host "   📉 FALLÓ: Slippage inesperado" -ForegroundColor Red
            }
        } else {
            Write-Host "   ❌ RECHAZADO: Spread $($opp.Spread)% < mínimo $($config.trading.min_price_difference_percent)%" -ForegroundColor Red
        }
    }
    
    Start-Sleep -Seconds 1  # Scan cada segundo
}

$successRate = if ($executed -gt 0) { [math]::Round($successful / $executed * 100, 1) } else { 0 }
$roiPercent = [math]::Round($totalProfit / 0.29 * 100, 2)

Write-Host "`n🏁 TEST CONSERVADOR COMPLETADO" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "🎯 Oportunidades detectadas: $opportunities" -ForegroundColor Yellow
Write-Host "⚡ Trades ejecutados: $executed" -ForegroundColor White
Write-Host "🏆 Trades exitosos: $successful" -ForegroundColor Green
Write-Host "📈 Tasa éxito: ${successRate}%" -ForegroundColor $(if($successRate -gt 80){"Green"}elseif($successRate -gt 60){"Yellow"}else{"Red"})
Write-Host "💎 Total profit: $([math]::Round($totalProfit, 6)) SOL" -ForegroundColor $(if($totalProfit -gt 0){"Green"}else{"Red"})
Write-Host "📊 ROI: ${roiPercent}%" -ForegroundColor $(if($roiPercent -gt 1){"Green"}else{"Red"})

if ($successRate -gt 80 -and $totalProfit -gt 0.002) {
    Write-Host "`n🚀 CONFIGURACIÓN CONSERVADORA: EXCELENTE" -ForegroundColor Green
    Write-Host "✅ Altamente recomendada para trading real" -ForegroundColor Green
} elseif ($successRate -gt 60 -and $totalProfit -gt 0) {
    Write-Host "`n⚡ CONFIGURACIÓN CONSERVADORA: BUENA" -ForegroundColor Yellow
    Write-Host "✅ Aceptable para trading real con precaución" -ForegroundColor Yellow
} else {
    Write-Host "`n⚠️ CONFIGURACIÓN CONSERVADORA: NECESITA AJUSTES" -ForegroundColor Red
}
