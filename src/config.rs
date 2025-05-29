use anyhow::Result;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub venear_contract_ids: Vec<String>,
}

impl Settings {
    pub fn new() -> Result<Self> {
        let environment = env::var("ENV").unwrap_or_else(|_| "DEV".to_string());
        
        let config = config::Config::builder()
            .add_source(config::File::with_name(&format!("config/{}", environment.to_lowercase())))
            .build()?;

        let settings = config.try_deserialize()?;
        Ok(settings)
    }
}
