use substreams_database_change::pb::database::DatabaseChanges;
use crate::pb::sf::near::r#type::v1::{ExecutionOutcomeWithId, BlockHeader, IndexerShard, execution_outcome};
use crate::pb::near::entities::v1::ExecutionOutcome as ExecutionOutcomeEntity;

use crate::pushers::push_create_execution_outcome;
use crate::processors::utils::bytes_to_string;

pub fn process_execution_outcome(
    changes: &mut DatabaseChanges,
    execution_outcome: &ExecutionOutcomeWithId,
    header: &BlockHeader,
    shard: &IndexerShard,
    receipt_id: &str,
) {
    if let Some(outcome) = &execution_outcome.outcome {
        let outcome_receipt_ids: Vec<String> = outcome.receipt_ids.iter()
            .map(|id| hex::encode(&id.bytes))
            .collect();

        let status = match &outcome.status {
            Some(execution_outcome::Status::Unknown(_)) => "Unknown".to_string(),
            Some(execution_outcome::Status::Failure(_)) => "Failure".to_string(),
            Some(execution_outcome::Status::SuccessValue(_)) => "SuccessValue".to_string(),
            Some(execution_outcome::Status::SuccessReceiptId(_)) => "SuccessReceiptId".to_string(),
            None => "Unknown".to_string(),
        };

        let tokens_burnt = if let Some(tb) = &outcome.tokens_burnt {
            bytes_to_string(&tb.bytes).parse::<f32>().unwrap_or(0.0)
        } else {
            0.0
        };

        let execution_outcome_entity = ExecutionOutcomeEntity {
            block_height: header.height,
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
            shard_id: shard.shard_id.to_string(),
            gas_burnt: outcome.gas_burnt,
            gas_used: outcome.gas_burnt as f32,
            tokens_burnt,
            executor_account_id: outcome.executor_id.clone(),
            status,
            outcome_receipt_ids,
            receipt_id: receipt_id.to_string(),
            executed_in_block_hash: if let Some(block_hash) = &execution_outcome.block_hash {
                hex::encode(&block_hash.bytes) 
            } else {
                "".to_string()
            },
            logs: outcome.logs.clone(),
        };

        let key = format!("{}-{}", header.height, receipt_id);
        push_create_execution_outcome(changes, &key, 0, &execution_outcome_entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pb::sf::near::r#type::v1::{CryptoHash, ExecutionOutcome, execution_outcome::Status};

    fn create_test_header() -> BlockHeader {
        BlockHeader {
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
        }
    }

    fn create_test_shard() -> IndexerShard {
        IndexerShard {
            shard_id: 0,
            chunk: None,
            receipt_execution_outcomes: vec![],
        }
    }

    #[test]
    fn test_process_execution_outcome_success() {
        let mut changes = DatabaseChanges::default();
        let header = create_test_header();
        let shard = create_test_shard();

        let execution_outcome = ExecutionOutcomeWithId {
            proof: None,
            block_hash: Some(CryptoHash { bytes: vec![1, 2, 3, 4] }),
            id: Some(CryptoHash { bytes: vec![5, 6, 7, 8] }),
            outcome: Some(ExecutionOutcome {
                logs: vec!["log1".to_string(), "log2".to_string()],
                receipt_ids: vec![CryptoHash { bytes: vec![9, 10, 11, 12] }],
                gas_burnt: 1000,
                tokens_burnt: Some(crate::pb::sf::near::r#type::v1::BigInt { bytes: vec![100, 0, 0, 0] }),
                executor_id: "test.executor".to_string(),
                metadata: 0,
                status: Some(Status::SuccessValue(crate::pb::sf::near::r#type::v1::SuccessValueExecutionStatus {
                    value: vec![1, 2, 3, 4],
                })),
            }),
        };

        process_execution_outcome(&mut changes, &execution_outcome, &header, &shard, "test-receipt-id");
        
        // Should create at least one table change
        assert!(changes.table_changes.len() > 0);
        
        // Verify the table change is for the execution_outcomes table
        let table_change = &changes.table_changes[0];
        assert_eq!(table_change.table, "execution_outcomes");
    }

    #[test]
    fn test_process_execution_outcome_failure() {
        let mut changes = DatabaseChanges::default();
        let header = create_test_header();
        let shard = create_test_shard();

        let execution_outcome = ExecutionOutcomeWithId {
            proof: None,
            block_hash: Some(CryptoHash { bytes: vec![1, 2, 3, 4] }),
            id: Some(CryptoHash { bytes: vec![5, 6, 7, 8] }),
            outcome: Some(ExecutionOutcome {
                logs: vec!["error log".to_string()],
                receipt_ids: vec![],
                gas_burnt: 500,
                tokens_burnt: Some(crate::pb::sf::near::r#type::v1::BigInt { bytes: vec![50, 0, 0, 0] }),
                executor_id: "test.executor".to_string(),
                metadata: 0,
                status: Some(Status::Failure(crate::pb::sf::near::r#type::v1::FailureExecutionStatus {
                    failure: None,
                })),
            }),
        };

        process_execution_outcome(&mut changes, &execution_outcome, &header, &shard, "test-receipt-id");
        
        // Should create at least one table change
        assert!(changes.table_changes.len() > 0);
        
        // Verify the table change is for the execution_outcomes table
        let table_change = &changes.table_changes[0];
        assert_eq!(table_change.table, "execution_outcomes");
    }

    #[test]
    fn test_process_execution_outcome_no_outcome() {
        let mut changes = DatabaseChanges::default();
        let header = create_test_header();
        let shard = create_test_shard();

        let execution_outcome = ExecutionOutcomeWithId {
            proof: None,
            block_hash: Some(CryptoHash { bytes: vec![1, 2, 3, 4] }),
            id: Some(CryptoHash { bytes: vec![5, 6, 7, 8] }),
            outcome: None,
        };

        process_execution_outcome(&mut changes, &execution_outcome, &header, &shard, "test-receipt-id");
        
        // Should not create any changes when there's no outcome
        assert_eq!(changes.table_changes.len(), 0);
    }

    #[test]
    fn test_process_execution_outcome_with_chunk() {
        let mut changes = DatabaseChanges::default();
        let header = create_test_header();
        
        let shard = IndexerShard {
            shard_id: 1,
            chunk: Some(crate::pb::sf::near::r#type::v1::IndexerChunk {
                author: "test.author".to_string(),
                header: Some(crate::pb::sf::near::r#type::v1::ChunkHeader {
                    chunk_hash: vec![1, 2, 3, 4],
                    prev_block_hash: vec![5, 6, 7, 8],
                    outcome_root: vec![9, 10, 11, 12],
                    prev_state_root: vec![13, 14, 15, 16],
                    encoded_merkle_root: vec![17, 18, 19, 20],
                    encoded_length: 1000,
                    height_created: 12340,
                    height_included: 12345,
                    shard_id: 1,
                    gas_used: 500,
                    gas_limit: 1000,
                    validator_reward: None,
                    balance_burnt: None,
                    outgoing_receipts_root: vec![21, 22, 23, 24],
                    tx_root: vec![25, 26, 27, 28],
                    validator_proposals: vec![],
                    signature: None,
                }),
                transactions: vec![],
                receipts: vec![],
            }),
            receipt_execution_outcomes: vec![],
        };

        let execution_outcome = ExecutionOutcomeWithId {
            proof: None,
            block_hash: Some(CryptoHash { bytes: vec![1, 2, 3, 4] }),
            id: Some(CryptoHash { bytes: vec![5, 6, 7, 8] }),
            outcome: Some(ExecutionOutcome {
                logs: vec!["log1".to_string()],
                receipt_ids: vec![CryptoHash { bytes: vec![9, 10, 11, 12] }],
                gas_burnt: 1000,
                tokens_burnt: None,
                executor_id: "test.executor".to_string(),
                metadata: 0,
                status: Some(Status::SuccessValue(crate::pb::sf::near::r#type::v1::SuccessValueExecutionStatus {
                    value: vec![1, 2, 3, 4],
                })),
            }),
        };

        process_execution_outcome(&mut changes, &execution_outcome, &header, &shard, "test-receipt-id");
        
        // Should create at least one table change
        assert!(changes.table_changes.len() > 0);
        
        // Verify the table change is for the execution_outcomes table
        let table_change = &changes.table_changes[0];
        assert_eq!(table_change.table, "execution_outcomes");
    }
} 