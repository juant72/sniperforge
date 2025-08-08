# TcpControlServer API Documentation

## Overview

The **TcpControlServer** provides enterprise-grade remote control capabilities for SniperForge through a TCP-based command interface. It enables external applications, dashboards, and management tools to interact with the bot control system programmatically.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   TcpControlServer                          │
├─────────────────────────────────────────────────────────────┤
│  Network Layer:                                             │
│  • TCP Listener (Port 8888)                                │
│  • JSON Command Protocol                                   │
│  • Concurrent Connection Handling                          │
│  • Auto Hot-reload Integration                             │
│                                                             │
│  Integration:                                               │
│  • BotController Interface                                  │
│  • Enterprise Command Processing                           │
│  • Comprehensive Error Handling                            │
│  • Response Streaming                                       │
└─────────────────────────────────────────────────────────────┘
```

## Constructor

### `new(bot_controller: Arc<BotController>, port: u16) -> Result<Self>`

Creates a new TcpControlServer instance with enterprise configuration.

**Parameters:**
- `bot_controller: Arc<BotController>` - Shared reference to the bot controller
- `port: u16` - TCP port to listen on (typically 8888)

**Returns:**
- `Result<TcpControlServer>` - Configured server instance

**Example:**
```rust
let controller = Arc::new(BotController::new().await?);
let server = TcpControlServer::new(controller, 8888).await?;
```

**Error Handling:**
- Returns `Err` if port is already in use
- Returns `Err` if insufficient network permissions

---

## Server Operations

### `run() -> Result<()>`

Starts the TCP server and begins accepting client connections.

**Returns:**
- `Result<()>` - Never returns on success, error on failure

**Process Flow:**
1. Binds to configured TCP port
2. Starts connection acceptance loop
3. Spawns async handler for each connection
4. Provides concurrent client support

**Example:**
```rust
let server = TcpControlServer::new(controller, 8888).await?;
server.run().await?; // Runs indefinitely
```

**Connection Management:**
- Concurrent connections: Unlimited
- Connection timeout: 30 seconds idle
- Auto-reconnection support
- Graceful connection cleanup

---

## Command Protocol

### TcpCommand Enum

The server accepts JSON-encoded commands with the following structure:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum TcpCommand {
    // Bot Management
    ListBots,
    CreateBot { bot_type: BotType, config: BotConfig },
    StartBot { bot_id: Uuid, config: BotConfig },
    StopBot { bot_id: Uuid },
    
    // Bot Information
    GetBotStatus { bot_id: Uuid },
    GetBotMetrics { bot_id: Uuid },
    
    // System Operations
    GetSystemMetrics,
    GetSystemState,
    GetMetricsHistory { hours: u32 },
    
    // Persistence
    CreateBackup,
    ForceSave,
    
    // Mass Control
    StartAllBots,
    StopAllBots,
    
    // Resource Management
    GetResourceStatus,
    
    // Utility
    Ping,
    Shutdown,
}
```

### TcpResponse Enum

