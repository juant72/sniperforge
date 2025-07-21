# ===== MAINNET REAL EXECUTION DEPLOYMENT SCRIPT =====
# Script para desplegar y ejecutar arbitraje real en mainnet
# ENTERPRISE-GRADE DEPLOYMENT FOR PRODUCTION TRADING

Write-Host "🚀 MAINNET REAL EXECUTION DEPLOYMENT" -ForegroundColor Green
Write-Host "⚠️  WARNING: This will use REAL MONEY on Solana mainnet" -ForegroundColor Yellow
Write-Host ""

# Configuration
$WALLET_PATH = "mainnet-wallet.json"
$LOG_FILE = "mainnet_arbitrage_$(Get-Date -Format 'yyyyMMdd_HHmmss').log"
$BACKUP_DIR = "backup_$(Get-Date -Format 'yyyyMMdd')"

# Safety checks
Write-Host "🛡️  PERFORMING SAFETY CHECKS..." -ForegroundColor Cyan

# Check if wallet exists
if (-not (Test-Path $WALLET_PATH)) {
    Write-Host "❌ ERROR: Wallet file not found: $WALLET_PATH" -ForegroundColor Red
    Write-Host "📝 Please create your mainnet wallet first:" -ForegroundColor Yellow
    Write-Host "   solana-keygen new --outfile $WALLET_PATH" -ForegroundColor White
    exit 1
}

Write-Host "✅ Wallet file found: $WALLET_PATH" -ForegroundColor Green

# Check wallet balance (requires solana CLI)
Write-Host "💰 Checking wallet balance..." -ForegroundColor Cyan
try {
    $balance = solana balance --keypair $WALLET_PATH --url mainnet-beta
    Write-Host "💳 Current balance: $balance" -ForegroundColor Green
    
    # Extract numeric value (assuming format like "1.5 SOL")
    $balanceValue = [decimal]($balance -replace " SOL", "")
    if ($balanceValue -lt 0.5) {
        Write-Host "⚠️  WARNING: Low balance detected ($balanceValue SOL)" -ForegroundColor Yellow
        Write-Host "💡 Recommended minimum: 1.0 SOL for safe operation" -ForegroundColor Yellow
        $continue = Read-Host "Continue anyway? (y/N)"
        if ($continue -ne "y" -and $continue -ne "Y") {
            Write-Host "🛑 Deployment cancelled for safety" -ForegroundColor Red
            exit 1
        }
    }
} catch {
    Write-Host "⚠️  Could not check balance (solana CLI required)" -ForegroundColor Yellow
}

# Build the project
Write-Host ""
Write-Host "🔨 BUILDING PROJECT..." -ForegroundColor Cyan
Write-Host "📋 Building with optimization for mainnet..." -ForegroundColor White

try {
    cargo build --release --bin arbitrage_mainnet_real
    if ($LASTEXITCODE -ne 0) {
        throw "Build failed"
    }
    Write-Host "✅ Build completed successfully" -ForegroundColor Green
} catch {
    Write-Host "❌ Build failed. Please check for compilation errors." -ForegroundColor Red
    exit 1
}

# Create backup directory
Write-Host ""
Write-Host "💾 CREATING BACKUP..." -ForegroundColor Cyan
New-Item -ItemType Directory -Path $BACKUP_DIR -Force | Out-Null
Copy-Item $WALLET_PATH -Destination "$BACKUP_DIR/" -Force
Write-Host "✅ Wallet backed up to: $BACKUP_DIR" -ForegroundColor Green

# Final confirmation
Write-Host ""
Write-Host "🚨 FINAL SAFETY CONFIRMATION" -ForegroundColor Red
Write-Host "═══════════════════════════════════════════" -ForegroundColor Red
Write-Host "⚠️  You are about to execute REAL arbitrage trading on Solana mainnet" -ForegroundColor Yellow
Write-Host "💰 This will use REAL SOL from your wallet" -ForegroundColor Yellow
Write-Host "📊 Profits and losses will be REAL" -ForegroundColor Yellow
Write-Host "🛡️  All safety systems are active, but risks remain" -ForegroundColor Yellow
Write-Host "═══════════════════════════════════════════" -ForegroundColor Red
Write-Host ""

$confirmation = Read-Host "Type 'I UNDERSTAND THE RISKS' to proceed"
if ($confirmation -ne "I UNDERSTAND THE RISKS") {
    Write-Host "🛑 Deployment cancelled - Safety confirmation failed" -ForegroundColor Red
    exit 1
}

# Execute the arbitrage bot
Write-Host ""
Write-Host "🚀 LAUNCHING MAINNET ARBITRAGE BOT..." -ForegroundColor Green
Write-Host "📝 Logs will be saved to: $LOG_FILE" -ForegroundColor Cyan
Write-Host "🛑 Press Ctrl+C to stop execution at any time" -ForegroundColor Yellow
Write-Host ""

# Set environment for production
$env:RUST_LOG = "info"
$env:SOLANA_NETWORK = "mainnet-beta"

try {
    # Execute with logging
    .\target\release\arbitrage_mainnet_real.exe 2>&1 | Tee-Object -FilePath $LOG_FILE
} catch {
    Write-Host "❌ Execution error: $_" -ForegroundColor Red
} finally {
    Write-Host ""
    Write-Host "🏁 ARBITRAGE SESSION COMPLETED" -ForegroundColor Cyan
    Write-Host "📊 Check logs for execution details: $LOG_FILE" -ForegroundColor White
    Write-Host "💾 Wallet backup available at: $BACKUP_DIR" -ForegroundColor White
}

# Post-execution summary
Write-Host ""
Write-Host "📈 POST-EXECUTION SUMMARY" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════" -ForegroundColor Cyan

try {
    $newBalance = solana balance --keypair $WALLET_PATH --url mainnet-beta
    Write-Host "💳 Final balance: $newBalance" -ForegroundColor White
    
    # Calculate profit/loss if we had the initial balance
    if ($balanceValue) {
        $finalValue = [decimal]($newBalance -replace " SOL", "")
        $difference = $finalValue - $balanceValue
        if ($difference -gt 0) {
            Write-Host "📈 Profit: +$difference SOL" -ForegroundColor Green
        } elseif ($difference -lt 0) {
            Write-Host "📉 Loss: $difference SOL" -ForegroundColor Red
        } else {
            Write-Host "📊 No change in balance" -ForegroundColor Yellow
        }
    }
} catch {
    Write-Host "⚠️  Could not check final balance" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "✅ Deployment script completed successfully" -ForegroundColor Green
Write-Host "🎯 Thank you for using Sniperforge Enterprise Arbitrage System" -ForegroundColor Cyan
