/*
 This is a dimensional table that returns all historical delegate/undelegate & stake/unstake events for a given user account, both successful and failed events are included 
 Primary key is the receipt_id (base58 encoded) associated with any given event per account_id
 Unique method_names referenced as an individual event: 
 (1) on_lockup_deployed, (2) lock_near, (3) on_lockup_update, (4) end_unlock_near, (5) delegate_all, (6) undelegate, (7) begin_unlock_near, (8) lock_pending_near
 
 1. Timestamp or date of the event
 2. House of Stake contract address 
 3. Event Type or Method Name 
 4. Event Status (one of succeeded or failed)
 5. Account ID (the user performing the NEAR delegation) 
 6. Near amount 
 7. Locked near balance 
 8. The block-related data for the event (block hash, block height) 
*/

DROP VIEW IF EXISTS {SCHEMA_NAME}.user_activities CASCADE;
CREATE VIEW {SCHEMA_NAME}.user_activities AS
WITH receipt_actions_prep AS (
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
	FROM {SCHEMA_NAME}.receipt_actions AS ra
	LEFT JOIN {SCHEMA_NAME}.execution_outcomes AS eo
		ON ra.receipt_id = eo.receipt_id
	WHERE
		ra.action_kind = 'FunctionCall'
		AND ra.method_name IN ('on_lockup_deployed', 'lock_near', 'on_lockup_update', 'delegate_all', 'undelegate', 'begin_unlock_near', 'lock_pending_near', 'withdraw_from_staking_pool', 'withdraw_all_from_staking_pool', 'unstake', 'unstake_all')
)
--------------------
--Account Creation--
--------------------
, on_lockup_deployed AS (
  	SELECT
  		ra.receipt_id AS id 
  		, ra.receipt_id
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
 		    WHEN safe_json_parse(CONVERT_FROM(ra.args_decoded, 'UTF8'))->>'error' IS NULL
 		    THEN (safe_json_parse(CONVERT_FROM(ra.args_decoded, 'UTF8'))->>'lockup_deposit')::NUMERIC
 		    ELSE NULL 
 		  END AS near_amount 
    	, CASE 
 		    WHEN safe_json_parse(REPLACE(ra.logs[1], 'EVENT_JSON:', ''))->>'error' IS NULL
 		    THEN (safe_json_parse(REPLACE(ra.logs[1], 'EVENT_JSON:', ''))->'data'->0->>'locked_near_balance')::NUMERIC
 		    ELSE NULL 
 		  END AS locked_near_balance --Field exists in the logs, but is ALWAYS NULL  
    	, ra.block_height
    	, ra.block_hash
  	FROM receipt_actions_prep AS ra
  	WHERE
    	ra.method_name = 'on_lockup_deployed'
		AND ra.receiver_id IN (     
			'{VENEAR_CONTRACT_PREFIX}.{HOS_CONTRACT}'
			, '{VOTING_CONTRACT_PREFIX}.{HOS_CONTRACT}'
 			)
)
-------------
--Lock NEAR--
-------------
, lock_near AS (
  	SELECT
  		ra.receipt_id AS id 
  		, ra.receipt_id
  		, ra.block_timestamp AS event_timestamp
  		, COALESCE(CASE 
 		    WHEN safe_json_parse(REPLACE(ra.logs[1], 'EVENT_JSON:', ''))->>'error' IS NULL
 		    THEN safe_json_parse(REPLACE(ra.logs[1], 'EVENT_JSON:', ''))->>'event'
 		    ELSE NULL 
 		  END, 'lockup_lock_near') AS event_type --COALESCE required WHEN log IS failed, RETURNS NULL otherwise 
  		, ra.method_name 
  		, ra.event_status
    	, ra.signer_account_id AS account_id
    	, SUBSTRING(ra.receiver_id FROM POSITION('.' IN ra.receiver_id) + 1) AS hos_contract_address 
 	    , CASE 
 		    WHEN safe_json_parse(CONVERT_FROM(ra.args_decoded, 'UTF8'))->>'error' IS NULL
 		    THEN (safe_json_parse(CONVERT_FROM(ra.args_decoded, 'UTF8'))->>'amount')::NUMERIC
 		    ELSE NULL 
 		  END AS near_amount 
  		, CASE 
 		    WHEN safe_json_parse(REPLACE(ra.logs[1], 'EVENT_JSON:', ''))->>'error' IS NULL
 		    THEN (safe_json_parse(REPLACE(ra.logs[1], 'EVENT_JSON:', ''))->'data'->0->>'locked_near_balance')::NUMERIC
 		    ELSE NULL 
 		  END AS locked_near_balance 
    	, ra.block_height
    	, ra.block_hash
  	FROM receipt_actions_prep AS ra
  	WHERE
    	ra.method_name = 'lock_near'
		AND SUBSTRING(ra.receiver_id FROM POSITION('.' IN ra.receiver_id) + 1) IN (   
			'{VENEAR_CONTRACT_PREFIX}.{HOS_CONTRACT}'
			, '{VOTING_CONTRACT_PREFIX}.{HOS_CONTRACT}'
 			)
)
, on_lockup_update_prep AS (
    --There are 2 event_json arrays per on_lockup_update method, 1st event is on_lockup_update, 2nd is ft_mint. 
	-- Single pass over logs array to extract all needed values
    SELECT 
    	ra.receipt_id AS id
    	, ra.receipt_id
        , ra.block_timestamp AS event_timestamp
        , ra.method_name 
        , ra.event_status 
        , ra.signer_account_id AS account_id 
        , ra.receiver_id AS hos_contract_address 
        , ra.block_hash AS block_hash 
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
			'{VENEAR_CONTRACT_PREFIX}.{HOS_CONTRACT}'
			, '{VOTING_CONTRACT_PREFIX}.{HOS_CONTRACT}'
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
 --Delegations and Undelegations--
 -------------------------------
 , delegations_undelegations AS (
   	SELECT
   		MD5(CONCAT(ra.receipt_id, '_',  	
 			CASE 
 		    WHEN safe_json_parse(REPLACE(unnested_logs, 'EVENT_JSON:', ''))->>'error' IS NULL
 		    THEN safe_json_parse(REPLACE(unnested_logs, 'EVENT_JSON:', ''))->'data'->0->>'owner_id'
 		    ELSE NULL 
 		  END)) AS id 
  		, ra.receipt_id
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
    	, ra.block_hash
  	FROM receipt_actions_prep AS ra
  	LEFT JOIN LATERAL UNNEST(ra.logs) AS unnested_logs 
 		ON TRUE
  	WHERE
    	ra.method_name IN ('delegate_all', 'undelegate')
		AND ra.receiver_id IN (     
			'{VENEAR_CONTRACT_PREFIX}.{HOS_CONTRACT}'
			, '{VOTING_CONTRACT_PREFIX}.{HOS_CONTRACT}'
 			)
)
---------------------
--Begin Unlock NEAR--
---------------------
, begin_unlock_near AS ( 
  	SELECT 
  		ra.receipt_id AS id 
  		, ra.receipt_id
  		, ra.block_timestamp AS event_timestamp
  		, method_name AS event_type 
  		, ra.method_name 
  		, ra.event_status
    	, ra.signer_account_id AS account_id
    	, SUBSTRING(ra.receiver_id FROM POSITION('.' IN ra.receiver_id) + 1) AS hos_contract_address 
		, CASE 
 		    WHEN safe_json_parse(CONVERT_FROM(ra.args_decoded, 'UTF8'))->>'error' IS NULL
 		    THEN (safe_json_parse(CONVERT_FROM(ra.args_decoded, 'UTF8'))->>'amount')::NUMERIC
 		    ELSE NULL 
 		  END AS near_amount 
		, NULL::NUMERIC AS locked_near_balance --There ARE NO logs FOR this event_type
        , ra.block_height
    	, ra.block_hash
  	FROM receipt_actions_prep AS ra
  	WHERE
    	ra.method_name = 'begin_unlock_near'
		AND SUBSTRING(ra.receiver_id FROM POSITION('.' IN ra.receiver_id) + 1) IN (   
			'{VENEAR_CONTRACT_PREFIX}.{HOS_CONTRACT}'
			, '{VOTING_CONTRACT_PREFIX}.{HOS_CONTRACT}'
 			)
 )
------------------------
--Re-Lock Pending NEAR--
------------------------
, relock_pending_near AS ( 
  	SELECT
  		ra.receipt_id AS id 
  		, ra.receipt_id
  		, ra.block_timestamp AS event_timestamp
  		, method_name AS event_type 
  		, ra.method_name 
  		, ra.event_status
    	, ra.signer_account_id AS account_id
    	, SUBSTRING(ra.receiver_id FROM POSITION('.' IN ra.receiver_id) + 1) AS hos_contract_address 
		, CASE 
 		    WHEN safe_json_parse(CONVERT_FROM(ra.args_decoded, 'UTF8'))->>'error' IS NULL
 		    THEN (safe_json_parse(CONVERT_FROM(ra.args_decoded, 'UTF8'))->>'amount')::NUMERIC
 		    ELSE NULL 
 		  END AS near_amount 
		, NULL::NUMERIC AS locked_near_balance --There ARE NO logs FOR this event_type
    	, ra.block_height
    	, ra.block_hash
  	FROM receipt_actions_prep AS ra
  	WHERE
    	ra.method_name = 'lock_pending_near'
		AND SUBSTRING(ra.receiver_id FROM POSITION('.' IN ra.receiver_id) + 1) IN (   
			'{VENEAR_CONTRACT_PREFIX}.{HOS_CONTRACT}'
			, '{VOTING_CONTRACT_PREFIX}.{HOS_CONTRACT}'
 			)
 )
----------------------------
--Withdraw From Staking Pool--
----------------------------
, withdraw_from_staking_pool AS ( 
  	SELECT
  		ra.receipt_id AS id 
  		, ra.receipt_id
  		, ra.block_timestamp AS event_timestamp
  		, method_name AS event_type 
  		, ra.method_name 
  		, ra.event_status
    	, ra.signer_account_id AS account_id
    	, SUBSTRING(ra.receiver_id FROM POSITION('.' IN ra.receiver_id) + 1) AS hos_contract_address 
		, COALESCE(
            CASE 
                WHEN safe_json_parse(CONVERT_FROM(ra.args_decoded, 'UTF8'))->>'error' IS NULL
                THEN (safe_json_parse(CONVERT_FROM(ra.args_decoded, 'UTF8'))->>'amount')::NUMERIC
                ELSE NULL
            END,
            CASE 
                WHEN safe_json_parse(REPLACE(unnested_logs, 'EVENT_JSON:', ''))->>'error' IS NULL
                THEN (safe_json_parse(REPLACE(unnested_logs, 'EVENT_JSON:', ''))->'data'->0->>'amount')::NUMERIC
                ELSE NULL 
            END,
            0
        ) AS near_amount 
		, NULL::NUMERIC AS locked_near_balance
    	, ra.block_height
    	, ra.block_hash
  	FROM receipt_actions_prep AS ra
    LEFT JOIN LATERAL UNNEST(ra.logs) AS unnested_logs ON TRUE
  	WHERE
    	ra.method_name IN ('withdraw_from_staking_pool', 'withdraw_all_from_staking_pool')
        AND ra.event_status = 'succeeded'
		AND SUBSTRING(ra.receiver_id FROM POSITION('.' IN ra.receiver_id) + 1) IN (   
			'{VENEAR_CONTRACT_PREFIX}.{HOS_CONTRACT}'
			, '{VOTING_CONTRACT_PREFIX}.{HOS_CONTRACT}'
 			)
 )
-----------
--Unstake--
-----------
, unstake AS ( 
  	SELECT
  		ra.receipt_id AS id 
  		, ra.receipt_id
  		, ra.block_timestamp AS event_timestamp
  		, method_name AS event_type 
  		, ra.method_name 
  		, ra.event_status
    	, ra.signer_account_id AS account_id
    	, SUBSTRING(ra.receiver_id FROM POSITION('.' IN ra.receiver_id) + 1) AS hos_contract_address 
		, COALESCE(
            CASE 
                WHEN safe_json_parse(CONVERT_FROM(ra.args_decoded, 'UTF8'))->>'error' IS NULL
                THEN (safe_json_parse(CONVERT_FROM(ra.args_decoded, 'UTF8'))->>'amount')::NUMERIC
                ELSE NULL
            END,
            CASE 
                WHEN safe_json_parse(REPLACE(unnested_logs, 'EVENT_JSON:', ''))->>'error' IS NULL
                THEN (safe_json_parse(REPLACE(unnested_logs, 'EVENT_JSON:', ''))->'data'->0->>'amount')::NUMERIC
                ELSE NULL 
            END,
            0
        ) AS near_amount 
		, NULL::NUMERIC AS locked_near_balance
    	, ra.block_height
    	, ra.block_hash
  	FROM receipt_actions_prep AS ra
    LEFT JOIN LATERAL UNNEST(ra.logs) AS unnested_logs ON TRUE
  	WHERE
    	ra.method_name IN ('unstake', 'unstake_all')
        AND ra.event_status = 'succeeded'
		AND SUBSTRING(ra.receiver_id FROM POSITION('.' IN ra.receiver_id) + 1) IN (   
			'{VENEAR_CONTRACT_PREFIX}.{HOS_CONTRACT}'
			, '{VOTING_CONTRACT_PREFIX}.{HOS_CONTRACT}'
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
        UNION ALL
    SELECT * FROM withdraw_from_staking_pool
        UNION ALL
    SELECT * FROM unstake
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
