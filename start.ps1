# SniperForge Development Startup Script (PowerShell)
# This script sets up the development environment and starts the platform

param(
    [string]$Mode = "interactive"
)

Write-Host "🚀 SniperForge Development Startup" -ForegroundColor Blue
Write-Host "==================================" -ForegroundColor Blue

# Function to print colored output
function Write-Status {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Check if Rust is installed
function Test-Rust {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Error "Rust/Cargo not found. Please install Rust from https://rustup.rs/"
        exit 1
    }
    
    $cargoVersion = cargo --version
    Write-Success "Rust/Cargo found: $cargoVersion"
}

# Check if required directories exist
function Test-Directories {
    Write-Status "Checking directory structure..."
    
    $directories = @("config", "logs", "tests")
    
    foreach ($dir in $directories) {
        if (-not (Test-Path $dir)) {
            Write-Status "Creating directory: $dir"
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
        }
    }
    
    Write-Success "Directory structure ready"
}

# Build the project
function Build-Project {
    Write-Status "Building SniperForge..."
    
    $buildResult = & cargo build
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Build completed successfully"
    } else {
        Write-Error "Build failed"
        exit 1
    }
}

# Run tests
function Invoke-Tests {
    Write-Status "Running tests..."
    
    $testResult = & cargo test --lib
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Tests passed"
    } else {
        Write-Warning "Some tests failed (this is expected in development)"
    }
}

# Start the platform
function Start-Platform {
    Write-Status "Starting SniperForge Platform..."
    
    # Check if config file exists
    if (-not (Test-Path "config/platform.toml")) {
        Write-Warning "Config file not found, using default configuration"
    }
    
    Write-Status "Available startup options:"
    Write-Host "  1) Start full platform"
    Write-Host "  2) Start with LP Sniper bot only" 
    Write-Host "  3) Interactive mode"
    Write-Host "  4) System test"
    Write-Host "  5) Show status only"
    
    $choice = Read-Host "Choose option (1-5)"
    
    switch ($choice) {
        "1" {
            Write-Status "Starting full platform..."
            & cargo run -- start
        }
        "2" {
            Write-Status "Starting platform with LP Sniper bot..."
            & cargo run -- start-bots lp-sniper
        }
        "3" {
            Write-Status "Starting interactive mode..."
            & cargo run -- interactive
        }
        "4" {
            Write-Status "Running system test..."
            & cargo run -- test-system
        }
        "5" {
            Write-Status "Showing platform status..."
            & cargo run -- status
        }
        default {
            Write-Warning "Invalid option, starting in interactive mode..."
            & cargo run -- interactive
        }
    }
}

# Main execution
function Main {
    Write-Host ""
    Write-Status "Starting SniperForge development environment setup..."
    
    Test-Rust
    Test-Directories
    Build-Project
    Invoke-Tests
    
    Write-Host ""
    Write-Success "Setup completed successfully!"
    Write-Host ""
    
    Start-Platform
}

# Handle Ctrl+C gracefully
try {
    Main
} catch {
    Write-Warning "Interrupted by user or error occurred: $_"
    exit 1
}
