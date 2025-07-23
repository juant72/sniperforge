# Script para crear tokens de prueba en DevNet
# Ejecutar con: .\setup-devnet-tokens.ps1

Write-Host "=== SniperForge: Setup de Tokens DevNet ===" -ForegroundColor Green

# Verificar que Solana CLI está instalado
try {
    $version = solana --version
    Write-Host "✅ Solana CLI encontrado: $version" -ForegroundColor Green
} catch {
    Write-Host "❌ ERROR: Solana CLI no encontrado. Instalar desde https://docs.solana.com/cli/install-solana-cli-tools" -ForegroundColor Red
    exit 1
}

# Verificar que spl-token está instalado
try {
    $splVersion = spl-token --version
    Write-Host "✅ SPL Token CLI encontrado: $splVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ ERROR: spl-token CLI no encontrado. Instalar con: cargo install spl-token-cli" -ForegroundColor Red
    exit 1
}

# Configurar DevNet
Write-Host "`n=== Configurando DevNet ===" -ForegroundColor Yellow
solana config set --url https://api.devnet.solana.com

# Verificar configuración
$config = solana config get
Write-Host $config

# Verificar balance y hacer airdrop si es necesario
$balance = solana balance --lamports
$balanceSOL = [math]::Round($balance / 1000000000, 4)

Write-Host "`n💰 Balance actual: $balanceSOL SOL" -ForegroundColor Cyan

if ($balanceSOL -lt 1) {
    Write-Host "💸 Balance bajo. Solicitando airdrop..." -ForegroundColor Yellow
    try {
        solana airdrop 2
        Start-Sleep 3
        $newBalance = solana balance --lamports
        $newBalanceSOL = [math]::Round($newBalance / 1000000000, 4)
        Write-Host "✅ Nuevo balance: $newBalanceSOL SOL" -ForegroundColor Green
    } catch {
        Write-Host "⚠️  Airdrop falló. Prueba manualmente: solana airdrop 2" -ForegroundColor Yellow
    }
}

# Función para crear y configurar un token
function Create-DevNetToken {
    param(
        [string]$Name,
        [int]$Decimals = 6,
        [int]$Supply = 1000
    )
    
    Write-Host "`n=== Creando $Name ===" -ForegroundColor Yellow
    
    # Crear token
    $createOutput = spl-token create-token --decimals $Decimals 2>&1
    $mintAddress = ""
    
    foreach ($line in $createOutput) {
        if ($line -match "Creating token (.+)") {
            $mintAddress = $matches[1]
            break
        }
    }
    
    if (-not $mintAddress) {
        Write-Host "❌ Error creando token $Name" -ForegroundColor Red
        return $null
    }
    
    Write-Host "🪙 Token creado: $mintAddress" -ForegroundColor Green
    
    # Crear cuenta
    Write-Host "📝 Creando cuenta para el token..."
    try {
        spl-token create-account $mintAddress | Out-Null
        Write-Host "✅ Cuenta creada" -ForegroundColor Green
    } catch {
        Write-Host "❌ Error creando cuenta" -ForegroundColor Red
        return $null
    }
    
    # Mintear tokens
    Write-Host "🏭 Minteando $Supply tokens..."
    try {
        spl-token mint $mintAddress $Supply | Out-Null
        Write-Host "✅ $Supply tokens minteados" -ForegroundColor Green
    } catch {
        Write-Host "❌ Error minteando tokens" -ForegroundColor Red
        return $null
    }
    
    return $mintAddress
}

# Crear tokens de prueba
$tokens = @{}

# Token 1: TEST_USDC
$tokens.TEST_USDC = Create-DevNetToken -Name "TEST_USDC" -Decimals 6 -Supply 5000

# Token 2: TEST_USDT  
$tokens.TEST_USDT = Create-DevNetToken -Name "TEST_USDT" -Decimals 6 -Supply 3000

# Token 3: TEST_RAY
$tokens.TEST_RAY = Create-DevNetToken -Name "TEST_RAY" -Decimals 6 -Supply 1000

# Verificar que todos los tokens se crearon
$allCreated = $true
foreach ($token in $tokens.Keys) {
    if (-not $tokens[$token]) {
        $allCreated = $false
        break
    }
}

if (-not $allCreated) {
    Write-Host "`n❌ Algunos tokens no se pudieron crear. Revisa los errores arriba." -ForegroundColor Red
    exit 1
}

# Mostrar resumen
Write-Host "`n=== RESUMEN DE TOKENS CREADOS ===" -ForegroundColor Green
Write-Host "SOL (nativo): So11111111111111111111111111111111111111112" -ForegroundColor Cyan
foreach ($token in $tokens.Keys) {
    Write-Host "${token}: $($tokens[$token])" -ForegroundColor Cyan
}

# Crear archivo de configuración actualizado
$configJson = @{
    network = "devnet"
    cluster_url = "https://api.devnet.solana.com"
    enable_real_swaps = $true
    programs = @{
        jupiter = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"
        orca = "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP"
        raydium = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        token = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    }
    tokens = @{
        SOL = @{
            mint = "So11111111111111111111111111111111111111112"
            symbol = "SOL"
            name = "Solana"
            decimals = 9
            verified = $true
        }
        TEST_USDC = @{
            mint = $tokens.TEST_USDC
            symbol = "USDC"
            name = "Test USDC (DevNet)"
            decimals = 6
            verified = $true
        }
        TEST_USDT = @{
            mint = $tokens.TEST_USDT
            symbol = "USDT"
            name = "Test USDT (DevNet)"
            decimals = 6
            verified = $true
        }
        TEST_RAY = @{
            mint = $tokens.TEST_RAY
            symbol = "RAY"
            name = "Test RAY (DevNet)"
            decimals = 6
            verified = $true
        }
    }
}

# Guardar configuración
$configPath = "config/devnet-custom.json"
$configJson | ConvertTo-Json -Depth 4 | Out-File -FilePath $configPath -Encoding UTF8

Write-Host "`n✅ Configuración guardada en: $configPath" -ForegroundColor Green

# Mostrar todos los tokens del usuario
Write-Host "`n=== TODOS TUS TOKENS ===" -ForegroundColor Yellow
spl-token accounts

Write-Host "`n🎉 Setup completado! Ahora puedes hacer swaps reales en DevNet." -ForegroundColor Green
Write-Host "💡 Para usar estos tokens, actualiza tu configuración para cargar desde: $configPath" -ForegroundColor Yellow

# Crear comando de prueba
$testCommand = "cargo run --bin test_real_swap_configured"
Write-Host "`n🧪 Comando de prueba sugerido:" -ForegroundColor Cyan
Write-Host $testCommand -ForegroundColor White

Write-Host "`n📝 Nota: Guarda estos mint addresses en un lugar seguro para futuras referencias." -ForegroundColor Yellow
