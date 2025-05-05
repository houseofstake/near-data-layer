mod pb;

use substreams::store::{self, DeltaProto, StoreSetIfNotExistsProto, StoreNew, StoreSetIfNotExists};
use substreams_database_change::pb::database::{table_change::Operation, DatabaseChanges};
use pb::sf::near::r#type::v1::{Block, receipt};
use pb::near::entities::v1::{Block as BlockEntity, Chunk, Receipt, ReceiptAction};
use substreams::pb::substreams::store_delta::Operation as DeltaOperation;
use chrono::{DateTime, Utc};

/// move to utils
fn bytes_to_string(bytes: &[u8]) -> String {
    if !bytes.is_empty() {
        let mut value = 0u128;
        for &byte in bytes {
            value = (value << 8) | (byte as u128);
        }
        value.to_string()
    } else {
        "0".to_string()
    }
}

/// Process NEAR blocks and output database changes
#[substreams::handlers::store]
fn store_block(block: Block, s: StoreSetIfNotExistsProto<BlockEntity>) {
    if let Some(header) = block.header.as_ref() {
        let seconds = (header.timestamp_nanosec / 1_000_000_000) as i64;
        let nanos = (header.timestamp_nanosec % 1_000_000_000) as u32;

        let datetime = DateTime::<Utc>::from_timestamp(seconds, nanos).unwrap();
        let timestamp = datetime.format("%Y-%m-%d %H:%M:%S.%f").to_string();

        let block_entity = BlockEntity {
            height: header.height,
            hash: if let Some(h) = &header.hash { hex::encode(&h.bytes) } else { "".to_string() },
            prev_hash: if let Some(h) = &header.prev_hash { hex::encode(&h.bytes) } else { "".to_string() },
            author: block.author.clone(),
            timestamp,
            gas_price: if let Some(gp) = &header.gas_price { bytes_to_string(&gp.bytes) } else { "0".to_string() },
            total_supply: if let Some(ts) = &header.total_supply { bytes_to_string(&ts.bytes) } else { "0".to_string() },
        };
        
        s.set_if_not_exists(header.height, header.height.to_string(), &block_entity);
    }
}

#[substreams::handlers::store]
fn store_chunk(block: Block, s: StoreSetIfNotExistsProto<Chunk>) {
    if let Some(header) = block.header.as_ref() {
        for chunk_header in &block.chunk_headers {
            let chunk = Chunk {
                height: header.height,
                chunk_hash: hex::encode(&chunk_header.chunk_hash),
                prev_block_hash: hex::encode(&chunk_header.prev_block_hash),
                outcome_root: hex::encode(&chunk_header.outcome_root),
                prev_state_root: hex::encode(&chunk_header.prev_state_root),
                encoded_merkle_root: hex::encode(&chunk_header.encoded_merkle_root),
                encoded_length: chunk_header.encoded_length,
                height_created: chunk_header.height_created,
                height_included: chunk_header.height_included,
                shard_id: chunk_header.shard_id,
                gas_used: chunk_header.gas_used,
                gas_limit: chunk_header.gas_limit,
                validator_reward: if let Some(vr) = &chunk_header.validator_reward { bytes_to_string(&vr.bytes) } else { "0".to_string() },
                balance_burnt: if let Some(bb) = &chunk_header.balance_burnt { bytes_to_string(&bb.bytes) } else { "0".to_string() },
                outgoing_receipts_root: hex::encode(&chunk_header.outgoing_receipts_root),
                tx_root: hex::encode(&chunk_header.tx_root),
                author: block.author.clone(),
            };

            let key = format!("{}-{}", header.height, hex::encode(&chunk_header.chunk_hash));
            s.set_if_not_exists(header.height, key, &chunk);
        }
    }
}

#[substreams::handlers::store]
fn store_receipt(block: Block, s: StoreSetIfNotExistsProto<Receipt>) {
    if let Some(header) = block.header.as_ref() {
        for shard in &block.shards {
            for receipt_exec_outcome in &shard.receipt_execution_outcomes {
                if let Some(receipt) = &receipt_exec_outcome.receipt {
                    let receipt_entity = Receipt {
                        height: header.height,
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
                        receipt_id: if let Some(id) = &receipt.receipt_id { hex::encode(&id.bytes) } else { "".to_string() },
                        predecessor_id: receipt.predecessor_id.clone(),
                        receiver_id: receipt.receiver_id.clone(),
                        receipt_kind: match &receipt.receipt {
                            Some(receipt::Receipt::Action(_)) => "Action".to_string(),
                            Some(receipt::Receipt::Data(_)) => "Data".to_string(),
                            None => "Unknown".to_string(),
                        },
                        author: block.author.clone(),
                    };

                    let key = format!("{}-{}", header.height, if let Some(id) = &receipt.receipt_id { hex::encode(&id.bytes) } else { "".to_string() });
                    s.set_if_not_exists(header.height, key, &receipt_entity);
                }
            }
        }
    }
}

