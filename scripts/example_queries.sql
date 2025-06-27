-- Example queries for execution outcome results

-- 1. Query contract initialization results (method_name = 'new')
SELECT 
    eo.receipt_id,
    eo.block_height,
    eo.block_timestamp,
    ra.method_name,
    ra.receiver_id as contract_address,
    ra.signer_account_id as deployer,
    eo.results_json,
    -- Extract specific config values
    eo.results_json::json->'config'->>'owner_account_id' as owner_account_id,
    eo.results_json::json->'config'->>'local_deposit' as local_deposit,
    eo.results_json::json->'config'->>'min_lockup_deposit' as min_lockup_deposit,
    eo.results_json::json->'config'->>'unlock_duration_ns' as unlock_duration_ns,
    eo.results_json::json->'config'->'guardians' as guardians,
    eo.results_json::json->'config'->'lockup_code_deployers' as lockup_code_deployers,
    eo.results_json::json->'config'->>'staking_pool_whitelist_account_id' as staking_pool_whitelist,
    -- Extract venear growth config
    eo.results_json::json->'venear_growth_config'->'annual_growth_rate_ns'->>'numerator' as growth_rate_numerator,
    eo.results_json::json->'venear_growth_config'->'annual_growth_rate_ns'->>'denominator' as growth_rate_denominator
FROM execution_outcomes eo
JOIN receipt_actions ra ON eo.receipt_id = ra.receipt_id
WHERE ra.method_name = 'new'
  AND eo.status = 'SuccessValue'
  AND eo.results_json IS NOT NULL
  AND eo.results_json != ''
ORDER BY eo.block_timestamp DESC;

-- 2. Query specific contract initialization by receipt ID
SELECT 
    eo.receipt_id,
    eo.block_height,
    eo.block_timestamp,
    ra.method_name,
    ra.receiver_id as contract_address,
    eo.results_json
FROM execution_outcomes eo
JOIN receipt_actions ra ON eo.receipt_id = ra.receipt_id
WHERE eo.receipt_id = '2cc8rV5qEeyLooJBTPYBW5dqNiSA8P2BQWgyyqhmPPUi'
  AND ra.method_name = 'new'
  AND eo.status = 'SuccessValue';

-- 3. Query all execution outcome results with their associated receipt actions
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

-- 4. Query contract configurations by owner account
SELECT 
    eo.receipt_id,
    eo.block_timestamp,
    ra.receiver_id as contract_address,
    eo.results_json::json->'config'->>'owner_account_id' as owner_account_id,
    eo.results_json::json->'config'->>'local_deposit' as local_deposit,
    eo.results_json::json->'config'->>'min_lockup_deposit' as min_lockup_deposit
FROM execution_outcomes eo
JOIN receipt_actions ra ON eo.receipt_id = ra.receipt_id
WHERE ra.method_name = 'new'
  AND eo.results_json::json->'config'->>'owner_account_id' = 'owner.r-1748895584.testnet'
ORDER BY eo.block_timestamp DESC;

-- 5. Query all successful function calls with return values
SELECT 
    eo.receipt_id,
    eo.block_height,
    eo.block_timestamp,
    ra.method_name,
    ra.receiver_id,
    ra.signer_account_id,
    CASE 
        WHEN ra.method_name = 'new' THEN 'Contract Initialization'
        WHEN ra.method_name LIKE '%get_%' THEN 'Getter Function'
        WHEN ra.method_name LIKE '%view_%' THEN 'View Function'
        ELSE 'Other Function'
    END as function_type,
    eo.results_json
FROM execution_outcomes eo
JOIN receipt_actions ra ON eo.receipt_id = ra.receipt_id
WHERE eo.status = 'SuccessValue'
  AND eo.results_json IS NOT NULL
  AND eo.results_json != ''
ORDER BY eo.block_timestamp DESC
LIMIT 100;

-- ============================================================================
-- RECEIPT ACTION ARGUMENTS QUERIES
-- ============================================================================

-- 1. Get contract initialization arguments (method_name = 'new')
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

-- 2. Get all function call arguments with their associated receipt actions
SELECT 
    ra.receipt_id,
    ra.action_index,
    ra.block_height,
    ra.block_timestamp,
    ra.method_name,
    ra.receiver_id,
    ra.signer_account_id,
    ra.args_base64,
    ra.args_json,
    ra.gas,
    ra.deposit
FROM receipt_actions ra
WHERE ra.action_kind = 'FunctionCall'
  AND ra.args_json IS NOT NULL
ORDER BY ra.block_timestamp DESC;

-- 3. Query specific contract initialization by owner
SELECT 
    ra.receipt_id,
    ra.block_height,
    ra.block_timestamp,
    ra.method_name,
    ra.receiver_id as contract_address,
    ra.args_json
FROM receipt_actions ra
WHERE ra.method_name = 'new'
  AND ra.args_json::json->'config'->>'owner_account_id' = 'owner.r-1748895584.testnet'
ORDER BY ra.block_timestamp DESC;

-- 4. Compare arguments vs results for the same transaction
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

-- 5. Query all function call arguments with their associated receipt actions
SELECT 
    ra.receipt_id,
    ra.action_index,
    ra.block_height,
    ra.block_timestamp,
    ra.method_name,
    ra.receiver_id,
    ra.signer_account_id,
    ra.args_base64,
    ra.args_json,
    ra.gas,
    ra.deposit
FROM receipt_actions ra
WHERE ra.block_height = 201857082  -- Replace with your block height
ORDER BY ra.block_timestamp DESC; 