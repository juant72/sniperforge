# Guía de Despliegue - SniperForge

## 🚀 Opciones de Despliegue

SniperForge está diseñado para ser flexible en su despliegue, desde desarrollo local hasta producción distribuida.

## 🏠 Desarrollo Local

### Prerrequisitos

```bash
# Instalar Rust (última versión estable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verificar instalación
rustc --version
cargo --version

# Instalar herramientas adicionales
cargo install cargo-watch
cargo install cargo-audit
```text

### Configuración Local

```bash
# Clonar el repositorio
git clone <repository-url>
cd sniperforge

# Crear archivo de configuración local
cp config/global.toml.example config/global.toml
cp config/bots/raydium-lp-sniper.toml.example config/bots/raydium-lp-sniper.toml

# Editar configuración para devnet
vim config/global.toml
```text

### Ejecutar en Desarrollo

```bash
# Compilar todos los componentes
cargo build

# Ejecutar el Raydium LP Sniper en modo debug
RUST_LOG=debug cargo run --bin raydium-lp-sniper

# Ejecutar con configuración específica
cargo run --bin raydium-lp-sniper -- --config config/dev.toml

# Ejecutar en modo simulación
SIMULATION_MODE=true cargo run --bin raydium-lp-sniper
```text

### Hot Reload para Desarrollo

```bash
# Usar cargo-watch para recompilación automática
cargo watch -x 'run --bin raydium-lp-sniper'

# Con configuración específica
cargo watch -x 'run --bin raydium-lp-sniper -- --config config/dev.toml'
```text

## 🐳 Despliegue con Docker

### Dockerfile

```dockerfile
# Imagen base multi-stage para optimizar tamaño
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .

# Instalar dependencias del sistema
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Compilar el proyecto
RUN cargo build --release --bin raydium-lp-sniper

# Imagen final mínima
FROM debian:bookworm-slim

# Instalar CA certificates para HTTPS
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Crear usuario no-root
RUN useradd -m -u 1000 sniperforge

# Copiar binario compilado
COPY --from=builder /app/target/release/raydium-lp-sniper /usr/local/bin/
COPY --from=builder /app/config /app/config

# Cambiar a usuario no-root
USER sniperforge
WORKDIR /app

# Exponer puerto para métricas
EXPOSE 9090

# Comando por defecto
CMD ["raydium-lp-sniper"]
```text

### Construcción de Imagen

```bash
# Construcción básica
docker build -t sniperforge/raydium-lp-sniper:latest .

# Con etiquetas específicas
docker build -t sniperforge/raydium-lp-sniper:v1.0.0 .

# Para múltiples arquitecturas
docker buildx build --platform linux/amd64,linux/arm64 \
  -t sniperforge/raydium-lp-sniper:latest --push .
```text

### Ejecución con Docker

```bash
# Ejecutar contenedor simple
docker run -d \
  --name raydium-sniper \
  -v $(pwd)/config:/app/config \
  -v $(pwd)/data:/app/data \
  -p 9090:9090 \
  sniperforge/raydium-lp-sniper:latest

# Con variables de entorno
docker run -d \
  --name raydium-sniper \
  -e SOLANA_RPC_URL="https://api.mainnet-beta.solana.com" \
  -e LOG_LEVEL="info" \
  -e SIMULATION_MODE="false" \
  -v $(pwd)/config:/app/config \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/logs:/var/log/sniperforge \
  sniperforge/raydium-lp-sniper:latest
```text

## 🎼 Orquestación con Docker Compose

### docker-compose.yml

```yaml
version: '3.8'

services:
  raydium-lp-sniper:
    build: .
    container_name: raydium-sniper
    restart: unless-stopped
    environment:
      - SOLANA_RPC_URL=${SOLANA_RPC_URL}
      - LOG_LEVEL=${LOG_LEVEL:-info}
      - SIMULATION_MODE=${SIMULATION_MODE:-false}
    volumes:
      - ./config:/app/config:ro
      - ./data:/app/data
      - ./logs:/var/log/sniperforge
      - ./keystore:/secure/keystore:ro
    ports:
      - "9090:9090"
    networks:
      - sniperforge-net
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9090/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    restart: unless-stopped
    ports:
      - "9091:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
    networks:
      - sniperforge-net

  grafana:
    image: grafana/grafana:latest
    container_name: grafana
    restart: unless-stopped
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD:-admin}
    volumes:
      - grafana-data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources:ro
    networks:
      - sniperforge-net

volumes:
  prometheus-data:
  grafana-data:

networks:
  sniperforge-net:
    driver: bridge
```text

### Variables de Entorno (.env)

```bash
# .env file
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
LOG_LEVEL=info
SIMULATION_MODE=false
GRAFANA_PASSWORD=secure_password_here
```text

### Ejecución del Stack Completo

```bash
# Iniciar todos los servicios
docker-compose up -d

# Ver logs en tiempo real
docker-compose logs -f raydium-lp-sniper

# Escalar servicios (si es necesario)
docker-compose up -d --scale raydium-lp-sniper=2

# Parar servicios
docker-compose down

# Parar y limpiar volúmenes
docker-compose down -v
```text

## ☸️ Despliegue en Kubernetes

