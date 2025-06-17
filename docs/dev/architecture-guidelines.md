# Architecture Guidelines - SniperForge

## 🏗 Principios Arquitectónicos

### **1. Modular Design**
- **Separación de responsabilidades**: Cada módulo tiene una función específica
- **Interfaces claras**: APIs bien definidas entre módulos
- **Dependencias mínimas**: Bajo acoplamiento entre componentes
- **Testing fácil**: Cada módulo testeable independientemente

### **2. Performance First**
- **Latencia crítica**: <100ms para decisiones de trading
- **Memory efficiency**: <500MB uso total en producción
- **CPU optimization**: <70% uso promedio
- **Network efficiency**: Mínimo número de RPC calls

### **3. Security by Design**
- **Never trust external data**: Validación exhaustiva
- **Secrets management**: Keys nunca en plaintext
- **Error handling**: No leak de información sensible
- **Audit trail**: Logging completo de decisiones

### **4. Reliability**
- **Fault tolerance**: Sistema sigue operando con fallos parciales
- **Graceful degradation**: Funcionalidad reducida vs. crash completo
- **Quick recovery**: Auto-recovery en <30 segundos
- **Data consistency**: Transacciones atómicas donde sea crítico

---

## 📁 Module Organization

### **Core Modules**

#### **solana_client**
**Purpose**: Abstracción sobre Solana RPC client  
**Responsibilities**:
- Connection management y health checks
- Rate limiting y retry logic
- Transaction building y signing
- Account y program data parsing

**Public API**:
```rust
pub struct SolanaClient {
    // Internal implementation
}

impl SolanaClient {
    pub fn new(config: ClientConfig) -> Result<Self>;
    pub async fn health_check(&self) -> Result<()>;
    pub async fn get_account(&self, pubkey: &Pubkey) -> Result<Account>;
    pub async fn send_transaction(&self, tx: &Transaction) -> Result<Signature>;
}
```

#### **pool_monitor**
**Purpose**: Detección y análisis de nuevos pools  
**Responsibilities**:
- WebSocket subscriptions para eventos
- Pool data parsing y validation
- Opportunity scoring básico
- Historical data tracking

**Public API**:
```rust
pub struct PoolMonitor {
    // Internal implementation
}

impl PoolMonitor {
    pub fn new(client: Arc<SolanaClient>) -> Self;
    pub async fn start_monitoring(&mut self) -> Result<()>;
    pub fn subscribe_to_opportunities(&self) -> Receiver<PoolOpportunity>;
}
```

#### **trading**
**Purpose**: Lógica de trading y ejecución  
**Responsibilities**:
- Filter chain evaluation
- Trade execution
- Position tracking
- Risk management

**Public API**:
```rust
pub struct TradingEngine {
    // Internal implementation
}

impl TradingEngine {
    pub fn new(config: TradingConfig) -> Self;
    pub async fn evaluate_opportunity(&self, pool: &PoolInfo) -> TradeDecision;
    pub async fn execute_trade(&self, decision: TradeDecision) -> Result<TradeResult>;
}
```

### **Supporting Modules**

#### **config**
**Purpose**: Configuration management  
**Features**:
- TOML-based configuration
- Environment-specific overrides
- Hot reload capability
- Validation y defaults

#### **types**
**Purpose**: Core data structures  
**Features**:
- Shared types across modules
- Serialization/deserialization
- Validation logic
- Type-safe identifiers

#### **error**
**Purpose**: Error handling framework  
**Features**:
- Structured error types
- Error context preservation
- Logging integration
- Recovery strategies

---

## 🔄 Data Flow Architecture

### **Primary Flow: Opportunity Detection → Execution**

```
1. Pool Monitor
   ├─ WebSocket Events
   ├─ Pool Data Parsing
   └─ Opportunity Detection
          ↓
2. Filter Chain
   ├─ Liquidity Filter
   ├─ Age Filter
   ├─ Token Filter
   └─ Risk Filter
          ↓
3. Trading Engine
   ├─ Opportunity Evaluation
   ├─ Position Sizing
   └─ Trade Execution
          ↓
4. Position Manager
   ├─ Position Tracking
   ├─ Stop Loss Monitoring
   └─ Take Profit Monitoring
```

