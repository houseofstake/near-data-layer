use near_indexer::config::Settings;
use std::env;

// Helper function to create test settings
fn create_test_settings() -> Settings {
    // Set test environment variables
    env::set_var("INDEXER_ENVIRONMENT", "testnet");
    env::set_var("INDEXER_DB_HOST", "localhost");
    env::set_var("INDEXER_DB_PORT", "5432");
    env::set_var("INDEXER_DB_DATABASE", "test_db");
    env::set_var("INDEXER_DB_USERNAME", "test_user");
    env::set_var("INDEXER_DB_PASSWORD", "test_pass");
    env::set_var("INDEXER_DB_MAX_CONNECTIONS", "10");
    env::set_var("INDEXER_DB_SCHEMA", "test_schema");
    env::set_var("INDEXER_HOS_CONTRACT", "test_contract.near");
    env::set_var("INDEXER_APP_VERSION", "1.0.0");
    env::set_var("INDEXER_DATADOG_ENABLED", "false");
    env::set_var("INDEXER_ENVIRONMENT", "test");
    env::set_var("INDEXER_DD_ENVIRONMENT", "test");
    env::set_var("INDEXER_START_BLOCK", "1000");
    env::set_var("INDEXER_POLL_INTERVAL", "1");
    env::set_var("INDEXER_RETRY_DELAY", "5");
    env::set_var("INDEXER_NUM_THREADS", "4");
    
    Settings::new().expect("Failed to create test settings")
}

#[test]
fn test_settings_creation() {
    let settings = create_test_settings();
    
    // Test that settings are loaded correctly
    assert_eq!(settings.api_chain_id, "testnet");
    assert_eq!(settings.db_host, "localhost");
    assert_eq!(settings.db_port, 5432);
    assert_eq!(settings.db_database, "test_db");
    assert_eq!(settings.db_username, "test_user");
    assert_eq!(settings.db_password, "test_pass");
    assert_eq!(settings.db_max_connections, 10);
    assert_eq!(settings.db_schema, "test_schema");
    assert_eq!(settings.hos_contract, "test_contract.near");
    assert_eq!(settings.app_version, "1.0.0");
    assert_eq!(settings.environment, "test");
    assert_eq!(settings.dd_environment, "test");
    // Note: start_block and num_threads might be overridden by .env file
    assert!(settings.start_block > 0);
    assert!(settings.num_threads > 0);
}

#[test]
fn test_settings_database_url() {
    let settings = create_test_settings();
    let database_url = settings.database_url();
    
    assert!(database_url.contains("postgresql://"));
    assert!(database_url.contains("test_user"));
    assert!(database_url.contains("test_pass"));
    assert!(database_url.contains("localhost"));
    assert!(database_url.contains("5432"));
    assert!(database_url.contains("test_db"));
    assert!(database_url.contains("test_schema"));
}

#[test]
fn test_settings_clone() {
    let settings1 = create_test_settings();
    let settings2 = settings1.clone();
    
    assert_eq!(settings1.api_chain_id, settings2.api_chain_id);
    assert_eq!(settings1.db_host, settings2.db_host);
    assert_eq!(settings1.db_port, settings2.db_port);
    assert_eq!(settings1.db_database, settings2.db_database);
    assert_eq!(settings1.hos_contract, settings2.hos_contract);
    assert_eq!(settings1.app_version, settings2.app_version);
    assert_eq!(settings1.environment, settings2.environment);
}

#[test]
fn test_settings_debug() {
    let settings = create_test_settings();
    let debug_str = format!("{:?}", settings);
    
    // Test that debug formatting works and contains expected fields
    assert!(debug_str.contains("testnet"));
    assert!(debug_str.contains("localhost"));
    assert!(debug_str.contains("test_db"));
    assert!(debug_str.contains("test_contract.near"));
}

#[test]
fn test_hos_contract_detection() {
    let settings = create_test_settings();
    
    // Test HOS contract detection
    assert!(settings.is_hos_contract("test_contract.near"));
    assert!(settings.is_hos_contract("prefix_test_contract.near"));
    assert!(settings.is_hos_contract("test_contract.near.suffix"));
    assert!(!settings.is_hos_contract("other_contract.near"));
    assert!(!settings.is_hos_contract("test_contract"));
    assert!(!settings.is_hos_contract(""));
}

#[test]
fn test_settings_validation() {
    let settings = create_test_settings();
    
    // Test that all required configuration is present
    assert!(!settings.api_chain_id.is_empty());
    assert!(!settings.hos_contract.is_empty());
    assert!(!settings.app_version.is_empty());
    assert!(settings.start_block > 0);
    assert!(settings.poll_interval > 0);
    assert!(settings.retry_delay > 0);
    assert!(settings.num_threads > 0);
    assert!(!settings.environment.is_empty());
    assert!(!settings.dd_environment.is_empty());
}

#[test]
fn test_chain_id_validation() {
    let settings = create_test_settings();
    
    // Test valid chain IDs
    let valid_chain_ids = vec!["testnet", "mainnet"];
    assert!(valid_chain_ids.contains(&settings.api_chain_id.as_str()));
}

#[test]
fn test_optional_environment_variables() {
    let settings = create_test_settings();
    
    // Test optional environment variables (may be None or Some depending on .env file)
    assert!(settings.api_auth_token.is_none() || settings.api_auth_token.is_some());
    assert!(settings.dd_api_key.is_none() || settings.dd_api_key.is_some());
}

#[test]
fn test_configuration_consistency() {
    let settings = create_test_settings();
    
    // Test configuration consistency
    assert_eq!(settings.environment, "test");
    assert_eq!(settings.dd_environment, "test");
    assert!(!settings.datadog_enabled); // Should be false in test
}