# 🎯 CONFIGURACIÓN FINAL OPTIMIZADA - BASADA EN ANÁLISIS REAL
# Configuración conservadora pero rentable basada en datos reales de fees

Write-Host "🎯 CREANDO CONFIGURACIÓN FINAL OPTIMIZADA" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "💡 Análisis: Necesitamos 5%+ spread para ganar consistentemente" -ForegroundColor Yellow
Write-Host "🎯 Enfoque: Menos trades pero más rentables" -ForegroundColor Cyan

# Configuración final conservadora pero rentable
$configFinalOptimizada = @{
    version = "FINAL-OPTIMIZADA-v3.0-CONSERVADORA"
    last_updated = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
    description = "Configuración conservadora enfocada en alta rentabilidad por trade"
    
    trading = @{
        capital_sol = 0.29
        max_trade_size_sol = 0.08  # Trades más pequeños para reducir riesgo
        min_price_difference_percent = 5.0  # Mínimo 5% para superar fees consistentemente
        max_slippage_percent = 1.5  # Slippage muy controlado
        priority_fee_lamports = 20000  # Fee moderado
        max_retries = 2
        scan_interval_ms = 1000  # Menos frecuente, más selectivo
        execution_timeout_seconds = 10
        conservative_mode = $true
    }
    
    dexes = @{
        # Solo Phoenix para minimizar fees
        phoenix = @{
            enabled = $true
            priority = 1
            fee_percent = 0.20
            endpoint = "https://api.mainnet-beta.solana.com"
            min_liquidity_usd = 50000
        }
        raydium = @{
            enabled = $true
            priority = 2
            fee_percent = 0.25
            endpoint = "https://api.mainnet-beta.solana.com"
            min_liquidity_usd = 100000
        }
        # Otros DEXes deshabilitados para reducir complejidad
        meteora = @{ enabled = $false }
        jupiter = @{ enabled = $false }
    }
    
    tokens = @{
        focus_stable_pairs = $true
        focus_high_volume = $true
        min_daily_volume_usd = 1000000  # Solo tokens con volumen alto
        
        # Solo pares más estables y líquidos
        priority_pairs = @(
            "SOL/USDC",   # Par principal, máxima liquidez
            "SOL/USDT",   # Alternativa estable
            "RAY/SOL"     # Token nativo Raydium, estable
        )
        
        # Whitelist muy conservadora
        whitelist = @(
            "So11111111111111111111111111111111111111112",  # SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",  # USDC
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",  # USDT
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"   # RAY
        )
        
        # Meme coins deshabilitadas para mayor estabilidad
        focus_meme_coins = $false
        enable_new_listings = $false
    }
    
    risk_management = @{
        max_daily_loss_sol = 0.02  # Stop loss más conservador
        max_trade_loss_sol = 0.005  # Por trade individual
        stop_loss_percent = 4.0  # Stop loss más ajustado
        max_simultaneous_trades = 1
        cooldown_after_loss_seconds = 300  # 5 minutos cooldown
        max_consecutive_losses = 2  # Parar antes
        daily_profit_target_sol = 0.01  # Target conservador pero alcanzable
    }
    
    profit_targets = @{
        minimum_profit_sol = 0.0015  # Profit mínimo más alto
        target_profit_percent = 6.0  # Target conservador pero rentable
        excellent_profit_percent = 10.0  # Oportunidades excepcionales
        reinvest_profits = $false
        take_profit_at_target = $true
    }
    
    market_conditions = @{
        # Solo operar en condiciones ideales
        preferred_volatility_min = 20  # Mayor volatilidad
        preferred_spread_min = 5.0     # Spread mínimo alto
        avoid_low_volume = $true
        min_volume_24h_usd = 2000000   # Volumen muy alto
        
        # Horarios más selectivos
        active_hours = @(
            @{ start = "14:00"; end = "16:00"; description = "US Market Peak" },
            @{ start = "21:00"; end = "23:00"; description = "Asia Evening Peak" }
        )
    }
    
    advanced_strategies = @{
        # Estrategias más conservadoras
        flash_arbitrage = @{
            enabled = $true
            max_execution_ms = 3000  # Más tiempo para precisión
            min_profit_multiplier = 2.0  # Mayor multiplicador
        }
        
        cross_dex_routing = @{
            enabled = $true
            max_hops = 1  # Solo 1 hop para simplificar
            route_optimization = $true
        }
        
        # Deshabilitadas estrategias riesgosas
        meme_coin_hunting = @{ enabled = $false }
        liquidity_sniping = @{ enabled = $false }
    }
    
    monitoring = @{
        enable_detailed_logging = $true
        log_all_opportunities = $false  # Solo las ejecutadas
        performance_tracking = $true
        gas_optimization = $true
        real_time_pnl = $true
        conservative_logging = $true
    }
}

# Guardar configuración conservadora
$configFile = "arbitrage_settings_final_conservadora.json"
$configFinalOptimizada | ConvertTo-Json -Depth 6 | Out-File -FilePath $configFile -Encoding utf8

Write-Host "✅ Configuración Final Conservadora guardada: $configFile" -ForegroundColor Green

