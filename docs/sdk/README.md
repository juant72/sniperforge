# 🛠️ SniperForge SDK para Frontend

## 🎯 Guía de Implementación Completa

Esta documentación proporciona una guía completa para implementar el SDK de frontend que se conecte con SniperForge Enterprise.

---

## 📦 Estructura del SDK

```typescript
// src/sdk/index.ts
export class SniperForgeSDK {
  // Conexión TCP
  private tcpClient: TCPClient;
  private wsClient?: WebSocketClient;
  
  // Estado reactivo
  private state: SniperForgeState;
  private eventEmitter: EventEmitter;
  
  // Configuración
  private config: SDKConfig;
}
```

---

## 🔌 Protocolo de Comunicación

### TCP Client Implementation

```typescript
// src/sdk/tcp-client.ts
export class TCPClient {
  private socket: Socket;
  private host: string;
  private port: number;
  
  constructor(host = 'localhost', port = 8888) {
    this.host = host;
    this.port = port;
  }
  
  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.socket = new Socket();
      
      this.socket.connect(this.port, this.host, () => {
        console.log('✅ Connected to SniperForge TCP Server');
        resolve();
      });
      
      this.socket.on('error', reject);
    });
  }
  
  async sendCommand<T>(command: TcpCommand): Promise<TcpResponse<T>> {
    return new Promise((resolve, reject) => {
      const jsonCommand = JSON.stringify(command);
      
      this.socket.write(jsonCommand, 'utf8');
      
      this.socket.once('data', (data) => {
        try {
          const response = JSON.parse(data.toString());
          resolve(response);
        } catch (error) {
          reject(new Error('Invalid JSON response'));
        }
      });
      
      // Timeout after 30 seconds
      setTimeout(() => {
        reject(new Error('Command timeout'));
      }, 30000);
    });
  }
  
  disconnect(): void {
    this.socket?.destroy();
  }
}
```

---

## 📊 Tipos TypeScript

### Core Types

```typescript
// src/sdk/types.ts

// Bot Types
export interface Bot {
  id: string;
  name: string;
  type: BotType;
  status: BotStatus;
  health: HealthLevel;
  created_at: string;
  updated_at: string;
  metrics: BotMetrics;
  config: BotConfig;
}

export enum BotType {
  Arbitrage = 'arbitrage',
  MLAnalytics = 'ml-analytics', 
  Sentiment = 'sentiment',
  FlashLoan = 'flash-loan',
  CrossChain = 'cross-chain'
}

export enum BotStatus {
  Stopped = 'Stopped',
  Running = 'Running', 
  Paused = 'Paused',
  Error = 'Error',
  Maintenance = 'Maintenance',
  Initializing = 'Initializing'
}

export enum HealthLevel {
  Healthy = 'Healthy',
  Warning = 'Warning', 
  Unhealthy = 'Unhealthy'
}

// Metrics Types
export interface BotMetrics {
  operational: OperationalMetrics;
  trading: TradingMetrics;
  performance: PerformanceMetrics;
}

export interface TradingMetrics {
  trades_executed: number;
  successful_trades: number;
  total_pnl_usd: number;
  success_rate: number;
  avg_profit_per_trade: number;
  total_volume_usd: number;
  sharpe_ratio?: number;
}

export interface PerformanceMetrics {
  cpu_usage_percent: number;
  memory_usage_mb: number;
  network_io: NetworkIOMetrics;
  api_calls: ApiCallMetrics;
  avg_response_time_ms: number;
  throughput_per_second: number;
}

export interface OperationalMetrics {
  uptime_seconds: number;
  restart_count: number;
  last_restart?: string;
  config_updates: number;
  error_count: number;
}

// Desired State Types
export interface DesiredState {
  reconciliation: ReconciliationConfig;
  bots: DesiredBotState[];
  system: DesiredSystemState;
}

export interface DesiredBotState {
  id: string;
  type: BotType;
  desired_status: DesiredBotStatus;
  config: DesiredBotConfig;
  resources: BotResourceConfig;
  health_checks?: DesiredHealthConfig;
  auto_scaling?: AutoScalingConfig;
}

export enum DesiredBotStatus {
  Running = 'running',
  Stopped = 'stopped', 
  Paused = 'paused',
  Maintenance = 'maintenance'
}

// Command Types
export interface TcpCommand {
  command: string;
  [key: string]: any;
}

export interface TcpResponse<T = any> {
  type: string;
  data?: T;
  message?: string;
  error?: string;
}

// System Types
export interface SystemMetrics {
  total_bots: number;
  running_bots: number;
  healthy_bots: number;
  total_trades_today: number;
  total_pnl_today: number;
  system_uptime_seconds: number;
  memory_usage_mb: number;
  cpu_usage_percent: number;
}

export interface SystemResourceStatus {
  total_memory_mb: number;
  used_memory_mb: number;
  total_cpu_cores: number;
  cpu_usage_percent: number;
  disk_usage_gb: number;
  network_rx_bytes: number;
  network_tx_bytes: number;
}
```

