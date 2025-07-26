#!/usr/bin/env pwsh
# ========================================
# SNIPERFORGE REAL TRADING LAUNCHER
# ========================================
# Professional Arbitrage Trading System
# Corporate Grade - Real Transactions Enabled
# ========================================

param(
    [string]$Mode = "conservative",
    [decimal]$MaxTradeSol = 0.005,
    [int]$MinProfitBps = 10,
    [string]$LogLevel = "info"
)

# Colors for output
$Red = "`e[31m"
$Green = "`e[32m"
$Yellow = "`e[33m"
$Blue = "`e[34m"
$Magenta = "`e[35m"
$Cyan = "`e[36m"
$White = "`e[37m"
$Reset = "`e[0m"

function Write-Header {
    Clear-Host
    Write-Host "${Cyan}╔══════════════════════════════════════════════════════════════╗${Reset}"
    Write-Host "${Cyan}║                  ${White}SNIPERFORGE REAL TRADING${Cyan}                    ║${Reset}"
    Write-Host "${Cyan}║                    ${Yellow}CORPORATE GRADE SYSTEM${Cyan}                     ║${Reset}"
    Write-Host "${Cyan}╚══════════════════════════════════════════════════════════════╝${Reset}"
    Write-Host ""
}

function Write-Status {
    param([string]$Message, [string]$Color = $White)
    Write-Host "${Color}[$(Get-Date -Format 'HH:mm:ss')] $Message${Reset}"
}

function Test-Prerequisites {
    Write-Status "🔍 Verificando prerrequisitos del sistema..." $Blue
    
    # Check if we're in the right directory
    if (!(Test-Path "Cargo.toml")) {
        Write-Status "❌ ERROR: No se encuentra Cargo.toml. Ejecute desde el directorio del proyecto." $Red
        exit 1
    }
    
    # Check if binary exists or needs compilation
    if (!(Test-Path "target/release/arbitrage_phase45_clean.exe")) {
        Write-Status "⚙️ Compilando sistema optimizado..." $Yellow
        cargo build --release --bin arbitrage_phase45_clean
        if ($LASTEXITCODE -ne 0) {
            Write-Status "❌ ERROR: Fallo en compilación" $Red
            exit 1
        }
    }
    
    Write-Status "✅ Prerrequisitos verificados" $Green
}

function Set-TradingEnvironment {
    param([string]$Mode, [decimal]$MaxTradeSol, [int]$MinProfitBps, [string]$LogLevel)
    
    Write-Status "🔧 Configurando entorno de trading real..." $Blue
    
    # Set trading mode based on parameter
    switch ($Mode.ToLower()) {
        "aggressive" {
            $env:FORCE_REAL_TRANSACTIONS = "true"
            $env:MAX_TRADE_SOL = "0.01"
            $env:MIN_PROFIT_BPS = "5"
            $env:TRADING_MODE = "aggressive"
            Write-Status "⚡ MODO AGRESIVO: Trades hasta 0.01 SOL, profit mín 0.05%" $Yellow
        }
        "moderate" {
            $env:FORCE_REAL_TRANSACTIONS = "true"
            $env:MAX_TRADE_SOL = "0.005"
            $env:MIN_PROFIT_BPS = "8"
            $env:TRADING_MODE = "moderate"
            Write-Status "🎯 MODO MODERADO: Trades hasta 0.005 SOL, profit mín 0.08%" $Cyan
        }
        "conservative" {
            $env:FORCE_REAL_TRANSACTIONS = "true"
            $env:MAX_TRADE_SOL = "0.002"
            $env:MIN_PROFIT_BPS = "12"
            $env:TRADING_MODE = "conservative"
            Write-Status "🛡️ MODO CONSERVADOR: Trades hasta 0.002 SOL, profit mín 0.12%" $Green
        }
        default {
            # Custom values
            $env:FORCE_REAL_TRANSACTIONS = "true"
            $env:MAX_TRADE_SOL = $MaxTradeSol.ToString()
            $env:MIN_PROFIT_BPS = $MinProfitBps.ToString()
            $env:TRADING_MODE = "custom"
            Write-Status "⚙️ MODO CUSTOM: Trades hasta $MaxTradeSol SOL, profit mín $($MinProfitBps/100)%" $Magenta
        }
    }
    
    # Set logging level
    $env:RUST_LOG = $LogLevel
    
    # Additional safety configurations
    $env:ENABLE_MEV_PROTECTION = "true"
    $env:ENABLE_SANDWICH_DETECTION = "true"
    $env:ENABLE_SLIPPAGE_PROTECTION = "true"
    $env:MAX_SLIPPAGE_BPS = "100"  # 1% max slippage
    $env:ENABLE_PROFIT_TRACKING = "true"
    
    Write-Status "✅ Configuración aplicada correctamente" $Green
}

