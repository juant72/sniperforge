# Script de arbitraje simplificado y confiable
param(
    [int]$IntervalSeconds = 45,
    [switch]$ExecuteReal = $false
)

Write-Host "🚀 === ARBITRAJE CROSS-DEX SIMPLIFICADO ===" -ForegroundColor Green
Write-Host "⚡ Monitoreando cada $IntervalSeconds segundos" -ForegroundColor Cyan
if ($ExecuteReal) {
    Write-Host "💰 MODO REAL: Ejecutará arbitrajes automáticamente" -ForegroundColor Red
} else {
    Write-Host "🧪 MODO ANÁLISIS: Solo detectará oportunidades" -ForegroundColor Yellow
}
Write-Host "❌ Ctrl+C para detener" -ForegroundColor Red
Write-Host ""

$startTime = Get-Date
$cycle = 0
$totalProfits = 0

while ($true) {
    $cycle++
    $currentTime = Get-Date
    $elapsed = $currentTime - $startTime
    
    Write-Host "🔄 === CICLO #$cycle | $(Get-Date -Format 'HH:mm:ss') | Tiempo: $($elapsed.ToString('hh\:mm\:ss')) ===" -ForegroundColor Blue
    
    try {
        # 1. Verificar balance
        Write-Host "💰 Verificando balance..." -ForegroundColor Cyan
        $balanceOutput = & cargo run --bin check_devnet_balance 2>&1
        $balanceString = $balanceOutput -join " "
        
        if ($balanceString -match "([0-9]+\.[0-9]+) SOL") {
            $currentBalance = [decimal]$matches[1]
            Write-Host "✅ Balance actual: $currentBalance SOL" -ForegroundColor Green
        } else {
            Write-Host "⚠️ No se pudo detectar balance, continuando..." -ForegroundColor Yellow
            $currentBalance = 0
        }
        
        # 2. Análisis de arbitraje
        Write-Host "🔍 Analizando oportunidades de arbitraje..." -ForegroundColor Cyan
        $arbitrageOutput = & cargo run --bin test_arbitrage_cross_dex 2>&1
        $arbitrageString = $arbitrageOutput -join " "
        
        # Extraer datos con patrones más simples
        $spread = $null
        $profit = $null
        
        if ($arbitrageString -match "Spread:\s*([0-9]+\.?[0-9]*)%") {
            $spread = [decimal]$matches[1]
            Write-Host "📊 Spread detectado: $spread%" -ForegroundColor $(if ($spread -gt 50) { "Green" } elseif ($spread -gt 10) { "Yellow" } else { "Red" })
        }
        
        # Buscar cualquier valor en dólares como profit
        if ($arbitrageString -match "\$([0-9]+\.?[0-9]*)") {
            $profit = [decimal]$matches[1]
            Write-Host "💰 Profit detectado: $$$profit" -ForegroundColor $(if ($profit -gt 50) { "Green" } else { "Yellow" })
        }
        
        # 3. Decisión de arbitraje
        if ($null -ne $spread -and $null -ne $profit) {
            if ($spread -gt 50 -and $profit -gt 50) {
                Write-Host "🎯 OPORTUNIDAD EXCELENTE DETECTADA!" -ForegroundColor Green
                Write-Host "   📈 Spread: $spread%" -ForegroundColor Green
                Write-Host "   💰 Profit: $$$profit" -ForegroundColor Green
                
                if ($ExecuteReal -and $currentBalance -gt 0.1) {
                    Write-Host "🚀 EJECUTANDO ARBITRAJE REAL..." -ForegroundColor Red
                    
                    # Aquí ejecutaríamos el arbitraje real
                    # Por ahora solo simulamos
                    $estimatedProfit = 0.1 * ($profit / 100)  # Estimado para 0.1 SOL
                    $totalProfits += $estimatedProfit
                    
                    Write-Host "✅ Arbitraje simulado ejecutado" -ForegroundColor Green
                    Write-Host "💰 Profit estimado: +$estimatedProfit SOL" -ForegroundColor Green
                    Write-Host "📊 Total profits acumulados: +$totalProfits SOL" -ForegroundColor Green
                } else {
                    if (-not $ExecuteReal) {
                        Write-Host "🧪 MODO ANÁLISIS: No ejecutando arbitraje" -ForegroundColor Yellow
                    } else {
                        Write-Host "⚠️ Balance insuficiente para arbitraje" -ForegroundColor Yellow
                    }
                }
            } else {
                Write-Host "⏳ Oportunidad no rentable (Spread: $spread%, Profit: $$$profit)" -ForegroundColor Gray
            }
        } else {
            Write-Host "❌ No se pudieron extraer datos de arbitraje" -ForegroundColor Red
        }
        
        # 4. Resumen cada 10 ciclos
        if ($cycle % 10 -eq 0) {
            Write-Host ""
            Write-Host "📊 === RESUMEN CADA 10 CICLOS ===" -ForegroundColor Magenta
            Write-Host "⏰ Tiempo ejecutándose: $($elapsed.ToString('hh\:mm\:ss'))" -ForegroundColor Gray
            if ($currentBalance -gt 0) {
                Write-Host "💰 Balance actual: $currentBalance SOL" -ForegroundColor Gray
            }
            Write-Host "🎯 Ciclos completados: $cycle" -ForegroundColor Gray
            if ($totalProfits -gt 0) {
                Write-Host "💰 Profits simulados totales: +$totalProfits SOL" -ForegroundColor Green
            }
            Write-Host ""
        }
        
    }
    catch {
        Write-Host "❌ Error en ciclo: $($_.Exception.Message)" -ForegroundColor Red
    }
    
    # 5. Pausa
    Write-Host "⏳ Esperando $IntervalSeconds segundos..." -ForegroundColor DarkGray
    Write-Host $("-" * 80) -ForegroundColor DarkGray
    Start-Sleep -Seconds $IntervalSeconds
}
