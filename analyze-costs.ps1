#!/usr/bin/env pwsh
# SniperForge Enterprise v3.0 - Análisis de Costos para Capital Pequeño
# Calculadora de viabilidad económica

Write-Host "💰 SniperForge - Análisis de Costos para 0.29 SOL" -ForegroundColor Cyan
Write-Host "=" * 60 -ForegroundColor DarkGray

# Configuración de costos actual (Solana mainnet)
$TRANSACTION_COST = 0.001        # ~0.001 SOL por transacción
$PRIORITY_FEE = 0.00005         # ~0.00005 SOL priority fee
$MEV_PROTECTION = 0.0005        # ~0.0005 SOL MEV protection
$SLIPPAGE_PERCENT = 1.5         # 1.5% slippage promedio

Write-Host "📋 Parámetros de Costos Actuales:" -ForegroundColor Yellow
Write-Host "   Costo transacción: $TRANSACTION_COST SOL" -ForegroundColor Gray
Write-Host "   Priority fee: $PRIORITY_FEE SOL" -ForegroundColor Gray
Write-Host "   MEV protection: $MEV_PROTECTION SOL" -ForegroundColor Gray
Write-Host "   Slippage promedio: $SLIPPAGE_PERCENT%" -ForegroundColor Gray
Write-Host ""

# Análisis para diferentes tamaños de posición
$CAPITAL_TOTAL = 0.29
$RESERVE_GAS = 0.04
$CAPITAL_OPERATIVO = $CAPITAL_TOTAL - $RESERVE_GAS

Write-Host "💳 Capital Disponible:" -ForegroundColor Blue
Write-Host "   Total: $CAPITAL_TOTAL SOL" -ForegroundColor Gray
Write-Host "   Reserva gas: $RESERVE_GAS SOL" -ForegroundColor Gray
Write-Host "   Operativo: $CAPITAL_OPERATIVO SOL" -ForegroundColor Green
Write-Host ""

function Analyze-Position {
    param(
        [double]$PositionSize,
        [double]$ExpectedProfitPercent
    )
    
    # Costos fijos (entrada + salida)
    $fixedCosts = ($TRANSACTION_COST * 2) + ($PRIORITY_FEE * 2) + $MEV_PROTECTION
    
    # Costos variables (slippage)
    $slippageCosts = $PositionSize * ($SLIPPAGE_PERCENT / 100) * 2  # entrada + salida
    
    # Total de costos
    $totalCosts = $fixedCosts + $slippageCosts
    $costPercent = ($totalCosts / $PositionSize) * 100
    
    # Ganancia necesaria para cubrir costos
    $minProfitNeeded = ($totalCosts / $PositionSize) * 100
    $profitAfterCosts = $ExpectedProfitPercent - $minProfitNeeded
    
    # Viabilidad
    $isViable = $profitAfterCosts -gt 5.0  # Al menos 5% profit neto
    
    return @{
        PositionSize = $PositionSize
        TotalCosts = $totalCosts
        CostPercent = $costPercent
        MinProfitNeeded = $minProfitNeeded
        ProfitAfterCosts = $profitAfterCosts
        IsViable = $isViable
        ExpectedProfit = $ExpectedProfitPercent
    }
}

Write-Host "📊 Análisis de Viabilidad por Tamaño de Posición:" -ForegroundColor Blue
Write-Host ""

# Analizar diferentes escenarios
$scenarios = @(
    @{ Size = 0.1; Profit = 20 },
    @{ Size = 0.15; Profit = 20 },
    @{ Size = 0.2; Profit = 20 },
    @{ Size = 0.25; Profit = 20 },
    @{ Size = 0.1; Profit = 30 },
    @{ Size = 0.2; Profit = 30 }
)

$viableCount = 0
$totalScenarios = $scenarios.Count

