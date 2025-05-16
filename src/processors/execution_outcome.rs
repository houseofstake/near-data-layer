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