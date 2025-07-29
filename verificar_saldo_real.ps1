# 🔍 VERIFICACIÓN DE SALDO REAL - WALLET SOLANA
# Script para verificar el saldo real de tu wallet en blockchain

Write-Host "🔍 VERIFICACIÓN DE SALDO REAL EN BLOCKCHAIN" -ForegroundColor Yellow
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Yellow

# Verificar si tienes solana CLI instalado
try {
    $solanaVersion = solana --version 2>$null
    if ($solanaVersion) {
        Write-Host "✅ Solana CLI detectado: $solanaVersion" -ForegroundColor Green
        
        # Verificar configuración
        Write-Host "`n🔧 Verificando configuración de Solana CLI..." -ForegroundColor Cyan
        $config = solana config get 2>$null
        Write-Host $config -ForegroundColor White
        
        # Obtener dirección de wallet
        Write-Host "`n🏦 Obteniendo dirección de wallet..." -ForegroundColor Cyan
        $walletAddress = solana address 2>$null
        if ($walletAddress) {
            Write-Host "📍 Dirección wallet: $walletAddress" -ForegroundColor White
            
            # Verificar saldo real
            Write-Host "`n💰 Verificando saldo real en blockchain..." -ForegroundColor Cyan
            $balance = solana balance 2>$null
            if ($balance) {
                Write-Host "💎 SALDO REAL: $balance" -ForegroundColor Green
                
                # Extraer valor numérico
                if ($balance -match "(\d+\.?\d*)\s*SOL") {
                    $balanceSOL = [decimal]$matches[1]
                    Write-Host "💰 Saldo numérico: $balanceSOL SOL" -ForegroundColor Yellow
                    
                    if ($balanceSOL -ge 0.29) {
                        Write-Host "✅ Saldo suficiente para trading" -ForegroundColor Green
                    } else {
                        Write-Host "⚠️ Saldo insuficiente para trading (mínimo 0.29 SOL)" -ForegroundColor Red
                    }
                }
            } else {
                Write-Host "❌ Error obteniendo saldo" -ForegroundColor Red
            }
            
            # Verificar transacciones recientes
            Write-Host "`n📋 Verificando últimas transacciones..." -ForegroundColor Cyan
            $recentTx = solana transaction-history $walletAddress --limit 5 2>$null
            if ($recentTx) {
                Write-Host $recentTx -ForegroundColor Gray
            }
            
        } else {
            Write-Host "❌ No se pudo obtener dirección de wallet" -ForegroundColor Red
        }
        
    } else {
        Write-Host "❌ Solana CLI no está instalado o no está en PATH" -ForegroundColor Red
        Write-Host "💡 Para instalar Solana CLI:" -ForegroundColor Yellow
        Write-Host "   sh -c `"$(curl -sSfL https://release.solana.com/v1.18.4/install)`"" -ForegroundColor Cyan
    }
} catch {
    Write-Host "❌ Error ejecutando Solana CLI: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`n🤔 ANÁLISIS DE LA SITUACIÓN:" -ForegroundColor Magenta
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Magenta
Write-Host "🔸 Los tests anteriores fueron SIMULACIONES (DRY RUN)" -ForegroundColor Yellow
Write-Host "🔸 No hubo transacciones reales en blockchain" -ForegroundColor Yellow
Write-Host "🔸 Tu saldo real no ha cambiado" -ForegroundColor Yellow
Write-Host "🔸 Necesitamos configurar trading REAL con blockchain" -ForegroundColor Yellow

Write-Host "`n🛠️ PARA TRADING REAL NECESITAS:" -ForegroundColor Cyan
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "1️⃣ Wallet Solana configurada con clave privada" -ForegroundColor White
Write-Host "2️⃣ Conexión a RPC de Solana mainnet" -ForegroundColor White
Write-Host "3️⃣ Integración con DEX APIs reales" -ForegroundColor White
Write-Host "4️⃣ Herramientas de trading real (Jupiter, Raydium SDKs)" -ForegroundColor White

Write-Host "`n💡 OPCIONES PARA TRADING REAL:" -ForegroundColor Green
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "🔸 Usar Jupiter API para swaps reales" -ForegroundColor White
Write-Host "🔸 Integrar con Raydium SDK" -ForegroundColor White
Write-Host "🔸 Usar herramientas como Solana CLI + scripts" -ForegroundColor White
Write-Host "🔸 Conectar con wallet como Phantom via Web3" -ForegroundColor White

Write-Host "`n⚠️ ADVERTENCIA IMPORTANTE:" -ForegroundColor Red
Write-Host "════════════════════════════════════════════════════════════════════════" -ForegroundColor Red
Write-Host "🔴 Trading real implica riesgo de pérdida de fondos" -ForegroundColor Red
Write-Host "🔴 Fees de red pueden ser altos durante congestión" -ForegroundColor Red
Write-Host "🔴 Slippage real puede diferir de estimaciones" -ForegroundColor Red
Write-Host "🔴 Siempre prueba con cantidades pequeñas primero" -ForegroundColor Red
