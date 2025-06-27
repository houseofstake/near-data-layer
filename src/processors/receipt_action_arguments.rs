use substreams_database_change::pb::database::DatabaseChanges;
use crate::pb::sf::near::r#type::v1::{ReceiptAction, BlockHeader, IndexerShard};
use crate::pb::near::entities::v1::ReceiptActionArguments as ReceiptActionArgumentsEntity;
use crate::pushers::push_create_receipt_action_arguments;
use crate::processors::utils::format_timestamp;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

pub fn process_receipt_action_arguments(
    changes: &mut DatabaseChanges,
    receipt_action: &ReceiptAction,
    header: &BlockHeader,
    shard: &IndexerShard,
    receipt_id: &str,
    action_index: u32,
) {
    // Only process FunctionCall actions
    if let Some(action) = &receipt_action.action {
        if let Some(function_call) = &action.function_call {
            let args_base64 = base64::encode(&function_call.args);
            
            // Try to decode the arguments and convert to JSON if possible
            let args_json = match BASE64.decode(&args_base64) {
                Ok(decoded_bytes) => {
                    match String::from_utf8(decoded_bytes.clone()) {
                        Ok(utf8_string) => {
                            match serde_json::from_str::<serde_json::Value>(&utf8_string) {
                                Ok(json_value) => serde_json::to_string(&json_value).ok(),
                                Err(_) => Some(format!("{:?}", utf8_string)),
                            }
                        }
                        Err(_) => Some(format!("0x{}", hex::encode(&decoded_bytes))),
                    }
                }
                Err(_) => Some(format!("0x{}", args_base64)),
            };

            let receipt_action_arguments_entity = ReceiptActionArgumentsEntity {
                id: format!("{}-{}", receipt_id, action_index),
                receipt_id: receipt_id.to_string(),
                action_index,
                block_height: header.height,
                block_hash: if let Some(h) = &header.hash { hex::encode(&h.bytes) } else { "".to_string() },
                chunk_hash: if let Some(chunk) = &shard.chunk {
                    if let Some(header) = &chunk.header {
                        hex::encode(&header.chunk_hash)
                    } else { "".to_string() }
                } else { "".to_string() },
                shard_id: shard.shard_id.to_string(),
                method_name: function_call.method_name.clone(),
                receiver_id: receipt_action.receiver_id.clone(),
                signer_account_id: receipt_action.predecessor_id.clone(),
                predecessor_id: receipt_action.predecessor_id.clone(),
                args_base64,
                args_json: args_json.unwrap_or_default(),
                gas: function_call.gas,
                deposit: if let Some(deposit) = &action.deposit { bytes_to_string(&deposit.bytes) } else { "0".to_string() },
                block_timestamp: format_timestamp(header.timestamp_nanosec),
            };

            let key = format!("{}-{}", receipt_id, action_index);
            push_create_receipt_action_arguments(changes, &key, 0, &receipt_action_arguments_entity);
        }
    }
}

fn bytes_to_string(bytes: &[u8]) -> String {
    // Convert bytes to string representation
    if bytes.is_empty() {
        "0".to_string()
    } else {
        // Try to convert to UTF-8 first, fallback to hex
        String::from_utf8(bytes.to_vec())
            .unwrap_or_else(|_| format!("0x{}", hex::encode(bytes)))
    }
} 