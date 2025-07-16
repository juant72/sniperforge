# Script de ejecución continua de arbitraje
# Este script ejecuta el arbitraje cada 60 segundos monitoreando profits

param(
    [int]$IntervalSeconds = 60,
    [switch]$DryRun = $false
)

Write-Host "🚀 === ARBITRAJE AUTOMATIZADO CROSS-DEX ===" -ForegroundColor Green
Write-Host "⚡ Monitoreando cada $IntervalSeconds segundos" -ForegroundColor Cyan
if ($DryRun) {
    Write-Host "🧪 MODO DRY-RUN: Solo análisis, sin ejecución real" -ForegroundColor Yellow
}
Write-Host "❌ Ctrl+C para detener" -ForegroundColor Red
Write-Host ""

$startTime = Get-Date
$initialBalance = $null
$totalProfits = 0
$executionCount = 0

# Función para obtener balance
function Get-SOLBalance {
    try {
        Write-Host "🔍 Ejecutando check_devnet_balance..." -ForegroundColor Gray
        $output = & cargo run --bin check_devnet_balance 2>&1
        $outputString = $output -join "`n"
        
        Write-Host "📝 Output completo:" -ForegroundColor Gray
        Write-Host $outputString -ForegroundColor DarkGray
        
        if ($outputString -match "💰 SOL Balance: ([\d.]+) SOL") {
            $balance = [decimal]$matches[1]
            Write-Host "✅ Balance extraído: $balance SOL" -ForegroundColor Green
            return $balance
        } else {
            Write-Host "❌ No se pudo extraer balance del output" -ForegroundColor Red
            Write-Host "🔍 Buscando patrón alternativo..." -ForegroundColor Yellow
            
            # Intentar patrón alternativo
            if ($outputString -match "Balance: ([\d.]+) SOL") {
                $balance = [decimal]$matches[1]
                Write-Host "✅ Balance extraído (patrón alternativo): $balance SOL" -ForegroundColor Green
                return $balance
            }
        }
    }
    catch {
        Write-Host "❌ Error obteniendo balance: $($_.Exception.Message)" -ForegroundColor Red
        Write-Host "🔍 StackTrace: $($_.ScriptStackTrace)" -ForegroundColor Red
    }
    return $null
}

# Función para verificar oportunidades
function Test-ArbitrageOpportunity {
    try {
        $output = cargo run --bin test_arbitrage_cross_dex 2>&1 | Out-String
        
        $spread = $null
        $profitPerSol = $null
        
        if ($output -match "Spread:\s+([\d.]+)%") {
            $spread = [decimal]$matches[1]
        }
        
        if ($output -match "Profit por SOL:\s+\$?([\d.]+)") {
            $profitPerSol = [decimal]$matches[1]
        }
        
        return @{
            Spread = $spread
            ProfitPerSol = $profitPerSol
            Output = $output
        }
    }
    catch {
        Write-Host "❌ Error verificando oportunidades: $($_.Exception.Message)" -ForegroundColor Red
        return $null
    }
}

# Función para ejecutar arbitraje real
function Execute-ArbitrageReal {
    param([decimal]$Amount)
    
    try {
        Write-Host "🎯 Ejecutando arbitraje con $Amount SOL..." -ForegroundColor Green
        
        # Por ahora usamos el test como simulación
        # En el futuro aquí iría la ejecución real
        $output = cargo run --bin test_arbitrage_cross_dex 2>&1 | Out-String
        
        # Simulamos el éxito por ahora
        $estimatedProfit = $Amount * 0.64  # Estimado basado en spread actual
        
        Write-Host "✅ Arbitraje simulado completado" -ForegroundColor Green
        Write-Host "💰 Profit estimado: +$estimatedProfit SOL" -ForegroundColor Green
        
        return @{
            Success = $true
            Profit = $estimatedProfit
            Output = $output
        }
    }
    catch {
        Write-Host "❌ Error ejecutando arbitraje: $($_.Exception.Message)" -ForegroundColor Red
        return @{
            Success = $false
            Profit = 0
            Output = $_.Exception.Message
        }
    }
}

# Obtener balance inicial
Write-Host "🔍 Obteniendo balance inicial..." -ForegroundColor Cyan
$initialBalance = Get-SOLBalance
if ($null -eq $initialBalance) {
    Write-Host "❌ No se pudo obtener balance inicial. Verifica la wallet." -ForegroundColor Red
    exit 1
}

Write-Host "💰 Balance inicial: $initialBalance SOL" -ForegroundColor White
Write-Host ""

