# 🚀 Quick Start Script for Real Arbitrage Bot
# This script automates the complete setup and execution of the real arbitrage bot

Write-Host "🚀 SniperForge Real Arbitrage Bot - Quick Start" -ForegroundColor Cyan
Write-Host "=================================================" -ForegroundColor Cyan

# Function to run commands with error handling
function Invoke-SafeCommand {
    param(
        [string]$Command,
        [string]$Description
    )
    
    Write-Host "`n📋 $Description..." -ForegroundColor Yellow
    Write-Host "Executing: $Command" -ForegroundColor Gray
    
    try {
        Invoke-Expression $Command
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ $Description completed successfully!" -ForegroundColor Green
        } else {
            Write-Host "❌ $Description failed with exit code $LASTEXITCODE" -ForegroundColor Red
            return $false
        }
    } catch {
        Write-Host "❌ $Description failed with error: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
    return $true
}

# Step 1: Build the project
Write-Host "`n🔨 Building the project..." -ForegroundColor Magenta
if (!(Invoke-SafeCommand "cargo build --release" "Project build")) {
    Write-Host "❌ Build failed. Please check your Rust installation and try again." -ForegroundColor Red
    exit 1
}

# Step 2: Check environment setup
Write-Host "`n🔍 Checking environment setup..." -ForegroundColor Magenta

# Check if .env file exists
if (!(Test-Path ".env")) {
    Write-Host "❌ .env file not found! Please create .env file with your wallet and RPC configuration." -ForegroundColor Red
    Write-Host "Required contents:" -ForegroundColor Yellow
    Write-Host "PRIVATE_KEY=your_base58_encoded_private_key" -ForegroundColor Gray
    Write-Host "SOLANA_RPC_URL=https://solana-devnet.g.alchemy.com/v2/your_api_key" -ForegroundColor Gray
    Write-Host "ALCHEMY_API_KEY=your_alchemy_api_key" -ForegroundColor Gray
    exit 1
}

Write-Host "✅ .env file found!" -ForegroundColor Green

# Step 3: Get wallet address
Write-Host "`n🔑 Getting wallet address..." -ForegroundColor Magenta
Invoke-SafeCommand "cargo run --release --bin get_wallet_address" "Wallet address retrieval"

# Step 4: Check initial balances
Write-Host "`n💰 Checking initial balances..." -ForegroundColor Magenta
Invoke-SafeCommand "cargo run --release --bin check_devnet_balance" "Initial balance check"

# Step 5: Offer to request airdrop if needed
Write-Host "`n💧 Do you need DevNet SOL airdrop? (y/N): " -ForegroundColor Cyan -NoNewline
$airdropResponse = Read-Host
if ($airdropResponse -eq "y" -or $airdropResponse -eq "Y") {
    Write-Host "Requesting DevNet SOL airdrop..." -ForegroundColor Magenta
    Invoke-SafeCommand "cargo run --release --bin request_devnet_airdrop" "DevNet SOL airdrop"
    
    # Check balances again after airdrop
    Write-Host "`n💰 Checking balances after airdrop..." -ForegroundColor Magenta
    Invoke-SafeCommand "cargo run --release --bin check_devnet_balance" "Post-airdrop balance check"
}

# Step 6: Choose arbitrage bot
Write-Host "`n🤖 Choose your arbitrage bot:" -ForegroundColor Cyan
Write-Host "1. Jupiter Real Arbitrage Bot (RECOMMENDED - Production ready)" -ForegroundColor Green
Write-Host "2. Custom DEX Real Arbitrage Bot (Advanced)" -ForegroundColor Yellow
Write-Host "3. Simple Real Transfer Bot (Testing)" -ForegroundColor Blue
Write-Host "4. Skip arbitrage execution" -ForegroundColor Gray

Write-Host "`nEnter your choice (1-4): " -ForegroundColor Cyan -NoNewline
$botChoice = Read-Host