---

## 🎮 SDK Core Implementation

### Main SDK Class

```typescript
// src/sdk/sniperforge-sdk.ts
export class SniperForgeSDK extends EventEmitter {
  private tcpClient: TCPClient;
  private config: SDKConfig;
  private state: SniperForgeState;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  
  constructor(config: SDKConfig = {}) {
    super();
    this.config = {
      host: 'localhost',
      port: 8888,
      autoReconnect: true,
      pollInterval: 1000,
      ...config
    };
    
    this.state = {
      connected: false,
      bots: [],
      systemMetrics: null,
      desiredState: null,
      reconciliationStatus: null
    };
  }
  
  // 🔌 Connection Management
  async connect(): Promise<void> {
    try {
      this.tcpClient = new TCPClient(this.config.host, this.config.port);
      await this.tcpClient.connect();
      
      this.state.connected = true;
      this.reconnectAttempts = 0;
      
      this.emit('connected');
      this.startPolling();
      
    } catch (error) {
      this.state.connected = false;
      this.emit('error', error);
      
      if (this.config.autoReconnect && this.reconnectAttempts < this.maxReconnectAttempts) {
        this.reconnectAttempts++;
        setTimeout(() => this.connect(), 5000);
      }
    }
  }
  
  disconnect(): void {
    this.tcpClient?.disconnect();
    this.state.connected = false;
    this.stopPolling();
    this.emit('disconnected');
  }
  
  // 📊 State Management
  private startPolling(): void {
    if (this.pollTimer) return;
    
    this.pollTimer = setInterval(async () => {
      try {
        await this.refreshState();
      } catch (error) {
        this.emit('error', error);
      }
    }, this.config.pollInterval);
  }
  
  private stopPolling(): void {
    if (this.pollTimer) {
      clearInterval(this.pollTimer);
      this.pollTimer = null;
    }
  }
  
  private async refreshState(): Promise<void> {
    const [bots, systemMetrics, desiredState] = await Promise.all([
      this.listBots(),
      this.getSystemMetrics(), 
      this.getDesiredState()
    ]);
    
    this.state.bots = bots;
    this.state.systemMetrics = systemMetrics;
    this.state.desiredState = desiredState;
    
    this.emit('stateChanged', this.state);
  }
  
  // 🤖 Bot Operations
  async listBots(): Promise<Bot[]> {
    const response = await this.tcpClient.sendCommand({
      command: 'ListBots'
    });
    
    if (response.type === 'BotList') {
      return response.data;
    } else {
      throw new Error(response.error || 'Failed to list bots');
    }
  }
  
  async createBot(type: BotType, config: BotConfig): Promise<string> {
    const response = await this.tcpClient.sendCommand({
      command: 'CreateBot',
      bot_type: type,
      config
    });
    
    if (response.type === 'BotCreated') {
      this.emit('botCreated', response.data.bot_id);
      return response.data.bot_id;
    } else {
      throw new Error(response.error || 'Failed to create bot');
    }
  }
  
  async startBot(botId: string, config?: BotConfig): Promise<void> {
    const response = await this.tcpClient.sendCommand({
      command: 'StartBot',
      bot_id: botId,
      config: config || {}
    });
    
    if (response.type === 'BotStarted') {
      this.emit('botStarted', botId);
    } else {
      throw new Error(response.error || 'Failed to start bot');
    }
  }
  
  async stopBot(botId: string): Promise<void> {
    const response = await this.tcpClient.sendCommand({
      command: 'StopBot', 
      bot_id: botId
    });
    
    if (response.type === 'BotStopped') {
      this.emit('botStopped', botId);
    } else {
      throw new Error(response.error || 'Failed to stop bot');
    }
  }
  
  async getBotStatus(botId: string): Promise<BotStatus> {
    const response = await this.tcpClient.sendCommand({
      command: 'GetBotStatus',
      bot_id: botId
    });
    
    if (response.type === 'BotStatus') {
      return response.data;
    } else {
      throw new Error(response.error || 'Failed to get bot status');
    }
  }
  
  async getBotMetrics(botId: string): Promise<BotMetrics> {
    const response = await this.tcpClient.sendCommand({
      command: 'GetBotMetrics',
      bot_id: botId
    });
    
    if (response.type === 'BotMetrics') {
      return response.data;
    } else {
      throw new Error(response.error || 'Failed to get bot metrics');
    }
  }
  
  // 📊 System Operations
  async getSystemMetrics(): Promise<SystemMetrics> {
    const response = await this.tcpClient.sendCommand({
      command: 'GetSystemMetrics'
    });
    
    if (response.type === 'SystemMetrics') {
      return response.data;
    } else {
      throw new Error(response.error || 'Failed to get system metrics');
    }
  }
  
  async getResourceStatus(): Promise<SystemResourceStatus> {
    const response = await this.tcpClient.sendCommand({
      command: 'GetResourceStatus'
    });
    
    if (response.type === 'ResourceStatus') {
      return response.data;
    } else {
      throw new Error(response.error || 'Failed to get resource status');
    }
  }
  
  // 🎯 Desired State Operations
  async getDesiredState(): Promise<DesiredState> {
    // Esta funcionalidad requiere implementación en el backend
    // Por ahora devolvemos un estado vacío
    return {
      reconciliation: {
        enabled: true,
        interval_seconds: 30,
        max_retries: 3,
        timeout_seconds: 300
      },
      bots: [],
      system: {
        maintenance_mode: false
      }
    };
  }
  
  async updateDesiredState(desiredState: DesiredState): Promise<void> {
    // Esta funcionalidad requiere implementación en el backend
    this.emit('desiredStateUpdated', desiredState);
  }
  
  // 🔄 Mass Operations
  async startAllBots(): Promise<void> {
    const response = await this.tcpClient.sendCommand({
      command: 'StartAllBots'
    });
    
    if (response.type === 'MassControlResult') {
      this.emit('allBotsStarted', response.data);
    } else {
      throw new Error(response.error || 'Failed to start all bots');
    }
  }
  
  async stopAllBots(): Promise<void> {
    const response = await this.tcpClient.sendCommand({
      command: 'StopAllBots'
    });
    
    if (response.type === 'MassControlResult') {
      this.emit('allBotsStopped', response.data);
    } else {
      throw new Error(response.error || 'Failed to stop all bots');
    }
  }
  
  // 🏥 Health & Utility
  async ping(): Promise<boolean> {
    try {
      const response = await this.tcpClient.sendCommand({
        command: 'Ping'
      });
      return response.type === 'Pong';
    } catch {
      return false;
    }
  }
  
  // 📄 Getters
  get isConnected(): boolean {
    return this.state.connected;
  }
  
  get currentState(): SniperForgeState {
    return { ...this.state };
  }
}
```

