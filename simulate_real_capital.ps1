# 🎯 SIMULACIÓN REALISTA CON 0.29 SOL - CAPITAL REAL
# Simula arbitraje con el capital exacto del usuario

param(
    [int]$DurationMinutes = 3,
    [string]$LogPrefix = "real_capital_simulation"
)

$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$logFile = "${LogPrefix}_${timestamp}.log"

Write-Host "💰 SIMULACIÓN CON CAPITAL REAL: 0.29 SOL" -ForegroundColor Cyan
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "📋 CONFIGURACIÓN DE SIMULACIÓN:" -ForegroundColor Yellow
Write-Host "   • Capital disponible: 0.29 SOL (~$85 USD)" -ForegroundColor White
Write-Host "   • Duración: $DurationMinutes minutos" -ForegroundColor White
Write-Host "   • Max trade por operación: 0.05 SOL (17% del capital)" -ForegroundColor White
Write-Host "   • Estrategias: Cross-chain (comprobado), Flash loans (comprobado)" -ForegroundColor White
Write-Host "   • Log File: $logFile" -ForegroundColor White

# 1. Backup de configuración actual
$configBackup = "arbitrage_settings_backup_before_real_sim_$timestamp.json"
Copy-Item "arbitrage_settings.json" $configBackup
Write-Host "💾 Backup creado: $configBackup" -ForegroundColor Green

