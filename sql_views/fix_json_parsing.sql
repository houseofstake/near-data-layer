-- Fix for JSON parsing errors in SQL views
-- This script shows the patterns to replace unsafe JSON parsing with safe parsing

-- First, create the helper function
CREATE OR REPLACE FUNCTION safe_json_parse(input_text TEXT)
RETURNS JSON AS $$
BEGIN
    IF input_text IS NULL OR input_text = '' THEN
        RETURN NULL;
    END IF;
    
    BEGIN
        RETURN input_text::JSON;
    EXCEPTION WHEN OTHERS THEN
        -- Wrap the invalid text in a JSON error object
        RETURN json_build_object(
            'error', 'invalid_json',
            'original_text', input_text,
            'message', 'Failed to parse as JSON'
        );
    END;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Pattern 1: Replace simple JSON field extraction
-- FROM: (convert_from(ra.args_decoded, 'UTF8')::json->>'field_name')
-- TO: CASE 
--       WHEN safe_json_parse(convert_from(ra.args_decoded, 'UTF8'))->>'error' IS NULL
--       THEN safe_json_parse(convert_from(ra.args_decoded, 'UTF8'))->>'field_name'
--       ELSE NULL 
--     END

-- Pattern 2: Replace numeric JSON field extraction
-- FROM: (convert_from(ra.args_decoded, 'UTF8')::json->>'field_name')::NUMERIC
-- TO: CASE 
--       WHEN safe_json_parse(convert_from(ra.args_decoded, 'UTF8'))->>'error' IS NULL
--       THEN (safe_json_parse(convert_from(ra.args_decoded, 'UTF8'))->>'field_name')::NUMERIC
--       ELSE NULL 
--     END

-- Pattern 3: Replace nested JSON field extraction
-- FROM: (convert_from(ra.args_decoded, 'UTF8')::json->'parent'->>'child')
-- TO: CASE 
--       WHEN safe_json_parse(convert_from(ra.args_decoded, 'UTF8'))->>'error' IS NULL
--       THEN safe_json_parse(convert_from(ra.args_decoded, 'UTF8'))->'parent'->>'child'
--       ELSE NULL 
--     END

-- Pattern 4: Replace array JSON field extraction
-- FROM: (REPLACE(ra.logs[1], 'EVENT_JSON:', '')::json->'data'->0->>'field_name')
-- TO: CASE 
--       WHEN safe_json_parse(REPLACE(ra.logs[1], 'EVENT_JSON:', ''))->>'error' IS NULL
--       THEN safe_json_parse(REPLACE(ra.logs[1], 'EVENT_JSON:', ''))->'data'->0->>'field_name'
--       ELSE NULL 
--     END

-- Pattern 5: Replace complex nested JSON field extraction
-- FROM: (convert_from(ra.args_decoded, 'UTF8')::json->'v_account'->'V0'->'balance'->>'near_balance')::NUMERIC
-- TO: CASE 
--       WHEN safe_json_parse(convert_from(ra.args_decoded, 'UTF8'))->>'error' IS NULL
--       THEN (safe_json_parse(convert_from(ra.args_decoded, 'UTF8'))->'v_account'->'V0'->'balance'->>'near_balance')::NUMERIC
--       ELSE NULL 
--     END

-- Example transformations for each view:

-- 1. proposals.sql - Already fixed above

-- 2. proposal_voting_history.sql needs these replacements:
/*
FROM: (convert_from(ra.args, 'UTF8')::json->'metadata'->>'title')
TO: CASE 
      WHEN safe_json_parse(convert_from(ra.args, 'UTF8'))->>'error' IS NULL
      THEN safe_json_parse(convert_from(ra.args, 'UTF8'))->'metadata'->>'title'
      ELSE NULL 
    END

FROM: (convert_from(ra.args, 'UTF8')::json->>'proposal_id')::NUMERIC
TO: CASE 
      WHEN safe_json_parse(convert_from(ra.args, 'UTF8'))->>'error' IS NULL
      THEN (safe_json_parse(convert_from(ra.args, 'UTF8'))->>'proposal_id')::NUMERIC
      ELSE NULL 
    END

FROM: (convert_from(ra.args, 'UTF8')::json->'v_account'->'V0'->'balance'->>'near_balance')::NUMERIC
TO: CASE 
      WHEN safe_json_parse(convert_from(ra.args, 'UTF8'))->>'error' IS NULL
      THEN (safe_json_parse(convert_from(ra.args, 'UTF8'))->'v_account'->'V0'->'balance'->>'near_balance')::NUMERIC
      ELSE NULL 
    END
*/

-- 3. user_activities.sql needs these replacements:
/*
FROM: (CONVERT_FROM(DECODE(ra.args_base64, 'base64'), 'UTF8')::json->>'amount')::NUMERIC
TO: CASE 
      WHEN safe_json_parse(CONVERT_FROM(DECODE(ra.args_base64, 'base64'), 'UTF8'))->>'error' IS NULL
      THEN (safe_json_parse(CONVERT_FROM(DECODE(ra.args_base64, 'base64'), 'UTF8'))->>'amount')::NUMERIC
      ELSE NULL 
    END
*/

-- 4. delegation_events.sql needs these replacements:
/*
FROM: (CONVERT_FROM(ra.args, 'UTF8')::json->>'receiver_id')
TO: CASE 
      WHEN safe_json_parse(CONVERT_FROM(ra.args, 'UTF8'))->>'error' IS NULL
      THEN safe_json_parse(CONVERT_FROM(ra.args, 'UTF8'))->>'receiver_id'
      ELSE NULL 
    END
*/

-- 5. approved_proposals.sql needs these replacements:
/*
FROM: (convert_from(ra.args, 'UTF8')::json->>'proposal_id')::numeric
TO: CASE 
      WHEN safe_json_parse(convert_from(ra.args, 'UTF8'))->>'error' IS NULL
      THEN (safe_json_parse(convert_from(ra.args, 'UTF8'))->>'proposal_id')::numeric
      ELSE NULL 
    END
*/

-- 6. registered_voters.sql needs these replacements:
/*
FROM: (convert_from(ra.args_decoded, 'UTF8')::json->'update'->'V1'->>'locked_near_balance')::NUMERIC
TO: CASE 
      WHEN safe_json_parse(convert_from(ra.args_decoded, 'UTF8'))->>'error' IS NULL
      THEN (safe_json_parse(convert_from(ra.args_decoded, 'UTF8'))->'update'->'V1'->>'locked_near_balance')::NUMERIC
      ELSE NULL 
    END
*/

-- Optional: If you want to see the error data for debugging, you can add a column like:
-- safe_json_parse(convert_from(ra.args_decoded, 'UTF8'))->>'original_text' as raw_args_text 