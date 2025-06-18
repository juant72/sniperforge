# Script de compilación rápida para SniperForge
# Uso: .\fast-build.ps1 [check|run|test]

param(
    [string]$Command = "check",
    [string]$ExtraArgs = ""
)

Write-Host "🚀 Compilación rápida de SniperForge" -ForegroundColor Green
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Green

# Configurar variables de entorno para compilación rápida
$env:CARGO_INCREMENTAL = "1"
$env:RUSTC_WRAPPER = ""
$env:CARGO_BUILD_JOBS = [System.Environment]::ProcessorCount
$env:CARGO_HTTP_MULTIPLEXING = "false"

Write-Host "⚙️  CPU Cores: $env:CARGO_BUILD_JOBS" -ForegroundColor Yellow
Write-Host "🔧 Compilación incremental: Habilitada" -ForegroundColor Yellow

switch ($Command.ToLower()) {
    "check" {
        Write-Host "🔍 Ejecutando cargo check..." -ForegroundColor Blue
        cargo check --color=always
    }    "run" {
        Write-Host "▶️  Ejecutando cargo run..." -ForegroundColor Blue
        if ($ExtraArgs) {
            Invoke-Expression "cargo run --color=always -- $ExtraArgs"
        } else {
            cargo run --color=always
        }
    }
    "test" {
        Write-Host "🧪 Ejecutando tests..." -ForegroundColor Blue
        cargo test --color=always
    }
    "clean" {
        Write-Host "🧹 Limpiando cache..." -ForegroundColor Blue
        cargo clean
        Write-Host "✅ Cache limpiado" -ForegroundColor Green
    }
    default {
        Write-Host "❌ Comando no reconocido: $Command" -ForegroundColor Red
        Write-Host "Comandos disponibles: check, run, test, clean" -ForegroundColor Yellow
    }
}

Write-Host "✅ Operación completada" -ForegroundColor Green
