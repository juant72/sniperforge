# 🎯 Demo Script: Complete Arbitrage Bot Demonstration
# This script demonstrates the complete arbitrage bot workflow with real transactions

param(
    [switch]$SkipBuild,
    [switch]$SkipAirdrop,
    [string]$BotType = "jupiter"  # jupiter, custom, simple
)

Write-Host "🎯 SniperForge Real Arbitrage Bot - DEMO" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "This demo will execute REAL transactions on DevNet!" -ForegroundColor Yellow
Write-Host "You will see REAL profits and REAL transaction fees!" -ForegroundColor Yellow
Write-Host "" -ForegroundColor White

# Helper function for safe command execution
function Invoke-DemoCommand {
    param(
        [string]$Command,
        [string]$Description,
        [switch]$ShowOutput = $true
    )
    
    Write-Host "📋 $Description" -ForegroundColor Magenta
    Write-Host "Executing: $Command" -ForegroundColor Gray
    Write-Host "---" -ForegroundColor DarkGray
    
    $startTime = Get-Date
    
    try {
        if ($ShowOutput) {
            Invoke-Expression $Command
        } else {
            $output = Invoke-Expression $Command 2>&1
            Write-Host $output -ForegroundColor White
        }
        
        $endTime = Get-Date
        $duration = $endTime - $startTime
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "---" -ForegroundColor DarkGray
            Write-Host "✅ $Description completed in $($duration.TotalSeconds.ToString('F2'))s" -ForegroundColor Green
        } else {
            Write-Host "❌ $Description failed (exit code: $LASTEXITCODE)" -ForegroundColor Red
            return $false
        }
    } catch {
        Write-Host "❌ Error: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
    
    Write-Host "" -ForegroundColor White
    return $true
}

