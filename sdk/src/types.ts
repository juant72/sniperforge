// ============================================================================
// SniperForge SDK - Core Types & Interfaces
// ============================================================================

// ============================================================================
// Bot Types & Enums
// ============================================================================

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

// ============================================================================
// Bot Interfaces
// ============================================================================

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

export interface BotConfig {
  trading?: TradingConfig;
  risk?: RiskConfig;
  performance?: PerformanceConfig;
  notifications?: NotificationConfig;
  [key: string]: any;
}

export interface TradingConfig {
  max_position_size_usd?: number;
  min_profit_threshold?: number;
  max_slippage_percent?: number;
  trading_pairs?: string[];
  exchanges?: string[];
}

export interface RiskConfig {
  max_daily_loss_usd?: number;
  max_drawdown_percent?: number;
  stop_loss_percent?: number;
  position_size_percent?: number;
}

export interface PerformanceConfig {
  cpu_limit_percent?: number;
  memory_limit_mb?: number;
  api_rate_limit?: number;
  concurrent_operations?: number;
}

export interface NotificationConfig {
  email?: boolean;
  webhook?: boolean;
  telegram?: boolean;
  discord?: boolean;
}

// ============================================================================
// Metrics Interfaces
// ============================================================================

export interface BotMetrics {
  operational: OperationalMetrics;
  trading: TradingMetrics;
  performance: PerformanceMetrics;
}

export interface OperationalMetrics {
  uptime_seconds: number;
  restart_count: number;
  last_restart?: string;
  config_updates: number;
  error_count: number;
}

export interface TradingMetrics {
  trades_executed: number;
  successful_trades: number;
  total_pnl_usd: number;
  success_rate: number;
  avg_profit_per_trade: number;
  total_volume_usd: number;
  sharpe_ratio?: number;
  max_drawdown_percent?: number;
  daily_pnl_usd?: number;
  weekly_pnl_usd?: number;
  monthly_pnl_usd?: number;
}

export interface PerformanceMetrics {
  cpu_usage_percent: number;
  memory_usage_mb: number;
  network_io: NetworkIOMetrics;
  api_calls: ApiCallMetrics;
  avg_response_time_ms: number;
  throughput_per_second: number;
}

export interface NetworkIOMetrics {
  bytes_sent: number;
  bytes_received: number;
  requests_per_second: number;
  errors_per_second: number;
}

export interface ApiCallMetrics {
  total_calls: number;
  successful_calls: number;
  failed_calls: number;
  avg_response_time_ms: number;
  rate_limit_hits: number;
}

// ============================================================================
// System Types
// ============================================================================

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
  load_average: number[];
}

// ============================================================================
// Desired State Types
// ============================================================================

export interface DesiredState {
  reconciliation: ReconciliationConfig;
  bots: DesiredBotState[];
  system: DesiredSystemState;
}

