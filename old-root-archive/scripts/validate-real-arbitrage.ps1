#!/usr/bin/env pwsh

Write-Host "🛡️ === REAL ARBITRAGE VALIDATION ===" -ForegroundColor Cyan
Write-Host ""

$allGood = $true

# 1. Check wallet
Write-Host "🔍 Checking wallet configuration..." -ForegroundColor Yellow
$walletPaths = @(
    $env:SOLANA_WALLET_PATH,
    "mainnet_wallet.json", 
    "wallet.json"
)

$foundWallet = $false
$walletPath = ""
foreach ($path in $walletPaths) {
    if ($path -and (Test-Path $path)) {
        $walletPath = $path
        $foundWallet = $true
        Write-Host "   ✅ Wallet found: $path" -ForegroundColor Green
        break
    }
}

if (-not $foundWallet) {
    Write-Host "   ❌ No wallet found!" -ForegroundColor Red
    $allGood = $false
} else {
    # Check wallet balance
    Write-Host "🔍 Checking wallet balance..." -ForegroundColor Yellow
    try {
        $balanceOutput = solana balance --url mainnet-beta 2>&1
        if ($LASTEXITCODE -eq 0) {
            $balance = [double]($balanceOutput -replace " SOL", "")
            Write-Host "   ✅ Current balance: $balance SOL" -ForegroundColor Green
            
            if ($balance -lt 0.01) {
                Write-Host "   ⚠️ Low balance - recommend at least 0.01 SOL" -ForegroundColor Yellow
            }
            if ($balance -lt 0.003) {
                Write-Host "   ❌ Balance too low for safe execution!" -ForegroundColor Red
                $allGood = $false
            }
        } else {
            Write-Host "   ⚠️ Could not check balance automatically" -ForegroundColor Yellow
        }
    } catch {
        Write-Host "   ⚠️ Could not check balance: $_" -ForegroundColor Yellow
    }
}

# 2. Check RPC configuration
Write-Host "🔍 Checking RPC configuration..." -ForegroundColor Yellow
$rpcUrl = $env:SOLANA_RPC_URL
if (-not $rpcUrl) {
    Write-Host "   ⚠️ Using default RPC - consider premium RPC for better performance" -ForegroundColor Yellow
    $rpcUrl = "https://api.mainnet-beta.solana.com"
} else {
    Write-Host "   ✅ Custom RPC configured: $rpcUrl" -ForegroundColor Green
}

# 3. Check network connectivity
Write-Host "🔍 Testing network connectivity..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "https://quote-api.jup.ag/v6/tokens" -TimeoutSec 5 -UseBasicParsing
    if ($response.StatusCode -eq 200) {
        Write-Host "   ✅ Jupiter API accessible" -ForegroundColor Green
    } else {
        Write-Host "   ❌ Jupiter API not accessible" -ForegroundColor Red
        $allGood = $false
    }
} catch {
    Write-Host "   ❌ Network connectivity issue: $_" -ForegroundColor Red
    $allGood = $false
}

# 4. Check if build works
Write-Host "🔍 Testing build..." -ForegroundColor Yellow
cargo check --bin mega_token_hunter --quiet
if ($LASTEXITCODE -eq 0) {
    Write-Host "   ✅ Code compiles successfully" -ForegroundColor Green
} else {
    Write-Host "   ❌ Build errors detected!" -ForegroundColor Red
    $allGood = $false
}

Write-Host ""
Write-Host "=== VALIDATION SUMMARY ===" -ForegroundColor Cyan
if ($allGood) {
    Write-Host "✅ ALL CHECKS PASSED - Ready for real arbitrage execution!" -ForegroundColor Green
    Write-Host ""
    Write-Host "🚀 To start real trading:" -ForegroundColor Yellow
    Write-Host "   .\run-real-arbitrage.ps1" -ForegroundColor White
} else {
    Write-Host "❌ VALIDATION FAILED - Please fix issues before proceeding!" -ForegroundColor Red
    Write-Host ""
    Write-Host "🛠️ Fix the issues above and run validation again:" -ForegroundColor Yellow
    Write-Host "   .\validate-real-arbitrage.ps1" -ForegroundColor White
}

Write-Host ""
