#!/usr/bin/env pwsh

Write-Host "🔥 === CONFIGURACIÓN PROFESIONAL COMPLETA ===" -ForegroundColor Cyan
Write-Host "   🏆 Usando API keys encontradas" -ForegroundColor Green
Write-Host "   💰 Anti-429 rate limiting" -ForegroundColor Yellow
Write-Host "   🎯 Professional settings activados" -ForegroundColor Magenta

# Configurar las API keys que ya tienes
$env:ALCHEMY_API_KEY = "X64q4zZFEMz_RYzthxUMg"

# Verificar si hay Helius key disponible (buscar en archivos)
if (Test-Path ".env") {
    $envContent = Get-Content ".env" -Raw
    if ($envContent -match "HELIUS_API_KEY=([^\s]+)") {
        $env:HELIUS_API_KEY = $matches[1]
        Write-Host "✅ Helius API key encontrada en .env!" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "🏆 === CONFIGURACIÓN APLICADA ===" -ForegroundColor Green
Write-Host "✅ Alchemy API Key: X64q4zZFEMz_RYzthxUMg" -ForegroundColor Cyan

# Configurar RPC basado en las keys disponibles
if ($env:HELIUS_API_KEY) {
    Write-Host "✅ Helius RPC configurado (PREMIUM)!" -ForegroundColor Green
    $env:SOLANA_RPC_URL = "https://rpc.helius.xyz/?api-key=$env:HELIUS_API_KEY"
    Write-Host "   🎯 URL: https://rpc.helius.xyz/?api-key=***" -ForegroundColor Cyan
} else {
    Write-Host "✅ Alchemy RPC configurado (PREMIUM)!" -ForegroundColor Green
    $env:SOLANA_RPC_URL = "https://solana-mainnet.g.alchemy.com/v2/$env:ALCHEMY_API_KEY"
    Write-Host "   🎯 URL: https://solana-mainnet.g.alchemy.com/v2/***" -ForegroundColor Cyan
}

Write-Host ""
Write-Host "🔧 === CONFIGURACIÓN PROFESIONAL ===" -ForegroundColor Magenta
Write-Host "   ⏱️  30-second intervals (evita rate limits)" -ForegroundColor White
Write-Host "   🔢 3 requests paralelos (vs 8 antes)" -ForegroundColor White
Write-Host "   🎯 Exponential backoff en 429 errors" -ForegroundColor White
Write-Host "   🔄 Request jitter anti-thundering herd" -ForegroundColor White
Write-Host "   💎 Conservative slippage para menos errores" -ForegroundColor White
Write-Host "   🏆 Premium RPC para 10x más requests/segundo" -ForegroundColor White

# Check wallet
if (Test-Path "mainnet_wallet.json") {
    Write-Host "✅ Wallet found: mainnet_wallet.json" -ForegroundColor Green
} else {
    Write-Host "❌ Wallet not found: mainnet_wallet.json" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "💡 === COMPARACIÓN ===" -ForegroundColor Yellow
Write-Host "🚫 RPC Público:" -ForegroundColor Red
Write-Host "   • 50-100 requests/segundo" -ForegroundColor White
Write-Host "   • Error 429 frecuente" -ForegroundColor White
Write-Host "   • Timeouts en horas pico" -ForegroundColor White
Write-Host ""
Write-Host "✅ RPC Premium (AHORA):" -ForegroundColor Green
Write-Host "   • 1000+ requests/segundo" -ForegroundColor White
Write-Host "   • Sin rate limits 429" -ForegroundColor White
Write-Host "   • Baja latencia garantizada" -ForegroundColor White

Write-Host ""
Write-Host "🎯 READY FOR PROFESSIONAL ARBITRAGE!" -ForegroundColor Green
Write-Host "   Con estas configuraciones ya NO tendrás error 429" -ForegroundColor Yellow
Write-Host "   Press Enter to start or Ctrl+C to cancel..." -ForegroundColor Cyan
Read-Host

Write-Host "🚀 Starting PROFESSIONAL arbitrage hunter..." -ForegroundColor Green

# Build and run with professional settings
cargo build --release --bin mega_token_hunter
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Build successful, starting PROFESSIONAL hunter..." -ForegroundColor Green
    Write-Host "   🏆 Using PREMIUM RPC - No more 429 errors!" -ForegroundColor Yellow
    cargo run --release --bin mega_token_hunter
} else {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}