### Namespace

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: sniperforge
```text

### ConfigMap

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: sniperforge-config
  namespace: sniperforge
data:
  global.toml: |
    [solana]
    rpc_url = "https://api.mainnet-beta.solana.com"
    commitment = "confirmed"
    
    [logging]
    level = "info"
    format = "json"
  
  raydium-lp-sniper.toml: |
    [detector]
    polling_interval = 1000
    
    [risk]
    max_position_size_pct = 0.02
    max_concurrent_positions = 5
```text

### Secret

```yaml
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: sniperforge-secrets
  namespace: sniperforge
type: Opaque
data:
  # Base64 encoded private key
  private-key: <base64-encoded-private-key>
  # Base64 encoded webhook URL
  webhook-url: <base64-encoded-webhook-url>
```text

### Deployment

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: raydium-lp-sniper
  namespace: sniperforge
  labels:
    app: raydium-lp-sniper
spec:
  replicas: 1
  selector:
    matchLabels:
      app: raydium-lp-sniper
  template:
    metadata:
      labels:
        app: raydium-lp-sniper
    spec:
      containers:
      - name: raydium-lp-sniper
        image: sniperforge/raydium-lp-sniper:v1.0.0
        ports:
        - containerPort: 9090
          name: metrics
        env:
        - name: SOLANA_RPC_URL
          value: "https://api.mainnet-beta.solana.com"
        - name: LOG_LEVEL
          value: "info"
        volumeMounts:
        - name: config
          mountPath: /app/config
          readOnly: true
        - name: secrets
          mountPath: /secure
          readOnly: true
        - name: data
          mountPath: /app/data
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 9090
          initialDelaySeconds: 30
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /ready
            port: 9090
          initialDelaySeconds: 5
          periodSeconds: 10
      volumes:
      - name: config
        configMap:
          name: sniperforge-config
      - name: secrets
        secret:
          secretName: sniperforge-secrets
      - name: data
        persistentVolumeClaim:
          claimName: sniperforge-data
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: sniperforge-data
  namespace: sniperforge
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
```text

### Service

```yaml
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: raydium-lp-sniper-service
  namespace: sniperforge
  labels:
    app: raydium-lp-sniper
spec:
  ports:
  - port: 9090
    targetPort: 9090
    name: metrics
  selector:
    app: raydium-lp-sniper
```text

### Despliegue en K8s

```bash
# Aplicar manifiestos
kubectl apply -f namespace.yaml
kubectl apply -f configmap.yaml
kubectl apply -f secret.yaml
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml

# Verificar estado
kubectl get pods -n sniperforge
kubectl get services -n sniperforge

# Ver logs
kubectl logs -f deployment/raydium-lp-sniper -n sniperforge

# Port forward para debugging
kubectl port-forward service/raydium-lp-sniper-service 9090:9090 -n sniperforge
```text

## 📊 Monitoreo y Observabilidad

### Configuración de Prometheus

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'sniperforge'
    static_configs:
      - targets: ['raydium-lp-sniper:9090']
    metrics_path: /metrics
    scrape_interval: 5s
```text

### Dashboard de Grafana

```json
{
  "dashboard": {
    "title": "SniperForge - Raydium LP Sniper",
    "panels": [
      {
        "title": "Pools Detected",
        "type": "stat",
        "targets": [
          {
            "expr": "increase(sniperforge_pools_detected_total[1h])"
          }
        ]
      },
      {
        "title": "Successful Trades",
        "type": "stat",
        "targets": [
          {
            "expr": "increase(sniperforge_trades_successful_total[1h])"
          }
        ]
      },
      {
        "title": "P&L",
        "type": "graph",
        "targets": [
          {
            "expr": "sniperforge_pnl_total"
          }
        ]
      }
    ]
  }
}
```text

## 🚨 Alertas

### Configuración de AlertManager

```yaml
# alertmanager.yml
global:
  smtp_smarthost: 'localhost:587'
  smtp_from: 'alerts@sniperforge.com'

route:
  group_by: ['alertname']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'web.hook'

receivers:
- name: 'web.hook'
  webhook_configs:
  - url: 'http://discord-webhook-url'
```text

### Reglas de Alerta

```yaml
# alert-rules.yml
groups:
- name: sniperforge.rules
  rules:
  - alert: HighErrorRate
    expr: rate(sniperforge_errors_total[5m]) > 0.1
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "High error rate detected"
      
  - alert: LowSuccessRate
    expr: rate(sniperforge_trades_successful_total[10m]) / rate(sniperforge_trades_attempted_total[10m]) < 0.5
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "Trade success rate below 50%"
```text

## 🔐 Seguridad en Producción

### Mejores Prácticas

1. **Gestión de Secretos**
   - Usar Kubernetes Secrets o HashiCorp Vault
   - Rotar claves privadas regularmente
   - Cifrar datos en reposo

2. **Acceso de Red**
   - Usar Network Policies en K8s
   - Restringir acceso a puertos de métricas
   - VPN para acceso administrativo

3. **Actualizaciones**
   - Automatizar actualizaciones de seguridad
   - Usar imágenes base mínimas
   - Escanear vulnerabilidades regularmente

4. **Backup y Recovery**
   - Backup automático de configuración
   - Plan de disaster recovery
   - Testing de procedimientos de recovery

Esta guía proporciona una base sólida para desplegar SniperForge en diferentes entornos, desde desarrollo hasta producción empresarial.
