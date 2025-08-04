//! Docker templates for containerized bots

# Enhanced Arbitrage Bot Container
FROM rust:1.75-slim as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src/ src/

# Build the arbitrage bot
RUN cargo build --release --bin enhanced-arbitrage-bot

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -ms /bin/bash appuser
USER appuser
WORKDIR /home/appuser

# Copy executable
COPY --from=builder /app/target/release/enhanced-arbitrage-bot /usr/local/bin/

# Bot configuration environment variables
ENV BOT_TYPE=enhanced_arbitrage
ENV BOT_ID=
ENV API_GATEWAY_URL=http://api-gateway:8080
ENV LOG_LEVEL=info
ENV RUST_LOG=enhanced_arbitrage_bot=info

# Health check configuration
ENV HEALTH_CHECK_PORT=9090
ENV HEALTH_CHECK_PATH=/health

# Resource limits (these will be overridden by Kubernetes)
ENV MAX_MEMORY_MB=512
ENV MAX_CPU_CORES=1.0

# Network configuration
ENV SOLANA_RPC_URL=https://api.devnet.solana.com
ENV SOLANA_WS_URL=wss://api.devnet.solana.com

# Expose health check port
EXPOSE 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:9090/health || exit 1

# Start the bot
CMD ["enhanced-arbitrage-bot"]
