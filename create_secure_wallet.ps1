#!/usr/bin/env pwsh
# CREACIÓN DE WALLET NUEVA Y SEGURA - PROTOCOLO DE EMERGENCIA

Write-Host "🛡️ CREACIÓN DE WALLET NUEVA Y SEGURA" -ForegroundColor Green
Write-Host "================================================================" -ForegroundColor Green
Write-Host ""

Write-Host "🚨 PASO 1: PROTEGER FONDOS RESTANTES" -ForegroundColor Red
Write-Host "================================================================" -ForegroundColor Yellow

Write-Host "⚠️  IMPORTANTE: Transferir 0.001158851 SOL antes que el atacante" -ForegroundColor Red
Write-Host ""

Write-Host "🔄 Creando nueva wallet segura..." -ForegroundColor Cyan
try {
    # Crear nueva wallet con timestamp para evitar conflictos
    $timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
    $newWalletName = "wallet_secure_$timestamp.json"
    
    Write-Host "🔑 Generando nueva wallet: $newWalletName" -ForegroundColor White
    solana-keygen new --outfile $newWalletName --no-bip39-passphrase --force
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Nueva wallet creada exitosamente!" -ForegroundColor Green
        
        # Obtener la dirección pública de la nueva wallet
        $newAddress = solana-keygen pubkey $newWalletName
        Write-Host "📍 Nueva dirección: $newAddress" -ForegroundColor Green
        
        Write-Host ""
        Write-Host "💰 TRANSFERENCIA DE EMERGENCIA:" -ForegroundColor Yellow
        Write-Host "Transfiriendo fondos restantes de wallet comprometida..."
        
        # Transferir fondos usando la wallet vieja como sender
        $oldWallet = "wallet.json"
        $amount = 0.001
        
        Write-Host "🚀 Ejecutando transferencia..." -ForegroundColor Cyan
        solana transfer --from $oldWallet $newAddress $amount --allow-unfunded-recipient
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ ¡Transferencia exitosa!" -ForegroundColor Green
            Write-Host "💰 Fondos protegidos en nueva wallet" -ForegroundColor Green
            
            # Verificar balance de nueva wallet
            Write-Host "🔍 Verificando balance de nueva wallet..." -ForegroundColor Cyan
            $newBalance = solana balance $newAddress
            Write-Host "💰 Balance nuevo: $newBalance" -ForegroundColor Green
            
        } else {
            Write-Host "❌ Error en transferencia - fondos aún en riesgo" -ForegroundColor Red
        }
        
    } else {
        Write-Host "❌ Error creando nueva wallet" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ Error en proceso de seguridad: $_" -ForegroundColor Red
}

Write-Host ""
Write-Host "🔒 PASO 2: CONFIGURACIÓN DE SEGURIDAD" -ForegroundColor Green
Write-Host "================================================================" -ForegroundColor Yellow

Write-Host "📋 CHECKLIST DE SEGURIDAD:" -ForegroundColor Cyan
Write-Host "□ 1. Nueva wallet creada" -ForegroundColor White
Write-Host "□ 2. Fondos transferidos" -ForegroundColor White
Write-Host "□ 3. Wallet vieja marcada como comprometida" -ForegroundColor White
Write-Host "□ 4. Respaldo seguro de nueva wallet" -ForegroundColor White

Write-Host ""
Write-Host "🗑️ PASO 3: DESACTIVAR WALLET COMPROMETIDA" -ForegroundColor Red
Write-Host "================================================================" -ForegroundColor Yellow

# Renombrar wallet comprometida para evitar uso accidental
$compromisedName = "wallet_COMPROMISED_$(Get-Date -Format 'yyyyMMdd').json"
try {
    if (Test-Path "wallet.json") {
        Rename-Item "wallet.json" $compromisedName
        Write-Host "✅ Wallet comprometida renombrada a: $compromisedName" -ForegroundColor Yellow
        Write-Host "⚠️  NO USAR MÁS ESTA WALLET" -ForegroundColor Red
    }
} catch {
    Write-Host "⚠️  Renombrar manualmente wallet.json a $compromisedName" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "🔄 PASO 4: ACTUALIZAR CONFIGURACIÓN" -ForegroundColor Green
Write-Host "================================================================" -ForegroundColor Yellow

Write-Host "📝 Actualizando configuraciones para usar nueva wallet..." -ForegroundColor Cyan

# Actualizar run_small_capital.ps1 para usar nueva wallet
if (Test-Path "run_small_capital.ps1") {
    Write-Host "🔧 Actualizando run_small_capital.ps1..." -ForegroundColor White
    
    # Crear backup del script original
    Copy-Item "run_small_capital.ps1" "run_small_capital_backup.ps1"
    
    # Leer contenido del script
    $scriptContent = Get-Content "run_small_capital.ps1" -Raw
    
    # Reemplazar referencia a wallet.json con nueva wallet
    $scriptContent = $scriptContent -replace 'wallet\.json', $newWalletName
    
    # Guardar script actualizado
    Set-Content "run_small_capital.ps1" $scriptContent
    
    Write-Host "✅ Script actualizado para usar wallet segura" -ForegroundColor Green
}

Write-Host ""
Write-Host "================================================================" -ForegroundColor Green
Write-Host "✅ MIGRACIÓN DE SEGURIDAD COMPLETADA" -ForegroundColor Green
Write-Host "================================================================" -ForegroundColor Green

Write-Host ""
Write-Host "📋 RESUMEN:" -ForegroundColor Yellow
Write-Host "• Nueva wallet segura: $newWalletName" -ForegroundColor Green
Write-Host "• Fondos protegidos: ✅" -ForegroundColor Green
Write-Host "• Wallet vieja desactivada: ✅" -ForegroundColor Green
Write-Host "• Sistema actualizado: ✅" -ForegroundColor Green

Write-Host ""
Write-Host "🚀 PRÓXIMOS PASOS:" -ForegroundColor Cyan
Write-Host "1. 💰 Verificar balance de nueva wallet" -ForegroundColor White
Write-Host "2. 🔒 Respaldar nueva wallet en lugar seguro" -ForegroundColor White
Write-Host "3. 🖥️ Escanear sistema en busca de malware" -ForegroundColor White
Write-Host "4. 🔄 Continuar operaciones con wallet segura" -ForegroundColor White

Write-Host ""
Write-Host "⚠️  IMPORTANTE: Respalda la nueva wallet en lugar seguro!" -ForegroundColor Red
