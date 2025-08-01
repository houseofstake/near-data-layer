/*
 Primary key on this table is receipt_id, with base58 encoding. 
 Every single row in this table is a unique, successful approve_proposal action.
 ("Successful" is defined by pulling only receipt_ids flagged as successful from the execution_outcomes table.)
 
 Each approve_proposal action is associated with: 
   1. A proposal approver ID                                              (eg. lighttea2007.testnet) 
   2. The related House of Stake Contract                                 (Voter contract address, vote.r-1745564650.testnet)
   3. The date + timestamp at which the proposal approval action occurred 
   4. The block-related data for this approve_proposal action             (Block hash or id, block height) 
*/

CREATE OR REPLACE VIEW {SCHEMA_NAME}.approved_proposals AS
WITH execution_outcomes_prep AS (
	SELECT
		receipt_id 
		, status
		, logs
	FROM {SCHEMA_NAME}.execution_outcomes
)
, approve_proposal_action_prep AS (
 	SELECT
    	decode(ra.args_base64, 'base64') AS args
    	, eo.status
    	, eo.logs
    	, ra.*
  	FROM {SCHEMA_NAME}.receipt_actions AS ra
  	INNER JOIN execution_outcomes_prep AS eo
 		ON ra.receipt_id = eo.receipt_id
	 	AND eo.status = 'SuccessReceiptId'
  	WHERE
    	ra.action_kind = 'FunctionCall'
    	AND ra.method_name = 'approve_proposal'
    	AND ra.receiver_id IN (     --House of Stake contracts
 			'v.hos-07.testnet'      --veNEAR contract 
 			, 'vote.hos-07.testnet' --Voting contract 
 			)
  	ORDER BY block_height DESC
 )
 SELECT
 	ra.receipt_id AS id
 	, ra.receipt_id AS receipt_id
 	, DATE(ra.block_timestamp) AS proposal_approved_date
 	, ra.block_timestamp AS proposal_approved_at

 	--Proposal Details
 	, ra.receiver_id AS hos_contract_address
 	, CASE 
 		    WHEN safe_json_parse(convert_from(ra.args, 'UTF8'))->>'error' IS NULL
 		    THEN (safe_json_parse(convert_from(ra.args, 'UTF8'))->>'proposal_id')::NUMERIC
 		    ELSE NULL 
 		  END AS proposal_id
 	, ra.signer_account_id AS proposal_approver_id

 	--Block details
 	, ra.block_hash
 	, ra.block_height
 FROM approve_proposal_action_prep AS ra
 ORDER BY block_timestamp DESC
 ;
