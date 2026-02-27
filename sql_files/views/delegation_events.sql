/*
 Primary key on this table is the receipt_id (base58 encoded) + delegatee_id associated with the most recent unique delegate_all or undelegate action per delegator_id. 
 This is a dimensional table that returns, for each unique delegate_all or undelegate event, the following: 
 
 1. Timestamp or date of the delegate/undelegate action 
 2. House of Stake contract address 
 3. Delegate Method Type (one of delegate_all or undelegate)
 4. Delegation Event Type (one of ft_burn or ft_mint) 
 5. Boolean indicating whether or not a given delegate_all or undelegate event per delegator_id was the most recent (is_latest_delegator_event)
 6. Delegator ID (the user performing the NEAR delegation) 
 7. Delegatee ID (the users who are receiving the delegated NEAR only, populated when delegate_method = 'delegate_all') 
 8. Owner ID (For ft_mint events, this is the user who is receiving the delegated NEAR, for ft_burn events, this is the user who is burning/delegating away the delegated NEAR)
 9. The amount of near that was delegated 
 10. The block-related data for the delegate_all or undelegate event (block hash or id, block height) 
 */

DROP VIEW IF EXISTS {SCHEMA_NAME}.delegation_events CASCADE;
CREATE VIEW {SCHEMA_NAME}.delegation_events AS 
WITH receipt_actions_prep AS (
	SELECT
 		decode(ra.args_base64, 'base64') AS args
 		, eo.status 					 
 		, eo.logs 						 
 		, ra.*
 	FROM {SCHEMA_NAME}.receipt_actions AS ra
 	INNER JOIN {SCHEMA_NAME}.execution_outcomes AS eo
 		ON ra.receipt_id = eo.receipt_id
 		AND eo.status IN ('SuccessReceiptId', 'SuccessValue')
 	WHERE
 		ra.action_kind = 'FunctionCall'
		AND ra.method_name IN ('delegate_all', 'undelegate')
		AND ra.receiver_id IN (     --House of Stake contracts
			'{VENEAR_CONTRACT_PREFIX}.{HOS_CONTRACT}'   --veNEAR contract
			, '{VOTING_CONTRACT_PREFIX}.{HOS_CONTRACT}' --Voting contract
 			)
)
, delegate_undelegate_events AS (
	SELECT 
		ra.*
		, ROW_NUMBER() OVER (PARTITION BY ra.predecessor_id ORDER BY ra.block_timestamp DESC) AS row_num 
	FROM receipt_actions_prep AS ra
)
SELECT
	MD5(CONCAT(ra.receipt_id, '_',  	
 		CASE 
 		    WHEN safe_json_parse(REPLACE(unnested_logs, 'EVENT_JSON:', ''))->>'error' IS NULL
 		    THEN safe_json_parse(REPLACE(unnested_logs, 'EVENT_JSON:', ''))->'data'->0->>'owner_id'
 		    ELSE NULL 
 		  END)) AS id 
 	, ra.receipt_id
 	, DATE(ra.block_timestamp) AS event_date
 	, ra.block_timestamp AS event_timestamp
 	, ra.receiver_id AS hos_contract_address 
 	, ra.predecessor_id AS delegator_id 
 	, CASE 
 		WHEN safe_json_parse(CONVERT_FROM(ra.args, 'UTF8'))->>'error' IS NULL
 		THEN safe_json_parse(CONVERT_FROM(ra.args, 'UTF8'))->>'receiver_id'
 		ELSE NULL 
 		END AS delegatee_id --null for the undelegate event 
 	, ra.method_name AS delegate_method
	, (
 		SELECT safe_json_parse(REPLACE(l, 'EVENT_JSON:', ''))->>'event'
 		FROM UNNEST(ra.logs) AS t(l)
 		WHERE safe_json_parse(REPLACE(l, 'EVENT_JSON:', ''))->>'error' IS NULL
 		  AND safe_json_parse(REPLACE(l, 'EVENT_JSON:', ''))->>'event' IN ('ft_mint', 'ft_burn')
 		LIMIT 1
 	  ) AS delegate_event 

	, CASE 
 	 	WHEN row_num = 1 
 	 	THEN TRUE 
 	 	ELSE FALSE END AS is_latest_delegator_event 

	, (
 		SELECT safe_json_parse(REPLACE(l, 'EVENT_JSON:', ''))->'data'->0->>'owner_id'
 		FROM UNNEST(ra.logs) AS t(l)
 		WHERE safe_json_parse(REPLACE(l, 'EVENT_JSON:', ''))->>'error' IS NULL
 		  AND safe_json_parse(REPLACE(l, 'EVENT_JSON:', ''))->>'event' = 'ft_mint'
 		LIMIT 1
 	  ) AS owner_id

	, (
 		SELECT (safe_json_parse(REPLACE(l, 'EVENT_JSON:', ''))->'data'->0->>'amount')::NUMERIC
 		FROM UNNEST(ra.logs) AS t(l)
 		WHERE safe_json_parse(REPLACE(l, 'EVENT_JSON:', ''))->>'error' IS NULL
 		  AND safe_json_parse(REPLACE(l, 'EVENT_JSON:', ''))->>'event' = 'ft_burn'
 		LIMIT 1
 	  ) AS near_amount
		
	--Block Data 
	, ra.block_height
 	, ra.block_hash
	, ra.block_timestamp
 FROM delegate_undelegate_events AS ra
 ORDER BY ra.block_timestamp DESC
;
