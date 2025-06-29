use anyhow::Result;
use std::collections::HashMap;
use std::env;
use tracing::{info, warn, error};

use crate::config::{NetworkConfig, PremiumRpcConfig, EndpointPriority};

/// Manager for premium RPC endpoints with API key authentication
#[derive(Debug, Clone)]
pub struct PremiumRpcManager {
    endpoints: Vec<PremiumEndpoint>,
    current_index: usize,
}

#[derive(Debug, Clone)]
pub struct PremiumEndpoint {
    pub provider: String,
    pub url: String,
    pub websocket_url: Option<String>,
    pub priority: u8,
    pub max_requests_per_second: u32,
    pub requests_this_second: u32,
    pub last_reset: std::time::Instant,
}

impl PremiumRpcManager {
    /// Create a new premium RPC manager from network configuration
    pub fn new(network_config: &NetworkConfig) -> Result<Self> {
        let mut endpoints = Vec::new();
        
        if let Some(premium_config) = &network_config.premium_rpc {
            if !premium_config.enabled {
                info!("🔧 Premium RPC disabled in configuration");
                return Ok(Self {
                    endpoints,
                    current_index: 0,
                });
            }
            
            info!("🔧 Loading premium RPC endpoints...");
            
            // Load API keys from environment variables
            let api_keys = Self::load_api_keys();
            
            // Build endpoints based on available API keys
            endpoints.extend(Self::build_helius_endpoint(premium_config, &api_keys)?);
            endpoints.extend(Self::build_ankr_endpoint(premium_config, &api_keys)?);
            endpoints.extend(Self::build_quicknode_endpoint(premium_config, &api_keys)?);
            endpoints.extend(Self::build_alchemy_endpoint(premium_config, &api_keys)?);
            endpoints.extend(Self::build_tatum_endpoint(premium_config, &api_keys)?);  // NEW: Tatum support
            
            // Sort by priority (lower number = higher priority)
            endpoints.sort_by_key(|e| e.priority);
            
            info!("✅ Loaded {} premium RPC endpoints", endpoints.len());
            for endpoint in &endpoints {
                info!("   📡 {} (priority: {})", endpoint.provider, endpoint.priority);
            }
        }
        
        Ok(Self {
            endpoints,
            current_index: 0,
        })
    }
    
    /// Load API keys from environment variables
    fn load_api_keys() -> HashMap<String, String> {
        let mut api_keys = HashMap::new();
        
        // Check for each supported API key
        if let Ok(key) = env::var("HELIUS_API_KEY") {
            if !key.is_empty() {
                api_keys.insert("helius".to_string(), key);
                info!("✅ Found Helius API key");
            }
        }
        
        if let Ok(key) = env::var("ANKR_API_KEY") {
            if !key.is_empty() {
                api_keys.insert("ankr".to_string(), key);
                info!("✅ Found Ankr API key");
            }
        }
        
        if let Ok(endpoint) = env::var("QUICKNODE_ENDPOINT") {
            if !endpoint.is_empty() {
                api_keys.insert("quicknode".to_string(), endpoint);
                info!("✅ Found QuickNode endpoint");
            }
        }
        
        if let Ok(key) = env::var("ALCHEMY_API_KEY") {
            if !key.is_empty() {
                api_keys.insert("alchemy".to_string(), key);
                info!("✅ Found Alchemy API key");
            }
        }
        
        // Check for Tatum API keys (different for mainnet and devnet)
        if let Ok(key) = env::var("TATUM_API_KEY_MAINNET") {
            if !key.is_empty() {
                api_keys.insert("tatum_mainnet".to_string(), key);
                info!("✅ Found Tatum Mainnet API key");
            }
        }
        
        if let Ok(key) = env::var("TATUM_API_KEY_DEVNET") {
            if !key.is_empty() {
                api_keys.insert("tatum_devnet".to_string(), key);
                info!("✅ Found Tatum Devnet API key");
            }
        }
        
        if api_keys.is_empty() {
            warn!("⚠️ No premium API keys found in environment variables");
            warn!("   Set HELIUS_API_KEY, ANKR_API_KEY, QUICKNODE_ENDPOINT, ALCHEMY_API_KEY,");
            warn!("   TATUM_API_KEY_MAINNET, or TATUM_API_KEY_DEVNET");
        }
        
        api_keys
    }
    
