/*
 Primary key on this table is a composite of proposal_id and registered_voter_id. 
 This view returns a list of all the registered voters who have not yet voted on a given proposal_id. 
 References the proposals, registered_voters and proposal_voting_history views. 
*/

--Create the view
CREATE VIEW proposal_non_voters AS 
SELECT 
    MD5(CONCAT(p.proposal_id, '_', rv.registered_voter_id)) AS id
    , p.proposal_id
    , rv.registered_voter_id
FROM proposals AS p
CROSS JOIN registered_voters AS rv
WHERE NOT EXISTS (
    SELECT 
    	1 
    FROM proposal_voting_history AS h
    WHERE 
    	h.proposal_id = p.proposal_id 
    	AND h.voter_id = rv.registered_voter_id
)
ORDER BY 2 ASC, 3 ASC
;

-- --Create the unique index for the view 
-- CREATE UNIQUE INDEX idx_proposal_non_voters_id ON proposal_non_voters (id);

-- --Create the cron schedule
-- SELECT cron.schedule(
--     'refresh_proposal_non_voters', 
--     '* * * * *',                   -- every minute
--     $$REFRESH MATERIALIZED VIEW CONCURRENTLY proposal_non_voters;$$
-- );

-- --Activate the cron schedule 
-- SELECT cron.alter_job(13, active := true);