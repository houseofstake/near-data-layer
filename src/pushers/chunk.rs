use substreams_database_change::pb::database::{table_change::Operation, DatabaseChanges};
use crate::pb::near::entities::v1::Chunk;

// DEPRECATED: Chunks data is no longer needed
// Keeping function here in case we need it in the future
#[allow(dead_code)]
pub fn push_create_chunk(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    value: &Chunk,
) {
    changes
        .push_change("chunks", key, ordinal, Operation::Create)
        .change("height", (None, value.height))
        .change("chunk_hash", (None, &value.chunk_hash))
        .change("prev_block_hash", (None, &value.prev_block_hash))
        .change("outcome_root", (None, &value.outcome_root))
        .change("prev_state_root", (None, &value.prev_state_root))
        .change("encoded_merkle_root", (None, &value.encoded_merkle_root))
        .change("encoded_length", (None, value.encoded_length))
        .change("height_created", (None, value.height_created))
        .change("height_included", (None, value.height_included))
        .change("shard_id", (None, value.shard_id))
        .change("gas_used", (None, value.gas_used))
        .change("gas_limit", (None, value.gas_limit))
        .change("validator_reward", (None, &value.validator_reward))
        .change("balance_burnt", (None, &value.balance_burnt))
        .change("outgoing_receipts_root", (None, &value.outgoing_receipts_root))
        .change("tx_root", (None, &value.tx_root))
        .change("author", (None, &value.author));
} 