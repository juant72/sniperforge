# 📚 FUENTES OFICIALES PARA TOKENS EN SOLANA DEVNET

## **❌ Lo que NO hay (y por qué no encuentras info centralizada)**

**DevNet NO tiene un registro central de tokens preexistentes** como MainNet. La razón es simple:

1. **DevNet es para desarrollo/testing**, no producción
2. **Los tokens "oficiales" (USDC, USDT, RAY) NO existen en DevNet**
3. **Cada desarrollador crea sus propios tokens de prueba**
4. **Los tokens se eliminan cuando se reinicia el cluster**

## **✅ Fuentes Oficiales de Documentación**

### **1. Documentación Principal de Solana**
- **URL**: https://docs.solana.com/
- **Sección clave**: https://docs.solana.com/developing/test-validator
- **Qué encontrarás**: Instrucciones para crear test validator local

### **2. SPL Token Program Documentation**
- **URL**: https://spl.solana.com/token
- **Qué encontrarás**: 
  - Cómo crear tokens con `spl-token create-token`
  - Ejemplos completos de minting y transferencias
  - Configuración para diferentes clusters

### **3. Solana CLI Reference**
- **URL**: https://docs.anza.xyz/cli
- **Qué encontrarás**: Comandos completos para manejo de tokens

### **4. Jupiter Documentation**
- **URL**: https://station.jup.ag/ → https://dev.jup.ag/docs/
- **Importante**: Jupiter funciona en DevNet, pero **solo puede rutear tokens que realmente existen**
- **API de tokens**: https://dev.jup.ag/docs/token-api/

### **5. Orca Documentation**
- **URL**: https://orca-so.gitbook.io/orca-developer-portal/
- **Importante**: Orca tiene pools limitados en DevNet

## **🛠️ Herramientas Oficiales para DevNet**

### **1. Solana Faucet (SOL únicamente)**
- **URL**: https://faucet.solana.com/
- **Límite**: 2 SOL cada 8 horas
- **Nota**: Solo para SOL nativo, NO para tokens SPL

### **2. Solana Explorer DevNet**
- **URL**: https://explorer.solana.com/?cluster=devnet
- **Uso**: Verificar transacciones y cuentas en DevNet

### **3. spl-token CLI (Tool Oficial)**
```bash
# Instalar
cargo install spl-token-cli

# Usar en DevNet
solana config set --url https://api.devnet.solana.com
spl-token create-token --decimals 6
spl-token create-account <TOKEN_MINT>
spl-token mint <TOKEN_MINT> 1000
```

## **🎯 LA VERDAD: Por qué no hay "lista oficial" de tokens DevNet**

### **Filosofía de DevNet**
```
MainNet: Tokens permanentes con valor real
DevNet:   Tokens temporales para desarrollo
TestNet:  Similar a DevNet, más experimental  
LocalNet: Tu propio cluster privado
```

### **Workflow Oficial Recomendado**

1. **Para desarrollo básico**: Crear tokens propios con `spl-token`
2. **Para testing avanzado**: Usar `solana-test-validator` local
3. **Para producción**: Migrar a MainNet con tokens reales

## **📋 Lista de Tokens GARANTIZADOS en DevNet**

```json
{
  "tokens_nativos": {
    "SOL": {
      "mint": "So11111111111111111111111111111111111111112",
      "disponible": "✅ Siempre",
      "fuente": "Nativo de Solana"
    },
    "wSOL": {
      "mint": "So11111111111111111111111111111111111111112", 
      "disponible": "✅ Siempre",
      "fuente": "Wrapped SOL (mismo que SOL)"
    }
  },
  "tokens_mainnet_NO_disponibles": {
    "USDC": "❌ EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "USDT": "❌ Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", 
    "RAY":  "❌ 4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",
    "nota": "Estos existen en MainNet pero NO en DevNet"
  }
}
```

## **🔍 Cómo Descubrir Qué Tokens Existen en DevNet**

### **Método 1: RPC Query Programático**
```rust
// Verificar si un token existe
let client = RpcClient::new("https://api.devnet.solana.com");
match client.get_account(&token_mint_pubkey) {
    Ok(_) => println!("✅ Token existe"),
    Err(_) => println!("❌ Token no existe")
}
```

### **Método 2: CLI Check**
```bash
solana account <TOKEN_MINT_ADDRESS> --url devnet
```

### **Método 3: Explorer Web**
```
https://explorer.solana.com/address/<TOKEN_MINT>?cluster=devnet
```

## **🚨 ERRORES COMUNES y Por Qué Ocurren**

### **Error: "IncorrectProgramId"**
```
Causa: Intentar usar mint address de MainNet en DevNet
Solución: Usar solo mints que existan en DevNet
```

### **Error: "InvalidAccountData"**  
```
Causa: Token account no inicializado
Solución: Ejecutar spl-token create-account primero
```

### **Error: Jupiter devuelve rutas pero falla el swap**
```
Causa: Jupiter API cotiza rutas teóricas, no reales
Solución: Verificar que tokens existan antes de swap
```

## **📖 GUÍAS OFICIALES PASO A PASO**

### **1. Solana Cookbook (Community)**
- **URL**: https://solanacookbook.com/
- **Secciones relevantes**: Token Creation, DevNet Setup

### **2. Anchor Book**
- **URL**: https://book.anchor-lang.com/
- **Relevante para**: Desarrollo de programas que manejen tokens

### **3. Metaplex Docs (para NFTs)**
- **URL**: https://docs.metaplex.com/
- **Relevante para**: Token metadata y NFTs

## **💡 ALTERNATIVAS PROFESIONALES**

### **1. Usar Test Validator Local**
```bash
solana-test-validator --reset
# Cluster privado, control total, tokens persistentes
```

### **2. Fork MainNet para Testing**
```bash  
solana-test-validator --clone-account <USDC_MINT> --url mainnet-beta
# Clonar tokens reales para testing
```

### **3. Usar TestNet en lugar de DevNet**
```bash
solana config set --url testnet
# Similar a DevNet pero diferente cluster
```

## **🎯 CONCLUSIÓN**

**NO existe una "lista central" de tokens DevNet** porque:

1. **Por diseño**: DevNet es para crear tus propios tokens
2. **Por naturaleza**: Los tokens son temporales y específicos de cada desarrollador  
3. **Por propósito**: Fomenta el aprendizaje de creación de tokens

**La documentación oficial te enseña a CREAR tokens, no a consumir una lista preexistente.**

**Tu approach correcto fue**: Crear tokens propios y configurar tu engine para usarlos.

## **📚 RECURSOS ADICIONALES**

- **Discord Oficial Solana**: https://discord.gg/solana
- **Stack Exchange**: https://solana.stackexchange.com/
- **GitHub Solana Program Library**: https://github.com/solana-labs/solana-program-library
- **Anza (Agave) Docs**: https://docs.anza.xyz/
