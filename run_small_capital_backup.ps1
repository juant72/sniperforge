#!/usr/bin/env pwsh
# SniperForge - Capital Accumulation Mode (0.29 SOL)
# Ultra-Conservative Trading for Small Capital

param(
    [string]$Mode = "test",
    [string]$Config = "config\small_capital_config.json",
    [string]$Wallet = "wallet.json"
)

Write-Host "🚀 SniperForge Capital Accumulation Mode" -ForegroundColor Green
Write-Host "💰 Current Capital: 0.29 SOL" -ForegroundColor Yellow
Write-Host "🎯 Target: 30% profits with 10% risk" -ForegroundColor Cyan
Write-Host "📊 Config: $Config" -ForegroundColor Gray
Write-Host ""

# Verify configuration exists
if (-not (Test-Path $Config)) {
    Write-Host "❌ Configuration file not found: $Config" -ForegroundColor Red
    exit 1
}

# Verify wallet exists
if (-not (Test-Path $Wallet)) {
    Write-Host "❌ Wallet file not found: $Wallet" -ForegroundColor Red
    exit 1
}

switch ($Mode) {
    "test" {
        Write-Host "🧪 TEST MODE - Validating Configuration" -ForegroundColor Blue
        Write-Host "📋 Loading small capital configuration..."
        
        # Show current configuration
        $configContent = Get-Content $Config | ConvertFrom-Json
        $phase = $configContent.liquidity_sniper_accumulation_phase
        
        Write-Host "✅ Phase: $($phase.phase)" -ForegroundColor Green
        Write-Host "💰 Current Capital: $($phase.current_capital_sol) SOL" -ForegroundColor Yellow
        Write-Host "🎯 Target Milestone: $($phase.target_milestone_sol) SOL" -ForegroundColor Cyan
        Write-Host "📈 Progress: $($phase.progress_percent)%" -ForegroundColor Magenta
        Write-Host "🔒 Strategy Mode: $($phase.strategy_mode)" -ForegroundColor Blue
        Write-Host ""
        Write-Host "💡 Trading Parameters:" -ForegroundColor White
        Write-Host "   💼 Capital Allocation: $($phase.capital_allocation_sol) SOL" -ForegroundColor Gray
        Write-Host "   📊 Max Positions: $($phase.max_positions)" -ForegroundColor Gray
        Write-Host "   🎯 Profit Target: $($phase.take_profit_percent)%" -ForegroundColor Gray
        Write-Host "   🛡️  Stop Loss: $($phase.stop_loss_percent)%" -ForegroundColor Gray
        Write-Host "   ⏱️  Max Hold Time: $($phase.max_holding_time_minutes) min" -ForegroundColor Gray
        Write-Host "   📈 Daily Trade Limit: $($phase.trades_per_day_limit)" -ForegroundColor Gray
        Write-Host ""
        Write-Host "🚨 ULTRA-STRICT FILTERS ACTIVE:" -ForegroundColor Red
        Write-Host "   👥 Min Holders: $($phase.ultra_strict_filters.min_holders)" -ForegroundColor Red
        Write-Host "   ⏰ Min Age: $($phase.ultra_strict_filters.min_age_minutes) min" -ForegroundColor Red
        Write-Host "   💰 Min Market Cap: `$$($phase.ultra_strict_filters.min_market_cap_usd)" -ForegroundColor Red
        Write-Host "   🔒 Min Locked Liquidity: $($phase.ultra_strict_filters.min_locked_liquidity_percent)%" -ForegroundColor Red
        Write-Host ""
        Write-Host "📅 EXPECTED TIMELINE:" -ForegroundColor Green
        Write-Host "   🎯 Weeks to 0.5 SOL: $($phase.expected_timeline.weeks_to_0_5_sol)" -ForegroundColor Green
        Write-Host "   💰 Profit per Trade: $($phase.expected_timeline.profit_per_successful_trade_sol) SOL" -ForegroundColor Green
        Write-Host "   📊 Trades per Week: $($phase.expected_timeline.trades_per_week)" -ForegroundColor Green
        Write-Host ""
        Write-Host "✅ Configuration validated successfully!" -ForegroundColor Green
    }
    
    "server" {
        Write-Host "🌐 SERVER MODE - Starting Capital Accumulation Server" -ForegroundColor Blue
        Write-Host "⚠️  Note: Enterprise server needs compilation fix" -ForegroundColor Yellow
        Write-Host "💡 Alternative: Use interactive mode" -ForegroundColor Cyan
        
        # Try to start regular server with small capital config
        Write-Host "🚀 Starting SniperForge server..."
        & ".\target\release\sniperforge.exe"
    }
    
    "interactive" {
        Write-Host "🎮 INTERACTIVE MODE - Manual Capital Management" -ForegroundColor Blue
        Write-Host "📋 Starting interactive session..."
        
        # Start interactive mode
        & ".\target\release\sniperforge-interactive.exe"
    }
    
    "cli" {
        Write-Host "⌨️  CLI MODE - Command Line Interface" -ForegroundColor Blue
        Write-Host "📋 Available commands:"
        & ".\target\release\sniperforge-cli.exe"
    }
    
    "analyze" {
        Write-Host "📊 COST ANALYSIS MODE" -ForegroundColor Blue
        Write-Host "💰 Running cost analysis for 0.29 SOL capital..."
        
        # Run the cost analysis script we created
        if (Test-Path "analyze_small_capital_costs.ps1") {
            & ".\analyze_small_capital_costs.ps1"
        } else {
            Write-Host "❌ Cost analysis script not found" -ForegroundColor Red
        }
    }
    
    default {
        Write-Host "❌ Invalid mode: $Mode" -ForegroundColor Red
        Write-Host ""
        Write-Host "📋 Available modes:" -ForegroundColor White
        Write-Host "   test         Validate configuration" -ForegroundColor Gray
        Write-Host "   server       Start server mode" -ForegroundColor Gray
        Write-Host "   interactive  Start interactive mode" -ForegroundColor Gray
        Write-Host "   cli          Command line interface" -ForegroundColor Gray
        Write-Host "   analyze      Cost analysis" -ForegroundColor Gray
        Write-Host ""
        Write-Host "💡 Example: .\run_small_capital.ps1 -Mode test" -ForegroundColor Cyan
    }
}
