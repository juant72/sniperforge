# DexScreener API Integration

## Overview
DexScreener is a powerful DeFi analytics platform that provides real-time data about DEX pairs, tokens, and liquidity pools across multiple blockchains. Their API is free to use and doesn't require authentication.

## API Documentation
- **Base URL:** `https://api.dexscreener.com/`
- **Documentation:** https://docs.dexscreener.com/api/reference
- **Authentication:** Not required
- **Response Format:** JSON

## Rate Limits
- **Primary endpoints:** 300 requests per minute
  - `/latest/dex/pairs/{chainId}/{pairId}`
  - `/latest/dex/search`
  - `/token-pairs/v1/{chainId}/{tokenAddress}`
  - `/tokens/v1/{chainId}/{tokenAddresses}`
- **Secondary endpoints:** 60 requests per minute
  - `/token-profiles/latest/v1`
  - `/token-boosts/latest/v1`
  - `/token-boosts/top/v1`
  - `/orders/v1/{chainId}/{tokenAddress}`

## Primary Endpoints

### 1. Get Pair by Chain and Pair Address
```
GET /latest/dex/pairs/{chainId}/{pairId}
```
- **Parameters:**
  - `chainId`: Blockchain identifier (e.g., "solana")
  - `pairId`: Pair address (e.g., "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN")

### 2. Search Pairs
```
GET /latest/dex/search?q={query}
```
- **Parameters:**
  - `q`: Search query (e.g., "SOL/USDC", "BONK")

### 3. Get Pools for Token Address
```
GET /token-pairs/v1/{chainId}/{tokenAddress}
```
- **Parameters:**
  - `chainId`: Blockchain identifier (e.g., "solana")
  - `tokenAddress`: Token address

### 4. Get Multiple Tokens (Batch)
```
GET /tokens/v1/{chainId}/{tokenAddresses}
```
- **Parameters:**
  - `chainId`: Blockchain identifier (e.g., "solana")
  - `tokenAddresses`: Comma-separated token addresses (up to 30)

## Response Structure

### Pair Data Structure
```json
{
  "schemaVersion": "1.0.0",
  "pairs": [
    {
      "chainId": "solana",
      "dexId": "raydium",
      "url": "https://dexscreener.com/solana/...",
      "pairAddress": "...",
      "labels": ["v3"],
      "baseToken": {
        "address": "...",
        "name": "Token Name",
        "symbol": "SYMBOL"
      },
      "quoteToken": {
        "address": "...",
        "name": "USD Coin",
        "symbol": "USDC"
      },
      "priceNative": "0.123456",
      "priceUsd": "1.23",
      "txns": {
        "m5": { "buys": 10, "sells": 5 },
        "h1": { "buys": 100, "sells": 50 },
        "h6": { "buys": 600, "sells": 300 },
        "h24": { "buys": 2400, "sells": 1200 }
      },
      "volume": {
        "h24": 1000000,
        "h6": 250000,
        "h1": 50000,
        "m5": 5000
      },
      "priceChange": {
        "m5": 0.05,
        "h1": 0.15,
        "h6": 0.25,
        "h24": 0.35
      },
      "liquidity": {
        "usd": 500000,
        "base": 1000000,
        "quote": 500000
      },
      "fdv": 10000000,
      "marketCap": 8000000,
      "pairCreatedAt": 1640995200,
      "info": {
        "imageUrl": "https://...",
        "websites": [{"url": "https://..."}],
        "socials": [{"platform": "twitter", "handle": "@..."}]
      },
      "boosts": {
        "active": 3
      }
    }
  ]
}
```

## Key Data Points

### Trading Metrics
- **priceUsd**: Current price in USD
- **priceNative**: Price in native units
- **volume**: Trading volume for different time periods (m5, h1, h6, h24)
- **txns**: Transaction counts (buys/sells) for different time periods
- **priceChange**: Price change percentage for different time periods

