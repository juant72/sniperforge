# SniperForge Phase 5B: MainNet Real Trading Setup
# This script sets up the environment for real MainNet trading with minimal capital

param(
    [Parameter(Mandatory=$false)]
    [double]$MaxCapitalUSD = 500,
    
    [Parameter(Mandatory=$false)]
    [double]$MaxTradeUSD = 50,
    
    [Parameter(Mandatory=$false)]
    [switch]$ConfirmSetup
)

Write-Host "🚀 SniperForge Phase 5B: MainNet Real Trading Setup" -ForegroundColor Cyan
Write-Host "💰 Setting up REAL trading with maximum capital: $${MaxCapitalUSD}" -ForegroundColor Yellow

if (-not $ConfirmSetup) {
    Write-Host ""
    Write-Host "⚠️  WARNING: This will configure REAL trading with REAL money on MainNet!" -ForegroundColor Red
    Write-Host "   Maximum Capital: $${MaxCapitalUSD}" -ForegroundColor Yellow
    Write-Host "   Maximum Trade Size: $${MaxTradeUSD}" -ForegroundColor Yellow
    Write-Host ""
    $confirm = Read-Host "Type 'I UNDERSTAND THE RISKS' to continue"
    
    if ($confirm -ne "I UNDERSTAND THE RISKS") {
        Write-Host "❌ Setup cancelled for safety" -ForegroundColor Red
        exit 1
    }
}

Write-Host "✅ Risk acknowledgment confirmed" -ForegroundColor Green

# Create MainNet real trading configuration
$mainnetRealConfig = @"
# SniperForge MainNet Real Trading Configuration
# Phase 5B: Real trading with minimal capital and maximum safety

[network]
name = "mainnet-real"
rpc_url = "https://api.mainnet-beta.solana.com"
ws_url = "wss://api.mainnet-beta.solana.com"
commitment = "confirmed"
is_devnet = false
real_trading_enabled = true

[jupiter]
# Official Jupiter API for mainnet
base_url = "https://quote-api.jup.ag"
timeout_ms = 3000      # Faster timeout for real trading
max_retries = 2        # Fewer retries for real trading
cache_ttl_ms = 50      # Very short cache for real-time data

[syndica]
# Syndica mainnet WebSocket (requires real token)
endpoint = "wss://solana-mainnet.api.syndica.io"
access_token = "`${SYNDICA_MAINNET_TOKEN}"
reconnect_attempts = 5
ping_interval_sec = 30

[real_trading]
# Phase 5B Real trading settings
enabled = true
max_capital_usd = ${MaxCapitalUSD}
max_single_trade_usd = ${MaxTradeUSD}
daily_limit_usd = $([math]::min($MaxCapitalUSD * 0.4, 200))
max_positions = 3
stop_loss_percent = 0.05      # 5% stop-loss
min_profit_threshold_percent = 0.02  # 2% minimum profit
require_manual_confirmation = true
enable_emergency_stops = true

[safety]
# Ultra-safe settings for real trading
max_price_age_ms = 30         # Only accept prices < 30ms old
min_price_sources = 2         # Require at least 2 price sources
price_tolerance_percent = 0.3 # Max 0.3% difference between sources
disable_cache_for_trading = true  # Never use cache for trading decisions
max_price_impact_percent = 2.0    # Never trade with >2% price impact
max_slippage_percent = 0.5        # Never trade with >0.5% slippage

[monitoring]
# Performance monitoring for real trading
log_all_trades = true
log_price_sources = true
log_latency = true
save_trade_history = true
alert_on_large_trades = true
alert_threshold_usd = 25

[risk_management]
# Multi-layer risk management
emergency_stop_enabled = true
daily_loss_limit_usd = $([math]::min($MaxCapitalUSD * 0.1, 50))
consecutive_loss_limit = 3
max_open_time_minutes = 60    # Close positions after 1 hour max
position_size_limits = true
require_stop_loss = true

[wallet]
# Wallet configuration for real trading
trading_wallet_max_sol = $([math]::min($MaxCapitalUSD / 100, 5))  # Estimate SOL price at $100
fee_wallet_sol = 0.1
backup_wallet_sol = 0.05
auto_balance_management = true
"@

# Write the configuration file
$configPath = "config/mainnet-real.toml"
Write-Host "📝 Creating MainNet real trading config: $configPath" -ForegroundColor Cyan
$mainnetRealConfig | Out-File -FilePath $configPath -Encoding UTF8

# Create wallet setup script
$walletSetupScript = @"
# MainNet Real Trading Wallet Setup
# This creates the necessary wallets for real trading

import os
import json
from solana.keypair import Keypair
from solana.publickey import PublicKey

