/*
 Primary key on this table is receipt_id, with base58 encoding. 
 Every single row in this table is a unique, successful approve_proposal action.
 ("Successful" is defined by pulling only receipt_ids flagged as successful from the execution_outcomes table.)
 
 Each approve_proposal action is associated with: 
   1. A proposal approver ID                                              (eg. lighttea2007.testnet) 
   2. The related House of Stake Contract                                 (Voter contract address, vote.r-1745564650.testnet)
   3. The date + timestamp at which the proposal approval action occurred 
   4. The block-related data for this approve_proposal action             (Block hash/id, block height) 
*/

--Create the materialized view
CREATE MATERIALIZED VIEW approved_proposals AS
WITH execution_outcomes_prep AS (
	SELECT
		SPLIT_PART(receipt_id, '-', 2) AS receipt_id
		, status
		, logs
	FROM execution_outcomes
)
, approve_proposal_action_prep AS (
 	SELECT
    	decode(ra.args_base64, 'base64') AS args
    	, eo.status
    	, eo.logs
    	, base58_encode(ra.receipt_id) AS encoded_receipt_id
    	, ra.*
  	FROM receipt_actions AS ra
  	INNER JOIN execution_outcomes_prep AS eo
 		ON ra.receipt_id = eo.receipt_id
	 	AND eo.status = 'SuccessReceiptId'
  	WHERE
    	ra.action_kind = 'FunctionCall'
    	AND ra.method_name = 'approve_proposal'
    	AND ra.receiver_id IN (           --House of Stake contracts
 			'v.r-1748895584.testnet'      --veNEAR contract 
 			, 'vote.r-1748895584.testnet' --Voting contract 
 			)
  	ORDER BY block_height DESC
 )
 SELECT
 	base58_encode(ra.receipt_id) AS id
 	, base58_encode(ra.receipt_id) AS receipt_id
 	, DATE(ra.block_timestamp)     AS proposal_approved_date
 	, ra.block_timestamp           AS proposal_approved_at

 	--Proposal Details
 	, ra.receiver_id           												   AS hos_contract_address
 	, (convert_from(ra.args, 'UTF8')::json->>'proposal_id')::numeric           AS proposal_id
 	, ra.signer_account_id     												   AS proposal_approver_id

 	--Block details
 	, ra.block_hash
 	, ra.block_height
 FROM approve_proposal_action_prep AS ra
 ORDER BY block_timestamp DESC
 WITH DATA
 ;

--Create the unique index for the view 
 CREATE UNIQUE INDEX idx_pproved_proposals_id ON approved_proposals (id);

--Create the cron schedule
SELECT cron.schedule(
    'refresh_approved_proposals', 
    '* * * * *',                   -- every minute
    $$REFRESH MATERIALIZED VIEW CONCURRENTLY approved_proposals;$$
);

--Pause the cron schedule 
SELECT cron.alter_job(9, active := false);