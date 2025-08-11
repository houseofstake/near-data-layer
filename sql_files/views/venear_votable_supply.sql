/*
  Returns the current total votable veNEAR supply by summing all ft_mint and ft_burn events.
  Positive for ft_mint, negative for ft_burn.
*/

CREATE OR REPLACE VIEW {SCHEMA_NAME}.venear_votable_supply AS
WITH execution_outcomes_prep AS (
  SELECT
    receipt_id,
    status,
    logs
  FROM {SCHEMA_NAME}.execution_outcomes
)
, receipt_actions_prep AS (
  SELECT
    eo.logs,
    ra.*
  FROM {SCHEMA_NAME}.receipt_actions AS ra
  INNER JOIN execution_outcomes_prep AS eo
    ON ra.receipt_id = eo.receipt_id
   AND eo.status IN ('SuccessReceiptId', 'SuccessValue')
  WHERE ra.action_kind = 'FunctionCall'
)
, mint_burn_events AS (
  SELECT
    ra.receipt_id,
    ra.block_timestamp,
    CASE 
      WHEN safe_json_parse(REPLACE(log, 'EVENT_JSON:', ''))->>'error' IS NULL
      THEN safe_json_parse(REPLACE(log, 'EVENT_JSON:', ''))->>'event'
      ELSE NULL 
    END AS event_type,
    CASE 
      WHEN safe_json_parse(REPLACE(log, 'EVENT_JSON:', ''))->>'error' IS NULL
      THEN (safe_json_parse(REPLACE(log, 'EVENT_JSON:', ''))->'data'->0->>'amount')::NUMERIC
      ELSE NULL 
    END AS near_amount
  FROM receipt_actions_prep AS ra
  CROSS JOIN LATERAL UNNEST(ra.logs) AS log
)
SELECT
  'venear_votable_supply' AS id,
  COALESCE(
    SUM(
      CASE 
        WHEN event_type = 'ft_mint' THEN near_amount
        WHEN event_type = 'ft_burn' THEN -near_amount
        ELSE 0
      END
    ),
    0
  )::NUMERIC AS total_venear
FROM mint_burn_events;


