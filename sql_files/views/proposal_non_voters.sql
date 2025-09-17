/*
 Primary key on this table is a composite of proposal_id and registered_voter_id. 
 This view returns a list of all the registered voters who have not yet voted on a given proposal_id. 
 References the proposals, registered_voters and proposal_voting_history views. 
*/

DROP VIEW IF EXISTS {SCHEMA_NAME}.proposal_non_voters CASCADE;
CREATE VIEW {SCHEMA_NAME}.proposal_non_voters AS 
SELECT 
    MD5(CONCAT(p.proposal_id, '_', rv.registered_voter_id)) AS id
    , p.proposal_id
    , rv.registered_voter_id
FROM {SCHEMA_NAME}.proposals AS p
CROSS JOIN {SCHEMA_NAME}.registered_voters AS rv
WHERE NOT EXISTS (
    SELECT 
    	1 
    FROM {SCHEMA_NAME}.proposal_voting_history AS h
    WHERE 
    	h.proposal_id = p.proposal_id 
    	AND h.voter_id = rv.registered_voter_id
)
ORDER BY 2 ASC, 3 ASC
;
