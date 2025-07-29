# Auto-Complete Settings Configuration
# Detecta automáticamente campos faltantes y los agrega con valores por defecto

param(
    [switch]$AutoFix,
    [switch]$DryRun
)

Write-Host "=== Auto-Complete Settings Configuration ===" -ForegroundColor Cyan

$configFile = "arbitrage_settings.json"
$backupFile = "arbitrage_settings_backup_$(Get-Date -Format 'yyyyMMdd_HHmmss').json"

# Hacer backup antes de cualquier cambio
if (Test-Path $configFile) {
    Copy-Item $configFile $backupFile
    Write-Host "✓ Backup creado: $backupFile" -ForegroundColor Green
}

# Función para detectar campos faltantes ejecutando el sistema
function Test-Configuration {
    Write-Host "`n🔍 Probando configuración actual..." -ForegroundColor Yellow
    
    $output = & cargo run --bin arbitrage_phase45_clean 2>&1
    $errors = @()
    
    foreach ($line in $output) {
        if ($line -match "missing field `([^`]+)`") {
            $missingField = $Matches[1]
            $errors += $missingField
            Write-Host "❌ Campo faltante detectado: $missingField" -ForegroundColor Red
        }
    }
    
    if ($errors.Count -eq 0) {
        if ($output -match "✅.*JSON.*exitosa") {
            Write-Host "✅ Configuración completamente válida" -ForegroundColor Green
            return $null
        } else {
            Write-Host "⚠️  No se detectaron campos faltantes, pero revisa la salida:" -ForegroundColor Yellow
            Write-Host ($output | Select-Object -Last 3) -ForegroundColor Gray
            return $null
        }
    }
    
    return $errors
}

# Campos por defecto para completar automáticamente
$defaultValues = @{
    "mode" = "simulation"
    "force_real_transactions" = $false
    "min_profit_threshold_sol" = 0.001
    "min_confidence_threshold" = 0.75
    "emergency_stop_enabled" = $true
    "max_consecutive_losses" = 3
    "daily_loss_limit_sol" = 1.0
    "position_sizing_model" = "kelly"
    "volatility_threshold" = 25.0
    "correlation_limit" = 0.8
    "max_drawdown_percentage" = 15.0
    "wallet_keypair_file" = "keypairs/main_wallet.json"
    "backup_keypair_file" = "keypairs/backup_wallet.json"
    "use_env_private_key" = $false
    "enable_ml_prediction" = $true
    "pattern_recognition_enabled" = $true
    "adaptive_parameters_enabled" = $true
    "ml_confidence_threshold" = 0.8
    "min_score_threshold" = 0.7
    "enabled" = $true
    "max_hops" = 3
    "min_net_profit_bps" = 15
    "circular_detection_enabled" = $true
    "max_same_token_repeats" = 1
    "prevent_same_dex_arbitrage" = $true
    "military_min_profit_bps" = 20
}

# Secciones típicas donde pueden faltar campos
$sections = @{
    "trading" = @("mode", "force_real_transactions", "min_profit_threshold_sol", "min_confidence_threshold", "military_min_profit_bps")
    "risk_management" = @("emergency_stop_enabled", "max_consecutive_losses", "daily_loss_limit_sol", "position_sizing_model", "volatility_threshold", "correlation_limit", "max_drawdown_percentage")
    "wallet" = @("wallet_keypair_file", "backup_keypair_file", "use_env_private_key")
    "ml_analysis" = @("enabled", "pattern_recognition_enabled", "adaptive_parameters_enabled", "ml_confidence_threshold", "min_score_threshold")
    "triangular_arbitrage" = @("enabled", "max_hops", "min_net_profit_bps", "circular_detection_enabled", "max_same_token_repeats")
    "anti_circular" = @("enabled", "prevent_same_dex_arbitrage", "circular_detection_enabled")
}

# Función para agregar campos faltantes
function Add-MissingFields {
    param(
        [string[]]$missingFields
    )
    
    if (!$missingFields -or $missingFields.Count -eq 0) {
        return
    }
    
    Write-Host "`n🔧 Agregando campos faltantes..." -ForegroundColor Yellow
    
    try {
        $config = Get-Content $configFile -Raw | ConvertFrom-Json
        $modified = $false
        
        foreach ($field in $missingFields) {
            Write-Host "  • Agregando campo: $field" -ForegroundColor Gray
            
            # Determinar en qué sección agregar el campo
            $sectionToModify = $null
            foreach ($sectionName in $sections.Keys) {
                if ($sections[$sectionName] -contains $field) {
                    $sectionToModify = $sectionName
                    break
                }
            }
            
            if (!$sectionToModify) {
                # Intentar adivinar la sección basándose en el nombre del campo
                if ($field -match "trading|trade|profit|slippage|volume") {
                    $sectionToModify = "trading"
                } elseif ($field -match "risk|loss|emergency|drawdown") {
                    $sectionToModify = "risk_management"  
                } elseif ($field -match "wallet|keypair|key") {
                    $sectionToModify = "wallet"
                } elseif ($field -match "ml|confidence|pattern|adaptive") {
                    $sectionToModify = "ml_analysis"
                } elseif ($field -match "triangular|hop|circular") {
                    $sectionToModify = "triangular_arbitrage"
                } else {
                    $sectionToModify = "trading" # default
                }
            }
            
            # Crear sección si no existe
            if (!$config.$sectionToModify) {
                $config | Add-Member -Type NoteProperty -Name $sectionToModify -Value (New-Object PSObject)
            }
            
            # Agregar campo con valor por defecto
            if ($defaultValues.ContainsKey($field)) {
                $config.$sectionToModify | Add-Member -Type NoteProperty -Name $field -Value $defaultValues[$field] -Force
                $modified = $true
                Write-Host "    ✓ Agregado $field = $($defaultValues[$field])" -ForegroundColor Green
            } else {
                Write-Host "    ⚠️ No se encontró valor por defecto para: $field" -ForegroundColor Yellow
            }
        }
        
        if ($modified) {
            if (!$DryRun) {
                $config | ConvertTo-Json -Depth 10 | Out-File -FilePath $configFile -Encoding utf8 -Force
                Write-Host "✅ Configuración actualizada en $configFile" -ForegroundColor Green
            } else {
                Write-Host "🔍 DRY RUN: No se guardaron cambios" -ForegroundColor Yellow
            }
        }
        
    } catch {
        Write-Host "❌ Error procesando configuración: $($_.Exception.Message)" -ForegroundColor Red
    }
}

# Ejecutar proceso principal
if ($DryRun) {
    Write-Host "🔍 MODO DRY RUN - No se realizarán cambios" -ForegroundColor Yellow
}

do {
    $missingFields = Test-Configuration
    
    if ($missingFields) {
        Write-Host "`n📝 Campos faltantes detectados: $($missingFields -join ', ')" -ForegroundColor Yellow
        
        if ($AutoFix) {
            Add-MissingFields -missingFields $missingFields
            Write-Host "`n🔄 Reintentando validación..." -ForegroundColor Cyan
        } else {
            Write-Host "`n💡 Ejecuta con -AutoFix para agregar automáticamente" -ForegroundColor Cyan
            break
        }
    } else {
        Write-Host "`n✅ Configuración completamente validada" -ForegroundColor Green
        break
    }
    
    # Evitar bucle infinito
    Start-Sleep -Seconds 1
    
} while ($AutoFix -and $missingFields)

Write-Host "`n=== Proceso Completado ===" -ForegroundColor Cyan
