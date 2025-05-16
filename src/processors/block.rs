use substreams_database_change::pb::database::DatabaseChanges;
use crate::pb::sf::near::r#type::v1::Block;
use crate::pb::near::entities::v1::Block as BlockEntity;

use crate::pushers::push_create_block;
use crate::processors::{process_chunk, process_shard};
use crate::processors::utils::{bytes_to_string, format_timestamp};

pub fn process_block(changes: &mut DatabaseChanges, block: &Block) {
    if let Some(header) = &block.header {
        process_block_header(changes, header, &block.author);
        
        for chunk_header in &block.chunk_headers {
            process_chunk(changes, chunk_header, header.height, &block.author);
        }

        for shard in &block.shards {
            process_shard(changes, shard, header, &block.author);
        }
    }
}

pub fn process_block_header(changes: &mut DatabaseChanges, header: &crate::pb::sf::near::r#type::v1::BlockHeader, author: &str) {
    let block_entity = BlockEntity {
        height: header.height,
        hash: if let Some(h) = &header.hash { hex::encode(&h.bytes) } else { "".to_string() },
        prev_hash: if let Some(h) = &header.prev_hash { hex::encode(&h.bytes) } else { "".to_string() },
        author: author.to_string(),
        timestamp: format_timestamp(header.timestamp_nanosec),
        gas_price: if let Some(gp) = &header.gas_price { bytes_to_string(&gp.bytes) } else { "0".to_string() },
        total_supply: if let Some(ts) = &header.total_supply { bytes_to_string(&ts.bytes) } else { "0".to_string() },
    };
    
    let key = header.height.to_string();
    push_create_block(changes, &key, 0, &block_entity);
}
