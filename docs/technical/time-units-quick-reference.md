# ⏱️ Time Units Quick Reference - SniperForge

## 📋 Summary: Time Units by Context

### 🚀 Monitoring Methods (Most Common)

| Method | Time Unit | Example Usage |
|--------|-----------|---------------|
| `start_event_driven_monitoring_seconds()` | **SECONDS** ✅ | `detector.start_event_driven_monitoring_seconds(300)` // 5 min |
| `start_monitoring_with_reports_seconds()` | **SECONDS** ✅ | `detector.start_monitoring_with_reports_seconds(180)` // 3 min |
| `start_ultra_fast_monitoring_seconds()` | **SECONDS** ✅ | `detector.start_ultra_fast_monitoring_seconds(60)` // 1 min |
| `start_monitoring_with_reports()` | **MINUTES** ⚠️ | `detector.start_monitoring_with_reports(3)` // 3 min (LEGACY) |

### 🖥️ CLI Commands

| Command | Parameter | Time Unit | Default | Example |
|---------|-----------|-----------|---------|---------|
| `quick-scan` | `--duration` | **SECONDS** | 300s | `--duration 300` (5 minutes) |
| `continuous-monitoring` | `--duration` | **HOURS** | 4h | `--duration 4` (4 hours) |
| `ultra-fast-scan` | `--duration` | **SECONDS** | 60s | `--duration 60` (1 minute) |
| `speed-test` | `--duration` | **SECONDS** | 30s | `--duration 30` (30 seconds) |
| `deep-analysis` | `--duration` | **SECONDS** | 180s | `--duration 180` (3 minutes) |
| `trading-automation` | `--duration` | **SECONDS** | 300s | `--duration 300` (5 minutes) |
| `real-trading` | `--duration` | **SECONDS** | 180s | `--duration 180` (3 minutes) |
| `mainnet-real-trading` | `--duration` | **MINUTES** | 60min | `--duration 60` (1 hour) |

### ⚙️ Configuration Fields

| Section | Field | Time Unit | Example Value |
|---------|-------|-----------|---------------|
| `[safety]` | `max_price_age_ms` | **MILLISECONDS** | `50` (50ms) |
| `[pool_detection]` | `monitoring_interval_ms` | **MILLISECONDS** | `2000` (2 seconds) |
| `[pool_detection]` | `max_execution_time_ms` | **MILLISECONDS** | `5000` (5 seconds) |
| `[trading_session]` | `default_duration_minutes` | **MINUTES** | `30` (30 minutes) |

## 🎯 Quick Decision Guide

### When to Use Each Unit

| Context | Use | Reasoning |
|---------|-----|-----------|
| **High-frequency operations** | MILLISECONDS | API timeouts, price age, execution limits |
| **Monitoring sessions** | SECONDS | Pool detection, scanning, analysis |
| **Trading sessions** | MINUTES or HOURS | User-friendly for longer sessions |
| **System limits** | HOURS | Daily limits, long-term configurations |

### 🔧 Code Examples

#### Monitoring (Use SECONDS)

```rust
// ✅ RECOMMENDED: Event-driven monitoring
detector.start_event_driven_monitoring_seconds(300).await?;  // 5 minutes

// ✅ ACCEPTABLE: Polling fallback  
detector.start_monitoring_with_reports_seconds(180).await?;  // 3 minutes

// ⚠️ LEGACY: Avoid in new code
detector.start_monitoring_with_reports(3).await?;  // 3 minutes (deprecated)
```

#### CLI Commands

```bash
# ✅ Most commands use SECONDS
cargo run -- test quick-scan --duration 300        # 5 minutes
cargo run -- test ultra-fast-scan --duration 60    # 1 minute  
cargo run -- test real-trading --duration 180      # 3 minutes

# ✅ Extended monitoring uses HOURS
cargo run -- test continuous-monitoring --duration 4  # 4 hours

# ✅ Real trading uses MINUTES (user-friendly)
cargo run -- test mainnet-real-trading --duration 60  # 1 hour
```

#### Configuration

```toml
[pool_detection]
monitoring_interval_ms = 2000     # 2 seconds (MILLISECONDS)
max_execution_time_ms = 5000      # 5 seconds (MILLISECONDS)

[trading_session]  
default_duration_minutes = 30     # 30 minutes (MINUTES)
```

## 🚨 Important Notes

1. **Always check method signatures** for `_seconds`, `_minutes`, or `_ms` suffixes
2. **CLI help text** always specifies the unit (e.g., "duration in seconds")
3. **Configuration comments** include unit clarification
4. **Most monitoring operations use SECONDS** for consistency
5. **Only legacy methods use MINUTES** (being deprecated)
6. **High-frequency operations use MILLISECONDS** for precision

## 🔄 Migration Path

### If you see MINUTES-based methods

```rust
// ❌ OLD: Minutes-based (being deprecated)
detector.start_monitoring_with_reports(5).await?;

// ✅ NEW: Seconds-based (recommended)  
detector.start_event_driven_monitoring_seconds(300).await?;  // 5 * 60 = 300 seconds
```

### If you see ambiguous CLI

```bash
# ❌ OLD: Unclear units
--duration 5

# ✅ NEW: Check help text for units
cargo run -- test quick-scan --help  # Shows "duration in seconds"
```

This ensures all time-related operations are clear and consistent across the SniperForge codebase.
