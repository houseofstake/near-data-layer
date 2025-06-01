use substreams::store::{self, DeltaProto};
use substreams::pb::substreams::store_delta::Operation as DeltaOperation;
use substreams_database_change::pb::database::{table_change::Operation as DbOperation, DatabaseChanges};
use crate::pb::near::entities::v1::Receipt as ReceiptEntity;

pub fn transform_receipts_deltas(
    changes: &mut DatabaseChanges,
    deltas: store::Deltas<DeltaProto<ReceiptEntity>>,
) {
    for delta in deltas.deltas {
        match delta.operation {
            DeltaOperation::Create => {
                push_create_receipt(changes, &delta.key, delta.ordinal, &delta.new_value);
            }
            // DeltaOperation::Update => {
            //     push_create_receipt(changes, &delta.key, delta.ordinal, &delta.new_value);
            // }
            // DeltaOperation::Delete => {
            //     push_delete_receipt(changes, &delta.key, delta.ordinal);
            // }
            _ => {}
        }
    }
}

fn push_create_receipt(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    value: &ReceiptEntity,
) {
    changes
        .push_change("receipts", key, ordinal, DbOperation::Create)
        .change("height", (None, value.height))
        .change("block_hash", (None, &value.block_hash))
        .change("chunk_hash", (None, &value.chunk_hash))
        .change("receipt_id", (None, &value.receipt_id))
        .change("predecessor_id", (None, &value.predecessor_id))
        .change("receiver_id", (None, &value.receiver_id))
        .change("receipt_kind", (None, &value.receipt_kind))
        .change("author", (None, &value.author));
}

fn push_delete_receipt(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
) {
    changes.push_change("receipts", key, ordinal, DbOperation::Delete);
} 