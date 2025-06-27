use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 VERIFICACIÓN DE TOKENS EN DEVNET");
    println!("==================================================");
    
    let devnet_client = RpcClient::new("https://api.devnet.solana.com".to_string());
    let mainnet_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
    
    let tokens = vec![
        ("USDC (Jupiter quote)", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"),
        ("SOL", "So11111111111111111111111111111111111111112"),
        ("DevNet USDC (known)", "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr"),
        ("DevNet USDC (alt)", "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"),
        ("USDT", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
        ("RAY", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"), // Este parece estar mal en el output
    ];
    
    for (name, address_str) in tokens {
        println!("\n📊 {}: {}", name, address_str);
        
        if let Ok(pubkey) = Pubkey::from_str(address_str) {
            // Verificar en DevNet
            match devnet_client.get_account(&pubkey) {
                Ok(account) => {
                    println!("   DevNet: ✅ EXISTS");
                    println!("   Owner: {}", account.owner);
                    if account.owner.to_string() == "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" {
                        println!("   ✅ Owned by SPL Token Program");
                    } else {
                        println!("   ⚠️  Different owner: {}", account.owner);
                    }
                },
                Err(_) => println!("   DevNet: ❌ NOT FOUND"),
            }
            
            // Verificar en Mainnet
            match mainnet_client.get_account(&pubkey) {
                Ok(_) => println!("   Mainnet: ✅ EXISTS"),
                Err(_) => println!("   Mainnet: ❌ NOT FOUND"),
            }
        } else {
            println!("   ❌ Invalid pubkey format");
        }
    }
    
    Ok(())
}
