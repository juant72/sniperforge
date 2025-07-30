# 🚀 TRADING REAL CON JUPITER API - SWAPS REALES EN SOLANA
# Sistema que ejecuta trades reales usando Jupiter aggregator

param(
    [switch]$TestMode = $false,
    [double]$TradeAmount = 0.05,  # Cantidad conservadora para pruebas
    [int]$MaxTrades = 1
)

Write-Host "🚀 TRADING REAL CON JUPITER API" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "💰 CAPITAL: $($TradeAmount) SOL por trade" -ForegroundColor Yellow
Write-Host "🎯 MAX TRADES: $MaxTrades" -ForegroundColor Cyan
Write-Host "🔍 MODO: $(if($TestMode){'TEST (Quotes only)'}else{'REAL TRADING'})" -ForegroundColor $(if($TestMode){'Yellow'}else{'Red'})

# Variables globales
$JUPITER_API = "https://quote-api.jup.ag/v6"
$SOLANA_RPC = "https://api.mainnet-beta.solana.com"
$WALLET_PATH = "./keypair.json"

# Tokens principales para arbitraje
$TOKENS = @{
    "SOL" = "So11111111111111111111111111111111111111112"
    "USDC" = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    "USDT" = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"
    "RAY" = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"
    "BONK" = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"
}

function Write-RealLog {
    param($Message, $Color = "White")
    $timestamp = Get-Date -Format "yyyy-MM-ddTHH:mm:ss.fffZ"
    Write-Host "[$timestamp] $Message" -ForegroundColor $Color
}

function Get-JupiterQuote {
    param(
        [string]$InputMint,
        [string]$OutputMint,
        [string]$Amount
    )
    
    try {
        $url = "$JUPITER_API/quote?inputMint=$InputMint&outputMint=$OutputMint&amount=$Amount&slippageBps=50"
        Write-RealLog "🔍 Obteniendo quote: $InputMint → $OutputMint ($Amount lamports)" "Cyan"
        
        $response = Invoke-RestMethod -Uri $url -Method GET -TimeoutSec 10
        
        if ($response.data) {
            return $response.data[0]  # Mejor ruta
        } else {
            return $response  # Quote directo
        }
    } catch {
        Write-RealLog "❌ Error obteniendo quote: $($_.Exception.Message)" "Red"
        return $null
    }
}

function Test-ArbitrageOpportunity {
    param(
        [string]$Token1,
        [string]$Token2,
        [double]$Amount
    )
    
    $amountLamports = [math]::Floor($Amount * 1000000000)  # Convertir SOL a lamports
    
    Write-RealLog "🔍 Buscando arbitraje: $Token1 ↔ $Token2 (Amount: $Amount SOL)" "Yellow"
    
    # Quote 1: Token1 → Token2
    $quote1 = Get-JupiterQuote -InputMint $TOKENS[$Token1] -OutputMint $TOKENS[$Token2] -Amount $amountLamports
    if (-not $quote1) { return $null }
    
    Start-Sleep -Milliseconds 500  # Evitar rate limiting
    
    # Quote 2: Token2 → Token1 (usando output del quote 1)
    $quote2 = Get-JupiterQuote -InputMint $TOKENS[$Token2] -OutputMint $TOKENS[$Token1] -Amount $quote1.outAmount
    if (-not $quote2) { return $null }
    
    # Calcular arbitraje
    $initialAmount = [double]$amountLamports
    $finalAmount = [double]$quote2.outAmount
    $difference = $finalAmount - $initialAmount
    $percentageDiff = ($difference / $initialAmount) * 100
    
    Write-RealLog "📊 Ruta: $Token1 ($amountLamports) → $Token2 ($($quote1.outAmount)) → $Token1 ($($quote2.outAmount))" "White"
    Write-RealLog "💰 Diferencia: $difference lamports ($([math]::Round($percentageDiff, 4))%)" "White"
    
    if ($percentageDiff -gt 0.5) {  # Mínimo 0.5% para ser rentable
        return @{
            Token1 = $Token1
            Token2 = $Token2
            Quote1 = $quote1
            Quote2 = $quote2
            Profit = $difference
            ProfitPercent = $percentageDiff
            ProfitSOL = $difference / 1000000000
        }
    }
    
    return $null
}

