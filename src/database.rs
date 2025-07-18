use crate::config::Settings;
use anyhow::Result;
use chrono::Utc;
use fastnear_primitives::block_with_tx_hash::BlockWithTxHashes;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use tracing::info;
use serde_json;

#[derive(Debug, Clone)]
pub struct ReceiptActionRow {
    pub id: String,
    pub block_height: i64,
    pub receipt_id: String,
    pub signer_account_id: String,
    pub signer_public_key: String,
    pub gas_price: String,
    pub action_kind: String,
    pub predecessor_id: String,
    pub receiver_id: String,
    pub block_hash: String,
    pub chunk_hash: String,
    pub author: String,
    pub method_name: String,
    pub gas: i64,
    pub deposit: String,
    pub args_base64: String,
    pub args_json: serde_json::Value,
    pub action_index: i32,
    pub block_timestamp: chrono::NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct ExecutionOutcomeRow {
    pub receipt_id: String,
    pub block_height: i64,
    pub block_hash: String,
    pub chunk_hash: String,
    pub shard_id: String,
    pub gas_burnt: i64,
    pub gas_used: f64,
    pub tokens_burnt: f64,
    pub executor_account_id: String,
    pub status: String,
    pub outcome_receipt_ids: Vec<String>,
    pub executed_in_block_hash: String,
    pub logs: Vec<String>,
    pub results_json: Option<serde_json::Value>,
    pub block_timestamp: Option<chrono::NaiveDateTime>,
}

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(settings: Settings) -> Result<Self> {
        let pool: sqlx::Pool<sqlx::Postgres> = PgPoolOptions::new()
            .max_connections(settings.db_max_connections)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(&settings.database_url())
            .await?;

        info!("Connected to database: {}", settings.db_database);
        Ok(Self { pool })
    }

    pub async fn initialize_tables(&self, settings: &Settings) -> Result<()> {
        // Read schema from schema.sql file
        let schema_content = std::fs::read_to_string("schema.sql")
            .map_err(|e| anyhow::anyhow!("Failed to read schema.sql: {}", e))?;
        
        // Replace schema name placeholder with actual schema name from config
        let schema_content = schema_content.replace("{SCHEMA_NAME}", &settings.db_schema);
        
        // Split the schema into individual statements
        let statements: Vec<&str> = schema_content
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.starts_with("--"))
            .collect();
        
        // Execute each statement
        for statement in statements {
            if !statement.trim().is_empty() {
                sqlx::query(statement).execute(&self.pool).await?;
            }
        }
        
