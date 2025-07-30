//! Input validation utilities

use crate::types::ApiResult;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Validates a Solana public key string
pub fn validate_pubkey(pubkey_str: &str) -> ApiResult<Pubkey> {
    Pubkey::from_str(pubkey_str)
        .map_err(|e| format!("Invalid pubkey: {}", e))
}

/// Validates an amount is positive and within reasonable bounds
pub fn validate_amount(amount: f64) -> ApiResult<()> {
    if amount <= 0.0 {
        return Err("Amount must be positive".to_string());
    }
    
    if amount > 1_000_000.0 {
        return Err("Amount too large".to_string());
    }
    
    Ok(())
}

/// Validates percentage is between 0 and 100
pub fn validate_percentage(percentage: f64) -> ApiResult<()> {
    if percentage < 0.0 || percentage > 100.0 {
        return Err("Percentage must be between 0 and 100".to_string());
    }
    Ok(())
}

/// Validates slippage tolerance is reasonable
pub fn validate_slippage(slippage: f64) -> ApiResult<()> {
    if slippage < 0.0 {
        return Err("Slippage cannot be negative".to_string());
    }
    
    if slippage > 50.0 {
        return Err("Slippage too high (max 50%)".to_string());
    }
    
    Ok(())
}

/// Validates API URL format
pub fn validate_api_url(url: &str) -> ApiResult<()> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("Invalid URL format".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_amount() {
        assert!(validate_amount(1.0).is_ok());
        assert!(validate_amount(0.1).is_ok());
        assert!(validate_amount(-1.0).is_err());
        assert!(validate_amount(0.0).is_err());
        assert!(validate_amount(2_000_000.0).is_err());
    }

    #[test]
    fn test_validate_percentage() {
        assert!(validate_percentage(50.0).is_ok());
        assert!(validate_percentage(0.0).is_ok());
        assert!(validate_percentage(100.0).is_ok());
        assert!(validate_percentage(-1.0).is_err());
        assert!(validate_percentage(101.0).is_err());
    }

    #[test]
    fn test_validate_slippage() {
        assert!(validate_slippage(1.0).is_ok());
        assert!(validate_slippage(0.0).is_ok());
        assert!(validate_slippage(5.0).is_ok());
        assert!(validate_slippage(-1.0).is_err());
        assert!(validate_slippage(60.0).is_err());
    }
}
