#!/usr/bin/env pwsh
# Test script to reproduce the wallet draining bug

Write-Host "🚨 TESTING JUPITER AMOUNT BUG" -ForegroundColor Red
Write-Host "==============================" -ForegroundColor Red
Write-Host ""

Write-Host "This script will test the Jupiter amount mismatch bug on DevNet" -ForegroundColor Yellow
Write-Host "We will request a small amount but check if Jupiter tries to swap more" -ForegroundColor Yellow
Write-Host ""

# Test 1: Very small amount (should be safe)
Write-Host "📝 Test 1: Requesting 0.000001 SOL swap (1000 lamports)" -ForegroundColor Cyan
Write-Host "Command: cargo run --bin sniperforge -- test swap-real --wallet test-wallet-new.json --network devnet --amount 0.000001 --confirm" -ForegroundColor Gray
Write-Host ""
Write-Host "🔍 Watch the logs for:"
Write-Host "  - 'DEBUG: Jupiter quote request parameters'"
Write-Host "  - 'WARNING: Jupiter returned different amount'"
Write-Host "  - 'CRITICAL BUG DETECTED: Jupiter trying to swap MORE'"
Write-Host ""

# Test 2: Normal small amount
Write-Host "📝 Test 2: Requesting 0.001 SOL swap (normal test amount)" -ForegroundColor Cyan
Write-Host "Command: cargo run --bin sniperforge -- test swap-real --wallet test-wallet-new.json --network devnet --amount 0.001 --confirm" -ForegroundColor Gray
Write-Host ""

Write-Host "⚠️  IMPORTANT: These tests should be BLOCKED by our new safety checks" -ForegroundColor Red
Write-Host "If Jupiter tries to swap more than requested, the transaction will be aborted" -ForegroundColor Red
Write-Host ""

Write-Host "🔗 Check wallet on DevNet explorer:"
Write-Host "https://explorer.solana.com/address/YOUR_WALLET_ADDRESS?cluster=devnet" -ForegroundColor Blue
Write-Host ""

Write-Host "📊 Expected behavior:"
Write-Host "  ✅ Transaction should be BLOCKED if Jupiter returns different amount"
Write-Host "  ✅ Clear error messages about amount mismatch"
Write-Host "  ✅ No funds should be lost"
Write-Host ""
Write-Host "❌ If funds are still being drained, there's another bug we need to find"
Write-Host ""
