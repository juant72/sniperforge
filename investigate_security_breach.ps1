#!/usr/bin/env pwsh
# INVESTIGACIÓN DE SEGURIDAD - SNIPERFORGE
# Análisis completo de posible compromiso de wallet

Write-Host "🔍 INVESTIGACIÓN DE SEGURIDAD - WALLET COMPROMISE" -ForegroundColor Red
Write-Host "================================================================" -ForegroundColor Red
Write-Host ""

# Información de la wallet
$WALLET_ADDRESS = "JDzF9LkpoQVac6c6ufHW1c6Gevd3vFB4RXSSjceL8Kq7"
$EXPECTED_BALANCE = 0.29
$ACTUAL_BALANCE = 0.001158851

Write-Host "📋 DATOS DE LA INVESTIGACIÓN:" -ForegroundColor Yellow
Write-Host "🔑 Wallet Address: $WALLET_ADDRESS" -ForegroundColor White
Write-Host "💰 Balance Esperado: $EXPECTED_BALANCE SOL" -ForegroundColor Green
Write-Host "💰 Balance Actual: $ACTUAL_BALANCE SOL" -ForegroundColor Red
Write-Host "📉 Diferencia: $($EXPECTED_BALANCE - $ACTUAL_BALANCE) SOL" -ForegroundColor Red
Write-Host ""

# 1. Verificar que wallet.json corresponde a la dirección
Write-Host "🔍 PASO 1: VERIFICACIÓN DE IDENTIDAD DE WALLET" -ForegroundColor Cyan
Write-Host "Verificando que wallet.json corresponde a $WALLET_ADDRESS..."

try {
    $verificationResult = solana-keygen verify $WALLET_ADDRESS wallet.json 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ CONFIRMADO: wallet.json corresponde a la dirección" -ForegroundColor Green
    } else {
        Write-Host "❌ ERROR: wallet.json NO corresponde a la dirección" -ForegroundColor Red
        Write-Host "Resultado: $verificationResult" -ForegroundColor Gray
    }
} catch {
    Write-Host "❌ ERROR en verificación: $_" -ForegroundColor Red
}
Write-Host ""

# 2. Obtener balance actual
Write-Host "🔍 PASO 2: VERIFICACIÓN DE BALANCE ACTUAL" -ForegroundColor Cyan
Write-Host "Obteniendo balance actual..."

try {
    $currentBalance = solana balance $WALLET_ADDRESS 2>&1
    Write-Host "💰 Balance Actual: $currentBalance" -ForegroundColor White
} catch {
    Write-Host "❌ ERROR obteniendo balance: $_" -ForegroundColor Red
}
Write-Host ""

# 3. Analizar historial de transacciones
Write-Host "🔍 PASO 3: ANÁLISIS DE HISTORIAL DE TRANSACCIONES" -ForegroundColor Cyan
Write-Host "Obteniendo últimas 30 transacciones..."

