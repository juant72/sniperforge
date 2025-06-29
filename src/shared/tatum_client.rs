use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_json::{Value, json};
use std::collections::HashMap;
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
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)  // Tatum uses x-api-key header
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow!("Tatum RPC request failed: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Tatum RPC returned error {}: {}", status, error_text));
        }
        
        let json_response: Value = response.json().await
            .map_err(|e| anyhow!("Failed to parse Tatum RPC response: {}", e))?;
        
        if let Some(error) = json_response.get("error") {
            return Err(anyhow!("Tatum RPC error: {}", error));
        }
        
        json_response.get("result")
            .cloned()
            .ok_or_else(|| anyhow!("No result in Tatum RPC response"))
    }
    
    /// Get account info (common Solana RPC method)
    pub async fn get_account_info(&self, pubkey: &str, encoding: Option<&str>) -> Result<Value> {
        let params = if let Some(enc) = encoding {
            json!([pubkey, {"encoding": enc}])
        } else {
            json!([pubkey])
        };
        
        self.call("getAccountInfo", params).await
    }
    
    /// Get program accounts (common Solana RPC method)
    pub async fn get_program_accounts(&self, program_id: &str, config: Option<Value>) -> Result<Value> {
        let params = if let Some(cfg) = config {
            json!([program_id, cfg])
        } else {
            json!([program_id])
        };
        
        self.call("getProgramAccounts", params).await
    }
    
    /// Get slot (to test connectivity)
    pub async fn get_slot(&self) -> Result<u64> {
        let result = self.call("getSlot", json!([])).await?;
        result.as_u64()
            .ok_or_else(|| anyhow!("Invalid slot response from Tatum"))
    }
    
    /// Get health status
    pub async fn get_health(&self) -> Result<String> {
        let result = self.call("getHealth", json!([])).await?;
        Ok(result.as_str().unwrap_or("unknown").to_string())
    }
    
    /// Test the connection to Tatum
    pub async fn test_connection(&self) -> Result<()> {
        debug!("🔍 Testing Tatum connection to {}", self.base_url);
        
        match self.get_slot().await {
            Ok(slot) => {
                debug!("✅ Tatum connection successful, current slot: {}", slot);
                Ok(())
            },
            Err(e) => {
                warn!("❌ Tatum connection failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Get the base URL for this client
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// Wrapper to integrate Tatum client with the existing RPC pool infrastructure
pub struct TatumRpcWrapper {
    client: TatumClient,
}

impl TatumRpcWrapper {
    pub fn new(base_url: String, api_key: String) -> Result<Self> {
        let client = TatumClient::new(base_url, api_key)?;
        Ok(Self { client })
    }
    
    /// Get the underlying Tatum client
    pub fn client(&self) -> &TatumClient {
        &self.client
    }
    
    /// Test the connection
    pub async fn test_connection(&self) -> Result<()> {
        self.client.test_connection().await
    }
    
    /// Make a generic RPC call
    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        self.client.call(method, params).await
    }
}
