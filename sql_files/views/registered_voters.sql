/*
 Primary key on this table is receipt_id. 
 Every single row in this table is a unique, successful deploy_lockup action, which translates into a voter registration action.
 ("Successful" is defined by pulling only receipt_ids flagged as successful from the execution_outcomes table.)
 
 In this view, each row is associated with: 
   1. A registered voter ID                                                                 (The voter account, eg. lighttea2007.testnet; also the delegator account, if the account has delegated its NEAR balance away) 
   2. Boolean flag indicating whether registered voter has locked or unlocked any NEAR      (Sourced from the latest on_lockup_update event that is emitted upon a user's latest lock or unlock action)
   3. Boolean flag indicating whether registered voter is currently delegating their NEAR   (Sourced from the fastnear.delegation_events view)
   4. A delegatee ID                                                                        (The account to which the registered voter has delegated their entire NEAR balance; NULL if the voter has not delegated their balance)
   5. The related House of Stake Contract                                                   (veNEAR contract address, v.voteagora.near)
   6. The registerd voter's current voting power                                            (Calculated via aggregating the voting power from initial vote registration, the user's latest lock or unlock action, any delegated balances, and accrued rewards aka extra venear earned)  
   7. The registered voter's initial voting power, aka voting power from vote registration  (Sourced from the storage_deposit event that is emitted upon a user's vote registration action) 
   8. the registered voter's voting power from locks and unlocks                            (Sourced from the latest on_lockup_update event that is emitted upon a user's latest lock or unlock action)
   9. The registered voter's principal balance                                              (Calculated as the sum of a user's initial voting power and their voting power from locks or unlocks) 
   10. The extra venear earned on a principal balance 										(The voting power a registered voter earns from it's principal NEAR balance; calculated using an APY growth rate sourced from the contract upon contract definition) 
   11. The registered voter's voting power from delegations, aka delegated balance          (Equivalent to the principal balance of the associated delegator account) 
   12. The extra venear earned on a delegated balance                                       (Equivalent to the extra venear earned on principal for the associated delegator account) 
   13. The registered voter's proposal participation rate                                   (Calculated as a count of the vote_options - only considering the latest vote_option per proposal - a user makes on any of the 10 most recently approved proposals for the veNEAR contract; always a percentage out of 10)
   14. The timestamp at which the voter registration action occurred 
   15. The block-related data for this deploy_lockup action                                 (Block hash or id, block height) 
*/


