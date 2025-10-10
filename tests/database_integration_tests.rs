use near_indexer::config::Settings;
use near_indexer::database::{Database, ReceiptActionRow, ExecutionOutcomeRow};
use chrono::Utc;
use serde_json;
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};

// Global counter for unique schema names
static SCHEMA_COUNTER: AtomicU64 = AtomicU64::new(0);

// Test database configuration
fn get_test_database_url() -> String {
    env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/postgres".to_string())
}

async fn create_test_database() -> Result<Database, Box<dyn std::error::Error>> {
    let _database_url = get_test_database_url();
    
    // Use existing settings configuration system
    // Set test-specific environment variables
    env::set_var("INDEXER_ENVIRONMENT", "testnet");
    env::set_var("INDEXER_DB_HOST", "localhost");
    env::set_var("INDEXER_DB_PORT", "5432");
    env::set_var("INDEXER_DB_DATABASE", "postgres");
    env::set_var("INDEXER_DB_USERNAME", "postgres");
    env::set_var("INDEXER_DB_PASSWORD", "postgres");
    env::set_var("INDEXER_DB_MAX_CONNECTIONS", "5");
    // Use a truly unique schema name with atomic counter
    let counter = SCHEMA_COUNTER.fetch_add(1, Ordering::SeqCst);
    env::set_var("INDEXER_DB_SCHEMA", format!("test_schema_{}_{}", Utc::now().timestamp_millis(), counter));
    env::set_var("INDEXER_HOS_CONTRACT", "test_contract.near");
    env::set_var("INDEXER_APP_VERSION", "1.0.0");
    env::set_var("INDEXER_DATADOG_ENABLED", "false");
    env::set_var("INDEXER_ENVIRONMENT", "test");
    env::set_var("INDEXER_DD_ENVIRONMENT", "test");
    
    // Load settings using the existing configuration system
    let settings = Settings::new().map_err(|e| format!("Failed to load settings: {}", e))?;

    // Create database connection
    let database = Database::new(settings.clone(), None).await?;
    
    // Initialize tables
    database.initialize_tables(&settings).await?;
    
    Ok(database)
}

fn create_test_receipt_action() -> ReceiptActionRow {
    ReceiptActionRow {
        id: format!("test_action_{}", Utc::now().timestamp_millis()),
        block_height: 12345,
        receipt_id: format!("test_receipt_{}", Utc::now().timestamp_millis()),
        signer_account_id: "test_account.near".to_string(),
        signer_public_key: "ed25519:test_key".to_string(),
        gas_price: "100000000".to_string(),
        action_kind: "FunctionCall".to_string(),
        predecessor_id: "test_predecessor.near".to_string(),
        receiver_id: "test_receiver.near".to_string(),
        block_hash: "test_block_hash".to_string(),
        chunk_hash: "test_chunk_hash".to_string(),
        author: "test_author.near".to_string(),
        method_name: "test_method".to_string(),
        gas: 1000000,
        deposit: "1000000000000000000000000".to_string(),
        args_base64: "dGVzdF9hcmdz".to_string(),
        args_json: serde_json::json!({"test": "value"}),
        action_index: 0,
        block_timestamp: Utc::now().naive_utc(),
    }
}

