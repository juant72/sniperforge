# Tatum API Key Setup Script
# This script sets up the Tatum API keys for SniperForge

Write-Host "Setting up Tatum API keys for SniperForge..." -ForegroundColor Green

# API Keys provided by the user
$TATUM_MAINNET_KEY = "t-67b3d0b4dff4f7a9cf84fbf7-e095b9354ff54bc59b09fc04"
$TATUM_DEVNET_KEY = "t-67b3d0b4dff4f7a9cf84fbf7-687708fdb90e4aa59ff9a9cb"

Write-Host "Setting environment variables..." -ForegroundColor Yellow

# Set environment variables for current session
$env:TATUM_API_KEY_MAINNET = $TATUM_MAINNET_KEY
$env:TATUM_API_KEY_DEVNET = $TATUM_DEVNET_KEY

Write-Host "Environment variables set for current session:" -ForegroundColor Green
Write-Host "   TATUM_API_KEY_MAINNET = $($TATUM_MAINNET_KEY.Substring(0, 20))..." -ForegroundColor Gray
Write-Host "   TATUM_API_KEY_DEVNET = $($TATUM_DEVNET_KEY.Substring(0, 20))..." -ForegroundColor Gray

# Set persistent environment variables (user level)
Write-Host "Setting persistent environment variables..." -ForegroundColor Yellow

try {
    [Environment]::SetEnvironmentVariable("TATUM_API_KEY_MAINNET", $TATUM_MAINNET_KEY, "User")
    [Environment]::SetEnvironmentVariable("TATUM_API_KEY_DEVNET", $TATUM_DEVNET_KEY, "User")
    Write-Host "Persistent environment variables set successfully!" -ForegroundColor Green
} catch {
    Write-Host "Failed to set persistent environment variables: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "   You may need to run this script as Administrator" -ForegroundColor Yellow
}

# Test the environment variables
Write-Host "Testing environment variables..." -ForegroundColor Yellow

$mainnet_test = [Environment]::GetEnvironmentVariable("TATUM_API_KEY_MAINNET", "User")
$devnet_test = [Environment]::GetEnvironmentVariable("TATUM_API_KEY_DEVNET", "User")

if ($mainnet_test -eq $TATUM_MAINNET_KEY) {
    Write-Host "Mainnet API key verified" -ForegroundColor Green
} else {
    Write-Host "Mainnet API key verification failed" -ForegroundColor Red
}

if ($devnet_test -eq $TATUM_DEVNET_KEY) {
    Write-Host "Devnet API key verified" -ForegroundColor Green
} else {
    Write-Host "Devnet API key verification failed" -ForegroundColor Red
}

Write-Host ""
Write-Host "Configuration Summary:" -ForegroundColor Cyan
Write-Host "=========================" -ForegroundColor Cyan
Write-Host "Tatum Mainnet endpoint: https://solana-mainnet.gateway.tatum.io" -ForegroundColor White
Write-Host "Tatum Devnet endpoint:  https://solana-devnet.gateway.tatum.io" -ForegroundColor White
Write-Host "Authentication method:  x-api-key header" -ForegroundColor White
Write-Host "Priority in mainnet:    4 (after QuickNode, Helius, Alchemy)" -ForegroundColor White
Write-Host "Priority in devnet:     3 (after Helius, Alchemy)" -ForegroundColor White

Write-Host ""
Write-Host "Next Steps:" -ForegroundColor Yellow
Write-Host "1. Restart PowerShell/VS Code to refresh environment variables" -ForegroundColor White
Write-Host "2. Enable premium RPC in config: Set premium_rpc.enabled = true" -ForegroundColor White
Write-Host "3. Test with: cargo run -- test rpc" -ForegroundColor White
Write-Host "4. Run integration test: cargo run -- test integration" -ForegroundColor White

Write-Host ""
Write-Host "Note: Tatum endpoints use header authentication (x-api-key)" -ForegroundColor Cyan
Write-Host "   This is different from URL-based authentication used by other providers" -ForegroundColor Gray

Write-Host ""
Write-Host "Tatum setup completed successfully!" -ForegroundColor Green