CREATE OR REPLACE VIEW {SCHEMA_NAME}.registered_voters AS
---------------------------
--BASE DATA SOURCING PREP--
---------------------------
WITH 
/* Base Table Prep */
receipt_actions_prep AS (
	SELECT
		decode(ra.args_base64, 'base64') AS args_decoded
		, eo.status AS action_status
		, eo.logs AS action_logs
		, ra.*
	FROM {SCHEMA_NAME}.receipt_actions AS ra
	INNER JOIN {SCHEMA_NAME}.execution_outcomes AS eo
		ON ra.receipt_id = eo.receipt_id
		AND eo.status IN ('SuccessReceiptId', 'SuccessValue')
	WHERE
		ra.action_kind = 'FunctionCall'
        AND ra.receiver_id IN (     --House of Stake contracts
 			'v.{HOS_CONTRACT}'      --veNEAR contract 
 			, 'vote.{HOS_CONTRACT}' --Voting contract 
 			)
)
/* Sourcing APY Growth Rate Variables (to calc Voting Power from Rewards) */
,  venear_contract_growth_config AS (
	SELECT 
    	signer_account_id AS hos_contract_address 
    	, CASE
          	WHEN (safe_json_parse(convert_from(ra.args_decoded, 'UTF8')) ->> 'error') IS NULL 
            THEN (((safe_json_parse(convert_from(ra.args_decoded, 'UTF8')) -> 'venear_growth_config') -> 'annual_growth_rate_ns') ->> 'numerator')::NUMERIC
            ELSE NULL
            END AS growth_rate_numerator_ns
        , CASE
            WHEN (safe_json_parse(convert_from(ra.args_decoded, 'UTF8')) ->> 'error') IS NULL 
            THEN (((safe_json_parse(convert_from(ra.args_decoded, 'UTF8')) -> 'venear_growth_config') -> 'annual_growth_rate_ns') ->> 'denominator')::NUMERIC
            ELSE NULL
            END AS growth_rate_denominator_ns
    FROM receipt_actions_prep AS ra
    WHERE 
    	ra.method_name = 'new'

)
/* Sourcing Voting Power from Vote Registration */
, voting_power_from_vote_registration AS (
	SELECT vpvr.*
	FROM (
  			SELECT
  				ra.block_timestamp
    			, ra.receipt_id
    			, COALESCE(
    				CASE 
 		     			WHEN safe_json_parse(REPLACE(ra.action_logs[1], 'EVENT_JSON:', ''))->>'error' IS NULL
 		   	 			THEN safe_json_parse(REPLACE(ra.action_logs[1], 'EVENT_JSON:', ''))->'data'->0->>'owner_id'
 		     			ELSE NULL END
						, ra.signer_account_id
		  			) AS registered_voter_id
    			, CASE 
 		    		WHEN safe_json_parse(REPLACE(ra.action_logs[1], 'EVENT_JSON:', ''))->>'error' IS NULL
 		    		THEN (safe_json_parse(REPLACE(ra.action_logs[1], 'EVENT_JSON:', ''))->'data'->0->>'amount')::NUMERIC
 		    		ELSE NULL 
 		    		END AS voting_power_from_vote_registration
  			FROM receipt_actions_prep AS ra
  			WHERE
    			ra.method_name = 'storage_deposit'
    	) AS vpvr
    WHERE 
    	vpvr.voting_power_from_vote_registration IS NOT NULL 
)
/* Sourcing Latest Voting Power from Locks + Unlocks */ 
, voting_power_from_locks_unlocks AS (
	SELECT 
		vplu.*
	FROM (
			SELECT
				ra.block_timestamp
				, ra.receipt_id
				, COALESCE(CASE 
 		    		WHEN safe_json_parse(REPLACE(ra.action_logs[1], 'EVENT_JSON:', ''))->>'error' IS NULL
 		    		THEN safe_json_parse(REPLACE(ra.action_logs[1], 'EVENT_JSON:', ''))->'data'->0->>'account_id'
 		    		ELSE NULL 
 		    		END
					, ra.signer_account_id) AS registered_voter_id
				, CASE 
 		    		WHEN safe_json_parse(REPLACE(ra.action_logs[1], 'EVENT_JSON:', ''))->>'error' IS NULL
 		    		THEN (safe_json_parse(REPLACE(ra.action_logs[1], 'EVENT_JSON:', ''))->'data'->0->>'locked_near_balance')::NUMERIC
 		    		ELSE NULL 
 		    		END AS voting_power_from_locks_unlocks
 				, CASE 
 		    		WHEN safe_json_parse(REPLACE(ra.action_logs[1], 'EVENT_JSON:', ''))->>'error' IS NULL
 		    		THEN (safe_json_parse(REPLACE(ra.action_logs[1], 'EVENT_JSON:', ''))->'data'->0->>'timestamp')::NUMERIC
 		    		ELSE NULL 
 		    		END AS lockup_update_at_ns  --Timestamp (nanoseconds) when user locks or unlocks near 
    			, ROW_NUMBER() OVER (PARTITION BY signer_account_id ORDER BY block_timestamp DESC) AS row_num
  			FROM receipt_actions_prep AS ra
  			WHERE
    			ra.method_name = 'on_lockup_update'
    	) AS vplu
    WHERE 
    	vplu.row_num = 1
)
/* Sourcing Registered Voters from Deploy Lockup Event (Excluding dupes due to account already being registered) */
  --Whenever a user registers to vote, there should always be a non-null storage deposit; aka, initial voting power amount.
  --Inner join excludes scenarios where a vote registration action (aka deploy_lockup for a given user) is duped bc the user's account was already registered and the subsequent storage_deposit event tracks a null initial_voting_power amount.
