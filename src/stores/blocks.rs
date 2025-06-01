use substreams::store::{StoreNew, StoreSet, StoreSetProto};
use crate::pb::sf::near::r#type::v1::Block;
use crate::pb::near::entities::v1::Block as BlockEntity;
use crate::processors::utils::{bytes_to_string, format_timestamp};

#[substreams::handlers::store]
fn store_blocks(block: Block, store: StoreSetProto<BlockEntity>) {
    if let Some(header) = &block.header {
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
        store.set(0, &key, &block_entity);
    }
} 