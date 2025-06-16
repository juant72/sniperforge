# Configuración Global de SniperForge

## 📋 Estructura de Configuración

SniperForge utiliza un sistema de configuración jerárquico basado en archivos TOML que permite flexibilidad y reutilización.

## 🗂 Archivos de Configuración

### Configuración Global (`config/global.toml`)

```toml
[solana]
# URL del RPC de Solana
rpc_url = "https://api.mainnet-beta.solana.com"
# Nivel de commitment para transacciones
commitment = "confirmed"
# Timeout para requests RPC (segundos)
timeout = 30
# Número máximo de conexiones concurrentes
max_connections = 100

[logging]
# Nivel de logging: trace, debug, info, warn, error
level = "info"
# Formato: json, text
format = "json"
# Archivo de log (opcional, si no se especifica usa stdout)
file = "/var/log/sniperforge/app.log"
# Rotación de logs
rotation = "daily"
# Retención de logs (días)
retention_days = 30

[metrics]
# Puerto para servidor de métricas Prometheus
port = 9090
# Intervalo de recolección (segundos)
collection_interval = 60
# Métricas habilitadas
enabled = true

[database]
# URL de conexión a la base de datos (opcional)
url = "postgresql://user:pass@localhost/sniperforge"
# Pool de conexiones máximo
max_connections = 20
# Timeout de conexión (segundos)
timeout = 10

[security]
# Directorio para almacenar claves privadas encriptadas
keystore_path = "/secure/keystore"
# Algoritmo de encriptación
encryption_algorithm = "AES-256-GCM"
```

### Configuración del Raydium LP Sniper (`config/bots/raydium-lp-sniper.toml`)

```toml
[detector]
# Intervalo de polling para nuevos pools (milisegundos)
polling_interval = 1000
# Programas de Raydium a monitorear
raydium_programs = [
    "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",  # Raydium AMM V4
    "5quBtoiQqxF9Jv6KYKctB59NT3gtJD2Py818b2LDjSap",  # Raydium AMM V5
]
# Filtrar por market authority específica (opcional)
market_authority = []
# Máximo número de pools a procesar simultáneamente
max_concurrent_pools = 50

[extractor]
# Timeout para obtener metadatos (segundos)
metadata_timeout = 5
# Intentos de retry en caso de fallo
retry_attempts = 3
# Delay entre intentos (milisegundos)
retry_delay = 500
# Verificar supply del token
verify_token_supply = true
# Verificar metadatos del token
verify_token_metadata = true

[filter]
# Liquidez mínima en SOL
min_liquidity_sol = 10.0
# Liquidez máxima en SOL (para evitar pools demasiado grandes)
max_liquidity_sol = 10000.0
# Supply máximo del token
max_token_supply = 1000000000
# Supply mínimo del token
min_token_supply = 1000000
# Tiempo máximo desde creación del pool (segundos)
max_pool_age = 300
# Verificar honeypot
check_honeypot = true
# Blacklist de tokens conocidos como scam
token_blacklist = [
    "So11111111111111111111111111111111111111112",  # Wrapped SOL (ejemplo)
]
# Whitelist de tokens seguros (opcional)
token_whitelist = []
# Verificar ownership del token
verify_token_ownership = true
# Ratio mínimo de liquidez SOL/Token
min_sol_ratio = 0.1

[scorer]
# Peso para liquidez inicial en el score (0.0 - 1.0)
liquidity_weight = 0.3
# Peso para supply del token en el score (0.0 - 1.0)
supply_weight = 0.2
# Peso para tiempo desde creación en el score (0.0 - 1.0)
age_weight = 0.2
# Peso para ratio SOL/Token en el score (0.0 - 1.0)
ratio_weight = 0.3
# Score mínimo para ejecutar compra (0.0 - 1.0)
min_score_threshold = 0.6
# Multiplicador para ajuste dinámico del score
dynamic_multiplier = 1.2

[risk]
# Porcentaje máximo del capital total por trade (0.0 - 1.0)
max_position_size_pct = 0.02
# Número máximo de posiciones concurrentes
max_concurrent_positions = 5
# Capital mínimo a mantener como reserva (SOL)
min_reserve_sol = 5.0
# Stop loss por defecto (porcentaje)
default_stop_loss_pct = 0.20
# Take profit por defecto (porcentaje)
default_take_profit_pct = 2.0
# Trailing stop habilitado
trailing_stop_enabled = true
# Porcentaje para trailing stop
trailing_stop_pct = 0.10
# Máximo drawdown permitido antes de pausar trading
max_drawdown_pct = 0.15

[execution]
# Slippage máximo aceptable (porcentaje)
max_slippage_pct = 0.05
# Prioridad de fee para transacciones (lamports)
priority_fee = 10000
# Fee máximo total aceptable (lamports)
max_total_fee = 50000
# Tiempo máximo de espera para confirmación (milisegundos)
confirmation_timeout = 30000
# Número de intentos de retry para transacciones fallidas
retry_attempts = 3
# Usar Jito para MEV protection
use_jito = true
# Tip para Jito (lamports)
jito_tip = 50000

[exit_strategy]
# Estrategia de salida: "time_based", "price_based", "hybrid"
strategy = "hybrid"
# Tiempo máximo para mantener posición (segundos)
max_hold_time = 3600
# Verificar volumen antes de salida
check_volume_before_exit = true
# Volumen mínimo para salida exitosa (SOL)
min_exit_volume = 5.0
# Usar partial exits
partial_exits_enabled = true
# Porcentaje de la posición para primera salida parcial
first_exit_pct = 0.5
# Precio objetivo para primera salida (multiplicador)
first_exit_target = 1.5

[monitoring]
# Intervalo para verificar posiciones abiertas (milisegundos)
position_check_interval = 5000
# Enviar alertas por Discord/Telegram
alerts_enabled = true
# Webhook para notificaciones
webhook_url = "https://discord.com/api/webhooks/your_webhook_url"
# Nivel mínimo de alerta: "info", "warning", "error"
min_alert_level = "warning"

[simulation]
# Modo simulación habilitado (no ejecuta trades reales)
enabled = false
# Capital virtual para simulación (SOL)
virtual_balance = 100.0
# Incluir fees en simulación
include_fees = true
# Incluir slippage en simulación
include_slippage = true
```

