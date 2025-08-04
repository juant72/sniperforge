# ML Analytics Bot Container
FROM python:3.11-slim as ml-builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    gcc \
    g++ \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Rust for Python integration
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

# Copy Python requirements
COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

# Copy Rust dependencies
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src/ src/
COPY python/ python/

# Build the ML bot (hybrid Rust/Python)
RUN cargo build --release --bin ml-analytics-bot

# Runtime stage
FROM python:3.11-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -ms /bin/bash appuser
USER appuser
WORKDIR /home/appuser

# Copy Python dependencies
COPY --from=ml-builder /usr/local/lib/python3.11/site-packages /usr/local/lib/python3.11/site-packages

# Copy executable and Python modules
COPY --from=ml-builder /app/target/release/ml-analytics-bot /usr/local/bin/
COPY --from=ml-builder /app/python/ ./python/

# Bot configuration
ENV BOT_TYPE=ml_analytics
ENV BOT_ID=
ENV API_GATEWAY_URL=http://api-gateway:8080
ENV LOG_LEVEL=info
ENV PYTHON_PATH=/home/appuser/python

# ML specific configuration
ENV ML_MODEL_PATH=/home/appuser/models
ENV ML_CACHE_SIZE=1000
ENV ML_PREDICTION_INTERVAL=60

# Health check configuration
ENV HEALTH_CHECK_PORT=9091
ENV HEALTH_CHECK_PATH=/health

# Resource limits (ML needs more resources)
ENV MAX_MEMORY_MB=1024
ENV MAX_CPU_CORES=2.0

# Expose health check port
EXPOSE 9091

# Health check
HEALTHCHECK --interval=30s --timeout=15s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:9091/health || exit 1

# Start the ML bot
CMD ["ml-analytics-bot"]
