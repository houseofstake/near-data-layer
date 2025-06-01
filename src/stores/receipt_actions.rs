use substreams::store::{StoreNew, StoreSet, StoreDelete, StoreSetProto};
use crate::pb::sf::near::r#type::v1::{Block, Action, action, receipt};
use crate::pb::near::entities::v1::ReceiptAction as ReceiptActionEntity;
use crate::config::Settings;
use crate::processors::utils::{bytes_to_string, format_timestamp};

#[substreams::handlers::store]
fn store_receipt_actions(block: Block, store: StoreSetProto<ReceiptActionEntity>) {
    let settings = Settings::new().expect("Failed to load config");

    // Helper function to check if an id matches our criteria
    let is_valid_id = |id: &str| -> bool {
        settings.venear_contract_ids.iter().any(|contract_id| {
            id.ends_with(&format!("v.{}", contract_id)) // id ends with v.contract_id
            || id == format!("vote.{}", contract_id)       // id is vote.contract_id
        })
    };

    if let Some(header) = &block.header {
        let current_height = header.height;
        
        for shard in &block.shards {
            if let Some(chunk) = &shard.chunk {
                for (i, receipt) in chunk.receipts.iter().enumerate() {
                    // Skip if neither receiver_id nor predecessor_id matches our criteria
                    if !is_valid_id(&receipt.receiver_id) && !is_valid_id(&receipt.predecessor_id) {
                        continue;
                    }

                    if let Some(receipt::Receipt::Action(action_receipt)) = &receipt.receipt {
                        let receipt_id = if i < shard.receipt_execution_outcomes.len() {
                            if let Some(outcome) = &shard.receipt_execution_outcomes[i].execution_outcome {
                                if let Some(id) = &outcome.id {
                                    hex::encode(&id.bytes)
                                } else {
                                    format!("{}-{}", header.height, i)
                                }
                            } else {
                                format!("{}-{}", header.height, i)
                            }
                        } else {
                            format!("{}-{}", header.height, i)
                        };

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

                        // Process only FunctionCall actions
                        for (action_index, action) in action_receipt.actions.iter()
                            .enumerate()
                            .filter(|(_, action)| matches!(&action.action, Some(action::Action::FunctionCall(_)))) 
                        {
                            let (action_kind, method_name, gas, deposit, args_base64) = process_action(action);
                            
                            let unique_id = format!("{}-{}-{}", header.height, receipt_id, action_index);
                            
                            let receipt_action_entity = ReceiptActionEntity {
                                id: unique_id.clone(),
                                block_height: header.height,
                                receipt_id: receipt_id.clone(),
                                signer_account_id: signer_account_id.clone(),
                                signer_public_key: signer_public_key.clone(),
                                gas_price: gas_price.clone(),
                                action_kind,
                                predecessor_id: receipt.predecessor_id.clone(),
                                receiver_id: receipt.receiver_id.clone(),
                                block_hash: if let Some(h) = &header.hash { hex::encode(&h.bytes) } else { "".to_string() },
                                chunk_hash: if let Some(chunk_header) = &chunk.header {
                                    hex::encode(&chunk_header.chunk_hash)
                                } else {
                                    "".to_string()
                                },
                                author: block.author.clone(),
                                method_name,
                                gas,
                                deposit,
                                args_base64,
                                action_index: action_index as u32,
                                block_timestamp: format_timestamp(header.timestamp_nanosec),
                            };

                            let key = unique_id;
                            store.set(current_height, &key, &receipt_action_entity);
                        }
                    }
                }
            }
        }

        // Prune receipt actions older than 1,000 blocks
        if current_height > 1000 {
            let prune_height = current_height - 1000;
            store.delete_prefix(prune_height.try_into().unwrap(), &prune_height.to_string());
        }
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
            args_base64 = base64::encode(&func_call.args);
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