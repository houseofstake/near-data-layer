use substreams_database_change::pb::database::{table_change::Operation, DatabaseChanges};
use crate::pb::near::entities::v1::ExecutionOutcomeResult;

pub fn push_create_execution_outcome_result(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    value: &ExecutionOutcomeResult,
) {
    changes
        .push_change("execution_outcome_results", key, ordinal, Operation::Create)
        .change("receipt_id", (None, &value.receipt_id))
        .change("block_height", (None, value.block_height))
        .change("block_hash", (None, &value.block_hash))
        .change("chunk_hash", (None, &value.chunk_hash))
        .change("shard_id", (None, &value.shard_id))
        .change("status", (None, &value.status))
        .change("result_value", (None, &value.result_value))
        .change("result_json", (None, &value.result_json))
        .change("block_timestamp", (None, &value.block_timestamp));
} 