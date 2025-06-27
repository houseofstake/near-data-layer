use substreams_database_change::pb::database::DatabaseChanges;
use crate::pb::sf::near::r#type::v1::{ExecutionOutcomeWithId, BlockHeader, IndexerShard, execution_outcome};
use crate::pb::near::entities::v1::ExecutionOutcome as ExecutionOutcomeEntity;

use crate::pushers::push_create_execution_outcome;
// use crate::processors::process_execution_outcome_result;
use crate::processors::utils::{bytes_to_string, format_timestamp};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

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

        // Clean, fault-tolerant serialization for all statuses
        let (result_value, result_json) = if let Some(status) = &outcome.status {
            match status {
                execution_outcome::Status::SuccessValue(inner) => {
                    let result_value = BASE64.encode(&inner.value);
                    let result_json = decode_and_format_result(&result_value);
                    (result_value, result_json)
                },
                execution_outcome::Status::Failure(inner) => {
                    // For failures, encode the error information
                    let error_bytes = format!("{:?}", inner).as_bytes().to_vec();
                    let result_value = BASE64.encode(&error_bytes);
                    let result_json = format!("{{\"error_type\": \"ExecutionFailure\", \"details\": \"{:?}\"}}", inner);
                    (result_value, result_json)
                },
                execution_outcome::Status::SuccessReceiptId(inner) => {
                    // For receipt IDs, encode the receipt ID bytes
                    let receipt_id_bytes = if let Some(id) = &inner.id {
                        id.bytes.clone()
                    } else {
                        vec![]
                    };
                    let result_value = BASE64.encode(&receipt_id_bytes);
                    let result_json = format!("{{\"receipt_id\": \"{}\", \"status\": \"SuccessReceiptId\"}}", 
                        if let Some(id) = &inner.id { hex::encode(&id.bytes) } else { "".to_string() });
                    (result_value, result_json)
                },
                execution_outcome::Status::Unknown(inner) => {
                    // For unknown status, encode the debug information
                    let unknown_bytes = format!("{:?}", inner).as_bytes().to_vec();
                    let result_value = BASE64.encode(&unknown_bytes);
                    let result_json = format!("{{\"status\": \"Unknown\", \"details\": \"{:?}\"}}", inner);
                    (result_value, result_json)
                },
            }
        } else {
            ("".to_string(), "".to_string())
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
            result_value,
            result_json,
            block_timestamp: format_timestamp(header.timestamp_nanosec),
        };

        let key = format!("{}-{}", header.height, receipt_id);
        push_create_execution_outcome(changes, &key, 0, &execution_outcome_entity);
    }
}

// Helper function to decode and format result data (DRY approach)
fn decode_and_format_result(result_value: &str) -> String {
    match BASE64.decode(result_value) {
        Ok(decoded_bytes) => {
            // Try to parse as UTF-8 string first
            match String::from_utf8(decoded_bytes.clone()) {
                Ok(utf8_string) => {
                    // Try to parse as JSON
                    match serde_json::from_str::<serde_json::Value>(&utf8_string) {
                        Ok(json_value) => serde_json::to_string(&json_value).unwrap_or(utf8_string),
                        Err(_) => {
                            // If not valid JSON, try to format as a readable string
                            format!("{:?}", utf8_string)
                        }
                    }
                }
                Err(_) => {
                    // If not valid UTF-8, format as hex
                    format!("0x{}", hex::encode(&decoded_bytes))
                }
            }
        }
        Err(_) => {
            // If base64 decode fails, store as hex
            format!("0x{}", result_value)
        }
    }
} 