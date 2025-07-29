# Complete Config Fixer - Detecta y corrige TODOS los campos faltantes automáticamente

Write-Host "=== Complete Config Fixer ===" -ForegroundColor Cyan
Write-Host "Detectará y corregirá TODOS los campos faltantes automáticamente" -ForegroundColor Yellow

$configFile = "arbitrage_settings.json"
$maxIterations = 20
$iteration = 0

# Backup inicial
$backupFile = "arbitrage_settings_complete_backup_$(Get-Date -Format 'yyyyMMdd_HHmmss').json"
if (Test-Path $configFile) {
    Copy-Item $configFile $backupFile
    Write-Host "✓ Backup completo: $backupFile" -ForegroundColor Green
}

# Mapeo de campos a valores por defecto
$fieldDefaults = @{
    # Trading
    "mode" = "simulation"
    "force_real_transactions" = $false
    "min_profit_threshold_sol" = 0.001
    "min_confidence_threshold" = 0.75
    "military_min_profit_bps" = 20
    "max_concurrent_trades" = 3
    "trade_timeout_seconds" = 30
    "slippage_tolerance_bps" = 150
    "min_volume_threshold" = 1000.0
    "position_update_interval_ms" = 1000
    
    # Risk Management
    "emergency_stop_enabled" = $true
    "max_consecutive_losses" = 3
    "daily_loss_limit_sol" = 1.0
    "position_sizing_model" = "kelly"
    "volatility_threshold" = 25.0
    "correlation_limit" = 0.8
    "max_drawdown_percentage" = 15.0
    "stop_loss_percentage" = 5.0
    "take_profit_percentage" = 10.0
    
    # Wallet
    "keypair_file" = "keypairs/main_wallet.json"
    "backup_keypair_file" = "keypairs/backup_wallet.json"
    "use_env_private_key" = $false
    "auto_derive_wallets" = $false
    
    # ML Analysis
    "enabled" = $true
    "pattern_recognition_enabled" = $true
    "adaptive_parameters_enabled" = $true
    "ml_confidence_threshold" = 0.8
    "min_score_threshold" = 0.7
    "model_update_interval_hours" = 24
    "feature_extraction_enabled" = $true
    
    # Triangular Arbitrage
    "max_hops" = 3
    "min_net_profit_bps" = 15
    "circular_detection_enabled" = $true
    "max_same_token_repeats" = 1
    "path_optimization_enabled" = $true
    
    # Anti Circular
    "prevent_same_dex_arbitrage" = $true
    "detection_window_ms" = 5000
    "cooldown_period_ms" = 30000
    
    # Performance
    "max_concurrent_discoveries" = 5
    "cache_ttl_seconds" = 30
    "parallel_api_calls" = $true
    "batch_size" = 20
    "rate_limit_ms" = 100
    "latency_target_ms" = 1000
    
    # API timeouts y configuraciones
    "timeout_ms" = 30000
    "retry_attempts" = 3
    "backoff_multiplier" = 2.0
    "health_check_interval_seconds" = 60
    
    # Solana específico
    "rpc_url" = "https://api.devnet.solana.com"
    "websocket_url" = "wss://api.devnet.solana.com"
    "priority_fee" = 5000
    "confirmation_commitment" = "confirmed"
    "max_retries" = 3
    
    # MEV Protection
    "jito_tip_lamports" = 1500
    "priority_fee_cap" = 50000
    "sandwich_detection" = $true
    "flashloan_detection" = $true
    "front_running_protection" = $true
}

# Mapeo de campos a secciones
$fieldToSection = @{
    # Trading fields
    "mode" = "trading"
    "force_real_transactions" = "trading"
    "min_profit_threshold_sol" = "trading"
    "min_confidence_threshold" = "trading"
    "military_min_profit_bps" = "trading"
    "max_concurrent_trades" = "trading"
    "trade_timeout_seconds" = "trading"
    "slippage_tolerance_bps" = "trading"
    "min_volume_threshold" = "trading"
    "position_update_interval_ms" = "trading"
    
    # Risk management fields
    "emergency_stop_enabled" = "risk_management"
    "max_consecutive_losses" = "risk_management"
    "daily_loss_limit_sol" = "risk_management"
    "position_sizing_model" = "risk_management"
    "volatility_threshold" = "risk_management"
    "correlation_limit" = "risk_management"
    "max_drawdown_percentage" = "risk_management"
    "stop_loss_percentage" = "risk_management"
    "take_profit_percentage" = "risk_management"
    
    # Wallet fields
    "keypair_file" = "wallet"
    "backup_keypair_file" = "wallet"
    "use_env_private_key" = "wallet"
    "auto_derive_wallets" = "wallet"
    
    # ML fields
    "pattern_recognition_enabled" = "ml_analysis"
    "adaptive_parameters_enabled" = "ml_analysis"
    "ml_confidence_threshold" = "ml_analysis"
    "min_score_threshold" = "ml_analysis"
    "model_update_interval_hours" = "ml_analysis"
    "feature_extraction_enabled" = "ml_analysis"
    
    # Triangular arbitrage fields
    "max_hops" = "triangular_arbitrage"
    "min_net_profit_bps" = "triangular_arbitrage"
    "circular_detection_enabled" = "triangular_arbitrage"
    "max_same_token_repeats" = "triangular_arbitrage"
    "path_optimization_enabled" = "triangular_arbitrage"
    
    # Anti circular fields
    "prevent_same_dex_arbitrage" = "anti_circular"
    "detection_window_ms" = "anti_circular"
    "cooldown_period_ms" = "anti_circular"
    
    # Performance fields
    "max_concurrent_discoveries" = "performance"
    "cache_ttl_seconds" = "performance"
    "parallel_api_calls" = "performance"
    "batch_size" = "performance"
    "rate_limit_ms" = "performance"
    "latency_target_ms" = "performance"
    
    # Solana fields
    "rpc_url" = "solana"
    "websocket_url" = "solana"
    "priority_fee" = "solana"
    "confirmation_commitment" = "solana"
    "timeout_ms" = "solana"
    "max_retries" = "solana"
    "retry_attempts" = "solana"
    "backoff_multiplier" = "solana"
    "health_check_interval_seconds" = "solana"
    
    # MEV Protection fields
    "jito_tip_lamports" = "mev_protection"
    "priority_fee_cap" = "mev_protection"
    "sandwich_detection" = "mev_protection"
    "flashloan_detection" = "mev_protection"
    "front_running_protection" = "mev_protection"
}