---

## ⚛️ React Integration

### React Hook

```typescript
// src/hooks/use-sniperforge.ts
import { useState, useEffect, useCallback } from 'react';
import { SniperForgeSDK, SniperForgeState, Bot, SystemMetrics } from '../sdk';

export function useSniperForge(config?: SDKConfig) {
  const [sdk] = useState(() => new SniperForgeSDK(config));
  const [state, setState] = useState<SniperForgeState>(sdk.currentState);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  useEffect(() => {
    // Event listeners
    sdk.on('stateChanged', setState);
    sdk.on('connected', () => {
      setError(null);
      setLoading(false);
    });
    sdk.on('error', (err) => {
      setError(err.message);
      setLoading(false);
    });
    
    // Auto-connect
    setLoading(true);
    sdk.connect().catch((err) => {
      setError(err.message);
      setLoading(false);
    });
    
    return () => {
      sdk.removeAllListeners();
      sdk.disconnect();
    };
  }, [sdk]);
  
  // Bot operations
  const createBot = useCallback(async (type: BotType, config: BotConfig) => {
    setLoading(true);
    try {
      const botId = await sdk.createBot(type, config);
      setError(null);
      return botId;
    } catch (err) {
      setError(err.message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [sdk]);
  
  const startBot = useCallback(async (botId: string) => {
    setLoading(true);
    try {
      await sdk.startBot(botId);
      setError(null);
    } catch (err) {
      setError(err.message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [sdk]);
  
  const stopBot = useCallback(async (botId: string) => {
    setLoading(true);
    try {
      await sdk.stopBot(botId);
      setError(null);
    } catch (err) {
      setError(err.message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [sdk]);
  
  return {
    // State
    ...state,
    loading,
    error,
    
    // Operations
    createBot,
    startBot,
    stopBot,
    startAllBots: sdk.startAllBots.bind(sdk),
    stopAllBots: sdk.stopAllBots.bind(sdk),
    
    // Utils
    sdk
  };
}
```

