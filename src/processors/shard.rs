use substreams_database_change::pb::database::DatabaseChanges;
use crate::pb::sf::near::r#type::v1::{IndexerShard, BlockHeader, receipt};

use crate::processors::{process_receipt, process_receipt_actions, process_execution_outcome};

pub fn process_shard(
    changes: &mut DatabaseChanges,
    shard: &IndexerShard,
    header: &BlockHeader,
    author: &str,
) {
    for receipt_exec_outcome in &shard.receipt_execution_outcomes {
        if let Some(receipt) = &receipt_exec_outcome.receipt {
            let receipt_id = if let Some(id) = &receipt.receipt_id { hex::encode(&id.bytes) } else { "".to_string() };
            
            // Process receipt
            process_receipt(changes, receipt, header, shard, &receipt_id, author);

            // Process receipt actions if it's an action receipt
            if let Some(receipt::Receipt::Action(action_receipt)) = &receipt.receipt {
                process_receipt_actions(changes, action_receipt, receipt, header, shard, &receipt_id, author);
            }

            // Process execution outcome
            if let Some(execution_outcome) = &receipt_exec_outcome.execution_outcome {
                process_execution_outcome(changes, execution_outcome, header, shard, &receipt_id);
            }
        }
    }
} 