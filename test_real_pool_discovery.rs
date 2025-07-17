use solana_sdk::{
    pubkey::Pubkey,
    commitment_config::CommitmentConfig,
};
use solana_client::rpc_client::RpcClient;
use std::str::FromStr;
use anyhow::{Result, anyhow};

// Constantes de programas DEX
const RAYDIUM_AMM_PROGRAM: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
const ORCA_WHIRLPOOL_PROGRAM: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";

#[derive(Debug, Clone)]
struct RealPool {
    address: Pubkey,
    program: String,
    token_a: Pubkey,
    token_b: Pubkey,
    reserves_a: u64,
    reserves_b: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 REAL POOL DISCOVERY TEST");
    println!("============================");
    
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    
    // Buscar pools reales de Raydium
    println!("🚀 Searching for real Raydium pools...");
    let raydium_pools = find_real_raydium_pools(&client).await?;
    
    // Buscar pools reales de Orca Whirlpool
    println!("🌊 Searching for real Orca Whirlpool pools...");
    let orca_pools = find_real_orca_pools(&client).await?;
    
    // Mostrar resultados
    println!("\n📊 REAL POOLS FOUND:");
    println!("=====================");
    
    let mut total_pools = 0;
    
    if !raydium_pools.is_empty() {
        println!("\n🎯 RAYDIUM POOLS ({}):", raydium_pools.len());
        for (i, pool) in raydium_pools.iter().enumerate() {
            println!("  {}. {} ({}...)", 
                i + 1, 
                pool.address.to_string(),
                &pool.address.to_string()[..8]
            );
            println!("     Token A: {} ({:.6})", pool.token_a, pool.reserves_a as f64 / 1_000_000.0);
            println!("     Token B: {} ({:.6})", pool.token_b, pool.reserves_b as f64 / 1_000_000.0);
            total_pools += 1;
        }
    }
    
    if !orca_pools.is_empty() {
        println!("\n🌊 ORCA WHIRLPOOL POOLS ({}):", orca_pools.len());
        for (i, pool) in orca_pools.iter().enumerate() {
            println!("  {}. {} ({}...)", 
                i + 1, 
                pool.address.to_string(),
                &pool.address.to_string()[..8]
            );
            println!("     Token A: {} ({:.6})", pool.token_a, pool.reserves_a as f64 / 1_000_000.0);
            println!("     Token B: {} ({:.6})", pool.token_b, pool.reserves_b as f64 / 1_000_000.0);
            total_pools += 1;
        }
    }
    
    println!("\n✅ SUMMARY:");
    println!("   Total real pools found: {}", total_pools);
    println!("   Raydium pools: {}", raydium_pools.len());
    println!("   Orca pools: {}", orca_pools.len());
    
    if total_pools > 0 {
        println!("\n🎉 SUCCESS: Found real pools that can be used for arbitrage!");
        
        // Guardar las direcciones reales para uso futuro
        save_real_pool_addresses(&raydium_pools, &orca_pools)?;
    } else {
        println!("\n❌ FAILED: No real pools found");
    }
    
    Ok(())
}

