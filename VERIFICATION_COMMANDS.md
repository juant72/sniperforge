# 🔧 SniperForge - Comandos de Verificación Rápida

**Uso**: Para verificar que todas las implementaciones estén funcionando correctamente

---

## 🚀 VERIFICACIÓN CLI BÁSICA

```powershell
# 1. Verificar ayuda general
cargo run --bin sniperforge --help

# 2. Verificar ayuda comando swap-real
cargo run --bin sniperforge test swap-real --help

# 3. Verificar simulación swap-real (SEGURO - no ejecuta transacción real)
cargo run --bin sniperforge test swap-real --network devnet --wallet test-wallet-new.json --amount 0.001
```

---

## 🛡️ VERIFICACIÓN DE SEGURIDAD

```powershell
# 1. Probar sin --network (debe requerir selección explícita)
cargo run --bin sniperforge test swap-real --wallet test-wallet-new.json

# 2. Probar sin --confirm (debe mostrar warnings y no ejecutar)
cargo run --bin sniperforge test swap-real --network devnet --wallet test-wallet-new.json

# 3. Verificar límites de cantidad (debe mostrar warnings específicos por red)
cargo run --bin sniperforge test swap-real --network mainnet --wallet test-wallet-new.json --amount 1.0
```

---

## 💰 VERIFICACIÓN WALLET

```powershell
# 1. Generar nueva wallet para testing
cargo run --bin sniperforge wallet generate --network devnet --output test-nueva.json

# 2. Verificar balance
cargo run --bin sniperforge wallet balance --network devnet test-nueva.json

# 3. Solicitar airdrop DevNet (si necesario)
cargo run --bin sniperforge wallet airdrop --network devnet test-nueva.json
```

---

## 🛡️ VERIFICACIÓN CACHE-FREE TRADING

```powershell
# 1. SIMULACIÓN BÁSICA (SEGURO)
cargo run --bin sniperforge test cache-free-trading --network devnet

# 2. CON WALLET REAL DEVNET (requiere SOL DevNet)
cargo run --bin sniperforge test cache-free-trading --network devnet --wallet test-wallet-new.json

# 3. CON WALLET REAL MAINNET (¡CUIDADO! - Solo para testing)
cargo run --bin sniperforge test cache-free-trading --network mainnet --wallet mainnet-validation-wallet.json
```

---

## 🧪 VERIFICACIÓN SPRINT 1 - SWAP REAL

```powershell
# 1. SIMULACIÓN COMPLETA (SEGURO)
cargo run --bin sniperforge test swap-real --network devnet --wallet test-wallet-new.json --amount 0.001

# 2. EJECUCIÓN REAL DEVNET (requiere SOL DevNet y confirmación explícita)
cargo run --bin sniperforge test swap-real --network devnet --wallet test-wallet-new.json --amount 0.001 --confirm

# 3. VERIFICACIÓN MAINNET (¡CUIDADO! - Solo para testing con cantidades mínimas)
cargo run --bin sniperforge test swap-real --network mainnet --wallet mainnet-validation-wallet.json --amount 0.001 --confirm
```

---

## 📋 COMANDOS DE ESTADO

```powershell
# 1. Estado de la plataforma
cargo run --bin sniperforge status --network devnet

# 2. Configuración actual
cargo run --bin sniperforge config --network devnet

# 3. Verificar balance de cualquier dirección
cargo run --bin sniperforge check-balance --network devnet --address 7BgBvyjrZX8YKHGoM7BXJnK2vhABwxnVUvRSHFHHkLjr
```

---

## ✅ RESULTADOS ESPERADOS

### ✅ **Comando de Ayuda**:
- Debe mostrar toda la ayuda sin errores
- Warnings de seguridad visibles
- Parámetros correctos documentados

### ✅ **Simulación Swap-Real**:
- Debe conectar a Jupiter API
- Debe obtener quote real
- Debe mostrar warnings de seguridad
- NO debe ejecutar transacción sin `--confirm`

### ✅ **Medidas de Seguridad**:
- Debe requerir `--network` explícitamente
- Debe mostrar límites por red
- Debe verificar balance antes de ejecutar
- Debe requerir `--confirm` para transacciones reales

---

## 🚨 TROUBLESHOOTING

### **Error "Network required"**:
- ✅ **CORRECTO**: Añadir `--network devnet` o `--network mainnet`

### **Error "No wallet provided"**:
- ✅ **CORRECTO**: Añadir `--wallet archivo.json`

### **Warning "Add --confirm flag"**:
- ✅ **CORRECTO**: Es la medida de seguridad funcionando

### **Error "Insufficient balance"**:
- 🔧 **Solución**: Solicitar airdrop DevNet o usar cantidad menor

---

## 🎯 ESTADO FINAL ESPERADO

Si todos los comandos funcionan correctamente:

✅ **CLI completamente funcional**  
✅ **Implementaciones reales activas**  
✅ **Medidas de seguridad funcionando**  
✅ **Swap real execution operativo**  
✅ **Sprint 1 objetivos completados**

---

**¡SniperForge está listo para trading real!** 🚀
