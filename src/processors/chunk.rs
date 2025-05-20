use substreams_database_change::pb::database::DatabaseChanges;
use crate::pb::sf::near::r#type::v1::ChunkHeader;
use crate::pb::near::entities::v1::Chunk;

use crate::pushers::push_create_chunk;
use crate::processors::utils::bytes_to_string;

pub fn process_chunk(
    changes: &mut DatabaseChanges,
    chunk_header: &ChunkHeader,
    block_height: u64,
    author: &str,
) {
    let chunk = Chunk {
        height: block_height,
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
        author: author.to_string(),
    };

    let key = format!("{}-{}", block_height, hex::encode(&chunk_header.chunk_hash));
    push_create_chunk(changes, &key, 0, &chunk);
} 