# SniperForge Premium RPC Setup Script
# This script helps you configure premium RPC API keys on Windows

Write-Host "🌟 SniperForge Premium RPC Setup" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "This script will help you configure premium RPC endpoints for better" -ForegroundColor Yellow
Write-Host "reliability and performance on Solana mainnet." -ForegroundColor Yellow
Write-Host ""

# Function to set environment variable persistently
function Set-PermanentEnvVar {
    param(
        [string]$Name,
        [string]$Value
    )
    
    # Set for current session
    [Environment]::SetEnvironmentVariable($Name, $Value, "Process")
    
    # Set permanently for user
    [Environment]::SetEnvironmentVariable($Name, $Value, "User")
    
    Write-Host "✅ Set $Name environment variable" -ForegroundColor Green
}

# Function to prompt for API key
function Prompt-ForApiKey {
    param(
        [string]$Provider,
        [string]$EnvVarName,
        [string]$Description,
        [string]$SignupUrl
    )
    
    Write-Host ""
    Write-Host "🔑 $Provider Setup" -ForegroundColor Cyan
    Write-Host "Description: $Description" -ForegroundColor Gray
    Write-Host "Sign up at: $SignupUrl" -ForegroundColor Gray
    Write-Host ""
    
    $existingValue = [Environment]::GetEnvironmentVariable($EnvVarName, "User")
    if ($existingValue) {
        Write-Host "Current value: $($existingValue.Substring(0, [Math]::Min(20, $existingValue.Length)))..." -ForegroundColor Yellow
        $response = Read-Host "Keep existing value? (y/n)"
        if ($response -eq "y" -or $response -eq "Y" -or $response -eq "") {
            Write-Host "✅ Keeping existing $Provider configuration" -ForegroundColor Green
            return
        }
    }
    
    $value = Read-Host "Enter your $Provider API key/endpoint (or press Enter to skip)"
    if ($value -and $value.Trim() -ne "") {
        Set-PermanentEnvVar -Name $EnvVarName -Value $value.Trim()
    } else {
        Write-Host "⏭️ Skipping $Provider configuration" -ForegroundColor Yellow
    }
}

Write-Host "Let's configure your premium RPC providers..." -ForegroundColor Green
Write-Host ""

# Helius
Prompt-ForApiKey -Provider "Helius" -EnvVarName "HELIUS_API_KEY" -Description "Solana-specialized RPC with enhanced APIs" -SignupUrl "https://helius.xyz"

# QuickNode  
Prompt-ForApiKey -Provider "QuickNode" -EnvVarName "QUICKNODE_ENDPOINT" -Description "Enterprise-grade global infrastructure" -SignupUrl "https://quicknode.com"

# Alchemy
Prompt-ForApiKey -Provider "Alchemy" -EnvVarName "ALCHEMY_API_KEY" -Description "Developer-friendly with enhanced APIs" -SignupUrl "https://alchemy.com"

# Ankr
Prompt-ForApiKey -Provider "Ankr" -EnvVarName "ANKR_API_KEY" -Description "Cost-effective with global CDN" -SignupUrl "https://ankr.com"

Write-Host ""
Write-Host "🎯 Configuration Summary" -ForegroundColor Cyan
Write-Host "========================" -ForegroundColor Cyan

$configuredProviders = @()

if ([Environment]::GetEnvironmentVariable("HELIUS_API_KEY", "User")) {
    $configuredProviders += "Helius"
}
if ([Environment]::GetEnvironmentVariable("QUICKNODE_ENDPOINT", "User")) {
    $configuredProviders += "QuickNode"
}
if ([Environment]::GetEnvironmentVariable("ALCHEMY_API_KEY", "User")) {
    $configuredProviders += "Alchemy"
}
if ([Environment]::GetEnvironmentVariable("ANKR_API_KEY", "User")) {
    $configuredProviders += "Ankr"
}

if ($configuredProviders.Count -gt 0) {
    Write-Host "✅ Configured providers: $($configuredProviders -join ', ')" -ForegroundColor Green
    Write-Host ""
    
    # Enable premium RPC in config
    Write-Host "📝 Enabling premium RPC in configuration..." -ForegroundColor Yellow
    
    $configPath = "config\mainnet.toml"
    if (Test-Path $configPath) {
        $configContent = Get-Content $configPath -Raw
        if ($configContent -match 'enabled = false') {
            $newContent = $configContent -replace 'enabled = false', 'enabled = true'
            Set-Content -Path $configPath -Value $newContent
            Write-Host "✅ Enabled premium RPC in $configPath" -ForegroundColor Green
        } else {
            Write-Host "✅ Premium RPC already enabled in $configPath" -ForegroundColor Green
        }
    } else {
        Write-Host "⚠️  Config file not found: $configPath" -ForegroundColor Yellow
        Write-Host "   You may need to manually enable premium RPC in your config" -ForegroundColor Yellow
    }
    
    Write-Host ""
    Write-Host "🚀 Next Steps:" -ForegroundColor Cyan
    Write-Host "1. Restart your terminal/PowerShell session" -ForegroundColor White
    Write-Host "2. Run: cargo run --bin sniperforge test basic --network mainnet" -ForegroundColor White
    Write-Host "3. Look for 'Premium endpoints' in the log output" -ForegroundColor White
    Write-Host ""
    Write-Host "💡 Tip: You can add more providers later by running this script again" -ForegroundColor Yellow
    
} else {
    Write-Host "⚠️  No premium providers configured" -ForegroundColor Yellow
    Write-Host "   SniperForge will continue using public RPC endpoints" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "💡 You can run this script again anytime to add premium providers" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "📚 For more information, see: docs\user-guides\premium-rpc-setup.md" -ForegroundColor Gray
Write-Host ""
Write-Host "Press any key to exit..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
