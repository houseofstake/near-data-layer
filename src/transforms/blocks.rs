use substreams::store::{self, DeltaProto};
use substreams::pb::substreams::store_delta::Operation as DeltaOperation;
use substreams_database_change::pb::database::{table_change::Operation as DbOperation, DatabaseChanges};
use crate::pb::near::entities::v1::Block as BlockEntity;
use prost::Message;

pub fn transform_blocks_deltas(
    changes: &mut DatabaseChanges,
    deltas: store::Deltas<DeltaProto<BlockEntity>>,
) {
    for delta in deltas.deltas {
        match delta.operation {
            DeltaOperation::Create => {
                push_create_block(changes, &delta.key, delta.ordinal, &delta.new_value);
            }
            DeltaOperation::Update => {
                push_create_block(changes, &delta.key, delta.ordinal, &delta.new_value);
            }
            DeltaOperation::Delete => {
                push_delete_block(changes, &delta.key, delta.ordinal);
            }
            _ => {}
        }
    }
}

fn push_create_block(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    value: &BlockEntity,
) {
    changes
        .push_change("blocks", key, ordinal, DbOperation::Create)
        .change("height", (None, value.height))
        .change("hash", (None, &value.hash))
        .change("prev_hash", (None, &value.prev_hash))
        .change("author", (None, &value.author))
        .change("timestamp", (None, &value.timestamp))
        .change("gas_price", (None, &value.gas_price))
        .change("total_supply", (None, &value.total_supply));
}

fn push_delete_block(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
) {
    changes.push_change("blocks", key, ordinal, DbOperation::Delete);
} 