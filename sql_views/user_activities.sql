/*
 This is a dimensional table that returns all historical delegate/undelegate & stake/unstake events for a given user account; both successful and failed events are included. 
 Primary key is the receipt_id (base58 encoded) associated with any given event per account_id.
 Unique method_names referenced as an individual event: 
 (1) on_lockup_deployed, (2) lock_near, (3) on_lockup_update, (4) end_unlock_near, (5) delegate_all, (6) undelegate, (7) begin_unlock_near, (8) lock_pending_near
 
 1. Timestamp / date of the event
 2. House of Stake contract address 
 3. Event Type / Method Name 
 4. Event Status (one of succeeded or failed)
 5. Account ID (the user performing the NEAR delegation) 
 6. Near amount 
 7. Locked near balance 
 8. The block-related data for the event (block hash, block height) 
 */

CREATE VIEW user_activities AS
WITH execution_outcomes_prep AS (
	SELECT
		SPLIT_PART(receipt_id, '-', 2) AS receipt_id
		, status
		, logs
	FROM execution_outcomes
)
, receipt_actions_prep AS (
	SELECT
		decode(ra.args_base64, 'base64') AS args_decoded
		, CASE 
			WHEN eo.status IN ('SuccessReceiptId', 'SuccessValue') THEN 'succeeded'
			WHEN eo.status IN ('Failure') THEN 'failed'
			ELSE NULL
			END AS event_status
		, eo.status                      AS status
		, eo.logs                        AS logs
		, ra.*
	FROM receipt_actions AS ra
	LEFT JOIN execution_outcomes_prep AS eo
		ON ra.receipt_id = eo.receipt_id
	WHERE
		ra.action_kind = 'FunctionCall'
)
--------------------
--Account Creation--
--------------------
, on_lockup_deployed AS (
  	SELECT
  		base58_encode(ra.receipt_id) AS id 
  		, base58_encode(ra.receipt_id) AS receipt_id
  		, ra.block_timestamp AS event_timestamp
  		, COALESCE(CASE 
 		    WHEN safe_json_parse(REPLACE(ra.logs[1], 'EVENT_JSON:', ''))->>'error' IS NULL
 		    THEN safe_json_parse(REPLACE(ra.logs[1], 'EVENT_JSON:', ''))->>'event'
 		    ELSE NULL 
 		  END, 'lockup_deployed') AS event_type
  		, ra.method_name 
  		, ra.event_status
    	, ra.signer_account_id AS account_id
    	, ra.predecessor_id AS hos_contract_address 
    	, CASE 
 		    WHEN safe_json_parse(CONVERT_FROM(DECODE(ra.args_base64, 'base64'), 'UTF8'))->>'error' IS NULL
 		    THEN (safe_json_parse(CONVERT_FROM(DECODE(ra.args_base64, 'base64'), 'UTF8'))->>'lockup_deposit')::NUMERIC
 		    ELSE NULL 
 		  END AS near_amount 
    	, CASE 
 		    WHEN safe_json_parse(REPLACE(ra.logs[1], 'EVENT_JSON:', ''))->>'error' IS NULL
 		    THEN (safe_json_parse(REPLACE(ra.logs[1], 'EVENT_JSON:', ''))->'data'->0->>'locked_near_balance')::NUMERIC
 		    ELSE NULL 
 		  END AS locked_near_balance --Field exists in the logs, but is ALWAYS NULL  
    	, ra.block_height
    	, base58_encode(ra.block_hash) AS block_hash
  	FROM receipt_actions_prep AS ra
  	WHERE
    	ra.method_name = 'on_lockup_deployed'
		AND ra.receiver_id IN (          
 			'v.r-1748895584.testnet'  
 			, 'vote.r-1748895584.testnet' 
 			)
)
-------------
--Lock NEAR--
-------------
, lock_near AS (
  	SELECT
  		base58_encode(ra.receipt_id) AS id 
  		, base58_encode(ra.receipt_id) AS receipt_id
  		, ra.block_timestamp AS event_timestamp
  		, COALESCE(CASE 
 		    WHEN safe_json_parse(REPLACE(ra.logs[1], 'EVENT_JSON:', ''))->>'error' IS NULL
 		    THEN safe_json_parse(REPLACE(ra.logs[1], 'EVENT_JSON:', ''))->>'event'
 		    ELSE NULL 
 		  END, 'lockup_lock_near') AS event_type --COALESCE required WHEN log IS failed; RETURNS NULL otherwise 
  		, ra.method_name 
  		, ra.event_status
    	, ra.signer_account_id AS account_id
    	, SUBSTRING(ra.receiver_id FROM POSITION('.' IN ra.receiver_id) + 1) AS hos_contract_address 
		, CASE 
 		    WHEN safe_json_parse(CONVERT_FROM(DECODE(ra.args_base64, 'base64'), 'UTF8'))->>'error' IS NULL
 		    THEN (safe_json_parse(CONVERT_FROM(DECODE(ra.args_base64, 'base64'), 'UTF8'))->>'amount')::NUMERIC
 		    ELSE NULL 
 		  END AS near_amount 
  		, CASE 
 		    WHEN safe_json_parse(REPLACE(ra.logs[1], 'EVENT_JSON:', ''))->>'error' IS NULL
 		    THEN (safe_json_parse(REPLACE(ra.logs[1], 'EVENT_JSON:', ''))->'data'->0->>'locked_near_balance')::NUMERIC
 		    ELSE NULL 
 		  END AS locked_near_balance 
    	, ra.block_height
    	, base58_encode(ra.block_hash) AS block_hash
  	FROM receipt_actions_prep AS ra
  	WHERE
    	ra.method_name = 'lock_near'
		AND SUBSTRING(ra.receiver_id FROM POSITION('.' IN ra.receiver_id) + 1) IN (
 			'v.r-1748895584.testnet'  
 			, 'vote.r-1748895584.testnet' 
 			)
)
, on_lockup_update_prep AS (
    --There are 2 event_json arrays per on_lockup_update method; 1st event is on_lockup_update; 2nd is ft_mint. 
	-- Single pass over logs array to extract all needed values
    SELECT 
    	base58_encode(receipt_id) AS id
    	, base58_encode(receipt_id) AS receipt_id 
        , ra.block_timestamp AS event_timestamp
        , ra.method_name 
        , ra.event_status 
        , ra.signer_account_id AS account_id 
        , ra.receiver_id AS hos_contract_address 
        , base58_encode(ra.block_hash) AS block_hash 
        , ra.block_height
        -- Extract event type (ft_mint or ft_burn)
        , MAX(CASE 
            WHEN safe_json_parse(REPLACE(log, 'EVENT_JSON:', ''))->>'error' IS NULL
            AND safe_json_parse(REPLACE(log, 'EVENT_JSON:', ''))->>'event' IN ('ft_mint', 'ft_burn') 
            THEN safe_json_parse(REPLACE(log, 'EVENT_JSON:', ''))->>'event' 
        	END) AS ft_event_type
        -- Extract locked_near_balance from lockup_update event
        , MAX(CASE 
            WHEN safe_json_parse(REPLACE(log, 'EVENT_JSON:', ''))->>'error' IS NULL
            AND safe_json_parse(REPLACE(log, 'EVENT_JSON:', ''))->>'event' = 'lockup_update' 
            THEN (safe_json_parse(REPLACE(log, 'EVENT_JSON:', ''))->'data'->0->>'locked_near_balance')::NUMERIC 
        	END) AS locked_near_balance
        -- Extract amount from ft_mint or ft_burn event
        , MAX(CASE 
            WHEN safe_json_parse(REPLACE(log, 'EVENT_JSON:', ''))->>'error' IS NULL
            AND safe_json_parse(REPLACE(log, 'EVENT_JSON:', ''))->>'event' IN ('ft_mint', 'ft_burn') 
            THEN (safe_json_parse(REPLACE(log, 'EVENT_JSON:', ''))->'data'->0->>'amount')::NUMERIC 
        	END) AS near_amount
    FROM receipt_actions_prep AS ra
    CROSS JOIN LATERAL UNNEST(ra.logs) AS log
    WHERE 
    	ra.method_name = 'on_lockup_update'
        AND ra.receiver_id IN (        
 			'v.r-1748895584.testnet'  
 			, 'vote.r-1748895584.testnet' 
 			)
    GROUP BY 1,2,3,4,5,6,7,8,9
)
, on_lockup_update AS (
    SELECT
    	id
    	, receipt_id
    	, event_timestamp
    	, COALESCE(method_name || '_' || ft_event_type, method_name) AS event_type
        , method_name 
    	, event_status 
    	, account_id 
    	, hos_contract_address
    	, near_amount 
    	, locked_near_balance 
    	, block_height 
    	, block_hash
    FROM on_lockup_update_prep
)
 -------------------------------
 --Delegations / Undelegations--
 -------------------------------
 , delegations_undelegations AS (
   	SELECT
   		MD5(CONCAT(base58_encode(ra.receipt_id), '_',  	
 			CASE 
 		    WHEN safe_json_parse(REPLACE(unnested_logs, 'EVENT_JSON:', ''))->>'error' IS NULL
 		    THEN safe_json_parse(REPLACE(unnested_logs, 'EVENT_JSON:', ''))->'data'->0->>'owner_id'
 		    ELSE NULL 
 		  END)) AS id 
  		, base58_encode(ra.receipt_id) AS receipt_id
  		, ra.block_timestamp AS event_timestamp
  		, COALESCE(ra.method_name || '_' || CASE 
 		    WHEN safe_json_parse(REPLACE(unnested_logs, 'EVENT_JSON:', ''))->>'error' IS NULL
 		    THEN safe_json_parse(REPLACE(unnested_logs, 'EVENT_JSON:', ''))->>'event'
 		    ELSE NULL 
 		  END, ra.method_name) AS event_type 
  		, ra.method_name 
  		, ra.event_status
    	, COALESCE(CASE 
 		    WHEN safe_json_parse(REPLACE(unnested_logs, 'EVENT_JSON:', ''))->>'error' IS NULL
 		    THEN safe_json_parse(REPLACE(unnested_logs, 'EVENT_JSON:', ''))->'data'->0->>'owner_id'
 		    ELSE NULL 
 		  END, ra.signer_account_id) AS account_id
    	, ra.receiver_id AS hos_contract_address 
	    , CASE 
 		    WHEN safe_json_parse(REPLACE(unnested_logs, 'EVENT_JSON:', ''))->>'error' IS NULL
 		    THEN (safe_json_parse(REPLACE(unnested_logs, 'EVENT_JSON:', ''))->'data'->0->>'amount')::NUMERIC
 		    ELSE NULL 
 		  END AS near_amount
	    , NULL::NUMERIC AS locked_near_balance --This does NOT exist FOR delegate_all AND undelegate events 
    	, ra.block_height
    	, base58_encode(ra.block_hash) AS block_hash 
  	FROM receipt_actions_prep AS ra
  	LEFT JOIN LATERAL UNNEST(ra.logs) AS unnested_logs 
 		ON TRUE
  	WHERE
    	ra.method_name IN ('delegate_all', 'undelegate')
		AND ra.receiver_id IN (           
 			'v.r-1748895584.testnet'  
 			, 'vote.r-1748895584.testnet' 
 			)
)
---------------------
--Begin Unlock NEAR--
---------------------
, begin_unlock_near AS ( 
  	SELECT 
  		base58_encode(ra.receipt_id) AS id 
  		, base58_encode(ra.receipt_id) AS receipt_id
  		, ra.block_timestamp AS event_timestamp
  		, method_name AS event_type 
  		, ra.method_name 
  		, ra.event_status
    	, ra.signer_account_id AS account_id
    	, SUBSTRING(ra.receiver_id FROM POSITION('.' IN ra.receiver_id) + 1) AS hos_contract_address 
		, CASE 
 		    WHEN safe_json_parse(CONVERT_FROM(DECODE(ra.args_base64, 'base64'), 'UTF8'))->>'error' IS NULL
 		    THEN (safe_json_parse(CONVERT_FROM(DECODE(ra.args_base64, 'base64'), 'UTF8'))->>'amount')::NUMERIC
 		    ELSE NULL 
 		  END AS near_amount 
		, NULL::NUMERIC AS locked_near_balance --There ARE NO logs FOR this event_type
        , ra.block_height
    	, base58_encode(ra.block_hash) AS block_hash
  	FROM receipt_actions_prep AS ra
  	WHERE
    	ra.method_name = 'begin_unlock_near'
		AND SUBSTRING(ra.receiver_id FROM POSITION('.' IN ra.receiver_id) + 1) IN (           
 			'v.r-1748895584.testnet'  
 			, 'vote.r-1748895584.testnet' 
 			)
 )
