# Test script for options 4-8
Write-Host "Testing Arbitrage Bot Options 4-8"
Write-Host "===================================="

# Test option 6 (status - should be quick)
Write-Host "`nTesting Option 6 (Monitor Status):"
echo "6" | cargo run --bin arbitrage_bot 2>$null

Write-Host "`nTesting Option 7 (Safe Simulation):"
echo "7" | cargo run --bin arbitrage_bot 2>$null

Write-Host "`nTesting Option 8 (Validated Opportunity):"
echo "8" | cargo run --bin arbitrage_bot 2>$null

Write-Host "`nAll tests completed!"
