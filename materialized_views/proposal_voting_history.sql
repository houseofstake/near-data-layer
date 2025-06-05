/*
 Primary key on this table is receipt_id, with base58 encoding. 
 Every single row in this table is a unique, successful vote action.
 ("Successful" is defined by pulling only receipt_ids flagged as successful from the execution_outcomes table.)
 
 Each vote action is associated with: 
   1. A voter account 
   2. The proposal voted on 
   3. The vote option made 
   4. The voting power of the voter at the time the vote was executed (from execution_outcomes.logs)
   5. The voting power delegated on a vote action 
   6. The timestamp at which the vote action occurred 
   7. The block-related data for this vote (block hash/id, block height) 
*/

--Create the materialized view
CREATE MATERIALIZED VIEW proposal_voting_history AS
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
        , ra.*
        , eo.logs 
      FROM receipt_actions AS ra
      INNER JOIN execution_outcomes_prep AS eo 
         ON ra.receipt_id = eo.receipt_id 
         AND eo.status = 'SuccessValue'
      WHERE 
        ra.action_kind = 'FunctionCall'
        AND ra.receiver_id IN (           --House of Stake contracts
            'v.r-1745564650.testnet'      --veNEAR contract 
            , 'vote.r-1745564650.testnet' --Voting contract
            , 'v.r-1748895584.testnet' -- v0.0.2 veNEAR contract
            , 'vote.r-1748895584.testnet' -- v0.0.2 Voting contract
        ) 
)
, proposal_metadata AS (
    SELECT 
        (convert_from(ra.args, 'UTF8')::json->'metadata'->>'title') AS proposal_name
        , (REPLACE(ra.logs[1], 'EVENT_JSON:', '')::json->'data'->0->>'proposal_id')::NUMERIC AS proposal_id 
    FROM receipt_actions_prep AS ra
    WHERE 
        ra.method_name = 'create_proposal'
)
, proposal_voting_history AS (
    SELECT 
        base58_encode(ra.receipt_id)   AS id  
        , base58_encode(ra.receipt_id) AS receipt_id 
        , DATE(ra.block_timestamp)     AS voted_date 
        , ra.block_timestamp           AS voted_at 
    
        --IDs 																					
        , (convert_from(ra.args, 'UTF8')::json->>'proposal_id')::NUMERIC AS proposal_id
        , ra.receiver_id    											 AS hos_contract_address 
        , ra.predecessor_id 											 AS voter_id 
    
        /* Voter Data Per Proposal */
        --Votes Info
        , (convert_from(ra.args, 'UTF8')::json->>'vote')::NUMERIC                                				AS vote_option
        , (REPLACE(ra.logs[1], 'EVENT_JSON:', '')::json->'data'->0->>'account_balance')::NUMERIC 			    AS voting_power
        , (convert_from(ra.args, 'UTF8')::json->'v_account'->'V0'->'balance'->>'near_balance')::NUMERIC         AS near_balance 
        , (convert_from(ra.args, 'UTF8')::json->'v_account'->'V0'->'balance'->>'extra_venear_balance')::NUMERIC AS extra_venear_balance
    
        --Delegation Info
        , (convert_from(ra.args, 'UTF8')::json->'v_account'->'V0'->'delegation'->>'account_id')        					  AS delegator_account_id 
        , (convert_from(ra.args, 'UTF8')::json->'v_account'->'V0'->'delegated_balance'->>'near_balance')::NUMERIC         AS delegated_near_balance
        , (convert_from(ra.args, 'UTF8')::json->'v_account'->'V0'->'delegated_balance'->>'extra_venear_balance')::NUMERIC AS delegated_extra_venear_balance
    
        --Logs 
        , ra.logs
    
        --Block Data 
        , ra.block_height 
        , base58_encode(ra.block_hash) AS block_hash 
    FROM receipt_actions_prep AS ra
     WHERE 
        ra.method_name = 'vote'
        AND (convert_from(ra.args, 'UTF8')::json->>'proposal_id')::NUMERIC IS NOT NULL
    ORDER BY proposal_id ASC, voted_at ASC 
)
, latest_vote_per_proposal_and_voter AS (
    SELECT 
        *
        , ROW_NUMBER() OVER (PARTITION BY proposal_id, voter_id ORDER BY voted_at DESC) as row_num 
    FROM proposal_voting_history 
)
SELECT 
    l.id 
    , l.receipt_id 
    , l.voted_date 
    , l.voted_at 
    , l.proposal_id 
    , pm.proposal_name
    , l.hos_contract_address 
    , l.voter_id 
    , l.vote_option 
    , l.voting_power 
    , l.near_balance 
    , l.extra_venear_balance 
    , l.delegator_account_id 
    , l.delegated_near_balance 
    , l.delegated_extra_venear_balance 
    , l.block_height 
    , l.block_hash
FROM latest_vote_per_proposal_and_voter AS l
LEFT JOIN proposal_metadata AS pm 
    ON l.proposal_id = pm.proposal_id
WHERE 
    l.row_num = 1
WITH DATA
;

--Create the unique index for the view 
CREATE UNIQUE INDEX proposal_voting_history_id_idx ON proposal_voting_history (id);

--Create the cron schedule
SELECT cron.schedule(
    'refresh_proposal_voting_history', 
    '* * * * *',                   -- every minute
    $$REFRESH MATERIALIZED VIEW CONCURRENTLY proposal_voting_history;$$
);

--Pause the cron schedule 
SELECT cron.alter_job(8, active := false);