### **Supporting Flows**

#### **Configuration Flow**
```
TOML Files → Config Parser → Hot Reload → Module Updates
```

#### **Logging Flow**
```
All Modules → Structured Logger → File + Console Output
```

#### **Error Flow**
```
Error Detection → Context Addition → Recovery Attempt → Logging
```

---

## 📊 Performance Architecture

### **Latency Optimization**

#### **Hot Path Optimization**
- **Pool Detection**: WebSocket events processed en <10ms
- **Filter Evaluation**: All filters evaluated en <50ms
- **Trade Execution**: Transaction built y sent en <100ms
- **Position Updates**: Price updates processed en <20ms

#### **Memory Management**
- **Object Pooling**: Reuse de structures pesadas
- **Lazy Loading**: Load data solo cuando necesario
- **Cache Strategy**: Cache de resultados frecuentes
- **Memory Bounds**: Límites estrictos por módulo

#### **Network Optimization**
- **Connection Pooling**: Reuse de RPC connections
- **Batch Requests**: Batch múltiples requests cuando posible
- **Compression**: Gzip para large responses
- **CDN Usage**: Use geografically close RPC endpoints

### **Concurrency Model**

#### **Async Runtime**
```rust
// Main runtime setup
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    // Application setup
}
```

#### **Task Separation**
- **Pool Monitoring**: Dedicated task pool
- **Trade Execution**: High-priority task queue
- **Position Management**: Background task scheduler
- **Logging**: Non-blocking async logging

#### **Synchronization**
```rust
// Shared state patterns
Arc<RwLock<Config>>          // Configuration
Arc<Mutex<PositionManager>>  // Position state
broadcast::Sender<Event>     // Event distribution
mpsc::Sender<Command>        // Command queues
```

---

## 🔒 Security Architecture

### **Key Management**
- **Encryption at Rest**: All private keys encrypted with AES-256
- **Memory Protection**: Use `zeroize` para clear sensitive data
- **Access Control**: Keys accessible solo via secure API
- **Rotation**: Automated key rotation every 30 days

### **Input Validation**
```rust
// Example validation pattern
pub fn validate_pool_data(data: &RawPoolData) -> Result<PoolInfo> {
    // Validate all fields
    let liquidity = data.liquidity
        .ok_or(ValidationError::MissingLiquidity)?
        .check_bounds(MIN_LIQUIDITY, MAX_LIQUIDITY)?;
    
    let tokens = validate_token_pair(&data.base_mint, &data.quote_mint)?;
    
    Ok(PoolInfo {
        liquidity,
        tokens,
        // ... other validated fields
    })
}
```

### **Error Information Control**
```rust
// Never expose internal details
pub enum PublicError {
    InvalidConfiguration,
    NetworkError,
    InsufficientFunds,
    // No internal implementation details
}

impl From<InternalError> for PublicError {
    fn from(err: InternalError) -> Self {
        // Log internal error securely
        log::error!("Internal error: {:?}", err);
        
        // Return safe public error
        match err {
            InternalError::DatabaseConnection(_) => PublicError::NetworkError,
            InternalError::KeyDecryption(_) => PublicError::InvalidConfiguration,
            // ... other mappings
        }
    }
}
```

---

## 🧪 Testing Architecture

### **Testing Pyramid**

