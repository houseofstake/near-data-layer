mod pb;

use substreams::store::{self, DeltaProto, StoreSetIfNotExistsProto, StoreNew, StoreSetIfNotExists};
use substreams_database_change::pb::database::{table_change::Operation, DatabaseChanges};
use pb::sf::near::r#type::v1::Block;
use pb::near::block_meta::v1::BlockMeta;
use substreams::pb::substreams::store_delta::Operation as DeltaOperation;
use chrono::{DateTime, Utc};

/// Process NEAR blocks and output database changes
#[substreams::handlers::store]
fn store_block_meta(block: Block, s: StoreSetIfNotExistsProto<BlockMeta>) {
    if let Some(header) = block.header.as_ref() {
        let seconds = (header.timestamp_nanosec / 1_000_000_000) as i64;
        let nanos = (header.timestamp_nanosec % 1_000_000_000) as u32;
        
        let datetime = DateTime::<Utc>::from_timestamp(seconds, nanos).unwrap();
        
        let timestamp = datetime.format("%Y-%m-%d %H:%M:%S.%f").to_string();
        
        let block_meta = BlockMeta {
            height: header.height,
            hash: if let Some(h) = &header.hash { hex::encode(&h.bytes) } else { "".to_string() },
            prev_hash: if let Some(h) = &header.prev_hash { hex::encode(&h.bytes) } else { "".to_string() },
            author: block.author.clone(),
            timestamp,
            gas_price: if let Some(gp) = &header.gas_price { 
                // Access the bytes and convert to a number string
                let bytes = &gp.bytes;
                if !bytes.is_empty() {
                    // Convert bytes to a number
                    let mut value = 0u128;
                    for &byte in bytes {
                        value = (value << 8) | (byte as u128);
                    }
                    value.to_string()
                } else {
                    "0".to_string()
                }
            } else { 
                "0".to_string() 
            },
            total_supply: if let Some(ts) = &header.total_supply {
                // Access the bytes and convert to a number string
                let bytes = &ts.bytes;
                if !bytes.is_empty() {
                    // Convert bytes to a number
                    let mut value = 0u128;
                    for &byte in bytes {
                        value = (value << 8) | (byte as u128);
                    }
                    value.to_string()
                } else {
                    "0".to_string()
                }
            } else { 
                "0".to_string() 
            },
        };
        
        s.set_if_not_exists(header.height, header.height.to_string(), &block_meta);
    }
}

#[substreams::handlers::map]
fn db_out(block_meta_deltas: store::Deltas<DeltaProto<BlockMeta>>) -> Result<DatabaseChanges, substreams::errors::Error> {
    let mut database_changes = DatabaseChanges::default();
    transform_block_meta_to_database_changes(&mut database_changes, block_meta_deltas);
    Ok(database_changes)
}

fn transform_block_meta_to_database_changes(
    changes: &mut DatabaseChanges,
    deltas: store::Deltas<DeltaProto<BlockMeta>>,
) {
    for delta in deltas.deltas {
        match delta.operation {
            DeltaOperation::Create => push_create(changes, &delta.key, delta.ordinal, &delta.new_value),
            _ => {}
        }
    }
}

fn push_create(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    value: &BlockMeta,
) {
    changes
        .push_change("block_meta", key, ordinal, Operation::Create)
        .change("height", (None, value.height))
        .change("hash", (None, &value.hash))
        .change("prev_hash", (None, &value.prev_hash))
        .change("author", (None, &value.author))
        .change("timestamp", (None, &value.timestamp))
        .change("gas_price", (None, &value.gas_price))
        .change("total_supply", (None, &value.total_supply));
}
