use substreams_database_change::pb::database::{table_change::Operation, DatabaseChanges};
use crate::pb::near::entities::v1::Block;

pub fn push_create_block(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    value: &Block,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_create_block() {
        let mut changes = DatabaseChanges::default();
        
        let block = Block {
            height: 12345,
            hash: "test_hash".to_string(),
            prev_hash: "prev_hash".to_string(),
            author: "test_author".to_string(),
            timestamp: "2022-01-01 00:00:00".to_string(),
            gas_price: "100".to_string(),
            total_supply: "1000000".to_string(),
        };

        push_create_block(&mut changes, "test-key", 0, &block);
        
        // Should create exactly one table change
        assert_eq!(changes.table_changes.len(), 1);
        
        let table_change = &changes.table_changes[0];
        assert_eq!(table_change.table, "blocks");
        
        // Verify all fields are present
        let field_names: Vec<&str> = table_change.fields.iter().map(|f| f.name.as_str()).collect();
        assert!(field_names.contains(&"height"));
        assert!(field_names.contains(&"hash"));
        assert!(field_names.contains(&"prev_hash"));
        assert!(field_names.contains(&"author"));
        assert!(field_names.contains(&"timestamp"));
        assert!(field_names.contains(&"gas_price"));
        assert!(field_names.contains(&"total_supply"));
    }

    #[test]
    fn test_push_create_block_with_empty_strings() {
        let mut changes = DatabaseChanges::default();
        
        let block = Block {
            height: 0,
            hash: "".to_string(),
            prev_hash: "".to_string(),
            author: "".to_string(),
            timestamp: "".to_string(),
            gas_price: "".to_string(),
            total_supply: "".to_string(),
        };

        push_create_block(&mut changes, "empty-key", 1, &block);
        
        // Should still create a table change even with empty values
        assert_eq!(changes.table_changes.len(), 1);
        
        let table_change = &changes.table_changes[0];
        assert_eq!(table_change.table, "blocks");
    }
} 