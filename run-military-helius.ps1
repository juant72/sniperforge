#!/usr/bin/env pwsh
# ===== MILITARY ARBITRAGE SYSTEM CON HELIUS PREMIUM =====
# Script para ejecutar el sistema de arbitraje militar con Helius Premium

Write-Host "⚔️  === MILITARY ARBITRAGE SYSTEM - HELIUS PREMIUM ===" -ForegroundColor Yellow
Write-Host "   Sistema de arbitraje militar con acceso premium a Solana" -ForegroundColor Gray
Write-Host ""

# Verificar configuración de Helius
$heliusKey = $env:HELIUS_API_KEY
$hasHelius = $heliusKey -ne $null -and $heliusKey -ne ""

Write-Host "🔍 === VERIFICACIÓN DE CONFIGURACIÓN ===" -ForegroundColor Cyan
Write-Host "   Helius API Key: " -NoNewline
if ($hasHelius) { 
    Write-Host "✅ Configurada" -ForegroundColor Green
    Write-Host "      Value: $($heliusKey.Substring(0, [Math]::Min(8, $heliusKey.Length)))..." -ForegroundColor DarkGray
} else { 
    Write-Host "❌ No configurada" -ForegroundColor Red
}

Write-Host "   Solana RPC URL: " -NoNewline
if ($env:SOLANA_RPC_URL) {
    Write-Host "✅ Configurada" -ForegroundColor Green
    Write-Host "      Value: $($env:SOLANA_RPC_URL.Split('?')[0])..." -ForegroundColor DarkGray
} else {
    Write-Host "❌ No configurada" -ForegroundColor Red
}

Write-Host "   Wallet: " -NoNewline
if (Test-Path "mainnet_wallet.json") {
    Write-Host "✅ Encontrada" -ForegroundColor Green
} else {
    Write-Host "❌ No encontrada" -ForegroundColor Red
}
Write-Host ""

# Verificar si Helius está configurado
if (-not $hasHelius) {
    Write-Host "⚠️  HELIUS API KEY NO CONFIGURADA" -ForegroundColor Yellow
    Write-Host "   Para obtener pools activos reales, configure su API key:" -ForegroundColor Gray
    Write-Host ""
    Write-Host "   1. Obtenga una API key de Helius: https://helius.xyz" -ForegroundColor Cyan
    Write-Host "   2. Ejecute: .\setup-helius-premium.ps1 -ApiKey 'su-api-key'" -ForegroundColor Cyan
    Write-Host "   3. Reinicie el terminal y ejecute este script nuevamente" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "   Continuando con RPC estándar..." -ForegroundColor Yellow
    Write-Host ""
}

# Verificar wallet
if (-not (Test-Path "mainnet_wallet.json")) {
    Write-Host "❌ WALLET NO ENCONTRADA" -ForegroundColor Red
    Write-Host "   Cree un wallet primero:" -ForegroundColor Gray
    Write-Host "   cargo run --bin generate_wallet" -ForegroundColor Cyan
    Write-Host ""
    exit 1
}

# Compilar el sistema
Write-Host "🔧 === COMPILACIÓN ===" -ForegroundColor Cyan
Write-Host "   Compilando sistema de arbitraje militar..." -ForegroundColor Gray

try {
    cargo build --release --bin military_arbitrage_system
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Error en compilación" -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ Compilación exitosa" -ForegroundColor Green
} catch {
    Write-Host "❌ Error compilando: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

Write-Host ""

# Configurar logging
$env:RUST_LOG = "military_arbitrage_system=info,warn"

# Ejecutar el sistema
Write-Host "🚀 === EJECUTANDO SISTEMA MILITAR ===" -ForegroundColor Green
Write-Host "   Iniciando sistema de arbitraje con Helius Premium..." -ForegroundColor Gray
Write-Host ""

if ($hasHelius) {
    Write-Host "🔥 MODO HELIUS PREMIUM: Acceso completo a pools activos" -ForegroundColor Yellow
} else {
    Write-Host "⚠️  MODO ESTÁNDAR: Usando APIs públicas" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "🎯 === INICIANDO OPERACIONES MILITARES ===" -ForegroundColor Green
Write-Host "   Presione Ctrl+C para detener el sistema" -ForegroundColor Gray
Write-Host ""

# Ejecutar el sistema
try {
    cargo run --release --bin military_arbitrage_system
} catch {
    Write-Host ""
    Write-Host "❌ Error ejecutando sistema: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "✅ Sistema finalizado" -ForegroundColor Green
