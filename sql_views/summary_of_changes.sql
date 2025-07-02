-- Summary of Changes Made to Fix JSON Parsing Issues
-- ===================================================

-- 1. HELPER FUNCTION CREATED
-- ==========================
-- File: near-data-layer/sql_views/helper_queries/safe_json_parse.sql
-- 
-- Created a helper function that safely parses JSON and wraps invalid JSON in an error object:
-- 
-- CREATE OR REPLACE FUNCTION safe_json_parse(input_text TEXT)
-- RETURNS JSON AS $$
-- BEGIN
--     IF input_text IS NULL OR input_text = '' THEN
--         RETURN NULL;
--     END IF;
--     
--     BEGIN
--         RETURN input_text::JSON;
--     EXCEPTION WHEN OTHERS THEN
--         -- Wrap the invalid text in a JSON error object
--         RETURN json_build_object(
--             'error', 'invalid_json',
--             'original_text', input_text,
--             'message', 'Failed to parse as JSON'
--         );
--     END;
-- END;
-- $$ LANGUAGE plpgsql IMMUTABLE;

-- 2. FILES UPDATED
-- ================

-- A. proposals.sql
--    - Fixed 4 JSON parsing locations
--    - Lines: proposal_id, proposal_title, proposal_description, proposal_url
--    - Pattern: Added CASE statements with safe_json_parse() checks

-- B. proposal_voting_history.sql  
--    - Fixed 8 JSON parsing locations
--    - Lines: proposal_name, proposal_id, vote_option, voting_power, near_balance, 
--             extra_venear_balance, delegator_account_id, delegated_near_balance, 
--             delegated_extra_venear_balance
--    - Pattern: Added CASE statements with safe_json_parse() checks

-- C. user_activities.sql
--    - Fixed 12+ JSON parsing locations across multiple CTEs
--    - Lines: event_type, near_amount, locked_near_balance, ft_event_type, 
--             owner_id, amount, etc.
--    - Pattern: Added CASE statements with safe_json_parse() checks

-- D. delegation_events.sql
--    - Fixed 5 JSON parsing locations
--    - Lines: id, delegatee_id, delegate_event, owner_id, near_amount
--    - Pattern: Added CASE statements with safe_json_parse() checks

-- E. approved_proposals.sql
--    - Fixed 1 JSON parsing location
--    - Lines: proposal_id
--    - Pattern: Added CASE statement with safe_json_parse() check

-- F. registered_voters.sql
--    - Fixed 4 JSON parsing locations
--    - Lines: registered_voter_id, initial_voting_power, current_voting_power_logs, 
--             current_voting_power_args
--    - Pattern: Added CASE statements with safe_json_parse() checks

-- 3. PATTERN USED
-- ===============
-- 
-- BEFORE (unsafe):
-- (convert_from(ra.args_decoded, 'UTF8')::json->>'field_name')::NUMERIC
-- 
-- AFTER (safe):
-- CASE 
--   WHEN safe_json_parse(convert_from(ra.args_decoded, 'UTF8'))->>'error' IS NULL
--   THEN (safe_json_parse(convert_from(ra.args_decoded, 'UTF8'))->>'field_name')::NUMERIC
--   ELSE NULL 
-- END

-- 4. BENEFITS
-- ===========
-- - No more "invalid input syntax for type json" errors
-- - Invalid JSON data is wrapped in error objects for debugging
-- - Valid JSON continues to work normally
-- - Graceful degradation when JSON parsing fails
-- - Ability to track which records have invalid JSON data

-- 5. TESTING
-- ==========
-- Run the test script: near-data-layer/sql_views/test_safe_json.sql
-- This will verify that the helper function works correctly with both valid and invalid JSON.

-- 6. NEXT STEPS
-- =============
-- 1. Run the helper function creation script in your database
-- 2. Test the updated views to ensure they work correctly
-- 3. Monitor for any remaining JSON parsing issues
-- 4. Consider adding debug columns to see original invalid JSON text if needed 