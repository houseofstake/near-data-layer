mod pb;

use substreams::store::{self, DeltaProto, StoreSetIfNotExistsProto, StoreNew, StoreSetIfNotExists};
use substreams_database_change::pb::database::{table_change::Operation, DatabaseChanges};
use pb::sf::near::r#type::v1::{Block, receipt};
use pb::near::block_meta::v1::{BlockMeta, ChunkMeta, ReceiptMeta};
use substreams::pb::substreams::store_delta::Operation as DeltaOperation;
use chrono::{DateTime, Utc};

/// move to utils
fn bytes_to_string(bytes: &[u8]) -> String {
    if !bytes.is_empty() {
        let mut value = 0u128;
        for &byte in bytes {
            value = (value << 8) | (byte as u128);
        }
        value.to_string()
    } else {
        "0".to_string()
    }
}

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
            gas_price: if let Some(gp) = &header.gas_price { bytes_to_string(&gp.bytes) } else { "0".to_string() },
            total_supply: if let Some(ts) = &header.total_supply { bytes_to_string(&ts.bytes) } else { "0".to_string() },
        };
        
        s.set_if_not_exists(header.height, header.height.to_string(), &block_meta);
    }
}

#[substreams::handlers::store]
fn store_chunk_meta(block: Block, s: StoreSetIfNotExistsProto<ChunkMeta>) {
    if let Some(header) = block.header.as_ref() {
        for chunk_header in &block.chunk_headers {
            let chunk_meta = ChunkMeta {
                height: header.height,
                chunk_hash: hex::encode(&chunk_header.chunk_hash),
                prev_block_hash: hex::encode(&chunk_header.prev_block_hash),
                outcome_root: hex::encode(&chunk_header.outcome_root),
                prev_state_root: hex::encode(&chunk_header.prev_state_root),
                encoded_merkle_root: hex::encode(&chunk_header.encoded_merkle_root),
                encoded_length: chunk_header.encoded_length,
                height_created: chunk_header.height_created,
                height_included: chunk_header.height_included,
                shard_id: chunk_header.shard_id,
                gas_used: chunk_header.gas_used,
                gas_limit: chunk_header.gas_limit,
                validator_reward: if let Some(vr) = &chunk_header.validator_reward { bytes_to_string(&vr.bytes) } else { "0".to_string() },
                balance_burnt: if let Some(bb) = &chunk_header.balance_burnt { bytes_to_string(&bb.bytes) } else { "0".to_string() },
                outgoing_receipts_root: hex::encode(&chunk_header.outgoing_receipts_root),
                tx_root: hex::encode(&chunk_header.tx_root),
                author: block.author.clone(),
            };

            let key = format!("{}-{}", header.height, hex::encode(&chunk_header.chunk_hash));
            s.set_if_not_exists(header.height, key, &chunk_meta);
        }
    }
}

#[substreams::handlers::store]
fn store_receipt_meta(block: Block, s: StoreSetIfNotExistsProto<ReceiptMeta>) {
    if let Some(header) = block.header.as_ref() {
        for shard in &block.shards {
            for receipt_exec_outcome in &shard.receipt_execution_outcomes {
                if let Some(receipt) = &receipt_exec_outcome.receipt {
                    let receipt_meta = ReceiptMeta {
                        height: header.height,
                        block_hash: if let Some(h) = &header.hash { hex::encode(&h.bytes) } else { "".to_string() },
                        chunk_hash: if let Some(chunk) = &shard.chunk {
                            if let Some(header) = &chunk.header {
                                hex::encode(&header.chunk_hash)
                            } else {
                                "".to_string()
                            }
                        } else {
                            "".to_string()
                        },
                        receipt_id: if let Some(id) = &receipt.receipt_id { hex::encode(&id.bytes) } else { "".to_string() },
                        predecessor_id: receipt.predecessor_id.clone(),
                        receiver_id: receipt.receiver_id.clone(),
                        receipt_kind: match &receipt.receipt {
                            Some(receipt::Receipt::Action(_)) => "Action".to_string(),
                            Some(receipt::Receipt::Data(_)) => "Data".to_string(),
                            None => "Unknown".to_string(),
                        },
                        author: block.author.clone(),
                    };

                    let key = format!("{}-{}", header.height, if let Some(id) = &receipt.receipt_id { hex::encode(&id.bytes) } else { "".to_string() });
                    s.set_if_not_exists(header.height, key, &receipt_meta);
                }
            }
        }
    }
}

#[substreams::handlers::map]
fn db_out(block_meta_deltas: store::Deltas<DeltaProto<BlockMeta>>,
          chunk_meta_deltas: store::Deltas<DeltaProto<ChunkMeta>>,
          receipt_meta_deltas: store::Deltas<DeltaProto<ReceiptMeta>>) 
    -> Result<DatabaseChanges, substreams::errors::Error> {
    let mut database_changes = DatabaseChanges::default();
    
    transform_block_meta_to_database_changes(&mut database_changes, block_meta_deltas);
    transform_chunk_meta_to_database_changes(&mut database_changes, chunk_meta_deltas);
    transform_receipt_meta_to_database_changes(&mut database_changes, receipt_meta_deltas);
    
    Ok(database_changes)
}

fn transform_block_meta_to_database_changes(
    changes: &mut DatabaseChanges,
    deltas: store::Deltas<DeltaProto<BlockMeta>>,
) {
    for delta in deltas.deltas {
        match delta.operation {
            DeltaOperation::Create => push_create_block_meta(changes, &delta.key, delta.ordinal, &delta.new_value),
            _ => {}
        }
    }
}

fn transform_chunk_meta_to_database_changes(
    changes: &mut DatabaseChanges,
    deltas: store::Deltas<DeltaProto<ChunkMeta>>,
) {
    for delta in deltas.deltas {
        match delta.operation {
            DeltaOperation::Create => push_create_chunk_meta(changes, &delta.key, delta.ordinal, &delta.new_value),
            _ => {}
        }
    }
}

fn transform_receipt_meta_to_database_changes(
    changes: &mut DatabaseChanges,
    deltas: store::Deltas<DeltaProto<ReceiptMeta>>,
) {
    for delta in deltas.deltas {
        match delta.operation {
            DeltaOperation::Create => push_create_receipt_meta(changes, &delta.key, delta.ordinal, &delta.new_value),
            _ => {}
        }
    }
}

fn push_create_block_meta(
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

fn push_create_chunk_meta(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    value: &ChunkMeta,
) {
    changes
        .push_change("chunk_meta", key, ordinal, Operation::Create)
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

fn push_create_receipt_meta(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    value: &ReceiptMeta,
) {
    changes
        .push_change("receipt_meta", key, ordinal, Operation::Create)
        .change("height", (None, value.height))
        .change("block_hash", (None, &value.block_hash))
        .change("chunk_hash", (None, &value.chunk_hash))
        .change("receipt_id", (None, &value.receipt_id))
        .change("predecessor_id", (None, &value.predecessor_id))
        .change("receiver_id", (None, &value.receiver_id))
        .change("receipt_kind", (None, &value.receipt_kind))
        .change("author", (None, &value.author));
}
