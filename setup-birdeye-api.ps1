# Birdeye API Key Setup Script for SniperForge
# This script helps you configure the Birdeye API key for enhanced data access

Write-Host "`n🐦 SniperForge - Birdeye API Key Setup" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Check if API key is already set
$existingKey = [Environment]::GetEnvironmentVariable("BIRDEYE_API_KEY", "User")

if ($existingKey) {
    Write-Host "`n✅ Birdeye API key is already configured!" -ForegroundColor Green
    $maskedKey = $existingKey.Substring(0, 8) + "..." + $existingKey.Substring($existingKey.Length - 4)
    Write-Host "   Current key: $maskedKey" -ForegroundColor Yellow
    
    $choice = Read-Host "`nDo you want to update it? (y/N)"
    if ($choice -notmatch '^[yY]$') {
        Write-Host "`n👍 Keeping existing API key. You're all set!" -ForegroundColor Green
        exit 0
    }
}

Write-Host "`n📝 Instructions:" -ForegroundColor Yellow
Write-Host "1. Visit https://docs.birdeye.so/docs/authentication-api-keys" -ForegroundColor White
Write-Host "2. Sign up for a Birdeye account if you don't have one" -ForegroundColor White
Write-Host "3. Generate your API key from the dashboard" -ForegroundColor White
Write-Host "4. Copy the API key and paste it below" -ForegroundColor White

Write-Host "`n💡 Benefits of Birdeye API:" -ForegroundColor Yellow
Write-Host "• Higher rate limits (1000+ req/min vs public limits)" -ForegroundColor White
Write-Host "• Access to premium endpoints" -ForegroundColor White
Write-Host "• Better data accuracy and freshness" -ForegroundColor White
Write-Host "• Historical price data access" -ForegroundColor White

# Pre-fill with the provided API key
$defaultKey = "c0c3df44d98c4baf8099542c7064e0b0"
Write-Host "`n🔑 Enter your Birdeye API key:" -ForegroundColor Cyan
Write-Host "   (Press Enter to use the provided key: $($defaultKey.Substring(0,8))...)" -ForegroundColor Gray

$apiKey = Read-Host

# Use default if empty
if ([string]::IsNullOrWhiteSpace($apiKey)) {
    $apiKey = $defaultKey
    Write-Host "   Using provided API key..." -ForegroundColor Green
}

# Validate API key format (basic validation)
if ($apiKey.Length -lt 20) {
    Write-Host "`n❌ Error: API key seems too short. Please check and try again." -ForegroundColor Red
    exit 1
}

try {
    # Set environment variable for current user
    [Environment]::SetEnvironmentVariable("BIRDEYE_API_KEY", $apiKey, "User")
    
    # Also set for current session
    $env:BIRDEYE_API_KEY = $apiKey
    
    Write-Host "`n✅ Birdeye API key configured successfully!" -ForegroundColor Green
    Write-Host "`n📋 Configuration Summary:" -ForegroundColor Cyan
    Write-Host "   • Environment Variable: BIRDEYE_API_KEY" -ForegroundColor White
    Write-Host "   • Scope: Current User" -ForegroundColor White
    Write-Host "   • Effective: Immediately for new processes" -ForegroundColor White
    
    $maskedKey = $apiKey.Substring(0, 8) + "..." + $apiKey.Substring($apiKey.Length - 4)
    Write-Host "   • Value: $maskedKey" -ForegroundColor White
    
    Write-Host "`n🔄 Next Steps:" -ForegroundColor Yellow
    Write-Host "1. Restart your terminal/IDE for the changes to take effect" -ForegroundColor White
    Write-Host "2. Run SniperForge tests to verify the API key works:" -ForegroundColor White
    Write-Host "   cargo run -- test dexscreener" -ForegroundColor Gray
    Write-Host "3. Check the logs for 'Using Birdeye API key' messages" -ForegroundColor White
    
    Write-Host "`n⚠️  Security Note:" -ForegroundColor Yellow
    Write-Host "Keep your API key secure and never commit it to version control!" -ForegroundColor White
    
} catch {
    Write-Host "`n❌ Error setting environment variable: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

Write-Host "`n🎉 Setup complete! SniperForge will now use Birdeye API for enhanced data access." -ForegroundColor Green