### React Components

```tsx
// src/components/BotDashboard.tsx
import React from 'react';
import { useSniperForge } from '../hooks/use-sniperforge';
import { BotCard } from './BotCard';
import { SystemMetricsPanel } from './SystemMetricsPanel';

export function BotDashboard() {
  const {
    bots,
    systemMetrics,
    loading,
    error,
    startBot,
    stopBot,
    startAllBots,
    stopAllBots
  } = useSniperForge();
  
  if (loading && !bots.length) {
    return <div className="loading">🔄 Connecting to SniperForge...</div>;
  }
  
  if (error) {
    return <div className="error">❌ Error: {error}</div>;
  }
  
  return (
    <div className="bot-dashboard">
      <header className="dashboard-header">
        <h1>🚀 SniperForge Enterprise</h1>
        <div className="controls">
          <button onClick={() => startAllBots()}>▶️ Start All</button>
          <button onClick={() => stopAllBots()}>⏸️ Stop All</button>
        </div>
      </header>
      
      <SystemMetricsPanel metrics={systemMetrics} />
      
      <div className="bots-grid">
        {bots.map(bot => (
          <BotCard
            key={bot.id}
            bot={bot}
            onStart={() => startBot(bot.id)}
            onStop={() => stopBot(bot.id)}
          />
        ))}
      </div>
    </div>
  );
}
```

```tsx
// src/components/BotCard.tsx
import React from 'react';
import { Bot, BotStatus, HealthLevel } from '../sdk/types';

interface BotCardProps {
  bot: Bot;
  onStart: () => void;
  onStop: () => void;
}

export function BotCard({ bot, onStart, onStop }: BotCardProps) {
  const getStatusIcon = (status: BotStatus) => {
    switch (status) {
      case BotStatus.Running: return '▶️';
      case BotStatus.Stopped: return '⏹️';
      case BotStatus.Paused: return '⏸️';
      case BotStatus.Error: return '❌';
      default: return '⚪';
    }
  };
  
  const getHealthIcon = (health: HealthLevel) => {
    switch (health) {
      case HealthLevel.Healthy: return '✅';
      case HealthLevel.Warning: return '⚠️';
      case HealthLevel.Unhealthy: return '🔴';
      default: return '⚪';
    }
  };
  
  return (
    <div className={`bot-card status-${bot.status.toLowerCase()}`}>
      <header className="bot-header">
        <h3>{bot.name}</h3>
        <div className="bot-status">
          {getStatusIcon(bot.status)} {bot.status}
          {getHealthIcon(bot.health)} {bot.health}
        </div>
      </header>
      
      <div className="bot-metrics">
        <div className="metric">
          <label>Uptime:</label>
          <span>{Math.floor(bot.metrics.operational.uptime_seconds / 3600)}h</span>
        </div>
        <div className="metric">
          <label>Trades:</label>
          <span>{bot.metrics.trading.trades_executed}</span>
        </div>
        <div className="metric">
          <label>P&L:</label>
          <span className={bot.metrics.trading.total_pnl_usd >= 0 ? 'positive' : 'negative'}>
            ${bot.metrics.trading.total_pnl_usd.toFixed(2)}
          </span>
        </div>
        <div className="metric">
          <label>Success Rate:</label>
          <span>{(bot.metrics.trading.success_rate * 100).toFixed(1)}%</span>
        </div>
      </div>
      
      <div className="bot-controls">
        {bot.status === BotStatus.Stopped ? (
          <button onClick={onStart} className="btn-start">▶️ Start</button>
        ) : (
          <button onClick={onStop} className="btn-stop">⏹️ Stop</button>
        )}
      </div>
    </div>
  );
}
```

---

## 🎨 CSS Styles

