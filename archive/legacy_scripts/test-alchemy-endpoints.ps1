# Test Alchemy RPC Endpoints
# This script tests the Alchemy premium RPC endpoints for connectivity and performance

Write-Host "🔍 Testing Alchemy RPC Endpoints" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan
Write-Host ""

# Alchemy endpoints
$ALCHEMY_DEVNET = "https://solana-devnet.g.alchemy.com/v2/X64q4zZFEMz_RYzthxUMg"
$ALCHEMY_MAINNET = "https://solana-mainnet.g.alchemy.com/v2/X64q4zZFEMz_RYzthxUMg"

# Test function
function Test-RpcEndpoint {
    param(
        [string]$Endpoint,
        [string]$Name
    )
    
    Write-Host "Testing $Name..." -ForegroundColor Yellow
    
    $testPayload = @{
        jsonrpc = "2.0"
        id = 1
        method = "getHealth"
        params = @()
    } | ConvertTo-Json
    
    try {
        $startTime = Get-Date
        $response = Invoke-RestMethod -Uri $Endpoint -Method Post -Body $testPayload -ContentType "application/json" -TimeoutSec 10
        $endTime = Get-Date
        $responseTime = ($endTime - $startTime).TotalMilliseconds
        
        if ($response.result -eq "ok") {
            Write-Host "✅ ${Name}: HEALTHY (${responseTime}ms)" -ForegroundColor Green
            return $true
        } else {
            Write-Host "⚠️  ${Name}: Response not OK - $($response.result)" -ForegroundColor Yellow
            return $false
        }
    } catch {
        Write-Host "❌ ${Name}: ERROR - $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Test version endpoint
function Test-RpcVersion {
    param(
        [string]$Endpoint,
        [string]$Name
    )
    
    $testPayload = @{
        jsonrpc = "2.0"
        id = 1
        method = "getVersion"
        params = @()
    } | ConvertTo-Json
    
    try {
        $response = Invoke-RestMethod -Uri $Endpoint -Method Post -Body $testPayload -ContentType "application/json" -TimeoutSec 10
        if ($response.result) {
            Write-Host "   Version: $($response.result.'solana-core')" -ForegroundColor Gray
        }
    } catch {
        Write-Host "   Version check failed" -ForegroundColor Red
    }
}

# Test slot endpoint
function Test-RpcSlot {
    param(
        [string]$Endpoint,
        [string]$Name
    )
    
    $testPayload = @{
        jsonrpc = "2.0"
        id = 1
        method = "getSlot"
        params = @()
    } | ConvertTo-Json
    
    try {
        $response = Invoke-RestMethod -Uri $Endpoint -Method Post -Body $testPayload -ContentType "application/json" -TimeoutSec 10
        if ($response.result) {
            Write-Host "   Current Slot: $($response.result)" -ForegroundColor Gray
        }
    } catch {
        Write-Host "   Slot check failed" -ForegroundColor Red
    }
}

Write-Host "🌐 Testing Alchemy DevNet..." -ForegroundColor Magenta
if (Test-RpcEndpoint -Endpoint $ALCHEMY_DEVNET -Name "Alchemy DevNet") {
    Test-RpcVersion -Endpoint $ALCHEMY_DEVNET -Name "Alchemy DevNet"
    Test-RpcSlot -Endpoint $ALCHEMY_DEVNET -Name "Alchemy DevNet"
}

Write-Host ""
Write-Host "🌐 Testing Alchemy MainNet..." -ForegroundColor Magenta
if (Test-RpcEndpoint -Endpoint $ALCHEMY_MAINNET -Name "Alchemy MainNet") {
    Test-RpcVersion -Endpoint $ALCHEMY_MAINNET -Name "Alchemy MainNet"
    Test-RpcSlot -Endpoint $ALCHEMY_MAINNET -Name "Alchemy MainNet"
}

Write-Host ""
Write-Host "🔧 Environment Check:" -ForegroundColor Cyan
Write-Host "ALCHEMY_DEVNET_RPC: $env:ALCHEMY_DEVNET_RPC" -ForegroundColor Gray
Write-Host "ALCHEMY_MAINNET_RPC: $env:ALCHEMY_MAINNET_RPC" -ForegroundColor Gray

Write-Host ""
Write-Host "✨ Alchemy endpoint testing complete!" -ForegroundColor Green