        info!("Database tables initialized from schema.sql with schema: {}", settings.db_schema);
        Ok(())
    }

    pub async fn get_latest_cursor(&self) -> Result<Option<(String, u64)>> {
        let row = sqlx::query("SELECT id, block_num FROM cursors ORDER BY block_num DESC LIMIT 1")
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => Ok(Some((row.get::<String, _>("id"), row.get::<i64, _>("block_num") as u64))),
            None => Ok(None),
        }
    }

    pub async fn update_cursor(&self, id: &str, block_num: u64, block_hash: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO cursors (id, cursor, block_num, block_id) 
             VALUES ($1, $2, $3, $4) 
             ON CONFLICT (id) DO UPDATE SET 
                cursor = EXCLUDED.cursor, 
                block_num = EXCLUDED.block_num, 
                block_id = EXCLUDED.block_id"
        )
        .bind(id)
        .bind(block_hash)
        .bind(block_num as i64)
        .bind(block_hash)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn store_block(&self, block: &BlockWithTxHashes) -> Result<()> {
        let header = &block.block.header;
        // Convert timestamp from nanoseconds to DateTime<Utc>
        let secs = (header.timestamp_nanosec / 1_000_000_000) as i64;
        let nsecs = (header.timestamp_nanosec % 1_000_000_000) as u32;
        let timestamp = chrono::DateTime::<Utc>::from_timestamp(secs, nsecs)
            .unwrap_or_else(|| Utc::now());
        sqlx::query(
            "INSERT INTO blocks (height, hash, prev_hash, author, timestamp, gas_price, total_supply) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (height) DO UPDATE SET \
                hash = EXCLUDED.hash, 
                prev_hash = EXCLUDED.prev_hash, 
                author = EXCLUDED.author, 
                timestamp = EXCLUDED.timestamp, 
                gas_price = EXCLUDED.gas_price, 
                total_supply = EXCLUDED.total_supply"
        )
        .bind(header.height as i64)
        .bind(&header.hash.to_string())
        .bind(&header.prev_hash.to_string())
        .bind(&block.block.author.to_string())
        .bind(timestamp)
        .bind(&header.gas_price.to_string())
        .bind(&header.total_supply.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn store_receipt_actions(
        &self,
        actions: Vec<ReceiptActionRow>,
    ) -> Result<()> {
        for action in actions {
            sqlx::query(
                "INSERT INTO receipt_actions (
                    id, block_height, receipt_id, signer_account_id, signer_public_key, 
                    gas_price, action_kind, predecessor_id, receiver_id, block_hash, 
                    chunk_hash, author, method_name, gas, deposit, args_base64, 
                    args_json, action_index, block_timestamp
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
                ON CONFLICT (id) DO UPDATE SET
                    block_height = EXCLUDED.block_height,
                    receipt_id = EXCLUDED.receipt_id,
                    signer_account_id = EXCLUDED.signer_account_id,
                    signer_public_key = EXCLUDED.signer_public_key,
                    gas_price = EXCLUDED.gas_price,
                    action_kind = EXCLUDED.action_kind,
                    predecessor_id = EXCLUDED.predecessor_id,
                    receiver_id = EXCLUDED.receiver_id,
                    block_hash = EXCLUDED.block_hash,
                    chunk_hash = EXCLUDED.chunk_hash,
                    author = EXCLUDED.author,
                    method_name = EXCLUDED.method_name,
                    gas = EXCLUDED.gas,
                    deposit = EXCLUDED.deposit,
                    args_base64 = EXCLUDED.args_base64,
                    args_json = EXCLUDED.args_json,
                    action_index = EXCLUDED.action_index,
                    block_timestamp = EXCLUDED.block_timestamp"
            )
            .bind(&action.id)
            .bind(action.block_height)
            .bind(&action.receipt_id)
            .bind(&action.signer_account_id)
            .bind(&action.signer_public_key)
            .bind(&action.gas_price)
            .bind(&action.action_kind)
            .bind(&action.predecessor_id)
            .bind(&action.receiver_id)
            .bind(&action.block_hash)
            .bind(&action.chunk_hash)
            .bind(&action.author)
            .bind(&action.method_name)
            .bind(action.gas)
            .bind(&action.deposit)
            .bind(&action.args_base64)
            .bind(&action.args_json)
            .bind(action.action_index)
            .bind(action.block_timestamp)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn store_execution_outcomes(
        &self,
        outcomes: Vec<ExecutionOutcomeRow>,
    ) -> Result<()> {
        for outcome in outcomes {
            sqlx::query(
                "INSERT INTO execution_outcomes (
                    receipt_id, block_height, block_hash, chunk_hash, shard_id,
                    gas_burnt, gas_used, tokens_burnt, executor_account_id, status,
                    outcome_receipt_ids, executed_in_block_hash, logs, results_json, block_timestamp
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                ON CONFLICT (receipt_id) DO UPDATE SET
                    block_height = EXCLUDED.block_height,
                    block_hash = EXCLUDED.block_hash,
                    chunk_hash = EXCLUDED.chunk_hash,
                    shard_id = EXCLUDED.shard_id,
                    gas_burnt = EXCLUDED.gas_burnt,
                    gas_used = EXCLUDED.gas_used,
                    tokens_burnt = EXCLUDED.tokens_burnt,
                    executor_account_id = EXCLUDED.executor_account_id,
                    status = EXCLUDED.status,
                    outcome_receipt_ids = EXCLUDED.outcome_receipt_ids,
                    executed_in_block_hash = EXCLUDED.executed_in_block_hash,
                    logs = EXCLUDED.logs,
                    results_json = EXCLUDED.results_json,
                    block_timestamp = EXCLUDED.block_timestamp"
            )
            .bind(&outcome.receipt_id)
            .bind(outcome.block_height)
            .bind(&outcome.block_hash)
            .bind(&outcome.chunk_hash)
            .bind(&outcome.shard_id)
            .bind(outcome.gas_burnt)
            .bind(outcome.gas_used)
            .bind(outcome.tokens_burnt)
            .bind(&outcome.executor_account_id)
            .bind(&outcome.status)
            .bind(&outcome.outcome_receipt_ids)
            .bind(&outcome.executed_in_block_hash)
            .bind(&outcome.logs)
            .bind(&outcome.results_json)
            .bind(outcome.block_timestamp)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

} 