# Guía: Crear Tokens SPL en DevNet para Testing

## ¿Por qué necesitas crear tus propios tokens?

**DevNet NO tiene tokens pre-existentes** como USDC, USDT, RAY que están en MainNet. Estos tokens son específicos de cada red. Para hacer swaps reales en DevNet necesitas crear tus propios tokens SPL.

## Método 1: Usar spl-token CLI (Recomendado)

### Paso 1: Configurar el entorno
```powershell
# Verificar que estás en DevNet
solana config get
# Si no estás en DevNet:
solana config set --url https://api.devnet.solana.com

# Verificar balance (necesitas SOL para pagar las transacciones)
solana balance
# Si no tienes SOL:
solana airdrop 2
```

### Paso 2: Crear un token personalizado
```powershell
# Crear un nuevo token (mint)
spl-token create-token

# Ejemplo de salida:
# Creating token AQoKYV7tYpTrFZN6P5oUufbQKAUr9mNYGe1TTJC9wajM
# Signature: 47hsLFxWRCg8azaZZPSnQR8DNTRsGyPNfUK7jqyzgt7wf9eag3nSnewqoZrVZHKm8zt3B6gzxhr91gdQ5qYrsRG4

# El identificador único del token será algo como: AQoKYV7tYpTrFZN6P5oUufbQKAUr9mNYGe1TTJC9wajM
```

### Paso 3: Crear cuenta para el token
```powershell
# Crear una cuenta para mantener balance del token
spl-token create-account AQoKYV7tYpTrFZN6P5oUufbQKAUr9mNYGe1TTJC9wajM

# Ejemplo de salida:
# Creating account 7UX2i7SucgLMQcfZ75s3VXmZZY4YRUyJN9X1RgfMoDUi
# Signature: 42Sa5eK9dMEQyvD9GMHuKxXf55WLZ7tfjabUKDhNoZRAxj9MsnN7omriWMEHXLea3aYpjZ862qocRLVikvkHkyfy
```

### Paso 4: Mintear tokens
```powershell
# Mintear 1000 tokens a tu cuenta
spl-token mint AQoKYV7tYpTrFZN6P5oUufbQKAUr9mNYGe1TTJC9wajM 1000

# Verificar balance
spl-token balance AQoKYV7tYpTrFZN6P5oUufbQKAUr9mNYGe1TTJC9wajM
# Output: 1000
```

### Paso 5: Crear múltiples tokens para swaps
```powershell
# Crear segundo token (para simular USDC)
spl-token create-token --decimals 6
# Guardar el mint address, ej: BTokenAddress123...

# Crear cuenta y mintear
spl-token create-account BTokenAddress123...
spl-token mint BTokenAddress123... 500

# Crear tercer token (para simular USDT)
spl-token create-token --decimals 6
# Guardar el mint address, ej: CTokenAddress456...

spl-token create-account CTokenAddress456...
spl-token mint CTokenAddress456... 750
```

## Método 2: Script Automatizado

```powershell
# Crear y ejecutar script para setup completo
# create_devnet_tokens.ps1

$Env:SOLANA_CONFIG = "devnet"
solana config set --url https://api.devnet.solana.com

Write-Host "=== Creando Token 1 (TEST_USDC) ==="
$token1 = spl-token create-token --decimals 6 | Select-String "Creating token" | ForEach-Object { $_.ToString().Split(" ")[2] }
Write-Host "Token 1: $token1"

spl-token create-account $token1
spl-token mint $token1 1000

Write-Host "=== Creando Token 2 (TEST_USDT) ==="
$token2 = spl-token create-token --decimals 6 | Select-String "Creating token" | ForEach-Object { $_.ToString().Split(" ")[2] }
Write-Host "Token 2: $token2"

spl-token create-account $token2
spl-token mint $token2 1500

Write-Host "=== Tokens creados ==="
Write-Host "TEST_USDC: $token1"
Write-Host "TEST_USDT: $token2"
Write-Host "wSOL: So11111111111111111111111111111111111111112"

# Mostrar todos los tokens
spl-token accounts
```

## Actualizar Configuración de SniperForge

Una vez que tengas los mint addresses de tus tokens, actualiza tu `config/devnet.json`:

```json
{
  "network": "devnet",
  "cluster_url": "https://api.devnet.solana.com",
  "enable_real_swaps": true,
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
    },
    "TEST_USDC": {
      "mint": "TU_TOKEN_1_AQUI",
      "symbol": "USDC",
      "name": "Test USDC",
      "decimals": 6,
      "verified": true
    },
    "TEST_USDT": {
      "mint": "TU_TOKEN_2_AQUI", 
      "symbol": "USDT",
      "name": "Test USDT",
      "decimals": 6,
      "verified": true
    }
  }
}
```

## Notas Importantes

1. **Cada vez que reinicies DevNet**: Los tokens que crees persisten mientras el cluster esté activo
2. **Costo**: Cada token cuesta ~0.00203928 SOL para rent-exempt
3. **Límites**: No hay límites en cuántos tokens puedes crear
4. **Compatibilidad**: Estos tokens funcionan con todos los DEXs en DevNet (Orca, Raydium, etc.)

## Alternativa: Test Validator Local

Si quieres máximo control, puedes usar un test validator local:

```powershell
# En una terminal separada
solana-test-validator

# En otra terminal
solana config set --url http://127.0.0.1:8899
solana airdrop 10
# Luego crear tokens como arriba
```

## Recursos Adiciales

- [Documentación SPL Token](https://spl.solana.com/token)
- [Solana CLI Reference](https://docs.solana.com/cli)
- [Faucet DevNet](https://faucet.solana.com/) - Para obtener SOL inicial
