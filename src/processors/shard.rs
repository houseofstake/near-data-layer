use substreams_database_change::pb::database::DatabaseChanges;
use crate::pb::sf::near::r#type::v1::{IndexerShard, BlockHeader, receipt};
use std::collections::HashSet;

use crate::processors::{process_receipt_actions, process_execution_outcome};

pub fn process_shard(
    changes: &mut DatabaseChanges,
    shard: &IndexerShard,
    header: &BlockHeader,
    author: &str,
) {
    // First pass: collect receipt IDs from filtered receipt actions
    let mut valid_receipt_ids: HashSet<String> = HashSet::new();
    
    for receipt_exec_outcome in &shard.receipt_execution_outcomes {
        if let Some(receipt) = &receipt_exec_outcome.receipt {
            let receipt_id = if let Some(id) = &receipt.receipt_id { 
                hex::encode(&id.bytes) 
            } else { 
                "".to_string() 
            };
            
            // Check if this receipt action would be processed (using same logic as receipt_actions processor)
            if let Some(receipt::Receipt::Action(action_receipt)) = &receipt.receipt {
                if should_process_receipt_action(receipt) {
                    valid_receipt_ids.insert(receipt_id.clone());
                    process_receipt_actions(changes, action_receipt, receipt, header, shard, &receipt_id, author);
                }
            }
        }
    }

    // Second pass: process execution outcomes only for valid receipt IDs
    for receipt_exec_outcome in &shard.receipt_execution_outcomes {
        if let Some(receipt) = &receipt_exec_outcome.receipt {
            let receipt_id = if let Some(id) = &receipt.receipt_id { 
                hex::encode(&id.bytes) 
            } else { 
                "".to_string() 
            };
            
            // Only process execution outcome if the receipt ID is in our valid set
            if valid_receipt_ids.contains(&receipt_id) {
                if let Some(execution_outcome) = &receipt_exec_outcome.execution_outcome {
                    process_execution_outcome(changes, execution_outcome, header, shard, &receipt_id);
                }
            }
        }
    }
}

