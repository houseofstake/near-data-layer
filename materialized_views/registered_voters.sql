/*
 Primary key on this table is receipt_id, with base58 encoding. 
 Every single row in this table is a unique, successful deploy_lockup action, which translates into a voter registration action.
 ("Successful" is defined by pulling only receipt_ids flagged as successful from the execution_outcomes table.)
 
 Each deploy_lockup action is associated with: 
   1. A registered voter ID                                           (The voter account; eg. lighttea2007.testnet) 
   2. The related House of Stake Contract                             (veNEAR contract address, v.r-1745564650.testnet)
   3. The timestamp at which the voter registration action occurred 
   4. The block-related data for this deploy_lockup action            (Block hash/id, block height) 
   5. The registerd voter's current voting power                      (Sourced from the execution_outcomes.logs value associated with the voter account's latest on_lockup_update event from receipt_actions)  
   6. The registered voter's initial voting power                     (Sourced from the execution_outcomes.logs value associated with the storage_deposit event that gets emitted upon vote registration) 
   7. The registered voter's proposal participation rate              (Calculated as a count of the vote_options - only considering the latest vote_option per proposal - a user makes on any of the 10 most recently approved proposals for the veNEAR contract; always a percentage out of 10)
*/

--Create the view
CREATE VIEW registered_voters AS
WITH
/* Sourcing Registered Voters */
execution_outcomes_prep AS (
	SELECT
		SPLIT_PART(receipt_id, '-', 2) AS receipt_id
		, status
		, logs
	FROM execution_outcomes
)
, receipt_actions_prep AS (
	SELECT
		decode(ra.args_base64, 'base64') AS args_decoded
		, eo.status                      AS action_status
		, eo.logs                        AS action_logs
		, ra.*
	FROM receipt_actions AS ra
	INNER JOIN execution_outcomes_prep AS eo
		ON ra.receipt_id = eo.receipt_id
		AND eo.status IN ('SuccessReceiptId', 'SuccessValue')
	WHERE
		ra.action_kind = 'FunctionCall'
    	AND ra.receiver_id IN (
 			'v.{{ venear_contract }}'      --veNEAR contract
 			, 'vote.{{ voting_contract }}' --Voting contract
 		)
)
, registered_voters_prep AS (
  	SELECT
    	decode(ra.args_base64, 'base64') AS args
    	, ra.*
  	FROM receipt_actions_prep AS ra
  	WHERE
    	ra.method_name = 'deploy_lockup'
)

/* Sourcing Voting Power per Registered Voter */
, initial_voting_power_from_locks_unlocks AS (
  	SELECT
  		ra.block_timestamp
  		, args_decoded
    	, base58_encode(ra.receipt_id) 																		            AS receipt_id
    	, COALESCE((REPLACE(ra.action_logs[1], 'EVENT_JSON:', '')::json->'data'->0->>'owner_id'), ra.signer_account_id) AS registered_voter_id
    	, (REPLACE(ra.action_logs[1], 'EVENT_JSON:', '')::json->'data'->0->>'amount')::NUMERIC 					        AS initial_voting_power
    	, ra.receiver_id 																				                AS hos_contract_address
    	, ra.block_height
    	, base58_encode(ra.block_hash) AS block_hash
  	FROM receipt_actions_prep AS ra
  	WHERE
    	ra.method_name = 'storage_deposit'
)
, current_voting_power_from_locks_unlocks AS (
	SELECT
		ra.block_timestamp
		, base58_encode(ra.receipt_id) 																	    			AS receipt_id
		, COALESCE(REPLACE(ra.action_logs[1], 'EVENT_JSON:', '')::json->'data'->0->>'account_id', ra.signer_account_id) AS registered_voter_id
		, (REPLACE(ra.action_logs[1], 'EVENT_JSON:', '')::json->'data'->0->>'locked_near_balance')::NUMERIC             AS current_voting_power_logs
    	, (convert_from(ra.args_decoded, 'UTF8')::json->'update'->'V1'->>'locked_near_balance')::NUMERIC                AS current_voting_power_args
    	, ra.receiver_id 																								AS hos_contract_address
    	, ra.block_height
    	, base58_encode(ra.block_hash) 																					AS block_hash																				
    	, ra.action_logs
    	, ROW_NUMBER() OVER (PARTITION BY signer_account_id ORDER BY block_timestamp DESC) 				                AS row_num
  	FROM receipt_actions_prep AS ra
  	WHERE
    	ra.method_name = 'on_lockup_update'
)
, actively_delegating_accounts AS (
  --List of accounts that are actively delegating right now & the accounts to which they are delegating ALL their voting power.
  --Note: Every time you delegate, you are delegating away ALL your power by default. However, this excludes amounts that are being delegated to you simultaneously; see below cte).
  --This info is important for calculating the total voting power from delegations on any given registered voter; it's the sum of voting power from others who are actively delgating to them! 
	SELECT DISTINCT 
		delegator_id 
		, delegatee_id
		, near_amount
	FROM delegation_events 
	WHERE 
		is_latest_delegator_event = TRUE 
		AND delegate_method = 'delegate_all'
		AND delegate_event = 'ft_mint'
)
, delegations_voting_power AS ( 
  --Total voting power delegated to your account, as a registered voter 
	SELECT 
 		delegatee_id 
 		, SUM(near_amount) AS delegations_voting_power 
 	FROM actively_delegating_accounts
 	GROUP BY 1
)

