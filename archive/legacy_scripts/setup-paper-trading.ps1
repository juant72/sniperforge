# Paper Trading Setup Script for Mainnet
# Ejecutar con: .\setup-paper-trading.ps1

Write-Host "🎯 SETTING UP PAPER TRADING WITH MAINNET DATA" -ForegroundColor Green
Write-Host "=============================================" -ForegroundColor Green

# Verificar que Rust esté instalado
Write-Host "`n🔍 Checking Rust installation..."
try {
    $rustVersion = cargo --version
    Write-Host "✅ $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Rust not found. Please install Rust first." -ForegroundColor Red
    exit 1
}

# Verificar configuración de variables de entorno
Write-Host "`n📋 Checking environment configuration..."
if (Test-Path ".env") {
    Write-Host "✅ .env file found" -ForegroundColor Green
} else {
    Write-Host "⚠️  Creating .env file from template..." -ForegroundColor Yellow
    if (Test-Path ".env.example") {
        Copy-Item ".env.example" ".env"
        Write-Host "✅ .env created from .env.example" -ForegroundColor Green
    } else {
        Write-Host "❌ No .env.example found. Creating basic .env..." -ForegroundColor Yellow
        @"
# Paper Trading Configuration
PAPER_TRADING_MODE=true
USE_MAINNET_DATA=true
INITIAL_SOL_BALANCE=10.0
INITIAL_USDC_BALANCE=1000.0

# Jupiter API (Mainnet)
JUPITER_API_URL=https://quote-api.jup.ag/v6

# Syndica WebSocket (Mainnet)
SYNDICA_TOKEN=your_syndica_token_here
SYNDICA_ENDPOINT=wss://solana-mainnet.api.syndica.io
"@ | Out-File -FilePath ".env" -Encoding UTF8
        Write-Host "✅ Basic .env created" -ForegroundColor Green
    }
}

# Compilar el proyecto
Write-Host "`n🏗️  Building project..."
cargo build --release
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Build successful" -ForegroundColor Green
} else {
    Write-Host "❌ Build failed" -ForegroundColor Red
    exit 1
}

# Ejecutar tests de paper trading
Write-Host "`n🧪 Running paper trading tests..."
Write-Host "================================`n" -ForegroundColor Cyan

Write-Host "1️⃣ Basic paper trading test:" -ForegroundColor Yellow
cargo run --release -- test paper-trading

Write-Host "`n2️⃣ Cache-free trading test:" -ForegroundColor Yellow
cargo run --release -- test cache-free-trading

Write-Host "`n3️⃣ Jupiter speed test:" -ForegroundColor Yellow
cargo run --release -- test jupiter-speed

Write-Host "`n4️⃣ Syndica WebSocket test:" -ForegroundColor Yellow
cargo run --release -- test syndica

# Resumen final
Write-Host "`n🎉 PAPER TRADING SETUP COMPLETE!" -ForegroundColor Green
Write-Host "=================================" -ForegroundColor Green
Write-Host "✅ Project compiled successfully" -ForegroundColor Green
Write-Host "✅ Environment configured" -ForegroundColor Green
Write-Host "✅ Paper trading tests executed" -ForegroundColor Green
Write-Host "`n📊 Next steps:" -ForegroundColor Cyan
Write-Host "  • Run: cargo run -- test paper-trading" -ForegroundColor White
Write-Host "  • Monitor: cargo run -- interactive" -ForegroundColor White
Write-Host "  • Configure: Edit .env file for your settings" -ForegroundColor White
Write-Host "`n⚠️  IMPORTANT: This is PAPER TRADING only!" -ForegroundColor Yellow
Write-Host "   No real money will be traded." -ForegroundColor Yellow
