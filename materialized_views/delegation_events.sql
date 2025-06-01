/*
 Primary key on this table is the receipt_id (base58 encoded) + delegatee_id associated with the most recent unique delegate_all or undelegate action per delegator_id. 
 This is a dimensional table that returns, for each unique delegate_all / undelegate event, the following: 
 
 1. Timestamp / date of the delegate/undelegate action 
 2. House of Stake contract address 
 3. Delegate Method Type (one of delegate_all or undelegate)
 4. Delegation Event Type (one of ft_burn or ft_mint) 
 5. Boolean indicating whether or not a given delegate_all or undelegate event per delegator_id was the most recent (is_latest_delegator_event)
 6. Delegator ID (the user performing the NEAR delegation) 
 7. Delegatee ID (the users who are receiving the delegated NEAR; only populated when delegate_method = 'delegate_all') 
 8. Owner ID (For ft_mint events, this is the user who is receiving the delegated NEAR; for ft_burn events, this is the user who is burning/delegating away the delegated NEAR)
 9. The amount of near that was delegated 
 10. The block-related data for the delegate_all or undelegate event (block hash/id, block height) 
 */

--Create the materialized view
CREATE MATERIALIZED VIEW delegation_events AS 
WITH execution_outcomes_prep AS (
	SELECT
 		SPLIT_PART(receipt_id, '-', 2) AS receipt_id
 		, status
 		, logs
 	FROM execution_outcomes 
)
, receipt_actions_prep AS (
	SELECT
 		decode(ra.args_base64, 'base64') AS args
 		, eo.status 					 
 		, eo.logs 						 
 		, ra.*
 	FROM receipt_actions AS ra
 	INNER JOIN execution_outcomes_prep AS eo
 		ON ra.receipt_id = eo.receipt_id
 		AND eo.status IN ('SuccessReceiptId', 'SuccessValue')
 	WHERE
 		ra.action_kind = 'FunctionCall'
 		AND ra.receiver_id IN (           --House of Stake contracts
 			'v.r-1745564650.testnet'      --veNEAR contract
 			, 'vote.r-1745564650.testnet' --Voting contract
 			)
)
, delegate_undelegate_events AS (
	SELECT 
		ra.*
		, ROW_NUMBER() OVER (PARTITION BY ra.predecessor_id ORDER BY ra.block_timestamp DESC) AS row_num 
	FROM receipt_actions_prep AS ra
	WHERE 
		ra.method_name IN ('delegate_all', 'undelegate')
)
SELECT
	MD5(CONCAT(base58_encode(ra.receipt_id), '_',  	
 		REPLACE(unnested_logs, 'EVENT_JSON:', '')::json->'data'->0->>'owner_id')) AS id 
 	, base58_encode(ra.receipt_id) AS receipt_id
 	, DATE(ra.block_timestamp) AS event_date
 	, ra.block_timestamp AS event_timestamp
 	, ra.receiver_id AS hos_contract_address 
 	, ra.predecessor_id AS delegator_id 
 	, (CONVERT_FROM(ra.args, 'UTF8')::json->>'receiver_id') AS delegatee_id --null for the undelegate event 
 	, ra.method_name AS delegate_method
	, REPLACE(unnested_logs, 'EVENT_JSON:', '')::json->>'event' AS delegate_event 
	, CASE 
 	 	WHEN row_num = 1 
 	 	THEN TRUE 
 	 	ELSE FALSE END AS is_latest_delegator_event 
	, REPLACE(unnested_logs, 'EVENT_JSON:', '')::json->'data'->0->>'owner_id' AS owner_id
	, (REPLACE(unnested_logs, 'EVENT_JSON:', '')::json->'data'->0->>'amount')::NUMERIC AS near_amount
		
	--Block Data 
	, ra.block_height
 	, base58_encode(ra.block_hash) AS block_hash
 FROM delegate_undelegate_events AS ra
 LEFT JOIN LATERAL UNNEST(ra.logs) AS unnested_logs 
 	ON TRUE
 ORDER BY ra.block_timestamp DESC
 WITH DATA
;

--Create the unique index for the view 
CREATE UNIQUE INDEX delegation_events_id_idx ON delegation_events (id);

--Create the cron schedule
SELECT cron.schedule(
    'refresh_delegation_events', 
    '* * * * *',                   -- every minute
    $$REFRESH MATERIALIZED VIEW CONCURRENTLY delegation_events;$$
);

--Pause the cron schedule 
SELECT cron.alter_job(7, active := false);