async fn find_real_raydium_pools(client: &RpcClient) -> Result<Vec<RealPool>> {
    let mut real_pools = Vec::new();
    
    let program_id = Pubkey::from_str(RAYDIUM_AMM_PROGRAM)?;
    
    // Obtener todas las cuentas del programa Raydium
    println!("Fetching Raydium program accounts...");
    let accounts = client.get_program_accounts_with_config(
        &program_id,
        solana_client::rpc_config::RpcProgramAccountsConfig {
            filters: Some(vec![
                solana_client::rpc_filter::RpcFilterType::DataSize(752), // Tamaño típico de pool Raydium
            ]),
            account_config: solana_client::rpc_config::RpcAccountInfoConfig {
                encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    
    match accounts {
        Ok(accounts) => {
            println!("Found {} Raydium accounts", accounts.len());
            
            for (address, account) in accounts.into_iter().take(5) { // Limitar a 5 para test
                if let Ok(pool) = parse_raydium_pool(address, &account).await {
                    real_pools.push(pool);
                }
            }
        }
        Err(e) => {
            println!("Error fetching Raydium accounts: {}", e);
        }
    }
    
    Ok(real_pools)
}

async fn find_real_orca_pools(client: &RpcClient) -> Result<Vec<RealPool>> {
    let mut real_pools = Vec::new();
    
    let program_id = Pubkey::from_str(ORCA_WHIRLPOOL_PROGRAM)?;
    
    // Obtener cuentas del programa Orca Whirlpool
    println!("Fetching Orca Whirlpool program accounts...");
    let accounts = client.get_program_accounts_with_config(
        &program_id,
        solana_client::rpc_config::RpcProgramAccountsConfig {
            filters: Some(vec![
                solana_client::rpc_filter::RpcFilterType::DataSize(653), // Tamaño típico de Whirlpool
            ]),
            account_config: solana_client::rpc_config::RpcAccountInfoConfig {
                encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    
    match accounts {
        Ok(accounts) => {
            println!("Found {} Orca Whirlpool accounts", accounts.len());
            
            for (address, account) in accounts.into_iter().take(5) { // Limitar a 5 para test
                if let Ok(pool) = parse_orca_whirlpool(address, &account).await {
                    real_pools.push(pool);
                }
            }
        }
        Err(e) => {
            println!("Error fetching Orca accounts: {}", e);
        }
    }
    
    Ok(real_pools)
}

async fn parse_raydium_pool(address: Pubkey, account: &solana_sdk::account::Account) -> Result<RealPool> {
    let data = &account.data;
    
    if data.len() < 680 {
        return Err(anyhow!("Invalid Raydium pool data length"));
    }
    
    // Verificar status (debe ser 6 o 7 para pools activos)
    let status = u64::from_le_bytes(data[0..8].try_into().unwrap());
    if status != 6 && status != 7 {
        return Err(anyhow!("Invalid AMM status: {}", status));
    }
    
    // Offsets según documentación oficial de Raydium
    let coin_mint_offset = 296;
    let pc_mint_offset = 328;
    
    let token_a = Pubkey::new_from_array(
        data[coin_mint_offset..coin_mint_offset+32].try_into()
            .map_err(|_| anyhow!("Invalid token A mint"))?
    );
    let token_b = Pubkey::new_from_array(
        data[pc_mint_offset..pc_mint_offset+32].try_into()
            .map_err(|_| anyhow!("Invalid token B mint"))?
    );
    
    // Validar que no son direcciones por defecto
    if token_a == Pubkey::default() || token_b == Pubkey::default() {
        return Err(anyhow!("Invalid default token addresses"));
    }
    
    Ok(RealPool {
        address,
        program: RAYDIUM_AMM_PROGRAM.to_string(),
        token_a,
        token_b,
        reserves_a: 1_000_000, // Placeholder - necesitaríamos leer los vaults
        reserves_b: 1_000_000, // Placeholder
    })
}

async fn parse_orca_whirlpool(address: Pubkey, account: &solana_sdk::account::Account) -> Result<RealPool> {
    let data = &account.data;
    
    if data.len() < 653 {
        return Err(anyhow!("Invalid Whirlpool data length"));
    }
    
    // Offsets según documentación de Whirlpool
    let token_mint_a_offset = 103;
    let token_mint_b_offset = 183;
    
    let token_a = Pubkey::new_from_array(
        data[token_mint_a_offset..token_mint_a_offset+32].try_into()
            .map_err(|_| anyhow!("Invalid token A mint"))?
    );
    let token_b = Pubkey::new_from_array(
        data[token_mint_b_offset..token_mint_b_offset+32].try_into()
            .map_err(|_| anyhow!("Invalid token B mint"))?
    );
    
    // Validar que no son direcciones por defecto
    if token_a == Pubkey::default() || token_b == Pubkey::default() {
        return Err(anyhow!("Invalid default token addresses"));
    }
    
    Ok(RealPool {
        address,
        program: ORCA_WHIRLPOOL_PROGRAM.to_string(),
        token_a,
        token_b,
        reserves_a: 1_000_000, // Placeholder
        reserves_b: 1_000_000, // Placeholder
    })
}

fn save_real_pool_addresses(raydium_pools: &[RealPool], orca_pools: &[RealPool]) -> Result<()> {
    let mut output = String::new();
    
    output.push_str("# REAL POOL ADDRESSES FOUND\n");
    output.push_str("# Generated by pool discovery test\n\n");
    
    if !raydium_pools.is_empty() {
        output.push_str("## RAYDIUM POOLS\n");
        for pool in raydium_pools {
            output.push_str(&format!("\"{}\" // {} <-> {}\n", 
                pool.address, 
                &pool.token_a.to_string()[..8],
                &pool.token_b.to_string()[..8]
            ));
        }
        output.push_str("\n");
    }
    
    if !orca_pools.is_empty() {
        output.push_str("## ORCA WHIRLPOOL POOLS\n");
        for pool in orca_pools {
            output.push_str(&format!("\"{}\" // {} <-> {}\n", 
                pool.address, 
                &pool.token_a.to_string()[..8],
                &pool.token_b.to_string()[..8]
            ));
        }
    }
    
    std::fs::write("real_pool_addresses.txt", output)?;
    println!("\n💾 Real pool addresses saved to: real_pool_addresses.txt");
    
    Ok(())
}