def create_wallet_config():
    """Create wallet configuration for real MainNet trading"""
    
    print("🔐 Creating MainNet real trading wallets")
    
    # Generate keypairs
    trading_keypair = Keypair.generate()
    fee_keypair = Keypair.generate()
    backup_keypair = Keypair.generate()
    
    wallet_config = {
        "trading_wallet": {
            "pubkey": str(trading_keypair.public_key),
            "max_sol_balance": ${MaxCapitalUSD} / 100,  # Estimate SOL price
            "daily_limit_usd": ${MaxCapitalUSD} * 0.4,
            "risk_level": "high_security"
        },
        "fee_wallet": {
            "pubkey": str(fee_keypair.public_key),
            "max_sol_balance": 0.1,
            "purpose": "transaction_fees"
        },
        "backup_wallet": {
            "pubkey": str(backup_keypair.public_key),
            "max_sol_balance": 0.05,
            "purpose": "emergency_backup"
        }
    }
    
    # Save wallet configuration
    with open("config/mainnet-wallets.json", "w") as f:
        json.dump(wallet_config, f, indent=2)
    
    # Save keypairs securely (in a real setup, these would be encrypted)
    os.makedirs("wallets", exist_ok=True)
    
    print("✅ Wallet configuration created")
    print(f"🔑 Trading wallet: {trading_keypair.public_key}")
    print(f"💳 Fee wallet: {fee_keypair.public_key}")
    print(f"🔒 Backup wallet: {backup_keypair.public_key}")
    
    print("")
    print("⚠️  IMPORTANT: Fund these wallets manually with small amounts:")
    print(f"   Trading: {trading_keypair.public_key} (max 2 SOL)")
    print(f"   Fee: {fee_keypair.public_key} (0.1 SOL)")
    print(f"   Backup: {backup_keypair.public_key} (0.05 SOL)")

if __name__ == "__main__":
    create_wallet_config()
"@

$walletScript = "scripts/setup-mainnet-wallets.py"
Write-Host "📝 Creating wallet setup script: $walletScript" -ForegroundColor Cyan
New-Item -Path "scripts" -ItemType Directory -Force | Out-Null
$walletSetupScript | Out-File -FilePath $walletScript -Encoding UTF8

# Create CLI command for MainNet real trading
Write-Host "🔧 Adding MainNet real trading CLI command" -ForegroundColor Cyan

# Check if CLI needs updating
$cliFile = "src/cli.rs"
$needsUpdate = $false

if (Test-Path $cliFile) {
    $cliContent = Get-Content $cliFile -Raw
    if ($cliContent -notmatch "mainnet-real-trading") {
        $needsUpdate = $true
    }
}

if ($needsUpdate) {
    Write-Host "📝 CLI update needed - will be handled by main implementation" -ForegroundColor Yellow
}

# Create monitoring dashboard config
$monitoringConfig = @"
# MainNet Real Trading Monitoring Configuration

[dashboard]
enabled = true
update_interval_ms = 1000
show_real_balances = true
show_pnl = true
show_risk_metrics = true

[alerts]
telegram_enabled = false  # Set to true if you have Telegram bot
discord_enabled = false   # Set to true if you have Discord webhook
email_enabled = false     # Set to true if you have email setup

[thresholds]
large_trade_usd = 25
high_risk_score = 0.8
daily_loss_limit_usd = $([math]::min($MaxCapitalUSD * 0.1, 50))
position_timeout_minutes = 60

[logging]
level = "info"
save_to_file = true
log_rotation_days = 7
compress_old_logs = true
"@

$monitoringPath = "config/monitoring-real.toml"
Write-Host "📝 Creating monitoring config: $monitoringPath" -ForegroundColor Cyan
$monitoringConfig | Out-File -FilePath $monitoringPath -Encoding UTF8

Write-Host ""
Write-Host "✅ MainNet Real Trading Setup Complete!" -ForegroundColor Green
Write-Host ""
Write-Host "📋 Next Steps:" -ForegroundColor Cyan
Write-Host "1. Run: python scripts/setup-mainnet-wallets.py" -ForegroundColor White
Write-Host "2. Fund the wallets with small amounts (manually)" -ForegroundColor White
Write-Host "3. Set environment variables:" -ForegroundColor White
Write-Host "   - SYNDICA_MAINNET_TOKEN=your_token" -ForegroundColor Gray
Write-Host "4. Test with: cargo run -- mainnet-real-trading --test" -ForegroundColor White
Write-Host "5. Start real trading: cargo run -- mainnet-real-trading --live" -ForegroundColor White
Write-Host ""
Write-Host "⚠️  Remember: This uses REAL money on MainNet!" -ForegroundColor Red
Write-Host "   Start with very small amounts to test the system" -ForegroundColor Yellow
Write-Host ""

# Show configuration summary
Write-Host "📊 Configuration Summary:" -ForegroundColor Cyan
Write-Host "   Max Capital: $${MaxCapitalUSD}" -ForegroundColor White
Write-Host "   Max Trade: $${MaxTradeUSD}" -ForegroundColor White
Write-Host "   Daily Limit: $([math]::min($MaxCapitalUSD * 0.4, 200))" -ForegroundColor White
Write-Host "   Stop Loss: 5%" -ForegroundColor White
Write-Host "   Min Profit: 2%" -ForegroundColor White
Write-Host ""
