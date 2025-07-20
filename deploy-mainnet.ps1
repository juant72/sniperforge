# 🚀 MAINNET DEPLOYMENT SCRIPT
Write-Host "╔═══════════════════════════════════════════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "║                    🚀 DEPLOYING TO MAINNET - LIVE TRADING                   ║" -ForegroundColor Green  
Write-Host "╚═══════════════════════════════════════════════════════════════════════════════╝" -ForegroundColor Green

# STEP 1: Configure Mainnet RPC
Write-Host "🌐 STEP 1: Configuring Mainnet RPC..." -ForegroundColor Yellow
$env:SOLANA_RPC_URL = "https://api.mainnet-beta.solana.com"
Write-Host "✅ RPC URL set to: $env:SOLANA_RPC_URL" -ForegroundColor Green

# STEP 2: Check Solana CLI Configuration
Write-Host "🔧 STEP 2: Checking Solana CLI..." -ForegroundColor Yellow
try {
    $config = solana config get
    Write-Host "✅ Solana CLI configured:" -ForegroundColor Green
    Write-Host $config -ForegroundColor White
} catch {
    Write-Host "❌ Solana CLI not found. Please install: https://docs.solana.com/cli/install-solana-cli-tools" -ForegroundColor Red
    exit 1
}

# STEP 3: Set to Mainnet
Write-Host "🎯 STEP 3: Setting Solana CLI to Mainnet..." -ForegroundColor Yellow
solana config set --url mainnet-beta
Write-Host "✅ Solana CLI set to Mainnet" -ForegroundColor Green

# STEP 4: Check wallet balance
Write-Host "💰 STEP 4: Checking wallet balance..." -ForegroundColor Yellow
try {
    $balance = solana balance
    Write-Host "💰 Current Mainnet Balance: $balance" -ForegroundColor Green
    
    # Parse balance to check if sufficient
    $balanceValue = [double]($balance -split ' ')[0]
    if ($balanceValue -lt 1.0) {
        Write-Host "⚠️  WARNING: Balance is low ($balance). Recommended minimum: 2 SOL for profitable trading" -ForegroundColor Yellow
        $continue = Read-Host "Continue anyway? (y/N)"
        if ($continue -ne 'y' -and $continue -ne 'Y') {
            Write-Host "❌ Deployment cancelled. Please fund your wallet and try again." -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host "✅ Sufficient balance for trading" -ForegroundColor Green
    }
} catch {
    Write-Host "❌ Could not check balance. Make sure you have a wallet configured." -ForegroundColor Red
    Write-Host "💡 Generate new wallet: solana-keygen new" -ForegroundColor Blue
    exit 1
}

# STEP 5: Build optimized release
Write-Host "🔨 STEP 5: Building optimized release version..." -ForegroundColor Yellow
cargo build --release --bin military_arbitrage_system
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed. Please check compilation errors above." -ForegroundColor Red
    exit 1
}
Write-Host "✅ Build successful" -ForegroundColor Green

# STEP 6: Safety reminder
Write-Host "🛡️  STEP 6: Safety Reminders..." -ForegroundColor Yellow
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue
Write-Host "⚠️  MAINNET TRADING SAFETY:" -ForegroundColor Red
Write-Host "   • This is LIVE trading with REAL money" -ForegroundColor Yellow
Write-Host "   • Start with small amounts (1-2 SOL)" -ForegroundColor Yellow  
Write-Host "   • Monitor performance closely" -ForegroundColor Yellow
Write-Host "   • Use Ctrl+C to stop at any time" -ForegroundColor Yellow
Write-Host "   • Maximum trade size: 2 SOL per operation" -ForegroundColor Yellow
Write-Host "   • Daily loss limit: 1 SOL" -ForegroundColor Yellow
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue

$confirm = Read-Host "🚨 Ready to start LIVE MAINNET trading? Type 'LIVE' to confirm"
if ($confirm -ne 'LIVE') {
    Write-Host "❌ Deployment cancelled for safety." -ForegroundColor Red
    exit 1
}

# STEP 7: Launch with real data
Write-Host "🎯 STEP 7: Launching Military Arbitrage System with REAL MAINNET DATA..." -ForegroundColor Red
Write-Host "🔥 STARTING LIVE TRADING..." -ForegroundColor Green
Write-Host ""

# Set environment variables for production
$env:RUST_LOG = "info"
$env:SOLANA_RPC_URL = "https://api.mainnet-beta.solana.com"

# Launch the system
cargo run --release --bin military_arbitrage_system