The server responds with JSON-encoded responses:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum TcpResponse {
    // Bot Responses
    BotList(Vec<BotSummary>),
    BotCreated { bot_id: Uuid },
    BotStarted { bot_id: Uuid },
    BotStopped { bot_id: Uuid },
    BotStatus(BotStatus),
    BotMetrics(BotMetrics),
    
    // System Responses
    SystemMetrics(SystemMetrics),
    SystemState(SystemStateSummary),
    MetricsHistory(Vec<PersistedSystemMetrics>),
    BackupCreated(String),
    MassControlResult(MassControlResult),
    ResourceStatus(SystemResourceStatus),
    
    // Utility Responses
    Pong,
    Success(String),
    Error(String),
}
```

---

## Bot Management Commands

### ListBots

Lists all registered bots with summary information.

**Command:**
```json
{
    "ListBots": null
}
```

**Response:**
```json
{
    "BotList": [
        {
            "id": "uuid-string",
            "bot_type": "LiquiditySniper",
            "status": "Running",
            "metrics": { /* BotMetrics */ },
            "is_default": false
        }
    ]
}
```

**Example (TCP Client):**
```typescript
const response = await client.sendCommand({ ListBots: null });
console.log(`Found ${response.BotList.length} bots`);
```

### CreateBot

Creates a new bot instance with specified configuration.

**Command:**
```json
{
    "CreateBot": {
        "bot_type": "LiquiditySniper",
        "config": {
            "bot_type": "LiquiditySniper",
            "trading_pairs": ["BTC/USDT"],
            "max_position_size": 1000.0
        }
    }
}
```

**Response:**
```json
{
    "BotCreated": {
        "bot_id": "uuid-string"
    }
}
```

**Example:**
```typescript
const command = {
    CreateBot: {
        bot_type: "LiquiditySniper",
        config: {
            bot_type: "LiquiditySniper",
            trading_pairs: ["BTC/USDT"],
            max_position_size: 1000.0
        }
    }
};
const response = await client.sendCommand(command);
console.log(`Created bot: ${response.BotCreated.bot_id}`);
```

### StartBot

Starts a specific bot with provided configuration.

**Command:**
```json
{
    "StartBot": {
        "bot_id": "uuid-string",
        "config": {
            "bot_type": "LiquiditySniper",
            "trading_pairs": ["BTC/USDT"],
            "max_position_size": 1000.0
        }
    }
}
```

**Response:**
```json
{
    "BotStarted": {
        "bot_id": "uuid-string"
    }
}
```

### StopBot

Stops a specific running bot.

**Command:**
```json
{
    "StopBot": {
        "bot_id": "uuid-string"
    }
}
```

**Response:**
```json
{
    "BotStopped": {
        "bot_id": "uuid-string"
    }
}
```

---

## Bot Information Commands

### GetBotStatus

Retrieves the current status of a specific bot.

**Command:**
```json
{
    "GetBotStatus": {
        "bot_id": "uuid-string"
    }
}
```

**Response:**
```json
{
    "BotStatus": "Running"
}
```

**Status Values:**
- `"Running"` - Bot is actively trading
- `"Stopped"` - Bot is not running
- `"Paused"` - Bot is temporarily paused
- `"Error"` - Bot encountered an error

### GetBotMetrics

Retrieves comprehensive metrics for a specific bot.

**Command:**
```json
{
    "GetBotMetrics": {
        "bot_id": "uuid-string"
    }
}
```

**Response:**
```json
{
    "BotMetrics": {
        "trading": {
            "total_pnl_usd": 1250.75,
            "trades_executed": 42,
            "win_rate": 78.5,
            "avg_trade_duration_ms": 1500
        },
        "performance": {
            "uptime_seconds": 3600,
            "memory_usage_mb": 12.5,
            "cpu_usage_percent": 2.1
        },
        "errors": {
            "total_errors": 2,
            "network_errors": 1,
            "trading_errors": 1
        }
    }
}
```

---

## System Commands

### GetSystemMetrics

Retrieves system-wide performance metrics.

**Command:**
```json
{
    "GetSystemMetrics": null
}
```

**Response:**
```json
{
    "SystemMetrics": {
        "total_bots": 5,
        "running_bots": 3,
        "total_profit": 2500.50,
        "total_trades": 156,
        "uptime_seconds": 7200,
        "memory_usage_mb": 85.2
    }
}
```

### GetSystemState

Retrieves comprehensive system state information.

**Command:**
```json
{
    "GetSystemState": null
}
```

**Response:**
```json
{
    "SystemState": {
        "total_registered_bots": 5,
        "server_restart_count": 3,
        "system_start_time": "2024-01-15T10:30:00Z",
        "persisted_bots": ["uuid1", "uuid2", "uuid3"]
    }
}
```

### GetMetricsHistory

Retrieves historical metrics for specified time period.

**Command:**
```json
{
    "GetMetricsHistory": {
        "hours": 24
    }
}
```

**Response:**
```json
{
    "MetricsHistory": [
        {
            "timestamp": "2024-01-15T10:00:00Z",
            "total_bots": 5,
            "running_bots": 3,
            "total_profit": 2450.25,
            "total_trades": 150,
            "system_uptime_seconds": 6600,
            "memory_usage_mb": 82.1
        }
    ]
}
```

---

## Persistence Commands

### CreateBackup

Creates a complete backup of current system state.

**Command:**
```json
{
    "CreateBackup": null
}
```

**Response:**
```json
{
    "BackupCreated": "/path/to/backup/backup_2024-01-15_10-30-00.yaml"
}
```

### ForceSave

Forces immediate save of all system state to persistence.

**Command:**
```json
{
    "ForceSave": null
}
```

**Response:**
```json
{
    "Success": "All state saved to persistence"
}
```

---

## Mass Control Commands

### StartAllBots

Starts all registered bots that are currently stopped.

**Command:**
```json
{
    "StartAllBots": null
}
```

**Response:**
```json
{
    "MassControlResult": {
        "successful": ["uuid1", "uuid2"],
        "failed": [
            ["uuid3", "Configuration validation failed"]
        ],
        "total_attempted": 3
    }
}
```

### StopAllBots

Stops all currently running bots.

**Command:**
```json
{
    "StopAllBots": null
}
```

**Response:**
```json
{
    "MassControlResult": {
        "successful": ["uuid1", "uuid2"],
        "failed": [],
        "total_attempted": 2
    }
}
```

---

## Resource Management Commands

### GetResourceStatus

Retrieves comprehensive system resource usage and limits.

**Command:**
```json
{
    "GetResourceStatus": null
}
```

**Response:**
```json
{
    "ResourceStatus": {
        "total_bots": 5,
        "running_bots": 3,
        "memory_usage_mb": 85.2,
        "estimated_max_bots": 20,
        "resource_warning": "Running 15 bots, recommended max: 20 for optimal performance",
        "cpu_cores": 8
    }
}
```

---

## Utility Commands

### Ping

Tests server connectivity and responsiveness.

**Command:**
```json
{
    "Ping": null
}
```

**Response:**
```json
{
    "Pong": null
}
```

### Shutdown

Initiates graceful server shutdown.

**Command:**
```json
{
    "Shutdown": null
}
```

**Response:**
```json
{
    "Success": "Shutdown initiated"
}
```

**Note:** The connection will be closed after this response.

---

## Enterprise Features

### Hot-Reload Integration

Every command automatically triggers configuration hot-reload:

```
🔄 Hot-reload: Updating configurations from disk...
✅ Hot-reload completed successfully
```

This ensures that:
- Latest configuration templates are always used
- Bot configurations are synchronized from disk
- No system restart required for configuration updates

### Comprehensive Error Handling

The server provides detailed error information:

```json
{
    "Error": "Bot not found: 12345678-1234-1234-1234-123456789abc"
}
```

Common error categories:
- **Configuration Errors**: Invalid bot configuration parameters
- **Bot Not Found**: Specified bot does not exist
- **Resource Limits**: System resource constraints
- **Network Errors**: Trading network connectivity issues
- **Persistence Errors**: State save/load failures

### Connection Management

- **Concurrent Connections**: Supports unlimited simultaneous clients
- **Connection Pooling**: Efficient resource management
- **Auto-reconnection**: Client library supports automatic reconnection
- **Graceful Shutdown**: Proper connection cleanup on server shutdown

---

## Client Integration

### TCP Client Library (TypeScript)

```typescript
import { TCPClient } from '@sniperforge/sdk';

