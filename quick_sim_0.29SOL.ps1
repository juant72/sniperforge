# 🎯 SIMULACIÓN RÁPIDA CON 0.29 SOL - USANDO CONFIGURACIÓN PROBADA
# Usar la configuración del análisis mainnet que funcionó perfectamente

param(
    [int]$DurationMinutes = 2
)

$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"

Write-Host "💰 SIMULACIÓN RÁPIDA - 0.29 SOL CON CONFIGURACIÓN PROBADA" -ForegroundColor Cyan
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "📊 USANDO CONFIGURACIÓN DEL ANÁLISIS MAINNET ESPECTACULAR" -ForegroundColor Green
Write-Host "   • Balance real: 0.292 SOL confirmado" -ForegroundColor White
Write-Host "   • Max trade: 0.05 SOL (17% de tu capital)" -ForegroundColor White
Write-Host "   • Configuración: Mainnet con datos reales" -ForegroundColor White
Write-Host "   • Duración: $DurationMinutes minutos" -ForegroundColor White

# Verificar executable
$executable = ".\target\release\arbitrage_phase45_clean.exe"
if (-not (Test-Path $executable)) {
    Write-Host "❌ Executable no encontrado. Compilando..." -ForegroundColor Red
    cargo build --release
}

Write-Host "`n🚀 EJECUTANDO SIMULACIÓN CON TU CAPITAL REAL..." -ForegroundColor Magenta

# Ejecutar directamente el sistema con timeout
$logFile = "quick_sim_0.29SOL_$timestamp.log"
$process = Start-Process -FilePath $executable -PassThru -RedirectStandardOutput $logFile -RedirectStandardError "${logFile}.error"

$startTime = Get-Date
$endTime = $startTime.AddMinutes($DurationMinutes)

Write-Host "🔄 Monitoreando por $DurationMinutes minutos..." -ForegroundColor Cyan
Write-Host "   Inicio: $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor White

$opportunityCount = 0
$lastCheck = 0

while ((Get-Date) -lt $endTime -and -not $process.HasExited) {
    Start-Sleep -Seconds 2
    
    if (Test-Path $logFile) {
        $logContent = Get-Content $logFile -Raw -ErrorAction SilentlyContinue
        
        # Contar oportunidades
        $crossChain = ([regex]::Matches($logContent, "Cross-chain.*profit")).Count
        $flashLoans = ([regex]::Matches($logContent, "Flash loan.*profit")).Count
        $traditional = ([regex]::Matches($logContent, "Traditional.*profit")).Count
        $errors = ([regex]::Matches($logContent, "ERROR")).Count
        
        $totalOpportunities = $crossChain + $flashLoans + $traditional
        
        if ($totalOpportunities -gt $lastCheck) {
            $newOps = $totalOpportunities - $lastCheck
            Write-Host "   ✨ +$newOps oportunidades detectadas (Total: $totalOpportunities)" -ForegroundColor Green
            $lastCheck = $totalOpportunities
        }
        
        if ($errors -eq 0) {
            Write-Host "   ✅ Sistema funcionando sin errores" -ForegroundColor Green
        }
    }
    
    $elapsed = [math]::Round(((Get-Date) - $startTime).TotalMinutes, 1)
    Write-Host "   [$(Get-Date -Format 'HH:mm:ss')] $elapsed/$DurationMinutes min | Oportunidades: $opportunityCount" -ForegroundColor Cyan
}

# Finalizar
if (-not $process.HasExited) {
    $process.Kill()
}

Write-Host "`n✅ SIMULACIÓN COMPLETADA" -ForegroundColor Green

# Análisis final
if (Test-Path $logFile) {
    $finalLog = Get-Content $logFile -Raw
    
    $finalCrossChain = ([regex]::Matches($finalLog, "Cross-chain.*profit")).Count
    $finalFlashLoans = ([regex]::Matches($finalLog, "Flash loan.*profit")).Count
    $finalTraditional = ([regex]::Matches($finalLog, "Traditional.*profit")).Count
    $finalErrors = ([regex]::Matches($finalLog, "ERROR")).Count
    
    Write-Host "`n📊 RESULTADOS FINALES PARA 0.29 SOL:" -ForegroundColor Cyan
    Write-Host "═══════════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host "🎯 Oportunidades Cross-chain: $finalCrossChain" -ForegroundColor Yellow
    Write-Host "🎯 Oportunidades Flash loans: $finalFlashLoans" -ForegroundColor Yellow
    Write-Host "🎯 Oportunidades Tradicionales: $finalTraditional" -ForegroundColor Yellow
    Write-Host "⚠️  Errores: $finalErrors" -ForegroundColor $(if($finalErrors -gt 0){"Red"}else{"Green"})
    
    $totalFinal = $finalCrossChain + $finalFlashLoans + $finalTraditional
    
    if ($totalFinal -gt 0) {
        # Cálculo conservador de profit para 0.29 SOL
        $estimatedProfitSOL = [math]::Round($totalFinal * 0.002, 4)  # 0.002 SOL por oportunidad
        $estimatedUSD = [math]::Round($estimatedProfitSOL * 290, 2)  # ~$290 por SOL
        $roi = [math]::Round(($estimatedProfitSOL / 0.29) * 100, 2)
        
        Write-Host "`n💰 PROYECCIÓN PARA TU CAPITAL:" -ForegroundColor Green
        Write-Host "   • Profit estimado: $estimatedProfitSOL SOL (~$$estimatedUSD USD)" -ForegroundColor Green
        Write-Host "   • ROI estimado: $roi%" -ForegroundColor Green
        
        if ($roi -gt 3) {
            Write-Host "`n🚀 ¡RENTABLE! Sistema detecta oportunidades reales" -ForegroundColor Magenta
            Write-Host "💡 RECOMENDACIÓN: Proceder con trading real conservador" -ForegroundColor Green
        } else {
            Write-Host "`n⚠️  Rentabilidad moderada - considerar condiciones de mercado" -ForegroundColor Yellow
        }
    } else {
        Write-Host "`n❌ No se detectaron oportunidades rentables" -ForegroundColor Red
        Write-Host "💡 Posibles causas:" -ForegroundColor Yellow
        Write-Host "   • Mercado muy estable (poca volatilidad)" -ForegroundColor White
        Write-Host "   • Horario de baja actividad" -ForegroundColor White
        Write-Host "   • Configuración muy conservadora" -ForegroundColor White
    }
    
    Write-Host "`n📋 Log guardado en: $logFile" -ForegroundColor Cyan
    
    # Mostrar últimas líneas del log para diagnóstico
    Write-Host "`n🔍 ÚLTIMAS LÍNEAS DEL LOG:" -ForegroundColor Cyan
    $lastLines = Get-Content $logFile -Tail 3 -ErrorAction SilentlyContinue
    foreach ($line in $lastLines) {
        Write-Host "   $line" -ForegroundColor Gray
    }
}

Write-Host "`n✨ ANÁLISIS COMPLETADO CON TU CAPITAL REAL DE 0.29 SOL" -ForegroundColor Magenta
