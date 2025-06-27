#!/usr/bin/env python3
"""
Verificar tokens en DevNet vs Mainnet
"""
import requests
import json

def check_account(address, network="devnet"):
    """Verificar si una cuenta existe en la red especificada"""
    url = f"https://api.{network}.solana.com" if network == "devnet" else "https://api.mainnet-beta.solana.com"
    
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAccountInfo",
        "params": [
            address,
            {"encoding": "base64"}
        ]
    }
    
    response = requests.post(url, json=payload)
    data = response.json()
    
    if "error" in data:
        return False, data["error"]
    
    return data["result"]["value"] is not None, data["result"]

def main():
    # Tokens para verificar
    tokens = {
        "USDC (current)": "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",
        "RAY": "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",  # Esto parece estar mal
        "SOL": "So11111111111111111111111111111111111111112",
        "DevNet USDC (known)": "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr",
        "DevNet USDC (alt)": "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"
    }
    
    print("🔍 VERIFICACIÓN DE TOKENS EN DEVNET")
    print("=" * 50)
    
    for name, address in tokens.items():
        print(f"\n📊 {name}: {address}")
        
        # DevNet
        exists_devnet, result_devnet = check_account(address, "devnet")
        print(f"   DevNet: {'✅ EXISTS' if exists_devnet else '❌ NOT FOUND'}")
        
        # Mainnet
        exists_mainnet, result_mainnet = check_account(address, "mainnet-beta")
        print(f"   Mainnet: {'✅ EXISTS' if exists_mainnet else '❌ NOT FOUND'}")
        
        if exists_devnet and result_devnet["value"]:
            # Verificar owner del token
            account_data = result_devnet["value"]
            if account_data and "owner" in account_data:
                owner = account_data["owner"]
                print(f"   Owner: {owner}")
                if owner == "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA":
                    print("   ✅ Owned by SPL Token Program")
                else:
                    print("   ⚠️  Different owner")

if __name__ == "__main__":
    main()