, registered_voters_prep AS (
  	SELECT
    	ra.*
    	, (EXTRACT(EPOCH FROM ra.block_timestamp) * 1e9)::BIGINT AS registered_at_ns 
  	FROM receipt_actions_prep AS ra
	INNER JOIN voting_power_from_vote_registration AS vpvr 
		ON ra.receipt_id = vpvr.receipt_id 
  	WHERE
    	ra.method_name = 'deploy_lockup'
)
--------------------------------------
--PROPOSAL PARTICIPATION CALCULATION--
--------------------------------------
/* VI. Sourcing Proposal Participation (from 10 most recently approved proposals) */
, ten_most_recently_approved_proposals AS (
	SELECT *
	FROM {SCHEMA_NAME}.approved_proposals
	ORDER BY proposal_approved_at DESC
	LIMIT 10
)
, registered_voter_proposal_voting_history AS (
	SELECT 
		rv.signer_account_id AS registered_voter_id
		, pvh.proposal_id 
		, CASE 
			WHEN t.proposal_id IS NULL THEN 0 
			ELSE 1 
			END AS is_proposal_from_ten_most_recently_approved 
	FROM registered_voters_prep AS rv 
	INNER JOIN {SCHEMA_NAME}.proposal_voting_history AS pvh 
		ON rv.signer_account_id = pvh.voter_id
	LEFT JOIN ten_most_recently_approved_proposals AS t
		ON pvh.proposal_id = t.proposal_id
)
, proposal_participation AS (
	SELECT
		registered_voter_id
		, SUM(is_proposal_from_ten_most_recently_approved)::NUMERIC AS num_recently_approved_proposals_voted_on
		, SUM(is_proposal_from_ten_most_recently_approved)::NUMERIC / 10 AS proposal_participation_rate 
	FROM registered_voter_proposal_voting_history
	GROUP BY 1
)
------------------------------------------
--REGISTERED VOTERS + BASIC VOTING POWER--
------------------------------------------
, table_joins AS (
	SELECT
		ra.*
		
		--New Field Additions
 		, CASE
	 		WHEN vplu.registered_voter_id IS NULL THEN FALSE
	 		ELSE TRUE
	 		END AS has_locked_unlocked_near
	 		
		, CASE 
 			WHEN de.delegator_id IS NULL THEN FALSE 
 			ELSE TRUE 
 			END AS is_actively_delegating 
 			
 		, de.delegatee_id
 		, pp.proposal_participation_rate
 		
 		--Voting Power from Rewards - Calculation Variables
 		, gc.growth_rate_numerator_ns
 		, gc.growth_rate_denominator_ns
 		, (EXTRACT(EPOCH FROM NOW()) * 1e9)::NUMERIC 				AS now_ns 
 		, vplu.lockup_update_at_ns 									AS latest_lockup_update_at_ns --we ALSO need the timestamp FOR users who have NOT had ANY locks/unlocks; what IS it?

 		--Voting Powers
		, COALESCE(vplu.voting_power_from_locks_unlocks, 0)     	AS voting_power_from_locks_unlocks 
 		, COALESCE(vpvr.voting_power_from_vote_registration, 0) 	AS voting_power_from_vote_registration --aka initial voting power, AS a registered voter! 
 		, COALESCE(vplu.voting_power_from_locks_unlocks, 0) 
 			+ COALESCE(vpvr.voting_power_from_vote_registration, 0) AS principal_balance 

	FROM registered_voters_prep AS ra 					    --Sourced from the deploy_lockup event
	LEFT JOIN venear_contract_growth_config AS gc           --Sourced from method_name = 'new'; function call sets up the contract state WHEN it IS FIRST deployed
        ON ra.receiver_id = gc.hos_contract_address 
	LEFT JOIN voting_power_from_locks_unlocks AS vplu 	    --Sourced from the voter's most recent on_lockup_update event
		ON ra.signer_account_id = vplu.registered_voter_id	
	LEFT JOIN voting_power_from_vote_registration AS vpvr 	--Sourced from the voter's storage_deposit event associated with the vote registration action
		ON ra.signer_account_id = vpvr.registered_voter_id
	LEFT JOIN proposal_participation AS pp
		ON ra.signer_account_id = pp.registered_voter_id 
	LEFT JOIN {SCHEMA_NAME}.delegation_events AS de 
	 	ON ra.signer_account_id = de.delegator_id 
		AND de.is_latest_delegator_event = TRUE 
		AND de.delegate_method = 'delegate_all'
		AND de.delegate_event = 'ft_mint'
)
-----------------------------------------
--CALCULATING VOTING POWER FROM REWARDS--
-----------------------------------------
, voting_power_from_rewards AS (
	SELECT 
		tj.* 
		
		--Calculating Voting Power from Rewards
		, CASE 
    		WHEN has_locked_unlocked_near = TRUE
    		THEN 
    			( (FLOOR(principal_balance/1e21)*1e21)  )
    	  	  * ( growth_rate_numerator_ns / growth_rate_denominator_ns ) 
    	      * ( (FLOOR(now_ns/1e9)*1e9) - (FLOOR(latest_lockup_update_at_ns/1e9)*1e9) )
    		ELSE 
    			( (FLOOR(principal_balance/1e21)*1e21) )
    	  	  * ( growth_rate_numerator_ns / growth_rate_denominator_ns ) 
    	  	  * ( (FLOOR(now_ns/1e9)*1e9) - (FLOOR(registered_at_ns/1e9)*1e9) ) 
    		END AS extra_venear_on_principal
    		
	FROM table_joins AS tj 
)
, delegated_voting_power AS (
	SELECT 
		delegatee_id 
		, SUM(principal_balance) 		 AS delegated_balance 
		, SUM(extra_venear_on_principal) AS delegated_extra_venear
	FROM voting_power_from_rewards
	GROUP BY 1 
)
-------------
--FINAL CTE--
-------------
SELECT 
	MD5(ra.receipt_id) 				AS id
	, ra.receipt_id
 	, DATE(ra.block_timestamp) 		AS registered_date
 	, ra.block_timestamp 			AS registered_at
 	, ra.signer_account_id 			AS registered_voter_id
 	, ra.receiver_id 				AS hos_contract_address
 	, ra.has_locked_unlocked_near
 	, ra.is_actively_delegating
 	, ra.delegatee_id
 	, ra.proposal_participation_rate
 	
 	--Voting Power
 	, ra.voting_power_from_locks_unlocks
 	, ra.voting_power_from_vote_registration  AS initial_voting_power          --Keeping original column name per front-end request
 	, ra.principal_balance 
 	, ra.extra_venear_on_principal
 	, COALESCE(dvp.delegated_balance, 0)      AS voting_power_from_delegations --Keeping original column name per front-end request
 	, COALESCE(dvp.delegated_extra_venear, 0) AS delegated_extra_venear 
 	, CASE 
	 	WHEN ra.is_actively_delegating = TRUE THEN 0 
	 	ELSE (
	 	       COALESCE(principal_balance, 0) 
	 	     + COALESCE(extra_venear_on_principal,0) 
	 	     + COALESCE(delegated_balance, 0) 
	 	     + COALESCE(delegated_extra_venear,0)
	 	     )
 	  	END AS current_voting_power
 	
 	--Block Details (For the deploy_lockup - aka "vote registration" - action on the veNEAR HOS contract address)  	
 	, ra.block_height 
 	, ra.block_hash
FROM voting_power_from_rewards AS ra
LEFT JOIN delegated_voting_power AS dvp
	ON ra.signer_account_id = dvp.delegatee_id
ORDER BY ra.block_timestamp ASC
;
