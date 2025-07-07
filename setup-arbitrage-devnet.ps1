#!/usr/bin/env pwsh
# Preparación completa para testing del ArbitrageBot en DevNet

Write-Host "🚀 Preparando entorno DevNet para ArbitrageBot" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan

# 1. Verificar configuración
Write-Host "📋 Paso 1: Verificando configuración..." -ForegroundColor Yellow
if (Test-Path "config/devnet.toml") {
    Write-Host "✅ Configuración DevNet encontrada" -ForegroundColor Green
} else {
    Write-Host "❌ Configuración DevNet no encontrada" -ForegroundColor Red
    exit 1
}

# 2. Verificar wallet de testing
Write-Host "💳 Paso 2: Verificando wallet de testing..." -ForegroundColor Yellow
if (Test-Path "test-wallet.json") {
    Write-Host "✅ Wallet de testing encontrado" -ForegroundColor Green
} else {
    Write-Host "⚠️  Wallet de testing no encontrado. Creando..." -ForegroundColor Yellow
    cargo run --bin create_test_wallet
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Wallet de testing creado" -ForegroundColor Green
    } else {
        Write-Host "❌ Error creando wallet de testing" -ForegroundColor Red
        exit 1
    }
}

# 3. Verificar balance de SOL en DevNet
Write-Host "💰 Paso 3: Verificando balance DevNet..." -ForegroundColor Yellow
cargo run --bin check_wallet_balance
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  Solicitando airdrop de DevNet..." -ForegroundColor Yellow
    cargo run --bin request_devnet_airdrop
    Start-Sleep -Seconds 5
    cargo run --bin check_wallet_balance
}

# 4. Compilar el proyecto
Write-Host "🔨 Paso 4: Compilando proyecto..." -ForegroundColor Yellow
cargo check --lib
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Error de compilación" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Compilación exitosa" -ForegroundColor Green

# 5. Verificar conectividad DevNet
Write-Host "🌐 Paso 5: Verificando conectividad DevNet..." -ForegroundColor Yellow
cargo run --bin test_basic_connectivity
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Error de conectividad DevNet" -ForegroundColor Red
    exit 1
}

Write-Host "🎉 Preparación completa! Listo para testing" -ForegroundColor Green
Write-Host "
📋 COMANDOS DE TESTING:
   cargo run --bin test_arbitrage_devnet        # Test completo del ArbitrageBot
   cargo run -- start --bot arbitrage --network devnet  # Iniciar bot via CLI

⚠️  IMPORTANTE:
   - Usa DevNet solamente (sin riesgo financiero)
   - El bot usará cantidades pequeñas para testing
   - Monitorea los logs para verificar funcionamiento

🔍 MONITOREO:
   - Verifica precios obtenidos de Jupiter API
   - Observa detección de oportunidades
   - Confirma que no hay errores de conectividad
" -ForegroundColor Cyan
