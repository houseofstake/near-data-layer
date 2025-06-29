use substreams_database_change::pb::database::DatabaseChanges;
use crate::pb::sf::near::r#type::v1::{ExecutionOutcomeWithId, BlockHeader, IndexerShard, execution_outcome};
use crate::pb::near::entities::v1::ExecutionOutcome as ExecutionOutcomeEntity;

use crate::pushers::push_create_execution_outcome;
// use crate::processors::process_execution_outcome_result;
use crate::processors::utils::{bytes_to_string, format_timestamp};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::{json, Value};

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
                    // For failures, create a structured JSON with error details
                    let error_info = json!({
                        "status": "Failure",
                        "error_type": "ExecutionFailure",
                        "error_details": format!("{:?}", inner.failure),
                        "gas_burnt": outcome.gas_burnt,
                        "tokens_burnt": if let Some(tb) = &outcome.tokens_burnt {
                            bytes_to_string(&tb.bytes)
                        } else {
                            "0".to_string()
                        }
                    });
                    let result_value = BASE64.encode(&serde_json::to_vec(&error_info).unwrap_or_default());
                    let result_json = serde_json::to_string_pretty(&error_info).unwrap_or_default();
                    (result_value, result_json)
                },
                execution_outcome::Status::SuccessReceiptId(inner) => {
                    // For receipt IDs, create a structured JSON
                    let receipt_info = json!({
                        "status": "SuccessReceiptId",
                        "receipt_id": if let Some(id) = &inner.id { 
                            hex::encode(&id.bytes) 
                        } else { 
                            "".to_string() 
                        },
                        "gas_burnt": outcome.gas_burnt,
                        "tokens_burnt": if let Some(tb) = &outcome.tokens_burnt {
                            bytes_to_string(&tb.bytes)
                        } else {
                            "0".to_string()
                        }
                    });
                    let result_value = BASE64.encode(&serde_json::to_vec(&receipt_info).unwrap_or_default());
                    let result_json = serde_json::to_string_pretty(&receipt_info).unwrap_or_default();
                    (result_value, result_json)
                },
                execution_outcome::Status::Unknown(inner) => {
                    // For unknown status, create a structured JSON
                    let unknown_info = json!({
                        "status": "Unknown",
                        "raw_data": format!("{:?}", inner),
                        "gas_burnt": outcome.gas_burnt,
                        "tokens_burnt": if let Some(tb) = &outcome.tokens_burnt {
                            bytes_to_string(&tb.bytes)
                        } else {
                            "0".to_string()
                        }
                    });
                    let result_value = BASE64.encode(&serde_json::to_vec(&unknown_info).unwrap_or_default());
                    let result_json = serde_json::to_string_pretty(&unknown_info).unwrap_or_default();
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
            results_base64: result_value,
            results_json: result_json,
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
                    // Try to parse as JSON and format it nicely
                    match serde_json::from_str::<Value>(&utf8_string) {
                        Ok(json_value) => {
                            // Return pretty-printed JSON
                            serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| {
                                // Fallback to compact JSON if pretty printing fails
                                serde_json::to_string(&json_value).unwrap_or(utf8_string)
                            })
                        },
                        Err(_) => {
                            // If not valid JSON, try to create a structured response
                            if utf8_string.trim().is_empty() {
                                json!({
                                    "status": "SuccessValue",
                                    "value": "",
                                    "type": "empty_string"
                                }).to_string()
                            } else {
                                // Try to determine if it's a number, boolean, or string
                                if let Ok(num) = utf8_string.parse::<f64>() {
                                    json!({
                                        "status": "SuccessValue",
                                        "value": num,
                                        "type": "number"
                                    }).to_string()
                                } else if let Ok(boolean) = utf8_string.parse::<bool>() {
                                    json!({
                                        "status": "SuccessValue",
                                        "value": boolean,
                                        "type": "boolean"
                                    }).to_string()
                                } else {
                                    json!({
                                        "status": "SuccessValue",
                                        "value": utf8_string,
                                        "type": "string"
                                    }).to_string()
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    // If not valid UTF-8, create a structured response with hex data
                    json!({
                        "status": "SuccessValue",
                        "value": format!("0x{}", hex::encode(&decoded_bytes)),
                        "type": "binary_data",
                        "encoding": "hex"
                    }).to_string()
                }
            }
        }
        Err(_) => {
            // If base64 decode fails, create a structured error response
            json!({
                "status": "SuccessValue",
                "error": "Failed to decode base64 data",
                "type": "decode_error"
            }).to_string()
        }
    }
} 
