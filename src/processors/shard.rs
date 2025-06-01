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