// Connect to server
const client = new TCPClient('localhost', 8888);
await client.connect();

// Send commands
const bots = await client.sendCommand({ ListBots: null });
console.log(`Found ${bots.BotList.length} bots`);

// Create and start bot
const createResponse = await client.sendCommand({
    CreateBot: {
        bot_type: "LiquiditySniper",
        config: { /* bot config */ }
    }
});

const botId = createResponse.BotCreated.bot_id;
await client.sendCommand({
    StartBot: {
        bot_id: botId,
        config: { /* updated config */ }
    }
});

// Monitor bot
setInterval(async () => {
    const metrics = await client.sendCommand({
        GetBotMetrics: { bot_id: botId }
    });
    console.log(`PnL: $${metrics.BotMetrics.trading.total_pnl_usd}`);
}, 5000);

// Disconnect
client.disconnect();
```

### Raw TCP Integration

```python
import socket
import json

# Connect to server
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.connect(('localhost', 8888))

# Send command
command = {"ListBots": None}
sock.send((json.dumps(command) + '\n').encode())

# Receive response
response = sock.recv(4096).decode()
data = json.loads(response)
print(f"Bots: {len(data['BotList'])}")

sock.close()
```

---

## Performance Characteristics

### Latency

- Command processing: < 50ms average
- Bot operations: < 500ms
- System queries: < 10ms
- Network overhead: < 5ms on localhost

### Throughput

- Commands per second: 1000+
- Concurrent connections: 100+
- Command queue depth: Unlimited
- Memory per connection: ~1MB

### Reliability

- Connection timeout: 30 seconds
- Command timeout: 30 seconds (configurable)
- Auto-retry on network errors
- Graceful degradation under load

---

## Error Recovery & Monitoring

### Connection Recovery

```typescript
const client = new TCPClient('localhost', 8888);