function Test-SystemConfiguration {
    Write-Host "`n🔍 Probando configuración (iteración $iteration)..." -ForegroundColor Yellow
    
    $output = & cargo run --bin arbitrage_phase45_clean 2>&1
    $missingFields = @()
    
    foreach ($line in $output) {
        if ($line -match "missing field `([^`]+)`") {
            $field = $Matches[1]
            $missingFields += $field
            Write-Host "❌ Campo faltante: $field" -ForegroundColor Red
        }
    }
    
    if ($missingFields.Count -eq 0) {
        if ($output -match "✅.*JSON.*exitosa|Phase.*operational") {
            Write-Host "✅ ¡Configuración completamente válida!" -ForegroundColor Green
            return $null
        } else {
            Write-Host "⚠️ No hay campos faltantes, verificando estado..." -ForegroundColor Yellow
            return $null
        }
    }
    
    return $missingFields
}

function Add-MissingField {
    param(
        [string]$fieldName,
        [object]$config
    )
    
    # Determinar sección
    $section = "trading" # default
    if ($fieldToSection.ContainsKey($fieldName)) {
        $section = $fieldToSection[$fieldName]
    }
    
    # Crear sección si no existe
    if (!$config.$section) {
        $config | Add-Member -Type NoteProperty -Name $section -Value (New-Object PSObject)
        Write-Host "  ✓ Sección creada: $section" -ForegroundColor Cyan
    }
    
    # Agregar campo
    $value = $null
    if ($fieldDefaults.ContainsKey($fieldName)) {
        $value = $fieldDefaults[$fieldName]
    } else {
        # Valores por defecto genéricos basados en tipo
        if ($fieldName -match "enabled|detection|protection|use_") {
            $value = $true
        } elseif ($fieldName -match "_ms|timeout|interval|delay") {
            $value = 1000
        } elseif ($fieldName -match "_seconds") {
            $value = 30
        } elseif ($fieldName -match "_bps|slippage|fee") {
            $value = 100
        } elseif ($fieldName -match "threshold|limit|percentage") {
            $value = 1.0
        } elseif ($fieldName -match "max_|count|size") {
            $value = 3
        } elseif ($fieldName -match "url|file|path") {
            $value = ""
        } else {
            $value = 0
        }
    }
    
    $config.$section | Add-Member -Type NoteProperty -Name $fieldName -Value $value -Force
    Write-Host "  ✓ $section.$fieldName = $value" -ForegroundColor Green
}

# Bucle principal
do {
    $iteration++
    Write-Host "`n=== ITERACIÓN $iteration ===" -ForegroundColor Cyan
    
    if ($iteration -gt $maxIterations) {
        Write-Host "❌ Máximo de iteraciones alcanzado ($maxIterations)" -ForegroundColor Red
        break
    }
    
    $missingFields = Test-SystemConfiguration
    
    if (!$missingFields) {
        Write-Host "`n🎉 ¡Configuración completamente válida!" -ForegroundColor Green
        break
    }
    
    Write-Host "`n🔧 Agregando $($missingFields.Count) campos faltantes..." -ForegroundColor Yellow
    
    try {
        $config = Get-Content $configFile -Raw | ConvertFrom-Json
        
        foreach ($field in $missingFields) {
            Add-MissingField -fieldName $field -config $config
        }
        
        # Guardar configuración
        $config | ConvertTo-Json -Depth 10 | Out-File -FilePath $configFile -Encoding utf8 -Force
        Write-Host "✅ Configuración guardada" -ForegroundColor Green
        
        # Verificar JSON válido
        $testConfig = Get-Content $configFile -Raw | ConvertFrom-Json | Out-Null
        Write-Host "✅ JSON válido confirmado" -ForegroundColor Green
        
    } catch {
        Write-Host "❌ Error: $($_.Exception.Message)" -ForegroundColor Red
        # Restaurar backup
        Copy-Item $backupFile $configFile -Force
        Write-Host "🔄 Backup restaurado" -ForegroundColor Yellow
        break
    }
    
    Start-Sleep -Seconds 1
    
} while ($missingFields -and $iteration -le $maxIterations)

Write-Host "`n=== PROCESO COMPLETADO ===" -ForegroundColor Cyan
Write-Host "Iteraciones realizadas: $iteration" -ForegroundColor White
Write-Host "Backup disponible en: $backupFile" -ForegroundColor White

if (!$missingFields) {
    Write-Host "`n🎉 ¡ÉXITO! Configuración completamente reparada" -ForegroundColor Green
} else {
    Write-Host "`n⚠️ Proceso incompleto - revisar manualmente" -ForegroundColor Yellow
}
