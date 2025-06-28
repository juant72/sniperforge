use reqwest::Client;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    println!("=== Testing DexScreener Search Endpoint ===");
    let search_url = "https://api.dexscreener.com/latest/dex/search?q=SOL/USDC";
    let response = client.get(search_url).send().await?;
    let text = response.text().await?;
    println!("Search response length: {}", text.len());
    println!("First 1000 chars: {}", &text[..std::cmp::min(1000, text.len())]);
    
    // Try to parse as JSON to see the structure
    if let Ok(json_value) = serde_json::from_str::<Value>(&text) {
        println!("JSON structure keys: {:?}", json_value.as_object().map(|o| o.keys().collect::<Vec<_>>()));
    } else {
        println!("Failed to parse as JSON");
    }
    
    println!("\n=== Testing DexScreener Batch Endpoint ===");
    let batch_url = "https://api.dexscreener.com/tokens/v1/solana/So11111111111111111111111111111111111111112,EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    let response = client.get(batch_url).send().await?;
    let text = response.text().await?;
    println!("Batch response length: {}", text.len());
    println!("First 1000 chars: {}", &text[..std::cmp::min(1000, text.len())]);
    
    // Try to parse as JSON to see the structure
    if let Ok(json_value) = serde_json::from_str::<Value>(&text) {
        if let Some(array) = json_value.as_array() {
            println!("Array with {} items", array.len());
            if let Some(first_item) = array.get(0) {
                println!("First item keys: {:?}", first_item.as_object().map(|o| o.keys().collect::<Vec<_>>()));
            }
        } else {
            println!("Not an array: {:?}", json_value);
        }
    } else {
        println!("Failed to parse as JSON");
    }
    
    Ok(())
}
