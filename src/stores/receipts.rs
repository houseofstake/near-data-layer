use substreams::store::{StoreNew, StoreSet, StoreDelete, StoreSetProto};
use crate::pb::sf::near::r#type::v1::{Block, receipt};
use crate::pb::near::entities::v1::Receipt as ReceiptEntity;

#[substreams::handlers::store]
fn store_receipts(block: Block, store: StoreSetProto<ReceiptEntity>) {
    if let Some(header) = &block.header {
        let current_height = header.height;

        for shard in &block.shards {
            if let Some(chunk) = &shard.chunk {
                // Process execution outcomes to get receipt IDs
                for (receipt_idx, outcome) in shard.receipt_execution_outcomes.iter().enumerate() {
                    if let Some(execution_outcome) = &outcome.execution_outcome {
                        // Get the receipt from the chunk if available
                        if receipt_idx < chunk.receipts.len() {
                            let receipt = &chunk.receipts[receipt_idx];
                            let receipt_id = if let Some(id) = &execution_outcome.id {
                                hex::encode(&id.bytes)
                            } else {
                                format!("{}-{}", header.height, receipt_idx)
                            };
                            
                            let receipt_entity = ReceiptEntity {
                                height: header.height,
                                block_hash: if let Some(h) = &header.hash { hex::encode(&h.bytes) } else { "".to_string() },
                                chunk_hash: if let Some(chunk_header) = &chunk.header {
                                    hex::encode(&chunk_header.chunk_hash)
                                } else {
                                    "".to_string()
                                },
                                receipt_id: receipt_id.clone(),
                                predecessor_id: receipt.predecessor_id.clone(),
                                receiver_id: receipt.receiver_id.clone(),
                                receipt_kind: match &receipt.receipt {
                                    Some(receipt::Receipt::Action(_)) => "Action".to_string(),
                                    Some(receipt::Receipt::Data(_)) => "Data".to_string(),
                                    None => "Unknown".to_string(),
                                },
                                author: block.author.clone(),
                            };

                            let key = format!("{}-{}", header.height, receipt_id);
                            store.set(current_height, &key, &receipt_entity);
                        }
                    }
                }
            }
        }

        // Prune receipts older than 1,000 blocks
        if current_height > 1000 {
            let prune_height = current_height - 1000;
            store.delete_prefix(prune_height.try_into().unwrap(), &prune_height.to_string());
        }
    }
} 