#### **Unit Tests (70%)**
- **Scope**: Individual functions y methods
- **Speed**: <1ms per test
- **Coverage**: >90% line coverage
- **Mocking**: External dependencies mocked

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_filter_evaluation() {
        let filter = LiquidityFilter::new(1000);
        let pool = create_test_pool(500); // Below minimum
        
        assert!(!filter.evaluate(&pool).passed());
    }
}
```

#### **Integration Tests (20%)**
- **Scope**: Module interactions
- **Speed**: <100ms per test
- **Environment**: Testnet/devnet
- **Data**: Real blockchain data

```rust
#[tokio::test]
async fn test_end_to_end_trading() {
    let client = create_devnet_client().await;
    let engine = TradingEngine::new(test_config());
    
    // Test complete flow with real devnet data
    let result = engine.process_opportunity(test_pool_data()).await;
    assert!(result.is_ok());
}
```

#### **E2E Tests (10%)**
- **Scope**: Complete system
- **Speed**: <10 seconds per test
- **Environment**: Staging environment
- **Monitoring**: Performance y reliability metrics

### **Test Data Management**

#### **Deterministic Test Data**
```rust
// Test data builders for consistent testing
pub struct PoolInfoBuilder {
    liquidity: Option<u64>,
    age: Option<Duration>,
    tokens: Option<(Pubkey, Pubkey)>,
}

impl PoolInfoBuilder {
    pub fn with_liquidity(mut self, liquidity: u64) -> Self {
        self.liquidity = Some(liquidity);
        self
    }
    
    pub fn build(self) -> PoolInfo {
        PoolInfo {
            liquidity: self.liquidity.unwrap_or(DEFAULT_LIQUIDITY),
            age: self.age.unwrap_or(DEFAULT_AGE),
            tokens: self.tokens.unwrap_or(DEFAULT_TOKENS),
            // ... other fields
        }
    }
}
```

#### **Performance Benchmarks**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_filter_chain(c: &mut Criterion) {
    let chain = create_filter_chain();
    let pool = create_benchmark_pool();
    
    c.bench_function("filter_chain_evaluation", |b| {
        b.iter(|| chain.evaluate(black_box(&pool)))
    });
}

criterion_group!(benches, bench_filter_chain);
criterion_main!(benches);
```

---

## 📈 Monitoring Architecture

### **Metrics Collection**

#### **Business Metrics**
- **Trade Success Rate**: Percentage of successful trades
- **Profit/Loss Tracking**: PnL per trade y aggregate
- **Opportunity Capture**: Detected vs. executed opportunities
- **Risk Adherence**: Stop loss y take profit compliance

#### **Technical Metrics**
- **Latency Metrics**: P50, P95, P99 for critical paths
- **Error Rates**: Errors per module per time period
- **Resource Usage**: CPU, memory, network utilization
- **Uptime**: System availability percentage

#### **Custom Metrics Implementation**
```rust
use prometheus::{Counter, Histogram, Gauge, register_counter};

lazy_static! {
    static ref TRADES_TOTAL: Counter = register_counter!(
        "trades_total", "Total number of trades executed"
    ).unwrap();
    
    static ref TRADE_LATENCY: Histogram = register_histogram!(
        "trade_latency_seconds", "Trade execution latency"
    ).unwrap();
    
    static ref ACTIVE_POSITIONS: Gauge = register_gauge!(
        "active_positions", "Number of currently active positions"
    ).unwrap();
}

// Usage in code
pub async fn execute_trade(&self, opportunity: TradeOpportunity) -> Result<TradeResult> {
    let timer = TRADE_LATENCY.start_timer();
    
    let result = self.internal_execute(opportunity).await;
    
    timer.observe_duration();
    
    match &result {
        Ok(_) => TRADES_TOTAL.inc(),
        Err(_) => {
            // Error metrics...
        }
    }
    
    result
}
```

### **Alerting Strategy**

#### **Critical Alerts** (Immediate Response)
- System downtime >1 minute
- Trade execution failures >5% rate
- Security-related errors
- Memory usage >90%

#### **Warning Alerts** (Response within 1 hour)
- Latency degradation >2x normal
- Error rate >1%
- Unusual trading patterns
- Configuration validation failures

#### **Info Alerts** (Daily Review)
- Performance optimization opportunities
- Daily PnL summaries
- System health reports
- Capacity planning metrics

---

**Esta arquitectura proporciona una base sólida, escalable y mantenible para el desarrollo de SniperForge, balanceando performance, seguridad y confiabilidad.**