#[substreams::handlers::store]
fn store_receipt_action(block: Block, s: StoreSetIfNotExistsProto<ReceiptAction>) {
    if let Some(header) = block.header.as_ref() {
        let seconds = (header.timestamp_nanosec / 1_000_000_000) as i64;
        let nanos = (header.timestamp_nanosec % 1_000_000_000) as u32;
        
        let datetime = DateTime::<Utc>::from_timestamp(seconds, nanos).unwrap();
        let timestamp = datetime.format("%Y-%m-%d %H:%M:%S.%f").to_string();
        
        for shard in &block.shards {
            for receipt_exec_outcome in &shard.receipt_execution_outcomes {
                if let Some(receipt) = &receipt_exec_outcome.receipt {
                    let receipt_id = if let Some(id) = &receipt.receipt_id { hex::encode(&id.bytes) } else { "".to_string() };
                    
                    // Only process action receipts
                    if let Some(receipt::Receipt::Action(action_receipt)) = &receipt.receipt {
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
                        
                        // Process each action in the receipt
                        for (action_index, action) in action_receipt.actions.iter().enumerate() {
                            let action_kind;
                            let mut method_name = "".to_string();
                            let mut gas = 0u64;
                            let mut deposit = "0".to_string();
                            let mut args_base64 = "".to_string();
                            
                            // Determine action type and extract relevant fields
                            match &action.action {
                                Some(pb::sf::near::r#type::v1::action::Action::CreateAccount(_)) => {
                                    action_kind = "CreateAccount".to_string();
                                }
                                Some(pb::sf::near::r#type::v1::action::Action::DeployContract(_)) => {
                                    action_kind = "DeployContract".to_string();
                                }
                                Some(pb::sf::near::r#type::v1::action::Action::FunctionCall(func_call)) => {
                                    action_kind = "FunctionCall".to_string();
                                    method_name = func_call.method_name.clone();
                                    gas = func_call.gas;
                                    if let Some(dep) = &func_call.deposit {
                                        deposit = bytes_to_string(&dep.bytes);
                                    }
                                    args_base64 = base64::encode(&func_call.args);
                                }
                                Some(pb::sf::near::r#type::v1::action::Action::Transfer(transfer)) => {
                                    action_kind = "Transfer".to_string();
                                    if let Some(dep) = &transfer.deposit {
                                        deposit = bytes_to_string(&dep.bytes);
                                    }
                                }
                                Some(pb::sf::near::r#type::v1::action::Action::Stake(_)) => {
                                    action_kind = "Stake".to_string();
                                }
                                Some(pb::sf::near::r#type::v1::action::Action::AddKey(_)) => {
                                    action_kind = "AddKey".to_string();
                                }
                                Some(pb::sf::near::r#type::v1::action::Action::DeleteKey(_)) => {
                                    action_kind = "DeleteKey".to_string();
                                }
                                Some(pb::sf::near::r#type::v1::action::Action::DeleteAccount(_)) => {
                                    action_kind = "DeleteAccount".to_string();
                                }
                                Some(pb::sf::near::r#type::v1::action::Action::Delegate(_)) => {
                                    action_kind = "Delegate".to_string();
                                }
                                Some(pb::sf::near::r#type::v1::action::Action::DeployGlobalContract(_)) => {
                                    action_kind = "DeployGlobalContract".to_string();
                                }
                                Some(pb::sf::near::r#type::v1::action::Action::DeployGlobalContractByAccountId(_)) => {
                                    action_kind = "DeployGlobalContractByAccountId".to_string();
                                }
                                Some(pb::sf::near::r#type::v1::action::Action::UseGlobalContract(_)) => {
                                    action_kind = "UseGlobalContract".to_string();
                                }
                                Some(pb::sf::near::r#type::v1::action::Action::UseGlobalContractByAccountId(_)) => {
                                    action_kind = "UseGlobalContractByAccountId".to_string();
                                }
                                None => {
                                    action_kind = "Unknown".to_string();
                                }
                            }
                            
                            // Create a unique ID by combining receipt_id and action_index
                            let unique_id = format!("{}-{}", receipt_id, action_index);
                            
                            let receipt_action = ReceiptAction {
                                id: unique_id.clone(), // Set the new primary key field
                                block_height: header.height,
                                receipt_id: receipt_id.clone(),
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
                                author: block.author.clone(),
                                method_name,
                                gas,
                                deposit,
                                args_base64,
                                action_index: action_index as u32,
                                block_timestamp: timestamp.clone(),
                            };
                            
                            // Use the unique ID as the key for the store
                            s.set_if_not_exists(header.height, unique_id, &receipt_action);
                        }
                    }
                }
            }
        }
    }
}

#[substreams::handlers::map]
fn db_out(
    block_deltas: store::Deltas<DeltaProto<BlockEntity>>,
    chunk_deltas: store::Deltas<DeltaProto<Chunk>>,
    receipt_deltas: store::Deltas<DeltaProto<Receipt>>,
    receipt_action_deltas: store::Deltas<DeltaProto<ReceiptAction>>)
