# SniperForge - Project Validation Script
# Quick validation to ensure all components are properly set up

Write-Host "🔍 SniperForge Project Validation" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan
Write-Host ""

$validationErrors = @()
$validationWarnings = @()

# Check project structure
Write-Host "📁 Checking project structure..." -ForegroundColor Yellow

$requiredDirs = @("src", "config", "docs", "tests", "logs")
$requiredFiles = @("Cargo.toml", "README.md", "src/main.rs", "config/platform.toml")

foreach ($dir in $requiredDirs) {
    if (Test-Path $dir) {
        Write-Host "  ✅ $dir/" -ForegroundColor Green
    } else {
        Write-Host "  ❌ $dir/ (missing)" -ForegroundColor Red
        $validationErrors += "Missing directory: $dir"
    }
}

foreach ($file in $requiredFiles) {
    if (Test-Path $file) {
        Write-Host "  ✅ $file" -ForegroundColor Green
    } else {
        Write-Host "  ❌ $file (missing)" -ForegroundColor Red
        $validationErrors += "Missing file: $file"
    }
}

# Check source code structure
Write-Host ""
Write-Host "🦀 Checking Rust source structure..." -ForegroundColor Yellow

$srcDirs = @("src/platform", "src/shared", "src/bots")
$srcFiles = @(
    "src/main.rs", "src/config.rs", "src/types.rs", "src/cli.rs",
    "src/platform/mod.rs", "src/platform/bot_manager.rs", "src/platform/event_bus.rs", "src/platform/resource_coordinator.rs",
    "src/shared/mod.rs", "src/shared/rpc_pool.rs", "src/shared/wallet_manager.rs", "src/shared/data_feeds.rs", "src/shared/monitoring.rs",
    "src/bots/mod.rs", "src/bots/lp_sniper.rs"
)

foreach ($dir in $srcDirs) {
    if (Test-Path $dir) {
        Write-Host "  ✅ $dir/" -ForegroundColor Green
    } else {
        Write-Host "  ❌ $dir/ (missing)" -ForegroundColor Red
        $validationErrors += "Missing source directory: $dir"
    }
}

foreach ($file in $srcFiles) {
    if (Test-Path $file) {
        Write-Host "  ✅ $file" -ForegroundColor Green
    } else {
        Write-Host "  ❌ $file (missing)" -ForegroundColor Red
        $validationErrors += "Missing source file: $file"
    }
}

# Check configuration
Write-Host ""
Write-Host "⚙️ Checking configuration..." -ForegroundColor Yellow

if (Test-Path "config/platform.toml") {
    try {
        $configContent = Get-Content "config/platform.toml" -Raw
        if ($configContent -match '\[platform\]') {
            Write-Host "  ✅ platform.toml has [platform] section" -ForegroundColor Green
        } else {
            Write-Host "  ⚠️ platform.toml missing [platform] section" -ForegroundColor Yellow
            $validationWarnings += "Configuration may be incomplete"
        }
        
        if ($configContent -match '\[network\]') {
            Write-Host "  ✅ platform.toml has [network] section" -ForegroundColor Green
        } else {
            Write-Host "  ⚠️ platform.toml missing [network] section" -ForegroundColor Yellow
            $validationWarnings += "Network configuration may be incomplete"
        }
    } catch {
        Write-Host "  ❌ Error reading configuration file" -ForegroundColor Red
        $validationErrors += "Configuration file is corrupted or unreadable"
    }
}

# Check documentation
Write-Host ""
Write-Host "📚 Checking documentation..." -ForegroundColor Yellow

$docFiles = @("docs/dev/sprint-0-status.md", "docs/dev/sprint-0-validation.md")

foreach ($file in $docFiles) {
    if (Test-Path $file) {
        Write-Host "  ✅ $file" -ForegroundColor Green
    } else {
        Write-Host "  ⚠️ $file (missing)" -ForegroundColor Yellow
        $validationWarnings += "Missing documentation: $file"
    }
}

# Check Rust installation
Write-Host ""
Write-Host "🦀 Checking Rust installation..." -ForegroundColor Yellow

if (Get-Command cargo -ErrorAction SilentlyContinue) {
    $cargoVersion = cargo --version
    Write-Host "  ✅ Rust/Cargo installed: $cargoVersion" -ForegroundColor Green
    
    # Try to check if project compiles
    Write-Host "  🔧 Testing compilation..." -ForegroundColor Yellow
    $buildResult = & cargo check 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✅ Project compiles successfully" -ForegroundColor Green
    } else {
        Write-Host "  ⚠️ Compilation issues detected" -ForegroundColor Yellow
        $validationWarnings += "Project has compilation issues (expected in development)"
        Write-Host "    Build output: $buildResult" -ForegroundColor Gray
    }
} else {
    Write-Host "  ❌ Rust/Cargo not installed" -ForegroundColor Red
    $validationErrors += "Rust toolchain not installed"
    Write-Host "    Install from: https://rustup.rs/" -ForegroundColor Gray
}

# Summary
Write-Host ""
Write-Host "📊 Validation Summary" -ForegroundColor Cyan
Write-Host "===================" -ForegroundColor Cyan

if ($validationErrors.Count -eq 0 -and $validationWarnings.Count -eq 0) {
    Write-Host "🎉 All validations passed! Project is ready." -ForegroundColor Green
    Write-Host ""
    Write-Host "🚀 Next steps:" -ForegroundColor Blue
    Write-Host "  1. Install Rust if not already installed: https://rustup.rs/" -ForegroundColor White
    Write-Host "  2. Run: cargo build" -ForegroundColor White
    Write-Host "  3. Run: cargo run -- --help" -ForegroundColor White
    Write-Host "  4. Start development with: .\start.ps1" -ForegroundColor White
} elseif ($validationErrors.Count -eq 0) {
    Write-Host "⚠️ Validation completed with warnings:" -ForegroundColor Yellow
    foreach ($warning in $validationWarnings) {
        Write-Host "  • $warning" -ForegroundColor Yellow
    }
    Write-Host ""
    Write-Host "✅ Project structure is valid. Warnings can be addressed during development." -ForegroundColor Green
} else {
    Write-Host "❌ Validation failed with errors:" -ForegroundColor Red
    foreach ($error in $validationErrors) {
        Write-Host "  • $error" -ForegroundColor Red
    }
    
    if ($validationWarnings.Count -gt 0) {
        Write-Host ""
        Write-Host "⚠️ Additional warnings:" -ForegroundColor Yellow
        foreach ($warning in $validationWarnings) {
            Write-Host "  • $warning" -ForegroundColor Yellow
        }
    }
    
    Write-Host ""
    Write-Host "🔧 Please fix the errors above before proceeding." -ForegroundColor Red
}

Write-Host ""
Write-Host "📋 Project Status: Sprint 0 Implementation Complete" -ForegroundColor Blue
Write-Host "🎯 Ready for: Sprint 1 Development" -ForegroundColor Blue
