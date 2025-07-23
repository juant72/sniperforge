use reqwest;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Raydium API: https://api.raydium.io/pairs");

    let client = reqwest::Client::new();
    let response = client.get("https://api.raydium.io/pairs").send().await?;

    if response.status().is_success() {
        let data: Value = response.json().await?;

        println!("✅ API Response successful!");
        println!("📊 Response structure:");

        if let Some(pairs) = data.as_array() {
            println!("🎯 Found {} pairs", pairs.len());

            // Analizar los primeros 5 pares para ver la estructura
            for (i, pair) in pairs.iter().take(5).enumerate() {
                println!(
                    "\n📋 Pair {}: {}",
                    i + 1,
                    serde_json::to_string_pretty(pair)?
                );
            }
        } else if let Some(pairs) = data.as_object() {
            println!(
                "🎯 Object with keys: {:?}",
                pairs.keys().collect::<Vec<_>>()
            );

            // Mostrar algunos valores de ejemplo
            for (key, value) in pairs.iter().take(5) {
                println!(
                    "\n📋 Key: {}\nValue: {}",
                    key,
                    serde_json::to_string_pretty(value)?
                );
            }
        }
    } else {
        println!("❌ API Error: {}", response.status());
    }

    Ok(())
}