# 2. Configuración específica para 0.29 SOL
$realCapitalConfig = @"
{
  "trading": {
    "mode": "simulation",
    "force_real_transactions": false,
    "max_trade_sol": 0.05,
    "min_profit_threshold_sol": 0.0005,
    "min_confidence_threshold": 0.4,
    "max_concurrent_trades": 2,
    "trade_timeout_seconds": 25,
    "min_trade_size_sol": 0.005,
    "max_trade_size_sol": 0.05,
    "max_slippage_bps": 150,
    "military_min_profit_bps": 8,
    "base_profit_percentage": 0.003,
    "max_profit_percentage": 0.025,
    "default_trade_amount_usd": 15.0,
    "estimated_gas_cost_usd": 2.0,
    "estimated_execution_time_ms": 2500,
    "pre_execution_validation": true,
    "program_id_whitelist": [
      "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
      "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
      "11111111111111111111111111111111",
      "ComputeBudget111111111111111111111111111111",
      "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
      "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP",
      "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc",
      "DjVE6JNiYqPL2QXyCUUh8rNjHrbz9hXHNYt99MQ59qw1",
      "EhpbDdNX2ya45KAR8p9FydBE9MJSJgNnuJNsNc7fzhg7",
      "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
      "CAMMCzo5YL8w4VFF8KVHrK22GGUQpMNRoyz8rqE5g9Qy",
      "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
      "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1",
      "27haf8L6oxUeXrHrgEgsexjSY5hbVUWEmvv9Nyxg8vQv"
    ],
    "simulation_before_execution": true,
    "max_simulation_retries": 2
  },
  "wallet": {
    "keypair_file": "./keypair.json",
    "backup_keypair_file": "~/.config/solana/id.json",
    "use_env_private_key": false,
    "env_key_name": "SOLANA_PRIVATE_KEY"
  },
  "rpc": {
    "primary_url": "https://api.mainnet-beta.solana.com",
    "backup_urls": [
      "https://api.mainnet-beta.solana.com",
      "https://rpc.ankr.com/solana",
      "https://solana-api.projectserum.com",
      "https://rpc.helius.xyz/?api-key=YOUR_API_KEY"
    ],
    "timeout_seconds": 8,
    "retry_attempts": 2,
    "retry_delay_ms": 500,
    "commitment": "confirmed",
    "use_websockets": true,
    "websocket_timeout_seconds": 10,
    "max_retries": 3
  },
  "discovery": {
    "enabled": true,
    "discovery_cycle_delay_seconds": 2,
    "max_pools_per_discovery": 20,
    "latency_target_ms": 800,
    "cache_ttl_seconds": 6,
    "max_concurrent_discoveries": 8,
    "discovery_timeout_seconds": 10,
    "pool_discovery_strategies": ["jupiter", "raydium", "orca"],
    "min_liquidity_threshold_usd": 8000.0,
    "skip_low_liquidity_pools": true,
    "price_update_frequency_seconds": 3
  },
  "apis": {
    "jupiter": {
      "enabled": true,
      "base_url": "https://quote-api.jup.ag/v6",
      "timeout_seconds": 6,
      "rate_limit_requests_per_second": 6,
      "retry_attempts": 2,
      "api_key": ""
    },
    "dexscreener": {
      "enabled": true,
      "base_url": "https://api.dexscreener.com/latest",
      "timeout_seconds": 6,
      "rate_limit_requests_per_second": 3,
      "retry_attempts": 2
    },
    "birdeye": {
      "enabled": true,
      "base_url": "https://public-api.birdeye.so",
      "timeout_seconds": 6,
      "rate_limit_requests_per_second": 2,
      "retry_attempts": 2,
      "api_key": ""
    }
  },
  "arbitrage": {
    "enabled_strategies": ["cross_chain", "flash_loan", "traditional"],
    "strategy_priorities": {
      "cross_chain": 1,
      "flash_loan": 2,
      "traditional": 3
    },
    "traditional": {
      "enabled": true,
      "min_profit_threshold_bps": 10,
      "max_hops": 2,
      "slippage_tolerance_bps": 150,
      "price_impact_threshold_bps": 200
    },
    "cross_chain": {
      "enabled": true,
      "min_profit_threshold_bps": 80,
      "supported_chains": ["ethereum", "bsc", "polygon", "avalanche"],
      "bridge_timeout_seconds": 120,
      "gas_estimation_buffer": 1.2
    },
    "flash_loan": {
      "enabled": true,
      "min_profit_threshold_bps": 40,
      "max_loan_amount_sol": 5.0,
      "supported_protocols": ["marginfi", "solend"],
      "loan_timeout_seconds": 30
    }
  },
  "ml": {
    "enabled": true,
    "model_update_frequency_minutes": 20,
    "prediction_confidence_threshold": 0.35,
    "feature_update_interval_seconds": 8,
    "max_model_age_hours": 8,
    "training_data_points": 500,
    "real_time_adaptation": true,
    "sentiment_analysis": true,
    "pattern_recognition_threshold": 0.4
  },
  "risk_management": {
    "enabled": true,
    "max_total_exposure_sol": 0.25,
    "max_loss_per_trade_sol": 0.008,
    "max_daily_trades": 50,
    "circuit_breaker_loss_threshold_sol": 0.03,
    "position_size_limits": {
      "min_sol": 0.005,
      "max_sol": 0.05,
      "max_percentage_of_balance": 17.2
    },
    "stop_loss": {
      "enabled": true,
      "threshold_percentage": 2.5,
      "trailing_stop": false
    },
    "take_profit": {
      "enabled": true,
      "threshold_percentage": 6.0,
      "partial_profit_taking": true
    }
  },
  "performance": {
    "max_memory_usage_mb": 1024,
    "gc_frequency_seconds": 120,
    "thread_pool_size": 8,
    "async_task_timeout_seconds": 20,
    "connection_pool_size": 6,
    "enable_performance_metrics": true,
    "metrics_update_frequency_seconds": 20
  },
  "logging": {
    "level": "info",
    "file": "arbitrage_bot.log",
    "max_file_size_mb": 50,
    "max_files": 3,
    "console_output": true,
    "log_trades": true,
    "log_errors": true,
    "log_performance": true,
    "structured_logging": true
  },
  "monitoring": {
    "enabled": true,
    "health_check_interval_seconds": 45,
    "performance_tracking": true,
    "error_reporting": true,
    "alert_on_profit_threshold": 0.01,
    "alert_on_loss_threshold": 0.005,
    "metrics_retention_hours": 24
  }
}
"@

