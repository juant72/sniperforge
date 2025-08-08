# SniperForge Enterprise - Guía de Inicio Rápido

## 🚀 Introducción

Esta guía te permitirá comenzar a usar SniperForge Enterprise en menos de 10 minutos, con ejemplos prácticos y configuraciones listas para producción.

## 📋 Prerrequisitos

### Software Requerido
- **Rust**: >= 1.70.0 (`rustup update`)
- **Node.js**: >= 18.0.0 (para SDK TypeScript)
- **Python**: >= 3.9 (para ejemplos Python)
- **Git**: Para clonación y actualizaciones

### Recursos del Sistema
- **RAM**: Mínimo 4GB, recomendado 8GB+
- **CPU**: 4+ cores recomendado
- **Almacenamiento**: 2GB espacio libre
- **Red**: Conexión estable a internet

## ⚡ Instalación Rápida

### 1. Clonar y Compilar

```bash
# Clonar repositorio
git clone https://github.com/juant72/sniperforge.git
cd sniperforge

# Compilar en modo optimizado
cargo build --release

# Verificar instalación
cargo test --workspace
```

### 2. Configuración Básica

```bash
# Crear configuración inicial
cp config/template.json config/config.json

# Crear wallet de prueba
cp test_wallet.json wallet.json

# Verificar configuración
cargo run --bin sniperforge -- --validate-config
```

## 🎯 Primer Bot en 5 Minutos

### Paso 1: Iniciar el Sistema

```bash
# Terminal 1: Iniciar servidor de control
cargo run --bin sniperforge -- \
  --config config.json \
  --wallet wallet.json \
  --mode server \
  --server-port 8888
```

### Paso 2: Conectar y Crear Bot (TypeScript)

```typescript
// quick-start.ts
import { TCPClient } from '@sniperforge/sdk';

async function quickStart() {
    // Conectar al servidor
    const client = new TCPClient('localhost', 8888);
    await client.connect();
    console.log('✅ Conectado a SniperForge Enterprise');

    // Crear configuración de bot
    const config = {
        bot_type: 'LiquiditySniper',
        trading_pairs: ['BTC/USDT', 'ETH/USDT'],
        max_position_size: 100.0,
        risk_level: 'Conservative',
        enable_ai_analysis: true
    };

    // Crear bot
    const createResponse = await client.sendCommand({
        CreateBot: {
            bot_type: 'LiquiditySniper',
            config: config
        }
    });

    const botId = createResponse.BotCreated.bot_id;
    console.log(`🤖 Bot creado: ${botId}`);

    // Iniciar bot
    await client.sendCommand({
        StartBot: {
            bot_id: botId,
            config: config
        }
    });

    console.log(`🚀 Bot iniciado y funcionando`);

    // Monitorear bot cada 10 segundos
    setInterval(async () => {
        const metrics = await client.sendCommand({
            GetBotMetrics: { bot_id: botId }
        });

        console.log(`📊 PnL: $${metrics.BotMetrics.trading.total_pnl_usd.toFixed(2)}`);
        console.log(`📈 Trades: ${metrics.BotMetrics.trading.trades_executed}`);
    }, 10000);
}

quickStart().catch(console.error);
```

### Paso 3: Ejecutar el Cliente

```bash
# Instalar dependencias TypeScript
npm install @sniperforge/sdk

# Ejecutar cliente
npx ts-node quick-start.ts
```

## 🐍 Ejemplo Python Alternativo

```python
# quick-start.py
import json
import socket
import time

class SniperForgeClient:
    def __init__(self, host='localhost', port=8888):
        self.host = host
        self.port = port
        self.socket = None
    
    def connect(self):
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.socket.connect((self.host, self.port))
        print("✅ Conectado a SniperForge Enterprise")
    
    def send_command(self, command):
        json_cmd = json.dumps(command) + '\n'
        self.socket.send(json_cmd.encode())
        response = self.socket.recv(4096).decode()
        return json.loads(response)
    
    def create_and_start_bot(self):
        # Crear bot
        config = {
            'bot_type': 'LiquiditySniper',
            'trading_pairs': ['BTC/USDT'],
            'max_position_size': 100.0,
            'risk_level': 'Conservative'
        }
        
        create_response = self.send_command({
            'CreateBot': {
                'bot_type': 'LiquiditySniper',
                'config': config
            }
        })
        
        bot_id = create_response['BotCreated']['bot_id']
        print(f"🤖 Bot creado: {bot_id}")
        
        # Iniciar bot
        self.send_command({
            'StartBot': {
                'bot_id': bot_id,
                'config': config
            }
        })
        
        print("🚀 Bot iniciado")
        return bot_id
    
    def monitor_bot(self, bot_id):
        while True:
            try:
                metrics = self.send_command({
                    'GetBotMetrics': {'bot_id': bot_id}
                })
                
                pnl = metrics['BotMetrics']['trading']['total_pnl_usd']
                trades = metrics['BotMetrics']['trading']['trades_executed']
                
                print(f"📊 PnL: ${pnl:.2f} | Trades: {trades}")
                time.sleep(10)
                
            except KeyboardInterrupt:
                print("\n🛑 Deteniendo monitoreo...")
                break

# Uso
if __name__ == "__main__":
    client = SniperForgeClient()
    client.connect()
    
    bot_id = client.create_and_start_bot()
    client.monitor_bot(bot_id)
```

## 🦀 Ejemplo Rust Nativo