    /// Build Helius endpoint if API key is available
    fn build_helius_endpoint(
        config: &PremiumRpcConfig, 
        api_keys: &HashMap<String, String>
    ) -> Result<Vec<PremiumEndpoint>> {
        if let (Some(template), Some(api_key)) = (
            &config.helius_rpc_template,
            api_keys.get("helius")
        ) {
            let url = template.replace("{API_KEY}", api_key);
            let websocket_url = config.helius_ws_template
                .as_ref()
                .map(|ws_template| ws_template.replace("{API_KEY}", api_key));
            
            let priority = Self::get_priority(config, "helius");
            let max_rps = Self::get_max_requests_per_second(config, "helius");
            
            Ok(vec![PremiumEndpoint {
                provider: "Helius".to_string(),
                url,
                websocket_url,
                priority,
                max_requests_per_second: max_rps,
                requests_this_second: 0,
                last_reset: std::time::Instant::now(),
            }])
        } else {
            Ok(vec![])
        }
    }
    
    /// Build Ankr endpoint if API key is available
    fn build_ankr_endpoint(
        config: &PremiumRpcConfig, 
        api_keys: &HashMap<String, String>
    ) -> Result<Vec<PremiumEndpoint>> {
        if let (Some(template), Some(api_key)) = (
            &config.ankr_rpc_template,
            api_keys.get("ankr")
        ) {
            let url = template.replace("{API_KEY}", api_key);
            
            let priority = Self::get_priority(config, "ankr");
            let max_rps = Self::get_max_requests_per_second(config, "ankr");
            
            Ok(vec![PremiumEndpoint {
                provider: "Ankr".to_string(),
                url,
                websocket_url: None, // Ankr doesn't provide WebSocket in basic plan
                priority,
                max_requests_per_second: max_rps,
                requests_this_second: 0,
                last_reset: std::time::Instant::now(),
            }])
        } else {
            Ok(vec![])
        }
    }
    
    /// Build QuickNode endpoint if configured
    fn build_quicknode_endpoint(
        config: &PremiumRpcConfig, 
        api_keys: &HashMap<String, String>
    ) -> Result<Vec<PremiumEndpoint>> {
        if let (Some(template), Some(endpoint)) = (
            &config.quicknode_rpc_template,
            api_keys.get("quicknode")
        ) {
            let url = template.replace("{ENDPOINT}", endpoint);
            
            let priority = Self::get_priority(config, "quicknode");
            let max_rps = Self::get_max_requests_per_second(config, "quicknode");
            
            // QuickNode WebSocket is usually the same endpoint with wss://
            let websocket_url = if endpoint.starts_with("https://") {
                Some(endpoint.replace("https://", "wss://"))
            } else {
                None
            };
            
            Ok(vec![PremiumEndpoint {
                provider: "QuickNode".to_string(),
                url,
                websocket_url,
                priority,
                max_requests_per_second: max_rps,
                requests_this_second: 0,
                last_reset: std::time::Instant::now(),
            }])
        } else {
            Ok(vec![])
        }
    }
    
    /// Build Alchemy endpoint if API key is available
    fn build_alchemy_endpoint(
        config: &PremiumRpcConfig, 
        api_keys: &HashMap<String, String>
    ) -> Result<Vec<PremiumEndpoint>> {
        if let (Some(template), Some(api_key)) = (
            &config.alchemy_rpc_template,
            api_keys.get("alchemy")
        ) {
            let url = template.replace("{API_KEY}", api_key);
            let websocket_url = config.alchemy_ws_template
                .as_ref()
                .map(|ws_template| ws_template.replace("{API_KEY}", api_key));
            
            let priority = Self::get_priority(config, "alchemy");
            let max_rps = Self::get_max_requests_per_second(config, "alchemy");
            
            Ok(vec![PremiumEndpoint {
                provider: "Alchemy".to_string(),
                url,
                websocket_url,
                priority,
                max_requests_per_second: max_rps,
                requests_this_second: 0,
                last_reset: std::time::Instant::now(),
            }])
        } else {
            Ok(vec![])
        }
    }
    
    /// Build Tatum endpoint if API key is available
    /// Note: Tatum uses header authentication (x-api-key), not URL parameters
    fn build_tatum_endpoint(
        config: &PremiumRpcConfig, 
        api_keys: &HashMap<String, String>
    ) -> Result<Vec<PremiumEndpoint>> {
        if let Some(template) = &config.tatum_rpc_template {
            // Determine which API key to use based on endpoint URL
            let api_key = if template.contains("mainnet") {
                api_keys.get("tatum_mainnet")
            } else if template.contains("devnet") {
                api_keys.get("tatum_devnet")
            } else {
                // Try mainnet first, then devnet
                api_keys.get("tatum_mainnet").or_else(|| api_keys.get("tatum_devnet"))
            };
            
            if let Some(_api_key) = api_key {
                let priority = Self::get_priority(config, "tatum");
                let max_rps = Self::get_max_requests_per_second(config, "tatum");
                
                Ok(vec![PremiumEndpoint {
                    provider: "Tatum".to_string(),
                    url: template.clone(),
                    websocket_url: None, // Tatum doesn't provide WebSocket in basic plan
                    priority,
                    max_requests_per_second: max_rps,
                    requests_this_second: 0,
                    last_reset: std::time::Instant::now(),
                }])
            } else {
                Ok(vec![])
            }
        } else {
            Ok(vec![])
        }
    }
    
