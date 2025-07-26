#!/usr/bin/env pwsh
# =========================================================================
# TEST FINAL - Verificar si ahora pasan oportunidades del filtro DEX
# =========================================================================

Write-Host "🎯 ═══════════════════════════════════════════════════════════════"
Write-Host "    TEST FINAL - ¿SE RESOLVIÓ EL PROBLEMA DEL FILTRO?"
Write-Host "🎯 ═══════════════════════════════════════════════════════════════"

$env:FORCE_REAL_TRANSACTIONS = "true"
$env:MAX_TRADE_SOL = "0.001"
$env:RUST_LOG = "info"

Write-Host "✅ Configuración de test:"
Write-Host "   FORCE_REAL_TRANSACTIONS = $($env:FORCE_REAL_TRANSACTIONS)"
Write-Host "   MAX_TRADE_SOL = $($env:MAX_TRADE_SOL)"
Write-Host ""
Write-Host "🔍 Esperamos ver:"
Write-Host "   - 'Enhanced Portfolio Optimization' con > 0 opportunities"
Write-Host "   - 'ML Analysis' ejecutándose"
Write-Host "   - 'EJECUTANDO TRADE REAL' si ML score > 0.2"
Write-Host ""

# Ejecutar por 30 segundos para ver varios ciclos
Write-Host "🚀 Ejecutando sistema (30 segundos)..."
$job = Start-Job {
    ./target/debug/arbitrage_phase45_clean.exe 2>&1
}

Start-Sleep -Seconds 30
Stop-Job $job
$output = Receive-Job $job
Remove-Job $job

# Analizar resultados
$optimizations = $output | Select-String "Enhanced Portfolio Optimization.*Optimized to (\d+)"
$mlAnalyses = $output | Select-String "ML Analysis"
$tradeExecutions = $output | Select-String "EJECUTANDO TRADE REAL"

Write-Host ""
Write-Host "📊 RESULTADOS DEL TEST:"
Write-Host ""

# Portfolio optimizations
if ($optimizations) {
    Write-Host "✅ Portfolio Optimizations encontradas:"
    foreach ($opt in $optimizations) {
        if ($opt -match "Optimized to (\d+)") {
            $count = $matches[1]
            if ([int]$count -gt 0) {
                Write-Host "   🟢 $opt" -ForegroundColor Green
            } else {
                Write-Host "   🔴 $opt" -ForegroundColor Red
            }
        }
    }
} else {
    Write-Host "❌ No se encontraron Portfolio Optimizations"
}

Write-Host ""

# ML Analyses
if ($mlAnalyses) {
    Write-Host "✅ ML Analyses encontrados: $($mlAnalyses.Count)"
    $mlAnalyses | Select-Object -First 3 | ForEach-Object {
        Write-Host "   🧠 $_" -ForegroundColor Cyan
    }
} else {
    Write-Host "❌ No se encontraron ML Analyses"
}

Write-Host ""

# Trade executions
if ($tradeExecutions) {
    Write-Host "✅ Trade Executions encontrados: $($tradeExecutions.Count)"
    $tradeExecutions | ForEach-Object {
        Write-Host "   💰 $_" -ForegroundColor Yellow
    }
} else {
    Write-Host "❌ No se encontraron Trade Executions"
}

Write-Host ""
Write-Host "🎯 ═══════════════════════════════════════════════════════════════"
if ($optimizations -and ($optimizations | Where-Object { $_ -match "Optimized to [1-9]" })) {
    Write-Host "    ✅ ÉXITO: El filtro DEX está dejando pasar oportunidades"
} else {
    Write-Host "    ❌ FALLO: El filtro DEX sigue bloqueando todas las oportunidades"
}
Write-Host "🎯 ═══════════════════════════════════════════════════════════════"