```rust
// examples/quick-start.rs
use sniperforge::control::BotController;
use sniperforge::api::{BotType, BotConfig};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Inicializar controlador
    let controller = BotController::new().await?;
    println!("✅ SniperForge Enterprise inicializado");

    // Crear configuración
    let config = BotConfig {
        bot_type: BotType::LiquiditySniper,
        trading_pairs: vec!["BTC/USDT".to_string()],
        max_position_size: 100.0,
        risk_level: "Conservative".to_string(),
        enable_ai_analysis: true,
        // ... otros campos
    };

    // Crear y iniciar bot
    let bot_id = controller.create_bot(BotType::LiquiditySniper, config.clone()).await?;
    println!("🤖 Bot creado: {}", bot_id);

    controller.start_bot(bot_id, config).await?;
    println!("🚀 Bot iniciado");

    // Monitorear bot
    loop {
        let metrics = controller.get_bot_metrics(bot_id).await?;
        println!("📊 PnL: ${:.2} | Trades: {}", 
                 metrics.trading.total_pnl_usd, 
                 metrics.trading.trades_executed);
        
        sleep(Duration::from_secs(10)).await;
    }
}
```

## 📊 Monitoreo del Sistema

### Comando CLI Rápido

```bash
# Ver estado general
cargo run --bin sniperforge -- --status

# Ver métricas detalladas
cargo run --bin sniperforge -- --metrics

# Ver bots activos
cargo run --bin sniperforge -- --list-bots
```

### Dashboard Web Simple

```html
<!-- dashboard.html -->
<!DOCTYPE html>
<html>
<head>
    <title>SniperForge Dashboard</title>
    <script src="https://cdn.socket.io/4.7.0/socket.io.min.js"></script>
</head>
<body>
    <h1>SniperForge Enterprise Dashboard</h1>
    <div id="status">Conectando...</div>
    <div id="metrics"></div>
    
    <script>
        // Conectar via WebSocket proxy o polling
        async function updateDashboard() {
            try {
                const response = await fetch('/api/system/metrics');
                const data = await response.json();
                
                document.getElementById('status').innerHTML = `
                    <h2>Estado del Sistema</h2>
                    <p>Bots Totales: ${data.total_bots}</p>
                    <p>Bots Activos: ${data.running_bots}</p>
                    <p>Ganancia Total: $${data.total_profit.toFixed(2)}</p>
                    <p>Uptime: ${Math.floor(data.uptime_seconds / 3600)}h</p>
                `;
            } catch (error) {
                document.getElementById('status').innerHTML = 'Error de conexión';
            }
        }
        
        // Actualizar cada 5 segundos
        setInterval(updateDashboard, 5000);
        updateDashboard();
    </script>
</body>
</html>
```

## 🔧 Configuraciones Avanzadas

### Configuración de Producción

```json
{
  "system": {
    "mode": "production",
    "log_level": "info",
    "max_concurrent_bots": 50,
    "auto_restart": true,
    "backup_interval_hours": 6
  },
  "trading": {
    "default_risk_level": "moderate",
    "max_position_per_bot": 1000.0,
    "enable_ai_optimization": true,
    "slippage_tolerance": 0.1
  },
  "monitoring": {
    "metrics_retention_days": 30,
    "alert_thresholds": {
      "memory_usage_mb": 1000,
      "error_rate_percent": 5.0,
      "daily_loss_usd": 500.0
    }
  }
}
```

### Variables de Entorno

```bash
# .env
SNIPERFORGE_MODE=production
SNIPERFORGE_LOG_LEVEL=info
SNIPERFORGE_MAX_BOTS=50
SNIPERFORGE_BACKUP_ENABLED=true
SNIPERFORGE_TCP_PORT=8888
SNIPERFORGE_WALLET_PATH=./secure_wallet.json
```

## ⚠️ Solución de Problemas Comunes

### Error: "Puerto en uso"

```bash
# Verificar qué proceso usa el puerto
netstat -ano | findstr :8888

# Matar proceso si es necesario
taskkill /PID <PID> /F

# Usar puerto alternativo
cargo run --bin sniperforge -- --server-port 8889
```

### Error: "Bot no encontrado"

```bash
# Listar todos los bots
cargo run --bin sniperforge -- --list-bots

# Verificar estado de persistencia
cargo run --bin sniperforge -- --system-state
```

### Error: "Configuración inválida"

```bash
# Validar configuración
cargo run --bin sniperforge -- --validate-config

# Regenerar configuración desde template
cp config/template.json config/config.json
```

## 🎯 Próximos Pasos

### 1. Explorar APIs Avanzadas
- **[BotController API](../api/control/bot_controller.md)** - Control avanzado de bots
- **[TcpControlServer API](../api/control/tcp_server.md)** - Control remoto empresarial
- **[PerformanceAnalytics API](../api/analytics/performance.md)** - Analytics con IA

### 2. Configuraciones Empresariales
- Configuración multi-exchange
- Integración con sistemas de monitoreo
- Configuración de alertas y notificaciones

### 3. Desarrollo Personalizado
- Crear bots personalizados
- Integrar con sistemas existentes
- Desarrollar dashboards personalizados

## 📞 Soporte

- **Documentación Completa**: Ver [../INDEX.md](../INDEX.md)
- **Ejemplos Avanzados**: Ver directorio `/examples`
- **Testing**: `cargo test --workspace`
- **Logs**: Ubicados en `./logs/` por defecto

---

*© 2025 SniperForge Enterprise. Guía de Inicio Rápido - Listo en 5 minutos.*
