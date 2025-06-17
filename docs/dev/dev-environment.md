# Development Environment Setup - SniperForge

## 🛠 Prerequisites

### **System Requirements**

#### **Hardware Minimum**
- **CPU**: 4 cores, 2.5GHz+
- **RAM**: 8GB minimum, 16GB recommended
- **Storage**: 50GB available space (SSD recommended)
- **Network**: Stable internet connection (low latency to Solana RPCs)

#### **Operating System**
- **Linux**: Ubuntu 20.04+ (preferred for production)
- **macOS**: 11.0+ (good for development)
- **Windows**: 10+ with WSL2 (supported)

### **Required Software**

#### **Core Development Tools**
```bash
# Rust toolchain (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup component add clippy rustfmt

# Node.js (for tooling)
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# Git
sudo apt-get install git

# Build essentials
sudo apt-get install build-essential pkg-config libudev-dev
```

#### **Development Environment**
```bash
# VS Code (recommended IDE)
wget -qO- https://packages.microsoft.com/keys/microsoft.asc | gpg --dearmor > packages.microsoft.gpg
sudo install -o root -g root -m 644 packages.microsoft.gpg /etc/apt/trusted.gpg.d/
sudo sh -c 'echo "deb [arch=amd64,arm64,armhf signed-by=/etc/apt/trusted.gpg.d/packages.microsoft.gpg] https://packages.microsoft.com/repos/code stable main" > /etc/apt/sources.list.d/vscode.list'
sudo apt update
sudo apt install code

# Essential VS Code extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension vadimcn.vscode-lldb
code --install-extension tamasfe.even-better-toml
code --install-extension serayuzgur.crates
```

---

## 🚀 Project Setup

### **Repository Setup**
```bash
# Clone the repository
git clone https://github.com/your-org/sniperforge.git
cd sniperforge

# Setup Git hooks
cp scripts/git-hooks/* .git/hooks/
chmod +x .git/hooks/*

# Install pre-commit dependencies
cargo install cargo-fmt cargo-clippy
```

### **Environment Configuration**

#### **.env File Setup**
```bash
# Create environment file
cp .env.example .env

# Edit with your specific values
nano .env
```

#### **Environment Variables**
```bash
# .env file content
RUST_LOG=info
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_WS_URL=wss://api.devnet.solana.com

# For production (keep secure)
PRIVATE_KEY_PATH=/secure/path/to/key.json
ENCRYPTION_KEY_PATH=/secure/path/to/encryption.key

# Development only
DEV_MODE=true
DEV_PRIVATE_KEY=your_dev_private_key_here
```

### **Dependencies Installation**

#### **Rust Dependencies**
```bash
# Install project dependencies
cargo build

# Install development tools
cargo install --force cargo-watch
cargo install cargo-tarpaulin    # Code coverage
cargo install cargo-audit        # Security auditing
cargo install cargo-outdated     # Dependency updates
cargo install criterion          # Benchmarking
```

#### **Solana CLI Tools**
```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.16.0/install)"

# Add to PATH
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Verify installation
solana --version

# Configure for devnet
solana config set --url devnet
```

---

## 🔧 Development Workflow

### **Daily Development Setup**

#### **Environment Verification**
```bash
# Check all tools are working
./scripts/verify-dev-environment.sh

# Should output:
# ✅ Rust toolchain: 1.70.0
# ✅ Solana CLI: 1.16.0
# ✅ Node.js: 18.x
# ✅ Git: 2.x
# ✅ Environment variables loaded
```

#### **Development Server Startup**
```bash
# Start development with hot reload
cargo watch -x 'run --bin raydium-sniper'

# Or with specific environment
RUST_LOG=debug cargo watch -x 'run --bin raydium-sniper'
```

### **Code Quality Tools**

#### **Formatting y Linting**
```bash
# Format code
cargo fmt

# Check formatting without changing
cargo fmt -- --check

# Run clippy lints
cargo clippy -- -D warnings

# Run all quality checks
./scripts/check-code-quality.sh
```

#### **Testing Workflow**
```bash
# Run all tests
cargo test

# Run tests with coverage
cargo tarpaulin --out Html

# Run integration tests only
cargo test --test integration

# Run specific test
cargo test test_trading_engine

# Run tests with logging
RUST_LOG=debug cargo test -- --nocapture
```

### **Performance y Profiling**

#### **Benchmarking**
```bash
# Run benchmarks
cargo bench

# Profile specific component
cargo bench --bench trading_engine

# Generate flame graph (requires flamegraph)
cargo install flamegraph
sudo cargo flamegraph --bin raydium-sniper
```

#### **Memory Profiling**
```bash
# Install valgrind (Linux only)
sudo apt-get install valgrind

# Run with memory check
cargo build
valgrind --tool=memcheck target/debug/raydium-sniper

# For detailed analysis
valgrind --tool=massif target/debug/raydium-sniper
```

---

## 📋 Configuration Management

### **Configuration Files Structure**
```
config/
├── default.toml          # Default configuration
├── development.toml      # Development overrides
├── staging.toml          # Staging environment
├── production.toml       # Production environment
└── local.toml           # Local overrides (gitignored)
```

### **Configuration Loading Order**
1. `default.toml` (base configuration)
2. `{environment}.toml` (environment-specific)
3. `local.toml` (local overrides)
4. Environment variables (highest priority)

