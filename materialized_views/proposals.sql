/*
 Primary key on this table is the receipt_id (base58 encoded) associated with a unique create_proposal action. 
 This is a dimensional table that returns, for each unique proposal id, the following: 
 
 1. Proposal metadata (name, description, URL, HoS Contract address) 
 2. Booleans indicating whether or not the proposal was: approved/rejected for public voting by a HoS reviewer, publicly voted on
 3. The timestamps of when the proposal was created, approved or rejected 
 4. Vote metadata (list of distinct voters, the count of distinct voters, the count of votes for or against the proposal) 
 5. The block-related data for the create_proposal action (block hash/id, block height) 
 */

--Create the view
CREATE VIEW proposals AS 
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
 		, eo.status AS action_status
 		, eo.logs AS action_logs
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
, create_proposal AS ( 
 	SELECT
 		base58_encode(ra.receipt_id) AS id
 		, base58_encode(ra.receipt_id) AS receipt_id
 		, DATE(ra.block_timestamp) AS proposal_created_date
 		, ra.block_timestamp AS proposal_created_at

 		--Proposal Details
 		, ra.receiver_id AS hos_contract_address
 		, (REPLACE(ra.action_logs[1], 'EVENT_JSON:', '')::json->'data'->0->>'proposal_id')::NUMERIC AS proposal_id 
 		, (convert_from(ra.args_decoded, 'UTF8')::json->'metadata'->>'title') AS proposal_title
 		, (convert_from(ra.args_decoded, 'UTF8')::json->'metadata'->>'description') AS proposal_description
 		, (convert_from(ra.args_decoded, 'UTF8')::json->'metadata'->>'link') AS proposal_url
	 	, ra.signer_account_id AS proposal_creator_id
	 	, ra.action_logs 
	 	
	 	--Block Data 
	 	, ra.block_height
 		, base58_encode(ra.block_hash) AS block_hash
 	FROM receipt_actions_prep AS ra
 	WHERE
 		ra.method_name = 'create_proposal'
)
, approve_proposal AS (
 	SELECT
 		base58_encode(ra.receipt_id) AS id
 		, base58_encode(ra.receipt_id) AS receipt_id
 		, DATE(ra.block_timestamp) AS proposal_approved_date
 		, ra.block_timestamp AS proposal_approved_at

 		--Proposal Details
 		, ra.receiver_id AS hos_contract_address
 		, (convert_from(ra.args_decoded, 'UTF8')::json->>'proposal_id')::NUMERIC AS proposal_id
 		, ra.signer_account_id AS proposal_approver_id
 		, ra.action_logs 
 	FROM receipt_actions_prep AS ra
 	WHERE
 		ra.method_name = 'approve_proposal'
 )
 , reject_proposal as (
 	SELECT
 		base58_encode(ra.receipt_id) AS id
 		, base58_encode(ra.receipt_id) AS receipt_id
 		, DATE(ra.block_timestamp) AS proposal_rejected_date
 		, ra.block_timestamp AS proposal_rejected_at

 		--Proposal Details
 		, ra.receiver_id AS hos_contract_address
 		, (convert_from(ra.args_decoded, 'UTF8')::json->>'proposal_id')::NUMERIC AS proposal_id
 		, ra.signer_account_id AS proposal_rejecter_id
 		, ra.action_logs 
 	FROM receipt_actions_prep AS ra
 	WHERE
 		ra.method_name = 'reject_proposal'
 )
 , proposal_votes AS ( 
 	SELECT 
 		proposal_id 
 		, COUNT(DISTINCT voter_id) AS num_distinct_voters 
 		, STRING_AGG(DISTINCT voter_id, ', ' ORDER BY voter_id ASC)	AS listagg_distinct_voters 
		, SUM(CASE WHEN vote_option = 0 THEN 1 ELSE 0 END) AS num_for_votes 
 		, SUM(CASE WHEN vote_option = 1 THEN 1 ELSE 0 END) AS num_against_votes 
 	FROM proposal_voting_history 
 	GROUP BY 1
 )
 SELECT
 	cp.receipt_id AS id 
 	, cp.receipt_id 
 	
 	--Proposal Details
 	, cp.proposal_id
 	, cp.proposal_title
 	, cp.proposal_description
 	, cp.proposal_url 
 	, cp.hos_contract_address 
 	, CASE 
 		WHEN ap.proposal_id IS NULL 
 		THEN FALSE ELSE TRUE
 		END AS is_approved
 	, CASE 
 		WHEN rp.proposal_id IS NULL 
 		THEN FALSE ELSE TRUE
 		END AS is_rejected
 	, CASE 
 		WHEN pv.num_distinct_voters IS NULL 
 		THEN FALSE ELSE TRUE 
 		END AS has_votes 
 	
 	--Creation Details
 	, cp.proposal_created_at AS created_at 
 	, cp.proposal_creator_id AS creator_id 
 	
 	--Approval Details 
 	, ap.proposal_approved_at AS approved_at 
 	, ap.proposal_approver_id AS approver_id 
 	
 	--Rejection Details 
 	, rp.proposal_rejected_at AS rejected_at 
 	, rp.proposal_rejecter_id AS rejecter_id 
 	
 	--Vote Details 
 	, pv.listagg_distinct_voters
 	, COALESCE(pv.num_distinct_voters, 0) AS num_distinct_voters 
 	, COALESCE(pv.num_for_votes, 0) AS num_for_votes
 	, COALESCE(pv.num_against_votes, 0) AS num_against_votes 
 	
 	--Block Data 
	, cp.block_height 
	, cp.block_hash 

 FROM create_proposal AS cp
 LEFT JOIN approve_proposal ap
 	ON cp.proposal_id = ap.proposal_id
 LEFT JOIN reject_proposal AS rp 
 	ON cp.proposal_id = rp.proposal_id 
 LEFT JOIN proposal_votes AS pv 
 	ON ap.proposal_id = pv.proposal_id 
 ORDER BY cp.proposal_created_at ASC
; 

-- --Create the unique index for the view 
-- CREATE UNIQUE INDEX idx_proposals_id ON proposals (id);

-- --Create the cron schedule
-- SELECT cron.schedule(
--     'refresh_proposals', 
--     '* * * * *',                   -- every minute
--     $$REFRESH MATERIALIZED VIEW CONCURRENTLY proposals;$$
-- );

-- --Activate the cron schedule 
-- SELECT cron.alter_job(12, active := true); 