# Loop principal
$cycle = 0
while ($true) {
    $cycle++
    $currentTime = Get-Date
    $elapsed = $currentTime - $startTime
    
    Write-Host "🔄 === CICLO #$cycle | $(Get-Date -Format 'HH:mm:ss') | Tiempo: $($elapsed.ToString('hh\:mm\:ss')) ===" -ForegroundColor Blue
    
    # 1. Verificar balance actual
    $currentBalance = Get-SOLBalance
    if ($null -ne $currentBalance) {
        $balanceChange = $currentBalance - $initialBalance
        Write-Host "💰 Balance: $currentBalance SOL" -ForegroundColor White
        
        if ($balanceChange -gt 0) {
            Write-Host "📈 Profit acumulado: +$balanceChange SOL (+$([math]::Round(($balanceChange / $initialBalance) * 100, 2))%)" -ForegroundColor Green
        } elseif ($balanceChange -lt 0) {
            Write-Host "📉 Pérdida acumulada: $balanceChange SOL ($([math]::Round(($balanceChange / $initialBalance) * 100, 2))%)" -ForegroundColor Red
        }
    }
    
    # 2. Verificar oportunidades de arbitraje
    Write-Host "🔍 Analizando oportunidades..." -ForegroundColor Cyan
    $opportunity = Test-ArbitrageOpportunity
    
    if ($null -ne $opportunity -and $null -ne $opportunity.Spread) {
        Write-Host "📊 Spread detectado: $($opportunity.Spread)%" -ForegroundColor $(if ($opportunity.Spread -gt 10) { "Green" } elseif ($opportunity.Spread -gt 5) { "Yellow" } else { "Red" })
        
        if ($null -ne $opportunity.ProfitPerSol) {
            Write-Host "💡 Profit por SOL: $$$($opportunity.ProfitPerSol)" -ForegroundColor $(if ($opportunity.ProfitPerSol -gt 50) { "Green" } else { "Yellow" })
        }
        
        # 3. Decidir si ejecutar arbitraje
        if ($opportunity.Spread -gt 10 -and $opportunity.ProfitPerSol -gt 50) {
            if (-not $DryRun -and $null -ne $currentBalance -and $currentBalance -gt 0.1) {
                $arbitrageAmount = [math]::Min(0.1, $currentBalance * 0.1)  # Usar máximo 10% del balance o 0.1 SOL
                
                Write-Host "🎯 EJECUTANDO ARBITRAJE!" -ForegroundColor Green
                Write-Host "💰 Cantidad: $arbitrageAmount SOL" -ForegroundColor Green
                
                $result = Execute-ArbitrageReal -Amount $arbitrageAmount
                
                if ($result.Success) {
                    $executionCount++
                    $totalProfits += $result.Profit
                    Write-Host "✅ Arbitraje #$executionCount completado" -ForegroundColor Green
                    Write-Host "💰 Profit total simulado: +$totalProfits SOL" -ForegroundColor Green
                } else {
                    Write-Host "❌ Fallo en arbitraje" -ForegroundColor Red
                }
                
                # Pausa extra después de ejecución
                Write-Host "⏸️ Pausando 30 segundos después de arbitraje..." -ForegroundColor Gray
                Start-Sleep -Seconds 30
            } else {
                if ($DryRun) {
                    Write-Host "🧪 DRY-RUN: Arbitraje profitable detectado pero no ejecutado" -ForegroundColor Yellow
                } else {
                    Write-Host "⚠️ Balance insuficiente para arbitraje" -ForegroundColor Yellow
                }
            }
        } else {
            Write-Host "⏳ Spread no suficiente para arbitraje rentable" -ForegroundColor Gray
        }
    } else {
        Write-Host "❌ No se pudieron obtener datos de oportunidades" -ForegroundColor Red
    }
    
    # 4. Mostrar resumen cada 10 ciclos
    if ($cycle % 10 -eq 0) {
        Write-Host ""
        Write-Host "📊 === RESUMEN (Ciclo $cycle) ===" -ForegroundColor Magenta
        Write-Host "⏰ Tiempo ejecutándose: $($elapsed.ToString('hh\:mm\:ss'))" -ForegroundColor Gray
        Write-Host "💰 Balance inicial: $initialBalance SOL" -ForegroundColor Gray
        if ($null -ne $currentBalance) {
            Write-Host "💰 Balance actual: $currentBalance SOL" -ForegroundColor Gray
            $totalChange = $currentBalance - $initialBalance
            Write-Host "📈 Cambio total: $totalChange SOL" -ForegroundColor $(if ($totalChange -gt 0) { "Green" } else { "Red" })
        }
        Write-Host "🎯 Arbitrajes ejecutados: $executionCount" -ForegroundColor Gray
        Write-Host "💰 Profits simulados: +$totalProfits SOL" -ForegroundColor Gray
        Write-Host ""
    }
    
    # 5. Pausa hasta el siguiente ciclo
    Write-Host "⏳ Esperando $IntervalSeconds segundos..." -ForegroundColor DarkGray
    Write-Host $("-" * 80) -ForegroundColor DarkGray
    Start-Sleep -Seconds $IntervalSeconds
}
