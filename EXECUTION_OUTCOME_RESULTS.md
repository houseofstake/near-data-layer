# Execution Outcome Results Indexer

This document describes the new functionality added to capture execution outcome results (return values) and receipt action arguments (input parameters) from NEAR function calls.

## Overview

The NEAR blockchain execution outcomes contain return values from successful function calls, and receipt actions contain input parameters passed to functions. These are not captured in the standard `execution_outcomes` and `receipt_actions` tables. This new functionality extracts and stores both:

1. **Execution Outcome Results** - Return values from successful function calls
2. **Receipt Action Arguments** - Input parameters passed to function calls

## New Components

### 1. Database Schema

Two new tables have been added to `schema.sql`:

```sql
-- For capturing return values from successful function calls
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

-- For capturing input parameters passed to function calls
CREATE TABLE IF NOT EXISTS receipt_action_arguments (
    id TEXT PRIMARY KEY, -- receipt_id + action_index as primary key
    receipt_id TEXT NOT NULL,
    action_index INTEGER NOT NULL,
    block_height BIGINT NOT NULL,
    block_hash TEXT NOT NULL,
    chunk_hash TEXT NOT NULL,
    shard_id TEXT NOT NULL,
    method_name TEXT NOT NULL,
    receiver_id TEXT NOT NULL,
    signer_account_id TEXT NOT NULL,
    predecessor_id TEXT NOT NULL,
    args_base64 TEXT NOT NULL, -- Original base64 encoded arguments
    args_json TEXT, -- JSON representation if parseable
    gas BIGINT NOT NULL,
    deposit TEXT NOT NULL,
    block_timestamp TIMESTAMP NOT NULL,
    FOREIGN KEY (receipt_id) REFERENCES receipt_actions(receipt_id)
);
```

### 2. Protobuf Entities

Two new entities have been added to `proto/entities.proto`:

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

message ReceiptActionArguments {
  string id = 1; // Primary key (receipt_id + action_index)
  string receipt_id = 2;
  uint32 action_index = 3;
  uint64 block_height = 4;
  string block_hash = 5;
  string chunk_hash = 6;
  string shard_id = 7;
  string method_name = 8;
  string receiver_id = 9;
  string signer_account_id = 10;
  string predecessor_id = 11;
  string args_base64 = 12; // Original base64 encoded arguments
  string args_json = 13; // JSON representation if parseable
  uint64 gas = 14;
  string deposit = 15;
  string block_timestamp = 16; // Timestamp from the block header
}
```

### 3. Processors

Two new processors have been created:

- **`process_execution_outcome_result`** - Extracts return values from `SuccessValue` execution outcomes
- **`process_receipt_action_arguments`** - Extracts input parameters from FunctionCall actions

### 4. Pushers

Two new pushers handle database changes for the new tables.

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

### Querying Receipt Action Arguments

You can query function call arguments (input parameters):

```sql
-- Get contract initialization arguments (method_name = 'new')
SELECT 
    raa.receipt_id,
    raa.block_height,
    raa.block_timestamp,
    raa.method_name,
    raa.receiver_id as contract_address,
    raa.signer_account_id as deployer,
    raa.args_json,
    -- Extract specific config values from arguments
    raa.args_json::json->'config'->>'owner_account_id' as owner_account_id,
    raa.args_json::json->'config'->>'local_deposit' as local_deposit,
    raa.args_json::json->'config'->>'min_lockup_deposit' as min_lockup_deposit,
    raa.args_json::json->'config'->>'unlock_duration_ns' as unlock_duration_ns,
    raa.args_json::json->'config'->'guardians' as guardians,
    raa.args_json::json->'config'->'lockup_code_deployers' as lockup_code_deployers,
    raa.args_json::json->'config'->>'staking_pool_whitelist_account_id' as staking_pool_whitelist,
    -- Extract venear growth config from arguments
    raa.args_json::json->'venear_growth_config'->'annual_growth_rate_ns'->>'numerator' as growth_rate_numerator,
    raa.args_json::json->'venear_growth_config'->'annual_growth_rate_ns'->>'denominator' as growth_rate_denominator
FROM receipt_action_arguments raa
WHERE raa.method_name = 'new'
  AND raa.args_json IS NOT NULL
  AND raa.args_json != ''
ORDER BY raa.block_timestamp DESC;
```

### Contract Initialization Data

The receipt action arguments indexer will capture contract initialization data from method calls like `new`. This includes configuration data that is passed as input parameters when contracts are deployed.

### Example: Querying the Specific Transaction

For the transaction mentioned in the requirements:

```sql
-- Query the specific transaction arguments
SELECT 
    raa.receipt_id,
    raa.block_height,
    raa.block_timestamp,
    raa.args_value,
    raa.args_json
FROM receipt_action_arguments raa
WHERE raa.receipt_id = '2cc8rV5qEeyLooJBTPYBW5dqNiSA8P2BQWgyyqhmPPUi'
  AND raa.method_name = 'new';
```

### Comparing Input vs Output

You can compare input arguments with output results for the same transaction:

```sql
-- Compare arguments vs results for the same transaction
SELECT 
    raa.receipt_id,
    raa.method_name,
    raa.receiver_id as contract_address,
    raa.args_json as input_arguments,
    eor.result_json as output_results
FROM receipt_action_arguments raa
LEFT JOIN execution_outcome_results eor ON raa.receipt_id = eor.receipt_id
WHERE raa.method_name = 'new'
  AND raa.args_json IS NOT NULL
ORDER BY raa.block_timestamp DESC;
```

### Parsing JSON Results

Both `result_json` and `args_json` fields contain parsed JSON when the values are valid JSON:

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
6. **Storage**: Both raw base64 values and parsed JSON (if available) are stored

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

## Integration

The new functionality is automatically integrated into the existing indexer:

- The `process_receipt_actions` function now calls `process_receipt_action_arguments`
- The `process_execution_outcome` function now calls `process_execution_outcome_result`
- No changes to the main processing pipeline are required
- The new tables are created automatically when the schema is applied

## Future Enhancements

Potential improvements could include:

1. **Selective Processing**: Only process specific contracts or method names
2. **Result Caching**: Cache parsed JSON results to avoid re-parsing
3. **Custom Parsers**: Add specific parsers for known contract formats
4. **Indexing**: Add database indexes for common query patterns
5. **Compression**: Compress large JSON structures for storage efficiency 