switch ($botChoice) {
    "1" {
        Write-Host "`n🚀 Starting Jupiter Real Arbitrage Bot..." -ForegroundColor Magenta
        Write-Host "This bot will execute REAL transactions and generate REAL profits!" -ForegroundColor Yellow
        Invoke-SafeCommand "cargo run --release --bin test_arbitrage_real_jupiter" "Jupiter Real Arbitrage Bot execution"
    }
    "2" {
        Write-Host "`n🚀 Starting Custom DEX Real Arbitrage Bot..." -ForegroundColor Magenta
        Write-Host "This bot uses custom mint/burn logic for specialized tokens!" -ForegroundColor Yellow
        Invoke-SafeCommand "cargo run --release --bin test_real_arbitrage_devnet" "Custom DEX Real Arbitrage Bot execution"
    }
    "3" {
        Write-Host "`n🚀 Starting Simple Real Transfer Bot..." -ForegroundColor Magenta
        Write-Host "This bot demonstrates real on-chain transfers!" -ForegroundColor Yellow
        Invoke-SafeCommand "cargo run --release --bin test_simple_arbitrage_real" "Simple Real Transfer Bot execution"
    }
    "4" {
        Write-Host "`n⏭️ Skipping arbitrage execution..." -ForegroundColor Gray
    }
    default {
        Write-Host "`n❌ Invalid choice. Defaulting to Jupiter Real Arbitrage Bot..." -ForegroundColor Red
        Invoke-SafeCommand "cargo run --release --bin test_arbitrage_real_jupiter" "Jupiter Real Arbitrage Bot execution"
    }
}

# Step 7: Check final balances if arbitrage was executed
if ($botChoice -ne "4") {
    Write-Host "`n💰 Checking final balances to verify profits..." -ForegroundColor Magenta
    Invoke-SafeCommand "cargo run --release --bin check_devnet_balance" "Final balance check"
}

# Step 8: Summary and next steps
Write-Host "`n🎉 Quick Start Completed!" -ForegroundColor Green
Write-Host "=================================================" -ForegroundColor Cyan

if ($botChoice -ne "4") {
    Write-Host "`n📊 What to verify:" -ForegroundColor Cyan
    Write-Host "✅ Check for real transaction signatures in the output above" -ForegroundColor White
    Write-Host "✅ Look for positive token balance changes (real profits)" -ForegroundColor White
    Write-Host "✅ Verify SOL balance reduction from real transaction fees" -ForegroundColor White
    Write-Host "✅ Confirm transactions on Solana Explorer (DevNet)" -ForegroundColor White
}

Write-Host "`n🔗 Useful links:" -ForegroundColor Cyan
Write-Host "📖 Full CLI Guide: .\CLI_ARBITRAGE_BOT_GUIDE.md" -ForegroundColor White
Write-Host "📊 Validation Report: .\VALIDACION_REAL_FINAL_REPORT.md" -ForegroundColor White
Write-Host "🌐 Solana Explorer (DevNet): https://explorer.solana.com/?cluster=devnet" -ForegroundColor White

Write-Host "`n🚀 To run individual bots manually:" -ForegroundColor Cyan
Write-Host "cargo run --release --bin test_arbitrage_real_jupiter" -ForegroundColor Gray
Write-Host "cargo run --release --bin test_real_arbitrage_devnet" -ForegroundColor Gray
Write-Host "cargo run --release --bin test_simple_arbitrage_real" -ForegroundColor Gray

Write-Host "`n🔍 To check balances anytime:" -ForegroundColor Cyan
Write-Host "cargo run --release --bin check_devnet_balance" -ForegroundColor Gray

Write-Host "`n💡 For MainNet deployment:" -ForegroundColor Yellow
Write-Host "Update SOLANA_RPC_URL in .env to MainNet endpoint and ensure you have real SOL!" -ForegroundColor Gray

Write-Host "`n🎯 Happy Trading! 🚀" -ForegroundColor Green
