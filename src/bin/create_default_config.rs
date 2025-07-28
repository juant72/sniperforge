// ================================================================================
// GENERADOR DE CONFIGURACIÓN POR DEFECTO
// ================================================================================
// Crea un archivo arbitrage_settings.json con configuración por defecto
// ================================================================================

use anyhow::Result;
use sniperforge::arbitrage_settings::ArbitrageSettings;

fn main() -> Result<()> {
    println!("🔧 Creando configuración por defecto...");
    
    // Crear configuración por defecto
    let settings = ArbitrageSettings::default();
    
    // Guardar en archivo
    settings.save_to_file("arbitrage_settings.json")?;
    
    println!("✅ Archivo creado: arbitrage_settings.json");
    println!();
    println!("📋 PRÓXIMOS PASOS:");
    println!("1. Edita arbitrage_settings.json con tus configuraciones");
    println!("2. Configura tu wallet en 'wallet.keypair_file'");
    println!("3. Ajusta 'trading.mode' según necesites");
    println!("4. Ejecuta: cargo run --bin arbitrage_with_json_config");
    println!();
    println!("🔥 Para trading real: cambia 'trading.mode' a 'real'");
    
    Ok(())
}
