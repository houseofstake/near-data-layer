use substreams_database_change::pb::database::DatabaseChanges;
use crate::pb::sf::near::r#type::v1::{ReceiptAction, Receipt, BlockHeader, IndexerShard, Action, action};
use crate::pb::near::entities::v1::ReceiptAction as ReceiptActionEntity;

use crate::pushers::push_create_receipt_action;
use crate::processors::utils::{bytes_to_string, format_timestamp};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

pub fn process_receipt_actions(
    changes: &mut DatabaseChanges,
    action_receipt: &ReceiptAction,
    receipt: &Receipt,
    header: &BlockHeader,
    shard: &IndexerShard,
    receipt_id: &str,
    author: &str,
) {
    // Note: Filtering is now done in the shard processor to enable cross-referencing 
    // between receipt actions and execution outcomes

    let signer_account_id = action_receipt.signer_id.clone();
    let signer_public_key = if let Some(pk) = &action_receipt.signer_public_key {
        format!("{:?}:{}", pk.r#type, hex::encode(&pk.bytes))
    } else {
        "".to_string()
    };
    
    let gas_price = if let Some(gp) = &action_receipt.gas_price {
        bytes_to_string(&gp.bytes)
    } else {
        "0".to_string()
    };
    
    // We will only process FunctionCall actions
    for (action_index, action) in action_receipt.actions.iter()
        .enumerate()
        .filter(|(_, action)| matches!(&action.action, Some(action::Action::FunctionCall(_)))) 
    {
        let (action_kind, method_name, gas, deposit, args_base64) = process_action(action);
        
        // Try to decode the args and convert to JSON if possible
        let args_json = match BASE64.decode(&args_base64) {
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
                Some(format!("0x{}", args_base64))
            }
        };
        
        let unique_id = format!("{}-{}-{}", header.height, receipt_id, action_index);
        
        let receipt_action = ReceiptActionEntity {
            id: unique_id.clone(),
            block_height: header.height,
            receipt_id: receipt_id.to_string(),
            signer_account_id: signer_account_id.clone(),
            signer_public_key: signer_public_key.clone(),
            gas_price: gas_price.clone(),
            action_kind,
            predecessor_id: receipt.predecessor_id.clone(),
            receiver_id: receipt.receiver_id.clone(),
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
            author: author.to_string(),
            method_name,
            gas,
            deposit,
            args_base64,
            args_json: args_json.unwrap_or_default(),
            action_index: action_index as u32,
            block_timestamp: format_timestamp(header.timestamp_nanosec),
        };

        push_create_receipt_action(changes, &unique_id, 0, &receipt_action);
    }
}

fn process_action(action: &Action) -> (String, String, u64, String, String) {
    let mut action_kind = "Unknown".to_string();
    let mut method_name = "".to_string();
    let mut gas = 0u64;
    let mut deposit = "0".to_string();
    let mut args_base64 = "".to_string();
    
    match &action.action {
        Some(action::Action::CreateAccount(_)) => {
            action_kind = "CreateAccount".to_string();
        }
        Some(action::Action::DeployContract(_)) => {
            action_kind = "DeployContract".to_string();
        }
        Some(action::Action::FunctionCall(func_call)) => {
            action_kind = "FunctionCall".to_string();
            method_name = func_call.method_name.clone();
            gas = func_call.gas;
            if let Some(dep) = &func_call.deposit {
                deposit = bytes_to_string(&dep.bytes);
            }
            args_base64 = BASE64.encode(&func_call.args);
        }
        Some(action::Action::Transfer(transfer)) => {
            action_kind = "Transfer".to_string();
            if let Some(dep) = &transfer.deposit {
                deposit = bytes_to_string(&dep.bytes);
            }
        }
        Some(action::Action::Stake(_)) => {
            action_kind = "Stake".to_string();
        }
        Some(action::Action::AddKey(_)) => {
            action_kind = "AddKey".to_string();
        }
        Some(action::Action::DeleteKey(_)) => {
            action_kind = "DeleteKey".to_string();
        }
        Some(action::Action::DeleteAccount(_)) => {
            action_kind = "DeleteAccount".to_string();
        }
        Some(action::Action::Delegate(_)) => {
            action_kind = "Delegate".to_string();
        }
        Some(action::Action::DeployGlobalContract(_)) => {
            action_kind = "DeployGlobalContract".to_string();
        }
        Some(action::Action::DeployGlobalContractByAccountId(_)) => {
            action_kind = "DeployGlobalContractByAccountId".to_string();
        }
        Some(action::Action::UseGlobalContract(_)) => {
            action_kind = "UseGlobalContract".to_string();
        }
        Some(action::Action::UseGlobalContractByAccountId(_)) => {
            action_kind = "UseGlobalContractByAccountId".to_string();
        }
        None => {}
    }
    
    (action_kind, method_name, gas, deposit, args_base64)
}