### Liquidity Metrics
- **liquidity.usd**: Total liquidity in USD
- **liquidity.base**: Liquidity in base token
- **liquidity.quote**: Liquidity in quote token
- **fdv**: Fully Diluted Valuation
- **marketCap**: Market Capitalization

### Pool Information
- **chainId**: Blockchain (e.g., "solana")
- **dexId**: DEX identifier (e.g., "raydium", "jupiter", "orca")
- **pairAddress**: Smart contract address of the pair
- **pairCreatedAt**: Unix timestamp of pair creation
- **labels**: Additional labels (e.g., ["v3"] for Raydium v3)

## Integration in SniperForge

### Current Implementation
Located in `src/shared/alternative_apis.rs`:

```rust
pub async fn fetch_dexscreener_pairs(&self, tokens: &[String]) -> Result<Vec<DexscreenerPair>> {
    // Uses the batch endpoint: /tokens/v1/solana/{comma_separated_tokens}
    // Supports up to 30 tokens per request
}
```

### Use Cases
1. **Pool Discovery**: Find all pools for a given token
2. **Price Validation**: Cross-reference prices with other sources
3. **Liquidity Analysis**: Assess pool liquidity before trading
4. **Volume Analysis**: Check trading activity and trends
5. **Market Intelligence**: Identify trending tokens and pairs

### Best Practices
1. **Batch Requests**: Use up to 30 tokens per request for efficiency
2. **Rate Limiting**: Respect the 300 req/min limit
3. **Error Handling**: Handle API failures gracefully with fallbacks
4. **Data Freshness**: Cache responses appropriately (data updates frequently)
5. **Cross-Validation**: Use DexScreener data alongside Jupiter/Raydium APIs

## Example Usage

### Fetch Multiple Token Pairs
```rust
let tokens = vec![
    "So11111111111111111111111111111111111111112".to_string(), // SOL
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
];

let pairs = alternative_apis.fetch_dexscreener_pairs(&tokens).await?;
for pair in pairs {
    println!("Pair: {}/{} - Price: ${} - Liquidity: ${}", 
        pair.base_token.symbol,
        pair.quote_token.symbol,
        pair.price_usd,
        pair.liquidity.usd
    );
}
```

## Advanced Features

### Token Profiles
- Get enhanced token information including logos, descriptions, and social links
- Endpoint: `/token-profiles/latest/v1`
- Rate limit: 60 req/min

### Token Boosts
- Information about promoted/boosted tokens
- Latest boosts: `/token-boosts/latest/v1`
- Top boosted: `/token-boosts/top/v1`
- Rate limit: 60 req/min

### Order Status
- Check status of paid promotional orders
- Endpoint: `/orders/v1/{chainId}/{tokenAddress}`
- Rate limit: 60 req/min

## Error Handling

### Common HTTP Status Codes
- **200**: Success
- **400**: Bad Request (invalid parameters)
- **429**: Rate limit exceeded
- **500**: Internal server error

### Retry Strategy
- Implement exponential backoff for rate limit errors
- Use circuit breaker pattern for persistent failures
- Fall back to other APIs when DexScreener is unavailable

## Integration Benefits
1. **Comprehensive Data**: Rich market data including social metrics
2. **Multi-DEX Coverage**: Data from multiple DEXs in one API
3. **Real-time Updates**: Frequently updated price and volume data
4. **Free Access**: No API key or payment required
5. **Reliable Infrastructure**: High uptime and performance

## Limitations
1. **Rate Limits**: 300 requests per minute for main endpoints
2. **No WebSocket**: Real-time data requires polling
3. **Solana Focus**: While multi-chain, Solana coverage may vary
4. **Data Latency**: Small delay compared to direct DEX APIs

## Future Enhancements
1. **Advanced Filtering**: Implement liquidity and volume filters
2. **Historical Data**: Add support for historical price/volume data
3. **Alert System**: Monitor significant price or volume changes
4. **Analytics Dashboard**: Create visual representations of DexScreener data
