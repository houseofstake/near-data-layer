-- Test script for safe JSON parsing function with error wrapping

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

-- Test cases
SELECT 'Valid JSON' as test_case, safe_json_parse('{"name": "test", "value": 123}') as result;
SELECT 'Invalid JSON' as test_case, safe_json_parse('This is not JSON') as result;
SELECT 'Empty string' as test_case, safe_json_parse('') as result;
SELECT 'NULL input' as test_case, safe_json_parse(NULL) as result;

-- Test complex JSON
SELECT 'Complex JSON' as test_case, 
       safe_json_parse('{"metadata": {"title": "Test Proposal", "description": "Test"}}') as result;

-- Test extraction from valid JSON
SELECT 'Extract field from valid JSON' as test_case,
       CASE 
         WHEN safe_json_parse('{"name": "test", "value": 123}')->>'error' IS NULL
         THEN safe_json_parse('{"name": "test", "value": 123}')->>'name'
         ELSE NULL 
       END as extracted_value;

-- Test extraction from invalid JSON
SELECT 'Extract field from invalid JSON' as test_case,
       CASE 
         WHEN safe_json_parse('This is not JSON')->>'error' IS NULL
         THEN safe_json_parse('This is not JSON')->>'name'
         ELSE NULL 
       END as extracted_value;

-- Show the error object structure
SELECT 'Error object structure' as test_case,
       safe_json_parse('This is not JSON')->>'error' as error_type,
       safe_json_parse('This is not JSON')->>'original_text' as original_text,
       safe_json_parse('This is not JSON')->>'message' as error_message; 