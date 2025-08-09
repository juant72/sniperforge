# SniperForge - Test del Sistema Completo
# Script para probar todas las funcionalidades antes del trading real

param(
    [switch]$QuickTest,
    [switch]$FullTest,
    [switch]$ProductionTest
)

Write-Host ""
Write-Host "🧪 SniperForge - Testing del Sistema" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan
Write-Host ""

function Test-Compilation {
    Write-Host "🔨 Probando compilación..." -ForegroundColor Yellow
    
    try {
        $result = cargo check --all-targets 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "   ✅ Compilación exitosa" -ForegroundColor Green
            return $true
        } else {
            Write-Host "   ❌ Error de compilación:" -ForegroundColor Red
            Write-Host "   $result" -ForegroundColor Gray
            return $false
        }
    } catch {
        Write-Host "   ❌ Error ejecutando cargo check: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

function Test-Configuration {
    Write-Host "⚙️  Probando configuración mainnet..." -ForegroundColor Yellow
    
    $configFiles = @(
        "bots\arbitrage-basic\.env.mainnet",
        "bots\arbitrage-basic\wallet-real.json"
    )
    
    $allValid = $true
    foreach ($file in $configFiles) {
        if (Test-Path $file) {
            Write-Host "   ✅ $file existe" -ForegroundColor Green
        } else {
            Write-Host "   ❌ $file NO encontrado" -ForegroundColor Red
            $allValid = $false
        }
    }
    
    # Verificar contenido de configuración
    if (Test-Path "bots\arbitrage-basic\.env.mainnet") {
        $envContent = Get-Content "bots\arbitrage-basic\.env.mainnet" -Raw
        if ($envContent -like "*JDzF9LkpoQVac6c6ufHW1c6Gevd3vFB4RXSSjceL8Kq7*") {
            Write-Host "   ✅ Wallet real configurada" -ForegroundColor Green
        } else {
            Write-Host "   ⚠️ Wallet real no encontrada en configuración" -ForegroundColor Yellow
            $allValid = $false
        }
    }
    
    return $allValid
}

function Test-Dependencies {
    Write-Host "📦 Probando dependencias..." -ForegroundColor Yellow
    
    try {
        $result = cargo tree --depth 1 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "   ✅ Dependencias resueltas correctamente" -ForegroundColor Green
            return $true
        } else {
            Write-Host "   ❌ Error con dependencias: $result" -ForegroundColor Red
            return $false
        }
    } catch {
        Write-Host "   ❌ Error verificando dependencias: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

function Test-WalletConnection {
    Write-Host "💳 Probando conexión de wallet..." -ForegroundColor Yellow
    
    if (Test-Path "bots\arbitrage-basic\wallet-real.json") {
        try {
            $walletContent = Get-Content "bots\arbitrage-basic\wallet-real.json" | ConvertFrom-Json
            $walletAddress = $walletContent.wallet_info.public_key
            
            if ($walletAddress -eq "JDzF9LkpoQVac6c6ufHW1c6Gevd3vFB4RXSSjceL8Kq7") {
                Write-Host "   ✅ Wallet address verificada: $walletAddress" -ForegroundColor Green
                Write-Host "   ✅ Balance esperado: 0.29 SOL" -ForegroundColor Green
                return $true
            } else {
                Write-Host "   ❌ Wallet address incorrecta: $walletAddress" -ForegroundColor Red
                return $false
            }
        } catch {
            Write-Host "   ❌ Error leyendo wallet: $($_.Exception.Message)" -ForegroundColor Red
            return $false
        }
    } else {
        Write-Host "   ❌ Archivo de wallet no encontrado" -ForegroundColor Red
        return $false
    }
}

function Test-RPCEndpoints {
    Write-Host "🌐 Probando endpoints RPC..." -ForegroundColor Yellow
    
    $envFile = "bots\arbitrage-basic\.env.mainnet"
    if (Test-Path $envFile) {
        $envContent = Get-Content $envFile
        
        $rpcCount = 0
        $premiumCount = 0
        
        foreach ($line in $envContent) {
            if ($line -match "^(SOLANA_RPC_URL|RPC_BACKUP_\d+)=(.+)$") {
                $rpcCount++
                $url = $matches[2]
                if ($url -like "*alchemy*" -or $url -like "*helius*" -or $url -like "*ankr*") {
                    $premiumCount++
                }
            }
        }
        
        Write-Host "   ✅ RPCs configurados: $rpcCount total" -ForegroundColor Green
        Write-Host "   ✅ RPCs premium: $premiumCount" -ForegroundColor Green
        
        if ($rpcCount -ge 3) {
            return $true
        } else {
            Write-Host "   ⚠️ Pocos RPCs configurados (recomendado: 3+)" -ForegroundColor Yellow
            return $false
        }
    } else {
        Write-Host "   ❌ Archivo de configuración no encontrado" -ForegroundColor Red
        return $false
    }
}

function Test-BuildProduction {
    Write-Host "🏗️ Probando build de producción..." -ForegroundColor Yellow
    
    try {
        Write-Host "   🔄 Compilando en modo release..." -ForegroundColor Cyan
        $result = cargo build --release 2>&1
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "   ✅ Build de producción exitoso" -ForegroundColor Green
            
            # Verificar que el binario existe
            if (Test-Path "target\release\arbitrage-basic.exe") {
                $fileInfo = Get-Item "target\release\arbitrage-basic.exe"
                $sizeMB = [math]::Round($fileInfo.Length / 1MB, 2)
                Write-Host "   ✅ Binario creado: $sizeMB MB" -ForegroundColor Green
                return $true
            } else {
                Write-Host "   ❌ Binario no encontrado después del build" -ForegroundColor Red
                return $false
            }
        } else {
            Write-Host "   ❌ Error en build de producción:" -ForegroundColor Red
            Write-Host "   $result" -ForegroundColor Gray
            return $false
        }
    } catch {
        Write-Host "   ❌ Error durante build: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

function Show-TestResults {
    param($results)
    
    Write-Host ""
    Write-Host "📊 RESULTADOS DEL TESTING" -ForegroundColor Cyan
    Write-Host "=========================" -ForegroundColor Cyan
    
    $totalTests = $results.Count
    $passedTests = ($results.Values | Where-Object { $_ -eq $true }).Count
    $failedTests = $totalTests - $passedTests
    
    foreach ($test in $results.GetEnumerator()) {
        $status = if ($test.Value) { "✅ PASS" } else { "❌ FAIL" }
        $color = if ($test.Value) { "Green" } else { "Red" }
        Write-Host "   $($test.Key): $status" -ForegroundColor $color
    }
    
    Write-Host ""
    Write-Host "📈 Resumen: $passedTests/$totalTests tests pasaron" -ForegroundColor Cyan
    
    if ($failedTests -eq 0) {
        Write-Host "🎉 ¡TODOS LOS TESTS PASARON!" -ForegroundColor Green
        Write-Host "🚀 Sistema listo para trading en mainnet" -ForegroundColor Green
        return $true
    } else {
        Write-Host "⚠️ $failedTests tests fallaron" -ForegroundColor Yellow
        Write-Host "🔧 Revisar configuración antes de trading" -ForegroundColor Yellow
        return $false
    }
}

# Ejecutar tests según parámetros
$results = @{}

if ($QuickTest -or (-not $FullTest -and -not $ProductionTest)) {
    Write-Host "🏃 Ejecutando Quick Test..." -ForegroundColor Cyan
    Write-Host ""
    
    $results["Compilación"] = Test-Compilation
    $results["Configuración"] = Test-Configuration
    $results["Wallet"] = Test-WalletConnection
}

if ($FullTest) {
    Write-Host "🔍 Ejecutando Full Test..." -ForegroundColor Cyan
    Write-Host ""
    
    $results["Compilación"] = Test-Compilation
    $results["Configuración"] = Test-Configuration
    $results["Dependencias"] = Test-Dependencies
    $results["Wallet"] = Test-WalletConnection
    $results["RPCs"] = Test-RPCEndpoints
}

if ($ProductionTest) {
    Write-Host "🏭 Ejecutando Production Test..." -ForegroundColor Cyan
    Write-Host ""
    
    $results["Compilación"] = Test-Compilation
    $results["Configuración"] = Test-Configuration
    $results["Dependencias"] = Test-Dependencies
    $results["Wallet"] = Test-WalletConnection
    $results["RPCs"] = Test-RPCEndpoints
    $results["Build Producción"] = Test-BuildProduction
}

$allPassed = Show-TestResults $results

Write-Host ""
if ($allPassed) {
    Write-Host "🎯 READY FOR TRADING!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Para iniciar trading:" -ForegroundColor Cyan
    Write-Host "   cargo run --release" -ForegroundColor Gray
    Write-Host "   # o usar el binario directamente:" -ForegroundColor Gray
    Write-Host "   target\release\arbitrage-basic.exe" -ForegroundColor Gray
} else {
    Write-Host "🚨 SISTEMA NO LISTO" -ForegroundColor Red
    Write-Host "   Resolver errores antes de trading" -ForegroundColor Yellow
}
Write-Host ""
