# 🔍 DETECTOR DE OPORTUNIDADES REALES - PRECIOS EN TIEMPO REAL
# Script para detectar oportunidades de arbitraje usando APIs públicas

Write-Host "🔍 DETECTOR DE OPORTUNIDADES DE ARBITRAJE REALES" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green

function Get-DexScreenerPrice {
    param([string]$TokenAddress)
    
    try {
        $url = "https://api.dexscreener.com/latest/dex/tokens/$TokenAddress"
        $response = Invoke-RestMethod -Uri $url -Method GET -TimeoutSec 10
        
        if ($response.pairs -and $response.pairs.Count -gt 0) {
            # Filtrar pares con volumen significativo
            $validPairs = $response.pairs | Where-Object { 
                $_.volume.h24 -gt 10000 -and $_.liquidity.usd -gt 50000 
            } | Sort-Object { [double]$_.volume.h24 } -Descending
            
            if ($validPairs.Count -gt 0) {
                return $validPairs[0..2]  # Top 3 pares por volumen
            }
        }
        return $null
    } catch {
        Write-Host "⚠️ Error obteniendo precio de DexScreener: $($_.Exception.Message)" -ForegroundColor Yellow
        return $null
    }
}

function Get-CoinGeckoPrice {
    param([string]$TokenId)
    
    try {
        $url = "https://api.coingecko.com/api/v3/simple/price?ids=$TokenId&vs_currencies=usd"
        $response = Invoke-RestMethod -Uri $url -Method GET -TimeoutSec 10
        return $response.$TokenId.usd
    } catch {
        Write-Host "⚠️ Error obteniendo precio de CoinGecko: $($_.Exception.Message)" -ForegroundColor Yellow
        return $null
    }
}

function Test-RealArbitrageOpportunity {
    param(
        [string]$TokenName,
        [string]$TokenAddress,
        [double]$TradeAmountSOL = 0.05
    )
    
    Write-Host "`n🔍 Analizando $TokenName..." -ForegroundColor Cyan
    Write-Host "══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
    
    # Obtener precios de diferentes DEXes
    $dexPairs = Get-DexScreenerPrice -TokenAddress $TokenAddress
    
    if (-not $dexPairs -or $dexPairs.Count -lt 2) {
        Write-Host "❌ No se encontraron suficientes pares líquidos para $TokenName" -ForegroundColor Red
        return $null
    }
    
    # Analizar diferencias de precio entre DEXes
    $opportunities = @()
    
    for ($i = 0; $i -lt $dexPairs.Count - 1; $i++) {
        for ($j = $i + 1; $j -lt $dexPairs.Count; $j++) {
            $pair1 = $dexPairs[$i]
            $pair2 = $dexPairs[$j]
            
            $price1 = [double]$pair1.priceUsd
            $price2 = [double]$pair2.priceUsd
            
            if ($price1 -gt 0 -and $price2 -gt 0) {
                $priceDiff = [math]::Abs($price1 - $price2)
                $percentDiff = ($priceDiff / [math]::Min($price1, $price2)) * 100
                
                if ($percentDiff -gt 1.0) {  # Mínimo 1% diferencia
                    $buyDex = if ($price1 -lt $price2) { $pair1.dexId } else { $pair2.dexId }
                    $sellDex = if ($price1 -lt $price2) { $pair2.dexId } else { $pair1.dexId }
                    $buyPrice = [math]::Min($price1, $price2)
                    $sellPrice = [math]::Max($price1, $price2)
                    
                    # Calcular profit estimado
                    $tokensToBuy = $TradeAmountSOL * 200  # Asumiendo SOL ≈ $200
                    $tokensReceived = $tokensToBy / $buyPrice
                    $solReceived = ($tokensReceived * $sellPrice) / 200
                    $grossProfit = $solReceived - $TradeAmountSOL
                    
                    # Estimar fees (conservador)
                    $estimatedFees = $TradeAmountSOL * 0.01  # 1% total fees
                    $netProfit = $grossProfit - $estimatedFees
                    
                    $opportunity = @{
                        Token = $TokenName
                        BuyDex = $buyDex
                        SellDex = $sellDex
                        BuyPrice = $buyPrice
                        SellPrice = $sellPrice
                        PriceDiff = $percentDiff
                        GrossProfit = $grossProfit
                        NetProfit = $netProfit
                        Volume24h = [math]::Min([double]$pair1.volume.h24, [double]$pair2.volume.h24)
                        Liquidity = [math]::Min([double]$pair1.liquidity.usd, [double]$pair2.liquidity.usd)
                    }
                    
                    $opportunities += $opportunity
                }
            }
        }
    }
    
    # Mostrar oportunidades encontradas
    if ($opportunities.Count -gt 0) {
        $bestOpp = $opportunities | Sort-Object NetProfit -Descending | Select-Object -First 1
        
        Write-Host "🎯 OPORTUNIDAD DETECTADA:" -ForegroundColor Yellow
        Write-Host "💱 Token: $($bestOpp.Token)" -ForegroundColor White
        Write-Host "🛒 Comprar en: $($bestOpp.BuyDex) a $($bestOpp.BuyPrice) USD" -ForegroundColor Green
        Write-Host "💰 Vender en: $($bestOpp.SellDex) a $($bestOpp.SellPrice) USD" -ForegroundColor Green
        Write-Host "📊 Diferencia: $([math]::Round($bestOpp.PriceDiff, 2))%" -ForegroundColor Cyan
        Write-Host "💎 Profit bruto: $([math]::Round($bestOpp.GrossProfit, 4)) SOL" -ForegroundColor Green
        Write-Host "💸 Fees estimados: $([math]::Round($estimatedFees, 4)) SOL" -ForegroundColor Red
        Write-Host "🏆 Profit neto: $([math]::Round($bestOpp.NetProfit, 4)) SOL" -ForegroundColor $(if($bestOpp.NetProfit -gt 0){"Green"}else{"Red"})
        Write-Host "📈 Volumen 24h: $([math]::Round($bestOpp.Volume24h / 1000, 0))K USD" -ForegroundColor Gray
        Write-Host "💧 Liquidez: $([math]::Round($bestOpp.Liquidity / 1000, 0))K USD" -ForegroundColor Gray
        
        return $bestOpp
    } else {
        Write-Host "❌ No se encontraron oportunidades rentables para $TokenName" -ForegroundColor Red
        return $null
    }
}

