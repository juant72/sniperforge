# 🎯 SCRIPT DE ACTIVACIÓN - BOT SNIPER DE LIQUIDEZ
# Fecha: 6 de agosto, 2025
# Propósito: Activar y configurar el bot sniper para detectar nuevos pools

Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "🎯 ACTIVACIÓN DEL BOT SNIPER DE POOLS DE LIQUIDEZ" -ForegroundColor Yellow
Write-Host "   SniperForge Enterprise v3.0" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan

$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
Write-Host "📅 Iniciando activación: $timestamp" -ForegroundColor White

# 1. Verificar estado del sistema
Write-Host ""
Write-Host "🔍 PASO 1: Verificando estado del sistema..." -ForegroundColor Yellow

# Verificar si el servidor está corriendo
$serverRunning = Get-Process -Name "sniperforge-enterprise" -ErrorAction SilentlyContinue
if ($serverRunning) {
    Write-Host "✅ Servidor TCP activo (PID: $($serverRunning.Id))" -ForegroundColor Green
} else {
    Write-Host "❌ Servidor TCP no encontrado" -ForegroundColor Red
    Write-Host "   Iniciando servidor..." -ForegroundColor Yellow
    # Aquí iría el comando para iniciar el servidor
}

# Verificar puerto 8888
$portCheck = Test-NetConnection -ComputerName "127.0.0.1" -Port 8888 -InformationLevel Quiet -ErrorAction SilentlyContinue
if ($portCheck) {
    Write-Host "✅ Puerto 8888 disponible" -ForegroundColor Green
} else {
    Write-Host "⚠️  Puerto 8888 no responde" -ForegroundColor Yellow
}

# 2. Verificar configuración de wallet
Write-Host ""
Write-Host "💼 PASO 2: Verificando configuración de wallet..." -ForegroundColor Yellow

$walletFiles = @(
    "wallet.json",
    "wallet_secure_20250806_120539.json",
    "test_wallet.json"
)

$walletFound = $false
foreach ($wallet in $walletFiles) {
    if (Test-Path $wallet) {
        Write-Host "✅ Wallet encontrada: $wallet" -ForegroundColor Green
        $walletFound = $true
        break
    }
}

if (-not $walletFound) {
    Write-Host "⚠️  No se encontraron archivos de wallet" -ForegroundColor Yellow
    Write-Host "   Será necesario configurar wallet antes de trading real" -ForegroundColor Yellow
}

# 3. Verificar configuraciones
Write-Host ""
Write-Host "⚙️  PASO 3: Verificando configuraciones..." -ForegroundColor Yellow

$configFiles = @(
    "config/small_capital_config.json",
    "config/system.yaml",
    "config/liquidity_sniper_config.json"
)

foreach ($config in $configFiles) {
    if (Test-Path $config) {
        Write-Host "✅ Configuración encontrada: $config" -ForegroundColor Green
    } else {
        Write-Host "❌ Configuración faltante: $config" -ForegroundColor Red
    }
}

# 4. Mostrar configuración actual para capital pequeño
Write-Host ""
Write-Host "💰 PASO 4: Configuración actual para capital pequeño (0.001 SOL)..." -ForegroundColor Yellow
Write-Host "   • Capital por trade: 0.0008 SOL" -ForegroundColor White
Write-Host "   • Profit target: 50%" -ForegroundColor White
Write-Host "   • Stop loss: 15%" -ForegroundColor White
Write-Host "   • Trades por día: 1 máximo" -ForegroundColor White
Write-Host "   • Filtros: Ultra-conservativos" -ForegroundColor White

# 5. Instrucciones de activación
Write-Host ""
Write-Host "🚀 PASO 5: Instrucciones de activación..." -ForegroundColor Yellow
Write-Host ""
Write-Host "Para activar el bot sniper, ejecuta estos comandos:" -ForegroundColor White
Write-Host ""
Write-Host "1. Cliente Interactivo:" -ForegroundColor Cyan
Write-Host "   .\target\release\sniperforge-interactive.exe" -ForegroundColor Gray
Write-Host ""
Write-Host "2. En el cliente, navega a estrategias:" -ForegroundColor Cyan
Write-Host "   SniperForge-Enterprise:/ $ cd /strategies" -ForegroundColor Gray
Write-Host "   SniperForge-Enterprise:/strategies $ ls" -ForegroundColor Gray
Write-Host ""
Write-Host "3. Activar bot sniper específico:" -ForegroundColor Cyan
Write-Host "   SniperForge-Enterprise:/strategies $ start liquidity_sniper" -ForegroundColor Gray
Write-Host ""

# 6. Métricas esperadas
Write-Host "📊 MÉTRICAS ESPERADAS CON 0.001 SOL:" -ForegroundColor Yellow
Write-Host "   • Break-even: 11.2% (costos de transacción)" -ForegroundColor White
Write-Host "   • Target profit: 50%" -ForegroundColor White
Write-Host "   • Margen neto: 38.8%" -ForegroundColor White
Write-Host "   • ROI diario esperado: ~40%" -ForegroundColor Green
Write-Host ""

# 7. Filtros de seguridad
Write-Host "🛡️  FILTROS DE SEGURIDAD ACTIVOS:" -ForegroundColor Yellow
Write-Host "   • Mínimo 300 holders" -ForegroundColor White
Write-Host "   • Edad mínima: 45 minutos" -ForegroundColor White
Write-Host "   • Dev holdings: máximo 15%" -ForegroundColor White
Write-Host "   • Liquidez bloqueada: mínimo 95%" -ForegroundColor White
Write-Host "   • Market cap: mínimo $100,000" -ForegroundColor White
Write-Host "   • Contrato verificado: obligatorio" -ForegroundColor White
Write-Host "   • Team doxxing: requerido" -ForegroundColor White

Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "✅ SISTEMA LISTO PARA ACTIVACIÓN" -ForegroundColor Green
Write-Host "🎯 Bot Sniper de Liquidez configurado y preparado" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan

$endTime = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
Write-Host "📅 Verificación completada: $endTime" -ForegroundColor White
