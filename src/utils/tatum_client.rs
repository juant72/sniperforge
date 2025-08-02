use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_json::{Value, json};
use std::time::Duration;
use tracing::{debug, warn, error};

/// Custom Tatum RPC client that handles header authentication
#[derive(Debug, Clone)]
pub struct TatumClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl TatumClient {
    /// Create a new Tatum client with the given base URL and API key
    pub fn new(base_url: String, api_key: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;
        
        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }
    
    /// Make a JSON-RPC call to the Tatum endpoint
    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        debug!("🔗 Making Tatum RPC call: {} with params: {:?}", method, params);
        
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });
        
        let response = self.client
            .post(&self.base_url)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow!("HTTP request failed: {}", e))?;
        
        let status = response.status();
        let text = response.text().await
            .map_err(|e| anyhow!("Failed to read response: {}", e))?;
        
        if !status.is_success() {
            warn!("❌ Tatum RPC call failed with status {}: {}", status, text);
            return Err(anyhow!("RPC call failed with status {}: {}", status, text));
        }
        
        let json_response: Value = serde_json::from_str(&text)
            .map_err(|e| anyhow!("Failed to parse JSON response: {} - Response: {}", e, text))?;
        
        if let Some(error) = json_response.get("error") {
            error!("❌ Tatum RPC returned error: {}", error);
            return Err(anyhow!("RPC error: {}", error));
        }
        
        if let Some(result) = json_response.get("result") {
            debug!("✅ Tatum RPC call successful");
            Ok(result.clone())
        } else {
            error!("❌ Tatum RPC response missing 'result' field: {}", json_response);
            Err(anyhow!("RPC response missing 'result' field"))
        }
    }
    
    /// Get balance for an account
    pub async fn get_balance(&self, account: &str) -> Result<u64> {
        let params = json!([account]);
        let result = self.call("getBalance", params).await?;
        
        let balance = result.get("value")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow!("Invalid balance response format"))?;
        
        debug!("💰 Account {} balance: {} lamports", account, balance);
        Ok(balance)
    }
    
    /// Get recent block hash
    pub async fn get_recent_blockhash(&self) -> Result<String> {
        let result = self.call("getRecentBlockhash", json!([])).await?;
        
        let blockhash = result.get("value")
            .and_then(|v| v.get("blockhash"))
            .and_then(|h| h.as_str())
            .ok_or_else(|| anyhow!("Invalid blockhash response format"))?;
        
        debug!("🔗 Recent blockhash: {}", blockhash);
        Ok(blockhash.to_string())
    }
    
    /// Send transaction
    pub async fn send_transaction(&self, transaction: &str) -> Result<String> {
        let params = json!([transaction]);
        let result = self.call("sendTransaction", params).await?;
        
        let signature = result.as_str()
            .ok_or_else(|| anyhow!("Invalid transaction response format"))?;
        
        debug!("📤 Transaction sent with signature: {}", signature);
        Ok(signature.to_string())
    }
    
    /// Simulate transaction
    pub async fn simulate_transaction(&self, transaction: &str) -> Result<Value> {
        let params = json!([transaction]);
        self.call("simulateTransaction", params).await
    }
    
    /// Get account info
    pub async fn get_account_info(&self, account: &str) -> Result<Value> {
        let params = json!([account]);
        self.call("getAccountInfo", params).await
    }
}

/// Wrapper for easy usage with configuration
#[derive(Debug)]
pub struct TatumRpcClient {
    client: TatumClient,
}

impl TatumRpcClient {
    pub fn new(base_url: String, api_key: String) -> Result<Self> {
        let client = TatumClient::new(base_url, api_key)?;
        Ok(Self { client })
    }
    
    pub fn client(&self) -> &TatumClient {
        &self.client
    }
    
    /// Quick health check
    pub async fn health_check(&self) -> Result<bool> {
        match self.client.get_recent_blockhash().await {
            Ok(_) => {
                debug!("✅ Tatum RPC health check passed");
                Ok(true)
            }
            Err(e) => {
                warn!("❌ Tatum RPC health check failed: {}", e);
                Ok(false)
            }
        }
    }
}
