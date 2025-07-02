use substreams_database_change::pb::database::DatabaseChanges;
use crate::pb::sf::near::r#type::v1::{Receipt, BlockHeader, IndexerShard, receipt};
use crate::pb::near::entities::v1::Receipt as ReceiptEntity;

use crate::pushers::push_create_receipt;

#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pb::sf::near::r#type::v1::{CryptoHash, ReceiptAction};

    #[test]
    fn test_process_receipt_action() {
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
            gas_price: Some(crate::pb::sf::near::r#type::v1::BigInt { bytes: vec![100] }),
            block_ordinal: 1,
            total_supply: Some(crate::pb::sf::near::r#type::v1::BigInt { bytes: vec![232, 3, 0, 0] }),
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

        let receipt = Receipt {
            predecessor_id: "test.predecessor".to_string(),
            receiver_id: "test.receiver".to_string(),
            receipt_id: Some(CryptoHash { bytes: vec![1, 2, 3, 4] }),
            receipt: Some(receipt::Receipt::Action(ReceiptAction {
                signer_id: "test.signer".to_string(),
                signer_public_key: None,
                gas_price: None,
                output_data_receivers: vec![],
                input_data_ids: vec![],
                actions: vec![],
            })),
        };

        let shard = IndexerShard {
            shard_id: 0,
            chunk: None,
            receipt_execution_outcomes: vec![],
        };

        process_receipt(&mut changes, &receipt, &header, &shard, "test-receipt-id", "test_author");
        
        // Should create at least one table change
        assert!(changes.table_changes.len() > 0);
        
        // Verify the table change is for the receipts table
        let table_change = &changes.table_changes[0];
        assert_eq!(table_change.table, "receipts");
    }

    #[test]
    fn test_process_receipt_data() {
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
            gas_price: Some(crate::pb::sf::near::r#type::v1::BigInt { bytes: vec![100] }),
            block_ordinal: 1,
            total_supply: Some(crate::pb::sf::near::r#type::v1::BigInt { bytes: vec![232, 3, 0, 0] }),
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

        let receipt = Receipt {
            predecessor_id: "test.predecessor".to_string(),
            receiver_id: "test.receiver".to_string(),
            receipt_id: Some(CryptoHash { bytes: vec![1, 2, 3, 4] }),
            receipt: Some(receipt::Receipt::Data(crate::pb::sf::near::r#type::v1::ReceiptData {
                data_id: Some(CryptoHash { bytes: vec![5, 6, 7, 8] }),
                data: vec![1, 2, 3, 4, 5],
            })),
        };

        let shard = IndexerShard {
            shard_id: 0,
            chunk: None,
            receipt_execution_outcomes: vec![],
        };

        process_receipt(&mut changes, &receipt, &header, &shard, "test-receipt-id", "test_author");
        
        // Should create at least one table change
        assert!(changes.table_changes.len() > 0);
        
        // Verify the table change is for the receipts table
        let table_change = &changes.table_changes[0];
        assert_eq!(table_change.table, "receipts");
    }
} 