#!/usr/bin/env pwsh

Write-Host "🔥 === PROFESSIONAL ARBITRAGE SETUP ===" -ForegroundColor Cyan
Write-Host "   🏆 Setting up premium RPC endpoints" -ForegroundColor Green
Write-Host "   💰 Professional rate limiting" -ForegroundColor Yellow
Write-Host "   🎯 Anti-429 error configuration" -ForegroundColor Magenta

# Check if premium RPC keys are available
$heliusKey = $env:HELIUS_API_KEY
$alchemyKey = $env:ALCHEMY_API_KEY

if (-not $heliusKey -and -not $alchemyKey) {
    Write-Host ""
    Write-Host "⚠️  === PREMIUM RPC SETUP REQUIRED ===" -ForegroundColor Yellow
    Write-Host "   Para evitar error 429, necesitas RPC premium:" -ForegroundColor White
    Write-Host ""
    Write-Host "🏆 HELIUS (Recomendado):" -ForegroundColor Cyan
    Write-Host "   1. Crear cuenta en https://www.helius.dev" -ForegroundColor White
    Write-Host "   2. Obtener API key gratis (100K requests/day)" -ForegroundColor White
    Write-Host "   3. Configurar: " -NoNewline -ForegroundColor White
    Write-Host "`$env:HELIUS_API_KEY = 'tu_key_aqui'" -ForegroundColor Green
    Write-Host ""
    Write-Host "🔮 ALCHEMY (Alternativa):" -ForegroundColor Cyan
    Write-Host "   1. Crear cuenta en https://www.alchemy.com" -ForegroundColor White
    Write-Host "   2. Obtener API key gratis" -ForegroundColor White
    Write-Host "   3. Configurar: " -NoNewline -ForegroundColor White
    Write-Host "`$env:ALCHEMY_API_KEY = 'tu_key_aqui'" -ForegroundColor Green
    Write-Host ""
    Write-Host "💎 QUICKNODE (Premium):" -ForegroundColor Cyan
    Write-Host "   1. Crear cuenta en https://www.quicknode.com" -ForegroundColor White
    Write-Host "   2. Plan desde $9/mes para uso profesional" -ForegroundColor White
    Write-Host ""
    Write-Host "🚀 Usando RPC público (limitado) por ahora..." -ForegroundColor Yellow
    $env:SOLANA_RPC_URL = "https://api.mainnet-beta.solana.com"
} elseif ($heliusKey) {
    Write-Host "✅ Helius RPC configurado!" -ForegroundColor Green
    $env:SOLANA_RPC_URL = "https://rpc.helius.xyz/?api-key=$heliusKey"
    Write-Host "   🎯 URL: $env:SOLANA_RPC_URL" -ForegroundColor Cyan
} elseif ($alchemyKey) {
    Write-Host "✅ Alchemy RPC configurado!" -ForegroundColor Green
    $env:SOLANA_RPC_URL = "https://solana-mainnet.g.alchemy.com/v2/$alchemyKey"
    Write-Host "   🎯 URL: $env:SOLANA_RPC_URL" -ForegroundColor Cyan
}

Write-Host ""
Write-Host "🔧 PROFESSIONAL SETTINGS APPLIED:" -ForegroundColor Magenta
Write-Host "   ⏱️  30-second intervals (vs 10s antes)" -ForegroundColor White
Write-Host "   🔢 3 requests paralelos (vs 8 antes)" -ForegroundColor White
Write-Host "   🎯 Exponential backoff en 429 errors" -ForegroundColor White
Write-Host "   🔄 Request jitter para evitar thundering herd" -ForegroundColor White
Write-Host "   💎 Conservative slippage para menos 429s" -ForegroundColor White

# Check wallet
if (Test-Path "mainnet_wallet.json") {
    Write-Host "✅ Wallet found: mainnet_wallet.json" -ForegroundColor Green
} else {
    Write-Host "❌ Wallet not found: mainnet_wallet.json" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "🎯 READY FOR PROFESSIONAL ARBITRAGE!" -ForegroundColor Green
Write-Host "   Press Enter to start or Ctrl+C to cancel..." -ForegroundColor Cyan
Read-Host

Write-Host "🚀 Starting professional arbitrage hunter..." -ForegroundColor Green

# Build and run with professional settings
cargo build --release --bin mega_token_hunter
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Build successful, starting professional hunter..." -ForegroundColor Green
    cargo run --release --bin mega_token_hunter
} else {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}
