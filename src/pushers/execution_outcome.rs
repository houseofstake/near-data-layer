use substreams_database_change::pb::database::{table_change::Operation, DatabaseChanges};
use crate::pb::near::entities::v1::ExecutionOutcome;

pub fn push_create_execution_outcome(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    value: &ExecutionOutcome,
) {
    // Format the array field as a Postgres-style array literal: '{a,b,c}'
    let array_literal = if !value.outcome_receipt_ids.is_empty() {
        format!(
            "'{{{}}}'",
            value
                .outcome_receipt_ids
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(",")
        )
    } else {
        "'{}'".to_string()
    };

    let logs_array_literal = if !value.logs.is_empty() {
        format!(
            "'{{{}}}'",
            value
                .logs
                .iter()
                .map(|s| {
                    let escaped = s
                        .replace('\\', "\\\\")  // Escape backslashes
                        .replace('"', "\\\"")   // Escape double quotes
                        .replace('\'', "''");   // Escape single quotes for SQL string literal context
                    format!("\"{}\"", escaped)  // Wrap each entry in double quotes for Postgres array
                })
                .collect::<Vec<_>>()
                .join(",")
        )
    } else {
        "'{}'".to_string()
    };

    changes
        .push_change("execution_outcomes", key, ordinal, Operation::Create)
        .change("block_height", (None, value.block_height))
        .change("block_hash", (None, &value.block_hash))
        .change("chunk_hash", (None, &value.chunk_hash))
        .change("shard_id", (None, &value.shard_id))
        .change("gas_burnt", (None, value.gas_burnt))
        .change("gas_used", (None, value.gas_used.to_string()))
        .change("tokens_burnt", (None, &value.tokens_burnt.to_string()))
        .change("executor_account_id", (None, &value.executor_account_id))
        .change("status", (None, &value.status))
        .change("receipt_id", (None, &value.receipt_id))
        .change("executed_in_block_hash", (None, &value.executed_in_block_hash))
        .change("outcome_receipt_ids", (None, array_literal))
        .change("logs", (None, logs_array_literal))
        .change("results_base64", (None, &value.results_base64))
        .change("results_json", (None, &value.results_json))
        .change("block_timestamp", (None, &value.block_timestamp));
} 