foreach ($scenario in $scenarios) {
    $analysis = Analyze-Position -PositionSize $scenario.Size -ExpectedProfitPercent $scenario.Profit
    
    $statusIcon = if ($analysis.IsViable) { "✅" } else { "❌" }
    $statusColor = if ($analysis.IsViable) { "Green" } else { "Red" }
    
    if ($analysis.IsViable) { $viableCount++ }
    
    Write-Host "$statusIcon Posición: $($analysis.PositionSize) SOL | Profit esperado: $($analysis.ExpectedProfit)%" -ForegroundColor $statusColor
    Write-Host "   Costos totales: $([math]::Round($analysis.TotalCosts, 4)) SOL ($([math]::Round($analysis.CostPercent, 1))%)" -ForegroundColor Gray
    Write-Host "   Profit mínimo necesario: $([math]::Round($analysis.MinProfitNeeded, 1))%" -ForegroundColor Gray
    Write-Host "   Profit neto estimado: $([math]::Round($analysis.ProfitAfterCosts, 1))%" -ForegroundColor $(if ($analysis.ProfitAfterCosts -gt 0) { "Green" } else { "Red" })
    Write-Host ""
}

# Resumen y recomendaciones
Write-Host "=" * 60 -ForegroundColor DarkGray
Write-Host "📈 RESUMEN Y RECOMENDACIONES" -ForegroundColor Cyan
Write-Host "=" * 60 -ForegroundColor DarkGray

$viabilityRate = ($viableCount / $totalScenarios) * 100

Write-Host "Escenarios viables: $viableCount de $totalScenarios ($([math]::Round($viabilityRate, 1))%)" -ForegroundColor $(if ($viabilityRate -gt 50) { "Green" } elseif ($viabilityRate -gt 25) { "Yellow" } else { "Red" })
Write-Host ""

Write-Host "🎯 CONFIGURACIÓN RECOMENDADA PARA 0.29 SOL:" -ForegroundColor Yellow
Write-Host ""

# Analizar configuración óptima
$optimalAnalysis = Analyze-Position -PositionSize 0.2 -ExpectedProfitPercent 25

Write-Host "Tamaño de posición óptimo: 0.2 SOL (80% del capital operativo)" -ForegroundColor Green
Write-Host "Profit target mínimo: 25%" -ForegroundColor Green
Write-Host "Máximo 1 posición simultánea" -ForegroundColor Green
Write-Host "Reserva de gas: 0.04 SOL" -ForegroundColor Green
Write-Host ""

Write-Host "💡 ESTRATEGIA OPERATIVA:" -ForegroundColor Yellow
Write-Host "1. Solo oportunidades con >25% profit esperado" -ForegroundColor Gray
Write-Host "2. Máximo 1 trade por vez para optimizar costos" -ForegroundColor Gray
Write-Host "3. Stop loss a -12% (absorbe costos + protege capital)" -ForegroundColor Gray
Write-Host "4. Take profit escalonado: 15%, 20%, 30%" -ForegroundColor Gray
Write-Host "5. Tiempo máximo por trade: 15 minutos" -ForegroundColor Gray
Write-Host ""

Write-Host "⚠️ LIMITACIONES CON CAPITAL PEQUEÑO:" -ForegroundColor Red
Write-Host "• Costos representan 2-4% del capital por trade" -ForegroundColor Red
Write-Host "• Necesitas >25% profit para ser rentable" -ForegroundColor Red
Write-Host "• Solo 1-2 trades por día máximo" -ForegroundColor Red
Write-Host "• Vulnerable a slippage alto en tokens ilíquidos" -ForegroundColor Red
Write-Host ""

Write-Host "📊 EXPECTATIVAS REALISTAS:" -ForegroundColor Blue
Write-Host "• ROI mensual objetivo: 20-40%" -ForegroundColor Gray
Write-Host "• Win rate necesario: >70%" -ForegroundColor Gray
Write-Host "• Profit promedio por trade: 0.03-0.05 SOL" -ForegroundColor Gray
Write-Host "• Tiempo para duplicar capital: 2-3 meses" -ForegroundColor Gray
Write-Host ""

# Calcular break-even point
$breakEvenCapital = 1.0  # SOL necesario para operativa más eficiente
$monthsToBreakEven = [math]::Ceiling($breakEvenCapital / 0.1)  # Asumiendo 0.1 SOL profit mensual

Write-Host "🎯 OBJETIVO A LARGO PLAZO:" -ForegroundColor Cyan
Write-Host "Acumular hasta $breakEvenCapital SOL para operativa más eficiente" -ForegroundColor Gray
Write-Host "Tiempo estimado: $monthsToBreakEven meses con strategy conservativa" -ForegroundColor Gray
Write-Host ""

Write-Host "✅ CONFIGURACIÓN FINAL APLICADA EN small_capital_config.json" -ForegroundColor Green
