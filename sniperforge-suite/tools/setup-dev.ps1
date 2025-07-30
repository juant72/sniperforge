# SniperForge Development Setup Script (PowerShell)
# Sets up the development environment on Windows

param(
    [switch]$SkipTools = $false,
    [switch]$SkipVSCode = $false
)

Write-Host "🚀 Setting up SniperForge development environment..." -ForegroundColor Green

# Check if Rust is installed
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Rust is not installed. Please install Rust first:" -ForegroundColor Red
    Write-Host "   https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

Write-Host "✅ Rust installation found" -ForegroundColor Green

# Install required Rust components
Write-Host "📦 Installing Rust components..." -ForegroundColor Cyan
rustup component add rustfmt clippy

# Install development tools
if (-not $SkipTools) {
    Write-Host "🔧 Installing development tools..." -ForegroundColor Cyan
    cargo install cargo-watch cargo-tarpaulin cargo-audit cargo-outdated
}

# Check if Solana CLI is installed
if (-not (Get-Command solana -ErrorAction SilentlyContinue)) {
    Write-Host "📥 Solana CLI not found. Please install manually:" -ForegroundColor Yellow
    Write-Host "   https://docs.solana.com/cli/install-solana-cli-tools" -ForegroundColor Yellow
} else {
    Write-Host "✅ Solana CLI installation found" -ForegroundColor Green
}

# Create environment file template
Write-Host "📝 Creating environment template..." -ForegroundColor Cyan
$envTemplate = @"
# SniperForge Configuration Template
# Copy this file to .env and configure your settings

# Solana Configuration
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_WS_URL=wss://api.devnet.solana.com/

# Trading Configuration
MAX_SLIPPAGE=0.005
MIN_PROFIT_THRESHOLD=0.001
MAX_POSITION_SIZE=0.1

# Security
PRIVATE_KEY_PATH=./wallet.json
ENABLE_SIMULATION=true

# Logging
RUST_LOG=info
LOG_LEVEL=info

# DexScreener API
DEXSCREENER_BASE_URL=https://api.dexscreener.com

# Rate Limits
MAX_REQUESTS_PER_SECOND=10
COOLDOWN_PERIOD_MS=100
"@

$envTemplate | Out-File -FilePath ".env.example" -Encoding UTF8

# Install VS Code extensions if VS Code is available
if (-not $SkipVSCode -and (Get-Command code -ErrorAction SilentlyContinue)) {
    Write-Host "🔌 Installing VS Code extensions..." -ForegroundColor Cyan
    code --install-extension rust-lang.rust-analyzer
    code --install-extension vadimcn.vscode-lldb
    code --install-extension serayuzgur.crates
    code --install-extension tamasfe.even-better-toml
}

# Create PowerShell development scripts
Write-Host "📜 Creating PowerShell development scripts..." -ForegroundColor Cyan

# Watch script
$watchScript = @"
# Watch and rebuild on changes
param(
    [string]`$Target = "test"
)

switch (`$Target) {
    "test" { cargo watch -x test }
    "bot" { cargo watch -x "run --bin arbitrage-bot" }
    "build" { cargo watch -x build }
    default { 
        Write-Host "Usage: .\tools\watch.ps1 [test|bot|build]" -ForegroundColor Yellow
        Write-Host "  test  - Watch and run tests" -ForegroundColor Cyan
        Write-Host "  bot   - Watch and run arbitrage bot" -ForegroundColor Cyan
        Write-Host "  build - Watch and build project" -ForegroundColor Cyan
    }
}
"@

$watchScript | Out-File -FilePath "tools\watch.ps1" -Encoding UTF8

# Quality check script
$qualityScript = @"
# Run quality checks
param(
    [switch]`$Fix = `$false
)

Write-Host "🔍 Running quality checks..." -ForegroundColor Green

if (`$Fix) {
    Write-Host "🔧 Formatting code..." -ForegroundColor Cyan
    cargo fmt
    
    Write-Host "🔧 Fixing clippy issues..." -ForegroundColor Cyan
    cargo clippy --fix --allow-dirty --allow-staged
} else {
    Write-Host "📏 Checking formatting..." -ForegroundColor Cyan
    cargo fmt -- --check
    
    Write-Host "🔍 Running clippy..." -ForegroundColor Cyan
    cargo clippy -- -D warnings
}

Write-Host "🧪 Running tests..." -ForegroundColor Cyan
cargo test

Write-Host "🔒 Running security audit..." -ForegroundColor Cyan
cargo audit

Write-Host "✅ Quality checks complete!" -ForegroundColor Green
"@

$qualityScript | Out-File -FilePath "tools\quality.ps1" -Encoding UTF8

# Build the project
Write-Host "🔨 Building project..." -ForegroundColor Cyan
cargo build

# Run initial tests
Write-Host "🧪 Running tests..." -ForegroundColor Cyan
cargo test

Write-Host ""
Write-Host "🎉 Development environment setup complete!" -ForegroundColor Green
Write-Host ""
Write-Host "📋 Next steps:" -ForegroundColor Yellow
Write-Host "   1. Copy .env.example to .env and configure your settings" -ForegroundColor Cyan
Write-Host "   2. Add your Solana wallet to the wallet.json file" -ForegroundColor Cyan
Write-Host "   3. Run 'cargo run --bin arbitrage-bot' to start the bot" -ForegroundColor Cyan
Write-Host ""
Write-Host "🔧 Development commands:" -ForegroundColor Yellow
Write-Host "   .\tools\watch.ps1 test        # Watch and run tests" -ForegroundColor Cyan
Write-Host "   .\tools\watch.ps1 bot         # Watch and run bot" -ForegroundColor Cyan
Write-Host "   .\tools\quality.ps1           # Run quality checks" -ForegroundColor Cyan
Write-Host "   .\tools\quality.ps1 -Fix      # Fix issues automatically" -ForegroundColor Cyan
Write-Host "   cargo clippy                  # Run linting" -ForegroundColor Cyan
Write-Host "   cargo fmt                     # Format code" -ForegroundColor Cyan
Write-Host "   cargo audit                   # Security audit" -ForegroundColor Cyan
Write-Host ""