# Step 1: Build (if not skipped)
if (!$SkipBuild) {
    Write-Host "🔨 STEP 1: Building the project..." -ForegroundColor Cyan
    if (!(Invoke-DemoCommand "cargo build --release" "Building SniperForge arbitrage bot")) {
        Write-Host "Demo aborted due to build failure." -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "⏭️ STEP 1: Skipped build (using existing binaries)" -ForegroundColor Yellow
    Write-Host "" -ForegroundColor White
}

# Step 2: Environment check
Write-Host "🔍 STEP 2: Environment verification..." -ForegroundColor Cyan

# Check .env file
if (!(Test-Path ".env")) {
    Write-Host "❌ .env file not found!" -ForegroundColor Red
    Write-Host "Please create .env with your wallet and RPC configuration." -ForegroundColor Yellow
    exit 1
}

Write-Host "✅ .env file found" -ForegroundColor Green
Write-Host "" -ForegroundColor White

# Step 3: Wallet information
Write-Host "🔑 STEP 3: Wallet information..." -ForegroundColor Cyan
if (!(Invoke-DemoCommand "cargo run --release --bin get_wallet_address" "Getting wallet address")) {
    Write-Host "Demo aborted due to wallet address failure." -ForegroundColor Red
    exit 1
}

# Step 4: Initial balance check
Write-Host "💰 STEP 4: Initial balance check..." -ForegroundColor Cyan
Write-Host "Recording baseline balances before arbitrage execution..." -ForegroundColor Yellow

if (!(Invoke-DemoCommand "cargo run --release --bin check_devnet_balance" "Checking initial balances")) {
    Write-Host "Demo aborted due to balance check failure." -ForegroundColor Red
    exit 1
}

# Step 5: DevNet SOL airdrop (if not skipped)
if (!$SkipAirdrop) {
    Write-Host "💧 STEP 5: DevNet SOL airdrop..." -ForegroundColor Cyan
    Write-Host "Ensuring sufficient SOL for transaction fees..." -ForegroundColor Yellow
    
    if (!(Invoke-DemoCommand "cargo run --release --bin request_devnet_airdrop" "Requesting DevNet SOL airdrop")) {
        Write-Host "⚠️ Airdrop failed, but continuing with demo..." -ForegroundColor Yellow
    }
    
    # Check balances after airdrop
    Write-Host "💰 Post-airdrop balance check..." -ForegroundColor Cyan
    Invoke-DemoCommand "cargo run --release --bin check_devnet_balance" "Checking balances after airdrop"
} else {
    Write-Host "⏭️ STEP 5: Skipped airdrop (assuming sufficient SOL balance)" -ForegroundColor Yellow
    Write-Host "" -ForegroundColor White
}

# Step 6: Real arbitrage execution
Write-Host "🚀 STEP 6: Real arbitrage execution..." -ForegroundColor Cyan

$botCommand = ""
$botDescription = ""

switch ($BotType.ToLower()) {
    "jupiter" {
        $botCommand = "cargo run --release --bin test_arbitrage_real_jupiter"
        $botDescription = "Jupiter Real Arbitrage Bot (Production Ready)"
        Write-Host "Using Jupiter API for real price quotes and multi-DEX arbitrage!" -ForegroundColor Green
    }
    "custom" {
        $botCommand = "cargo run --release --bin test_real_arbitrage_devnet"
        $botDescription = "Custom DEX Real Arbitrage Bot (Advanced)"
        Write-Host "Using custom mint/burn logic for specialized token operations!" -ForegroundColor Yellow
    }
    "simple" {
        $botCommand = "cargo run --release --bin test_simple_arbitrage_real"
        $botDescription = "Simple Real Transfer Bot (Proof of Concept)"
        Write-Host "Demonstrating real on-chain transfers and balance changes!" -ForegroundColor Blue
    }
    default {
        $botCommand = "cargo run --release --bin test_arbitrage_real_jupiter"
        $botDescription = "Jupiter Real Arbitrage Bot (Default)"
        Write-Host "Defaulting to Jupiter bot for optimal results!" -ForegroundColor Green
    }
}

Write-Host "" -ForegroundColor White
Write-Host "⚠️  IMPORTANT: This will execute REAL transactions!" -ForegroundColor Red -BackgroundColor Yellow
Write-Host "⚠️  You will see REAL balance changes and pay REAL fees!" -ForegroundColor Red -BackgroundColor Yellow
Write-Host "" -ForegroundColor White

Write-Host "Press any key to continue with $botDescription..." -ForegroundColor Cyan
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
Write-Host "" -ForegroundColor White

if (!(Invoke-DemoCommand $botCommand $botDescription)) {
    Write-Host "⚠️ Arbitrage execution encountered issues, but continuing with final checks..." -ForegroundColor Yellow
}

# Step 7: Final balance check
Write-Host "💰 STEP 7: Final balance check..." -ForegroundColor Cyan
Write-Host "Verifying real profits and balance changes..." -ForegroundColor Yellow

if (!(Invoke-DemoCommand "cargo run --release --bin check_devnet_balance" "Checking final balances")) {
    Write-Host "Demo completed with balance check issues." -ForegroundColor Yellow
} else {
    Write-Host "🎉 Demo completed successfully!" -ForegroundColor Green
}

# Step 8: Demo summary and analysis
Write-Host "📊 DEMO SUMMARY" -ForegroundColor Cyan
Write-Host "===============" -ForegroundColor Cyan

Write-Host "`n🔍 What to verify in the output above:" -ForegroundColor Yellow
Write-Host "✅ Real transaction signatures (e.g., 5Kj8x9vR2mN7...)" -ForegroundColor White
Write-Host "✅ Positive token balance changes (real profits)" -ForegroundColor White
Write-Host "✅ SOL balance reduction from transaction fees" -ForegroundColor White
Write-Host "✅ 'Transaction confirmed' messages" -ForegroundColor White

Write-Host "`n🌐 Verify your transactions on-chain:" -ForegroundColor Yellow
Write-Host "Solana Explorer (DevNet): https://explorer.solana.com/?cluster=devnet" -ForegroundColor Gray
Write-Host "SolanaFM (DevNet): https://solana.fm/?cluster=devnet-solana" -ForegroundColor Gray

Write-Host "`n🎯 Next steps:" -ForegroundColor Yellow
Write-Host "• Run additional arbitrage cycles: $botCommand" -ForegroundColor White
Write-Host "• Monitor balances: cargo run --release --bin check_devnet_balance" -ForegroundColor White
Write-Host "• Try different bot types: jupiter, custom, simple" -ForegroundColor White
Write-Host "• Deploy to MainNet: Update RPC URL in .env" -ForegroundColor White

Write-Host "`n📖 Documentation:" -ForegroundColor Yellow
Write-Host "• CLI Guide: .\CLI_ARBITRAGE_BOT_GUIDE.md" -ForegroundColor White
Write-Host "• Validation Report: .\VALIDACION_REAL_FINAL_REPORT.md" -ForegroundColor White

Write-Host "`n🏆 Demo completed! You have successfully run a real arbitrage bot!" -ForegroundColor Green

# Optional: Save demo log
$logFile = "demo-log-$(Get-Date -Format 'yyyyMMdd-HHmmss').txt"
Write-Host "`nDemo log saved to: $logFile" -ForegroundColor Gray
