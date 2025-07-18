#!/usr/bin/env pwsh
# ===== CONFIGURACIÓN DE HELIUS PREMIUM API =====
# Script para configurar la API key de Helius Premium

param(
    [string]$ApiKey = "",
    [switch]$Test = $false
)

Write-Host "🔥 === HELIUS PREMIUM API SETUP ===" -ForegroundColor Yellow
Write-Host "   Configurando acceso premium a Solana mainnet" -ForegroundColor Gray
Write-Host ""

# Verificar si ya hay una API key configurada
$currentKey = [Environment]::GetEnvironmentVariable("HELIUS_API_KEY", "User")
if ($currentKey) {
    Write-Host "✅ API Key actual: $($currentKey.Substring(0, [Math]::Min(8, $currentKey.Length)))..." -ForegroundColor Green
    Write-Host ""
}

# Si no se proporcionó una API key, pedirla
if (-not $ApiKey) {
    Write-Host "🔐 Ingrese su Helius API Key:" -ForegroundColor Cyan
    Write-Host "   (Se puede obtener en: https://helius.xyz)" -ForegroundColor Gray
    $ApiKey = Read-Host "   API Key"
    
    if (-not $ApiKey) {
        Write-Host "❌ No se proporcionó API Key. Saliendo..." -ForegroundColor Red
        exit 1
    }
}

# Configurar la variable de entorno
Write-Host "🔧 Configurando HELIUS_API_KEY..." -ForegroundColor Yellow
[Environment]::SetEnvironmentVariable("HELIUS_API_KEY", $ApiKey, "User")

# Configurar también para la sesión actual
$env:HELIUS_API_KEY = $ApiKey

Write-Host "✅ API Key configurada exitosamente!" -ForegroundColor Green
Write-Host ""

# Configurar el RPC URL para usar Helius
$heliusRpcUrl = "https://mainnet.helius-rpc.com/?api-key=$ApiKey"
[Environment]::SetEnvironmentVariable("SOLANA_RPC_URL", $heliusRpcUrl, "User")
$env:SOLANA_RPC_URL = $heliusRpcUrl

Write-Host "🌐 RPC URL configurado: https://mainnet.helius-rpc.com/?api-key=$($ApiKey.Substring(0, 8))..." -ForegroundColor Green
Write-Host ""

# Test de la conexión si se solicita
if ($Test) {
    Write-Host "🧪 Probando conexión con Helius..." -ForegroundColor Yellow
    
    try {
        $testBody = @{
            jsonrpc = "2.0"
            id = 1
            method = "getVersion"
        } | ConvertTo-Json
        
        $response = Invoke-RestMethod -Uri $heliusRpcUrl -Method POST -Body $testBody -ContentType "application/json"
        
        if ($response.result) {
            Write-Host "✅ Conexión exitosa con Helius!" -ForegroundColor Green
            Write-Host "   Versión Solana: $($response.result.'solana-core')" -ForegroundColor Gray
            Write-Host "   Feature Set: $($response.result.'feature-set')" -ForegroundColor Gray
        } else {
            Write-Host "❌ Error en la respuesta de Helius" -ForegroundColor Red
        }
    } catch {
        Write-Host "❌ Error conectando con Helius: $($_.Exception.Message)" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "🎯 === CONFIGURACIÓN COMPLETA ===" -ForegroundColor Green
Write-Host "   Helius Premium API configurada y lista para usar" -ForegroundColor Gray
Write-Host ""
Write-Host "📋 Próximos pasos:" -ForegroundColor Cyan
Write-Host "   1. Reinicia tu terminal para aplicar las variables de entorno" -ForegroundColor Gray
Write-Host "   2. Ejecuta el sistema de arbitraje: cargo run --bin military_arbitrage_system" -ForegroundColor Gray
Write-Host "   3. El sistema usará automáticamente Helius Premium para descubrir pools" -ForegroundColor Gray
Write-Host ""
Write-Host "💡 Para verificar la configuración:" -ForegroundColor Cyan
Write-Host "   .\setup-helius-premium.ps1 -Test" -ForegroundColor Gray
Write-Host ""