/* Sourcing Proposal Participation (From the 10 most recently approved proposals) */
, ten_most_recently_approved_proposals AS (
	SELECT
		*
	FROM approved_proposals
	ORDER BY proposal_approved_at DESC
	LIMIT 10
)
, registered_voter_proposal_voting_history AS (
	SELECT 
		rv.signer_account_id AS registered_voter_id
		, pvh.proposal_id 
		, CASE 
			WHEN t.proposal_id IS NULL THEN 0 
			ELSE 1 END AS is_proposal_from_ten_most_recently_approved 
	FROM registered_voters_prep AS rv 
	INNER JOIN proposal_voting_history AS pvh 
		ON rv.signer_account_id = pvh.voter_id
	LEFT JOIN ten_most_recently_approved_proposals AS t
		ON t.proposal_id = pvh.proposal_id
)
, proposal_participation AS (
	SELECT
		registered_voter_id
		, SUM(is_proposal_from_ten_most_recently_approved)::NUMERIC      AS num_recently_approved_proposals_voted_on
		, SUM(is_proposal_from_ten_most_recently_approved)::NUMERIC / 10 AS proposal_participation_rate 
	FROM registered_voter_proposal_voting_history
	GROUP BY 1
)
, final AS (
/* Registered Voters + Current Voting Power */
	SELECT
		MD5(base58_encode(ra.receipt_id)) AS id
 		, base58_encode(ra.receipt_id) AS receipt_id
 		, DATE(ra.block_timestamp) 	   AS registered_date
 		, ra.block_timestamp      	   AS registered_at

 		--Deploy Lockup Details
 		, ra.signer_account_id         AS registered_voter_id
 		, ra.receiver_id       		   AS hos_contract_address
 		, CASE
	 		WHEN cvp.row_num IS NULL THEN FALSE
	 		ELSE TRUE
	 		END AS has_locked_unlocked_near
		, CASE 
 			WHEN ad.delegator_id IS NULL 
 			THEN FALSE 
 			ELSE TRUE 
 			END AS is_actively_delegating --TRUE if the latest delegation event for this account = 'delegate_all'

 		--Voting Power
		, COALESCE(dvp.delegations_voting_power, 0)                         AS voting_power_from_delegations
		, COALESCE(cvp.current_voting_power_logs, ivp.initial_voting_power) AS voting_power_from_locks_unlocks
 		, COALESCE(ivp.initial_voting_power, 0)                             AS initial_voting_power
 		, pp.proposal_participation_rate

 		--Block Details (For the deploy_lockup - aka "vote registration" - action on the veNEAR HOS contract address)
 		, ra.block_height
 		, base58_encode(ra.block_hash) AS block_hash

	FROM registered_voters_prep AS ra 						    --Sourced from the deploy_lockup event
	LEFT JOIN current_voting_power_from_locks_unlocks AS cvp 	--Sourced from the voter's most recent on_lockup_update event
		ON ra.signer_account_id = cvp.registered_voter_id
		AND cvp.row_num = 1
	LEFT JOIN initial_voting_power_from_locks_unlocks AS ivp 	--Sourced from the voter's storage_deposit event associated with the vote registration action
		ON ra.signer_account_id = ivp.registered_voter_id
	LEFT JOIN proposal_participation AS pp
		ON pp.registered_voter_id = ra.signer_account_id
	LEFT JOIN delegations_voting_power AS dvp 
		ON ra.signer_account_id = dvp.delegatee_id
	LEFT JOIN actively_delegating_accounts AS ad 
		ON ra.signer_account_id = ad.delegator_id 
	WHERE
		COALESCE(cvp.row_num, 0) IN (0,1)
	ORDER BY ra.block_timestamp DESC
)
SELECT 
	id
	, receipt_id
	, registered_date
	, registered_at
	, registered_voter_id
	, hos_contract_address
	, has_locked_unlocked_near
	, is_actively_delegating
	, voting_power_from_delegations
	, voting_power_from_locks_unlocks
	, initial_voting_power
	, CASE 
		WHEN is_actively_delegating = TRUE THEN voting_power_from_delegations 
		WHEN is_actively_delegating = FALSE THEN initial_voting_power + voting_power_from_delegations + voting_power_from_locks_unlocks
		ELSE 0
		END AS current_voting_power
	, proposal_participation_rate
	, block_height
	, block_hash
FROM final 
;

-- --Create the unique index for the view 	
-- CREATE UNIQUE INDEX idx_registered_voters_id ON registered_voters (id);

-- --Create the cron schedule
-- SELECT cron.schedule(
--     'refresh_registered_voters', 
--     '* * * * *',                   -- every minute
--     $$REFRESH MATERIALIZED VIEW CONCURRENTLY registered_voters;$$
-- );

-- --Activate the cron schedule 
-- SELECT cron.alter_job(11, active := true);