```css
/* src/styles/dashboard.css */
.bot-dashboard {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
  padding-bottom: 20px;
  border-bottom: 1px solid #e0e0e0;
}

.dashboard-header h1 {
  margin: 0;
  color: #2c3e50;
}

.controls {
  display: flex;
  gap: 10px;
}

.controls button {
  padding: 10px 20px;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  font-weight: bold;
  transition: background-color 0.2s;
}

.controls button:first-child {
  background-color: #27ae60;
  color: white;
}

.controls button:last-child {
  background-color: #e74c3c;
  color: white;
}

.bots-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
  margin-top: 20px;
}

.bot-card {
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 20px;
  background: white;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  transition: transform 0.2s, box-shadow 0.2s;
}

.bot-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(0,0,0,0.15);
}

.bot-card.status-running {
  border-left: 4px solid #27ae60;
}

.bot-card.status-stopped {
  border-left: 4px solid #95a5a6;
}

.bot-card.status-error {
  border-left: 4px solid #e74c3c;
}

.bot-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.bot-header h3 {
  margin: 0;
  color: #2c3e50;
}

.bot-status {
  font-size: 0.9em;
  color: #7f8c8d;
}

.bot-metrics {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  margin-bottom: 15px;
}

.metric {
  display: flex;
  justify-content: space-between;
  padding: 5px 0;
  border-bottom: 1px solid #ecf0f1;
}

.metric label {
  color: #7f8c8d;
  font-size: 0.9em;
}

.metric span.positive {
  color: #27ae60;
  font-weight: bold;
}

.metric span.negative {
  color: #e74c3c;
  font-weight: bold;
}

.bot-controls {
  display: flex;
  gap: 10px;
}

.btn-start {
  background-color: #27ae60;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  flex: 1;
}

.btn-stop {
  background-color: #e74c3c;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  flex: 1;
}

.loading {
  text-align: center;
  padding: 50px;
  font-size: 1.2em;
  color: #7f8c8d;
}

.error {
  text-align: center;
  padding: 50px;
  font-size: 1.2em;
  color: #e74c3c;
  background-color: #fdf2f2;
  border: 1px solid #fecaca;
  border-radius: 8px;
  margin: 20px;
}
```

---

## 📦 Package.json

```json
{
  "name": "sniperforge-sdk",
  "version": "1.0.0",
  "description": "TypeScript SDK for SniperForge Enterprise",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "scripts": {
    "build": "tsc",
    "dev": "tsc --watch",
    "test": "jest",
    "demo": "SNIPERFORGE_DEMO_MODE=1 npm run build && node demo/index.js"
  },
  "dependencies": {
    "events": "^3.3.0"
  },
  "devDependencies": {
    "@types/node": "^18.0.0",
    "typescript": "^5.0.0",
    "jest": "^29.0.0",
    "@types/jest": "^29.0.0"
  },
  "peerDependencies": {
    "react": "^18.0.0"
  }
}
```

---

## 🎮 Demo & Testing

```typescript
// demo/index.ts
import { SniperForgeSDK, BotType } from '../src/sdk';

async function demo() {
  const sdk = new SniperForgeSDK();
  
  try {
    console.log('🔌 Connecting to SniperForge...');
    await sdk.connect();
    
    console.log('📊 Getting system metrics...');
    const metrics = await sdk.getSystemMetrics();
    console.log('System Metrics:', metrics);
    
    console.log('🤖 Listing bots...');
    const bots = await sdk.listBots();
    console.log('Bots:', bots);
    
    if (bots.length > 0) {
      const bot = bots[0];
      console.log(`📈 Getting metrics for bot ${bot.id}...`);
      const botMetrics = await sdk.getBotMetrics(bot.id);
      console.log('Bot Metrics:', botMetrics);
    }
    
  } catch (error) {
    console.error('❌ Demo failed:', error);
  } finally {
    sdk.disconnect();
  }
}

demo();
```

---

## 🚀 Próximos Pasos

### Implementaciones Pendientes:

1. **WebSocket Support**: Para updates en tiempo real
2. **Authentication**: Sistema de autenticación y autorización
3. **Error Handling**: Manejo robusto de errores y reconexión
4. **Caching**: Sistema de caché para mejor performance
5. **Offline Support**: Funcionalidad offline con sincronización

### Roadmap de Desarrollo:

```
Fase 1: ✅ Core TCP SDK
Fase 2: 🔄 React Integration  
Fase 3: 📋 Vue.js Support
Fase 4: 📋 WebSocket Real-time
Fase 5: 📋 Production Ready
```

---

*SDK Documentation para SniperForge Enterprise v5.8.1*
