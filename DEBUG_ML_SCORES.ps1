#!/usr/bin/env pwsh
# =========================================================================
# DEBUG ML SCORES - Ver exactamente qué scores está dando el ML
# =========================================================================

Write-Host "🔍 ═══════════════════════════════════════════════════════════════"
Write-Host "    DEBUGGING ML SCORES - ¿POR QUÉ NO HAY TRADES?"
Write-Host "🔍 ═══════════════════════════════════════════════════════════════"

# CONFIGURACIÓN PARA VER DEBUG DEL ML
$env:FORCE_REAL_TRANSACTIONS = "true"
$env:MAX_TRADE_SOL = "0.001"
$env:RUST_LOG = "debug"  # NIVEL DEBUG para ver scores exactos

Write-Host "✅ Configuración de debug:"
Write-Host "   FORCE_REAL_TRANSACTIONS = $($env:FORCE_REAL_TRANSACTIONS)"
Write-Host "   MAX_TRADE_SOL = $($env:MAX_TRADE_SOL)"
Write-Host "   RUST_LOG = $($env:RUST_LOG)"
Write-Host ""
Write-Host "🔍 Buscando específicamente:"
Write-Host "   - 'ML Score for' - ver scores exactos del ML"
Write-Host "   - 'ML RECOMIENDA' - ver si pasa el threshold 0.2"
Write-Host "   - 'EJECUTANDO TRADE REAL' - ver si llega a ejecución"
Write-Host ""

# Compilar primero
Write-Host "🔨 Compilando sistema..."
cargo build --bin arbitrage_phase45_clean 2>&1 | Out-Null

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Compilación exitosa"
    Write-Host ""
    Write-Host "🚀 Ejecutando sistema con debug ML..."
    Write-Host "   (Presiona Ctrl+C después de ver algunos ciclos)"
    Write-Host ""
    
    # Ejecutar y filtrar solo logs relevantes del ML
    ./target/debug/arbitrage_phase45_clean.exe 2>&1 | 
    Select-String -Pattern "ML Score for|ML RECOMIENDA|EJECUTANDO TRADE REAL|🧠 ML Analysis|composite=|🎯 ML Score" |
    ForEach-Object {
        $timestamp = Get-Date -Format "HH:mm:ss"
        Write-Host "[$timestamp] $_"
    }
} else {
    Write-Host "❌ Error en compilación"
    exit 1
}
