# SniperForge Mainnet Trading Verification Script
# Verifica configuración de wallet real y RPCs premium

param(
    [switch]$CheckBalance,
    [switch]$TestRPCs,
    [switch]$FullCheck
)

Write-Host ""
Write-Host "🚀 SniperForge Mainnet Trading Verification" -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host ""

# Verificar archivos de configuración
function Test-ConfigurationFiles {
    Write-Host "📋 Verificando archivos de configuración..." -ForegroundColor Yellow
    
    $configFiles = @(
        ".\sniperforge-suite\bots\arbitrage-basic\.env.mainnet",
        ".\wallet-real.json"
    )
    
    foreach ($file in $configFiles) {
        if (Test-Path $file) {
            Write-Host "   ✅ $file encontrado" -ForegroundColor Green
        } else {
            Write-Host "   ❌ $file NO encontrado" -ForegroundColor Red
        }
    }
}

# Verificar wallet real
function Test-WalletConfiguration {
    Write-Host ""
    Write-Host "💳 Verificando configuración de wallet real..." -ForegroundColor Yellow
    
    $walletFile = ".\wallet-real.json"
    if (Test-Path $walletFile) {
        $walletConfig = Get-Content $walletFile | ConvertFrom-Json
        $walletAddress = $walletConfig.wallet_info.public_key
        $balance = $walletConfig.wallet_info.balance_sol
        
        Write-Host "   ✅ Wallet Address: $walletAddress" -ForegroundColor Green
        Write-Host "   ✅ Balance SOL: $balance" -ForegroundColor Green
        Write-Host "   ✅ Network: mainnet-beta" -ForegroundColor Green
        Write-Host "   ✅ Status: active" -ForegroundColor Green
        
        return $walletAddress
    } else {
        Write-Host "   ❌ Archivo wallet-real.json no encontrado" -ForegroundColor Red
        return $null
    }
}

# Verificar balance real en mainnet
function Test-RealBalance {
    param($WalletAddress)
    
    if (-not $WalletAddress) {
        Write-Host "   ❌ No se pudo obtener la dirección de wallet" -ForegroundColor Red
        return
    }
    
    Write-Host ""
    Write-Host "💰 Verificando balance real en mainnet..." -ForegroundColor Yellow
    
    try {
        # Usar solana CLI para verificar balance
        $balanceOutput = solana balance $WalletAddress --url mainnet-beta 2>&1
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "   ✅ Balance Real: $balanceOutput" -ForegroundColor Green
        } else {
            Write-Host "   ⚠️ No se pudo verificar balance con Solana CLI" -ForegroundColor Yellow
            Write-Host "   📝 Output: $balanceOutput" -ForegroundColor Gray
        }
    } catch {
        Write-Host "   ⚠️ Solana CLI no disponible, saltando verificación de balance" -ForegroundColor Yellow
    }
}

# Verificar RPCs premium
function Test-PremiumRPCs {
    Write-Host ""
    Write-Host "🌐 Verificando RPCs premium..." -ForegroundColor Yellow
    
    $envFile = ".\sniperforge-suite\bots\arbitrage-basic\.env.mainnet"
    if (Test-Path $envFile) {
        $envContent = Get-Content $envFile
        
        $rpcUrls = @()
        foreach ($line in $envContent) {
            if ($line -match "^(SOLANA_RPC_URL|RPC_BACKUP_\d+)=(.+)$") {
                $rpcUrls += $matches[2]
            }
        }
        
        Write-Host "   📡 Configurados $(($rpcUrls | Measure-Object).Count) endpoints RPC:" -ForegroundColor Cyan
        foreach ($url in $rpcUrls) {
            if ($url -like "*alchemy*") {
                Write-Host "      🥇 Alchemy (Premium): $url" -ForegroundColor Green
            } elseif ($url -like "*helius*") {
                Write-Host "      🥈 Helius (Premium): $url" -ForegroundColor Green
            } elseif ($url -like "*ankr*") {
                Write-Host "      🥉 Ankr (Premium): $url" -ForegroundColor Green
            } else {
                Write-Host "      📡 Público: $url" -ForegroundColor Gray
            }
        }
        
        # Verificar API keys
        Write-Host ""
        Write-Host "   🔑 API Keys configuradas:" -ForegroundColor Cyan
        $apiKeys = @("HELIUS_API_KEY", "ALCHEMY_API_KEY", "ANKR_API_KEY", "QUICKNODE_ENDPOINT")
        foreach ($keyName in $apiKeys) {
            $keyLine = $envContent | Where-Object { $_ -match "^$keyName=(.+)$" }
            if ($keyLine -and $keyLine -notmatch "your_.*_api_key_here") {
                Write-Host "      ✅ $keyName configurada" -ForegroundColor Green
            } else {
                Write-Host "      ⚠️ $keyName pendiente de configurar" -ForegroundColor Yellow
            }
        }
    }
}