# Tokens principales para análisis
$tokensToAnalyze = @(
    @{ Name = "SOL"; Address = "So11111111111111111111111111111111111111112" },
    @{ Name = "BONK"; Address = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263" },
    @{ Name = "WIF"; Address = "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm" },
    @{ Name = "RAY"; Address = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R" },
    @{ Name = "JUP"; Address = "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN" }
)

Write-Host "🚀 Iniciando análisis de oportunidades reales..." -ForegroundColor Green
Write-Host "⏱️ Hora de análisis: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')" -ForegroundColor Gray

$realOpportunities = @()

foreach ($token in $tokensToAnalyze) {
    try {
        $opportunity = Test-RealArbitrageOpportunity -TokenName $token.Name -TokenAddress $token.Address
        if ($opportunity) {
            $realOpportunities += $opportunity
        }
        Start-Sleep -Milliseconds 1000  # Rate limiting
    } catch {
        Write-Host "❌ Error analizando $($token.Name): $($_.Exception.Message)" -ForegroundColor Red
    }
}

# Resumen final
Write-Host "`n📊 RESUMEN DE OPORTUNIDADES REALES:" -ForegroundColor Cyan
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

if ($realOpportunities.Count -gt 0) {
    $bestOverall = $realOpportunities | Sort-Object NetProfit -Descending | Select-Object -First 3
    
    Write-Host "🏆 TOP 3 OPORTUNIDADES:" -ForegroundColor Green
    for ($i = 0; $i -lt $bestOverall.Count; $i++) {
        $opp = $bestOverall[$i]
        Write-Host "$($i+1). $($opp.Token): $($opp.BuyDex) → $($opp.SellDex)" -ForegroundColor White
        Write-Host "   📊 Diff: $([math]::Round($opp.PriceDiff, 2))% | Profit: $([math]::Round($opp.NetProfit, 4)) SOL" -ForegroundColor $(if($opp.NetProfit -gt 0){"Green"}else{"Red"})
    }
    
    $profitableCount = ($realOpportunities | Where-Object { $_.NetProfit -gt 0 }).Count
    Write-Host "`n✅ Oportunidades rentables encontradas: $profitableCount de $($realOpportunities.Count)" -ForegroundColor Green
    
    if ($profitableCount -gt 0) {
        Write-Host "🚀 Hay oportunidades reales de arbitraje disponibles!" -ForegroundColor Green
        Write-Host "💡 Para ejecutar trades reales, necesitarías:" -ForegroundColor Yellow
        Write-Host "   🔸 Integración con DEX APIs específicos" -ForegroundColor White
        Write-Host "   🔸 Manejo de wallets y transacciones" -ForegroundColor White
        Write-Host "   🔸 Gestión de slippage y MEV" -ForegroundColor White
    }
} else {
    Write-Host "❌ No se encontraron oportunidades rentables en este momento" -ForegroundColor Red
    Write-Host "💡 Esto es normal - el arbitraje requiere condiciones específicas de mercado" -ForegroundColor Yellow
}

Write-Host "`n⏰ Siguiente análisis recomendado en 15-30 minutos" -ForegroundColor Gray
Write-Host "🎯 Mejores horarios: 14:00-16:00 UTC y 21:00-23:00 UTC" -ForegroundColor Gray
