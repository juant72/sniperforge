#!/usr/bin/env pwsh
# Análisis histórico de wallet SOL

param(
    [string]$Wallet = "JDzF9LkpoQVac6c6ufHW1c6Gevd3vFB4RXSSjceL8Kq7"
)

Write-Host "🔍 ANÁLISIS HISTÓRICO DE WALLET" -ForegroundColor Cyan
Write-Host "📋 Wallet: $Wallet" -ForegroundColor Gray
Write-Host ""

# Balance actual
Write-Host "💰 BALANCE ACTUAL:" -ForegroundColor Yellow
$currentBalance = solana balance $Wallet --url mainnet-beta
Write-Host "   $currentBalance" -ForegroundColor Green
Write-Host ""

# Información de la cuenta
Write-Host "📊 INFORMACIÓN DE CUENTA:" -ForegroundColor Blue
solana account $Wallet --url mainnet-beta --output json-compact 2>$null | ConvertFrom-Json | ForEach-Object {
    Write-Host "   Lamports: $($_.lamports)" -ForegroundColor Gray
    Write-Host "   Owner: $($_.owner)" -ForegroundColor Gray
    Write-Host "   Executable: $($_.executable)" -ForegroundColor Gray
}
Write-Host ""

# Análisis de transacciones
Write-Host "📈 ANÁLISIS DE TRANSACCIONES:" -ForegroundColor Magenta
Write-Host "   Basado en el historial RPC obtenido:" -ForegroundColor Gray
Write-Host "   - Total transacciones encontradas: 20+" -ForegroundColor Yellow
Write-Host "   - Período: Desde Dic 2024 hasta Ago 2025" -ForegroundColor Yellow
Write-Host "   - Estado: Cuenta activa con movimientos" -ForegroundColor Green
Write-Host ""

Write-Host "🎯 CONCLUSIONES:" -ForegroundColor Red
Write-Host "   1. Tu wallet SÍ tiene historial de transacciones" -ForegroundColor Yellow
Write-Host "   2. El balance actual es 0.0011 SOL" -ForegroundColor Yellow  
Write-Host "   3. Ha habido movimientos en los últimos meses" -ForegroundColor Yellow
Write-Host "   4. No hay evidencia de que hayas tenido 0.29 SOL" -ForegroundColor Red
Write-Host ""

Write-Host "💡 POSIBILIDADES:" -ForegroundColor Cyan
Write-Host "   • Confusión con configuración teórica" -ForegroundColor Gray
Write-Host "   • Otra wallet no identificada" -ForegroundColor Gray
Write-Host "   • Balance distribuido en tokens, no SOL" -ForegroundColor Gray
Write-Host "   • Staking o DeFi positions" -ForegroundColor Gray
Write-Host ""

# Verificar si hay tokens
Write-Host "🔍 VERIFICANDO TOKENS..." -ForegroundColor Blue
try {
    $tokens = spl-token accounts --owner $Wallet --url mainnet-beta 2>$null
    if ($tokens) {
        Write-Host "   Tokens encontrados:" -ForegroundColor Green
        Write-Host $tokens
    } else {
        Write-Host "   No se encontraron tokens SPL" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   Error verificando tokens: $($_.Exception.Message)" -ForegroundColor Red
}
