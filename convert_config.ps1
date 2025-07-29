# Convert Config Script
# Convierte arbitrage_settings.config a arbitrage_settings.json para el sistema

param(
    [switch]$Validate,
    [switch]$Backup
)

$configFile = "arbitrage_settings.config"
$jsonFile = "arbitrage_settings.json"
$backupDir = "config_backups"

Write-Host "=== Arbitrage Settings Converter ===" -ForegroundColor Cyan

# Crear directorio de respaldos si no existe
if ($Backup -and !(Test-Path $backupDir)) {
    New-Item -ItemType Directory -Path $backupDir -Force | Out-Null
    Write-Host "✓ Directorio de respaldos creado" -ForegroundColor Green
}

# Hacer respaldo si se solicita
if ($Backup -and (Test-Path $jsonFile)) {
    $timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
    $backupFile = "$backupDir/arbitrage_settings_$timestamp.json"
    Copy-Item $jsonFile $backupFile
    Write-Host "✓ Respaldo creado: $backupFile" -ForegroundColor Green
}

# Verificar que existe el archivo .config
if (!(Test-Path $configFile)) {
    Write-Host "❌ Error: No se encuentra $configFile" -ForegroundColor Red
    exit 1
}

try {
    # Leer y validar JSON del archivo .config
    $content = Get-Content $configFile -Raw
    $configData = $content | ConvertFrom-Json
    
    Write-Host "✓ Archivo .config leído correctamente" -ForegroundColor Green
    
    # Validaciones específicas
    $requiredSections = @("solana", "trading", "risk_management", "performance")
    $missingSection = $false
    
    foreach ($section in $requiredSections) {
        if (!$configData.$section) {
            Write-Host "❌ Sección faltante: $section" -ForegroundColor Red
            $missingSection = $true
        }
    }
    
    if ($missingSection) {
        Write-Host "❌ Configuración incompleta" -ForegroundColor Red
        exit 1
    }
    
    # Validaciones críticas
    if (!$configData.risk_management.emergency_stop_enabled) {
        Write-Host "⚠️  emergency_stop_enabled no está habilitado" -ForegroundColor Yellow
    }
    
    if (!$configData.trading.PSObject.Properties['force_real_transactions']) {
        Write-Host "⚠️  force_real_transactions no está definido" -ForegroundColor Yellow
    }
    
    if ($configData.trading.max_trade_sol -gt 0.1) {
        Write-Host "⚠️  max_trade_sol parece alto: $($configData.trading.max_trade_sol)" -ForegroundColor Yellow
    }
    
    # Solo validar si se solicita
    if ($Validate) {
        Write-Host "✓ Validación completada - configuración válida" -ForegroundColor Green
        return
    }
    
    # Convertir a JSON con formato bonito
    $jsonContent = $configData | ConvertTo-Json -Depth 10 -Compress:$false
    
    # Escribir archivo JSON
    $jsonContent | Out-File -FilePath $jsonFile -Encoding utf8 -Force
    
    Write-Host "✓ Archivo convertido: $configFile → $jsonFile" -ForegroundColor Green
    
    # Verificar que el JSON generado es válido
    $testJson = Get-Content $jsonFile -Raw | ConvertFrom-Json
    Write-Host "✓ JSON generado validado correctamente" -ForegroundColor Green
    
    # Mostrar resumen de configuración
    Write-Host "`n=== Resumen de Configuración ===" -ForegroundColor Cyan
    Write-Host "• Trading habilitado: $($configData.trading.enabled)" -ForegroundColor White
    Write-Host "• Modo: $($configData.trading.mode)" -ForegroundColor White
    Write-Host "• Force real transactions: $($configData.trading.force_real_transactions)" -ForegroundColor White
    Write-Host "• Máximo trade SOL: $($configData.trading.max_trade_sol)" -ForegroundColor White
    Write-Host "• Protección MEV: $($configData.mev_protection.enabled)" -ForegroundColor White
    Write-Host "• Emergency stop: $($configData.risk_management.emergency_stop_enabled)" -ForegroundColor White
    Write-Host "• Oportunidades concurrentes: $($configData.performance.max_concurrent_opportunities)" -ForegroundColor White
    
} catch {
    Write-Host "❌ Error procesando configuración: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

Write-Host "`n✅ Conversión completada exitosamente" -ForegroundColor Green
