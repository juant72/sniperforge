# Script de verificación rápida
Write-Host "🔍 Variables de entorno actuales:" -ForegroundColor Cyan
Write-Host "OPENSSL_DIR: $env:OPENSSL_DIR"
Write-Host "OPENSSL_STATIC: $env:OPENSSL_STATIC"
Write-Host "RUSTC_WRAPPER: $env:RUSTC_WRAPPER"
Write-Host "SCCACHE_DIR: $env:SCCACHE_DIR"
Write-Host "VCPKG_ROOT: $env:VCPKG_ROOT"

if ($env:RUSTC_WRAPPER -eq "sccache") {
    Write-Host "
📊 Estadísticas de sccache:" -ForegroundColor Blue
    sccache --show-stats
}
