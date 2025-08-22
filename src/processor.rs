use crate::config::Settings;
use crate::database::{Database, ReceiptActionRow, ExecutionOutcomeRow};
use anyhow::Result;
use fastnear_primitives::block_with_tx_hash::BlockWithTxHashes;
use fastnear_primitives::near_primitives::views::{ActionView, ReceiptEnumView, ExecutionStatusView};
use base64::{Engine as _, engine::general_purpose};

pub struct Processor {
    database: Database,
    settings: Settings,
}

impl Processor {
    pub fn new(database: Database, settings: Settings) -> Self {
        Self { database, settings }
    }

    /// Convert base64 encoded args to JSON Value
    /// If decoding fails or result is not valid JSON, return {"value": "invalid json"}
    fn args_base64_to_json(args_base64: &str) -> serde_json::Value {
        // Try to decode base64
        let decoded_bytes = match general_purpose::STANDARD.decode(args_base64) {
            Ok(bytes) => bytes,
            Err(_) => return serde_json::json!({"value": "invalid json"}),
        };

        // Try to convert bytes to UTF-8 string
        let decoded_str = match String::from_utf8(decoded_bytes) {
            Ok(s) => s,
            Err(_) => return serde_json::json!({"value": "invalid json"}),
        };

        // Try to parse as JSON
        match serde_json::from_str::<serde_json::Value>(&decoded_str) {
            Ok(json_value) => json_value, // Valid JSON, return the parsed value
            Err(_) => serde_json::json!({"value": "invalid json"}), // Invalid JSON, return error wrapper
        }
    }

    /// Extract data from ExecutionStatusView and convert to JSON
    /// Returns None if no data is available or Some(JSON) if data exists
    fn extract_results_json_from_status(status: &ExecutionStatusView) -> Option<serde_json::Value> {
        match status {
            ExecutionStatusView::SuccessValue(base64_data) => {
                if base64_data.is_empty() {
                    None
                } else {
                    // Convert Vec<u8> to base64 string, then decode and parse as JSON
                    let base64_str = general_purpose::STANDARD.encode(base64_data);
                    Some(Self::args_base64_to_json(&base64_str))
                }
            }
            ExecutionStatusView::SuccessReceiptId(receipt_id) => {
                // Store the receipt ID as JSON for tracking cross-receipt relationships
                Some(serde_json::json!({
                    "receipt_id": receipt_id.to_string(),
                    "status_type": "SuccessReceiptId"
                }))
            }
            ExecutionStatusView::Failure(failure_error) => {
                // Extract failure information for debugging and analysis
                Some(serde_json::json!({
                    "error": failure_error.to_string(),
                    "status_type": "Failure"
                }))
            }
            ExecutionStatusView::Unknown => {
                // Mark unknown status for investigation
                Some(serde_json::json!({
                    "status_type": "Unknown"
                }))
            }
        }
    }

    pub async fn initialize_tables(&self) -> Result<()> {
        self.database.initialize_tables(&self.settings).await
    }

    pub async fn get_cursor_for_app_version(&self) -> Result<Option<u64>> {
        self.database.get_cursor_for_version(&self.settings.app_version).await
    }

    pub async fn update_cursor(&self, id: &str, block_num: u64, block_hash: &str) -> Result<()> {
        self.database.update_cursor(id, block_num, block_hash).await
    }



    pub async fn process_block(&self, block: &BlockWithTxHashes) -> Result<()> {
        self.database.store_block(block).await?;
        self.process_receipt_actions_execution_outcomes(block).await?;
        Ok(())
    }