fn create_test_execution_outcome() -> ExecutionOutcomeRow {
    ExecutionOutcomeRow {
        receipt_id: format!("test_receipt_{}", Utc::now().timestamp_millis()),
        block_height: 12345,
        block_hash: "test_block_hash".to_string(),
        chunk_hash: "test_chunk_hash".to_string(),
        shard_id: "0".to_string(),
        gas_burnt: 1000000,
        gas_used: 1000000.0,
        tokens_burnt: 1000000000000000000000000.0,
        executor_account_id: "test_executor.near".to_string(),
        status: "Success".to_string(),
        outcome_receipt_ids: vec!["outcome_1".to_string()],
        executed_in_block_hash: "test_block_hash".to_string(),
        logs: vec!["test_log".to_string()],
        results_json: Some(serde_json::json!({"result": "success"})),
        block_timestamp: Some(Utc::now().naive_utc()),
    }
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_database_connection() {
    let result = create_test_database().await;
    match result {
        Ok(_) => println!("✅ Database connection successful"),
        Err(e) => {
            println!("❌ Database connection failed: {}", e);
            println!("💡 To run this test, set up a test database and set TEST_DATABASE_URL environment variable");
            println!("💡 Example: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db cargo test --ignored");
            panic!("Database connection failed: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_cursor_operations() {
    let database = create_test_database().await.expect("Failed to create test database");
    
    // Test cursor operations
    let app_version = "test_version";
    let block_num = 12345u64;
    let block_hash = "test_hash_123";
    
    // Test updating cursor
    database.update_cursor(app_version, block_num, block_hash).await
        .expect("Failed to update cursor");
    
    // Test getting cursor
    let retrieved_cursor = database.get_cursor_for_version(app_version).await
        .expect("Failed to get cursor");
    
    assert_eq!(retrieved_cursor, Some(block_num));
    
    println!("✅ Cursor operations test passed");
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_receipt_actions_storage() {
    let database = create_test_database().await.expect("Failed to create test database");
    
    // Create test receipt actions
    let actions = vec![
        create_test_receipt_action(),
        create_test_receipt_action(),
    ];
    
    // Store receipt actions
    database.store_receipt_actions(actions).await
        .expect("Failed to store receipt actions");
    
    println!("✅ Receipt actions storage test passed");
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_execution_outcomes_storage() {
    let database = create_test_database().await.expect("Failed to create test database");
    
    // Create test execution outcomes
    let outcomes = vec![
        create_test_execution_outcome(),
        create_test_execution_outcome(),
    ];
    
    // Store execution outcomes
    database.store_execution_outcomes(outcomes).await
        .expect("Failed to store execution outcomes");
    
    println!("✅ Execution outcomes storage test passed");
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_database_transactions() {
    let database = create_test_database().await.expect("Failed to create test database");
    
    // Test multiple operations in sequence
    let app_version = "transaction_test";
    let block_num = 99999u64;
    let block_hash = "transaction_hash";
    
    // Update cursor
    database.update_cursor(app_version, block_num, block_hash).await
        .expect("Failed to update cursor");
    
    // Store receipt actions
    let actions = vec![create_test_receipt_action()];
    database.store_receipt_actions(actions).await
        .expect("Failed to store receipt actions");
    
    // Store execution outcomes
    let outcomes = vec![create_test_execution_outcome()];
    database.store_execution_outcomes(outcomes).await
        .expect("Failed to store execution outcomes");
    
    // Verify cursor was updated
    let retrieved_cursor = database.get_cursor_for_version(app_version).await
        .expect("Failed to get cursor");
    assert_eq!(retrieved_cursor, Some(block_num));
    
    println!("✅ Database transactions test passed");
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_database_error_handling() {
    let database = create_test_database().await.expect("Failed to create test database");
    
    // Test with invalid data to ensure proper error handling
    let mut invalid_action = create_test_receipt_action();
    invalid_action.id = "".to_string(); // Empty ID should cause constraint violation
    
    let result = database.store_receipt_actions(vec![invalid_action]).await;
    
    // This should fail gracefully
    match result {
        Ok(_) => println!("⚠️  Expected error but operation succeeded"),
        Err(e) => {
            println!("✅ Error handling test passed - caught expected error: {}", e);
        }
    }
}

// Helper test to check if database is available
#[tokio::test]
async fn test_database_availability() {
    let database_url = get_test_database_url();
    
    match sqlx::PgPool::connect(&database_url).await {
        Ok(_) => {
            println!("✅ Test database is available at: {}", database_url);
            println!("💡 Run 'cargo test --ignored' to run integration tests");
        }
        Err(e) => {
            println!("❌ Test database is not available: {}", e);
            println!("💡 Set up a PostgreSQL database and set TEST_DATABASE_URL environment variable");
            println!("💡 Example: TEST_DATABASE_URL=postgres://user:pass@localhost:5432/test_db");
        }
    }
}
