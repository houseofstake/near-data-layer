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
