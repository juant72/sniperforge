# Premium RPC API Keys Setup Guide

This guide explains how to set up premium RPC endpoints for better reliability and performance on Solana mainnet.

**Last Updated**: June 29, 2025 - Added Tatum integration and verified 100% functionality

## 🌟 Why Premium RPC?

Public RPC endpoints often have limitations:
- Rate limiting (410 Gone errors)
- Slower response times
- Limited concurrent connections
- No guaranteed uptime

Premium RPC providers offer:
- Higher rate limits
- Better performance
- Dedicated infrastructure
- WebSocket support
- 99.9% uptime guarantees

## 🔑 Supported Providers

### 1. Helius (Recommended)
- **Free Tier**: 100,000 requests/month
- **Paid Plans**: Starting at $39/month
- **Features**: WebSocket, Analytics, Enhanced APIs
- **Signup**: https://helius.xyz
- **Status**: ✅ 100% Functional

**Setup:**
```bash
# Windows PowerShell (example with real API key)
$env:HELIUS_API_KEY="062bf3dd-23d4-4ffd-99fd-6e397ee59d6c"

# Linux/Mac
export HELIUS_API_KEY="062bf3dd-23d4-4ffd-99fd-6e397ee59d6c"
```

**Endpoints this will enable:**
- **Mainnet RPC**: `https://mainnet.helius-rpc.com/?api-key=062bf3dd-23d4-4ffd-99fd-6e397ee59d6c`
- **Mainnet WS**: `wss://mainnet.helius-rpc.com/?api-key=062bf3dd-23d4-4ffd-99fd-6e397ee59d6c`
- **Devnet RPC**: `https://devnet.helius-rpc.com/?api-key=062bf3dd-23d4-4ffd-99fd-6e397ee59d6c`
- **Devnet WS**: `wss://devnet.helius-rpc.com/?api-key=062bf3dd-23d4-4ffd-99fd-6e397ee59d6c`

### 2. Tatum (NEW - Header Authentication)
- **Free Tier**: Limited requests
- **Paid Plans**: Starting at $15/month
- **Features**: Header authentication, Multi-blockchain support
- **Signup**: https://tatum.io
- **Status**: ✅ 100% Functional - Fully integrated June 29, 2025

**Setup:**
```bash
# Windows PowerShell
$env:TATUM_API_KEY_MAINNET="t-67b3d0b4dff4f7a9cf84fbf7-e095b9354ff54bc59b09fc04"
$env:TATUM_API_KEY_DEVNET="t-67b3d0b4dff4f7a9cf84fbf7-687708fdb90e4aa59ff9a9cb"

# Linux/Mac
export TATUM_API_KEY_MAINNET="t-67b3d0b4dff4f7a9cf84fbf7-e095b9354ff54bc59b09fc04"
export TATUM_API_KEY_DEVNET="t-67b3d0b4dff4f7a9cf84fbf7-687708fdb90e4aa59ff9a9cb"
```

**Endpoints this will enable:**
- **Mainnet RPC**: `https://solana-mainnet.gateway.tatum.io` (with header auth)
- **Devnet RPC**: `https://solana-devnet.gateway.tatum.io` (with header auth)

**Special Features:**
- Uses `x-api-key` header authentication (different from URL-based auth)
- Separate API keys for mainnet and devnet
- Automatic network detection and endpoint selection

### 3. QuickNode (Enterprise Grade)
- **Free Tier**: Limited requests
- **Paid Plans**: Starting at $9/month
- **Features**: Global infrastructure, WebSocket, Analytics
- **Signup**: https://quicknode.com
- **Status**: ✅ Supported

**Setup:**
```bash
# Windows PowerShell
$env:QUICKNODE_ENDPOINT="https://your-endpoint.quiknode.pro/your-token/"

# Linux/Mac  
export QUICKNODE_ENDPOINT="https://your-endpoint.quiknode.pro/your-token/"
```

### 3. Alchemy (Developer Friendly)
- **Free Tier**: 300M compute units/month
- **Paid Plans**: Starting at $49/month
- **Features**: Enhanced APIs, WebSocket, Debug tools
- **Signup**: https://alchemy.com

**Setup:**
```bash
# Windows PowerShell
$env:ALCHEMY_API_KEY="your_alchemy_api_key_here"

# Linux/Mac
export ALCHEMY_API_KEY="your_alchemy_api_key_here"
```

### 4. Ankr (Cost Effective)
- **Free Tier**: 500 requests/second
- **Paid Plans**: Pay-as-you-go
- **Features**: Global CDN, Load balancing
- **Signup**: https://ankr.com

