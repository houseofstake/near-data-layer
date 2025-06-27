# Execution Outcome Results Indexer

This document describes the functionality added to capture execution outcome results (return values) and receipt action arguments (input parameters) from NEAR function calls.

## Overview

The NEAR blockchain execution outcomes contain return values from successful function calls, and receipt actions contain input parameters passed to functions. This functionality extracts and stores both:

1. **Execution Outcome Results** - Return values from successful function calls (stored in the `execution_outcomes` table)
2. **Receipt Action Arguments** - Input parameters passed to function calls (stored in the `receipt_actions` table with `args_json` field)

## Components

### 1. Database Schema

The `execution_outcomes` table has been enhanced with additional fields:

```sql
CREATE TABLE IF NOT EXISTS execution_outcomes (
    receipt_id TEXT PRIMARY KEY,
    block_height BIGINT NOT NULL,
    block_hash TEXT NOT NULL,
    chunk_hash TEXT NOT NULL,
    shard_id TEXT NOT NULL,
    gas_burnt BIGINT NOT NULL,
    gas_used FLOAT NOT NULL,
    tokens_burnt FLOAT NOT NULL,
    executor_account_id TEXT NOT NULL,
    status TEXT NOT NULL,
    outcome_receipt_ids TEXT[] NOT NULL,
    executed_in_block_hash TEXT NOT NULL,
    logs TEXT[],
    results_base64 TEXT, -- Base64 encoded return value (only for SuccessValue outcomes)
    results_json TEXT, -- JSON representation if parseable (only for SuccessValue outcomes)
    block_timestamp TIMESTAMP -- Timestamp from the block header
);
```

The `receipt_actions` table has been enhanced with an `args_json` field:

```sql
CREATE TABLE IF NOT EXISTS receipt_actions (
    id TEXT PRIMARY KEY,
    block_height BIGINT NOT NULL,
    receipt_id TEXT NOT NULL,
    signer_account_id TEXT NOT NULL,
    signer_public_key TEXT NOT NULL,
    gas_price TEXT NOT NULL,
    action_kind TEXT NOT NULL,
    predecessor_id TEXT NOT NULL,
    receiver_id TEXT NOT NULL,
    block_hash TEXT NOT NULL,
    chunk_hash TEXT NOT NULL,
    author TEXT NOT NULL,
    method_name TEXT NOT NULL,
    gas BIGINT NOT NULL,
    deposit TEXT NOT NULL,
    args_base64 TEXT NOT NULL, -- Original base64 encoded arguments
    args_json TEXT, -- JSON representation if parseable
    action_index INTEGER NOT NULL,
    block_timestamp TIMESTAMP NOT NULL
);
```

### 2. Protobuf Entities

The `ExecutionOutcome` entity has been enhanced with additional fields:

```protobuf
message ExecutionOutcome {
  uint64 block_height = 1;
  string block_hash = 2;
  string chunk_hash = 3;
  string shard_id = 4;
  uint64 gas_burnt = 5;
  float gas_used = 6;
  float tokens_burnt = 7;
  string executor_account_id = 8;
  string status = 9;
  repeated string outcome_receipt_ids = 10; // Will be stored as TEXT[] in PostgreSQL
  string receipt_id = 11;
  string executed_in_block_hash = 12;
  repeated string logs = 13;
  string results_base64 = 14; // The actual return value from the function call (base64 encoded, only for SuccessValue outcomes)
  string results_json = 15; // JSON representation of the result if it can be parsed (only for SuccessValue outcomes)
  string block_timestamp = 16; // Timestamp from the block header
}
```

The `ReceiptAction` entity has been enhanced with an `args_json` field:

```protobuf
message ReceiptAction {
  string id = 17; // Unique ID as primary key (receipt_id + action_index)
  uint64 block_height = 1;
  string receipt_id = 2;
  string signer_account_id = 3;
  string signer_public_key = 4;
  string gas_price = 5;
  string action_kind = 6;
  string predecessor_id = 7;
  string receiver_id = 8;
  string block_hash = 9;
  string chunk_hash = 10;
  string author = 11;
  string method_name = 12;
  uint64 gas = 13;
  string deposit = 14;
  string args_base64 = 15; // Base64 encoded args for FunctionCall
  string args_json = 19; // JSON representation if parseable
  uint32 action_index = 16; // Index of this action within the receipt
  string block_timestamp = 18; // Timestamp from the block header
}
```

### 3. Processors

Two processors handle the data extraction:

- **`process_execution_outcome`** - Extracts return values from `SuccessValue` execution outcomes and stores them in the unified `execution_outcomes` table
- **`process_receipt_actions`** - Extracts input parameters from FunctionCall actions and stores them in the `receipt_actions` table with `args_json` field

### 4. Pushers

Pushers handle database changes for the enhanced tables.

## Usage

### Querying Execution Outcome Results

You can now query execution outcome results directly from the `execution_outcomes` table:

```sql
-- Get execution outcome results with related data
SELECT 
    eo.receipt_id,
    eo.block_height,
    eo.block_timestamp,
    eo.status as execution_status,
    eo.results_base64,
    eo.results_json,
    ra.method_name,
    ra.receiver_id
FROM execution_outcomes eo
JOIN receipt_actions ra ON eo.receipt_id = ra.receipt_id
WHERE eo.status = 'SuccessValue'
  AND eo.results_json IS NOT NULL
  AND eo.results_json != ''
ORDER BY eo.block_timestamp DESC;
```

### Querying Receipt Action Arguments

You can query function call arguments (input parameters) from the `receipt_actions` table:

```sql
-- Get contract initialization arguments (method_name = 'new')
SELECT 
    ra.receipt_id,
    ra.block_height,
    ra.block_timestamp,
    ra.method_name,
    ra.receiver_id as contract_address,
    ra.signer_account_id as deployer,
    ra.args_json,
    -- Extract specific config values from arguments
    ra.args_json::json->'config'->>'owner_account_id' as owner_account_id,
    ra.args_json::json->'config'->>'local_deposit' as local_deposit,
    ra.args_json::json->'config'->>'min_lockup_deposit' as min_lockup_deposit,
    ra.args_json::json->'config'->>'unlock_duration_ns' as unlock_duration_ns,
    ra.args_json::json->'config'->'guardians' as guardians,
    ra.args_json::json->'config'->'lockup_code_deployers' as lockup_code_deployers,
    ra.args_json::json->'config'->>'staking_pool_whitelist_account_id' as staking_pool_whitelist,
    -- Extract venear growth config from arguments
    ra.args_json::json->'venear_growth_config'->'annual_growth_rate_ns'->>'numerator' as growth_rate_numerator,
    ra.args_json::json->'venear_growth_config'->'annual_growth_rate_ns'->>'denominator' as growth_rate_denominator
FROM receipt_actions ra
WHERE ra.method_name = 'new'
  AND ra.args_json IS NOT NULL
  AND ra.args_json != ''
ORDER BY ra.block_timestamp DESC;
```

### Contract Initialization Data

The receipt actions indexer will capture contract initialization data from method calls like `new`. This includes configuration data that is passed as input parameters when contracts are deployed.

### Example: Querying the Specific Transaction

For the transaction mentioned in the requirements:

```sql
-- Query the specific transaction arguments
SELECT 
    ra.receipt_id,
    ra.block_height,
    ra.block_timestamp,
    ra.args_base64,
    ra.args_json
FROM receipt_actions ra
WHERE ra.receipt_id = '2cc8rV5qEeyLooJBTPYBW5dqNiSA8P2BQWgyyqhmPPUi'
  AND ra.method_name = 'new';
```

### Comparing Input vs Output

You can compare input arguments with output results for the same transaction:

```sql
-- Compare arguments vs results for the same transaction
SELECT 
    ra.receipt_id,
    ra.method_name,
    ra.receiver_id as contract_address,
    ra.args_json as input_arguments,
    eo.results_json as output_results
FROM receipt_actions ra
LEFT JOIN execution_outcomes eo ON ra.receipt_id = eo.receipt_id
WHERE ra.method_name = 'new'
  AND ra.args_json IS NOT NULL
ORDER BY ra.block_timestamp DESC;
```

## Summary

This consolidation simplifies the database schema by:

1. **Eliminating redundancy**: The `receipt_action_arguments` table was essentially duplicating data already present in `receipt_actions`
2. **Improving performance**: Fewer joins needed when querying receipt action data
3. **Simplifying maintenance**: One table to maintain instead of two
4. **Better data consistency**: All receipt action data is now in one place

The `args_json` field in the `receipt_actions` table provides the same functionality as the previous `receipt_action_arguments` table, but with better integration and performance.

## Testing

Test scripts have been created to verify the processing logic:

```bash
# Test execution outcome results
cd scripts
python3 test_execution_outcome_result.py

# Test receipt action arguments
python3 test_receipt_action_arguments.py
```

These will test the base64 decoding and JSON parsing logic with the exact transaction data from the NEAR testnet transactions you mentioned.

## Implementation Details

### Data Flow

1. **Block Processing**: When a NEAR block is processed, both receipt actions and execution outcomes are extracted
2. **Function Call Detection**: Only FunctionCall actions are processed for arguments
3. **SuccessValue Detection**: Only execution outcomes with `SuccessValue` status are processed for results
4. **Base64 Decoding**: Both arguments and return values are decoded from base64
5. **JSON Parsing**: If the decoded values are valid UTF-8, they're attempted to be parsed as JSON
6. **Storage**: Both raw base64 values and parsed JSON (if available) are stored in the unified `execution_outcomes` table

### Error Handling

- If base64 decoding fails, the raw hex value is stored
- If UTF-8 decoding fails, the hex representation is stored
- If JSON parsing fails, a string representation is stored
- The processors continue processing other items even if one fails

### Performance Considerations

- Only FunctionCall actions are processed for arguments
- Only `SuccessValue` execution outcomes are processed for results
- The JSON parsing is done in-memory during processing
- The base64 decoding is efficient and doesn't impact performance significantly
- Using a unified table reduces JOIN complexity and improves query performance

## Integration

The new functionality is automatically integrated into the existing indexer:

- The `process_execution_outcome` function now includes result processing logic
- The `process_receipt_actions` function calls `process_receipt_action_arguments`
- No changes to the main processing pipeline are required
- The enhanced table structure is created automatically when the schema is applied

## Benefits of Unified Table Design

1. **Reduced Data Duplication**: No need to store the same block/transaction metadata twice
2. **Simplified Queries**: No need for JOINs between separate tables
3. **Better Performance**: Fewer table scans and joins
4. **Easier Maintenance**: Single table to manage instead of two related tables
5. **Atomic Operations**: All execution outcome data is stored together

## Future Enhancements

Potential improvements could include:

1. **Selective Processing**: Only process specific contracts or method names
2. **Result Caching**: Cache parsed JSON results to avoid re-parsing
3. **Custom Parsers**: Add specific parsers for known contract formats
4. **Indexing**: Add database indexes for common query patterns
5. **Compression**: Compress large JSON structures for storage efficiency 