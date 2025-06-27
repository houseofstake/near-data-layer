use substreams_database_change::pb::database::DatabaseChanges;
use crate::pb::sf::near::r#type::v1::{ExecutionOutcomeWithId, BlockHeader, IndexerShard, execution_outcome};
use crate::pb::near::entities::v1::ExecutionOutcomeResult as ExecutionOutcomeResultEntity;
use crate::pushers::push_create_execution_outcome_result;
use crate::processors::utils::format_timestamp;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

pub fn process_execution_outcome_result(
    changes: &mut DatabaseChanges,
    execution_outcome: &ExecutionOutcomeWithId,
    header: &BlockHeader,
    shard: &IndexerShard,
    receipt_id: &str,
) {
    if let Some(outcome) = &execution_outcome.outcome {
        // Only process successful execution outcomes that have return values
        if let Some(execution_outcome::Status::SuccessValue(value)) = &outcome.status {
            let result_value = BASE64.encode(&value.value);
            
            // Try to decode the result value and convert to JSON if possible
            let result_json = match BASE64.decode(&result_value) {
                Ok(decoded_bytes) => {
                    // Try to parse as UTF-8 string first
                    match String::from_utf8(decoded_bytes.clone()) {
                        Ok(utf8_string) => {
                            // Try to parse as JSON
                            match serde_json::from_str::<serde_json::Value>(&utf8_string) {
                                Ok(json_value) => serde_json::to_string(&json_value).ok(),
                                Err(_) => {
                                    // If not valid JSON, try to format as a readable string
                                    Some(format!("{:?}", utf8_string))
                                }
                            }
                        }
                        Err(_) => {
                            // If not valid UTF-8, format as hex
                            Some(format!("0x{}", hex::encode(&decoded_bytes)))
                        }
                    }
                }
                Err(_) => {
                    // If base64 decode fails, store as hex
                    Some(format!("0x{}", result_value))
                }
            };

            let execution_outcome_result_entity = ExecutionOutcomeResultEntity {
                receipt_id: receipt_id.to_string(),
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
                status: "SuccessValue".to_string(),
                result_value,
                result_json: result_json.unwrap_or_default(),
                block_timestamp: format_timestamp(header.timestamp_nanosec),
            };

            let key = format!("{}-{}", header.height, receipt_id);
            push_create_execution_outcome_result(changes, &key, 0, &execution_outcome_result_entity);
        }
    }
} 