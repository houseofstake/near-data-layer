use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use dotenvy;
use std::collections::HashMap;

#[derive(Deserialize, Clone)]
pub struct Settings {
    // Database
    pub db_host: String,
    pub db_port: u16,
    pub db_database: String,
    pub db_username: String,
    pub db_password: String,
    pub db_max_connections: u32,
    pub db_schema: String,
    // NEAR API
    pub api_url: String,
    pub api_auth_token: Option<String>,
    pub api_chain_id: String,
    pub api_finality: String,
    // Indexer
    pub start_block: u64,
    pub batch_size: u32,
    pub poll_interval: u64,
    pub max_retries: u32,
    pub retry_delay: u64,
    pub num_threads: u64,
    // Other
    pub venear_contracts: HashMap<String, Vec<String>>,
    pub log_level: String,
    // App version (from config file)
    pub app_version: String,
    // DataDog metrics
    pub dd_api_key: Option<String>,
    pub datadog_enabled: bool,
    pub environment: String,
    pub dd_environment: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Load .env file if present, so env vars are available for config
        dotenvy::dotenv().ok();

        let config = Config::builder()
            .add_source(File::with_name("config.toml").required(true))
            .add_source(Environment::with_prefix("INDEXER"))
            .build()?;

        config.try_deserialize()
    }

    pub fn database_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}?options=-csearch_path={}",
            self.db_username,
            self.db_password,
            self.db_host,
            self.db_port,
            self.db_database,
            self.db_schema
        )
    }

    pub fn get_venear_contracts(&self) -> &Vec<String> {
        self.venear_contracts
            .get(&self.api_chain_id)
            .unwrap_or_else(|| {
                // Fallback to testnet if chain_id not found
                self.venear_contracts
                    .get("testnet")
                    .expect("No venear contracts found for current chain_id or testnet fallback")
            })
    }

    pub fn is_venear_contract(&self, account_id: &str) -> bool {
        self.get_venear_contracts().iter().any(|id| account_id.contains(id))
    }
}
