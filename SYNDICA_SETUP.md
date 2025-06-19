# Syndica Configuration for Ultra-Low Latency Trading

## Syndica Endpoints (ACTIVE)

### DevNet
```
wss://solana-devnet.api.syndica.io/api-key/4gJVJtRPS6J2MMWPasUfQHitRZCzQShiJUtKFBTZgXgqmcyCnyVdRVZ1wcjYKkCF83MNSVyP12EDeYJgFMr3zqQjdArFmPXRwmT
```

### MainNet
```
wss://solana-mainnet.api.syndica.io/api-key/4gJVJtRPS6J2MMWPasUfQHitRZCzQShiJUtKFBTZgXgqmcyCnyVdRVZ1wcjYKkCF83MNSVyP12EDeYJgFMr3zqQjdArFmPXRwmT
```

## Setup Instructions

1. Set environment variable:
```bash
export SYNDICA_TOKEN=4gJVJtRPS6J2MMWPasUfQHitRZCzQShiJUtKFBTZgXgqmcyCnyVdRVZ1wcjYKkCF83MNSVyP12EDeYJgFMr3zqQjdArFmPXRwmT
```

2. Test ultra-fast performance:
```bash
cargo run -- test syndica
```

## Expected Performance
- Current HTTP: 50ms average
- Syndica WebSocket Target: 5-15ms (3-10x faster)
- Potential improvement: Up to 10x speed boost