// Helper function to check if a receipt should be processed (same logic as in receipt_actions processor)
fn should_process_receipt_action(receipt: &crate::pb::sf::near::r#type::v1::Receipt) -> bool {
    use crate::config::Settings;

    let settings = Settings::new().expect("Failed to load config");

    // Helper function to check if an id matches our criteria
    let is_valid_id = |id: &str| -> bool {
        settings.venear_contract_ids.iter().any(|contract_id| {
            id.ends_with(&format!("v.{}", contract_id)) // id ends with v.contract_id
            || id == format!("vote.{}", contract_id)       // id is vote.contract_id
        })
    };

    // Return true if either receiver_id or predecessor_id matches our criteria
    is_valid_id(&receipt.receiver_id) || is_valid_id(&receipt.predecessor_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pb::sf::near::r#type::v1::{CryptoHash, ReceiptAction, Action, FunctionCallAction, ExecutionOutcomeWithId, ExecutionOutcome, execution_outcome::Status};

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

    #[test]
    fn test_process_shard_empty() {
        let mut changes = DatabaseChanges::default();
        let header = create_test_header();
        
        let shard = IndexerShard {
            shard_id: 0,
            chunk: None,
            receipt_execution_outcomes: vec![],
        };

        process_shard(&mut changes, &shard, &header, "test_author");
        
        // Should not create any changes for empty shard
        assert_eq!(changes.table_changes.len(), 0);
    }

    #[test]
    fn test_process_shard_with_valid_receipt() {
        let mut changes = DatabaseChanges::default();
        let header = create_test_header();
        
        // Get contract IDs from config to create a valid receipt
        let settings = crate::config::Settings::new().expect("Failed to load config");
        let contract_id = &settings.venear_contract_ids[0];
        
        let action_receipt = ReceiptAction {
            signer_id: "test.signer".to_string(),
            signer_public_key: None,
            gas_price: None,
            output_data_receivers: vec![],
            input_data_ids: vec![],
            actions: vec![
                Action {
                    action: Some(crate::pb::sf::near::r#type::v1::action::Action::FunctionCall(FunctionCallAction {
                        method_name: "test_method".to_string(),
                        args: vec![1, 2, 3, 4],
                        gas: 1000,
                        deposit: Some(crate::pb::sf::near::r#type::v1::BigInt { bytes: vec![100, 0, 0, 0] }),
                    })),
                }
            ],
        };

        let receipt = crate::pb::sf::near::r#type::v1::Receipt {
            predecessor_id: "test.predecessor".to_string(),
            receiver_id: format!("v.{}", contract_id),
            receipt_id: Some(CryptoHash { bytes: vec![1, 2, 3, 4] }),
            receipt: Some(crate::pb::sf::near::r#type::v1::receipt::Receipt::Action(action_receipt)),
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

        let shard = IndexerShard {
            shard_id: 0,
            chunk: None,
            receipt_execution_outcomes: vec![
                crate::pb::sf::near::r#type::v1::IndexerExecutionOutcomeWithReceipt {
                    execution_outcome: Some(execution_outcome),
                    receipt: Some(receipt),
                }
            ],
        };

        process_shard(&mut changes, &shard, &header, "test_author");
        
        // Should create table changes for both receipt actions and execution outcomes
        assert!(changes.table_changes.len() > 0);
        
        // Verify we have both receipt_actions and execution_outcomes tables
        let has_receipt_actions = changes.table_changes.iter()
            .any(|change| change.table == "receipt_actions");
        let has_execution_outcomes = changes.table_changes.iter()
            .any(|change| change.table == "execution_outcomes");
        
        assert!(has_receipt_actions);
        assert!(has_execution_outcomes);
    }

    #[test]
    fn test_process_shard_with_invalid_receipt() {
        let mut changes = DatabaseChanges::default();
        let header = create_test_header();
        
        let action_receipt = ReceiptAction {
            signer_id: "test.signer".to_string(),
            signer_public_key: None,
            gas_price: None,
            output_data_receivers: vec![],
            input_data_ids: vec![],
            actions: vec![
                Action {
                    action: Some(crate::pb::sf::near::r#type::v1::action::Action::FunctionCall(FunctionCallAction {
                        method_name: "test_method".to_string(),
                        args: vec![1, 2, 3, 4],
                        gas: 1000,
                        deposit: Some(crate::pb::sf::near::r#type::v1::BigInt { bytes: vec![100, 0, 0, 0] }),
                    })),
                }
            ],
        };

        let receipt = crate::pb::sf::near::r#type::v1::Receipt {
            predecessor_id: "test.predecessor".to_string(),
            receiver_id: "invalid.contract".to_string(), // Not in our contract list
            receipt_id: Some(CryptoHash { bytes: vec![1, 2, 3, 4] }),
            receipt: Some(crate::pb::sf::near::r#type::v1::receipt::Receipt::Action(action_receipt)),
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

        let shard = IndexerShard {
            shard_id: 0,
            chunk: None,
            receipt_execution_outcomes: vec![
                crate::pb::sf::near::r#type::v1::IndexerExecutionOutcomeWithReceipt {
                    execution_outcome: Some(execution_outcome),
                    receipt: Some(receipt),
                }
            ],
        };

        process_shard(&mut changes, &shard, &header, "test_author");
        
        // Should not create any changes for invalid receipt
        assert_eq!(changes.table_changes.len(), 0);
    }

    #[test]
    fn test_should_process_receipt_action_valid_receiver() {
        let settings = crate::config::Settings::new().expect("Failed to load config");
        let contract_id = &settings.venear_contract_ids[0];
        
        let receipt = crate::pb::sf::near::r#type::v1::Receipt {
            predecessor_id: "test.predecessor".to_string(),
            receiver_id: format!("v.{}", contract_id),
            receipt_id: Some(CryptoHash { bytes: vec![1, 2, 3, 4] }),
            receipt: Some(crate::pb::sf::near::r#type::v1::receipt::Receipt::Action(ReceiptAction {
                signer_id: "test.signer".to_string(),
                signer_public_key: None,
                gas_price: None,
                output_data_receivers: vec![],
                input_data_ids: vec![],
                actions: vec![],
            })),
        };

        assert!(should_process_receipt_action(&receipt));
    }

    #[test]
    fn test_should_process_receipt_action_valid_predecessor() {
        let settings = crate::config::Settings::new().expect("Failed to load config");
        let contract_id = &settings.venear_contract_ids[0];
        
        let receipt = crate::pb::sf::near::r#type::v1::Receipt {
            predecessor_id: format!("vote.{}", contract_id),
            receiver_id: "test.receiver".to_string(),
            receipt_id: Some(CryptoHash { bytes: vec![1, 2, 3, 4] }),
            receipt: Some(crate::pb::sf::near::r#type::v1::receipt::Receipt::Action(ReceiptAction {
                signer_id: "test.signer".to_string(),
                signer_public_key: None,
                gas_price: None,
                output_data_receivers: vec![],
                input_data_ids: vec![],
                actions: vec![],
            })),
        };

        assert!(should_process_receipt_action(&receipt));
    }

    #[test]
    fn test_should_process_receipt_action_invalid() {
        let receipt = crate::pb::sf::near::r#type::v1::Receipt {
            predecessor_id: "test.predecessor".to_string(),
            receiver_id: "test.receiver".to_string(),
            receipt_id: Some(CryptoHash { bytes: vec![1, 2, 3, 4] }),
            receipt: Some(crate::pb::sf::near::r#type::v1::receipt::Receipt::Action(ReceiptAction {
                signer_id: "test.signer".to_string(),
                signer_public_key: None,
                gas_price: None,
                output_data_receivers: vec![],
                input_data_ids: vec![],
                actions: vec![],
            })),
        };

        assert!(!should_process_receipt_action(&receipt));
    }
}
