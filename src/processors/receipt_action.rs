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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pb::sf::near::r#type::v1::{CryptoHash, FunctionCallAction};

    fn create_test_header() -> BlockHeader {
        BlockHeader {
            height: 12345,
            prev_height: 12344,
            epoch_id: Some(CryptoHash { bytes: vec![1, 2, 3, 4] }),
            next_epoch_id: Some(CryptoHash { bytes: vec![5, 6, 7, 8] }),
            hash: Some(CryptoHash { bytes: vec![9, 10, 11, 12] }),
            prev_hash: Some(CryptoHash { bytes: vec![13, 14, 15, 16] }),
            prev_state_root: Some(CryptoHash { bytes: vec![17, 18, 19, 20] }),
            chunk_receipts_root: Some(CryptoHash { bytes: vec![21, 22, 23, 24] }),
            chunk_headers_root: Some(CryptoHash { bytes: vec![25, 26, 27, 28] }),
            chunk_tx_root: Some(CryptoHash { bytes: vec![29, 30, 31, 32] }),
            outcome_root: Some(CryptoHash { bytes: vec![33, 34, 35, 36] }),
            chunks_included: 1,
            challenges_root: Some(CryptoHash { bytes: vec![37, 38, 39, 40] }),
            timestamp: 1640995200,
            timestamp_nanosec: 1640995200000000000,
            random_value: Some(CryptoHash { bytes: vec![41, 42, 43, 44] }),
            validator_proposals: vec![],
            chunk_mask: vec![true],
            gas_price: Some(crate::pb::sf::near::r#type::v1::BigInt { bytes: vec![100] }),
            block_ordinal: 1,
            total_supply: Some(crate::pb::sf::near::r#type::v1::BigInt { bytes: vec![232, 3, 0, 0] }),
            challenges_result: vec![],
            last_final_block_height: 12340,
            last_final_block: Some(CryptoHash { bytes: vec![45, 46, 47, 48] }),
            last_ds_final_block_height: 12340,
            last_ds_final_block: Some(CryptoHash { bytes: vec![49, 50, 51, 52] }),
            next_bp_hash: Some(CryptoHash { bytes: vec![53, 54, 55, 56] }),
            block_merkle_root: Some(CryptoHash { bytes: vec![57, 58, 59, 60] }),
            epoch_sync_data_hash: vec![61, 62, 63, 64],
            approvals: vec![],
            signature: Some(crate::pb::sf::near::r#type::v1::Signature { 
                r#type: 0, 
                bytes: vec![65, 66, 67, 68] 
            }),
            latest_protocol_version: 1,
        }
    }

    fn create_test_shard() -> IndexerShard {
        IndexerShard {
            shard_id: 0,
            chunk: None,
            receipt_execution_outcomes: vec![],
        }
    }

    #[test]
    fn test_process_receipt_actions_function_call() {
        let mut changes = DatabaseChanges::default();
        let header = create_test_header();
        let shard = create_test_shard();

        let action_receipt = ReceiptAction {
            signer_id: "test.signer".to_string(),
            signer_public_key: Some(crate::pb::sf::near::r#type::v1::PublicKey {
                r#type: 0,
                bytes: vec![1, 2, 3, 4],
            }),
            gas_price: Some(crate::pb::sf::near::r#type::v1::BigInt { bytes: vec![100] }),
            output_data_receivers: vec![],
            input_data_ids: vec![],
            actions: vec![
                Action {
                    action: Some(action::Action::FunctionCall(FunctionCallAction {
                        method_name: "test_method".to_string(),
                        args: vec![1, 2, 3, 4],
                        gas: 1000,
                        deposit: Some(crate::pb::sf::near::r#type::v1::BigInt { bytes: vec![100, 0, 0, 0] }),
                    })),
                }
            ],
        };

        let receipt = Receipt {
            predecessor_id: "test.predecessor".to_string(),
            receiver_id: "test.receiver".to_string(),
            receipt_id: Some(CryptoHash { bytes: vec![1, 2, 3, 4] }),
            receipt: Some(crate::pb::sf::near::r#type::v1::receipt::Receipt::Action(action_receipt.clone())),
        };

        process_receipt_actions(&mut changes, &action_receipt, &receipt, &header, &shard, "test-receipt-id", "test_author");
        
        // Should create at least one table change for FunctionCall action
        assert!(changes.table_changes.len() > 0);
        
        // Verify the table change is for the receipt_actions table
        let table_change = &changes.table_changes[0];
        assert_eq!(table_change.table, "receipt_actions");
    }

    #[test]
    fn test_process_receipt_actions_transfer_only() {
        let mut changes = DatabaseChanges::default();
        let header = create_test_header();
        let shard = create_test_shard();

        let action_receipt = ReceiptAction {
            signer_id: "test.signer".to_string(),
            signer_public_key: None,
            gas_price: None,
            output_data_receivers: vec![],
            input_data_ids: vec![],
            actions: vec![
                Action {
                    action: Some(action::Action::Transfer(crate::pb::sf::near::r#type::v1::TransferAction {
                        deposit: Some(crate::pb::sf::near::r#type::v1::BigInt { bytes: vec![100, 0, 0, 0] }),
                    })),
                }
            ],
        };

        let receipt = Receipt {
            predecessor_id: "test.predecessor".to_string(),
            receiver_id: "test.receiver".to_string(),
            receipt_id: Some(CryptoHash { bytes: vec![1, 2, 3, 4] }),
            receipt: Some(crate::pb::sf::near::r#type::v1::receipt::Receipt::Action(action_receipt.clone())),
        };

        process_receipt_actions(&mut changes, &action_receipt, &receipt, &header, &shard, "test-receipt-id", "test_author");
        
        // Should not create any changes for non-FunctionCall actions
        assert_eq!(changes.table_changes.len(), 0);
    }

    #[test]
    fn test_process_receipt_actions_mixed_actions() {
        let mut changes = DatabaseChanges::default();
        let header = create_test_header();
        let shard = create_test_shard();

        let action_receipt = ReceiptAction {
            signer_id: "test.signer".to_string(),
            signer_public_key: None,
            gas_price: None,
            output_data_receivers: vec![],
            input_data_ids: vec![],
            actions: vec![
                Action {
                    action: Some(action::Action::Transfer(crate::pb::sf::near::r#type::v1::TransferAction {
                        deposit: Some(crate::pb::sf::near::r#type::v1::BigInt { bytes: vec![100, 0, 0, 0] }),
                    })),
                },
                Action {
                    action: Some(action::Action::FunctionCall(FunctionCallAction {
                        method_name: "test_method".to_string(),
                        args: vec![1, 2, 3, 4],
                        gas: 1000,
                        deposit: Some(crate::pb::sf::near::r#type::v1::BigInt { bytes: vec![200, 0, 0, 0] }),
                    })),
                },
                Action {
                    action: Some(action::Action::CreateAccount(crate::pb::sf::near::r#type::v1::CreateAccountAction {})),
                }
            ],
        };

        let receipt = Receipt {
            predecessor_id: "test.predecessor".to_string(),
            receiver_id: "test.receiver".to_string(),
            receipt_id: Some(CryptoHash { bytes: vec![1, 2, 3, 4] }),
            receipt: Some(crate::pb::sf::near::r#type::v1::receipt::Receipt::Action(action_receipt.clone())),
        };

        process_receipt_actions(&mut changes, &action_receipt, &receipt, &header, &shard, "test-receipt-id", "test_author");
        
        // Should create exactly one table change for the FunctionCall action only
        assert_eq!(changes.table_changes.len(), 1);
        
        // Verify the table change is for the receipt_actions table
        let table_change = &changes.table_changes[0];
        assert_eq!(table_change.table, "receipt_actions");
    }

    #[test]
    fn test_process_action_function_call() {
        let action = Action {
            action: Some(action::Action::FunctionCall(FunctionCallAction {
                method_name: "test_method".to_string(),
                args: vec![1, 2, 3, 4],
                gas: 1000,
                deposit: Some(crate::pb::sf::near::r#type::v1::BigInt { bytes: vec![100, 0, 0, 0] }),
            })),
        };

        let (action_kind, method_name, gas, deposit, args_base64) = process_action(&action);
        
        assert_eq!(action_kind, "FunctionCall");
        assert_eq!(method_name, "test_method");
        assert_eq!(gas, 1000);
        assert_eq!(deposit, "1677721600"); // bytes_to_string([100, 0, 0, 0]) = "1677721600" (big-endian)
        assert_eq!(args_base64, "AQIDBA=="); // base64 of [1, 2, 3, 4]
    }

    #[test]
    fn test_process_action_transfer() {
        let action = Action {
            action: Some(action::Action::Transfer(crate::pb::sf::near::r#type::v1::TransferAction {
                deposit: Some(crate::pb::sf::near::r#type::v1::BigInt { bytes: vec![200, 0, 0, 0] }),
            })),
        };

        let (action_kind, method_name, gas, deposit, args_base64) = process_action(&action);
        
        assert_eq!(action_kind, "Transfer");
        assert_eq!(method_name, "");
        assert_eq!(gas, 0);
        assert_eq!(deposit, "3355443200"); // bytes_to_string([200, 0, 0, 0]) = "3355443200" (big-endian)
        assert_eq!(args_base64, "");
    }

    #[test]
    fn test_process_action_create_account() {
        let action = Action {
            action: Some(action::Action::CreateAccount(crate::pb::sf::near::r#type::v1::CreateAccountAction {})),
        };

        let (action_kind, method_name, gas, deposit, args_base64) = process_action(&action);
        
        assert_eq!(action_kind, "CreateAccount");
        assert_eq!(method_name, "");
        assert_eq!(gas, 0);
        assert_eq!(deposit, "0");
        assert_eq!(args_base64, "");
    }
}
