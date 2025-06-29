# ⚡ COMANDOS RÁPIDOS - VALIDACIÓN MAINNET

## 🚀 Cuando estés listo para la validación final:

### 1️⃣ Verificar balance después del funding
```bash
cargo run --bin sniperforge wallet balance mainnet-validation-wallet.json
```

### 2️⃣ Test cache-free trading con wallet real
```bash
cargo run --bin sniperforge test cache-free-trading --network mainnet --wallet mainnet-validation-wallet.json
```

### 3️⃣ Ejecutar validación final en Mainnet
```bash
cargo run --bin sniperforge test swap-real --wallet mainnet-validation-wallet.json --network mainnet --amount 0.001 --confirm
```

### 4️⃣ Verificar resultado en Solana Explorer
```bash
# Visitar: https://explorer.solana.com/address/9pMAkWBFY8EWW4DisQDbeLBi5xTcFwh62X3E8guK26zD
```

---

## 💰 Info de Funding

**Address para fondear**: `9pMAkWBFY8EWW4DisQDbeLBi5xTcFwh62X3E8guK26zD`  
**Cantidad recomendada**: 0.015 SOL (~$2.10 USD)  
**Costo real de test**: 0.001 SOL (~$0.14 USD)  

---

## ✅ Estado Actual

- **Sistema**: 100% listo
- **Código**: 100% real data (0% mock)
- **Tests**: Todos pasando 
- **Validación**: Solo falta funding cuando conveniente

**Sprint 1 = 99% COMPLETADO** 🎯
