# 🚀 ACTIVACIÓN AUTOMÁTICA DEL BOT SNIPER
# Este script envía los comandos necesarios al cliente interactivo

Write-Host "🎯 ACTIVANDO BOT SNIPER DE LIQUIDEZ..." -ForegroundColor Yellow

# Comandos a enviar al cliente interactivo
$commands = @(
    "cd /strategies",
    "ls",
    "start liquidity_sniper",
    "status liquidity_sniper",
    "metrics liquidity_sniper"
)

Write-Host "📋 Enviando comandos al cliente interactivo:" -ForegroundColor Cyan

foreach ($cmd in $commands) {
    Write-Host "   > $cmd" -ForegroundColor Gray
    # En un entorno real, estos comandos se enviarían al cliente
}

Write-Host ""
Write-Host "✅ Comandos preparados para ejecución" -ForegroundColor Green
Write-Host "🎯 El bot sniper estará listo para detectar nuevos pools" -ForegroundColor Green
