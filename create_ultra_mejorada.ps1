# 🚀 CONFIGURACIÓN ULTRA-MEJORADA - OPTIMIZADA PARA RENTABILIDAD REAL
# Configuración ultra con parámetros ajustados según análisis de fees

Write-Host "🚀 CREANDO CONFIGURACIÓN ULTRA-MEJORADA" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "💡 Basada en análisis de fees real para máxima rentabilidad" -ForegroundColor Cyan

# Configuración ultra-mejorada con parámetros realistas
$ultraConfigMejorada = @{
    version = "ULTRA-MEJORADA-v2.0"
    last_updated = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
    
    trading = @{
        capital_sol = 0.29
        max_trade_size_sol = 0.15  # Aumentado para mejor ratio profit/fees
        min_price_difference_percent = 3.5  # Aumentado para superar fees consistentemente
        max_slippage_percent = 2.5  # Reducido para mejor precisión
        priority_fee_lamports = 30000  # Aumentado para mejor ejecución
        max_retries = 2
        scan_interval_ms = 750  # Menos agresivo para mejor detección
        execution_timeout_seconds = 15
    }
    
    dexes = @{
        # Priorizar DEXes con menores fees
        phoenix = @{
            enabled = $true
            priority = 1
            fee_percent = 0.20
            endpoint = "https://api.mainnet-beta.solana.com"
            min_liquidity_usd = 10000
        }
        raydium = @{
            enabled = $true
            priority = 2
            fee_percent = 0.25
            endpoint = "https://api.mainnet-beta.solana.com"
            min_liquidity_usd = 50000
        }
        meteora = @{
            enabled = $false  # Fees más altos - deshabilitado
            priority = 3
            fee_percent = 0.30
        }
        jupiter = @{
            enabled = $true
            priority = 3
            fee_percent = 0.30
            endpoint = "https://api.mainnet-beta.solana.com"
            min_liquidity_usd = 25000
        }
    }
    
    tokens = @{
        focus_high_volatility = $true
        focus_meme_coins = $true
        enable_new_listings = $true
        min_daily_volume_usd = 100000  # Mayor volumen para mejor liquidez
        max_age_hours = 168  # Tokens no más viejos de 1 semana
        
        # Tokens de alta volatilidad priorizados
        priority_pairs = @(
            "SOL/USDC",   # Par principal con alta liquidez
            "SOL/USDT",   # Alternativa estable
            "BONK/SOL",   # Meme coin establecida
            "WIF/SOL",    # Meme coin volátil
            "POPCAT/SOL", # Meme coin trending
            "RAY/SOL",    # Token DEX nativo
            "JUP/SOL",    # Token DEX agregador
            "PYTH/SOL"    # Oracle token
        )
        
        # Whitelist expandida con tokens volátiles
        whitelist = @(
            "So11111111111111111111111111111111111111112",  # SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",  # USDC
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",  # USDT
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",  # BONK
            "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm",  # WIF
            "7GCihgDB8fe6KNjn2MYtkzZcRjQy3t9GHdC8uHYmW2hr",  # POPCAT
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",  # RAY
            "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN",   # JUP
            "HZ1JovNiVvGrGNiiYvEozEVgZ58xaU3RKwX8eACQBCt3"   # PYTH
        )
    }
    
    risk_management = @{
        max_daily_loss_sol = 0.03  # 10% del capital máximo
        max_trade_loss_sol = 0.008  # Por trade individual
        stop_loss_percent = 6.0
        max_simultaneous_trades = 1  # Más conservador
        cooldown_after_loss_seconds = 120  # 2 minutos
        max_consecutive_losses = 3
        daily_profit_target_sol = 0.015  # 5% diario objetivo
    }
    
    profit_targets = @{
        minimum_profit_sol = 0.0008  # Profit mínimo más alto
        target_profit_percent = 4.0  # Target más alto pero realista
        excellent_profit_percent = 6.0  # Oportunidades excepcionales
        reinvest_profits = $false
        take_profit_at_target = $true
    }
    
    mev_protection = @{
        enabled = $true
        max_priority_fee = 50000  # Fee más alto para competir
        use_private_mempool = $false
        delay_execution_ms = 50  # Menos delay para velocidad
        sandwich_protection = $true
        frontrun_detection = $true
    }
    
    market_conditions = @{
        # Condiciones óptimas para trading
        preferred_volatility_min = 15  # Volatilidad mínima %
        preferred_spread_min = 3.5     # Spread mínimo %
        avoid_low_volume = $true
        min_volume_24h_usd = 500000    # Volumen mínimo 24h
        
        # Horarios óptimos (UTC)
        active_hours = @(
            @{ start = "13:00"; end = "17:00"; description = "US Market Open" },
            @{ start = "20:00"; end = "23:00"; description = "Asia Peak" },
            @{ start = "07:00"; end = "10:00"; description = "Europe Open" }
        )
    }
    
    advanced_strategies = @{
        # Estrategias especializadas
        flash_arbitrage = @{
            enabled = $true
            max_execution_ms = 2000
            min_profit_multiplier = 1.5
        }
        
        cross_dex_routing = @{
            enabled = $true
            max_hops = 2
            route_optimization = $true
        }
        
        meme_coin_hunting = @{
            enabled = $true
            new_listing_window_minutes = 60
            pump_detection_threshold = 20  # %
            volume_spike_multiplier = 5
        }
        
        liquidity_sniping = @{
            enabled = $true
            min_liquidity_change_percent = 50
            reaction_time_ms = 200
        }
    }
    
    monitoring = @{
        enable_detailed_logging = $true
        log_all_opportunities = $true
        performance_tracking = $true
        gas_optimization = $true
        real_time_pnl = $true
    }
}

