# Execution Outcome Results Indexer

This document describes the new functionality added to capture execution outcome results (return values) from successful NEAR function calls.

## Overview

The NEAR blockchain execution outcomes contain return values from successful function calls, but these are not captured in the standard `execution_outcomes` table. This new functionality extracts and stores these return values in a dedicated `execution_outcome_results` table.

## New Components

### 1. Database Schema

A new table `execution_outcome_results` has been added to `schema.sql`:

```sql
CREATE TABLE IF NOT EXISTS execution_outcome_results (
    receipt_id TEXT PRIMARY KEY,
    block_height BIGINT NOT NULL,
    block_hash TEXT NOT NULL,
    chunk_hash TEXT NOT NULL,
    shard_id TEXT NOT NULL,
    status TEXT NOT NULL,
    result_value TEXT, -- Base64 encoded return value
    result_json TEXT, -- JSON representation if parseable
    block_timestamp TIMESTAMP NOT NULL,
    FOREIGN KEY (receipt_id) REFERENCES execution_outcomes(receipt_id)
);
```

### 2. Protobuf Entity

A new `ExecutionOutcomeResult` entity has been added to `proto/entities.proto`:

```protobuf
message ExecutionOutcomeResult {
  string receipt_id = 1; // Primary key, matches execution_outcomes.receipt_id
  uint64 block_height = 2;
  string block_hash = 3;
  string chunk_hash = 4;
  string shard_id = 5;
  string status = 6; // Should be "SuccessValue" for this table
  string result_value = 7; // The actual return value from the function call (base64 encoded)
  string result_json = 8; // JSON representation of the result if it can be parsed
  string block_timestamp = 9; // Timestamp from the block header
}
```

### 3. Processor

A new processor `src/processors/execution_outcome_result.rs` has been created that:

- Extracts return values from `SuccessValue` execution outcomes
- Decodes base64-encoded return values
- Attempts to parse the result as JSON
- Stores both the raw base64 value and parsed JSON representation

### 4. Pusher

A new pusher `src/pushers/execution_outcome_result.rs` handles database changes for the new table.

## Usage

### Querying Execution Outcome Results

You can now query execution outcome results and join them with existing tables:

```sql
-- Get execution outcome results with related data
SELECT 
    eor.receipt_id,
    eor.block_height,
    eor.block_timestamp,
    eo.status as execution_status,
    eor.status as result_status,
    eor.result_value,
    eor.result_json,
    ra.method_name,
    ra.receiver_id
FROM execution_outcome_results eor
JOIN execution_outcomes eo ON eor.receipt_id = eo.receipt_id
JOIN receipt_actions ra ON eor.receipt_id = ra.receipt_id
WHERE eor.block_height = 201857082
ORDER BY eor.block_timestamp DESC;
```

### Example: Querying the Specific Transaction

For the transaction mentioned in the requirements:

```sql
-- Query the specific transaction result
SELECT 
    eor.receipt_id,
    eor.block_height,
    eor.block_timestamp,
    eor.result_value,
    eor.result_json
FROM execution_outcome_results eor
WHERE eor.receipt_id = 'EmAhjD7QDiwqdfHJDc8auAycNtiZz3KxAzAkzteHX9EV'
  AND eor.block_height = 201857082;
```

### Parsing JSON Results

The `result_json` field contains parsed JSON when the return value is valid JSON:

```sql
-- Extract specific values from JSON results
SELECT 
    receipt_id,
    result_json::json->1->'V0'->'total_venear_balance'->>'near_balance' as near_balance,
    result_json::json->1->'V0'->'total_venear_balance'->>'extra_venear_balance' as extra_balance
FROM execution_outcome_results
WHERE result_json IS NOT NULL 
  AND result_json != '';
```

## Testing

A test script has been created at `scripts/test_execution_outcome_result.py` to verify the processing logic with the specific transaction data provided.

To run the test:

```bash
cd scripts
python3 test_execution_outcome_result.py
```

This will test the base64 decoding and JSON parsing logic with the exact transaction data from the NEAR testnet transaction you mentioned. The test demonstrates that the execution outcome results processor will correctly handle and parse the return values.

## Implementation Details

### Data Flow

1. **Block Processing**: When a NEAR block is processed, execution outcomes are extracted
2. **SuccessValue Detection**: Only execution outcomes with `SuccessValue` status are processed for results
3. **Base64 Decoding**: The return value is decoded from base64
4. **JSON Parsing**: If the decoded value is valid UTF-8, it's attempted to be parsed as JSON
5. **Storage**: Both the raw base64 value and parsed JSON (if available) are stored

### Error Handling

- If base64 decoding fails, the raw hex value is stored
- If UTF-8 decoding fails, the hex representation is stored
- If JSON parsing fails, a string representation is stored
- The processor continues processing other execution outcomes even if one fails

### Performance Considerations

- Only `SuccessValue` execution outcomes are processed for results
- The JSON parsing is done in-memory during processing
- The base64 decoding is efficient and doesn't impact performance significantly

## Integration

The new functionality is automatically integrated into the existing indexer:

- The `process_execution_outcome` function now calls `process_execution_outcome_result`
- No changes to the main processing pipeline are required
- The new table is created automatically when the schema is applied

## Future Enhancements

Potential improvements could include:

1. **Selective Processing**: Only process results for specific contracts or method names
2. **Result Caching**: Cache parsed JSON results to avoid re-parsing
3. **Custom Parsers**: Add specific parsers for known contract return value formats
4. **Indexing**: Add database indexes for common query patterns 