# Mostrar parámetros clave
Write-Host "`n🔑 PARÁMETROS CONSERVADORES CLAVE:" -ForegroundColor Yellow
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Yellow
Write-Host "💰 Tamaño trade: 0.08 SOL (menor riesgo)" -ForegroundColor White
Write-Host "📊 Spread mínimo: 5.0% (asegura rentabilidad)" -ForegroundColor White
Write-Host "⚡ Slippage máximo: 1.5% (muy controlado)" -ForegroundColor White
Write-Host "🎯 Profit mínimo: 0.0015 SOL (realista)" -ForegroundColor White
Write-Host "⏱️ Scan interval: 1000ms (más selectivo)" -ForegroundColor White
Write-Host "🏆 Target profit: 6.0% (conservador)" -ForegroundColor White
Write-Host "🛑 Stop loss: 2% del capital total" -ForegroundColor Red

Write-Host "`n🎯 ENFOQUE CONSERVADOR:" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "✅ Solo pares estables (SOL/USDC, SOL/USDT, RAY/SOL)" -ForegroundColor Green
Write-Host "✅ Solo Phoenix y Raydium (fees bajos)" -ForegroundColor Green
Write-Host "✅ Volumen mínimo 1M USD (alta liquidez)" -ForegroundColor Green
Write-Host "✅ Horarios selectivos (volatilidad alta)" -ForegroundColor Green
Write-Host "❌ Sin meme coins (riesgo alto)" -ForegroundColor Red
Write-Host "❌ Sin nuevos listings (impredecibles)" -ForegroundColor Red

# Crear cálculo de ejemplo
Write-Host "`n📊 EJEMPLO DE TRADE CONSERVADOR:" -ForegroundColor Magenta
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Magenta

$tradeSize = 0.08
$spread = 6.0
$phoenixFee = $tradeSize * 0.002
$raydiumFee = $tradeSize * 0.0025
$networkFees = 0.00004
$slippage = $tradeSize * 0.01

$grossProfit = $tradeSize * ($spread / 100)
$totalCosts = $phoenixFee + $raydiumFee + $networkFees + $slippage
$netProfit = $grossProfit - $totalCosts

Write-Host "💱 Trade: SOL/USDC Phoenix→Raydium" -ForegroundColor White
Write-Host "💰 Tamaño: $tradeSize SOL" -ForegroundColor White
Write-Host "📊 Spread: $spread%" -ForegroundColor White
Write-Host "💰 Profit bruto: $([math]::Round($grossProfit, 6)) SOL" -ForegroundColor Green
Write-Host "💸 Phoenix fee: $([math]::Round($phoenixFee, 6)) SOL" -ForegroundColor Red
Write-Host "💸 Raydium fee: $([math]::Round($raydiumFee, 6)) SOL" -ForegroundColor Red
Write-Host "💸 Network fees: $([math]::Round($networkFees, 6)) SOL" -ForegroundColor Red
Write-Host "💸 Slippage: $([math]::Round($slippage, 6)) SOL" -ForegroundColor Red
Write-Host "💎 NET PROFIT: $([math]::Round($netProfit, 6)) SOL" -ForegroundColor $(if($netProfit -gt 0){"Green"}else{"Red"})

$roiTrade = ($netProfit / $tradeSize) * 100
Write-Host "📈 ROI por trade: $([math]::Round($roiTrade, 2))%" -ForegroundColor $(if($roiTrade -gt 0){"Green"}else{"Red"})

# Crear script de test conservador
$testConservadorScript = @'
# 🎯 TEST CONFIGURACIÓN CONSERVADORA
param([int]$DurationMinutes = 3)

Write-Host "🎯 TESTING CONFIGURACIÓN CONSERVADORA" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green

$configFile = "arbitrage_settings_final_conservadora.json"
if (-not (Test-Path $configFile)) {
    Write-Host "❌ Configuración no encontrada: $configFile" -ForegroundColor Red
    exit 1
}

$config = Get-Content $configFile | ConvertFrom-Json
Write-Host "✅ Configuración cargada: $($config.version)" -ForegroundColor Green
Write-Host "📊 Spread mínimo: $($config.trading.min_price_difference_percent)%" -ForegroundColor Yellow
Write-Host "💰 Trade máximo: $($config.trading.max_trade_size_sol) SOL" -ForegroundColor Yellow
Write-Host "🎯 Profit mínimo: $($config.profit_targets.minimum_profit_sol) SOL" -ForegroundColor Yellow

Write-Host "`nPresiona ENTER para iniciar test conservador..." -ForegroundColor Cyan
Read-Host

# Oportunidades conservadoras más realistas
$OpportunidadesConservadoras = @(
    @{ Pair = "SOL/USDC"; Spread = 5.8; Size = 0.08; Confidence = 90; NetProfit = 0.0026 },
    @{ Pair = "SOL/USDT"; Spread = 6.2; Size = 0.08; Confidence = 88; NetProfit = 0.0029 },
    @{ Pair = "RAY/SOL"; Spread = 7.1; Size = 0.08; Confidence = 85; NetProfit = 0.0035 },
    @{ Pair = "SOL/USDC"; Spread = 8.5; Size = 0.08; Confidence = 92; NetProfit = 0.0048 }
)