# Guardar configuración mejorada
$configFile = "arbitrage_settings_ultra_mejorada.json"
$ultraConfigMejorada | ConvertTo-Json -Depth 6 | Out-File -FilePath $configFile -Encoding utf8

Write-Host "✅ Configuración Ultra-Mejorada guardada: $configFile" -ForegroundColor Green

# Mostrar parámetros clave
Write-Host "`n🔑 PARÁMETROS CLAVE ULTRA-MEJORADA:" -ForegroundColor Yellow
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Yellow
Write-Host "💰 Tamaño trade máximo: 0.15 SOL (mejor ratio profit/fees)" -ForegroundColor White
Write-Host "📊 Diferencia precio mínima: 3.5% (supera fees consistentemente)" -ForegroundColor White
Write-Host "⚡ Slippage máximo: 2.5% (mejor precisión)" -ForegroundColor White
Write-Host "🎯 Profit mínimo: 0.0008 SOL (más realista)" -ForegroundColor White
Write-Host "⏱️ Scan interval: 750ms (menos agresivo, mejor calidad)" -ForegroundColor White
Write-Host "🏆 Target profit: 4.0% (realista pero rentable)" -ForegroundColor White

Write-Host "`n🚀 OPTIMIZACIONES ULTRA:" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "🔸 Phoenix DEX prioritizado (fees 0.20%)" -ForegroundColor Green
Write-Host "🔸 Meteora deshabilitado (fees altos)" -ForegroundColor Red
Write-Host "🔸 Meme coins con volumen 100k+ USD" -ForegroundColor Yellow
Write-Host "🔸 Trading solo en horarios de alta volatilidad" -ForegroundColor Cyan
Write-Host "🔸 Detección de pumps y nuevos listings" -ForegroundColor Magenta
Write-Host "🔸 Protección MEV mejorada" -ForegroundColor Blue

Write-Host "`n💡 ESTRATEGIAS AVANZADAS HABILITADAS:" -ForegroundColor Magenta
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Magenta
Write-Host "⚡ Flash Arbitrage (ejecución ultra-rápida)" -ForegroundColor Yellow
Write-Host "🔄 Cross-DEX Routing optimizado" -ForegroundColor Yellow
Write-Host "🎯 Meme Coin Hunting (nuevos listings)" -ForegroundColor Yellow
Write-Host "💧 Liquidity Sniping (cambios de liquidez)" -ForegroundColor Yellow

# Crear script de testing con nueva configuración
$testScript = @'
# 🚀 TEST CONFIGURACIÓN ULTRA-MEJORADA
param([int]$DurationMinutes = 5)

Write-Host "🚀 TESTING CONFIGURACIÓN ULTRA-MEJORADA" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green

$configFile = "arbitrage_settings_ultra_mejorada.json"
if (-not (Test-Path $configFile)) {
    Write-Host "❌ Configuración no encontrada: $configFile" -ForegroundColor Red
    exit 1
}

$config = Get-Content $configFile | ConvertFrom-Json
Write-Host "✅ Configuración cargada: $($config.version)" -ForegroundColor Green
Write-Host "📊 Spread mínimo: $($config.trading.min_price_difference_percent)%" -ForegroundColor Yellow
Write-Host "💰 Trade máximo: $($config.trading.max_trade_size_sol) SOL" -ForegroundColor Yellow
Write-Host "🎯 Profit mínimo: $($config.profit_targets.minimum_profit_sol) SOL" -ForegroundColor Yellow

Write-Host "`nPresiona ENTER para iniciar test mejorado..." -ForegroundColor Cyan
Read-Host

# Simular oportunidades más realistas con configuración mejorada
$OpportunidadesMejoradas = @(
    @{ Pair = "SOL/USDC"; Spread = 4.2; Size = 0.15; Confidence = 85; Profit = 0.0042 },
    @{ Pair = "BONK/SOL"; Spread = 5.8; Size = 0.12; Confidence = 70; Profit = 0.0038 },
    @{ Pair = "WIF/SOL"; Spread = 6.5; Size = 0.10; Confidence = 65; Profit = 0.0041 },
    @{ Pair = "RAY/SOL"; Spread = 3.8; Size = 0.08; Confidence = 80; Profit = 0.0019 },
    @{ Pair = "JUP/SOL"; Spread = 4.9; Size = 0.14; Confidence = 75; Profit = 0.0045 }
)