function Show-TradingConfig {
    Write-Host "${Yellow}╔══════════════════════════════════════════════════════════════╗${Reset}"
    Write-Host "${Yellow}║                    ${White}CONFIGURACIÓN ACTIVA${Yellow}                      ║${Reset}"
    Write-Host "${Yellow}╠══════════════════════════════════════════════════════════════╣${Reset}"
    Write-Host "${Yellow}║${Reset} Trading Real:       ${Green}$($env:FORCE_REAL_TRANSACTIONS)${Reset}"
    Write-Host "${Yellow}║${Reset} Max Trade Amount:   ${Cyan}$($env:MAX_TRADE_SOL) SOL${Reset}"
    Write-Host "${Yellow}║${Reset} Min Profit:         ${Cyan}$($env:MIN_PROFIT_BPS) BPS ($([math]::Round($env:MIN_PROFIT_BPS/100,3))%)${Reset}"
    Write-Host "${Yellow}║${Reset} Trading Mode:       ${Magenta}$($env:TRADING_MODE.ToUpper())${Reset}"
    Write-Host "${Yellow}║${Reset} Log Level:          ${White}$($env:RUST_LOG)${Reset}"
    Write-Host "${Yellow}║${Reset} MEV Protection:     ${Green}$($env:ENABLE_MEV_PROTECTION)${Reset}"
    Write-Host "${Yellow}║${Reset} Max Slippage:       ${Cyan}$($env:MAX_SLIPPAGE_BPS) BPS (1%)${Reset}"
    Write-Host "${Yellow}╚══════════════════════════════════════════════════════════════╝${Reset}"
    Write-Host ""
}

function Start-TradingSystem {
    Write-Status "🚀 INICIANDO SISTEMA DE TRADING REAL..." $Green
    Write-Status "💰 TRANSACCIONES REALES HABILITADAS" $Red
    Write-Status "⚠️  USANDO FONDOS REALES DE LA WALLET" $Red
    Write-Host ""
    
    # Final confirmation for real trading
    Write-Host "${Red}⚠️  CONFIRMACIÓN FINAL:${Reset}"
    Write-Host "${Red}   • Este sistema ejecutará trades REALES con SOL real${Reset}"
    Write-Host "${Red}   • Los fondos pueden perderse si hay errores${Reset}"
    Write-Host "${Red}   • Max amount por trade: $($env:MAX_TRADE_SOL) SOL${Reset}"
    Write-Host ""
    
    $confirmation = Read-Host "${Yellow}¿Confirma iniciar trading real? (escriba 'YES' para continuar)${Reset}"
    
    if ($confirmation -ne "YES") {
        Write-Status "❌ Trading cancelado por el usuario" $Red
        exit 0
    }
    
    Write-Status "✅ Confirmación recibida. Iniciando trading..." $Green
    Write-Host ""
    
    # Start the arbitrage system
    try {
        Write-Status "🔥 SISTEMA DE ARBITRAJE ACTIVADO" $Green
        Write-Status "📊 Monitoring dashboard iniciado" $Blue
        Write-Status "💹 Buscando oportunidades reales..." $Cyan
        Write-Host ""
        
        # Execute the arbitrage system
        cargo run --release --bin arbitrage_phase45_clean
        
    } catch {
        Write-Status "❌ ERROR durante ejecución: $($_.Exception.Message)" $Red
        exit 1
    }
}

function Show-Usage {
    Write-Host "${Cyan}USAGE:${Reset}"
    Write-Host "  ${White}.\start_real_trading.ps1${Reset} [options]"
    Write-Host ""
    Write-Host "${Cyan}MODOS PREDEFINIDOS:${Reset}"
    Write-Host "  ${Green}-Mode conservative${Reset}  # 0.002 SOL max, 0.12% profit mín (DEFAULT)"
    Write-Host "  ${Yellow}-Mode moderate${Reset}     # 0.005 SOL max, 0.08% profit mín"
    Write-Host "  ${Red}-Mode aggressive${Reset}    # 0.01 SOL max, 0.05% profit mín"
    Write-Host ""
    Write-Host "${Cyan}CONFIGURACIÓN CUSTOM:${Reset}"
    Write-Host "  ${White}-MaxTradeSol 0.003${Reset}     # Amount máximo por trade"
    Write-Host "  ${White}-MinProfitBps 15${Reset}       # Profit mínimo en basis points"
    Write-Host "  ${White}-LogLevel debug${Reset}        # Nivel de logging (info/debug/trace)"
    Write-Host ""
    Write-Host "${Cyan}EJEMPLOS:${Reset}"
    Write-Host "  ${White}.\start_real_trading.ps1${Reset}                                    # Modo conservador"
    Write-Host "  ${White}.\start_real_trading.ps1 -Mode aggressive${Reset}                  # Modo agresivo"
    Write-Host "  ${White}.\start_real_trading.ps1 -MaxTradeSol 0.008 -MinProfitBps 6${Reset}  # Custom"
}

# ========================================
# MAIN EXECUTION
# ========================================

# Show help if requested
if ($args -contains "-h" -or $args -contains "--help" -or $args -contains "help") {
    Show-Usage
    exit 0
}

# Main execution flow
try {
    Write-Header
    Test-Prerequisites
    Set-TradingEnvironment -Mode $Mode -MaxTradeSol $MaxTradeSol -MinProfitBps $MinProfitBps -LogLevel $LogLevel
    Show-TradingConfig
    Start-TradingSystem
    
} catch {
    Write-Status "❌ ERROR CRÍTICO: $($_.Exception.Message)" $Red
    Write-Status "🔧 Revise la configuración y vuelva a intentar" $Yellow
    exit 1
}

# Cleanup on exit
Write-Status "🛑 Sistema de trading detenido" $Yellow
Write-Status "💾 Logs guardados automáticamente" $Green
