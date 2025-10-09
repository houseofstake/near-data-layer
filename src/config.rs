use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use dotenvy;

/*
 * CONFIGURATION SOURCES HIERARCHY (in order of precedence):
 * 
 * 1. CONFIG FILES (.toml) - Base configuration loaded from configs/testnet.toml or configs/mainnet.toml
 *    - Contains default values for non-sensitive settings
 *    - File selection based on INDEXER_API_CHAIN_ID environment variable
 * 
 * 2. TERRAFORM VARIABLES (TF_VARS) - Non-sensitive environment variables set by Terraform deployment
 *    - Passed as INDEXER_* environment variables from terraform vm.tf
 *    - Override config file values for deployment-specific settings
 * 
 * 3. SECRETS MANAGER (SECRETS) - Sensitive values fetched via fetch_secrets.sh script
 *    - Retrieved from Google Cloud Secret Manager
 *    - Highest precedence for security-sensitive configuration
 * 
 * All environment variables with "INDEXER_" prefix override config file values.
 */

#[derive(Deserialize, Clone, Debug)]
pub struct Settings {
    // Database Configuration
    pub db_host: String,              // [SECRETS] - Database hostname from secrets manager
    pub db_port: u16,                 // [CONFIG] - Database port, should be set somewhere else
    pub db_database: String,          // [CONFIG] - Database name
    pub db_username: String,          // [SECRETS] - Database username from secrets manager
    pub db_password: String,          // [SECRETS] - Database password from secrets manager
    pub db_max_connections: u32,      // [CONFIG] - Connection pool size
    pub db_schema: String,            // [CONFIG] - Database schema name
    
    // NEAR API Configuration
    pub api_auth_token: Option<String>, // [SECRETS] - FastNEAR API authentication token
    pub api_chain_id: String,           // [TF_VARS] - Chain identifier (testnet/mainnet) from terraform
    
    // Indexer Configuration
    pub start_block: u64,             // [CONFIG] - Starting block height for indexing
    pub poll_interval: u64,           // [CONFIG] - Polling interval in milliseconds
    pub retry_delay: u64,             // [CONFIG] - Retry delay for failed operations
    pub num_threads: u64,             // [CONFIG] - Number of processing threads
    
    // Contract Configuration
    pub hos_contract: String,         // [CONFIG] - HOS contract address to monitor
    
    // Logging Configuration
    #[allow(dead_code)]
    pub log_level: String,            // [CONFIG] - Log level (debug, info, warn, error)
    
    // Application Metadata
    pub app_version: String,          // [CONFIG] - Application version for cursor tracking
    
    // DataDog Metrics Configuration
    pub dd_api_key: Option<String>,   // [SECRETS] - DataDog API key from secrets manager
    pub datadog_enabled: bool,        // [TF_VARS] - Enable/disable DataDog metrics via terraform
    pub environment: String,          // [TF_VARS] - Environment name (development/production) from terraform
    pub dd_environment: String,       // [TF_VARS] - DataDog environment tag from terraform
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
        
        let env = std::env::var("ENVIRONMENT")
            .map_err(|_| ConfigError::Message(
                "ENVIRONMENT environment variable is required. Set to 'development', 'staging' or 'prod'".to_string()
        ))

        let config_file = match env.as_str() {
            "development" => "configs/mainnet-development.toml",
            "staging" => "configs/mainnet-staging.toml",
            "prod" => "configs/mainnet-prod.toml",
            _ => return Err(ConfigError::Message(
                format!("Unsupported chain_id '{}'. Must be 'dev', 'staging' or 'prod'", chain_id)
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

    pub fn is_hos_contract(&self, account_id: &str) -> bool {
        account_id.contains(&self.hos_contract)
    }
}
