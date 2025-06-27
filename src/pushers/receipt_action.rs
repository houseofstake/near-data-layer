use substreams_database_change::pb::database::{table_change::Operation, DatabaseChanges};
use crate::pb::near::entities::v1::ReceiptAction;

pub fn push_create_receipt_action(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    value: &ReceiptAction,
) {
    changes
        .push_change("receipt_actions", key, ordinal, Operation::Create)
        .change("id", (None, &value.id))
        .change("block_height", (None, value.block_height))
        .change("receipt_id", (None, &value.receipt_id))
        .change("signer_account_id", (None, &value.signer_account_id))
        .change("signer_public_key", (None, &value.signer_public_key))
        .change("gas_price", (None, &value.gas_price))
        .change("action_kind", (None, &value.action_kind))
        .change("predecessor_id", (None, &value.predecessor_id))
        .change("receiver_id", (None, &value.receiver_id))
        .change("block_hash", (None, &value.block_hash))
        .change("chunk_hash", (None, &value.chunk_hash))
        .change("author", (None, &value.author))
        .change("method_name", (None, &value.method_name))
        .change("gas", (None, value.gas))
        .change("deposit", (None, &value.deposit))
        .change("args_base64", (None, &value.args_base64))
        .change("args_json", (None, &value.args_json))
        .change("action_index", (None, value.action_index))
        .change("block_timestamp", (None, &value.block_timestamp));
} 