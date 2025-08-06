# 📄 CONFIGURACIÓN YAML EMPRESARIAL - SniperForge v5.8.1

## 🎯 **VENTAJAS DE YAML SOBRE JSON**

### **Por qué YAML es Superior:**
- ✅ **Legibilidad humana**: Comentarios y estructura clara
- ✅ **Menos verboso**: Sin llaves ni comillas excesivas  
- ✅ **Comentarios integrados**: Documentación en el mismo archivo
- ✅ **Jerarquía visual**: Indentación natural
- ✅ **Soporte para ambientes**: Configuraciones por entorno
- ✅ **Variables de entorno**: Integración automática

### **Comparación JSON vs YAML:**

**JSON (Antes):**
```json
{
  "server": {
    "tcp": {
      "enabled": true,
      "port": 8888
    }
  }
}
```

**YAML (Ahora):**
```yaml
# 📡 Server Configuration
server:
  tcp:
    enabled: true
    port: 8888  # Default TCP port for bot control
```

---

## 🏗️ **ESTRUCTURA DE CONFIGURACIÓN**

### **Archivo Principal: `config/system.yaml`**

```yaml
# 🚀 SniperForge Enterprise Configuration
# Version: 5.8.1
# Environment: development

# 📡 Server Configuration
server:
  tcp:
    enabled: true
    port: 8888
    bind_address: "127.0.0.1"
    max_connections: 100
  
  http:
    enabled: true
    port: 8080
    bind_address: "127.0.0.1"

# 📊 Persistence Configuration  
persistence:
  enabled: true
  directory: "state"
  auto_save: true
  backup_retention_days: 30

# 🤖 Bot Configurations
bots:
  arbitrage:
    enabled: true
    min_profit_threshold: 0.01
    max_position_size: 1000.0
```

---

## 🌍 **CONFIGURACIÓN POR AMBIENTES**

### **Sección `environments` para Overrides:**

```yaml
# 🧪 Environment-Specific Overrides
environments:
  development:
    logging:
      level: "debug"
    network:
      solana:
        mainnet_rpc: "https://api.devnet.solana.com"  # Devnet para desarrollo
    trading:
      risk_management:
        max_daily_loss: 100.0  # Límites bajos para desarrollo
  
  production:
    logging:
      level: "info"
    security:
      api_key_required: true
    trading:
      risk_management:
        max_daily_loss: 5000.0  # Límites altos para producción
  
  testing:
    persistence:
      enabled: false  # No persistir durante tests
    trading:
      risk_management:
        max_daily_loss: 10.0  # Límites mínimos para testing
```

### **Uso de Ambientes:**

```bash
# Ejecutar en desarrollo (por defecto)
.\sniperforge.exe --environment development

# Ejecutar en producción
.\sniperforge.exe --environment production

# Ejecutar en testing
.\sniperforge.exe --environment testing
```

---

## 🔧 **COMANDOS CLI PARA YAML**

### **Cargar Configuración Específica:**
```bash
# Cargar configuración por defecto
.\sniperforge.exe

# Cargar configuración específica
.\sniperforge.exe --config config/custom.yaml

# Cargar con ambiente específico
.\sniperforge.exe --config config/system.yaml --environment production
```

### **Validar Configuración:**
```bash
# Nuevo comando CLI para validar
.\sniperforge-cli.exe validate-config --file config/system.yaml
```

**Salida esperada:**
```
✅ Configuration validation successful:
   📄 File: config/system.yaml
   🌍 Environment: development
   📊 Total sections: 8
   🔧 Applied overrides: 4
   🎯 Ready for deployment
```

### **Ver Configuración Efectiva:**
```bash
# Ver configuración final aplicada
.\sniperforge-cli.exe show-config --environment production
```

**Salida esperada:**
```yaml
# 📋 Effective Configuration (production environment)
server:
  tcp:
    enabled: true
    port: 9999  # Overridden by production environment
  http:
    port: 9998  # Overridden by production environment

security:
  api_key_required: true  # Overridden by production environment
```

---

## 📊 **CONFIGURACIONES AVANZADAS**

### **1. Configuración de Bots por Tipo:**

```yaml
bots:
  # 🎯 Arbitrage Bot Configuration
  arbitrage:
    enabled: true
    strategy:
      min_profit_threshold: 0.01      # 1% minimum profit
      max_position_size: 1000.0       # $1000 max position
      execution_timeout_ms: 5000      # 5 second timeout
    
    exchanges: ["binance", "kraken", "coinbase"]
    pairs: ["BTC/USDT", "ETH/USDT", "SOL/USDT"]
    
    risk_management:
      stop_loss_percentage: 5.0
      max_slippage: 0.02
  
  # ⚡ Flash Loan Bot Configuration  
  flash_loan:
    enabled: false
    strategy:
      min_profit_threshold: 0.005     # 0.5% minimum (lower due to leverage)
      max_loan_amount: 10000.0        # $10k max loan
      
    supported_protocols: ["aave", "compound", "dydx"]
    
    risk_management:
      max_gas_price_gwei: 100
      fallback_enabled: true
```

### **2. Configuración de Red y Conectividad:**

