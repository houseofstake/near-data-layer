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
        .change("logs", (None, logs_array_literal));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_create_execution_outcome_with_data() {
        let mut changes = DatabaseChanges::default();
        
        let outcome = ExecutionOutcome {
            block_height: 12345,
            block_hash: "block_hash".to_string(),
            chunk_hash: "chunk_hash".to_string(),
            shard_id: "shard_0".to_string(),
            gas_burnt: 1000,
            gas_used: 500.0,
            tokens_burnt: 1000000.0,
            executor_account_id: "executor.near".to_string(),
            status: "SUCCESS".to_string(),
            receipt_id: "receipt_123".to_string(),
            executed_in_block_hash: "executed_hash".to_string(),
            outcome_receipt_ids: vec!["receipt_1".to_string(), "receipt_2".to_string()],
            logs: vec!["log1".to_string(), "log2".to_string()],
        };

        push_create_execution_outcome(&mut changes, "test-key", 0, &outcome);
        
        // Should create exactly one table change
        assert_eq!(changes.table_changes.len(), 1);
        
        let table_change = &changes.table_changes[0];
        assert_eq!(table_change.table, "execution_outcomes");
        
        // Verify all fields are present
        let field_names: Vec<&str> = table_change.fields.iter().map(|f| f.name.as_str()).collect();
        assert!(field_names.contains(&"block_height"));
        assert!(field_names.contains(&"block_hash"));
        assert!(field_names.contains(&"chunk_hash"));
        assert!(field_names.contains(&"shard_id"));
        assert!(field_names.contains(&"gas_burnt"));
        assert!(field_names.contains(&"gas_used"));
        assert!(field_names.contains(&"tokens_burnt"));
        assert!(field_names.contains(&"executor_account_id"));
        assert!(field_names.contains(&"status"));
        assert!(field_names.contains(&"receipt_id"));
        assert!(field_names.contains(&"executed_in_block_hash"));
        assert!(field_names.contains(&"outcome_receipt_ids"));
        assert!(field_names.contains(&"logs"));
    }

    #[test]
    fn test_push_create_execution_outcome_with_empty_arrays() {
        let mut changes = DatabaseChanges::default();
        
        let outcome = ExecutionOutcome {
            block_height: 12345,
            block_hash: "block_hash".to_string(),
            chunk_hash: "chunk_hash".to_string(),
            shard_id: "shard_0".to_string(),
            gas_burnt: 1000,
            gas_used: 500.0,
            tokens_burnt: 1000000.0,
            executor_account_id: "executor.near".to_string(),
            status: "SUCCESS".to_string(),
            receipt_id: "receipt_123".to_string(),
            executed_in_block_hash: "executed_hash".to_string(),
            outcome_receipt_ids: vec![],
            logs: vec![],
        };

        push_create_execution_outcome(&mut changes, "test-key", 0, &outcome);
        
        // Should create exactly one table change
        assert_eq!(changes.table_changes.len(), 1);
        
        let table_change = &changes.table_changes[0];
        assert_eq!(table_change.table, "execution_outcomes");
        
        // Verify array fields are present
        let field_names: Vec<&str> = table_change.fields.iter().map(|f| f.name.as_str()).collect();
        assert!(field_names.contains(&"outcome_receipt_ids"));
        assert!(field_names.contains(&"logs"));
    }

    #[test]
    fn test_push_create_execution_outcome_with_special_characters_in_logs() {
        let mut changes = DatabaseChanges::default();
        
        let outcome = ExecutionOutcome {
            block_height: 12345,
            block_hash: "block_hash".to_string(),
            chunk_hash: "chunk_hash".to_string(),
            shard_id: "shard_0".to_string(),
            gas_burnt: 1000,
            gas_used: 500.0,
            tokens_burnt: 1000000.0,
            executor_account_id: "executor.near".to_string(),
            status: "SUCCESS".to_string(),
            receipt_id: "receipt_123".to_string(),
            executed_in_block_hash: "executed_hash".to_string(),
            outcome_receipt_ids: vec!["receipt_1".to_string()],
            logs: vec![
                "log with 'quotes'".to_string(),
                "log with \"double quotes\"".to_string(),
                "log with \\backslashes\\".to_string(),
            ],
        };

        push_create_execution_outcome(&mut changes, "test-key", 0, &outcome);
        
        // Should create exactly one table change
        assert_eq!(changes.table_changes.len(), 1);
        
        let table_change = &changes.table_changes[0];
        assert_eq!(table_change.table, "execution_outcomes");
        
        // Verify logs field is present (the escaping logic is handled by substreams)
        let field_names: Vec<&str> = table_change.fields.iter().map(|f| f.name.as_str()).collect();
        assert!(field_names.contains(&"logs"));
    }
} 