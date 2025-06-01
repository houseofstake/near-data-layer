use substreams::store::{StoreNew, StoreSet, StoreSetProto};
use crate::pb::sf::near::r#type::v1::Block;
use crate::pb::near::entities::v1::ExecutionOutcome as ExecutionOutcomeEntity;

#[substreams::handlers::store]
fn store_execution_outcomes(block: Block, store: StoreSetProto<ExecutionOutcomeEntity>) {
    if let Some(header) = &block.header {
        for shard in &block.shards {
            if let Some(chunk) = &shard.chunk {
                for outcome_with_receipt in &shard.receipt_execution_outcomes {
                    if let Some(execution_outcome) = &outcome_with_receipt.execution_outcome {
                        let receipt_id = if let Some(id) = &execution_outcome.id {
                            hex::encode(&id.bytes)
                        } else {
                            continue; // Skip if no id
                        };
                        
                        // Get the actual execution outcome data
                        if let Some(outcome_data) = &execution_outcome.outcome {
                            // For this example, we'll create an empty array since the field structure is complex
                            // You may need to adjust based on the actual NEAR protobuf structure
                            let outcome_receipt_ids: Vec<String> = Vec::new();

                            let execution_outcome_entity = ExecutionOutcomeEntity {
                                receipt_id: receipt_id.clone(),
                                block_height: header.height,
                                block_hash: if let Some(h) = &header.hash { hex::encode(&h.bytes) } else { "".to_string() },
                                chunk_hash: if let Some(chunk_header) = &chunk.header {
                                    hex::encode(&chunk_header.chunk_hash)
                                } else {
                                    "".to_string()
                                },
                                shard_id: shard.shard_id.to_string(),
                                gas_burnt: outcome_data.gas_burnt,
                                gas_used: 0.0, // You may need to calculate this if needed
                                tokens_burnt: 0.0, // You may need to calculate this if needed
                                executor_account_id: outcome_data.executor_id.clone(),
                                status: format!("{:?}", outcome_data.status), // Convert status enum to string
                                outcome_receipt_ids,
                                executed_in_block_hash: if let Some(h) = &header.hash { hex::encode(&h.bytes) } else { "".to_string() },
                                logs: outcome_data.logs.clone(),
                            };

                            store.set(0, &receipt_id, &execution_outcome_entity);
                        }
                    }
                }
            }
        }
    }
} 