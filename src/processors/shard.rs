use substreams_database_change::pb::database::DatabaseChanges;
use crate::pb::sf::near::r#type::v1::{IndexerShard, BlockHeader, receipt};
use std::collections::HashSet;

use crate::processors::{process_receipt,process_receipt_actions, process_execution_outcome};

pub fn process_shard(
    changes: &mut DatabaseChanges,
    shard: &IndexerShard,
    header: &BlockHeader,
    author: &str,
) {   
    // process data for relevant receipt IDs
    for receipt_exec_outcome in &shard.receipt_execution_outcomes {
        if let Some(receipt) = &receipt_exec_outcome.receipt {

            // We parse the receipt ID here so this doesn't have to be repeated in the downstream processors
            let receipt_id = if let Some(id) = &receipt.receipt_id { 
                hex::encode(&id.bytes) 
            } else { 
                "".to_string() 
            };

            if let Some(receipt::Receipt::Action(_)) = &receipt.receipt {

                // Check if this receipt should be processed based on predecessor_id and receiver_id
                if should_process_receipt(receipt) {

                    // Process receipt
                    process_receipt(changes, receipt, header, shard, &receipt_id, author);

                    // Process receipt actions
                    if let Some(receipt::Receipt::Action(action_receipt)) = &receipt.receipt {
                        process_receipt_actions(changes, action_receipt, receipt, header, shard, &receipt_id, author);
                    }

                    // Process execution outcomes
                    if let Some(execution_outcome) = &receipt_exec_outcome.execution_outcome {
                        process_execution_outcome(changes, execution_outcome, header, shard, &receipt_id);                   
                    }
                }
            }
        }
    }
}

// Check if a receipt should be processed based on the top-level receipt predecessor_id and receiver_id
// These fields identify the receipt itself (not the nested action data)
fn should_process_receipt(receipt: &crate::pb::sf::near::r#type::v1::Receipt) -> bool {
    use crate::config::Settings;

    let settings = Settings::new().expect("Failed to load config");

    // Helper function to check if an id matches our House of Stake criteria
    let is_valid_id = |id: &str| -> bool {
        settings.venear_contract_ids.iter().any(|contract_id| {
            id.ends_with(&format!("v.{}", contract_id)) // id ends with v.contract_id
            || id == format!("vote.{}", contract_id)       // id is vote.contract_id
        })
    };

    // Filter based on top-level receipt fields (available for both Action and Data receipts)
    // predecessor_id: who sent this receipt
    // receiver_id: who is receiving this receipt
    is_valid_id(&receipt.receiver_id) || is_valid_id(&receipt.predecessor_id)
}