try {
    $transactions = solana transaction-history $WALLET_ADDRESS --limit 30 2>&1
    if ($LASTEXITCODE -eq 0) {
        $transactionLines = $transactions -split "`n" | Where-Object { $_ -match "^[A-Za-z0-9]+$" -and $_.Length -gt 40 }
        Write-Host "📊 Transacciones encontradas: $($transactionLines.Count)" -ForegroundColor Yellow
        
        # Mostrar las primeras 5 transacciones
        Write-Host "🔍 Últimas 5 transacciones:" -ForegroundColor White
        $transactionLines | Select-Object -First 5 | ForEach-Object {
            Write-Host "   $_" -ForegroundColor Gray
        }
    } else {
        Write-Host "❌ ERROR obteniendo historial: $transactions" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ ERROR en historial: $_" -ForegroundColor Red
}
Write-Host ""

# 4. Buscar archivos de wallet adicionales
Write-Host "🔍 PASO 4: BÚSQUEDA DE WALLETS ADICIONALES" -ForegroundColor Cyan
Write-Host "Buscando otros archivos de wallet en el sistema..."

$walletFiles = @()
try {
    # Buscar archivos .json que podrían ser wallets
    $jsonFiles = Get-ChildItem -Recurse -Filter "*.json" | Where-Object { 
        $_.Name -notmatch "config|package|lock|node_modules|target" 
    }
    
    foreach ($file in $jsonFiles) {
        try {
            $content = Get-Content $file.FullName -Raw -ErrorAction SilentlyContinue
            if ($content -match '\[\s*\d+(?:,\s*\d+)*\s*\]' -and $content.Length -gt 100 -and $content.Length -lt 1000) {
                $walletFiles += $file.FullName
                Write-Host "   🔑 Posible wallet encontrado: $($file.Name)" -ForegroundColor Yellow
                
                # Intentar obtener la dirección pública
                try {
                    $pubkey = solana-keygen pubkey $file.FullName 2>$null
                    if ($LASTEXITCODE -eq 0) {
                        Write-Host "      📍 Dirección: $pubkey" -ForegroundColor White
                        
                        # Verificar balance de esta wallet
                        $balance = solana balance $pubkey 2>$null
                        if ($LASTEXITCODE -eq 0) {
                            Write-Host "      💰 Balance: $balance" -ForegroundColor Cyan
                        }
                    }
                } catch {
                    Write-Host "      ❌ No se pudo obtener dirección pública" -ForegroundColor Red
                }
            }
        } catch {
            # Ignorar archivos que no se pueden leer
        }
    }
    
    if ($walletFiles.Count -eq 0) {
        Write-Host "📋 No se encontraron wallets adicionales" -ForegroundColor Gray
    } else {
        Write-Host "📊 Total de wallets encontrados: $($walletFiles.Count)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "❌ ERROR en búsqueda: $_" -ForegroundColor Red
}
Write-Host ""

# 5. Verificar logs del sistema
Write-Host "🔍 PASO 5: BÚSQUEDA DE LOGS SOSPECHOSOS" -ForegroundColor Cyan
Write-Host "Revisando logs en el directorio..."

try {
    if (Test-Path "logs") {
        $logFiles = Get-ChildItem -Path "logs" -Filter "*.log" -Recurse | Sort-Object LastWriteTime -Descending
        Write-Host "📋 Archivos de log encontrados: $($logFiles.Count)" -ForegroundColor Yellow
        
        foreach ($logFile in $logFiles | Select-Object -First 3) {
            Write-Host "   📄 $($logFile.Name) - Modificado: $($logFile.LastWriteTime)" -ForegroundColor Gray
        }
    } else {
        Write-Host "📋 No hay directorio de logs" -ForegroundColor Gray
    }
} catch {
    Write-Host "❌ ERROR revisando logs: $_" -ForegroundColor Red
}
Write-Host ""

# 6. Conclusiones de seguridad
Write-Host "🔍 PASO 6: ANÁLISIS Y CONCLUSIONES" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Yellow

$missingFunds = $EXPECTED_BALANCE - $ACTUAL_BALANCE
Write-Host "📊 RESUMEN DEL ANÁLISIS:" -ForegroundColor White
Write-Host "💸 Fondos faltantes: $missingFunds SOL (~$([math]::Round($missingFunds * 150, 2)) USD aprox.)" -ForegroundColor Red
Write-Host "📈 Porcentaje de pérdida: $([math]::Round(($missingFunds / $EXPECTED_BALANCE) * 100, 1))%" -ForegroundColor Red

Write-Host ""
Write-Host "🚨 POSIBLES CAUSAS:" -ForegroundColor Red
Write-Host "1. 💸 Fondos transferidos a otra wallet (posible compromiso)" -ForegroundColor Yellow
Write-Host "2. 🔄 Fondos utilizados para transacciones/fees" -ForegroundColor Yellow
Write-Host "3. 📊 Balance inicial incorrecto (expectativa errónea)" -ForegroundColor Yellow
Write-Host "4. 🔑 Compromiso de private key" -ForegroundColor Yellow

Write-Host ""
Write-Host "🛡️ RECOMENDACIONES INMEDIATAS:" -ForegroundColor Green
Write-Host "1. 🔄 Crear nueva wallet inmediatamente" -ForegroundColor White
Write-Host "2. 💰 Transferir fondos restantes a wallet nueva" -ForegroundColor White
Write-Host "3. 🔍 Revisar todas las transacciones detalladamente" -ForegroundColor White
Write-Host "4. 🔒 Cambiar todas las contraseñas relacionadas" -ForegroundColor White
Write-Host "5. 🖥️ Escanear sistema en busca de malware" -ForegroundColor White

Write-Host ""
Write-Host "================================================================" -ForegroundColor Red
Write-Host "🔍 INVESTIGACIÓN COMPLETADA" -ForegroundColor Red
Write-Host "================================================================" -ForegroundColor Red