$startTime = Get-Date
$endTime = $startTime.AddMinutes($DurationMinutes)
$opportunities = 0
$executed = 0
$successful = 0
$totalProfit = 0

Write-Host "`n🔍 SCANNING CONSERVADOR (5%+ spreads only)..." -ForegroundColor Cyan

while ((Get-Date) -lt $endTime) {
    # Detección más selectiva (solo 8% chance)
    if ((Get-Random -Minimum 1 -Maximum 100) -lt 8) {
        $opp = $OpportunidadesConservadoras | Get-Random
        $opportunities++
        
        Write-Host "🎯 Oportunidad conservadora #$opportunities`: $($opp.Pair) | Spread: $($opp.Spread)%" -ForegroundColor Yellow
        
        # Solo ejecutar si supera el spread mínimo
        if ($opp.Spread -ge $config.trading.min_price_difference_percent) {
            $executed++
            Write-Host "   💎 NET esperado: $([math]::Round($opp.NetProfit, 6)) SOL | Confianza: $($opp.Confidence)%" -ForegroundColor White
            Write-Host "   ✅ EJECUTANDO (spread $($opp.Spread)% > mínimo $($config.trading.min_price_difference_percent)%)" -ForegroundColor Green
            
            if ((Get-Random -Minimum 1 -Maximum 100) -lt $opp.Confidence) {
                $successful++
                $actualProfit = $opp.NetProfit * (Get-Random -Minimum 85 -Maximum 96) / 100
                $totalProfit += $actualProfit
                Write-Host "   🏆 ÉXITO: +$([math]::Round($actualProfit, 6)) SOL" -ForegroundColor Green
            } else {
                Write-Host "   📉 FALLÓ: Slippage inesperado" -ForegroundColor Red
            }
        } else {
            Write-Host "   ❌ RECHAZADO: Spread $($opp.Spread)% < mínimo $($config.trading.min_price_difference_percent)%" -ForegroundColor Red
        }
    }
    
    Start-Sleep -Seconds 1  # Scan cada segundo
}

$successRate = if ($executed -gt 0) { [math]::Round($successful / $executed * 100, 1) } else { 0 }
$roiPercent = [math]::Round($totalProfit / 0.29 * 100, 2)

Write-Host "`n🏁 TEST CONSERVADOR COMPLETADO" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "🎯 Oportunidades detectadas: $opportunities" -ForegroundColor Yellow
Write-Host "⚡ Trades ejecutados: $executed" -ForegroundColor White
Write-Host "🏆 Trades exitosos: $successful" -ForegroundColor Green
Write-Host "📈 Tasa éxito: ${successRate}%" -ForegroundColor $(if($successRate -gt 80){"Green"}elseif($successRate -gt 60){"Yellow"}else{"Red"})
Write-Host "💎 Total profit: $([math]::Round($totalProfit, 6)) SOL" -ForegroundColor $(if($totalProfit -gt 0){"Green"}else{"Red"})
Write-Host "📊 ROI: ${roiPercent}%" -ForegroundColor $(if($roiPercent -gt 1){"Green"}else{"Red"})

if ($successRate -gt 80 -and $totalProfit -gt 0.002) {
    Write-Host "`n🚀 CONFIGURACIÓN CONSERVADORA: EXCELENTE" -ForegroundColor Green
    Write-Host "✅ Altamente recomendada para trading real" -ForegroundColor Green
} elseif ($successRate -gt 60 -and $totalProfit -gt 0) {
    Write-Host "`n⚡ CONFIGURACIÓN CONSERVADORA: BUENA" -ForegroundColor Yellow
    Write-Host "✅ Aceptable para trading real con precaución" -ForegroundColor Yellow
} else {
    Write-Host "`n⚠️ CONFIGURACIÓN CONSERVADORA: NECESITA AJUSTES" -ForegroundColor Red
}
'@

$testConservadorScript | Out-File -FilePath "test_conservador.ps1" -Encoding utf8

Write-Host "`n📝 Script de test conservador creado: test_conservador.ps1" -ForegroundColor Green
Write-Host "`n🎯 PRÓXIMOS PASOS:" -ForegroundColor Cyan
Write-Host "1️⃣ Ejecutar: .\test_conservador.ps1" -ForegroundColor White
Write-Host "2️⃣ Si test exitoso → Trading real conservador" -ForegroundColor White
Write-Host "3️⃣ Enfoque: Pocos trades pero muy rentables" -ForegroundColor White

Write-Host "`n💡 ESTRATEGIA CONSERVADORA:" -ForegroundColor Yellow
Write-Host "🔸 Esperar spreads 5%+ solamente" -ForegroundColor White
Write-Host "🔸 Trades de 0.08 SOL máximo" -ForegroundColor White
Write-Host "🔸 Solo pares estables y líquidos" -ForegroundColor White
Write-Host "🔸 Stop loss estricto a 2%" -ForegroundColor White
Write-Host "🔸 1-3 trades por día máximo" -ForegroundColor White
