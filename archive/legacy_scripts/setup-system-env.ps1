# Script para configurar variables de entorno a nivel SISTEMA
# ¡EJECUTAR COMO ADMINISTRADOR!

Write-Host "🔧 Configurando variables de entorno a nivel SISTEMA..." -ForegroundColor Green
Write-Host "⚠️  Este script debe ejecutarse como ADMINISTRADOR" -ForegroundColor Yellow

# Verificar si se ejecuta como administrador
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Host "❌ ERROR: Este script debe ejecutarse como Administrador" -ForegroundColor Red
    Write-Host "💡 Haz clic derecho en PowerShell → 'Ejecutar como administrador'" -ForegroundColor Yellow
    exit 1
}

Write-Host "✅ Ejecutándose como Administrador" -ForegroundColor Green

# Variables de OpenSSL
Write-Host "🔐 Configurando variables de OpenSSL..." -ForegroundColor Cyan
[Environment]::SetEnvironmentVariable("OPENSSL_DIR", "C:\vcpkg\installed\x64-windows-static", "Machine")
[Environment]::SetEnvironmentVariable("OPENSSL_LIB_DIR", "C:\vcpkg\installed\x64-windows-static\lib", "Machine")
[Environment]::SetEnvironmentVariable("OPENSSL_INCLUDE_DIR", "C:\vcpkg\installed\x64-windows-static\include", "Machine")
[Environment]::SetEnvironmentVariable("OPENSSL_STATIC", "1", "Machine")

# Variables de sccache
Write-Host "⚡ Configurando variables de sccache..." -ForegroundColor Cyan
[Environment]::SetEnvironmentVariable("RUSTC_WRAPPER", "sccache", "Machine")
[Environment]::SetEnvironmentVariable("SCCACHE_DIR", "C:\Users\juana\.sccache", "Machine")
[Environment]::SetEnvironmentVariable("SCCACHE_CACHE_SIZE", "15G", "Machine")

# Variables de vcpkg
Write-Host "🔧 Configurando variables de vcpkg..." -ForegroundColor Cyan
[Environment]::SetEnvironmentVariable("VCPKG_ROOT", "C:\vcpkg", "Machine")
[Environment]::SetEnvironmentVariable("VCPKGRS_DYNAMIC", "1", "Machine")

# Agregar vcpkg al PATH del sistema
Write-Host "🛣️ Agregando vcpkg al PATH del sistema..." -ForegroundColor Cyan
$currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($currentPath -notlike "*C:\vcpkg*") {
    $newPath = $currentPath + ";C:\vcpkg"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "Machine")
    Write-Host "✅ vcpkg agregado al PATH del sistema" -ForegroundColor Green
} else {
    Write-Host "✅ vcpkg ya está en el PATH del sistema" -ForegroundColor Green
}

Write-Host "`n🎉 ¡Variables configuradas a nivel SISTEMA!" -ForegroundColor Green
Write-Host "📋 Variables configuradas:" -ForegroundColor Blue
Write-Host "   OPENSSL_DIR = C:\vcpkg\installed\x64-windows-static"
Write-Host "   OPENSSL_LIB_DIR = C:\vcpkg\installed\x64-windows-static\lib"
Write-Host "   OPENSSL_INCLUDE_DIR = C:\vcpkg\installed\x64-windows-static\include"
Write-Host "   OPENSSL_STATIC = 1"
Write-Host "   RUSTC_WRAPPER = sccache"
Write-Host "   SCCACHE_DIR = C:\Users\juana\.sccache"
Write-Host "   SCCACHE_CACHE_SIZE = 15G"
Write-Host "   VCPKG_ROOT = C:\vcpkg"
Write-Host "   VCPKGRS_DYNAMIC = 1"
Write-Host "   PATH += C:\vcpkg"

Write-Host "`n🔄 IMPORTANTE: Cierra y abre una nueva terminal para que los cambios tomen efecto" -ForegroundColor Yellow
Write-Host "🚀 Después puedes verificar con: Get-ChildItem Env: | Where-Object Name -like '*OPENSSL*'" -ForegroundColor Cyan
