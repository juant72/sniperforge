# Time Units Standardization in SniperForge

## Overview

This document clarifies the time units used in all monitoring and configuration methods across the SniperForge codebase.

## Current State Analysis

### Monitoring Methods Time Units

| Method | Parameter | Unit | Status | Notes |
|--------|-----------|------|---------|-------|
| `start_monitoring_with_reports()` | `duration_minutes` | **MINUTES** | ⚠️ Legacy | Only used in quick demos |
| `start_event_driven_monitoring_seconds()` | `duration_seconds` | **SECONDS** | ✅ Primary | Event-driven, recommended |
| `start_monitoring_with_reports_seconds()` | `duration_seconds` | **SECONDS** | ⚠️ Deprecated | Polling-based, fallback only |
| `start_ultra_fast_monitoring_seconds()` | `duration_seconds` | **SECONDS** | ✅ Active | Wrapper for fast testing |

### CLI Command Time Units

| Command | Parameter | Unit | Default | Usage Context |
|---------|-----------|------|---------|---------------|
| `quick-scan` | `--duration` | **SECONDS** | 300s (5 min) | Quick pool scanning |
| `continuous-monitoring` | `--duration` | **HOURS** | 4h | Long-term monitoring |
| `ultra-fast-scan` | `--duration` | **SECONDS** | 60s | Fast testing |
| `speed-test` | `--duration` | **SECONDS** | 30s | Performance testing |
| `deep-analysis` | `--duration` | **SECONDS** | 180s (3 min) | Analysis mode |
| `trading-automation` | `--duration` | **SECONDS** | 300s (5 min) | Automated trading |
| `real-trading` | `--duration` | **SECONDS** | 180s (3 min) | Live trading |

### Configuration Time Units

| Config Section | Field | Unit | Default | Purpose |
|----------------|-------|------|---------|---------|
| `[pool_detection]` | `scan_interval_ms` | **MILLISECONDS** | 500ms | Scan frequency |
| `[pool_detection]` | `max_execution_time_ms` | **MILLISECONDS** | 10000ms | Max scan time |
| `[pool_detection]` | `cache_duration_seconds` | **SECONDS** | 300s | Cache validity |
| `[trading_session]` | `max_session_duration_hours` | **HOURS** | 24h | Session limit |
| `[trading_session]` | `position_timeout_minutes` | **MINUTES** | 30min | Position timeout |

## Standardization Rules

### ✅ RECOMMENDED: Use Seconds for All New Code

- **Primary Unit**: Seconds (`u64` seconds)
- **Method Naming**: Always suffix with `_seconds` (e.g., `start_monitoring_seconds()`)
- **CLI Help**: Always specify "in seconds" in help text
- **Config Fields**: Use `_seconds` suffix for second-based fields

### ⚠️ LEGACY: Minutes-based Methods

- `start_monitoring_with_reports(duration_minutes)` - **Only for quick demos**
- Will be **deprecated** in future versions
- Current usage: Quick 3-minute demo scans

### 🔧 SPECIALIZED: Sub-second Timing

- **Milliseconds**: For high-frequency operations (scan intervals, timeouts)
- **Nanoseconds**: For performance measurements only

## Implementation Guidelines

### 1. Method Signatures

```rust
// ✅ GOOD: Clear units in parameter name
pub async fn start_monitoring_seconds(&mut self, duration_seconds: u64) -> Result<()>

// ⚠️ LEGACY: Ambiguous units
pub async fn start_monitoring(&mut self, duration: u64) -> Result<()>

// ❌ BAD: No unit indication
pub async fn monitor(&mut self, time: u64) -> Result<()>
```

### 2. CLI Arguments

```rust
// ✅ GOOD: Clear help text with units
.arg(Arg::new("duration")
    .long("duration")
    .help("Monitoring duration in seconds (default: 300)")
    .default_value("300"))

// ⚠️ UNCLEAR: Ambiguous units
.arg(Arg::new("duration")
    .long("duration")
    .help("Monitoring duration (default: 5)")
    .default_value("5"))
```