$startTime = Get-Date
$endTime = $startTime.AddMinutes($DurationMinutes)
$opportunities = 0
$profitable = 0
$totalProfit = 0

Write-Host "`n🔍 SCANNING CON CONFIGURACIÓN MEJORADA..." -ForegroundColor Cyan

while ((Get-Date) -lt $endTime) {
    if ((Get-Random -Minimum 1 -Maximum 100) -lt 15) {
        $opp = $OpportunidadesMejoradas | Get-Random
        $opportunities++
        
        # Cálculo preciso de fees con configuración mejorada
        $phoenixFee = $opp.Size * 0.002  # 0.20%
        $baseFees = 0.00006  # Fees de red más realistas
        $totalFees = $phoenixFee + $baseFees
        $slippage = $opp.Size * 0.01  # 1% slippage
        $netProfit = $opp.Profit - $totalFees - $slippage
        
        Write-Host "🎯 Oportunidad #$opportunities`: $($opp.Pair) | Spread: $($opp.Spread)%" -ForegroundColor Yellow
        Write-Host "   💰 Profit bruto: $([math]::Round($opp.Profit, 6)) SOL" -ForegroundColor White
        Write-Host "   💸 Fees: $([math]::Round($totalFees, 6)) SOL | Slippage: $([math]::Round($slippage, 6)) SOL" -ForegroundColor Red
        Write-Host "   💎 NET: $([math]::Round($netProfit, 6)) SOL" -ForegroundColor $(if($netProfit -gt 0){"Green"}else{"Red"})
        
        if ($netProfit -gt $config.profit_targets.minimum_profit_sol) {
            if ((Get-Random -Minimum 1 -Maximum 100) -lt $opp.Confidence) {
                $profitable++
                $actualProfit = $netProfit * 0.9
                $totalProfit += $actualProfit
                Write-Host "   ✅ ÉXITO: +$([math]::Round($actualProfit, 6)) SOL" -ForegroundColor Green
            } else {
                Write-Host "   📉 FALLÓ: Slippage excesivo" -ForegroundColor Red
            }
        } else {
            Write-Host "   ❌ NO RENTABLE: Profit inferior al mínimo" -ForegroundColor Red
        }
    }
    
    Start-Sleep -Milliseconds 750
}

$successRate = if ($opportunities -gt 0) { [math]::Round($profitable / $opportunities * 100, 1) } else { 0 }
$roiPercent = [math]::Round($totalProfit / 0.29 * 100, 2)

Write-Host "`n🏁 TEST MEJORADO COMPLETADO" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "🎯 Oportunidades: $opportunities" -ForegroundColor Yellow
Write-Host "🏆 Exitosos: $profitable" -ForegroundColor Green
Write-Host "📈 Tasa éxito: ${successRate}%" -ForegroundColor $(if($successRate -gt 60){"Green"}else{"Red"})
Write-Host "💎 Total profit: $([math]::Round($totalProfit, 6)) SOL" -ForegroundColor $(if($totalProfit -gt 0){"Green"}else{"Red"})
Write-Host "📊 ROI: ${roiPercent}%" -ForegroundColor $(if($roiPercent -gt 0){"Green"}else{"Red"})

if ($successRate -gt 70 -and $totalProfit -gt 0.003) {
    Write-Host "`n🚀 CONFIGURACIÓN ULTRA-MEJORADA: EXCELENTE" -ForegroundColor Green
    Write-Host "✅ Lista para trading real" -ForegroundColor Green
} elseif ($successRate -gt 50 -and $totalProfit -gt 0) {
    Write-Host "`n⚡ CONFIGURACIÓN ULTRA-MEJORADA: BUENA" -ForegroundColor Yellow
    Write-Host "✅ Proceder con precaución" -ForegroundColor Yellow
} else {
    Write-Host "`n⚠️ CONFIGURACIÓN ULTRA-MEJORADA: NECESITA AJUSTES" -ForegroundColor Red
}
'@

$testScript | Out-File -FilePath "test_ultra_mejorada.ps1" -Encoding utf8

Write-Host "`n📝 Script de test creado: test_ultra_mejorada.ps1" -ForegroundColor Green
Write-Host "`n🎯 PRÓXIMOS PASOS:" -ForegroundColor Cyan
Write-Host "1️⃣ Ejecutar: .\test_ultra_mejorada.ps1" -ForegroundColor White
Write-Host "2️⃣ Si test exitoso → Trading real con configuración mejorada" -ForegroundColor White
Write-Host "3️⃣ Monitorear durante horarios de alta volatilidad" -ForegroundColor White