## 🔧 Variables de Entorno

Las siguientes variables de entorno pueden sobrescribir la configuración:

```bash
# Configuración de Solana
SOLANA_RPC_URL="https://api.mainnet-beta.solana.com"
SOLANA_COMMITMENT="confirmed"

# Logging
LOG_LEVEL="info"
LOG_FORMAT="json"

# Security
PRIVATE_KEY_PATH="/secure/keypair.json"
KEYSTORE_PASSWORD="your_secure_password"

# Bot específico
RAYDIUM_SNIPER_ENABLED="true"
RAYDIUM_SNIPER_MAX_POSITION_SIZE="0.02"
RAYDIUM_SNIPER_MIN_LIQUIDITY="10.0"

# Notificaciones
DISCORD_WEBHOOK_URL="https://discord.com/api/webhooks/..."
TELEGRAM_BOT_TOKEN="your_telegram_token"
TELEGRAM_CHAT_ID="your_chat_id"
```

## 📋 Validación de Configuración

### Reglas de Validación

```toml
# Ejemplo de validaciones automáticas
[validation]
# El capital total no puede ser menor que la reserva mínima
capital_vs_reserve = true
# La suma de pesos en scorer debe ser 1.0
scorer_weights_sum = true
# Max position size no puede ser mayor a 0.1 (10%)
max_position_size_limit = 0.1
# Timeout de confirmación debe ser realista
min_confirmation_timeout = 5000
max_confirmation_timeout = 120000
```

## 🔄 Configuración Dinámica

Algunas configuraciones pueden actualizarse en tiempo real sin reiniciar el bot:

### Parámetros Dinámicos
- Niveles de logging
- Parámetros de riesgo
- Thresholds de scoring
- Intervalos de polling
- Configuración de alertas

### Comando para Actualización
```bash
# Actualizar configuración específica
curl -X POST http://localhost:8080/config/update \
  -H "Content-Type: application/json" \
  -d '{"risk.max_position_size_pct": 0.015}'
```

## 🛡 Seguridad en Configuración

### Encriptación de Datos Sensibles
- Claves privadas siempre encriptadas
- Webhooks y tokens almacenados de forma segura
- Configuración de permisos restrictivos

### Mejores Prácticas
```bash
# Permisos recomendados para archivos de configuración
chmod 600 config/*.toml
chmod 700 config/

# Ownership correcto
chown sniperforge:sniperforge config/ -R
```

## 📊 Ejemplos de Configuración por Entorno

### Desarrollo
```toml
[solana]
rpc_url = "https://api.devnet.solana.com"

[simulation]
enabled = true
virtual_balance = 10.0

[logging]
level = "debug"
```

### Testing
```toml
[solana]
rpc_url = "https://api.testnet.solana.com"

[risk]
max_position_size_pct = 0.001
max_concurrent_positions = 1

[simulation]
enabled = false
```

### Producción
```toml
[solana]
rpc_url = "https://api.mainnet-beta.solana.com"

[risk]
max_position_size_pct = 0.02
max_concurrent_positions = 5

[monitoring]
alerts_enabled = true

[logging]
level = "info"
file = "/var/log/sniperforge/production.log"
```

Esta estructura de configuración garantiza flexibilidad, seguridad y facilidad de mantenimiento para todos los bots en SniperForge.