#### **Example Development Configuration**
```toml
# config/development.toml
[network]
rpc_url = "https://api.devnet.solana.com"
ws_url = "wss://api.devnet.solana.com"
commitment = "confirmed"

[trading]
enabled = true
environment = "devnet"
max_position_sol = 0.01
simulation_mode = true

[logging]
level = "debug"
file = "logs/dev-{date}.log"
console_enabled = true

[monitoring]
metrics_enabled = true
metrics_port = 9090
health_check_port = 8080
```

---

## 🧪 Testing Environment

### **Test Database Setup**
```bash
# For integration tests that need persistent data
docker run -d \
  --name sniperforge-test-db \
  -p 5433:5432 \
  -e POSTGRES_DB=sniperforge_test \
  -e POSTGRES_USER=test \
  -e POSTGRES_PASSWORD=test \
  postgres:14

# Run database migrations for testing
sqlx migrate run --database-url postgresql://test:test@localhost:5433/sniperforge_test
```

### **Mock Services**

#### **Solana RPC Mock**
```bash
# Start mock Solana RPC server for testing
cd tests/mocks
cargo run --bin mock-solana-rpc

# Mock server provides:
# - Predictable responses for testing
# - Controllable latency simulation
# - Error scenario simulation
```

#### **Test Data Generation**
```bash
# Generate test pool data
cargo run --bin generate-test-data -- --pools 100 --output tests/data/

# Generate test trading scenarios
cargo run --bin generate-test-scenarios
```

---

## 🐛 Debugging Setup

### **VS Code Debugging Configuration**

#### **.vscode/launch.json**
```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug SniperForge",
            "cargo": {
                "args": ["build", "--bin=raydium-sniper"],
                "filter": {
                    "name": "raydium-sniper",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}",
            "environment": [
                { "name": "RUST_LOG", "value": "debug" }
            ]
        },
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Tests",
            "cargo": {
                "args": ["test", "--no-run"],
                "filter": {
                    "name": "sniperforge",
                    "kind": "lib"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

### **Logging Configuration**

#### **Structured Logging Setup**
```rust
// src/logging.rs
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info,sniperforge=debug"))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
```

#### **Debug Logging in Development**
```bash
# Enable debug logging for specific modules
RUST_LOG=sniperforge::trading=debug,sniperforge::pool_monitor=trace cargo run

# Log to file and console
RUST_LOG=debug cargo run 2>&1 | tee logs/debug.log
```

---

## 📊 Monitoring y Metrics

### **Local Prometheus Setup**
```bash
# Install Prometheus locally
wget https://github.com/prometheus/prometheus/releases/download/v2.37.0/prometheus-2.37.0.linux-amd64.tar.gz
tar xvfz prometheus-2.37.0.linux-amd64.tar.gz
cd prometheus-2.37.0.linux-amd64

# Create config for SniperForge
cat > prometheus.yml << EOF
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'sniperforge'
    static_configs:
      - targets: ['localhost:9090']
EOF

# Start Prometheus
./prometheus --config.file=prometheus.yml
```

### **Grafana Dashboard**
```bash
# Install Grafana
sudo apt-get install -y software-properties-common
sudo add-apt-repository "deb https://packages.grafana.com/oss/deb stable main"
wget -q -O - https://packages.grafana.com/gpg.key | sudo apt-key add -
sudo apt-get update
sudo apt-get install grafana

# Start Grafana
sudo systemctl start grafana-server

# Access at http://localhost:3000 (admin/admin)
```

---

## 🚀 Production-like Testing

### **Docker Development Environment**
```bash
# Build development image
docker build -f docker/Dockerfile.dev -t sniperforge:dev .

# Run with development configuration
docker run -it \
  -v $(pwd):/workspace \
  -p 8080:8080 \
  -p 9090:9090 \
  sniperforge:dev

# Run complete development stack
docker-compose -f docker/docker-compose.dev.yml up
```

### **Load Testing Setup**
```bash
# Install wrk for load testing
sudo apt-get install wrk

# Test health endpoint
wrk -t12 -c400 -d30s http://localhost:8080/health

# Test metrics endpoint
wrk -t4 -c100 -d10s http://localhost:9090/metrics
```

---

## 🔐 Security Development Practices

### **Secret Management**
```bash
# Use environment variables for secrets
export PRIVATE_KEY=$(cat ~/.solana/devnet-key.json)

# Or use encrypted storage
gpg --symmetric --cipher-algo AES256 ~/.solana/mainnet-key.json
# Creates mainnet-key.json.gpg

# In development, decrypt when needed
gpg --decrypt ~/.solana/mainnet-key.json.gpg
```

### **Security Scanning**
```bash
# Audit dependencies for vulnerabilities
cargo audit

# Check for common security issues
cargo clippy -- -W clippy::all

# Run security-focused tests
cargo test security
```

---

## 📚 Documentation Generation

### **API Documentation**
```bash
# Generate documentation
cargo doc --open

# Generate documentation with private items
cargo doc --document-private-items --open

# Check documentation coverage
cargo doc --document-private-items 2>&1 | grep -E "warning|error"
```

### **Architecture Documentation**
```bash
# Generate dependency graph
cargo install cargo-deps
cargo deps --all-deps | dot -Tpng > architecture/dependency-graph.png

# Generate module structure
tree src/ > architecture/module-structure.txt
```

---

**Con este setup completo, cualquier desarrollador puede configurar un entorno de desarrollo funcional para SniperForge en menos de 30 minutos.**
