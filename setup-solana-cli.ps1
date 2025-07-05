# Install Solana CLI for Windows
# This script installs the Solana CLI tools

Write-Host "🚀 Installing Solana CLI for Windows" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan
Write-Host ""

# Check if already installed
$existingSolana = Get-Command solana -ErrorAction SilentlyContinue
if ($existingSolana) {
    Write-Host "✅ Solana CLI is already installed at: $($existingSolana.Source)" -ForegroundColor Green
    solana --version
    exit 0
}

Write-Host "📥 Downloading Solana CLI installer..." -ForegroundColor Yellow

# Create temp directory
$tempDir = Join-Path $env:TEMP "solana-install"
if (!(Test-Path $tempDir)) {
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
}

# Download the installer
$installerUrl = "https://release.solana.com/v1.18.25/solana-install-init-x86_64-pc-windows-msvc.exe"
$installerPath = Join-Path $tempDir "solana-install-init.exe"

try {
    Write-Host "Downloading from: $installerUrl" -ForegroundColor Gray
    Invoke-WebRequest -Uri $installerUrl -OutFile $installerPath -UseBasicParsing
    
    Write-Host "✅ Download completed" -ForegroundColor Green
    
    Write-Host "🔧 Running installer..." -ForegroundColor Yellow
    Write-Host "This will install Solana to: $env:USERPROFILE\.local\share\solana\install\active_release\bin" -ForegroundColor Gray
    
    # Run the installer
    Start-Process -FilePath $installerPath -Wait -NoNewWindow
    
    # Add to PATH
    $solanaPath = "$env:USERPROFILE\.local\share\solana\install\active_release\bin"
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    
    if ($currentPath -notlike "*$solanaPath*") {
        Write-Host "📁 Adding Solana to PATH..." -ForegroundColor Yellow
        $newPath = "$currentPath;$solanaPath"
        [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
        $env:PATH = "$env:PATH;$solanaPath"
        Write-Host "✅ Added to PATH" -ForegroundColor Green
    }
    
    # Test installation
    Write-Host "🧪 Testing installation..." -ForegroundColor Yellow
    try {
        $version = & "$solanaPath\solana.exe" --version
        Write-Host "✅ Solana CLI installed successfully!" -ForegroundColor Green
        Write-Host "Version: $version" -ForegroundColor Cyan
        
        Write-Host "`n🔧 Basic configuration..." -ForegroundColor Yellow
        & "$solanaPath\solana.exe" config set --url https://api.devnet.solana.com
        & "$solanaPath\solana.exe" config get
        
    } catch {
        Write-Host "❌ Installation test failed: $($_.Exception.Message)" -ForegroundColor Red
    }
    
} catch {
    Write-Host "❌ Download failed: $($_.Exception.Message)" -ForegroundColor Red
} finally {
    # Cleanup
    if (Test-Path $installerPath) {
        Remove-Item $installerPath -Force
    }
}

Write-Host "`n📝 Next steps:" -ForegroundColor Yellow
Write-Host "1. Close this terminal and open a new one" -ForegroundColor White
Write-Host "2. Run: solana --version" -ForegroundColor White
Write-Host "3. Run: solana config get" -ForegroundColor White
Write-Host "4. Generate a new keypair: solana-keygen new" -ForegroundColor White

Write-Host "`n✨ Solana CLI installation complete!" -ForegroundColor Green
