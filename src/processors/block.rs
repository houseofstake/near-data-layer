use substreams_database_change::pb::database::DatabaseChanges;
use crate::pb::sf::near::r#type::v1::Block;
use crate::pb::near::entities::v1::Block as BlockEntity;

use crate::pushers::push_create_block;
use crate::processors::process_shard;
use crate::processors::utils::{bytes_to_string, format_timestamp};

pub fn process_block(changes: &mut DatabaseChanges, block: &Block) {
    if let Some(header) = &block.header {
        process_block_header(changes, header, &block.author);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pb::sf::near::r#type::v1::{BlockHeader, CryptoHash, BigInt};

    #[test]
    fn test_process_block_with_no_header() {
        let mut changes = DatabaseChanges::default();
        let block = Block {
            author: "test_author".to_string(),
            header: None,
            chunk_headers: vec![],
            shards: vec![],
            state_changes: vec![],
        };

        process_block(&mut changes, &block);
        
        // Should not create any changes when there's no header
        assert_eq!(changes.table_changes.len(), 0);
    }

    #[test]
    fn test_process_block_with_header() {
        let mut changes = DatabaseChanges::default();
        
        let header = BlockHeader {
            height: 12345,
            prev_height: 12344,
            epoch_id: Some(CryptoHash { bytes: vec![1, 2, 3, 4] }),
            next_epoch_id: Some(CryptoHash { bytes: vec![5, 6, 7, 8] }),
            hash: Some(CryptoHash { bytes: vec![9, 10, 11, 12] }),
            prev_hash: Some(CryptoHash { bytes: vec![13, 14, 15, 16] }),
            prev_state_root: Some(CryptoHash { bytes: vec![17, 18, 19, 20] }),
            chunk_receipts_root: Some(CryptoHash { bytes: vec![21, 22, 23, 24] }),
            chunk_headers_root: Some(CryptoHash { bytes: vec![25, 26, 27, 28] }),
            chunk_tx_root: Some(CryptoHash { bytes: vec![29, 30, 31, 32] }),
            outcome_root: Some(CryptoHash { bytes: vec![33, 34, 35, 36] }),
            chunks_included: 1,
            challenges_root: Some(CryptoHash { bytes: vec![37, 38, 39, 40] }),
            timestamp: 1640995200,
            timestamp_nanosec: 1640995200000000000,
            random_value: Some(CryptoHash { bytes: vec![41, 42, 43, 44] }),
            validator_proposals: vec![],
            chunk_mask: vec![true],
            gas_price: Some(BigInt { bytes: vec![100] }),
            block_ordinal: 1,
            total_supply: Some(BigInt { bytes: vec![232, 3, 0, 0] }),
            challenges_result: vec![],
            last_final_block_height: 12340,
            last_final_block: Some(CryptoHash { bytes: vec![45, 46, 47, 48] }),
            last_ds_final_block_height: 12340,
            last_ds_final_block: Some(CryptoHash { bytes: vec![49, 50, 51, 52] }),
            next_bp_hash: Some(CryptoHash { bytes: vec![53, 54, 55, 56] }),
            block_merkle_root: Some(CryptoHash { bytes: vec![57, 58, 59, 60] }),
            epoch_sync_data_hash: vec![61, 62, 63, 64],
            approvals: vec![],
            signature: Some(crate::pb::sf::near::r#type::v1::Signature { 
                r#type: 0, 
                bytes: vec![65, 66, 67, 68] 
            }),
            latest_protocol_version: 1,
        };

        let block = Block {
            author: "test_author".to_string(),
            header: Some(header),
            chunk_headers: vec![],
            shards: vec![],
            state_changes: vec![],
        };

        process_block(&mut changes, &block);
        
        // Should create at least one table change for the block
        assert!(changes.table_changes.len() > 0);
    }

    #[test]
    fn test_process_block_header() {
        let mut changes = DatabaseChanges::default();
        
        let header = BlockHeader {
            height: 12345,
            prev_height: 12344,
            epoch_id: Some(CryptoHash { bytes: vec![1, 2, 3, 4] }),
            next_epoch_id: Some(CryptoHash { bytes: vec![5, 6, 7, 8] }),
            hash: Some(CryptoHash { bytes: vec![9, 10, 11, 12] }),
            prev_hash: Some(CryptoHash { bytes: vec![13, 14, 15, 16] }),
            prev_state_root: Some(CryptoHash { bytes: vec![17, 18, 19, 20] }),
            chunk_receipts_root: Some(CryptoHash { bytes: vec![21, 22, 23, 24] }),
            chunk_headers_root: Some(CryptoHash { bytes: vec![25, 26, 27, 28] }),
            chunk_tx_root: Some(CryptoHash { bytes: vec![29, 30, 31, 32] }),
            outcome_root: Some(CryptoHash { bytes: vec![33, 34, 35, 36] }),
            chunks_included: 1,
            challenges_root: Some(CryptoHash { bytes: vec![37, 38, 39, 40] }),
            timestamp: 1640995200,
            timestamp_nanosec: 1640995200000000000,
            random_value: Some(CryptoHash { bytes: vec![41, 42, 43, 44] }),
            validator_proposals: vec![],
            chunk_mask: vec![true],
            gas_price: Some(BigInt { bytes: vec![100] }),
            block_ordinal: 1,
            total_supply: Some(BigInt { bytes: vec![232, 3, 0, 0] }),
            challenges_result: vec![],
            last_final_block_height: 12340,
            last_final_block: Some(CryptoHash { bytes: vec![45, 46, 47, 48] }),
            last_ds_final_block_height: 12340,
            last_ds_final_block: Some(CryptoHash { bytes: vec![49, 50, 51, 52] }),
            next_bp_hash: Some(CryptoHash { bytes: vec![53, 54, 55, 56] }),
            block_merkle_root: Some(CryptoHash { bytes: vec![57, 58, 59, 60] }),
            epoch_sync_data_hash: vec![61, 62, 63, 64],
            approvals: vec![],
            signature: Some(crate::pb::sf::near::r#type::v1::Signature { 
                r#type: 0, 
                bytes: vec![65, 66, 67, 68] 
            }),
            latest_protocol_version: 1,
        };

        process_block_header(&mut changes, &header, "test_author");
        
        // Should create at least one table change
        assert!(changes.table_changes.len() > 0);
        
        // Verify the table change is for the blocks table
        let table_change = &changes.table_changes[0];
        assert_eq!(table_change.table, "blocks");
    }
}
