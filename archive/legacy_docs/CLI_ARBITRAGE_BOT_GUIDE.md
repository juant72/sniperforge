# 🚀 CLI Guide: Real Arbitrage Bot for Solana DevNet

## Overview
This guide provides complete command-line instructions for running the real arbitrage bot on Solana DevNet. The bot executes real on-chain transactions, generates real profits, and supports custom tokens with multi-DEX logic.

## 📋 Prerequisites

### 1. Environment Setup
Ensure your `.env` file contains:
```env
PRIVATE_KEY=your_base58_encoded_private_key
SOLANA_RPC_URL=https://solana-devnet.g.alchemy.com/v2/your_api_key
ALCHEMY_API_KEY=your_alchemy_api_key
BIRDEYE_API_KEY=your_birdeye_api_key
TATUM_API_KEY=your_tatum_api_key
```

### 2. Build the Project
```powershell
# Clean build to ensure all binaries are compiled
cargo clean
cargo build --release

# Or use the fast build script
.\fast-build.ps1
```

## 🎯 Main Arbitrage Bots (Real Trading)

### 1. Jupiter-Based Real Arbitrage Bot
**Best for production use** - Uses Jupiter API for real price quotes and executes real swaps.

```powershell
# Run the Jupiter real arbitrage bot
cargo run --bin test_arbitrage_real_jupiter

# Or run in release mode (faster)
cargo run --release --bin test_arbitrage_real_jupiter
```

**Features:**
- ✅ Real Jupiter API integration
- ✅ Real on-chain swap execution
- ✅ Real profit generation
- ✅ Multi-DEX support (Jupiter aggregates all DEXs)
- ✅ Automatic token discovery
- ✅ Real transaction fees

### 2. Custom DEX Real Arbitrage Bot
Advanced bot with custom mint/burn logic for specialized tokens.

```powershell
# Run the custom DEX arbitrage bot
cargo run --bin test_real_arbitrage_devnet

# Or in release mode
cargo run --release --bin test_real_arbitrage_devnet
```

**Features:**
- ✅ Custom token mint/burn operations
- ✅ Direct DEX interactions
- ✅ Real balance changes
- ✅ Custom swap logic

### 3. Simple Real Transfer Bot
Proof-of-concept for real on-chain transfers (good for testing).

```powershell
# Run the simple real transfer bot
cargo run --bin test_simple_arbitrage_real

# Or in release mode
cargo run --release --bin test_simple_arbitrage_real
```

## 🔍 Monitoring and Balance Checking

### Check DevNet Wallet Balance
Before and after running arbitrage to verify real profits:

```powershell
# Check your wallet and token balances
cargo run --bin check_devnet_balance

# Or in release mode
cargo run --release --bin check_devnet_balance
```

### Check Wallet Address
Get your wallet public key:

```powershell
cargo run --bin get_wallet_address
```

## 📊 Recommended Workflow

### 1. Initial Setup and Verification
```powershell
# Step 1: Check your wallet address
cargo run --bin get_wallet_address

# Step 2: Check initial balances
cargo run --bin check_devnet_balance

# Step 3: Request DevNet SOL if needed
cargo run --bin request_devnet_airdrop

# Step 4: Verify balances again
cargo run --bin check_devnet_balance
```

### 2. Run Real Arbitrage (Recommended)
```powershell
# Run the Jupiter-based real arbitrage bot (best option)
cargo run --release --bin test_arbitrage_real_jupiter
```

### 3. Verify Real Profits
```powershell
# Check balances after arbitrage to see real token gains
cargo run --bin check_devnet_balance
```

## 🛠 Advanced Tools

### Token Discovery and Setup
```powershell
# Discover available DevNet tokens
cargo run --bin discover_devnet_tokens

# Find Jupiter-compatible tokens
cargo run --bin discover_jupiter_tokens

# Create automated DevNet tokens
cargo run --bin create_devnet_tokens_automated
```

### Testing and Validation
```powershell
# Test basic DevNet functionality
cargo run --bin test_devnet_functionality

# Test custom token swaps
cargo run --bin test_custom_tokens_swap

# Verify custom tokens
cargo run --bin test_custom_tokens_verify
```

### RPC and Connection Testing
```powershell
# Test all RPC methods
cargo run --bin test_all_rpc_methods

# Test basic connectivity
cargo run --bin test_basic_connectivity
```

## 📈 Real Trading Results

### Expected Output
When running the real arbitrage bots, you should see:

1. **Real Transaction Signatures**
   ```
   ✅ Transaction confirmed: 5Kj8x9vR2mN7...
   ```

