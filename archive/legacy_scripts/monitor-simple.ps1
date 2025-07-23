# Arbitraje Cross-DEX - Monitor Simple y Confiable
param(
    [int]$Interval = 30,
    [switch]$RealMode = $false
)

Write-Host "🚀 MONITOR DE ARBITRAJE CROSS-DEX" -ForegroundColor Green
Write-Host "=================================" -ForegroundColor Green
Write-Host "⏱️  Intervalo: $Interval segundos" -ForegroundColor Cyan
if ($RealMode) {
    Write-Host "⚠️  MODO REAL: Ejecutará trades" -ForegroundColor Red
} else {
    Write-Host "🔍 MODO ANÁLISIS: Solo monitoreo" -ForegroundColor Yellow
}
Write-Host ""

$contador = 0
$inicioTime = Get-Date

while ($true) {
    $contador++
    $tiempoActual = Get-Date
    $duracion = $tiempoActual - $inicioTime
    
    Write-Host "🔄 CICLO $contador | $(Get-Date -Format 'HH:mm:ss') | Uptime: $($duracion.ToString('hh\:mm\:ss'))" -ForegroundColor Blue
    
    # Verificar balance
    Write-Host "💰 Verificando balance..." -ForegroundColor Cyan
    try {
        $balanceCmd = cargo run --bin check_devnet_balance
        $balanceTexto = $balanceCmd -join " "
        Write-Host "✅ Balance obtenido" -ForegroundColor Green
    }
    catch {
        Write-Host "❌ Error obteniendo balance" -ForegroundColor Red
    }
    
    # Análisis de arbitraje
    Write-Host "🔍 Analizando spread..." -ForegroundColor Cyan
    try {
        $arbitrageCmd = cargo run --bin test_arbitrage_cross_dex
        $arbitrageTexto = $arbitrageCmd -join " "
        
        # Buscar spread
        if ($arbitrageTexto -like "*Spread:*") {
            $spreadLine = ($arbitrageTexto -split " " | Where-Object { $_ -like "*%*" })[0]
            if ($spreadLine) {
                $spreadNum = $spreadLine -replace "%", ""
                Write-Host "📊 Spread encontrado: $spreadNum%" -ForegroundColor Green
                
                # Verificar si es profitable
                $spreadVal = [double]$spreadNum
                if ($spreadVal -gt 50) {
                    Write-Host "🎯 OPORTUNIDAD EXCELENTE! Spread > 50%" -ForegroundColor Green
                    
                    if ($RealMode) {
                        Write-Host "🚀 MODO REAL: Ejecutaría arbitraje aquí" -ForegroundColor Red
                    } else {
                        Write-Host "🧪 MODO ANÁLISIS: Oportunidad detectada" -ForegroundColor Yellow
                    }
                } elseif ($spreadVal -gt 10) {
                    Write-Host "💡 Oportunidad buena (>10%)" -ForegroundColor Yellow
                } else {
                    Write-Host "⏳ Spread bajo (<10%)" -ForegroundColor Gray
                }
            }
        } else {
            Write-Host "❌ No se detectó spread en output" -ForegroundColor Red
        }
    }
    catch {
        Write-Host "❌ Error en análisis de arbitraje" -ForegroundColor Red
    }
    
    # Resumen cada 10 ciclos
    if ($contador % 10 -eq 0) {
        Write-Host ""
        Write-Host "📊 RESUMEN - Ciclo $contador" -ForegroundColor Magenta
        Write-Host "⏰ Tiempo corriendo: $($duracion.ToString('hh\:mm\:ss'))" -ForegroundColor Gray
        Write-Host ""
    }
    
    Write-Host "⏳ Esperando $Interval segundos..." -ForegroundColor DarkGray
    Write-Host "$('-' * 60)" -ForegroundColor DarkGray
    
    Start-Sleep -Seconds $Interval
}
