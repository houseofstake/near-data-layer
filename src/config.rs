use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use dotenvy;
use std::fmt;

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
    pub venear_contracts: Vec<String>,
    pub log_level: String,
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

    pub fn is_venear_contract(&self, account_id: &str) -> bool {
        self.venear_contracts.iter().any(|id| account_id.contains(id))
    }
}

impl fmt::Debug for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Settings")
            .field("db_host", &self.db_host)
            .field("db_port", &self.db_port)
            .field("db_database", &self.db_database)
            .field("db_username", &self.db_username)
            .field("db_password", &"[REDACTED]")
            .field("db_max_connections", &self.db_max_connections)
            .field("db_schema", &self.db_schema)
            .field("api_url", &self.api_url)
            .field("api_auth_token", &self.api_auth_token.as_ref().map(|_| "[REDACTED]"))
            .field("api_chain_id", &self.api_chain_id)
            .field("api_finality", &self.api_finality)
            .field("start_block", &self.start_block)
            .field("batch_size", &self.batch_size)
            .field("poll_interval", &self.poll_interval)
            .field("max_retries", &self.max_retries)
            .field("retry_delay", &self.retry_delay)
            .field("num_threads", &self.num_threads)
            .field("venear_contracts", &self.venear_contracts)
            .field("log_level", &self.log_level)
            .finish()
    }
} 