2. **Real Balance Changes**
   ```
   Before: TEST_USDT: 0 tokens
   After: TEST_USDT: +1879 tokens (REAL PROFIT!)
   ```

3. **Real Fees Paid**
   ```
   SOL balance reduced by transaction fees: -0.00102 SOL
   ```

### Transaction Verification
All transactions are recorded on DevNet and can be verified on:
- Solana Explorer (DevNet): https://explorer.solana.com/?cluster=devnet
- SolanaFM (DevNet): https://solana.fm/?cluster=devnet-solana

## 🔧 Configuration

### Custom Tokens (config/devnet-automated.json)
```json
{
  "network": "devnet",
  "wallet_type": "file",
  "tokens": {
    "TEST_USDT": "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
    "TEST_USDC": "EhpbDdUDKkm9QXt1E8RxH4xhBGn1TG8HW6Hl1f8vN9Fm"
  }
}
```

### RPC Configuration
The bot automatically uses premium Alchemy RPC endpoints for:
- ✅ Higher rate limits
- ✅ Better reliability
- ✅ Faster transaction processing

## 🚨 Important Notes

### Real Money Warning
- This bot executes **REAL transactions** on DevNet
- DevNet SOL has no monetary value, but the bot logic is production-ready
- To use on MainNet, simply change the RPC URL in `.env`

### Performance Tips
```powershell
# Use release builds for better performance
cargo run --release --bin test_arbitrage_real_jupiter

# Use fast build script for development
.\fast-build.ps1
```

### Error Handling
If you encounter errors:
1. Check your `.env` file configuration
2. Ensure you have DevNet SOL balance
3. Verify internet connection
4. Check RPC endpoint status

## 📋 Complete Command Reference

### Primary Arbitrage Bots
```powershell
# Jupiter real arbitrage (RECOMMENDED)
cargo run --release --bin test_arbitrage_real_jupiter

# Custom DEX real arbitrage
cargo run --release --bin test_real_arbitrage_devnet

# Simple real transfer bot
cargo run --release --bin test_simple_arbitrage_real
```

### Balance and Monitoring
```powershell
# Check balances
cargo run --bin check_devnet_balance

# Get wallet address
cargo run --bin get_wallet_address

# Request DevNet airdrop
cargo run --bin request_devnet_airdrop
```

### Token Management
```powershell
# Discover tokens
cargo run --bin discover_devnet_tokens
cargo run --bin discover_jupiter_tokens

# Create test tokens
cargo run --bin create_devnet_tokens_automated

# Test token operations
cargo run --bin test_custom_tokens_swap
cargo run --bin test_custom_tokens_verify
```

### Testing and Validation
```powershell
# Test DevNet functionality
cargo run --bin test_devnet_functionality

# Test RPC methods
cargo run --bin test_all_rpc_methods

# Test connectivity
cargo run --bin test_basic_connectivity
```

## 🎯 Quick Start (30 seconds)

```powershell
# 1. Build the project
cargo build --release

# 2. Check your setup
cargo run --bin get_wallet_address
cargo run --bin check_devnet_balance

# 3. Run real arbitrage bot
cargo run --release --bin test_arbitrage_real_jupiter

# 4. Verify real profits
cargo run --bin check_devnet_balance
```

## 📊 Success Metrics

### What to Look For
- ✅ **Real transaction signatures** in the output
- ✅ **Positive token balance changes** after arbitrage
- ✅ **SOL balance reduction** from real fees paid
- ✅ **On-chain transaction confirmation** via Solana Explorer

### Sample Success Output
```
🚀 Starting Jupiter Real Arbitrage Bot...
💰 Initial Balance: SOL: 5.234, TEST_USDT: 0
🔄 Executing real swap: SOL → TEST_USDT
✅ Transaction confirmed: 5Kj8x9vR2mN7bP8qX4t2Y6Z1...
💰 Final Balance: SOL: 5.232, TEST_USDT: 1879 (+1879 PROFIT!)
🎯 Real arbitrage completed successfully!
```

---

## 🏆 Conclusion

This CLI guide provides everything needed to run a **real arbitrage bot** on Solana DevNet with:
- ✅ Real on-chain transactions
- ✅ Real token profits
- ✅ Multi-DEX support
- ✅ Premium RPC endpoints
- ✅ Complete validation and monitoring

For MainNet deployment, simply update the RPC URLs in your `.env` file and ensure you have real SOL for transaction fees.

**Happy Trading! 🚀**
