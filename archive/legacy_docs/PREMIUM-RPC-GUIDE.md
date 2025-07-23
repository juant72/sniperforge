# 🌟 Premium RPC Setup Guide
## Getting Real API Keys for Production Trading

### 📋 Quick Setup Checklist

✅ **Step 1**: Choose your premium RPC provider(s)  
✅ **Step 2**: Sign up and get API keys  
✅ **Step 3**: Configure environment variables  
✅ **Step 4**: Enable premium RPC in config  
✅ **Step 5**: Test the integration  

---

## 🏆 Recommended Providers (Ranked by Performance)

### 1. 🥇 **Helius** (Highly Recommended)
- **Why**: Fastest response times, dedicated Solana infrastructure
- **Best for**: High-frequency trading, low latency requirements
- **Pricing**: Free tier available, paid plans from $99/month
- **Website**: https://helius.xyz
- **Sign up**: Create account → Dashboard → API Keys → Copy key

**Configuration:**
```powershell
$env:HELIUS_API_KEY = "your-helius-api-key-here"
```

### 2. 🥈 **QuickNode** 
- **Why**: Reliable infrastructure, good Solana support
- **Best for**: Production trading, reliable connections
- **Pricing**: Free tier available, paid plans from $9/month
- **Website**: https://quicknode.com
- **Sign up**: Create account → Create Solana endpoint → Copy full URL

**Configuration:**
```powershell
$env:QUICKNODE_ENDPOINT = "https://your-quicknode-url.quiknode.pro/your-key/"
```

### 3. 🥉 **Alchemy**
- **Why**: Growing Solana support, good documentation
- **Best for**: Development and testing
- **Pricing**: Free tier available, paid plans from $49/month
- **Website**: https://alchemy.com
- **Sign up**: Create account → Create Solana app → Copy API key

**Configuration:**
```powershell
$env:ALCHEMY_API_KEY = "your-alchemy-api-key-here"
```

### 4. **Ankr**
- **Why**: Affordable option, basic functionality
- **Best for**: Budget-conscious setups
- **Pricing**: Free tier available, paid plans from $15/month
- **Website**: https://ankr.com
- **Sign up**: Create account → API → Create Solana endpoint → Copy key

**Configuration:**
```powershell
$env:ANKR_API_KEY = "your-ankr-api-key-here"
```

---

## 🚀 Setup Instructions

### Option A: Quick Setup (Recommended)
Use our automated setup script:
```powershell
.\setup-premium-rpc.ps1
```

### Option B: Manual Setup

1. **Get API Keys** from providers above
2. **Set Environment Variables** (choose one method):

#### Method 1: Temporary (This Session Only)
```powershell
# Set one or more API keys
$env:HELIUS_API_KEY = "your-helius-api-key"
$env:QUICKNODE_ENDPOINT = "https://your-quicknode-url.com"
$env:ALCHEMY_API_KEY = "your-alchemy-api-key"
$env:ANKR_API_KEY = "your-ankr-api-key"
```

#### Method 2: Permanent (All Sessions)
```powershell
# Set permanently for your user account
[Environment]::SetEnvironmentVariable("HELIUS_API_KEY", "your-helius-api-key", "User")
[Environment]::SetEnvironmentVariable("QUICKNODE_ENDPOINT", "https://your-quicknode-url.com", "User")
[Environment]::SetEnvironmentVariable("ALCHEMY_API_KEY", "your-alchemy-api-key", "User")
[Environment]::SetEnvironmentVariable("ANKR_API_KEY", "your-ankr-api-key", "User")
```

3. **Enable Premium RPC** in configuration:
Edit `config/mainnet.toml`:
```toml
[network.premium_rpc]
enabled = true  # Change from false to true
```

4. **Test the Setup**:
```powershell
.\test-premium-rpc.ps1
```

---

## ✅ Verification

After setup, you should see these messages when running tests:

```
✅ Found Helius API key
✅ Loaded X premium RPC endpoints
🌟 Premium endpoints: Helius (p:2), QuickNode (p:1)
📡 Premium URLs: ["https://mainnet.helius-rpc.com/?api-key=..."]
✅ Premium RPC 0 is working
✅ Found X healthy RPC endpoints (Y premium)
```

---

## 🔧 Troubleshooting

### Common Issues:

1. **"Premium RPC disabled in configuration"**
   - ✅ Set `enabled = true` in `config/mainnet.toml`

2. **"401 Unauthorized" errors**
   - ✅ Check API key is correct
   - ✅ Verify API key has Solana mainnet access
   - ✅ Check your account has sufficient credits

3. **"No premium API keys found"**
   - ✅ Set environment variables correctly
   - ✅ Restart PowerShell after setting variables
   - ✅ Check variable names match exactly

4. **Premium endpoints not being used**
   - ✅ Verify API keys are valid
   - ✅ Check network connectivity
   - ✅ Review logs for error messages

### Testing Commands:
```powershell
# Check environment variables
Get-ChildItem Env: | Where-Object {$_.Name -like "*API*" -or $_.Name -like "*QUICKNODE*"}

# Test with verbose logging
cargo run --bin sniperforge -- test solana --network mainnet

# Test specific RPC functionality
cargo run --bin sniperforge -- test websocket --network mainnet
```

---

## 💰 Cost Considerations

### Free Tiers (Good for Testing):
- **Helius**: 100,000 requests/month
- **QuickNode**: 100 requests/day
- **Alchemy**: 300M compute units/month
- **Ankr**: 500 requests/day

### Recommended for Production:
- **Helius Growth**: $99/month (2M requests, priority support)
- **QuickNode Build**: $9/month (unlimited requests, standard support)

### 💡 Pro Tips:
1. **Start with Helius free tier** for testing
2. **Use multiple providers** for maximum reliability
3. **Monitor usage** to avoid unexpected charges
4. **Set up billing alerts** on provider dashboards

---

## 🎯 Performance Benefits

With premium RPC endpoints, you'll see:
- ✅ **50-90% faster** response times
- ✅ **99.9% uptime** vs 95% with public RPCs
- ✅ **No rate limiting** on pool queries
- ✅ **Priority routing** for time-sensitive trades
- ✅ **Dedicated bandwidth** for your requests

---

## 🔐 Security Best Practices

1. **Never commit API keys** to version control
2. **Use environment variables** only
3. **Rotate keys regularly** (every 90 days)
4. **Monitor usage** for suspicious activity
5. **Set up IP restrictions** when available
6. **Use separate keys** for dev/staging/production

---

*For more help, see `docs/user-guides/premium-rpc-setup.md` or run `.\setup-premium-rpc.ps1`*