    /// Get priority for a provider
    fn get_priority(config: &PremiumRpcConfig, provider: &str) -> u8 {
        if let Some(priorities) = &config.endpoint_priorities {
            for priority_config in priorities {
                if priority_config.provider == provider {
                    return priority_config.priority;
                }
            }
        }
        255 // Default lowest priority
    }
    
    /// Get max requests per second for a provider
    fn get_max_requests_per_second(config: &PremiumRpcConfig, provider: &str) -> u32 {
        if let Some(priorities) = &config.endpoint_priorities {
            for priority_config in priorities {
                if priority_config.provider == provider {
                    return priority_config.max_requests_per_second;
                }
            }
        }
        10 // Default conservative limit
    }
    
    /// Get the next available endpoint (with rate limiting)
    pub fn get_next_endpoint(&mut self) -> Option<&PremiumEndpoint> {
        if self.endpoints.is_empty() {
            return None;
        }
        
        let now = std::time::Instant::now();
        
        // Try to find an endpoint that's not rate limited
        for _ in 0..self.endpoints.len() {
            let endpoint = &mut self.endpoints[self.current_index];
            
            // Reset counter if a second has passed
            if now.duration_since(endpoint.last_reset).as_secs() >= 1 {
                endpoint.requests_this_second = 0;
                endpoint.last_reset = now;
            }
            
            // Check if we can use this endpoint
            if endpoint.requests_this_second < endpoint.max_requests_per_second {
                endpoint.requests_this_second += 1;
                let current_endpoint = &self.endpoints[self.current_index];
                self.current_index = (self.current_index + 1) % self.endpoints.len();
                return Some(current_endpoint);
            }
            
            // Try next endpoint
            self.current_index = (self.current_index + 1) % self.endpoints.len();
        }
        
        // All endpoints are rate limited
        warn!("⚠️ All premium endpoints are rate limited");
        None
    }
    
    /// Get all available premium URLs
    pub fn get_all_urls(&self) -> Vec<String> {
        self.endpoints.iter().map(|e| e.url.clone()).collect()
    }
    
    /// Get the best WebSocket URL available
    pub fn get_websocket_url(&self) -> Option<String> {
        // Return the WebSocket URL from the highest priority endpoint that has one
        self.endpoints
            .iter()
            .find(|e| e.websocket_url.is_some())
            .and_then(|e| e.websocket_url.clone())
    }
    
    /// Check if any premium endpoints are available
    pub fn has_premium_endpoints(&self) -> bool {
        !self.endpoints.is_empty()
    }
    
    /// Get status summary
    pub fn get_status_summary(&self) -> String {
        if self.endpoints.is_empty() {
            return "No premium endpoints configured".to_string();
        }
        
        let providers: Vec<String> = self.endpoints
            .iter()
            .map(|e| format!("{} (p:{})", e.provider, e.priority))
            .collect();
        
        format!("Premium endpoints: {}", providers.join(", "))
    }
    
    /// Get Tatum client configurations for custom HTTP client creation
    pub fn get_tatum_configs(&self) -> Vec<(String, String)> {
        self.endpoints
            .iter()
            .filter(|e| e.provider == "Tatum")
            .map(|e| (e.url.clone(), "tatum".to_string()))
            .collect()
    }
    
    /// Get API key for Tatum based on endpoint URL
    pub fn get_tatum_api_key(endpoint_url: &str) -> Option<String> {
        use std::env;
        
        if endpoint_url.contains("mainnet") {
            env::var("TATUM_API_KEY_MAINNET").ok()
        } else if endpoint_url.contains("devnet") {
            env::var("TATUM_API_KEY_DEVNET").ok()
        } else {
            // Try mainnet first, then devnet
            env::var("TATUM_API_KEY_MAINNET")
                .or_else(|_| env::var("TATUM_API_KEY_DEVNET"))
                .ok()
        }
    }
}
