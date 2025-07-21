use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::str::FromStr;
use tokio;

// Raydium AMM Program ID
const RAYDIUM_AMM_PROGRAM: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Analyzing Raydium pool structure...");

    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    // SOL-USDC pool que sabemos que existe
    let pool_address = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2";
    let pool_pubkey = Pubkey::from_str(pool_address)?;

    println!("📡 Analyzing pool: {}", pool_address);

    match client.get_account(&pool_pubkey) {
        Ok(account) => {
            println!("✅ Pool account found");
            println!("   Owner: {}", account.owner);
            println!("   Data size: {} bytes", account.data.len());
            println!("   Executable: {}", account.executable);
            println!("   Rent epoch: {}", account.rent_epoch);

            if account.owner.to_string() == RAYDIUM_AMM_PROGRAM {
                println!("🔥 This is a Raydium AMM pool");

                // Parse Raydium pool structure
                let data = &account.data;
                if data.len() >= 752 {
                    println!("\n📊 Raydium Pool Data Analysis:");

                    // Raydium AMM pool layout (approximate)
                    // Status: 8 bytes
                    // nonce: 8 bytes
                    // max_order: 8 bytes
                    // depth: 8 bytes
                    // base_decimals: 8 bytes
                    // quote_decimals: 8 bytes
                    // state: 8 bytes
                    // reset_flag: 8 bytes
                    // min_size: 8 bytes
                    // vol_max_cut_ratio: 8 bytes
                    // amount_wave_ratio: 8 bytes
                    // base_lot_size: 8 bytes
                    // quote_lot_size: 8 bytes
                    // min_price_multiplier: 8 bytes
                    // max_price_multiplier: 8 bytes
                    // system_decimals_value: 8 bytes
                    // min_separate_numerator: 8 bytes
                    // min_separate_denominator: 8 bytes
                    // trade_fee_numerator: 8 bytes
                    // trade_fee_denominator: 8 bytes
                    // pnl_numerator: 8 bytes
                    // pnl_denominator: 8 bytes
                    // swap_fee_numerator: 8 bytes
                    // swap_fee_denominator: 8 bytes
                    // base_need_take_pnl: 8 bytes
                    // quote_need_take_pnl: 8 bytes
                    // quote_total_pnl: 8 bytes
                    // base_total_pnl: 8 bytes
                    // pool_open_time: 8 bytes
                    // punish_pc_amount: 8 bytes
                    // punish_coin_amount: 8 bytes
                    // orderbook_to_init_time: 8 bytes
                    // swap_base_in_amount: 16 bytes
                    // swap_quote_out_amount: 16 bytes
                    // swap_base2_quote_fee: 8 bytes
                    // swap_quote_in_amount: 16 bytes
                    // swap_base_out_amount: 16 bytes
                    // swap_quote2_base_fee: 8 bytes
                    // base_vault: 32 bytes (pubkey)
                    // quote_vault: 32 bytes (pubkey)
                    // base_mint: 32 bytes (pubkey)
                    // quote_mint: 32 bytes (pubkey)
                    // lp_mint: 32 bytes (pubkey)
                    // open_orders: 32 bytes (pubkey)
                    // market_id: 32 bytes (pubkey)
                    // market_program_id: 32 bytes (pubkey)
                    // target_orders: 32 bytes (pubkey)
                    // withdraw_queue: 32 bytes (pubkey)
                    // lp_vault: 32 bytes (pubkey)
                    // owner: 32 bytes (pubkey)

                    // Extract key addresses (approximate offsets)
                    let base_vault_offset = 344; // Approximate
                    let quote_vault_offset = 376; // Approximate
                    let base_mint_offset = 408; // Approximate
                    let quote_mint_offset = 440; // Approximate

                    if data.len() >= base_vault_offset + 32 {
                        let base_vault_bytes = &data[base_vault_offset..base_vault_offset + 32];
                        if let Ok(base_vault) = Pubkey::try_from(base_vault_bytes) {
                            println!("   Base Vault: {}", base_vault);
                        }
                    }

                    if data.len() >= quote_vault_offset + 32 {
                        let quote_vault_bytes = &data[quote_vault_offset..quote_vault_offset + 32];
                        if let Ok(quote_vault) = Pubkey::try_from(quote_vault_bytes) {
                            println!("   Quote Vault: {}", quote_vault);
                        }
                    }

                    if data.len() >= base_mint_offset + 32 {
                        let base_mint_bytes = &data[base_mint_offset..base_mint_offset + 32];
                        if let Ok(base_mint) = Pubkey::try_from(base_mint_bytes) {
                            println!("   Base Mint: {}", base_mint);
                        }
                    }

                    if data.len() >= quote_mint_offset + 32 {
                        let quote_mint_bytes = &data[quote_mint_offset..quote_mint_offset + 32];
                        if let Ok(quote_mint) = Pubkey::try_from(quote_mint_bytes) {
                            println!("   Quote Mint: {}", quote_mint);
                        }
                    }

                    // Print raw bytes for analysis
                    println!("\n🔍 Raw data (first 200 bytes):");
                    for (i, chunk) in data.chunks(32).take(6).enumerate() {
                        print!("   {:03}: ", i * 32);
                        for byte in chunk {
                            print!("{:02x} ", byte);
                        }
                        println!();
                    }
                } else {
                    println!("❌ Data too small for Raydium pool");
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to get account: {}", e);
        }
    }

    Ok(())
}
