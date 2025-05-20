use substreams_database_change::pb::database::{table_change::Operation, DatabaseChanges};
use crate::pb::near::entities::v1::Block;

pub fn push_create_block(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    value: &Block,
) {
    changes
        .push_change("blocks", key, ordinal, Operation::Create)
        .change("height", (None, value.height))
        .change("hash", (None, &value.hash))
        .change("prev_hash", (None, &value.prev_hash))
        .change("author", (None, &value.author))
        .change("timestamp", (None, &value.timestamp))
        .change("gas_price", (None, &value.gas_price))
        .change("total_supply", (None, &value.total_supply));
} 