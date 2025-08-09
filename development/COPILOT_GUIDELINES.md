# 🚫 DIRECTIVAS OBLIGATORIAS - COPILOT GUIDELINES

## ❌ **PROHIBIDO ABSOLUTO:**

### 🚨 **NUNCA HACER:**
1. **NO crear nuevos ejecutables/binarios** sin autorización explícita del usuario
2. **NO generar fake data, placeholders o datos simulados** - SOLO datos reales
3. **NO duplicar código** existente - reutilizar y extender
4. **NO crear archivos temporales** o basura innecesaria
5. **NO inventar APIs** o endpoints que no existen
6. **NO crear múltiples demos** del mismo concepto
7. **NO asumir** - **SIEMPRE VERIFICAR** el estado actual primero

### ✅ **PROTOCOLO OBLIGATORIO:**

#### **ANTES DE CUALQUIER ACCIÓN:**
```
1. VERIFICAR: ¿Qué existe ya? (usar read_file, file_search, grep_search)
2. ANALIZAR: ¿Qué funciona actualmente?
3. REUTILIZAR: ¿Cómo extender lo existente?
4. CONFIRMAR: ¿El usuario autoriza esta acción?
```

#### **PRINCIPIOS DE IMPLEMENTACIÓN:**
```
✅ USAR ejecutables existentes
✅ EXTENDER código actual
✅ INTEGRAR sin duplicar
✅ SOLO datos reales de APIs reales
✅ UNA función = UN propósito
✅ LIMPIAR código redundante
```

#### **DATOS REALES ÚNICAMENTE:**
```
✅ CoinGecko API para precios
✅ Twitter API para sentiment  
✅ Reddit scraping real
✅ Fear & Greed Index real
✅ DEX APIs reales (Jupiter, Raydium, etc.)
❌ CERO simulación o mock data
```

### 🎯 **PROCESO DE VERIFICACIÓN:**

#### **Paso 1: SIEMPRE PREGUNTAR**
- "¿Existe ya algo similar?"
- "¿Puedo usar/extender lo existente?"
- "¿El usuario quiere que cree algo nuevo?"

#### **Paso 2: MOSTRAR OPCIONES**
- Mostrar qué existe actualmente
- Proponer usar/mejorar lo existente
- Solo crear nuevo si es absolutamente necesario

#### **Paso 3: CONFIRMAR ANTES DE ACTUAR**
- Describir exactamente qué haré
- Esperar confirmación del usuario
- Proceder solo con autorización explícita

## 🚀 **EJEMPLOS DE APLICACIÓN CORRECTA:**

### ✅ **CORRECTO:**
```
Usuario: "Probemos el sistema"
Copilot: 
1. Verifico qué demos existen (unified_demo.exe ✅)
2. Uso el demo existente que funciona
3. Muestro resultados reales
4. Pregunto qué aspecto específico quiere ver
```

### ❌ **INCORRECTO:**
```
Usuario: "Probemos el sistema" 
Copilot: Voy a crear un nuevo demo especial...
```

## 📋 **CHECKLIST OBLIGATORIO:**

Antes de cada acción, verificar:
- [ ] ¿Existe algo similar?
- [ ] ¿Puedo reutilizar código existente?
- [ ] ¿Los datos son reales?
- [ ] ¿Tengo autorización del usuario?
- [ ] ¿Estoy duplicando funcionalidad?

---

**🎯 REGLA DORADA: "VERIFICAR → REUTILIZAR → CONFIRMAR → ACTUAR"**

*Estas directivas deben ser consultadas antes de cualquier implementación*