------------------------
--Re-Lock Pending NEAR--
------------------------
, relock_pending_near AS ( 
  	SELECT
  		base58_encode(ra.receipt_id) AS id 
  		, base58_encode(ra.receipt_id) AS receipt_id
  		, ra.block_timestamp AS event_timestamp
  		, method_name AS event_type 
  		, ra.method_name 
  		, ra.event_status
    	, ra.signer_account_id AS account_id
    	, SUBSTRING(ra.receiver_id FROM POSITION('.' IN ra.receiver_id) + 1) AS hos_contract_address 
		, CASE 
 		    WHEN safe_json_parse(CONVERT_FROM(DECODE(ra.args_base64, 'base64'), 'UTF8'))->>'error' IS NULL
 		    THEN (safe_json_parse(CONVERT_FROM(DECODE(ra.args_base64, 'base64'), 'UTF8'))->>'amount')::NUMERIC
 		    ELSE NULL 
 		  END AS near_amount 
		, NULL::NUMERIC AS locked_near_balance --There ARE NO logs FOR this event_type
    	, ra.block_height
    	, base58_encode(ra.block_hash) AS block_hash
  	FROM receipt_actions_prep AS ra
  	WHERE
    	ra.method_name = 'lock_pending_near'
		AND SUBSTRING(ra.receiver_id FROM POSITION('.' IN ra.receiver_id) + 1) IN (           
 			'v.r-1748895584.testnet'  
 			, 'vote.r-1748895584.testnet' 
 			)
 )
 ----------
 --UNIONS--
 ----------
 , unioned_events AS (
 	SELECT * FROM on_lockup_deployed
 		UNION ALL 
 	SELECT * FROM lock_near
 		UNION ALL 
 	SELECT * FROM on_lockup_update
 		UNION ALL 
 	SELECT * FROM delegations_undelegations
 		UNION ALL 
 	SELECT * FROM begin_unlock_near
 	 	UNION ALL 
 	SELECT * FROM relock_pending_near
)
 SELECT 
 	id 
 	, receipt_id 
 	, hos_contract_address 
 	, account_id
 	, DATE(event_timestamp) AS event_date 
 	, event_timestamp 
    , method_name 
 	, event_type 
 	, event_status 
 	, near_amount
 	, locked_near_balance
 	, block_height
 	, block_hash
 FROM unioned_events 
 ORDER BY account_id ASC, event_timestamp ASC
 ;