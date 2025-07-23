use serde_json::json;
use std::fs;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Obtener private key desde variable de entorno
    let private_key = env::var("DEVNET_PRIVATE_KEY")
        .expect("DEVNET_PRIVATE_KEY no encontrada");
    
    // Crear el formato esperado por el CLI
    let wallet_data = json!({
        "private_key": private_key
    });
    
    // Guardar en el archivo
    fs::write("test-cli-arbitrage.json", serde_json::to_string_pretty(&wallet_data)?)?;
    
    println!("✅ Wallet convertida exitosamente a test-cli-arbitrage.json");
    println!("Formato: {}", serde_json::to_string_pretty(&wallet_data)?);
    
    Ok(())
}