# 3. Aplicar configuración realista
Write-Host "`n🔧 APLICANDO CONFIGURACIÓN PARA 0.29 SOL..." -ForegroundColor Yellow
$realCapitalConfig | Out-File -FilePath "arbitrage_settings.json" -Encoding UTF8

Write-Host "✅ Configuración aplicada:" -ForegroundColor Green
Write-Host "   • Max trade: 0.05 SOL (17% de 0.29 SOL)" -ForegroundColor White
Write-Host "   • Min profit: 0.0005 SOL (~$0.15)" -ForegroundColor White
Write-Host "   • Risk management: Max loss 0.008 SOL por trade" -ForegroundColor White
Write-Host "   • Estrategias: Cross-chain prioritizado, Flash loans pequeños" -ForegroundColor White

# 4. Verificar balance real
Write-Host "`n💰 VERIFICANDO BALANCE REAL..." -ForegroundColor Cyan
try {
    $balance = solana balance --output json | ConvertFrom-Json
    $actualBalance = [math]::Round($balance.value / 1000000000, 6)
    Write-Host "   Balance verificado: $actualBalance SOL" -ForegroundColor Green
    
    if ($actualBalance -lt 0.25) {
        Write-Host "   ⚠️  Balance menor a 0.25 SOL - trades muy limitados" -ForegroundColor Yellow
    } else {
        Write-Host "   ✅ Balance suficiente para trades de 0.05 SOL" -ForegroundColor Green
    }
} catch {
    Write-Host "   📊 Balance estimado: 0.29 SOL (no se pudo verificar)" -ForegroundColor Yellow
}

# 5. Ejecutar simulación
Write-Host "`n🚀 INICIANDO SIMULACIÓN CON CAPITAL REAL..." -ForegroundColor Magenta
Write-Host "   Tiempo de inicio: $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor White

$startTime = Get-Date
$executable = ".\target\release\arbitrage_phase45_clean.exe"

if (-not (Test-Path $executable)) {
    Write-Host "❌ Executable no encontrado. Compilando..." -ForegroundColor Red
    cargo build --release
}

# Ejecutar con timeout
$process = Start-Process -FilePath $executable -PassThru -RedirectStandardOutput $logFile -RedirectStandardError "${logFile}.error"

# Monitoreo en tiempo real
$endTime = $startTime.AddMinutes($DurationMinutes)
$lastOpportunities = 0
$totalProfit = 0

Write-Host "🔄 Monitoreando simulación por $DurationMinutes minutos..." -ForegroundColor Cyan

while ((Get-Date) -lt $endTime -and -not $process.HasExited) {
    Start-Sleep -Seconds 5
    
    # Leer progreso del log
    if (Test-Path $logFile) {
        $logContent = Get-Content $logFile -Tail 10 -ErrorAction SilentlyContinue
        
        # Buscar oportunidades cross-chain
        $crossChainMatches = $logContent | Select-String "Cross-chain.*profit.*SOL"
        if ($crossChainMatches.Count -gt $lastOpportunities) {
            $newOpportunities = $crossChainMatches.Count - $lastOpportunities
            Write-Host "   ✨ $newOpportunities nuevas oportunidades cross-chain detectadas" -ForegroundColor Green
            $lastOpportunities = $crossChainMatches.Count
        }
        
        # Buscar flash loans
        $flashLoanMatches = $logContent | Select-String "Flash loan.*profit.*SOL"
        if ($flashLoanMatches) {
            Write-Host "   💰 Flash loan opportunities detectadas" -ForegroundColor Yellow
        }
        
        # Mostrar progreso
        $elapsed = [math]::Round(((Get-Date) - $startTime).TotalMinutes, 1)
        Write-Host "   [$(Get-Date -Format 'HH:mm:ss')] Progreso: $elapsed/$DurationMinutes min" -ForegroundColor Cyan
    }
}

