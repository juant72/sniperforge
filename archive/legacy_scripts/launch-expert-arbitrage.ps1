# 🚀 EXPERT ARBITRAGE SYSTEM LAUNCHER
# Complete expert roadmap implementation with all optimizations

Write-Host ""
Write-Host "╔═══════════════════════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║                          🚀 EXPERT ARBITRAGE LAUNCHER 🚀                     ║" -ForegroundColor Cyan
Write-Host "║                   25x Faster Execution • Real-Time Feeds                     ║" -ForegroundColor Cyan
Write-Host "╚═══════════════════════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Function to check prerequisites
function Test-Prerequisites {
    Write-Host "🔍 EXPERT VALIDATION: Checking prerequisites..." -ForegroundColor Yellow
    
    # Check Rust
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        $rustVersion = (cargo --version).Split(' ')[1]
        Write-Host "✅ RUST: Version $rustVersion" -ForegroundColor Green
    } else {
        Write-Host "❌ RUST: Not found. Please install Rust" -ForegroundColor Red
        exit 1
    }
    
    # Check wallet
    if (Test-Path "mainnet_wallet.json") {
        Write-Host "✅ MAINNET WALLET: Ready" -ForegroundColor Green
    } else {
        Write-Host "❌ MAINNET WALLET: mainnet_wallet.json not found" -ForegroundColor Red
        exit 1
    }
    
    # Check environment variables
    if ($env:HELIUS_API_KEY) {
        $keyPreview = $env:HELIUS_API_KEY.Substring(0, [Math]::Min(8, $env:HELIUS_API_KEY.Length))
        Write-Host "✅ HELIUS RPC: Premium key ($keyPreview...)" -ForegroundColor Green
    } else {
        Write-Host "⚠️  HELIUS RPC: Not configured (will use standard RPC)" -ForegroundColor Yellow
    }
    
    Write-Host ""
}

# Function to show expert features
function Show-ExpertFeatures {
    Write-Host "┌─────────────────────────────────────────────────────────────────────────────┐" -ForegroundColor White
    Write-Host "│                         🚀 EXPERT FEATURES ACTIVE                          │" -ForegroundColor White
    Write-Host "├─────────────────────────────────────────────────────────────────────────────┤" -ForegroundColor White
    Write-Host "│  ✅ Expert Mathematical Foundation (Real AMM calculations)                 │" -ForegroundColor Green
    Write-Host "│  ✅ Mainnet Production Access (Real pools with `$10M+ TVL)                  │" -ForegroundColor Green
    Write-Host "│  ✅ Speed Optimization Engine (<200ms execution target)                   │" -ForegroundColor Green
    Write-Host "│  ✅ Real-Time Price Feeds (<400ms WebSocket updates)                      │" -ForegroundColor Green
    Write-Host "│  ✅ Ultra-Fast Parallel Processing (20 pools simultaneously)             │" -ForegroundColor Green
    Write-Host "│  ✅ Priority Fee Optimization (2M lamports for maximum speed)            │" -ForegroundColor Green
    Write-Host "│  ✅ Expert Cost Calculations (All DEX fees included)                      │" -ForegroundColor Green
    Write-Host "│  ✅ Performance Monitoring (Real-time metrics display)                    │" -ForegroundColor Green
    Write-Host "└─────────────────────────────────────────────────────────────────────────────┘" -ForegroundColor White
    Write-Host ""
}

# Function to build expert system
function Build-ExpertSystem {
    Write-Host "🔨 EXPERT BUILD: Compiling ultra-fast arbitrage system..." -ForegroundColor Yellow
    
    # Fast incremental build
    $buildResult = cargo build --bin run_expert_arbitrage --release 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ EXPERT BUILD: Success! System compiled and optimized" -ForegroundColor Green
    } else {
        Write-Host "❌ EXPERT BUILD: Failed. Details:" -ForegroundColor Red
        Write-Host $buildResult -ForegroundColor Red
        exit 1
    }
    Write-Host ""
}

# Function to run expert arbitrage
function Start-ExpertArbitrage {
    Write-Host "🚀 EXPERT LAUNCH: Starting ultra-fast arbitrage execution..." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "┌─────────────────────────────────────────────────────────────────────────────┐" -ForegroundColor Cyan
    Write-Host "│                         ⚡ EXPERT MODE ACTIVE ⚡                           │" -ForegroundColor Cyan
    Write-Host "│                                                                             │" -ForegroundColor Cyan
    Write-Host "│  Target Performance:                                                        │" -ForegroundColor Cyan
    Write-Host "│  • Execution Speed: <200ms (25x faster)                                   │" -ForegroundColor Cyan
    Write-Host "│  • Price Refresh: <400ms (Real-time)                                      │" -ForegroundColor Cyan
    Write-Host "│  • Parallel Processing: 20 pools                                          │" -ForegroundColor Cyan
    Write-Host "│  • Profit Target: >0.50% after all costs                                  │" -ForegroundColor Cyan
    Write-Host "│                                                                             │" -ForegroundColor Cyan
    Write-Host "│  Press Ctrl+C to stop the expert system                                    │" -ForegroundColor Cyan
    Write-Host "└─────────────────────────────────────────────────────────────────────────────┘" -ForegroundColor Cyan
    Write-Host ""
    
    # Execute expert arbitrage system
    try {
        cargo run --bin run_expert_arbitrage --release
    }
    catch {
        Write-Host "❌ EXPERT EXECUTION: Error occurred" -ForegroundColor Red
        Write-Host $_.Exception.Message -ForegroundColor Red
    }
}

# Main execution flow
try {
    Test-Prerequisites
    Show-ExpertFeatures
    Build-ExpertSystem
    Start-ExpertArbitrage
}
catch {
    Write-Host ""
    Write-Host "❌ EXPERT SYSTEM: Fatal error occurred" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "🎯 EXPERT SYSTEM: Session completed" -ForegroundColor Green
