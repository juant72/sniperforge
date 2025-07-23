# Script para configurar OpenSSL precompilado en Windows
# Esto evita compilar OpenSSL desde el código fuente

Write-Host "🔧 Configurando OpenSSL precompilado para desarrollo rápido..." -ForegroundColor Cyan

# Verificar si vcpkg está instalado
if (Get-Command vcpkg -ErrorAction SilentlyContinue) {
    Write-Host "✅ vcpkg encontrado, instalando OpenSSL precompilado..." -ForegroundColor Green
    vcpkg install openssl:x64-windows-static
    
    # Configurar variables de entorno
    $env:OPENSSL_DIR = "C:\vcpkg\installed\x64-windows-static"
    $env:OPENSSL_STATIC = "1"
    $env:OPENSSL_LIB_DIR = "C:\vcpkg\installed\x64-windows-static\lib"
    $env:OPENSSL_INCLUDE_DIR = "C:\vcpkg\installed\x64-windows-static\include"
    
    Write-Host "✅ OpenSSL precompilado configurado!" -ForegroundColor Green
}
else {
    Write-Host "⚠️  vcpkg no encontrado. Instalando..." -ForegroundColor Yellow
    
    # Instalar vcpkg
    if (!(Test-Path "C:\vcpkg")) {
        Write-Host "📥 Descargando vcpkg..." -ForegroundColor Blue
        git clone https://github.com/Microsoft/vcpkg.git C:\vcpkg
        Set-Location C:\vcpkg
        .\bootstrap-vcpkg.bat
        .\vcpkg integrate install
    }
    
    # Instalar OpenSSL
    Write-Host "📦 Instalando OpenSSL precompilado..." -ForegroundColor Blue
    C:\vcpkg\vcpkg install openssl:x64-windows-static
    
    # Configurar variables de entorno
    $env:OPENSSL_DIR = "C:\vcpkg\installed\x64-windows-static"
    $env:OPENSSL_STATIC = "1"
    $env:OPENSSL_LIB_DIR = "C:\vcpkg\installed\x64-windows-static\lib"
    $env:OPENSSL_INCLUDE_DIR = "C:\vcpkg\installed\x64-windows-static\include"
    
    Write-Host "✅ Configuración completa!" -ForegroundColor Green
}

# Configurar para persistir las variables
[Environment]::SetEnvironmentVariable("OPENSSL_DIR", "C:\vcpkg\installed\x64-windows-static", "User")
[Environment]::SetEnvironmentVariable("OPENSSL_STATIC", "1", "User")
[Environment]::SetEnvironmentVariable("OPENSSL_LIB_DIR", "C:\vcpkg\installed\x64-windows-static\lib", "User")
[Environment]::SetEnvironmentVariable("OPENSSL_INCLUDE_DIR", "C:\vcpkg\installed\x64-windows-static\include", "User")

Write-Host "🎉 ¡Configuración persistente guardada! Reinicia tu terminal." -ForegroundColor Green
Write-Host "⚡ La próxima compilación será mucho más rápida." -ForegroundColor Yellow
