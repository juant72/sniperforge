# Manual Token Creation for DevNet Testing
# Ejecutar línea por línea para crear tokens de prueba

Write-Host "🚀 === Creación Manual de Tokens DevNet ===" -ForegroundColor Green

# Verificar configuración
Write-Host "`n1. Verificando configuración actual..." -ForegroundColor Yellow
solana config get

# Si no estás en DevNet, cambiar
Write-Host "`n2. Configurando DevNet con RPC premium..." -ForegroundColor Yellow
solana config set --url https://solana-devnet.g.alchemy.com/v2/X64q4zZFEMz_RYzthxUMg

# Verificar balance
Write-Host "`n3. Verificando balance..." -ForegroundColor Yellow
$balance = solana balance
Write-Host "Balance actual: $balance" -ForegroundColor Cyan

# Crear primer token (TEST_USDC)
Write-Host "`n4. Creando TEST_USDC..." -ForegroundColor Yellow
Write-Host "Comando: spl-token create-token --decimals 6" -ForegroundColor Gray
$token1 = Read-Host "⏸️  Presiona Enter para continuar y luego pega aquí el mint address generado"

if ($token1) {
    Write-Host "✅ TEST_USDC Mint: $token1" -ForegroundColor Green
    
    Write-Host "Creando cuenta para TEST_USDC..." -ForegroundColor Yellow
    spl-token create-account $token1
    
    Write-Host "Minteando 1000 TEST_USDC..." -ForegroundColor Yellow
    spl-token mint $token1 1000
    
    Write-Host "Verificando balance..." -ForegroundColor Yellow
    spl-token balance $token1
}

# Crear segundo token (TEST_USDT)
Write-Host "`n5. Creando TEST_USDT..." -ForegroundColor Yellow
Write-Host "Comando: spl-token create-token --decimals 6" -ForegroundColor Gray
$token2 = Read-Host "⏸️  Presiona Enter para continuar y luego pega aquí el mint address generado"

if ($token2) {
    Write-Host "✅ TEST_USDT Mint: $token2" -ForegroundColor Green
    
    Write-Host "Creando cuenta para TEST_USDT..." -ForegroundColor Yellow
    spl-token create-account $token2
    
    Write-Host "Minteando 800 TEST_USDT..." -ForegroundColor Yellow
    spl-token mint $token2 800
    
    Write-Host "Verificando balance..." -ForegroundColor Yellow
    spl-token balance $token2
}

# Crear tercer token (TEST_RAY)
Write-Host "`n6. Creando TEST_RAY..." -ForegroundColor Yellow
Write-Host "Comando: spl-token create-token --decimals 6" -ForegroundColor Gray
$token3 = Read-Host "⏸️  Presiona Enter para continuar y luego pega aquí el mint address generado"

if ($token3) {
    Write-Host "✅ TEST_RAY Mint: $token3" -ForegroundColor Green
    
    Write-Host "Creando cuenta para TEST_RAY..." -ForegroundColor Yellow
    spl-token create-account $token3
    
    Write-Host "Minteando 500 TEST_RAY..." -ForegroundColor Yellow
    spl-token mint $token3 500
    
    Write-Host "Verificando balance..." -ForegroundColor Yellow
    spl-token balance $token3
}

# Mostrar resumen
Write-Host "`n🎉 === RESUMEN DE TOKENS CREADOS ===" -ForegroundColor Green

if ($token1) { Write-Host "TEST_USDC: $token1" -ForegroundColor Cyan }
if ($token2) { Write-Host "TEST_USDT: $token2" -ForegroundColor Cyan }
if ($token3) { Write-Host "TEST_RAY: $token3" -ForegroundColor Cyan }
Write-Host "SOL/wSOL: So11111111111111111111111111111111111111112" -ForegroundColor Cyan

# Mostrar todos los tokens
Write-Host "`n7. Mostrando todos tus tokens..." -ForegroundColor Yellow
spl-token accounts

# Generar configuración JSON
Write-Host "`n8. Generando archivo de configuración..." -ForegroundColor Yellow

$configContent = @"
{
  "network": "devnet",
  "cluster_url": "https://solana-devnet.g.alchemy.com/v2/X64q4zZFEMz_RYzthxUMg",
  "enable_real_swaps": true,
  "description": "Tokens creados manualmente en DevNet con RPC premium",
  "created_at": "$(Get-Date -Format 'yyyy-MM-ddTHH:mm:ssZ')",
  "programs": {
    "jupiter": "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
    "orca": "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP",
    "raydium": "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
    "token": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
  },
  "tokens": {
    "SOL": {
      "mint": "So11111111111111111111111111111111111111112",
      "symbol": "SOL",
      "name": "Solana",
      "decimals": 9,
      "verified": true
    }$(if ($token1) { ",
    `"TEST_USDC`": {
      `"mint`": `"$token1`",
      `"symbol`": `"USDC`",
      `"name`": `"Test USDC (DevNet)`",
      `"decimals`": 6,
      `"verified`": true
    }" })$(if ($token2) { ",
    `"TEST_USDT`": {
      `"mint`": `"$token2`",
      `"symbol`": `"USDT`",
      `"name`": `"Test USDT (DevNet)`",
      `"decimals`": 6,
      `"verified`": true
    }" })$(if ($token3) { ",
    `"TEST_RAY`": {
      `"mint`": `"$token3`",
      `"symbol`": `"RAY`",
      `"name`": `"Test RAY (DevNet)`",
      `"decimals`": 6,
      `"verified`": true
    }" })
  }
}
"@

# Crear directorio config si no existe
if (!(Test-Path "config")) {
    New-Item -ItemType Directory -Path "config"
}

# Guardar configuración
$configContent | Out-File -FilePath "config/devnet-manual.json" -Encoding UTF8

Write-Host "✅ Configuración guardada en: config/devnet-manual.json" -ForegroundColor Green

Write-Host "`n🧪 === COMANDOS DE PRUEBA ===" -ForegroundColor Yellow
Write-Host "cargo run --bin test_basic_connectivity" -ForegroundColor White
Write-Host "cargo run --bin test_real_swap_configured" -ForegroundColor White

Write-Host "`n💡 === PRÓXIMOS PASOS ===" -ForegroundColor Yellow
Write-Host "1. Actualiza tu engine para usar config/devnet-manual.json" -ForegroundColor White
Write-Host "2. Prueba swaps entre los tokens creados" -ForegroundColor White
Write-Host "3. Verifica las transacciones en el explorer" -ForegroundColor White

Write-Host "`n📝 NOTAS IMPORTANTES:" -ForegroundColor Yellow
Write-Host "• Guarda los mint addresses en un lugar seguro" -ForegroundColor White
Write-Host "• Estos tokens solo existen en DevNet" -ForegroundColor White
Write-Host "• Usa el RPC de Alchemy que funciona mejor" -ForegroundColor White