function Invoke-RealSwap {
    param(
        [object]$Quote,
        [string]$Description
    )
    
    if ($TestMode) {
        Write-RealLog "🧪 [TEST MODE] Simulando swap: $Description" "Cyan"
        Start-Sleep -Seconds 2
        return $true
    }
    
    Write-RealLog "🚀 [REAL] Ejecutando swap: $Description" "Red"
    
    try {
        # Preparar transacción usando Jupiter API
        $swapData = @{
            quoteResponse = $Quote
            userPublicKey = (solana address)
            wrapAndUnwrapSol = $true
        } | ConvertTo-Json -Depth 10
        
        Write-RealLog "📝 Preparando transacción..." "Yellow"
        
        # Obtener transacción serializada
        $swapResponse = Invoke-RestMethod -Uri "$JUPITER_API/swap" -Method POST -Body $swapData -ContentType "application/json"
        
        if ($swapResponse.swapTransaction) {
            Write-RealLog "✅ Transacción preparada, enviando a blockchain..." "Green"
            
            # Guardar transacción en archivo temporal
            $txFile = "temp_tx_$(Get-Date -Format 'yyyyMMddHHmmss').json"
            $swapResponse.swapTransaction | Out-File -FilePath $txFile -Encoding utf8
            
            # Firmar y enviar usando Solana CLI
            $result = solana transaction send $txFile --keypair $WALLET_PATH 2>&1
            
            Remove-Item $txFile -ErrorAction SilentlyContinue
            
            if ($result -match "confirmed") {
                Write-RealLog "✅ Swap exitoso: $result" "Green"
                return $true
            } else {
                Write-RealLog "❌ Swap falló: $result" "Red"
                return $false
            }
        } else {
            Write-RealLog "❌ No se pudo preparar transacción" "Red"
            return $false
        }
        
    } catch {
        Write-RealLog "❌ Error ejecutando swap: $($_.Exception.Message)" "Red"
        return $false
    }
}

