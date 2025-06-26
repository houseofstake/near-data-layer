use substreams_database_change::pb::database::{table_change::Operation, DatabaseChanges};
use crate::pb::near::entities::v1::ReceiptAction;

pub fn push_create_receipt_action(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_create_receipt_action() {
        let mut changes = DatabaseChanges::default();
        
        let receipt_action = ReceiptAction {
            id: "receipt_123_0".to_string(),
            block_height: 12345,
            receipt_id: "receipt_123".to_string(),
            signer_account_id: "signer.near".to_string(),
            signer_public_key: "ed25519:abc123".to_string(),
            gas_price: "100".to_string(),
            action_kind: "FunctionCall".to_string(),
            predecessor_id: "predecessor.near".to_string(),
            receiver_id: "receiver.near".to_string(),
            block_hash: "block_hash".to_string(),
            chunk_hash: "chunk_hash".to_string(),
            author: "author.near".to_string(),
            method_name: "transfer".to_string(),
            gas: 1000,
            deposit: "1000000".to_string(),
            args_base64: "eyJhbW91bnQiOiIxMDAwMDAwIn0=".to_string(),
            action_index: 0,
            block_timestamp: "2022-01-01 00:00:00".to_string(),
        };

        push_create_receipt_action(&mut changes, "test-key", 0, &receipt_action);
        
        // Should create exactly one table change
        assert_eq!(changes.table_changes.len(), 1);
        
        let table_change = &changes.table_changes[0];
        assert_eq!(table_change.table, "receipt_actions");
        
        // Verify all fields are present
        let field_names: Vec<&str> = table_change.fields.iter().map(|f| f.name.as_str()).collect();
        assert!(field_names.contains(&"id"));
        assert!(field_names.contains(&"block_height"));
        assert!(field_names.contains(&"receipt_id"));
        assert!(field_names.contains(&"signer_account_id"));
        assert!(field_names.contains(&"signer_public_key"));
        assert!(field_names.contains(&"gas_price"));
        assert!(field_names.contains(&"action_kind"));
        assert!(field_names.contains(&"predecessor_id"));
        assert!(field_names.contains(&"receiver_id"));
        assert!(field_names.contains(&"block_hash"));
        assert!(field_names.contains(&"chunk_hash"));
        assert!(field_names.contains(&"author"));
        assert!(field_names.contains(&"method_name"));
        assert!(field_names.contains(&"gas"));
        assert!(field_names.contains(&"deposit"));
        assert!(field_names.contains(&"args_base64"));
        assert!(field_names.contains(&"action_index"));
        assert!(field_names.contains(&"block_timestamp"));
    }

    #[test]
    fn test_push_create_receipt_action_with_empty_strings() {
        let mut changes = DatabaseChanges::default();
        
        let receipt_action = ReceiptAction {
            id: "".to_string(),
            block_height: 0,
            receipt_id: "".to_string(),
            signer_account_id: "".to_string(),
            signer_public_key: "".to_string(),
            gas_price: "".to_string(),
            action_kind: "".to_string(),
            predecessor_id: "".to_string(),
            receiver_id: "".to_string(),
            block_hash: "".to_string(),
            chunk_hash: "".to_string(),
            author: "".to_string(),
            method_name: "".to_string(),
            gas: 0,
            deposit: "".to_string(),
            args_base64: "".to_string(),
            action_index: 0,
            block_timestamp: "".to_string(),
        };

        push_create_receipt_action(&mut changes, "empty-key", 1, &receipt_action);
        
        // Should still create a table change even with empty values
        assert_eq!(changes.table_changes.len(), 1);
        
        let table_change = &changes.table_changes[0];
        assert_eq!(table_change.table, "receipt_actions");
    }

    #[test]
    fn test_push_create_receipt_action_transfer() {
        let mut changes = DatabaseChanges::default();
        
        let receipt_action = ReceiptAction {
            id: "receipt_456_0".to_string(),
            block_height: 67890,
            receipt_id: "receipt_456".to_string(),
            signer_account_id: "alice.near".to_string(),
            signer_public_key: "ed25519:def456".to_string(),
            gas_price: "200".to_string(),
            action_kind: "Transfer".to_string(),
            predecessor_id: "alice.near".to_string(),
            receiver_id: "bob.near".to_string(),
            block_hash: "block_hash_456".to_string(),
            chunk_hash: "chunk_hash_456".to_string(),
            author: "validator.near".to_string(),
            method_name: "".to_string(), // Transfer doesn't have method_name
            gas: 500,
            deposit: "5000000".to_string(),
            args_base64: "".to_string(), // Transfer doesn't have args
            action_index: 0,
            block_timestamp: "2022-01-02 12:00:00".to_string(),
        };

        push_create_receipt_action(&mut changes, "transfer-key", 2, &receipt_action);
        
        // Should create exactly one table change
        assert_eq!(changes.table_changes.len(), 1);
        
        let table_change = &changes.table_changes[0];
        assert_eq!(table_change.table, "receipt_actions");
        
        // Verify key fields for transfer action
        let field_names: Vec<&str> = table_change.fields.iter().map(|f| f.name.as_str()).collect();
        assert!(field_names.contains(&"action_kind"));
        assert!(field_names.contains(&"predecessor_id"));
        assert!(field_names.contains(&"receiver_id"));
        assert!(field_names.contains(&"deposit"));
    }
} 