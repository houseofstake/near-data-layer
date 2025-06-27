-- Example queries for execution outcome results

-- 1. Query contract initialization results (method_name = 'new')
SELECT 
    eo.receipt_id,
    eo.block_height,
    eo.block_timestamp,
    ra.method_name,
    ra.receiver_id as contract_address,
    ra.signer_account_id as deployer,
    eo.result_json,
    -- Extract specific config values
    eo.result_json::json->'config'->>'owner_account_id' as owner_account_id,
    eo.result_json::json->'config'->>'local_deposit' as local_deposit,
    eo.result_json::json->'config'->>'min_lockup_deposit' as min_lockup_deposit,
    eo.result_json::json->'config'->>'unlock_duration_ns' as unlock_duration_ns,
    eo.result_json::json->'config'->'guardians' as guardians,
    eo.result_json::json->'config'->'lockup_code_deployers' as lockup_code_deployers,
    eo.result_json::json->'config'->>'staking_pool_whitelist_account_id' as staking_pool_whitelist,
    -- Extract venear growth config
    eo.result_json::json->'venear_growth_config'->'annual_growth_rate_ns'->>'numerator' as growth_rate_numerator,
    eo.result_json::json->'venear_growth_config'->'annual_growth_rate_ns'->>'denominator' as growth_rate_denominator
FROM execution_outcomes eo
JOIN receipt_actions ra ON eo.receipt_id = ra.receipt_id
WHERE ra.method_name = 'new'
  AND eo.status = 'SuccessValue'
  AND eo.result_json IS NOT NULL
  AND eo.result_json != ''
ORDER BY eo.block_timestamp DESC;

-- 2. Query specific contract initialization by receipt ID
SELECT 
    eo.receipt_id,
    eo.block_height,
    eo.block_timestamp,
    ra.method_name,
    ra.receiver_id as contract_address,
    eo.result_json
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
    eo.result_value,
    eo.result_json,
    ra.method_name,
    ra.receiver_id
FROM execution_outcomes eo
JOIN receipt_actions ra ON eo.receipt_id = ra.receipt_id
WHERE eo.status = 'SuccessValue'
  AND eo.result_json IS NOT NULL
  AND eo.result_json != ''
ORDER BY eo.block_timestamp DESC;

-- 4. Query contract configurations by owner account
SELECT 
    eo.receipt_id,
    eo.block_timestamp,
    ra.receiver_id as contract_address,
    eo.result_json::json->'config'->>'owner_account_id' as owner_account_id,
    eo.result_json::json->'config'->>'local_deposit' as local_deposit,
    eo.result_json::json->'config'->>'min_lockup_deposit' as min_lockup_deposit
FROM execution_outcomes eo
JOIN receipt_actions ra ON eo.receipt_id = ra.receipt_id
WHERE ra.method_name = 'new'
  AND eo.result_json::json->'config'->>'owner_account_id' = 'owner.r-1748895584.testnet'
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
    eo.result_json
FROM execution_outcomes eo
JOIN receipt_actions ra ON eo.receipt_id = ra.receipt_id
WHERE eo.status = 'SuccessValue'
  AND eo.result_json IS NOT NULL
  AND eo.result_json != ''
ORDER BY eo.block_timestamp DESC
LIMIT 100;

-- ============================================================================
-- RECEIPT ACTION ARGUMENTS QUERIES
-- ============================================================================

-- 6. Query contract initialization arguments (method_name = 'new')
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

-- 7. Query specific contract initialization arguments by receipt ID
SELECT 
    raa.receipt_id,
    raa.block_height,
    raa.block_timestamp,
    raa.method_name,
    raa.receiver_id as contract_address,
    raa.args_json
FROM receipt_action_arguments raa
WHERE raa.receipt_id = '2cc8rV5qEeyLooJBTPYBW5dqNiSA8P2BQWgyyqhmPPUi'
  AND raa.method_name = 'new';

-- 8. Query contract initialization arguments by owner account
SELECT 
    raa.receipt_id,
    raa.block_timestamp,
    raa.receiver_id as contract_address,
    raa.args_json::json->'config'->>'owner_account_id' as owner_account_id,
    raa.args_json::json->'config'->>'local_deposit' as local_deposit,
    raa.args_json::json->'config'->>'min_lockup_deposit' as min_lockup_deposit
FROM receipt_action_arguments raa
WHERE raa.method_name = 'new'
  AND raa.args_json::json->'config'->>'owner_account_id' = 'owner.r-1748895584.testnet'
ORDER BY raa.block_timestamp DESC;

-- 9. Compare arguments vs results for the same transaction
SELECT 
    raa.receipt_id,
    raa.method_name,
    raa.receiver_id as contract_address,
    raa.args_json as input_arguments,
    eo.result_json as output_results
FROM receipt_action_arguments raa
LEFT JOIN execution_outcomes eo ON raa.receipt_id = eo.receipt_id
WHERE raa.method_name = 'new'
  AND raa.args_json IS NOT NULL
ORDER BY raa.block_timestamp DESC;

-- 10. Query all function call arguments with their associated receipt actions
SELECT 
    raa.receipt_id,
    raa.action_index,
    raa.block_height,
    raa.block_timestamp,
    raa.method_name,
    raa.receiver_id,
    raa.signer_account_id,
    raa.args_base64,
    raa.args_json,
    raa.gas,
    raa.deposit
FROM receipt_action_arguments raa
WHERE raa.block_height = 201857082  -- Replace with your block height
ORDER BY raa.block_timestamp DESC; 