function Start-RealArbitrageTrading {
    param(
        [double]$Amount,
        [int]$MaxTrades
    )
    
    $tradesExecuted = 0
    $totalProfit = 0
    $successfulTrades = 0
    
    Write-RealLog "🚀 Iniciando búsqueda de arbitraje REAL..." "Green"
    
    # Pares de tokens para arbitraje
    $tradingPairs = @(
        @("SOL", "USDC"),
        @("SOL", "USDT"),
        @("SOL", "RAY"),
        @("USDC", "USDT")
    )
    
    while ($tradesExecuted -lt $MaxTrades) {
        Write-RealLog "🔍 Ciclo de búsqueda #$($tradesExecuted + 1)..." "Cyan"
        
        foreach ($pair in $tradingPairs) {
            $opportunity = Test-ArbitrageOpportunity -Token1 $pair[0] -Token2 $pair[1] -Amount $Amount
            
            if ($opportunity) {
                Write-RealLog "🎯 OPORTUNIDAD DETECTADA!" "Yellow"
                Write-RealLog "═══════════════════════════════════════════════════════════════════════" "Yellow"
                Write-RealLog "💱 Par: $($opportunity.Token1) ↔ $($opportunity.Token2)" "White"
                Write-RealLog "💰 Profit esperado: $([math]::Round($opportunity.ProfitSOL, 6)) SOL ($([math]::Round($opportunity.ProfitPercent, 2))%)" "Green"
                
                if (-not $TestMode) {
                    Write-Host "`n⚠️ CONFIRMACIÓN PARA TRADING REAL:" -ForegroundColor Red
                    Write-Host "💰 Trade amount: $Amount SOL" -ForegroundColor Yellow
                    Write-Host "💎 Profit esperado: $([math]::Round($opportunity.ProfitSOL, 6)) SOL" -ForegroundColor Green
                    Write-Host "🔄 Ruta: $($opportunity.Token1) → $($opportunity.Token2) → $($opportunity.Token1)" -ForegroundColor Cyan
                    Write-Host "`nEscribe 'EJECUTAR' para proceder:" -ForegroundColor Yellow
                    $confirm = Read-Host
                    
                    if ($confirm -ne "EJECUTAR") {
                        Write-RealLog "❌ Trade cancelado por usuario" "Red"
                        continue
                    }
                }
                
                Write-RealLog "🚀 Ejecutando arbitraje..." "Green"
                
                # Ejecutar primer swap
                $swap1Success = Invoke-RealSwap -Quote $opportunity.Quote1 -Description "$($opportunity.Token1) → $($opportunity.Token2)"
                
                if ($swap1Success) {
                    Start-Sleep -Seconds 3  # Esperar confirmación
                    
                    # Ejecutar segundo swap
                    $swap2Success = Invoke-RealSwap -Quote $opportunity.Quote2 -Description "$($opportunity.Token2) → $($opportunity.Token1)"
                    
                    if ($swap2Success) {
                        Write-RealLog "🏆 ARBITRAJE COMPLETADO EXITOSAMENTE!" "Green"
                        $successfulTrades++
                        $totalProfit += $opportunity.ProfitSOL
                    } else {
                        Write-RealLog "⚠️ Primer swap exitoso, segundo falló - posición parcial" "Yellow"
                    }
                } else {
                    Write-RealLog "❌ Primer swap falló" "Red"
                }
                
                $tradesExecuted++
                break  # Salir del loop de pares para este ciclo
            }
        }
        
        if ($tradesExecuted -eq 0) {
            Write-RealLog "😴 No hay oportunidades rentables, esperando..." "Gray"
            Start-Sleep -Seconds 30  # Esperar antes del siguiente ciclo
        }
    }
    
    # Resumen final
    Write-RealLog "`n📊 RESUMEN FINAL DE TRADING REAL:" "Cyan"
    Write-RealLog "════════════════════════════════════════════════════════════════════════" "Cyan"
    Write-RealLog "⚡ Trades intentados: $tradesExecuted" "White"
    Write-RealLog "🏆 Trades exitosos: $successfulTrades" "Green"
    Write-RealLog "💎 Profit total estimado: $([math]::Round($totalProfit, 6)) SOL" "Green"
    Write-RealLog "📈 Tasa de éxito: $([math]::Round($successfulTrades / [math]::Max($tradesExecuted, 1) * 100, 1))%" "White"
}

# Verificaciones previas
Write-RealLog "🔧 Verificando configuración..." "Cyan"

# Verificar wallet
if (-not (Test-Path $WALLET_PATH)) {
    Write-RealLog "❌ Keypair no encontrado en $WALLET_PATH" "Red"
    exit 1
}

# Verificar saldo
try {
    $balance = solana balance
    Write-RealLog "💰 Saldo actual: $balance" "Green"
} catch {
    Write-RealLog "❌ Error verificando saldo" "Red"
    exit 1
}

# Verificar conexión a Jupiter
try {
    $healthCheck = Invoke-RestMethod -Uri "$JUPITER_API/tokens" -Method GET -TimeoutSec 5
    Write-RealLog "✅ Conexión a Jupiter API activa" "Green"
} catch {
    Write-RealLog "❌ No se puede conectar a Jupiter API" "Red"
    exit 1
}

if (-not $TestMode) {
    Write-Host "`n🚨 ADVERTENCIA - TRADING REAL 🚨" -ForegroundColor Red
    Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Red
    Write-Host "🔴 Vas a ejecutar trades REALES con dinero real" -ForegroundColor Red
    Write-Host "🔴 Cada trade usará $TradeAmount SOL" -ForegroundColor Red
    Write-Host "🔴 Máximo $MaxTrades trades" -ForegroundColor Red
    Write-Host "🔴 Fees de red y slippage se aplicarán" -ForegroundColor Red
    Write-Host "`nEscribe 'CONFIRMO TRADING REAL' para continuar:" -ForegroundColor Yellow
    $finalConfirm = Read-Host
    
    if ($finalConfirm -ne "CONFIRMO TRADING REAL") {
        Write-RealLog "❌ Trading cancelado" "Red"
        exit 0
    }
}

# Iniciar trading
Start-RealArbitrageTrading -Amount $TradeAmount -MaxTrades $MaxTrades

Write-RealLog "`n🏁 Trading real completado" "Green"
