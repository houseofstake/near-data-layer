use substreams::store::{StoreNew, StoreSet, StoreDelete, StoreSetProto};
use crate::pb::sf::near::r#type::v1::Block;
use crate::pb::near::entities::v1::Block as BlockEntity;
use crate::processors::utils::{bytes_to_string, format_timestamp};

#[substreams::handlers::store]
fn store_blocks(block: Block, store: StoreSetProto<BlockEntity>) {
    if let Some(header) = &block.header {
        let current_height = header.height;

        let block_entity = BlockEntity {
            height: header.height,
            hash: if let Some(h) = &header.hash { hex::encode(&h.bytes) } else { "".to_string() },
            prev_hash: if let Some(h) = &header.prev_hash { hex::encode(&h.bytes) } else { "".to_string() },
            author: block.author.clone(),
            timestamp: format_timestamp(header.timestamp_nanosec),
            gas_price: if let Some(gp) = &header.gas_price { bytes_to_string(&gp.bytes) } else { "0".to_string() },
            total_supply: if let Some(ts) = &header.total_supply { bytes_to_string(&ts.bytes) } else { "0".to_string() },
        };
        
        let key = header.height.to_string();
        store.set(current_height, &key, &block_entity);

        // Prune blocks older than 1,000 blocks
        if current_height > 1000 {
            let prune_height = current_height - 1000;
            store.delete_prefix(prune_height.try_into().unwrap(), &prune_height.to_string());
        }
    }
} 