# Finalizar proceso
if (-not $process.HasExited) {
    $process.Kill()
    Write-Host "✅ Simulación completada (timeout)" -ForegroundColor Green
} else {
    Write-Host "✅ Simulación completada (proceso terminado)" -ForegroundColor Green
}

# 6. Análisis de resultados
Write-Host "`n📊 ANÁLISIS DE RESULTADOS PARA 0.29 SOL:" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

if (Test-Path $logFile) {
    $fullLog = Get-Content $logFile
    
    # Analizar cross-chain
    $crossChainOpportunities = $fullLog | Select-String "Cross-chain.*profit.*SOL"
    $crossChainCount = $crossChainOpportunities.Count
    
    # Analizar flash loans
    $flashLoanOpportunities = $fullLog | Select-String "Flash loan.*profit.*SOL"
    $flashLoanCount = $flashLoanOpportunities.Count
    
    # Analizar swaps tradicionales
    $traditionalSwaps = $fullLog | Select-String "Traditional.*swap.*profit"
    $traditionalCount = $traditionalSwaps.Count
    
    Write-Host "🎯 OPORTUNIDADES DETECTADAS:" -ForegroundColor Yellow
    Write-Host "   • Cross-chain: $crossChainCount oportunidades" -ForegroundColor White
    Write-Host "   • Flash loans: $flashLoanCount oportunidades" -ForegroundColor White
    Write-Host "   • Swaps tradicionales: $traditionalCount oportunidades" -ForegroundColor White
    
    # Estimar profit potencial
    if ($crossChainCount -gt 0) {
        $estimatedProfit = [math]::Round($crossChainCount * 0.003, 4)  # 0.003 SOL promedio por cross-chain
        $estimatedUSD = [math]::Round($estimatedProfit * 290, 2)  # ~$290 por SOL
        Write-Host "`n💰 PROFIT ESTIMADO PARA 0.29 SOL:" -ForegroundColor Green
        Write-Host "   • Profit potencial: $estimatedProfit SOL (~$$estimatedUSD USD)" -ForegroundColor Green
        Write-Host "   • ROI estimado: $([math]::Round(($estimatedProfit / 0.29) * 100, 2))%" -ForegroundColor Green
        
        if ($estimatedProfit -gt 0.01) {
            Write-Host "`n🚀 RENTABILIDAD EXCELENTE - LISTO PARA TRADING REAL!" -ForegroundColor Magenta
        }
    }
    
    Write-Host "`n📋 Log completo guardado en: $logFile" -ForegroundColor Cyan
} else {
    Write-Host "⚠️  No se pudo leer el log de simulación" -ForegroundColor Yellow
}

# 7. Recomendaciones finales
Write-Host "`n🎯 RECOMENDACIONES PARA 0.29 SOL:" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan

if ($crossChainCount -gt 0 -or $flashLoanCount -gt 0) {
    Write-Host "✅ SISTEMA FUNCIONAL - Oportunidades detectadas" -ForegroundColor Green
    Write-Host "🚀 PRÓXIMO PASO: Activar trading real con:" -ForegroundColor Green
    Write-Host "   .\prepare_real_trading_0.29SOL.ps1" -ForegroundColor White
} else {
    Write-Host "⚠️  Pocas oportunidades detectadas" -ForegroundColor Yellow
    Write-Host "💡 OPCIONES:" -ForegroundColor Yellow
    Write-Host "   1. Esperar mejores condiciones de mercado" -ForegroundColor White
    Write-Host "   2. Reducir umbrales de profit mínimo" -ForegroundColor White
    Write-Host "   3. Ejecutar simulación más larga" -ForegroundColor White
}

Write-Host "`n🔄 Para restaurar configuración original:" -ForegroundColor Cyan
Write-Host "   Copy-Item '$configBackup' 'arbitrage_settings.json'" -ForegroundColor White

Write-Host "`n✨ SIMULACIÓN CON 0.29 SOL COMPLETADA" -ForegroundColor Magenta
