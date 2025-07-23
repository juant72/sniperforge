# Setup script for Alternative APIs configuration (Birdeye, etc.)
# This script helps configure API keys for external data sources

param(
    [string]$BirdeyeApiKey = "",
    [switch]$ShowCurrent,
    [switch]$Clear,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

function Show-Help {
    Write-Host ""
    Write-Host "🔧 Alternative APIs Configuration Script" -ForegroundColor Cyan
    Write-Host "=======================================" -ForegroundColor Blue
    Write-Host ""
    Write-Host "This script configures API keys for external data sources used by SniperForge:"
    Write-Host ""
    Write-Host "📊 Supported APIs:" -ForegroundColor Yellow
    Write-Host "  • Birdeye API - Token data and market analytics"
    Write-Host "  • DexScreener API - Pool information (no key required)"
    Write-Host "  • Raydium API - DEX data (no key required)"
    Write-Host "  • Jupiter API - Pricing and routing (no key required)"
    Write-Host ""
    Write-Host "🔑 Usage:" -ForegroundColor Green
    Write-Host "  .\setup-alternative-apis.ps1 -BirdeyeApiKey ""your-api-key-here"""
    Write-Host "  .\setup-alternative-apis.ps1 -ShowCurrent"
    Write-Host "  .\setup-alternative-apis.ps1 -Clear"
    Write-Host ""
    Write-Host "📋 Examples:" -ForegroundColor Cyan
    Write-Host "  # Set Birdeye API key"
    Write-Host "  .\setup-alternative-apis.ps1 -BirdeyeApiKey ""c0c3df44d98c4baf8099542c7064e0b0"""
    Write-Host ""
    Write-Host "  # Show current configuration"
    Write-Host "  .\setup-alternative-apis.ps1 -ShowCurrent"
    Write-Host ""
    Write-Host "  # Clear all API keys"
    Write-Host "  .\setup-alternative-apis.ps1 -Clear"
    Write-Host ""
    Write-Host "🌐 Getting API Keys:" -ForegroundColor Magenta
    Write-Host "  Birdeye API: https://birdeye.so/developers"
    Write-Host "    - Free tier: 1,000 requests/day"
    Write-Host "    - Paid plans: Up to 100,000+ requests/day"
    Write-Host ""
    Write-Host "⚠️  Security Notes:" -ForegroundColor Red
    Write-Host "  • API keys are stored as environment variables"
    Write-Host "  • Never commit API keys to version control"
    Write-Host "  • Use different keys for development and production"
    Write-Host ""
}

function Show-CurrentConfig {
    Write-Host ""
    Write-Host "📊 Current Alternative APIs Configuration" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Blue
    Write-Host ""
    
    # Check Birdeye API key
    $birdeyeKey = [Environment]::GetEnvironmentVariable("BIRDEYE_API_KEY", "User")
    if ($birdeyeKey) {
        $maskedKey = $birdeyeKey.Substring(0, 8) + "..." + $birdeyeKey.Substring($birdeyeKey.Length - 4)
        Write-Host "🐦 Birdeye API Key: $maskedKey" -ForegroundColor Green
        Write-Host "   Status: Configured ✅" -ForegroundColor Green
    } else {
        Write-Host "🐦 Birdeye API Key: Not configured" -ForegroundColor Yellow
        Write-Host "   Status: Missing ⚠️" -ForegroundColor Yellow
    }
    
    Write-Host ""
    Write-Host "📡 Other APIs (no key required):" -ForegroundColor Cyan
    Write-Host "  • DexScreener API: Available ✅"
    Write-Host "  • Raydium API: Available ✅"
    Write-Host "  • Jupiter API: Available ✅"
    
    Write-Host ""
    Write-Host "📋 Environment Variables:" -ForegroundColor Cyan
    Write-Host "  BIRDEYE_API_KEY = $(if ($birdeyeKey) { 'Set' } else { 'Not Set' })"
    
    Write-Host ""
    Write-Host "🔧 To configure missing APIs:" -ForegroundColor Yellow
    Write-Host "  .\setup-alternative-apis.ps1 -BirdeyeApiKey ""your-key-here"""
    Write-Host ""
}

function Clear-ApiKeys {
    Write-Host ""
    Write-Host "🧹 Clearing Alternative API Keys" -ForegroundColor Yellow
    Write-Host "================================" -ForegroundColor Blue
    Write-Host ""
    
    # Remove Birdeye API key
    Write-Host "Removing BIRDEYE_API_KEY..." -ForegroundColor Yellow
    [Environment]::SetEnvironmentVariable("BIRDEYE_API_KEY", $null, "User")
    
    Write-Host ""
    Write-Host "✅ All alternative API keys cleared!" -ForegroundColor Green
    Write-Host ""
    Write-Host "💡 Note: You may need to restart your terminal for changes to take effect."
    Write-Host ""
}

function Set-BirdeyeApiKey {
    param([string]$ApiKey)
    
    if ([string]::IsNullOrWhiteSpace($ApiKey)) {
        Write-Error "Birdeye API key cannot be empty"
        return
    }
    
    # Validate API key format (basic validation)
    if ($ApiKey.Length -lt 10) {
        Write-Error "Birdeye API key seems too short. Please check your key."
        return
    }
    
    Write-Host ""
    Write-Host "🔑 Configuring Birdeye API Key" -ForegroundColor Cyan
    Write-Host "==============================" -ForegroundColor Blue
    Write-Host ""
    
    # Set the environment variable
    Write-Host "Setting BIRDEYE_API_KEY..." -ForegroundColor Yellow
    [Environment]::SetEnvironmentVariable("BIRDEYE_API_KEY", $ApiKey, "User")
    
    # Verify it was set
    $verifyKey = [Environment]::GetEnvironmentVariable("BIRDEYE_API_KEY", "User")
    if ($verifyKey -eq $ApiKey) {
        $maskedKey = $ApiKey.Substring(0, 8) + "..." + $ApiKey.Substring($ApiKey.Length - 4)
        Write-Host "✅ Birdeye API key configured successfully!" -ForegroundColor Green
        Write-Host "   Key: $maskedKey" -ForegroundColor Green
        Write-Host ""
        Write-Host "🔄 Testing API key..." -ForegroundColor Yellow
        
        # Test the API key by making a simple request
        try {
            $testUrl = "https://public-api.birdeye.so/public/tokenlist?sort_by=v24hUSD&sort_type=desc&offset=0&limit=1"
            $headers = @{
                "X-API-KEY" = $ApiKey
                "Accept" = "application/json"
            }
            
            $response = Invoke-RestMethod -Uri $testUrl -Headers $headers -TimeoutSec 10
            
            if ($response) {
                Write-Host "✅ API key test successful! Birdeye API is responding." -ForegroundColor Green
            }
        }
        catch {
            Write-Host "⚠️  API key set but test failed. This might be normal if you're offline or the API is busy." -ForegroundColor Yellow
            Write-Host "   Error: $($_.Exception.Message)" -ForegroundColor Yellow
        }
        
        Write-Host ""
        Write-Host "🎯 Next steps:" -ForegroundColor Cyan
        Write-Host "  1. Restart your terminal to load the new environment variable"
        Write-Host "  2. Test the integration with: cargo run --bin sniperforge test dexscreener"
        Write-Host "  3. Use in trading: cargo run --bin sniperforge start --network devnet"
        Write-Host ""
        Write-Host "💡 The API key is now available to SniperForge and will be used automatically."
        Write-Host ""
    } else {
        Write-Error "Failed to set API key. Please try again or set manually."
    }
}

# Main script logic
if ($Help) {
    Show-Help
    exit 0
}

if ($ShowCurrent) {
    Show-CurrentConfig
    exit 0
}

if ($Clear) {
    Clear-ApiKeys
    exit 0
}

if (-not [string]::IsNullOrWhiteSpace($BirdeyeApiKey)) {
    Set-BirdeyeApiKey -ApiKey $BirdeyeApiKey
    exit 0
}

# No parameters provided, show help
Write-Host ""
Write-Host "🔧 Alternative APIs Configuration" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Blue
Write-Host ""
Write-Host "No parameters provided. Use -Help for detailed usage information." -ForegroundColor Yellow
Write-Host ""
Write-Host "Quick setup:" -ForegroundColor Cyan
Write-Host "  .\setup-alternative-apis.ps1 -BirdeyeApiKey ""c0c3df44d98c4baf8099542c7064e0b0"""
Write-Host ""
Write-Host "Current status:" -ForegroundColor Cyan
Show-CurrentConfig
