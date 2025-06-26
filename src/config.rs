use anyhow::Result;
use serde_json::Value;

const DEV_CONFIG: &str = include_str!("../config/dev.json");

#[derive(Debug)]
pub struct Settings {
    pub venear_contract_ids: Vec<String>,
}

impl Settings {
    pub fn new() -> Result<Self> {
        let config_str = match std::env::var("ENV").unwrap_or_else(|_| "DEV".to_string()).as_str() {
            "DEV" => DEV_CONFIG,
            _ => DEV_CONFIG,
        };

        let config: Value = serde_json::from_str(config_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse config: {}", e))?;

        let contract_ids = config["venear_contract_ids"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("venear_contract_ids must be an array"))?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        Ok(Settings {
            venear_contract_ids: contract_ids,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_new_success() {
        let result = Settings::new();
        assert!(result.is_ok());
        
        let settings = result.unwrap();
        assert!(!settings.venear_contract_ids.is_empty());
    }

    #[test]
    fn test_settings_contract_ids_not_empty() {
        let settings = Settings::new().unwrap();
        // Verify that we have some contract IDs loaded
        assert!(!settings.venear_contract_ids.is_empty());
        
        // Verify that all contract IDs are valid strings
        for contract_id in &settings.venear_contract_ids {
            assert!(!contract_id.is_empty());
            assert!(contract_id.contains('.'));
        }
    }

    #[test]
    fn test_env_variable_handling() {
        // Test with no ENV variable set (should default to DEV)
        let result = Settings::new();
        assert!(result.is_ok());
        
        // Test with ENV=DEV
        std::env::set_var("ENV", "DEV");
        let result = Settings::new();
        assert!(result.is_ok());
        
        // Clean up
        std::env::remove_var("ENV");
    }
}
