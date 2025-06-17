use anyhow::Result;

#[cfg(test)]
mod platform_tests {
    use super::*;
    use sniperforge::config::Config;
    use sniperforge::platform::SniperForgePlatform;

    #[tokio::test]
    async fn test_platform_initialization() -> Result<()> {
        // Test that platform can be initialized with default config
        let config = Config::default();
        let platform = SniperForgePlatform::new(config).await?;
        
        // Platform should be created successfully
        assert!(!platform.is_running().await);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_platform_lifecycle() -> Result<()> {
        let config = Config::default();
        let platform = SniperForgePlatform::new(config).await?;
        
        // Start platform
        platform.start_platform().await?;
        assert!(platform.is_running().await);
        
        // Stop platform
        platform.shutdown().await?;
        assert!(!platform.is_running().await);
        
        Ok(())
    }
}

#[cfg(test)]
mod bot_tests {
    use super::*;
    use sniperforge::bots::lp_sniper::LpSniperBot;
    use sniperforge::types::LpSniperConfig;
    use sniperforge::shared::SharedServices;
    use sniperforge::platform::event_bus::EventBus;
    use sniperforge::config::Config;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_lp_sniper_creation() -> Result<()> {
        let config = Config::default();
        let shared_services = Arc::new(SharedServices::new(&config).await?);
        let event_bus = Arc::new(EventBus::new(None));
        let bot_config = LpSniperConfig::default();
        
        let (bot, _cmd_tx, _event_rx) = LpSniperBot::new(
            bot_config,
            shared_services,
            event_bus,
        ).await?;
        
        // Bot should start in stopped state
        assert_eq!(bot.get_status().await, sniperforge::types::BotStatus::Stopped);
        
        Ok(())
    }
}

#[cfg(test)]
mod config_tests {
    use sniperforge::config::Config;

    #[test]
    fn test_config_validation() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_config_loading() {
        // Test loading from TOML file
        let _config_content = r#"
[platform]
name = "SniperForge Test"
environment = "testnet"
log_level = "debug"

[network]
cluster = "testnet"
rpc_endpoints = ["https://api.testnet.solana.com"]
"#;
        
        // In a real test, this would load from a file
        // For now, just verify the default config works
        let config = Config::default();
        assert_eq!(config.platform.name, "SniperForge");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_full_system_startup() -> Result<()> {
        // Test complete system startup and shutdown
        let config = sniperforge::config::Config::default();
        let platform = sniperforge::platform::SniperForgePlatform::new(config).await?;
        
        // Start platform with timeout
        let start_result = timeout(Duration::from_secs(10), platform.start_platform()).await;
        assert!(start_result.is_ok());
        
        // Verify running state
        assert!(platform.is_running().await);
        
        // Get metrics
        let metrics = platform.get_metrics().await?;
        assert_eq!(metrics.total_bots, 0); // No bots started yet
        
        // Shutdown with timeout
        let shutdown_result = timeout(Duration::from_secs(5), platform.shutdown()).await;
        assert!(shutdown_result.is_ok());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_bot_lifecycle() -> Result<()> {
        let config = sniperforge::config::Config::default();
        let platform = sniperforge::platform::SniperForgePlatform::new(config).await?;
        
        platform.start_platform().await?;
        
        // Start LP Sniper bot
        let bot_types = vec!["lp-sniper".to_string()];
        platform.start_specific_bots(bot_types).await?;
        
        // Give it a moment to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Check metrics
        let metrics = platform.get_metrics().await?;
        assert!(metrics.total_bots > 0);
        
        platform.shutdown().await?;
        
        Ok(())
    }
}
