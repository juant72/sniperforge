# 🧪 Estrategia de Testing en DevNet - Guía Completa

> **Objetivo:** Probar todos los comandos CLI y funcionalidades sin perder dinero real

## 📋 **RESUMEN DE LIMITACIONES DEVNET**

### ❌ **Problemas Identificados:**
1. **Jupiter API en DevNet:** Rutas limitadas, muchos 404 "Route not found"
2. **Liquidez Limitada:** Pocos DEXs operativos en DevNet
3. **Rate Limits:** 100 requests/10s por IP
4. **Airdrops:** Máximo 5 SOL por request

### ✅ **Soluciones Estratégicas:**

## 🎯 **ESTRATEGIA 1: Testing con Tokens Específicos de DevNet**

### **Tokens Confiables en DevNet:**
```bash
# Tokens que SÍ tienen liquidez en DevNet
SOL (nativo): So11111111111111111111111111111111111111112
USDC-Dev: 4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU
WSOL: So11111111111111111111111111111111111111112

# Tokens de prueba estables
USDT-Dev: Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB
```

### **Comando Seguro para Testing:**
```bash
# 1. DCA Strategy con cantidades pequeñas
./target/release/sniperforge strategy-run \
  --type dca \
  --config configs/strategies/dca_devnet_safe.json \
  --network devnet

# 2. Order Create con SOL → USDC (más estable)
./target/release/sniperforge order-create \
  --type stop-loss \
  --token SOL \
  --amount 0.001 \
  --trigger 100 \
  --network devnet
```

## 🎯 **ESTRATEGIA 2: Local Testing con solana-test-validator**

### **Setup Local Validator:**
```powershell
# Terminal 1: Iniciar validator local
solana-test-validator --reset

# Terminal 2: Configurar CLI para localhost
solana config set --url localhost

# Terminal 3: Nuestras pruebas
./target/release/sniperforge strategy-run --network devnet
```

### **Ventajas del Local Validator:**
- ✅ **Sin rate limits**
- ✅ **SOL ilimitado**
- ✅ **Control total del entorno**
- ✅ **Sin dependencia de Jupiter DevNet**

## 🎯 **ESTRATEGIA 3: Mock Testing Mode**

### **Implementar Modo de Prueba:**
```rust
// En jupiter_config.rs
impl JupiterConfig {
    pub fn devnet_mock() -> Self {
        Self {
            base_url: "http://localhost:8899".to_string(), // Local validator
            enable_mocks: true,
            // ... otras configuraciones
        }
    }
}
```

## 🎯 **ESTRATEGIA 4: Configuraciones Específicas para DevNet**

### **dca_devnet_safe.json:**
```json
{
  "strategy_id": "dca_devnet_test",
  "from_token": "SOL",
  "to_token": "USDC-Dev",
  "total_amount": 0.01,
  "intervals": 3,
  "interval_duration_seconds": 10,
  "slippage_tolerance": 0.02
}
```

### **Parámetros Ultra-Seguros:**
- **Cantidad máxima:** 0.01 SOL ($2-3 USD)
- **Slippage alto:** 2% (para tolerar baja liquidez)
- **Intervalos cortos:** 10 segundos
- **Timeouts largos:** 60 segundos

## 🎯 **ESTRATEGIA 5: Monitoring y Logging Detallado**

### **Logging Mejorado:**
```rust
// En strategy_executor.rs
info!("DevNet Test Mode: Using {} SOL (${:.2} USD)", amount, amount * sol_price);
warn!("DevNet: Limited liquidity, expect some failures");
debug!("Jupiter route attempt {}/{}", attempt, max_attempts);
```

### **Métricas de Éxito:**
- ✅ **Wallet creation:** 100%
- ✅ **Airdrop request:** 100%
- ✅ **CLI command parsing:** 100%
- ⚠️ **Jupiter routing:** 30-50% (esperado en DevNet)
- ✅ **Error handling:** 100%

## 🚀 **PLAN DE TESTING PROGRESIVO**

### **Fase 1: Validación Básica (DevNet)**
```bash
# 1. Test wallet operations
./target/release/sniperforge wallet generate --network devnet
./target/release/sniperforge wallet balance --network devnet

# 2. Test basic connectivity
./target/release/sniperforge test solana --network devnet
./target/release/sniperforge test jupiter --network devnet
```

