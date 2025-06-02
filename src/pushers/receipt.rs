use substreams_database_change::pb::database::{table_change::Operation, DatabaseChanges};
use crate::pb::near::entities::v1::Receipt;

pub fn push_create_receipt(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    value: &Receipt,
) {
    changes
        .push_change("receipts", key, ordinal, Operation::Create)
        .change("height", (None, value.height))
        .change("block_hash", (None, &value.block_hash))
        .change("chunk_hash", (None, &value.chunk_hash))
        .change("receipt_id", (None, &value.receipt_id))
        .change("predecessor_id", (None, &value.predecessor_id))
        .change("receiver_id", (None, &value.receiver_id))
        .change("receipt_kind", (None, &value.receipt_kind))
        .change("author", (None, &value.author))
        .change("transaction_hash", (None, &value.transaction_hash));
} 