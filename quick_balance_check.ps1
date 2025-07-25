#!/usr/bin/env pwsh
# ================================================================
# QUICK BALANCE CHECK - Verificación Rápida de Incrementos
# ================================================================

Write-Host "🔍 VERIFICACIÓN RÁPIDA DE BALANCE" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan

$walletFile = "mainnet-arbitrage-wallet.json"
$walletAddress = "JDzF9LkpoQVac6c6ufHW1c6Gevd3vFB4RXSSjceL8Kq7"

# Verificar si existe el archivo de wallet
if (-not (Test-Path $walletFile)) {
    Write-Host "❌ ERROR: No se encuentra el archivo $walletFile" -ForegroundColor Red
    exit 1
}

Write-Host "📍 Wallet: $walletAddress" -ForegroundColor Yellow
Write-Host "📁 Archivo: $walletFile" -ForegroundColor Yellow
Write-Host "⏰ Timestamp: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor Yellow
Write-Host ""

# Obtener balance actual
Write-Host "💰 Obteniendo balance actual..." -ForegroundColor Green
try {
    $balance = solana balance $walletFile
    Write-Host "💰 Balance Actual: $balance" -ForegroundColor Green
    
    # Convertir a número para cálculos
    if ($balance -match "(\d+\.\d+)") {
        $balanceNum = [decimal]$matches[1]
        $usdValue = $balanceNum * 180  # Estimado $180 por SOL
        Write-Host "💵 Valor USD Estimado: `$$($usdValue.ToString('F2'))" -ForegroundColor Green
    }
}
catch {
    Write-Host "❌ ERROR obteniendo balance: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

Write-Host ""

# Obtener últimas transacciones
Write-Host "🔗 Últimas 5 transacciones:" -ForegroundColor Cyan
try {
    $transactions = solana transaction-history $walletAddress --limit 5
    if ($transactions) {
        Write-Host $transactions -ForegroundColor Yellow
    } else {
        Write-Host "ℹ️  No se encontraron transacciones recientes" -ForegroundColor Yellow
    }
}
catch {
    Write-Host "⚠️  Error obteniendo historial: $($_.Exception.Message)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "✅ Verificación completada" -ForegroundColor Green
Write-Host ""

# Agregar al tracking file
$timestamp = Get-Date -Format 'yyyy-MM-dd HH:mm:ss'
$trackingEntry = @"

### 🕐 Balance verificado: $timestamp
```
Balance: $balance
USD Estimado: ~`$$($usdValue.ToString('F2')) USD (@ `$180/SOL)
```
"@

try {
    Add-Content -Path "WALLET_BALANCE_TRACKING.md" -Value $trackingEntry
    Write-Host "📝 Balance guardado en WALLET_BALANCE_TRACKING.md" -ForegroundColor Green
}
catch {
    Write-Host "⚠️  No se pudo guardar en tracking file: $($_.Exception.Message)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "🎯 Para verificar incrementos, ejecuta este script periódicamente" -ForegroundColor Cyan
Write-Host "📊 Compara con las ganancias reportadas por el sistema de arbitraje" -ForegroundColor Cyan
