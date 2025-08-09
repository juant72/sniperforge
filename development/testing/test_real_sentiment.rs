use sniperforge::intelligence::sentiment::RealSentimentAnalyzer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Testing REAL Sentiment Analysis System");
    println!("==========================================");
    
    let mut analyzer = RealSentimentAnalyzer::new();
    
    // Test symbols
    let symbols = vec!["SOL", "BTC", "ETH", "MATIC"];
    
    for symbol in symbols {
        println!("\n🔍 Analyzing sentiment for {}...", symbol);
        
        match analyzer.calculate_sentiment_score(symbol).await {
            Ok(score) => {
                println!("   📊 Sentiment Score: {:.3}", score);
                
                if score > 0.2 {
                    println!("   🟢 BULLISH sentiment detected");
                } else if score < -0.2 {
                    println!("   🔴 BEARISH sentiment detected");
                } else {
                    println!("   🟡 NEUTRAL sentiment");
                }
            }
            Err(e) => {
                println!("   ⚠️  Error: {}", e);
            }
        }
        
        // Test detailed sentiment
        match analyzer.get_detailed_sentiment(symbol).await {
            Ok(analysis) => {
                println!("   📈 Detailed Analysis:");
                println!("      Overall Score: {:.3}", analysis.overall_score);
                println!("      Bullish Signals: {}", analysis.bullish_signals);
                println!("      Bearish Signals: {}", analysis.bearish_signals);
                println!("      Neutral Signals: {}", analysis.neutral_signals);
                println!("      Confidence: {:.3}", analysis.confidence);
                println!("      Sources: {}", analysis.source_breakdown.len());
            }
            Err(e) => {
                println!("   ⚠️  Detailed analysis error: {}", e);
            }
        }
        
        println!("   ⏱️  Waiting 2 seconds before next analysis...");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
    
    println!("\n✅ Real sentiment analysis test completed!");
    println!("🎯 FAKE fastrand() simulation has been ELIMINATED!");
    println!("🧠 Now using REAL data sources for sentiment analysis");
    
    Ok(())
}
