use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// Custom RPC client for Tatum that uses header authentication
#[derive(Debug, Clone)]
pub struct TatumRpcClient {
    client: Client,
    base_url: String,
    api_key: String,
    request_id_counter: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl TatumRpcClient {
    /// Create a new Tatum RPC client
    pub fn new(base_url: String, api_key: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            base_url,
            api_key,
            request_id_counter: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
        })
    }

    /// Make a JSON-RPC call to Tatum
    async fn json_rpc_call(&self, method: &str, params: Value) -> Result<Value> {
        let request_id = self
            .request_id_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let payload = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": request_id
        });

        debug!("🌐 Tatum RPC call: {} with params: {}", method, params);

        let response = self
            .client
            .post(&self.base_url)
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header("x-api-key", &self.api_key)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Tatum RPC error {}: {}", status, text));
        }

        let json_response: Value = response.json().await?;

        if let Some(error) = json_response.get("error") {
            return Err(anyhow::anyhow!("Tatum RPC error: {}", error));
        }

        if let Some(result) = json_response.get("result") {
            Ok(result.clone())
        } else {
            Err(anyhow::anyhow!("No result in Tatum RPC response"))
        }
    }

    /// Get version (equivalent to getVersion)
    pub async fn get_version(&self) -> Result<Value> {
        self.json_rpc_call("getVersion", json!([])).await
    }

    /// Get current slot (equivalent to getSlot)
    pub async fn get_slot(&self) -> Result<u64> {
        let result = self.json_rpc_call("getSlot", json!([])).await?;
        result
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("Invalid slot response from Tatum"))
    }

    /// Get account info (equivalent to getAccountInfo)
    pub async fn get_account_info(&self, pubkey: &str) -> Result<Option<Value>> {
        let params = json!([pubkey, {"encoding": "base64"}]);
        match self.json_rpc_call("getAccountInfo", params).await {
            Ok(result) => {
                if result.is_null() {
                    Ok(None)
                } else {
                    Ok(Some(result))
                }
            }
            Err(e) => {
                // Account not found is not an error
                if e.to_string().contains("not found") {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Get balance (equivalent to getBalance)
    pub async fn get_balance(&self, pubkey: &str) -> Result<u64> {
        let params = json!([pubkey]);
        let result = self.json_rpc_call("getBalance", params).await?;

        if let Some(value) = result.get("value") {
            value
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("Invalid balance response from Tatum"))
        } else {
            Err(anyhow::anyhow!("No value in balance response from Tatum"))
        }
    }

    /// Get latest blockhash (equivalent to getLatestBlockhash)
    pub async fn get_latest_blockhash(&self) -> Result<String> {
        let params = json!([{"commitment": "confirmed"}]);
        let result = self.json_rpc_call("getLatestBlockhash", params).await?;

        if let Some(value) = result.get("value") {
            if let Some(blockhash) = value.get("blockhash") {
                blockhash
                    .as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| anyhow::anyhow!("Invalid blockhash format from Tatum"))
            } else {
                Err(anyhow::anyhow!("No blockhash in response from Tatum"))
            }
        } else {
            Err(anyhow::anyhow!("No value in blockhash response from Tatum"))
        }
    }

    /// Send transaction (equivalent to sendTransaction)
    pub async fn send_transaction(&self, transaction: &str) -> Result<String> {
        let params = json!([transaction, {"encoding": "base64"}]);
        let result = self.json_rpc_call("sendTransaction", params).await?;

        result
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Invalid transaction signature response from Tatum"))
    }

    /// Get program accounts (equivalent to getProgramAccounts)
    /// Note: This may be rate-limited or disabled on public Tatum endpoints
    pub async fn get_program_accounts(&self, program_id: &str) -> Result<Vec<Value>> {
        let params = json!([program_id, {"encoding": "base64"}]);
        let result = self.json_rpc_call("getProgramAccounts", params).await?;

        result
            .as_array()
            .map(|arr| arr.clone())
            .ok_or_else(|| anyhow::anyhow!("Invalid program accounts response from Tatum"))
    }

    /// Test connection to Tatum
    pub async fn test_connection(&self) -> Result<()> {
        match self.get_version().await {
            Ok(version) => {
                info!("✅ Tatum connection successful: {:?}", version);
                Ok(())
            }
            Err(e) => {
                error!("❌ Tatum connection failed: {}", e);
                Err(e)
            }
        }
    }

    /// Get the base URL for this client
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Check if this is a mainnet or devnet endpoint
    pub fn is_mainnet(&self) -> bool {
        self.base_url.contains("mainnet")
    }

    /// Check if this is a devnet endpoint
    pub fn is_devnet(&self) -> bool {
        self.base_url.contains("devnet")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tatum_mainnet_connection() {
        // Only run if API key is available
        if let Ok(api_key) = std::env::var("TATUM_API_KEY_MAINNET") {
            let client = TatumRpcClient::new(
                "https://solana-mainnet.gateway.tatum.io".to_string(),
                api_key,
            )
            .unwrap();

            let result = client.test_connection().await;
            assert!(result.is_ok(), "Tatum mainnet connection should work");
        }
    }

    #[tokio::test]
    async fn test_tatum_devnet_connection() {
        // Only run if API key is available
        if let Ok(api_key) = std::env::var("TATUM_API_KEY_DEVNET") {
            let client = TatumRpcClient::new(
                "https://solana-devnet.gateway.tatum.io".to_string(),
                api_key,
            )
            .unwrap();

            let result = client.test_connection().await;
            assert!(result.is_ok(), "Tatum devnet connection should work");
        }
    }
}
