-- Example showing how the error-wrapping JSON parsing works

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

-- Example 1: Valid JSON
SELECT 'Valid JSON' as example,
       safe_json_parse('{"proposal_id": 123, "title": "Test Proposal"}') as parsed_json;

-- Example 2: Invalid JSON (like what you're getting from the Rust code)
SELECT 'Invalid JSON' as example,
       safe_json_parse('The quick brown fox') as parsed_json;

-- Example 3: How to extract fields safely
SELECT 'Safe field extraction' as example,
       CASE 
         WHEN safe_json_parse('{"proposal_id": 123, "title": "Test Proposal"}')->>'error' IS NULL
         THEN safe_json_parse('{"proposal_id": 123, "title": "Test Proposal"}')->>'title'
         ELSE NULL 
       END as extracted_title;

-- Example 4: How to extract numeric fields safely
SELECT 'Safe numeric extraction' as example,
       CASE 
         WHEN safe_json_parse('{"proposal_id": 123, "title": "Test Proposal"}')->>'error' IS NULL
         THEN (safe_json_parse('{"proposal_id": 123, "title": "Test Proposal"}')->>'proposal_id')::NUMERIC
         ELSE NULL 
       END as extracted_proposal_id;

-- Example 5: What happens with invalid JSON
SELECT 'Invalid JSON extraction' as example,
       CASE 
         WHEN safe_json_parse('The quick brown fox')->>'error' IS NULL
         THEN safe_json_parse('The quick brown fox')->>'title'
         ELSE NULL 
       END as extracted_title;

-- Example 6: See the error details
SELECT 'Error details' as example,
       safe_json_parse('The quick brown fox')->>'error' as error_type,
       safe_json_parse('The quick brown fox')->>'original_text' as original_text,
       safe_json_parse('The quick brown fox')->>'message' as error_message; 