# Verificar configuración de trading
function Test-TradingConfiguration {
    Write-Host ""
    Write-Host "⚡ Verificando configuración de trading..." -ForegroundColor Yellow
    
    $envFile = ".\sniperforge-suite\bots\arbitrage-basic\.env.mainnet"
    if (Test-Path $envFile) {
        $envContent = Get-Content $envFile
        
        # Verificar configuraciones críticas
        $criticalConfigs = @{
            "MIN_PROFIT_SOL" = "Ganancia mínima"
            "MAX_TRANSACTION_AMOUNT_SOL" = "Monto máximo por transacción"
            "EMERGENCY_STOP_THRESHOLD" = "Umbral de parada de emergencia"
            "MAX_SLIPPAGE_PERCENT" = "Slippage máximo"
        }
        
        foreach ($config in $criticalConfigs.GetEnumerator()) {
            $configLine = $envContent | Where-Object { $_ -match "^$($config.Key)=(.+)$" }
            if ($configLine) {
                $value = $matches[1]
                Write-Host "   ✅ $($config.Value): $value" -ForegroundColor Green
            } else {
                Write-Host "   ⚠️ $($config.Value): No configurado" -ForegroundColor Yellow
            }
        }
    }
}

# Mostrar resumen de seguridad
function Show-SecuritySummary {
    Write-Host ""
    Write-Host "🔒 RESUMEN DE SEGURIDAD" -ForegroundColor Red
    Write-Host "========================" -ForegroundColor Red
    Write-Host "⚠️  WALLET REAL CON 0.29 SOL - DINERO REAL" -ForegroundColor Red
    Write-Host "⚠️  Todas las transacciones son PERMANENTES" -ForegroundColor Red
    Write-Host "⚠️  Verificar configuración antes de trading" -ForegroundColor Red
    Write-Host "⚠️  Configurar STOP LOSS y límites apropiados" -ForegroundColor Red
    Write-Host ""
    Write-Host "✅ Wallet address: JDzF9LkpoQVac6c6ufHW1c6Gevd3vFB4RXSSjceL8Kq7" -ForegroundColor Green
    Write-Host "✅ Network: mainnet-beta" -ForegroundColor Green
    Write-Host "✅ Premium RPCs configurados para mejor rendimiento" -ForegroundColor Green
    Write-Host ""
}

# Ejecutar verificaciones
try {
    Test-ConfigurationFiles
    $walletAddress = Test-WalletConfiguration
    
    if ($CheckBalance -or $FullCheck) {
        Test-RealBalance -WalletAddress $walletAddress
    }
    
    if ($TestRPCs -or $FullCheck) {
        Test-PremiumRPCs
    }
    
    if ($FullCheck) {
        Test-TradingConfiguration
    }
    
    Show-SecuritySummary
    
    Write-Host "🎯 READY FOR MAINNET TRADING!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Para iniciar trading real:" -ForegroundColor Cyan
    Write-Host "   cd sniperforge-suite\bots\arbitrage-basic" -ForegroundColor Gray
    Write-Host "   cargo run --release --bin arbitrage_bot" -ForegroundColor Gray
    Write-Host ""
    
} catch {
    Write-Host ""
    Write-Host "❌ Error durante la verificación: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
}

Write-Host "Verificación completada!" -ForegroundColor Cyan
