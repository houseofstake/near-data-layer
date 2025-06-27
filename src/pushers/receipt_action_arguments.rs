use substreams_database_change::pb::database::{table_change::Operation, DatabaseChanges};
use crate::pb::near::entities::v1::ReceiptActionArguments;

pub fn push_create_receipt_action_arguments(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    value: &ReceiptActionArguments,
) {
    changes
        .push_change("receipt_action_arguments", key, ordinal, Operation::Create)
        .change("id", (None, &value.id))
        .change("receipt_id", (None, &value.receipt_id))
        .change("action_index", (None, value.action_index))
        .change("block_height", (None, value.block_height))
        .change("block_hash", (None, &value.block_hash))
        .change("chunk_hash", (None, &value.chunk_hash))
        .change("shard_id", (None, &value.shard_id))
        .change("method_name", (None, &value.method_name))
        .change("receiver_id", (None, &value.receiver_id))
        .change("signer_account_id", (None, &value.signer_account_id))
        .change("predecessor_id", (None, &value.predecessor_id))
        .change("args_base64", (None, &value.args_base64))
        .change("args_json", (None, &value.args_json))
        .change("gas", (None, value.gas))
        .change("deposit", (None, &value.deposit))
        .change("block_timestamp", (None, &value.block_timestamp));
} 