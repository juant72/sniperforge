# Monitor continuo del balance y profits del arbitraje
# Ejecutar: .\monitor-arbitrage-profits.ps1

param(
    [int]$IntervalSeconds = 30
)

Write-Host "🔍 === MONITOR DE PROFITS ARBITRAJE ===" -ForegroundColor Green
Write-Host "📊 Monitoreando cada $IntervalSeconds segundos..." -ForegroundColor Cyan
Write-Host "❌ Ctrl+C para detener" -ForegroundColor Yellow
Write-Host ""

$startTime = Get-Date
$initialBalance = $null

while ($true) {
    $currentTime = Get-Date
    $elapsed = $currentTime - $startTime
    
    Write-Host "⏰ $(Get-Date -Format 'HH:mm:ss') | Tiempo ejecutándose: $($elapsed.ToString('hh\:mm\:ss'))" -ForegroundColor Gray
    
    try {
        # Verificar balance actual
        $balanceOutput = cargo run --bin check_devnet_balance 2>&1 | Out-String
        
        if ($balanceOutput -match "💰 SOL Balance: ([\d.]+) SOL") {
            $currentBalance = [decimal]$matches[1]
            
            if ($null -eq $initialBalance) {
                $initialBalance = $currentBalance
                Write-Host "💰 Balance inicial: $initialBalance SOL" -ForegroundColor White
            }
            else {
                $profit = $currentBalance - $initialBalance
                $profitPercent = if ($initialBalance -gt 0) { ($profit / $initialBalance) * 100 } else { 0 }
                
                Write-Host "💰 Balance actual: $currentBalance SOL" -ForegroundColor White
                
                if ($profit -gt 0) {
                    Write-Host "📈 PROFIT: +$profit SOL (+$([math]::Round($profitPercent, 2))%)" -ForegroundColor Green
                } elseif ($profit -lt 0) {
                    Write-Host "📉 Pérdida: $profit SOL ($([math]::Round($profitPercent, 2))%)" -ForegroundColor Red
                } else {
                    Write-Host "➖ Sin cambios en balance" -ForegroundColor Gray
                }
            }
        }
        
        # Verificar si hay oportunidades de arbitraje detectadas
        Write-Host "🔍 Verificando oportunidades..." -ForegroundColor Cyan
        $arbitrageOutput = cargo run --bin test_arbitrage_cross_dex 2>&1 | Out-String
        
        if ($arbitrageOutput -match "Spread:\s+([\d.]+)%") {
            $spread = [decimal]$matches[1]
            Write-Host "📊 Spread actual: $spread%" -ForegroundColor $(if ($spread -gt 10) { "Green" } else { "Yellow" })
        }
        
        if ($arbitrageOutput -match "Profit por SOL:\s+\$?([\d.]+)") {
            $profitPerSol = [decimal]$matches[1]
            Write-Host "💡 Profit por SOL: $$$profitPerSol" -ForegroundColor $(if ($profitPerSol -gt 50) { "Green" } else { "Yellow" })
        }
        
    }
    catch {
        Write-Host "❌ Error monitoreando: $($_.Exception.Message)" -ForegroundColor Red
    }
    
    Write-Host "$("`" " * 50)" -ForegroundColor DarkGray
    Start-Sleep -Seconds $IntervalSeconds
}
