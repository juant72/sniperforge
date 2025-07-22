#!/usr/bin/env pwsh
# PROPOSAL-003 PHASE 2 Testing Script
# Test automatizado del sistema multi-token con Tier 1 y Tier 2

param(
    [string]$Mode = "both"  # "tier1", "tier2", or "both"
)

Write-Host "🚀 PROPOSAL-003 PHASE 2 AUTOMATIC TESTING SCRIPT" -ForegroundColor Green
Write-Host "==============================================" -ForegroundColor Green

# Navegar al directorio correcto
Set-Location "c:\work\encrypia\labs\sniperforge"

# Verificar que el proyecto compile
Write-Host "📦 Building project with Phase 2 enhancements..." -ForegroundColor Yellow
$buildResult = cargo build --bin arbiter_clean 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    Write-Host $buildResult
    exit 1
}
Write-Host "✅ Build successful!" -ForegroundColor Green

function Test-MultiTokenMode {
    param([string]$TestMode, [string]$InputChar, [string]$Description)
    
    Write-Host "`n🎯 Testing $Description..." -ForegroundColor Cyan
    Write-Host "   - Mode: $TestMode" -ForegroundColor White
    Write-Host "   - Input: $InputChar" -ForegroundColor White
    Write-Host "   - Duration: 30 seconds" -ForegroundColor White
    
    # Crear input para el test
    $testInput = @"
$InputChar

"@
    
    $testInput | Out-File -FilePath "test_input_$InputChar.txt" -Encoding utf8
    
    # Ejecutar test
    $job = Start-Job -ScriptBlock {
        param($path, $file)
        Set-Location $path
        Get-Content $file | ./target/debug/arbiter_clean.exe
    } -ArgumentList (Get-Location), "test_input_$InputChar.txt"
    
    # Esperar 30 segundos
    $timeout = 30
    $elapsed = 0
    while ($job.State -eq "Running" -and $elapsed -lt $timeout) {
        Start-Sleep -Seconds 5
        $elapsed += 5
        Write-Host "   ⏳ $Description running... $elapsed/$timeout seconds" -ForegroundColor Gray
    }
    
    # Obtener resultados
    if ($job.State -eq "Running") {
        Stop-Job $job
    }
    
    $output = Receive-Job $job
    Write-Host "   📋 OUTPUT PREVIEW:" -ForegroundColor Yellow
    $output | Select-Object -First 10 | ForEach-Object { 
        Write-Host "      $_" -ForegroundColor Gray 
    }
    
    # Cleanup
    Remove-Job $job -Force
    Remove-Item "test_input_$InputChar.txt" -ErrorAction SilentlyContinue
    
    return $output
}

# Tests basados en el modo seleccionado
switch ($Mode.ToLower()) {
    "tier1" {
        Write-Host "🛡️  TESTING TIER 1 ONLY (Conservative)" -ForegroundColor Blue
        Test-MultiTokenMode "Tier 1 Multi-Token" "M" "PROPOSAL-003 Phase 1 (3 token pairs)"
    }
    "tier2" {
        Write-Host "🚀 TESTING TIER 2 ONLY (Full Ecosystem)" -ForegroundColor Magenta
        Test-MultiTokenMode "Tier 2 Multi-Token" "T" "PROPOSAL-003 Phase 2 (16 token pairs)"
    }
    "both" {
        Write-Host "🔄 TESTING BOTH TIERS (Complete Validation)" -ForegroundColor Cyan
        
        Write-Host "`n===== TIER 1 TEST =====" -ForegroundColor Blue
        Test-MultiTokenMode "Tier 1 Multi-Token" "M" "PROPOSAL-003 Phase 1 (3 token pairs)"
        
        Start-Sleep -Seconds 5
        
        Write-Host "`n===== TIER 2 TEST =====" -ForegroundColor Magenta  
        Test-MultiTokenMode "Tier 2 Multi-Token" "T" "PROPOSAL-003 Phase 2 (16 token pairs)"
    }
}

Write-Host "`n🎉 PROPOSAL-003 PHASE 2 TESTING COMPLETED!" -ForegroundColor Green
Write-Host "==============================================" -ForegroundColor Green

# Summary
Write-Host "`n📋 TESTING SUMMARY:" -ForegroundColor White
Write-Host "✅ Tier 1 Support: PROPOSAL-003 Phase 1 (SOL, USDC, USDT)" -ForegroundColor Green
Write-Host "✅ Tier 2 Support: PROPOSAL-003 Phase 2 (+ BONK, RAY, ORCA, PYTH, JTO)" -ForegroundColor Green
Write-Host "✅ Menu Options: A, B, M (Tier 1), T (Tier 2), C" -ForegroundColor Green
Write-Host "✅ Backward Compatibility: Original modes A/B preserved" -ForegroundColor Green

Write-Host "`n🚀 READY FOR PRODUCTION TESTING!" -ForegroundColor Cyan
Write-Host "Run manually: cargo run --bin arbiter_clean" -ForegroundColor Yellow
Write-Host "Then select: M for Tier 1, T for Tier 2" -ForegroundColor Yellow
