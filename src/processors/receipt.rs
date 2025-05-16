use substreams_database_change::pb::database::DatabaseChanges;
use crate::pb::sf::near::r#type::v1::{Receipt, BlockHeader, IndexerShard, receipt};
use crate::pb::near::entities::v1::Receipt as ReceiptEntity;

use crate::pushers::push_create_receipt;

pub fn process_receipt(
    changes: &mut DatabaseChanges,
    receipt: &Receipt,
    header: &BlockHeader,
    shard: &IndexerShard,
    receipt_id: &str,
    author: &str,
) {
    let receipt_entity = ReceiptEntity {
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
        receipt_id: receipt_id.to_string(),
        predecessor_id: receipt.predecessor_id.clone(),
        receiver_id: receipt.receiver_id.clone(),
        receipt_kind: match &receipt.receipt {
            Some(receipt::Receipt::Action(_)) => "Action".to_string(),
            Some(receipt::Receipt::Data(_)) => "Data".to_string(),
            None => "Unknown".to_string(),
        },
        author: author.to_string(),
    };

    let key = format!("{}-{}", header.height, receipt_id);
    push_create_receipt(changes, &key, 0, &receipt_entity);
} 