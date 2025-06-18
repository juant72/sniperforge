# Script para instalar y configurar sccache
# sccache acelera la compilación usando cache distribuido

Write-Host "⚡ Configurando sccache para compilación ultrarrápida..." -ForegroundColor Cyan

# Verificar si sccache ya está instalado
if (Get-Command sccache -ErrorAction SilentlyContinue) {
    Write-Host "✅ sccache ya está instalado" -ForegroundColor Green
} else {
    Write-Host "📥 Instalando sccache..." -ForegroundColor Blue
    cargo install sccache
}

# Configurar variables de entorno para sccache
$env:RUSTC_WRAPPER = "sccache"
$env:SCCACHE_CACHE_SIZE = "10G"
$env:SCCACHE_DIR = "$env:LOCALAPPDATA\sccache"

# Hacer persistentes las variables
[Environment]::SetEnvironmentVariable("RUSTC_WRAPPER", "sccache", "User")
[Environment]::SetEnvironmentVariable("SCCACHE_CACHE_SIZE", "10G", "User")
[Environment]::SetEnvironmentVariable("SCCACHE_DIR", "$env:LOCALAPPDATA\sccache", "User")

Write-Host "✅ sccache configurado!" -ForegroundColor Green
Write-Host "📊 Mostrando estadísticas de cache:" -ForegroundColor Blue
sccache --show-stats

Write-Host "🚀 ¡Compilación acelerada configurada!" -ForegroundColor Green
Write-Host "💡 Tip: Las próximas compilaciones serán mucho más rápidas" -ForegroundColor Yellow
