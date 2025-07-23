# 🔧 Script para configurar variables de entorno PERMANENTES
# Configura OpenSSL y sccache de forma persistente en Windows

Write-Host "🔧 Configurando variables de entorno PERMANENTES..." -ForegroundColor Green
Write-Host "=================================================" -ForegroundColor Green

# Función para configurar variable persistente
function Set-PermanentEnvVar {
    param(
        [string]$Name,
        [string]$Value,
        [string]$Target = "User"
    )
    
    Write-Host "Setting $Name = $Value" -ForegroundColor Cyan
    [Environment]::SetEnvironmentVariable($Name, $Value, $Target)
    
    # También setear para la sesión actual
    Set-Item -Path "env:$Name" -Value $Value
}

# Verificar vcpkg y OpenSSL
$vcpkgPath = "C:\vcpkg"
if (!(Test-Path $vcpkgPath)) {
    $vcpkgPath = "C:\tools\vcpkg"
}

if (!(Test-Path $vcpkgPath)) {
    Write-Host "❌ vcpkg no encontrado. Instalando..." -ForegroundColor Yellow
    
    # Crear directorio y clonar vcpkg
    $vcpkgPath = "C:\tools\vcpkg"
    New-Item -ItemType Directory -Path "C:\tools" -Force | Out-Null
    
    git clone https://github.com/Microsoft/vcpkg.git $vcpkgPath
    Set-Location $vcpkgPath
    .\bootstrap-vcpkg.bat
    .\vcpkg integrate install
    
    Write-Host "✅ vcpkg instalado en $vcpkgPath" -ForegroundColor Green
}

# Instalar OpenSSL si no existe
$opensslDir = "$vcpkgPath\installed\x64-windows-static"
if (!(Test-Path "$opensslDir\lib\libssl.lib")) {
    Write-Host "📦 Instalando OpenSSL precompilado..." -ForegroundColor Blue
    & "$vcpkgPath\vcpkg.exe" install openssl:x64-windows-static
}

# 1. CONFIGURAR VARIABLES DE OPENSSL PERMANENTES
Write-Host "`n🔐 Configurando OpenSSL permanentemente..." -ForegroundColor Blue
Set-PermanentEnvVar -Name "OPENSSL_DIR" -Value $opensslDir
Set-PermanentEnvVar -Name "OPENSSL_LIB_DIR" -Value "$opensslDir\lib"
Set-PermanentEnvVar -Name "OPENSSL_INCLUDE_DIR" -Value "$opensslDir\include"
Set-PermanentEnvVar -Name "OPENSSL_STATIC" -Value "1"
Set-PermanentEnvVar -Name "VCPKG_ROOT" -Value $vcpkgPath

# 2. CONFIGURAR SCCACHE PERMANENTE
Write-Host "`n⚡ Configurando sccache permanentemente..." -ForegroundColor Blue

# Verificar si sccache está instalado
if (!(Get-Command sccache -ErrorAction SilentlyContinue)) {
    Write-Host "📦 Instalando sccache..." -ForegroundColor Yellow
    cargo install sccache
}

Set-PermanentEnvVar -Name "RUSTC_WRAPPER" -Value "sccache"
Set-PermanentEnvVar -Name "SCCACHE_DIR" -Value "$env:USERPROFILE\.sccache"
Set-PermanentEnvVar -Name "SCCACHE_CACHE_SIZE" -Value "15G"

# 3. AGREGAR VCPKG AL PATH PERMANENTE
Write-Host "`n📂 Configurando PATH permanente..." -ForegroundColor Blue
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($currentPath -notlike "*$vcpkgPath*") {
    $newPath = "$currentPath;$vcpkgPath"
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Host "✅ vcpkg agregado al PATH" -ForegroundColor Green
}

# 4. VERIFICAR CONFIGURACIÓN
Write-Host "`n✅ Verificando configuración..." -ForegroundColor Green
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Green

# Función para verificar variable
function Test-EnvVar {
    param([string]$Name)
    
    $userValue = [Environment]::GetEnvironmentVariable($Name, "User")
    $currentValue = [Environment]::GetEnvironmentVariable($Name, "Process")
    
    if ($userValue) {
        Write-Host "✅ $Name (Permanente): $userValue" -ForegroundColor Green
        if ($currentValue -ne $userValue) {
            Write-Host "⚠️  $Name (Sesión actual): $currentValue" -ForegroundColor Yellow
            Write-Host "   (Se aplicará en nueva terminal)" -ForegroundColor Gray
        }
    } else {
        Write-Host "❌ ${Name}: NO CONFIGURADA" -ForegroundColor Red
    }
}

Test-EnvVar "OPENSSL_DIR"
Test-EnvVar "OPENSSL_STATIC"
Test-EnvVar "RUSTC_WRAPPER"
Test-EnvVar "SCCACHE_DIR"
Test-EnvVar "VCPKG_ROOT"

# 5. CREAR SCRIPT DE VERIFICACIÓN RÁPIDA
$verifyScript = @"
# Script de verificación rápida
Write-Host "🔍 Variables de entorno actuales:" -ForegroundColor Cyan
Write-Host "OPENSSL_DIR: `$env:OPENSSL_DIR"
Write-Host "OPENSSL_STATIC: `$env:OPENSSL_STATIC"
Write-Host "RUSTC_WRAPPER: `$env:RUSTC_WRAPPER"
Write-Host "SCCACHE_DIR: `$env:SCCACHE_DIR"
Write-Host "VCPKG_ROOT: `$env:VCPKG_ROOT"

if (`$env:RUSTC_WRAPPER -eq "sccache") {
    Write-Host "`n📊 Estadísticas de sccache:" -ForegroundColor Blue
    sccache --show-stats
}
"@

$verifyScript | Out-File -FilePath "verify-env.ps1" -Encoding utf8

Write-Host "`n🎉 ¡Configuración permanente completada!" -ForegroundColor Magenta
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Magenta
Write-Host "📝 Se creó 'verify-env.ps1' para verificación rápida" -ForegroundColor Cyan
Write-Host "`n🔄 Para que los cambios tomen efecto:" -ForegroundColor Yellow
Write-Host "   1. Cierra esta terminal" -ForegroundColor White
Write-Host "   2. Abre una nueva terminal" -ForegroundColor White
Write-Host "   3. Ejecuta: .\verify-env.ps1" -ForegroundColor White
Write-Host "   4. Ejecuta: cargo clean && cargo build" -ForegroundColor White

Write-Host "`n🚀 ¡Los builds serán mucho más rápidos!" -ForegroundColor Green
