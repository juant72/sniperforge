# Quick Fix para Settings - Agrega todos los campos básicos que suelen faltar

Write-Host "=== Quick Fix Settings ===" -ForegroundColor Cyan

$configFile = "arbitrage_settings.json"

if (!(Test-Path $configFile)) {
    Write-Host "❌ No se encuentra $configFile" -ForegroundColor Red
    exit 1
}

# Backup
$backupFile = "arbitrage_settings_quickfix_$(Get-Date -Format 'yyyyMMdd_HHmmss').json"
Copy-Item $configFile $backupFile
Write-Host "✓ Backup: $backupFile" -ForegroundColor Green

try {
    $config = Get-Content $configFile -Raw | ConvertFrom-Json
    
    # Asegurar que todas las secciones existen
    $requiredSections = @("trading", "risk_management", "wallet", "ml_analysis", "triangular_arbitrage", "anti_circular")
    
    foreach ($section in $requiredSections) {
        if (!$config.$section) {
            $config | Add-Member -Type NoteProperty -Name $section -Value (New-Object PSObject)
            Write-Host "✓ Sección creada: $section" -ForegroundColor Green
        }
    }
    
    # Trading section - campos críticos
    $tradingFields = @{
        "mode" = "simulation"
        "force_real_transactions" = $false
        "min_profit_threshold_sol" = 0.001
        "min_confidence_threshold" = 0.75
        "military_min_profit_bps" = 20
    }
    
    foreach ($field in $tradingFields.Keys) {
        if (!$config.trading.$field) {
            $config.trading | Add-Member -Type NoteProperty -Name $field -Value $tradingFields[$field] -Force
            Write-Host "✓ trading.$field = $($tradingFields[$field])" -ForegroundColor Green
        }
    }
    
    # Risk Management section
    $riskFields = @{
        "emergency_stop_enabled" = $true
        "max_consecutive_losses" = 3
        "daily_loss_limit_sol" = 1.0
        "position_sizing_model" = "kelly"
        "volatility_threshold" = 25.0
        "correlation_limit" = 0.8
        "max_drawdown_percentage" = 15.0
    }
    
    foreach ($field in $riskFields.Keys) {
        if (!$config.risk_management.$field) {
            $config.risk_management | Add-Member -Type NoteProperty -Name $field -Value $riskFields[$field] -Force
            Write-Host "✓ risk_management.$field = $($riskFields[$field])" -ForegroundColor Green
        }
    }
    
    # Wallet section
    $walletFields = @{
        "keypair_file" = "keypairs/main_wallet.json"
        "backup_keypair_file" = "keypairs/backup_wallet.json"
        "use_env_private_key" = $false
    }
    
    foreach ($field in $walletFields.Keys) {
        if (!$config.wallet.$field) {
            $config.wallet | Add-Member -Type NoteProperty -Name $field -Value $walletFields[$field] -Force
            Write-Host "✓ wallet.$field = $($walletFields[$field])" -ForegroundColor Green
        }
    }
    
    # ML Analysis section
    $mlFields = @{
        "enabled" = $true
        "pattern_recognition_enabled" = $true
        "adaptive_parameters_enabled" = $true
        "ml_confidence_threshold" = 0.8
        "min_score_threshold" = 0.7
    }
    
    foreach ($field in $mlFields.Keys) {
        if (!$config.ml_analysis.$field) {
            $config.ml_analysis | Add-Member -Type NoteProperty -Name $field -Value $mlFields[$field] -Force
            Write-Host "✓ ml_analysis.$field = $($mlFields[$field])" -ForegroundColor Green
        }
    }
    
    # Triangular Arbitrage section
    $triangularFields = @{
        "enabled" = $true
        "max_hops" = 3
        "min_net_profit_bps" = 15
        "circular_detection_enabled" = $true
        "max_same_token_repeats" = 1
    }
    
    foreach ($field in $triangularFields.Keys) {
        if (!$config.triangular_arbitrage.$field) {
            $config.triangular_arbitrage | Add-Member -Type NoteProperty -Name $field -Value $triangularFields[$field] -Force
            Write-Host "✓ triangular_arbitrage.$field = $($triangularFields[$field])" -ForegroundColor Green
        }
    }
    
    # Anti-Circular section
    $antiCircularFields = @{
        "enabled" = $true
        "prevent_same_dex_arbitrage" = $true
        "circular_detection_enabled" = $true
    }
    
    foreach ($field in $antiCircularFields.Keys) {
        if (!$config.anti_circular.$field) {
            $config.anti_circular | Add-Member -Type NoteProperty -Name $field -Value $antiCircularFields[$field] -Force
            Write-Host "✓ anti_circular.$field = $($antiCircularFields[$field])" -ForegroundColor Green
        }
    }
    
    # Guardar configuración actualizada
    $config | ConvertTo-Json -Depth 10 | Out-File -FilePath $configFile -Encoding utf8 -Force
    Write-Host "✅ Configuración guardada en $configFile" -ForegroundColor Green
    
    # Verificar JSON válido
    $testConfig = Get-Content $configFile -Raw | ConvertFrom-Json
    Write-Host "✅ JSON validado correctamente" -ForegroundColor Green
    
} catch {
    Write-Host "❌ Error: $($_.Exception.Message)" -ForegroundColor Red
    # Restaurar backup
    Copy-Item $backupFile $configFile -Force
    Write-Host "🔄 Backup restaurado" -ForegroundColor Yellow
}

Write-Host "`n✅ Quick Fix completado" -ForegroundColor Green