### 3. Configuration Fields

```toml
# ✅ GOOD: Clear field names
[pool_detection]
scan_interval_ms = 500           # milliseconds
cache_duration_seconds = 300     # seconds

[trading_session]
max_session_duration_hours = 24  # hours
position_timeout_minutes = 30    # minutes

# ❌ BAD: Ambiguous field names
[pool_detection]
scan_interval = 500              # What unit?
cache_duration = 300             # What unit?
```

## Migration Plan

### Phase 1: Documentation (✅ COMPLETED)

- [x] Document all current time units
- [x] Identify inconsistencies
- [x] Define standardization rules

### Phase 2: New Code Standards (In Progress)

- [ ] All new methods use `_seconds` suffix
- [ ] All new CLI args specify units in help text
- [ ] All new config fields have unit suffixes

### Phase 3: Legacy Method Updates (Future)

- [ ] Add deprecation warnings to minutes-based methods
- [ ] Provide seconds-based alternatives
- [ ] Update CLI to use seconds consistently

### Phase 4: Breaking Changes (Future Major Version)

- [ ] Remove all minutes-based legacy methods
- [ ] Standardize all config field names
- [ ] Update all documentation

## Current Usage Patterns

### Event-Driven Monitoring (Primary)

```rust
// ✅ PRIMARY: Event-driven detection (seconds)
detector.start_event_driven_monitoring_seconds(300).await?;  // 5 minutes
```

### Polling-Based Monitoring (Fallback)

```rust
// ⚠️ DEPRECATED: Polling-based detection (seconds)
detector.start_monitoring_with_reports_seconds(180).await?;  // 3 minutes
```

### Quick Demo Monitoring (Legacy)

```rust
// ⚠️ LEGACY: Quick demo (minutes) - minimal usage
detector.start_monitoring_with_reports(3).await?;  // 3 minutes
```

## Best Practices

### For Developers

1. **Always** use seconds as the primary unit for duration parameters
2. **Always** include unit suffix in parameter names (`_seconds`, `_minutes`, `_hours`)
3. **Always** specify units in CLI help text and config comments
4. **Prefer** event-driven methods over polling-based methods
5. **Document** any non-standard time units clearly

### For Configuration

1. Use appropriate units for the context:
   - **Milliseconds**: High-frequency operations (< 1 second)
   - **Seconds**: Most monitoring and detection operations
   - **Minutes**: Medium-term timeouts and limits
   - **Hours**: Session durations and long-term limits

### For CLI Design

1. Default values should be reasonable for the use case
2. Help text should always specify the unit
3. Consider providing shortcuts for common durations

## Examples

### Good Examples

```rust
// Method with clear units
pub async fn start_monitoring_seconds(&mut self, duration_seconds: u64) -> Result<()> {
    let total_duration = Duration::from_secs(duration_seconds);
    // ...
}

// CLI with clear help
.arg(Arg::new("duration")
    .long("duration")
    .help("Trading session duration in seconds (default: 180)")
    .default_value("180"))

// Config with unit suffixes
[pool_detection]
scan_interval_ms = 500
cache_duration_seconds = 300
```

### Bad Examples

```rust
// Ambiguous method
pub async fn monitor(&mut self, time: u64) -> Result<()> {
    // What unit is 'time'?
}

// Unclear CLI
.arg(Arg::new("duration")
    .help("Duration")  // No unit specified
    .default_value("5"))  // 5 what?

// Ambiguous config
[detection]
interval = 500  # milliseconds? seconds?
timeout = 30    # seconds? minutes?
```

## Conclusion

The SniperForge codebase currently uses a mix of time units, but follows these patterns:

- **Seconds**: Primary unit for most monitoring operations
- **Minutes**: Legacy methods and some timeouts
- **Hours**: Long-term session limits
- **Milliseconds**: High-frequency operations

Going forward, **seconds** should be the primary unit for all new monitoring methods, with clear naming conventions and documentation.