export interface ReconciliationConfig {
  enabled: boolean;
  interval_seconds: number;
  max_retries: number;
  timeout_seconds: number;
  drift_tolerance_percent?: number;
  auto_heal?: boolean;
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

export interface DesiredBotConfig {
  trading?: TradingConfig;
  risk?: RiskConfig;
  performance?: PerformanceConfig;
  monitoring?: MonitoringConfig;
}

export interface BotResourceConfig {
  cpu_cores?: number;
  memory_mb?: number;
  disk_gb?: number;
  network_bandwidth_mbps?: number;
}

export interface DesiredHealthConfig {
  health_check_interval_seconds: number;
  failure_threshold: number;
  recovery_threshold: number;
  timeout_seconds: number;
}

export interface AutoScalingConfig {
  enabled: boolean;
  min_instances: number;
  max_instances: number;
  target_cpu_percent: number;
  scale_up_cooldown_seconds: number;
  scale_down_cooldown_seconds: number;
}

export interface DesiredSystemState {
  maintenance_mode: boolean;
  log_level?: string;
  backup_enabled?: boolean;
  monitoring_enabled?: boolean;
}

export interface MonitoringConfig {
  metrics_enabled: boolean;
  alerts_enabled: boolean;
  log_level: string;
  retention_days: number;
}

// ============================================================================
// TCP Protocol Types
// ============================================================================

export interface TcpCommand {
  command: string;
  [key: string]: any;
}

export interface TcpResponse<T = any> {
  type: string;
  data?: T;
  message?: string;
  error?: string;
  timestamp?: string;
  request_id?: string;
}

// ============================================================================
// SDK Configuration Types
// ============================================================================

export interface SDKConfig {
  host?: string;
  port?: number;
  autoReconnect?: boolean;
  pollInterval?: number;
  maxReconnectAttempts?: number;
  timeout?: number;
  enableMetrics?: boolean;
  enableWebSocket?: boolean;
  authToken?: string;
}

export interface SniperForgeState {
  connected: boolean;
  bots: Bot[];
  systemMetrics: SystemMetrics | null;
  desiredState: DesiredState | null;
  reconciliationStatus: ReconciliationStatus | null;
}

export interface ReconciliationStatus {
  enabled: boolean;
  last_run: string;
  next_run: string;
  drift_detected: boolean;
  bots_out_of_sync: string[];
  actions_pending: ReconciliationAction[];
}

export interface ReconciliationAction {
  bot_id: string;
  action_type: string;
  current_state: string;
  desired_state: string;
  status: string;
  error?: string;
}

// ============================================================================
// Event Types
// ============================================================================

export interface BotEvent {
  type: BotEventType;
  bot_id: string;
  timestamp: string;
  data?: any;
}

export enum BotEventType {
  Created = 'created',
  Started = 'started',
  Stopped = 'stopped',
  Paused = 'paused',
  Error = 'error',
  MetricsUpdated = 'metrics_updated',
  ConfigUpdated = 'config_updated',
  HealthChanged = 'health_changed'
}

export interface SystemEvent {
  type: SystemEventType;
  timestamp: string;
  data?: any;
}

export enum SystemEventType {
  SystemStarted = 'system_started',
  SystemStopped = 'system_stopped',
  MetricsUpdated = 'metrics_updated',
  AlertTriggered = 'alert_triggered',
  MaintenanceMode = 'maintenance_mode',
  ReconciliationCompleted = 'reconciliation_completed'
}

// ============================================================================
// Error Types
// ============================================================================

export class SniperForgeError extends Error {
  constructor(
    message: string,
    public code?: string,
    public data?: any
  ) {
    super(message);
    this.name = 'SniperForgeError';
  }
}

export class ConnectionError extends SniperForgeError {
  constructor(message: string, data?: any) {
    super(message, 'CONNECTION_ERROR', data);
    this.name = 'ConnectionError';
  }
}

export class BotOperationError extends SniperForgeError {
  constructor(message: string, public botId: string, data?: any) {
    super(message, 'BOT_OPERATION_ERROR', data);
    this.name = 'BotOperationError';
  }
}

export class ValidationError extends SniperForgeError {
  constructor(message: string, public field: string, data?: any) {
    super(message, 'VALIDATION_ERROR', data);
    this.name = 'ValidationError';
  }
}

// ============================================================================
// Utility Types
// ============================================================================

export type EventCallback<T = any> = (data: T) => void;

export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

export interface LogEntry {
  timestamp: string;
  level: LogLevel;
  message: string;
  context?: any;
}

// ============================================================================
// Mass Operation Types
// ============================================================================

export interface MassControlResult {
  successful_operations: string[];
  failed_operations: Array<{
    bot_id: string;
    error: string;
  }>;
  total_attempted: number;
  total_successful: number;
  total_failed: number;
}

// ============================================================================
// CLI Types
// ============================================================================

export interface CLICommand {
  name: string;
  description: string;
  options?: CLIOption[];
  handler: (args: any) => Promise<void>;
}

export interface CLIOption {
  name: string;
  description: string;
  type: 'string' | 'number' | 'boolean';
  required?: boolean;
  default?: any;
}