    pub async fn process_receipt_actions_execution_outcomes(&self, block: &BlockWithTxHashes) -> Result<()> {
        let mut actions = Vec::new();
        let mut execution_outcomes = Vec::new();
        
        let block_height = block.block.header.height;
        let block_hash = block.block.header.hash.to_string();
        let block_timestamp = {
            let secs = (block.block.header.timestamp_nanosec / 1_000_000_000) as i64;
            let nsecs = (block.block.header.timestamp_nanosec % 1_000_000_000) as u32;
            chrono::DateTime::<chrono::Utc>::from_timestamp(secs, nsecs)
                .unwrap_or_else(|| chrono::Utc::now())
                .naive_utc()
        };
        let block_author = block.block.author.to_string();

        for shard in &block.shards {
            // Get chunk hash from chunk field
            let chunk_hash = if let Some(chunk) = &shard.chunk {
                chunk.header.chunk_hash.to_string()
            } else {
                "".to_string() // Use empty string for missing chunks
            };
            let shard_id = shard.shard_id.to_string();

            // Process receipt actions
            for reo in &shard.receipt_execution_outcomes {
                let receipt = &reo.receipt;
                let receiver_id = receipt.receiver_id.to_string();
                let predecessor_id = receipt.predecessor_id.to_string();
                let receipt_id = receipt.receipt_id.to_string();

                // Filter for HOS contracts
                if self.settings.is_hos_contract(&receiver_id) || self.settings.is_hos_contract(&predecessor_id) {
                    // Process receipt actions
                    if let ReceiptEnumView::Action { 
                        signer_id, 
                        signer_public_key, 
                        gas_price, 
                        actions: action_list, 
                        .. 
                    } = &receipt.receipt {
                        
                        for (action_index, action) in action_list.iter().enumerate() {
                            if let ActionView::FunctionCall { method_name, args, gas, deposit } = action {
                                // Encode args to base64
                                let args_base64 = general_purpose::STANDARD.encode(args.as_ref() as &[u8]);
                                
                                // Compute args_json from base64
                                let args_json = Self::args_base64_to_json(&args_base64);
                                
                                // Generate unique ID for this action
                                let action_id = format!("{}-{}", receipt_id, action_index);
                                
                                let action_row = ReceiptActionRow {
                                    id: action_id,
                                    block_height: block_height as i64,
                                    receipt_id: receipt_id.clone(),
                                    signer_account_id: signer_id.to_string(),
                                    signer_public_key: signer_public_key.to_string(),
                                    gas_price: gas_price.to_string(),
                                    action_kind: "FunctionCall".to_string(),
                                    predecessor_id: predecessor_id.clone(),
                                    receiver_id: receiver_id.clone(),
                                    block_hash: block_hash.clone(),
                                    chunk_hash: chunk_hash.clone(),
                                    author: block_author.clone(),
                                    method_name: method_name.clone(),
                                    gas: *gas as i64,
                                    deposit: deposit.to_string(),
                                    args_base64,
                                    args_json,
                                    action_index: action_index as i32,
                                    block_timestamp,
                                };
                                
                                actions.push(action_row);
                            }
                        }
                    }

                    // Process execution outcome
                    let outcome = &reo.execution_outcome.outcome;
                    let outcome_receipt_ids: Vec<String> = outcome.receipt_ids.iter()
                        .map(|id| id.to_string())
                        .collect();

                    // Parse tokens_burnt as f64
                    let tokens_burnt = outcome.tokens_burnt as f64 / 1e24; // Convert from yoctoNEAR to NEAR
                    let gas_used = outcome.gas_burnt as f64; // Gas used is the same as gas burnt for most cases

                    // Extract only the status variant name without inner data
                    let status = match &outcome.status {
                        ExecutionStatusView::Unknown => "Unknown".to_string(),
                        ExecutionStatusView::Failure(_) => "Failure".to_string(),
                        ExecutionStatusView::SuccessValue(_) => "SuccessValue".to_string(),
                        ExecutionStatusView::SuccessReceiptId(_) => "SuccessReceiptId".to_string(),
                    };

                    // Extract results_json from status
                    let results_json = Self::extract_results_json_from_status(&outcome.status);

                    let outcome_row = ExecutionOutcomeRow {
                        receipt_id: receipt_id.clone(),
                        block_height: block_height as i64,
                        block_hash: block_hash.clone(),
                        chunk_hash: chunk_hash.clone(),
                        shard_id: shard_id.clone(),
                        gas_burnt: outcome.gas_burnt as i64,
                        gas_used,
                        tokens_burnt,
                        executor_account_id: outcome.executor_id.to_string(),
                        status,
                        outcome_receipt_ids,
                        executed_in_block_hash: block_hash.clone(),
                        logs: outcome.logs.clone(),
                        results_json,
                        block_timestamp: Some(block_timestamp),
                    };
                    
                    execution_outcomes.push(outcome_row);
                }
            }
        }

        // Store both actions and outcomes
        if !actions.is_empty() {
            self.database.store_receipt_actions(actions).await?;
        }
        if !execution_outcomes.is_empty() {
            self.database.store_execution_outcomes(execution_outcomes).await?;
        }

        Ok(())
    }

} 