-> Result<DatabaseChanges, substreams::errors::Error> {
    let mut database_changes: DatabaseChanges = Default::default();

    transform_block_to_database_changes(&mut database_changes, block_deltas);
    transform_chunk_to_database_changes(&mut database_changes, chunk_deltas);
    transform_receipt_to_database_changes(&mut database_changes, receipt_deltas);
    transform_receipt_action_to_database_changes(&mut database_changes, receipt_action_deltas);

    Ok(database_changes)
}

fn transform_block_to_database_changes(
    changes: &mut DatabaseChanges,
    deltas: store::Deltas<DeltaProto<BlockEntity>>,
) {
    for delta in deltas.deltas {
        match delta.operation {
            DeltaOperation::Create => {
                push_create_block(changes, &delta.key, delta.ordinal, &delta.new_value)
            }
            _ => {}
        }
    }
}

fn transform_chunk_to_database_changes(
    changes: &mut DatabaseChanges,
    deltas: store::Deltas<DeltaProto<Chunk>>,
) {
    for delta in deltas.deltas {
        match delta.operation {
            DeltaOperation::Create => {
                push_create_chunk(changes, &delta.key, delta.ordinal, &delta.new_value)
            }
            _ => {}
        }
    }
}

fn transform_receipt_to_database_changes(
    changes: &mut DatabaseChanges,
    deltas: store::Deltas<DeltaProto<Receipt>>,
) {
    for delta in deltas.deltas {
        match delta.operation {
            DeltaOperation::Create => {
                push_create_receipt(changes, &delta.key, delta.ordinal, &delta.new_value)
            }
            _ => {}
        }
    }
}

fn transform_receipt_action_to_database_changes(
    changes: &mut DatabaseChanges,
    deltas: store::Deltas<DeltaProto<ReceiptAction>>,
) {
    for delta in deltas.deltas {
        match delta.operation {
            DeltaOperation::Create => {
                push_create_receipt_action(changes, &delta.key, delta.ordinal, &delta.new_value)
            }
            _ => {}
        }
    }
}

fn push_create_block(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    value: &BlockEntity,
) {
    changes
        .push_change("blocks", key, ordinal, Operation::Create)
        .change("height", (None, value.height))
        .change("hash", (None, &value.hash))
        .change("prev_hash", (None, &value.prev_hash))
        .change("author", (None, &value.author))
        .change("timestamp", (None, &value.timestamp))
        .change("gas_price", (None, &value.gas_price))
        .change("total_supply", (None, &value.total_supply));
}

fn push_create_chunk(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    value: &Chunk,
) {
    changes
        .push_change("chunks", key, ordinal, Operation::Create)
        .change("height", (None, value.height))
        .change("chunk_hash", (None, &value.chunk_hash))
        .change("prev_block_hash", (None, &value.prev_block_hash))
        .change("outcome_root", (None, &value.outcome_root))
        .change("prev_state_root", (None, &value.prev_state_root))
        .change("encoded_merkle_root", (None, &value.encoded_merkle_root))
        .change("encoded_length", (None, value.encoded_length))
        .change("height_created", (None, value.height_created))
        .change("height_included", (None, value.height_included))
        .change("shard_id", (None, value.shard_id))
        .change("gas_used", (None, value.gas_used))
        .change("gas_limit", (None, value.gas_limit))
        .change("validator_reward", (None, &value.validator_reward))
        .change("balance_burnt", (None, &value.balance_burnt))
        .change("outgoing_receipts_root", (None, &value.outgoing_receipts_root))
        .change("tx_root", (None, &value.tx_root))
        .change("author", (None, &value.author));
}

fn push_create_receipt(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    value: &Receipt,
) {
    changes
        .push_change("receipts", key, ordinal, Operation::Create)
        .change("height", (None, value.height))
        .change("block_hash", (None, &value.block_hash))
        .change("chunk_hash", (None, &value.chunk_hash))
        .change("receipt_id", (None, &value.receipt_id))
        .change("predecessor_id", (None, &value.predecessor_id))
        .change("receiver_id", (None, &value.receiver_id))
        .change("receipt_kind", (None, &value.receipt_kind))
        .change("author", (None, &value.author));
}

fn push_create_receipt_action(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    value: &ReceiptAction,
) {
    changes
        .push_change("receipt_actions", key, ordinal, Operation::Create)
        .change("id", (None, &value.id))
        .change("block_height", (None, value.block_height))
        .change("receipt_id", (None, &value.receipt_id))
        .change("signer_account_id", (None, &value.signer_account_id))
        .change("signer_public_key", (None, &value.signer_public_key))
        .change("gas_price", (None, &value.gas_price))
        .change("action_kind", (None, &value.action_kind))
        .change("predecessor_id", (None, &value.predecessor_id))
        .change("receiver_id", (None, &value.receiver_id))
        .change("block_hash", (None, &value.block_hash))
        .change("chunk_hash", (None, &value.chunk_hash))
        .change("author", (None, &value.author))
        .change("method_name", (None, &value.method_name))
        .change("gas", (None, value.gas))
        .change("deposit", (None, &value.deposit))
        .change("args_base64", (None, &value.args_base64))
        .change("action_index", (None, value.action_index))
        .change("block_timestamp", (None, &value.block_timestamp));
}