### **Fase 2: Trading Logic (Local Validator)**
```bash
# Start local validator first
solana-test-validator --reset

# Test with unlimited resources
./target/release/sniperforge strategy-run --config configs/strategies/dca_local.json
```

### **Fase 3: Production Readiness (Mainnet Minimal)**
```bash
# Only when everything works perfectly
# Use minimal amounts: 0.00001 SOL
./target/release/sniperforge strategy-run \
  --type dca \
  --config configs/strategies/dca_mainnet_minimal.json \
  --network mainnet
```

## ⚡ **COMANDOS DE VERIFICACIÓN RÁPIDA**

### **Pre-Test Checklist:**
```powershell
# 1. Verify compilation
cargo check

# 2. Verify DevNet connectivity
solana config set --url devnet
solana epoch-info

# 3. Verify wallet balance
solana balance

# 4. Test Jupiter quote (manually)
curl "https://quote-api.jup.ag/v6/quote?inputMint=So11111111111111111111111111111111111111112&outputMint=4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU&amount=1000000"
```

## 🔄 **ALTERNATIVAS A JUPITER EN SOLANA**

### ❌ **El Problema:** 
```
2025-07-03T08:40:22.521867Z ERROR DCA interval 3 failed: Trading error: Quote failed: Quote API request failed: 404 Not Found - Route not found
```

### ✅ **La Solución: Jupiter NO es la única opción**

**Principales DEXs/Aggregators en Solana:**

| DEX/Aggregator | API Disponible | DevNet Support | Ventajas |
|----------------|----------------|----------------|-----------|
| **Jupiter** | ✅ V6 API | ⚠️ Limitado | Mejor aggregator general |
| **Orca** | ✅ CLMM API | ✅ Completo | CLMM concentrado, muy estable |
| **Raydium** | ✅ V4 API | ✅ Bueno | Alto volumen, AMM clásico |
| **Serum** | ✅ Order Book | ✅ Disponible | Order book real |
| **1inch** | ✅ V5 API | ⚠️ Limitado | Aggregator multiplataforma |
| **SPL Token-Swap** | ✅ Direct | ✅ Nativo | Programa nativo de Solana |

### **🎯 ESTRATEGIA MULTI-DEX:**

#### **1. Implementar Fallback Chain:**
```rust
// En jupiter_client.rs - agregar fallback a múltiples DEXs
pub async fn get_quote_with_fallback(
    &self,
    request: &QuoteRequest,
) -> Result<QuoteResponse> {
    // 1. Intentar Jupiter primero
    if let Ok(quote) = self.get_quote(request).await {
        return Ok(quote);
    }
    
    // 2. Fallback a Orca
    if let Ok(quote) = self.orca_client.get_quote(request).await {
        return Ok(quote);
    }
    
    // 3. Fallback a Raydium
    if let Ok(quote) = self.raydium_client.get_quote(request).await {
        return Ok(quote);
    }
    
    // 4. Error solo si todos fallan
    Err(anyhow::anyhow!("All DEX routes failed"))
}
```

#### **2. Configuración Multi-DEX:**
```json
{
  "strategy_id": "dca_multi_dex_test",
  "from_token": "SOL",
  "to_token": "USDC",
  "total_amount": 0.01,
  "intervals": 3,
  "interval_duration_seconds": 10,
  "slippage_tolerance": 0.02,
  "dex_preference": ["orca", "raydium", "jupiter"],
  "enable_multi_dex_fallback": true
}
```

## 🔧 **IMPLEMENTACIÓN PRÁCTICA**

### **ESTRATEGIA 2A: Usar Orca Directamente (Más Estable en DevNet)**

#### **Agregar Orca Client:**
```rust
// En src/shared/orca_client.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct OrcaClient {
    client: Client,
    base_url: String,
}

impl OrcaClient {
    pub fn new(network: &str) -> Self {
        let base_url = match network {
            "mainnet" => "https://api.orca.so",
            "devnet" => "https://api.devnet.orca.so",
            _ => "https://api.devnet.orca.so",
        }.to_string();
        
        Self {
            client: Client::new(),
            base_url,
        }
    }
    
    pub async fn get_quote(&self, request: &QuoteRequest) -> Result<OrcaQuote> {
        let url = format!(
            "{}/v1/quote?inputMint={}&outputMint={}&amount={}",
            self.base_url, request.inputMint, request.outputMint, request.amount
        );
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
            
        let quote: OrcaQuote = response.json().await?;
        Ok(quote)
    }
}
```