client.on('disconnect', () => {
    console.log('Connection lost, attempting reconnection...');
    setTimeout(() => client.connect(), 5000);
});

client.on('error', (error) => {
    console.error('TCP Error:', error);
});
```

### Health Monitoring

```typescript
// Periodic health check
setInterval(async () => {
    try {
        await client.sendCommand({ Ping: null });
        console.log('✅ Server healthy');
    } catch (error) {
        console.error('❌ Server health check failed:', error);
    }
}, 30000);
```

### Enterprise Monitoring Integration

```typescript
// System monitoring dashboard
const systemMetrics = await client.sendCommand({ GetSystemMetrics: null });
const resourceStatus = await client.sendCommand({ GetResourceStatus: null });

// Send to monitoring system
await monitoringSystem.recordMetrics({
    timestamp: Date.now(),
    bots_total: systemMetrics.SystemMetrics.total_bots,
    bots_running: systemMetrics.SystemMetrics.running_bots,
    memory_usage: resourceStatus.ResourceStatus.memory_usage_mb,
    profit_total: systemMetrics.SystemMetrics.total_profit
});
```

---

## Security Considerations

### Network Security

- **Local Binding**: Server binds to localhost (127.0.0.1) by default
- **No Authentication**: Currently no authentication layer (internal system)
- **JSON Validation**: All commands are validated before processing
- **Rate Limiting**: Consider implementing for production deployments

### Operational Security

- **Command Validation**: All parameters validated before execution
- **Error Sanitization**: Sensitive information not exposed in error messages
- **Audit Logging**: All commands logged for enterprise auditing
- **Graceful Degradation**: System remains stable under invalid commands

### Production Deployment

For production environments, consider:

1. **TLS Encryption**: Implement TLS for encrypted communication
2. **Authentication**: Add API key or JWT-based authentication
3. **Authorization**: Role-based access control for commands
4. **Network Isolation**: Deploy behind firewall or VPN
5. **Monitoring**: Comprehensive logging and alerting

---

## Complete Integration Example

```typescript
import { TCPClient } from '@sniperforge/sdk';

class SniperForgeManager {
    private client: TCPClient;
    
    constructor(host = 'localhost', port = 8888) {
        this.client = new TCPClient(host, port);
    }
    
    async initialize() {
        await this.client.connect();
        console.log('✅ Connected to SniperForge');
    }
    
    async createAndStartBot(botType: string, config: any) {
        // Create bot
        const createResponse = await this.client.sendCommand({
            CreateBot: { bot_type: botType, config }
        });
        
        const botId = createResponse.BotCreated.bot_id;
        console.log(`Created bot: ${botId}`);
        
        // Start bot
        await this.client.sendCommand({
            StartBot: { bot_id: botId, config }
        });
        
        console.log(`Started bot: ${botId}`);
        return botId;
    }
    
    async monitorSystem() {
        const metrics = await this.client.sendCommand({ GetSystemMetrics: null });
        const resources = await this.client.sendCommand({ GetResourceStatus: null });
        
        return {
            totalBots: metrics.SystemMetrics.total_bots,
            runningBots: metrics.SystemMetrics.running_bots,
            totalProfit: metrics.SystemMetrics.total_profit,
            memoryUsage: resources.ResourceStatus.memory_usage_mb,
            resourceWarning: resources.ResourceStatus.resource_warning
        };
    }
    
    async shutdown() {
        // Stop all bots
        await this.client.sendCommand({ StopAllBots: null });
        
        // Create backup
        const backup = await this.client.sendCommand({ CreateBackup: null });
        console.log(`Backup created: ${backup.BackupCreated}`);
        
        // Shutdown server
        await this.client.sendCommand({ Shutdown: null });
        
        this.client.disconnect();
    }
}

// Usage
const manager = new SniperForgeManager();
await manager.initialize();

const botId = await manager.createAndStartBot('LiquiditySniper', {
    bot_type: 'LiquiditySniper',
    trading_pairs: ['BTC/USDT'],
    max_position_size: 1000.0
});

// Monitor
setInterval(async () => {
    const status = await manager.monitorSystem();
    console.log('System Status:', status);
}, 10000);
```
