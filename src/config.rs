use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use dotenvy;

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
    pub api_auth_token: Option<String>,
    pub api_chain_id: String,
    // Indexer
    pub start_block: u64,
    pub poll_interval: u64,
    pub retry_delay: u64,
    pub num_threads: u64,
    // Other
    pub hos_contracts: Vec<String>,
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

        // Determine config file based on required INDEXER_API_CHAIN_ID environment variable
        let chain_id = std::env::var("INDEXER_API_CHAIN_ID")
            .map_err(|_| ConfigError::Message(
                "INDEXER_API_CHAIN_ID environment variable is required. Set to 'testnet' or 'mainnet'".to_string()
            ))?;

        let config_file = match chain_id.as_str() {
            "testnet" => "configs/testnet.toml",
            "mainnet" => "configs/mainnet.toml",
            _ => return Err(ConfigError::Message(
                format!("Unsupported chain_id '{}'. Must be 'testnet' or 'mainnet'", chain_id)
            )),
        };

        let config = Config::builder()
            .add_source(File::with_name(config_file).required(true))
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

    pub fn get_hos_contracts(&self) -> &Vec<String> {
        &self.hos_contracts
    }

    pub fn is_hos_contract(&self, account_id: &str) -> bool {
        self.hos_contracts.iter().any(|id| account_id.contains(id))
    }
}