#### **Configuración DevNet con Orca:**
```json
{
  "strategy_id": "dca_orca_devnet_test",
  "from_token": "So11111111111111111111111111111111111111112",
  "to_token": "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
  "total_amount": 0.01,
  "intervals": 3,
  "interval_duration_seconds": 10,
  "slippage_tolerance": 0.02,
  "preferred_dex": "orca",
  "enable_fallback": true
}
```

### **ESTRATEGIA 2B: SPL Token-Swap (Programa Nativo)**

#### **Usar SPL Token-Swap Directamente:**
```rust
// En src/shared/spl_swap_client.rs
use solana_client::rpc_client::RpcClient;
use spl_token_swap::processor::SwapInstruction;

pub struct SPLSwapClient {
    rpc_client: RpcClient,
}

impl SPLSwapClient {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            rpc_client: RpcClient::new(rpc_url.to_string()),
        }
    }
    
    pub async fn execute_swap(&self, params: SwapParams) -> Result<SwapResult> {
        // Implementar swap directo usando SPL Token-Swap
        // Sin dependencia de APIs externas
    }
}
```

### **ESTRATEGIA 2C: Local Test Validator con Pools Preconfigurados**

#### **Setup de Pools Locales:**
```powershell
# Terminal 1: Crear validator local con pools
solana-test-validator --reset --bpf-program TokenSwap TokenSwap.so --quiet

# Terminal 2: Crear pools de prueba
spl-token-swap create-pool --pool-token-mint POOL_MINT --fee-numerator 25 --fee-denominator 10000

# Terminal 3: Nuestras pruebas sin rate limits
cargo run --bin sniperforge -- strategy-run --config configs/strategies/dca_local.json
```

## 🛡️ **SAFETY MEASURES**

### **Protecciones Implementadas:**
1. **Balance checks** antes de cada trade
2. **Slippage limits** configurables
3. **Timeout handling** para requests largos
4. **Error recovery** con reintentos exponenciales
5. **Dry-run mode** para analysis sin ejecución

### **Límites de Seguridad:**
```rust
const MAX_DEVNET_AMOUNT_SOL: f64 = 0.1; // $20-30 USD max
const MAX_RETRIES: u32 = 3;
const TIMEOUT_SECONDS: u64 = 30;
```

## 📊 **MÉTRICAS DE ÉXITO ESPERADAS EN DEVNET**

| Componente | Tasa de Éxito Esperada | Nota |
|------------|------------------------|------|
| Wallet Operations | 100% | Siempre funciona |
| RPC Calls | 95% | Rate limits ocasionales |
| Jupiter Quotes | 40% | Liquidez limitada |
| Swap Execution | 20% | Solo para pares estables |
| Error Handling | 100% | Crítico para production |

## 🎯 **PRÓXIMOS PASOS PRIORIZADOS**

### **🚀 ALTA PRIORIDAD (Inmediato):**
1. **✅ COMPLETADO: Configuraciones específicas para DevNet** 
2. **✅ COMPLETADO: CLI strategy-run funcional**
3. **🔄 EN PROGRESO: Implementar Orca client como fallback**
4. **⭐ SIGUIENTE: Probar order-create command**
5. **⭐ SIGUIENTE: Probar execution-optimize command**

### **🛠️ MEDIA PRIORIDAD (Esta semana):**
6. **Implementar multi-DEX fallback chain**
7. **Agregar SPL Token-Swap como opción nativa**
8. **Local validator testing con pools preconfigurados**
9. **Modo mock para testing sin dependencias externas**

### **📊 BAJA PRIORIDAD (Próximas iteraciones):**
10. **Integrar Raydium API**
11. **Agregar Serum order book integration**
12. **Documentar casos de éxito/fallo por DEX**
13. **Métricas de success rate por proveedor**

---

**💡 Conclusión Actualizada:** 

El error "404 Route not found" de Jupiter en DevNet es esperado y NO es un bloqueador. **Jupiter NO es la única opción** - tenemos múltiples alternativas:

1. **✅ Orca**: Más estable en DevNet, excelente CLMM
2. **✅ Raydium**: Buen soporte DevNet, alto volumen
3. **✅ SPL Token-Swap**: Programa nativo, sin APIs externas
4. **✅ Local Validator**: Control total del entorno

**Estrategia Recomendada**: Implementar fallback chain (Jupiter → Orca → Raydium → SPL) para máxima compatibilidad sin riesgos financieros.