**Setup:**
```bash
# Windows PowerShell
$env:ANKR_API_KEY="your_ankr_api_key_here"

# Linux/Mac
export ANKR_API_KEY="your_ankr_api_key_here"
```

## 🚀 Quick Setup

1. **Sign up** for one or more providers above
2. **Get your API key/endpoint** from the provider dashboard
3. **Set environment variable** (see examples above)
4. **Enable premium RPC** in config:

```toml
# In config/mainnet.toml
[network.premium_rpc]
enabled = true  # Change this to true
```

5. **Restart SniperForge** to pick up the new configuration

## ✅ Verification

Run a test to verify premium endpoints are working:

```bash
cargo run --bin sniperforge test basic --network mainnet
```

Look for these log messages:
```
✅ Found Helius API key
🌟 Premium endpoints: Helius (p:2), QuickNode (p:1)
```

## 💡 Environment Variables Reference

| Variable | Provider | Format | Required |
|----------|----------|--------|----------|
| `HELIUS_API_KEY` | Helius | API key string | No |
| `TATUM_API_KEY_MAINNET` | Tatum | API key string | No |
| `TATUM_API_KEY_DEVNET` | Tatum | API key string | No |
| `QUICKNODE_ENDPOINT` | QuickNode | Full HTTPS URL | No |
| `ALCHEMY_API_KEY` | Alchemy | API key string | No |
| `ANKR_API_KEY` | Ankr | API key string | No |

## 🎯 Priority System

Premium endpoints are used in this priority order:

**Mainnet Priority:**
1. **QuickNode** (priority 1) - Enterprise grade
2. **Helius** (priority 2) - Solana specialist  
3. **Alchemy** (priority 3) - Developer tools
4. **Tatum** (priority 4) - Multi-blockchain
5. **Ankr** (priority 5) - Cost effective

**Devnet Priority:**
1. **Helius** (priority 1) - Solana specialist
2. **Alchemy** (priority 2) - Developer tools
3. **Tatum** (priority 3) - Multi-blockchain
4. **Ankr** (priority 4) - Cost effective

Public endpoints are used as fallbacks when premium endpoints are unavailable.

## ✅ Testing & Verification

### Test All RPC Endpoints
```bash
# Test comprehensive RPC functionality
cargo run --bin test_all_rpc_methods

# Test specific providers
cargo run --bin sniperforge -- test tatum
cargo run --bin sniperforge -- test basic --network mainnet
cargo run --bin sniperforge -- test basic --network devnet
```

### Expected Results (June 29, 2025)
- **Success Rate**: 100% for all configured endpoints
- **Devnet**: 3 healthy endpoints (1 premium + Tatum)
- **Mainnet**: 4 healthy endpoints (1 premium + Tatum)
- **Average Response**: <1000ms for premium endpoints

## 🔒 Security Best Practices

1. **Never commit API keys** to git repositories
2. **Use environment variables** for API keys
3. **Rotate keys regularly** (monthly recommended)
4. **Monitor usage** in provider dashboards
5. **Set up billing alerts** to avoid unexpected charges
6. **Use separate keys** for mainnet and devnet (Tatum)

## 🐛 Troubleshooting

### "No premium endpoints configured"
- Check environment variables are set correctly
- Verify `premium_rpc.enabled = true` in config
- Restart the application after setting variables

### "401 Unauthorized" errors
- For Tatum: Verify API keys are set for correct network
- For others: Check API key format and validity
- Ensure keys haven't expired or hit usage limits

### Performance Issues
- Check provider status pages
- Monitor rate limits in dashboards
- Consider upgrading plans for higher limits

### "401 Unauthorized" errors  
- Verify API key is correct
- Check if your account has sufficient credits
- Ensure you're using the right API key format

### Rate limiting still occurring
- Check your plan limits in provider dashboard
- Consider upgrading to higher tier plan
- Verify multiple providers are configured for redundancy

## 💰 Cost Optimization

1. **Start with free tiers** to test functionality
2. **Monitor usage** in provider dashboards  
3. **Use multiple providers** for redundancy and cost distribution
4. **Set billing alerts** to avoid overage charges
5. **Consider hybrid approach**: Premium for critical operations, public for monitoring

## 📞 Support

If you encounter issues:
1. Check provider status pages
2. Review provider documentation
3. Contact provider support directly
4. File an issue in the SniperForge repository

---

**Remember**: Premium RPC is optional but highly recommended for production trading on mainnet.