```yaml
network:
  # 🌐 Solana Network Configuration
  solana:
    clusters:
      mainnet:
        rpc: "https://api.mainnet-beta.solana.com"
        ws: "wss://api.mainnet-beta.solana.com"
        commitment: "confirmed"
      
      devnet:
        rpc: "https://api.devnet.solana.com"
        ws: "wss://api.devnet.solana.com"
        commitment: "processed"
    
    timeouts:
      rpc_timeout_seconds: 30
      websocket_timeout_seconds: 10
      retry_attempts: 3
      retry_delay_ms: 1000
  
  # 🔗 External APIs
  external_apis:
    coingecko:
      enabled: true
      api_key: "${COINGECKO_API_KEY}"  # Environment variable
      rate_limit: 50  # requests per minute
    
    dexscreener:
      enabled: true
      timeout_seconds: 15
```

### **3. Variables de Entorno:**

```yaml
# 🔐 Environment Variables Integration
security:
  wallet:
    private_key: "${SOLANA_PRIVATE_KEY}"     # From environment
    address: "${SOLANA_WALLET_ADDRESS}"
  
  api_keys:
    binance_key: "${BINANCE_API_KEY}"
    binance_secret: "${BINANCE_SECRET_KEY}"
    
database:
  connection_string: "${DATABASE_URL}"
  max_connections: 10
  
monitoring:
  webhook_url: "${DISCORD_WEBHOOK_URL}"      # For alerts
  telegram_bot_token: "${TELEGRAM_BOT_TOKEN}"
```

---

## 🔄 **MIGRACIÓN DESDE JSON**

### **Script de Migración Automática:**

```bash
# Convertir configuración JSON existente a YAML
.\sniperforge-cli.exe migrate-config --from config.json --to config/system.yaml
```

### **Proceso de Migración:**

1. **Backup automático** del JSON existente
2. **Conversión** a formato YAML con comentarios
3. **Validación** de la nueva configuración
4. **Prueba** en modo dry-run
5. **Activación** de la nueva configuración

### **Resultado de Migración:**

```
🔄 Configuration Migration Report:
   📄 Source: config.json (142 lines)
   📄 Target: config/system.yaml (89 lines, 45% reduction)
   ✅ All settings preserved
   ✅ Added 23 explanatory comments  
   ✅ Organized into 8 logical sections
   ✅ Ready for environment-specific overrides
```

---

## 📋 **CASOS DE USO PRÁCTICOS**

### **Caso 1: Desarrollo Local**

```yaml
# config/development.yaml
environments:
  development:
    logging:
      level: "debug"
      console_enabled: true
    
    bots:
      arbitrage:
        min_profit_threshold: 0.001  # Muy bajo para testing
    
    network:
      solana:
        mainnet_rpc: "https://api.devnet.solana.com"  # Devnet
```

### **Caso 2: Producción Escalable**

```yaml
# config/production.yaml  
environments:
  production:
    server:
      tcp:
        max_connections: 1000  # Alto throughput
        
    persistence:
      backup_retention_days: 90  # Retención extendida
      
    security:
      api_key_required: true
      rate_limiting:
        requests_per_minute: 200
        
    monitoring:
      alerts_enabled: true
      webhook_url: "${PRODUCTION_WEBHOOK}"
```

### **Caso 3: Testing Automatizado**

```yaml
# config/testing.yaml
environments:
  testing:
    persistence:
      enabled: false  # No persistir durante tests
      
    bots:
      arbitrage:
        execution_timeout_ms: 100  # Tests rápidos
        
    network:
      timeouts:
        rpc_timeout_seconds: 5  # Timeouts cortos
```

---

## 🎯 **VENTAJAS OPERACIONALES**

### **Para Desarrolladores:**
- 🚀 **Configuración por ambiente** - sin duplicar archivos
- 📝 **Comentarios integrados** - documentación en vivo  
- 🔧 **Validación automática** - errores detectados temprano
- 🎨 **Syntax highlighting** - mejor experiencia de edición

### **Para DevOps:**
- 🌍 **Gestión multi-ambiente** - desarrollo, staging, producción
- 🔐 **Variables de entorno** - secretos seguros
- 📊 **Configuración versionada** - cambios rastreables
- 🔄 **Despliegue simplificado** - un archivo, múltiples ambientes

### **Para Operaciones:**
- 🛡️ **Configuración robusta** - validación estricta
- 📈 **Monitoreo integrado** - métricas y alertas configurables
- 🔧 **Mantenimiento fácil** - cambios sin recompilación
- 📋 **Auditabilidad completa** - historial de cambios

---

## 🚀 **MIGRACIÓN INMEDIATA**

### **Pasos para Adoptar YAML:**

1. **Crear configuración base:**
   ```bash
   cp config/system.yaml config/my-system.yaml
   ```

2. **Personalizar para tu entorno:**
   ```yaml
   # Editar config/my-system.yaml
   server:
     tcp:
       port: 8889  # Tu puerto personalizado
   ```

3. **Validar configuración:**
   ```bash
   .\sniperforge-cli.exe validate-config --file config/my-system.yaml
   ```

4. **Ejecutar con nueva configuración:**
   ```bash
   .\sniperforge.exe --config config/my-system.yaml --environment development
   ```

### **Resultado:**
- ✅ **Configuración más limpia** y mantenible
- ✅ **Flexibilidad por ambiente** sin duplicación
- ✅ **Documentación integrada** en el archivo
- ✅ **Validación automática** de configuraciones
- ✅ **Escalabilidad empresarial** lista para producción

---

**¡El futuro de la configuración es YAML!** 🎉
