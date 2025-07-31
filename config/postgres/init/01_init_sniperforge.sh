#!/bin/bash
# PostgreSQL Database Initialization Script for SniperForge

set -e

# Create extensions
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    -- Enable required extensions
    CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
    CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";
    CREATE EXTENSION IF NOT EXISTS "btree_gin";
    CREATE EXTENSION IF NOT EXISTS "pg_trgm";
    
    -- Create schemas
    CREATE SCHEMA IF NOT EXISTS trading;
    CREATE SCHEMA IF NOT EXISTS analytics;
    CREATE SCHEMA IF NOT EXISTS monitoring;
    CREATE SCHEMA IF NOT EXISTS security;
    
    -- Create trading tables
    CREATE TABLE IF NOT EXISTS trading.positions (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
        symbol VARCHAR(20) NOT NULL,
        side VARCHAR(10) NOT NULL CHECK (side IN ('buy', 'sell')),
        size DECIMAL(20,8) NOT NULL,
        entry_price DECIMAL(20,8) NOT NULL,
        current_price DECIMAL(20,8),
        unrealized_pnl DECIMAL(20,8) DEFAULT 0,
        realized_pnl DECIMAL(20,8) DEFAULT 0,
        status VARCHAR(20) NOT NULL DEFAULT 'open',
        opened_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        closed_at TIMESTAMP WITH TIME ZONE,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
    );
    
    CREATE TABLE IF NOT EXISTS trading.trades (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
        position_id UUID REFERENCES trading.positions(id),
        symbol VARCHAR(20) NOT NULL,
        side VARCHAR(10) NOT NULL CHECK (side IN ('buy', 'sell')),
        size DECIMAL(20,8) NOT NULL,
        price DECIMAL(20,8) NOT NULL,
        fee DECIMAL(20,8) DEFAULT 0,
        tx_signature VARCHAR(128),
        dex VARCHAR(50),
        strategy VARCHAR(100),
        execution_time_ms INTEGER,
        slippage_percent DECIMAL(10,4),
        executed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
    );
    
    CREATE TABLE IF NOT EXISTS trading.arbitrage_opportunities (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
        symbol VARCHAR(20) NOT NULL,
        dex_buy VARCHAR(50) NOT NULL,
        dex_sell VARCHAR(50) NOT NULL,
        buy_price DECIMAL(20,8) NOT NULL,
        sell_price DECIMAL(20,8) NOT NULL,
        profit_percent DECIMAL(10,4) NOT NULL,
        profit_usd DECIMAL(20,8) NOT NULL,
        size_limit DECIMAL(20,8),
        status VARCHAR(20) DEFAULT 'pending',
        executed BOOLEAN DEFAULT FALSE,
        discovered_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        expires_at TIMESTAMP WITH TIME ZONE,
        executed_at TIMESTAMP WITH TIME ZONE
    );
    
    -- Create analytics tables
    CREATE TABLE IF NOT EXISTS analytics.performance_snapshots (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
        total_trades INTEGER NOT NULL DEFAULT 0,
        successful_trades INTEGER NOT NULL DEFAULT 0,
        failed_trades INTEGER NOT NULL DEFAULT 0,
        total_volume_usd DECIMAL(20,8) NOT NULL DEFAULT 0,
        total_profit_usd DECIMAL(20,8) NOT NULL DEFAULT 0,
        win_rate_percent DECIMAL(10,4) NOT NULL DEFAULT 0,
        sharpe_ratio DECIMAL(10,4) DEFAULT 0,
        max_drawdown_percent DECIMAL(10,4) DEFAULT 0,
        avg_trade_duration_ms INTEGER DEFAULT 0,
        snapshot_date DATE NOT NULL DEFAULT CURRENT_DATE,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
    );
    
    -- Create monitoring tables
    CREATE TABLE IF NOT EXISTS monitoring.system_metrics (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
        instance_id VARCHAR(50) NOT NULL,
        cpu_usage_percent DECIMAL(5,2),
        memory_usage_mb DECIMAL(10,2),
        memory_usage_percent DECIMAL(5,2),
        disk_usage_percent DECIMAL(5,2),
        network_rx_bytes_per_sec BIGINT,
        network_tx_bytes_per_sec BIGINT,
        open_file_descriptors INTEGER,
        thread_count INTEGER,
        uptime_seconds BIGINT,
        recorded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
    );
    
    CREATE TABLE IF NOT EXISTS monitoring.api_metrics (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
        endpoint VARCHAR(200) NOT NULL,
        method VARCHAR(10) NOT NULL,
        status_code INTEGER NOT NULL,
        response_time_ms INTEGER NOT NULL,
        request_size_bytes INTEGER,
        response_size_bytes INTEGER,
        user_agent TEXT,
        ip_address INET,
        recorded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
    );
    
    -- Create security tables
    CREATE TABLE IF NOT EXISTS security.audit_logs (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
        event_type VARCHAR(50) NOT NULL,
        severity VARCHAR(20) NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
        description TEXT NOT NULL,
        user_id VARCHAR(100),
        ip_address INET,
        user_agent TEXT,
        metadata JSONB,
        occurred_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
    );
    
    CREATE TABLE IF NOT EXISTS security.api_keys (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
        key_name VARCHAR(100) NOT NULL,
        key_hash VARCHAR(128) NOT NULL UNIQUE,
        permissions JSONB NOT NULL DEFAULT '[]',
        rate_limit_per_minute INTEGER DEFAULT 60,
        is_active BOOLEAN DEFAULT TRUE,
        expires_at TIMESTAMP WITH TIME ZONE,
        last_used_at TIMESTAMP WITH TIME ZONE,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
    );
    
    -- Create indexes for performance
    CREATE INDEX IF NOT EXISTS idx_positions_symbol ON trading.positions(symbol);
    CREATE INDEX IF NOT EXISTS idx_positions_status ON trading.positions(status);
    CREATE INDEX IF NOT EXISTS idx_positions_opened_at ON trading.positions(opened_at);
    
    CREATE INDEX IF NOT EXISTS idx_trades_symbol ON trading.trades(symbol);
    CREATE INDEX IF NOT EXISTS idx_trades_executed_at ON trading.trades(executed_at);
    CREATE INDEX IF NOT EXISTS idx_trades_position_id ON trading.trades(position_id);
    
    CREATE INDEX IF NOT EXISTS idx_arbitrage_symbol ON trading.arbitrage_opportunities(symbol);
    CREATE INDEX IF NOT EXISTS idx_arbitrage_status ON trading.arbitrage_opportunities(status);
    CREATE INDEX IF NOT EXISTS idx_arbitrage_discovered_at ON trading.arbitrage_opportunities(discovered_at);
    
    CREATE INDEX IF NOT EXISTS idx_performance_snapshot_date ON analytics.performance_snapshots(snapshot_date);
    
    CREATE INDEX IF NOT EXISTS idx_system_metrics_instance ON monitoring.system_metrics(instance_id);
    CREATE INDEX IF NOT EXISTS idx_system_metrics_recorded_at ON monitoring.system_metrics(recorded_at);
    
    CREATE INDEX IF NOT EXISTS idx_api_metrics_endpoint ON monitoring.api_metrics(endpoint);
    CREATE INDEX IF NOT EXISTS idx_api_metrics_recorded_at ON monitoring.api_metrics(recorded_at);
    
    CREATE INDEX IF NOT EXISTS idx_audit_logs_event_type ON security.audit_logs(event_type);
    CREATE INDEX IF NOT EXISTS idx_audit_logs_severity ON security.audit_logs(severity);
    CREATE INDEX IF NOT EXISTS idx_audit_logs_occurred_at ON security.audit_logs(occurred_at);
    
    CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON security.api_keys(key_hash);
    CREATE INDEX IF NOT EXISTS idx_api_keys_is_active ON security.api_keys(is_active);
    
    -- Create functions for automatic timestamp updates
    CREATE OR REPLACE FUNCTION update_updated_at_column()
    RETURNS TRIGGER AS \$\$
    BEGIN
        NEW.updated_at = NOW();
        RETURN NEW;
    END;
    \$\$ language 'plpgsql';
    
    -- Create triggers for automatic timestamp updates
    DROP TRIGGER IF EXISTS update_positions_updated_at ON trading.positions;
    CREATE TRIGGER update_positions_updated_at
        BEFORE UPDATE ON trading.positions
        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    
    DROP TRIGGER IF EXISTS update_api_keys_updated_at ON security.api_keys;
    CREATE TRIGGER update_api_keys_updated_at
        BEFORE UPDATE ON security.api_keys
        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
    
    -- Create views for common queries
    CREATE OR REPLACE VIEW trading.active_positions AS
    SELECT *
    FROM trading.positions
    WHERE status = 'open';
    
    CREATE OR REPLACE VIEW trading.daily_trades AS
    SELECT
        DATE(executed_at) as trade_date,
        COUNT(*) as total_trades,
        SUM(CASE WHEN side = 'buy' THEN size * price ELSE 0 END) as total_buy_volume,
        SUM(CASE WHEN side = 'sell' THEN size * price ELSE 0 END) as total_sell_volume,
        SUM(fee) as total_fees
    FROM trading.trades
    WHERE executed_at >= CURRENT_DATE - INTERVAL '30 days'
    GROUP BY DATE(executed_at)
    ORDER BY trade_date DESC;
    
    CREATE OR REPLACE VIEW analytics.daily_performance AS
    SELECT
        DATE(t.executed_at) as trade_date,
        COUNT(*) as trades_count,
        SUM(t.size * t.price) as volume_usd,
        COUNT(CASE WHEN p.realized_pnl > 0 THEN 1 END) as winning_trades,
        COUNT(CASE WHEN p.realized_pnl < 0 THEN 1 END) as losing_trades,
        SUM(p.realized_pnl) as total_pnl,
        AVG(p.realized_pnl) as avg_pnl,
        COALESCE(COUNT(CASE WHEN p.realized_pnl > 0 THEN 1 END) * 100.0 / NULLIF(COUNT(*), 0), 0) as win_rate_percent
    FROM trading.trades t
    LEFT JOIN trading.positions p ON t.position_id = p.id
    WHERE t.executed_at >= CURRENT_DATE - INTERVAL '30 days'
    GROUP BY DATE(t.executed_at)
    ORDER BY trade_date DESC;
    
    -- Grant permissions
    GRANT USAGE ON SCHEMA trading TO sniperforge;
    GRANT USAGE ON SCHEMA analytics TO sniperforge;
    GRANT USAGE ON SCHEMA monitoring TO sniperforge;
    GRANT USAGE ON SCHEMA security TO sniperforge;
    
    GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA trading TO sniperforge;
    GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA analytics TO sniperforge;
    GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA monitoring TO sniperforge;
    GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA security TO sniperforge;
    
    GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA trading TO sniperforge;
    GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA analytics TO sniperforge;
    GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA monitoring TO sniperforge;
    GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA security TO sniperforge;
    
    -- Insert initial data
    INSERT INTO security.api_keys (key_name, key_hash, permissions, rate_limit_per_minute)
    VALUES (
        'default_admin',
        'admin_key_hash_placeholder',
        '["read", "write", "admin"]',
        1000
    ) ON CONFLICT (key_hash) DO NOTHING;
    
    -- Log successful initialization
    INSERT INTO security.audit_logs (event_type, severity, description)
    VALUES ('database_init', 'low', 'SniperForge Enterprise database initialized successfully');

EOSQL

echo "SniperForge Enterprise database initialization completed successfully!"
