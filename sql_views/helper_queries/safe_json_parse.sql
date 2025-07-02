-- Helper function to safely parse JSON strings
-- If the string is not valid JSON, wraps it in an error object
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

-- Helper function to safely extract a simple JSON field (e.g., 'field_name')
-- Returns NULL if the JSON is invalid or the field doesn't exist
CREATE OR REPLACE FUNCTION safe_json_extract_simple(input_text TEXT, field_name TEXT)
RETURNS TEXT AS $$
DECLARE
    parsed_json JSON;
BEGIN
    IF input_text IS NULL OR input_text = '' THEN
        RETURN NULL;
    END IF;
    
    BEGIN
        parsed_json := input_text::JSON;
        RETURN parsed_json->>field_name;
    EXCEPTION WHEN OTHERS THEN
        RETURN NULL;
    END;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Helper function to safely extract a numeric value from JSON
-- Returns NULL if the JSON is invalid or the field doesn't exist
CREATE OR REPLACE FUNCTION safe_json_extract_numeric_simple(input_text TEXT, field_name TEXT)
RETURNS NUMERIC AS $$
DECLARE
    parsed_json JSON;
    result TEXT;
BEGIN
    IF input_text IS NULL OR input_text = '' THEN
        RETURN NULL;
    END IF;
    
    BEGIN
        parsed_json := input_text::JSON;
        result := parsed_json->>field_name;
        IF result IS NULL THEN
            RETURN NULL;
        END IF;
        RETURN result::NUMERIC;
    EXCEPTION WHEN OTHERS THEN
        RETURN NULL;
    END;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Example usage in views:
-- Instead of: (convert_from(ra.args_decoded, 'UTF8')::json->'metadata'->>'title')
-- Use: CASE 
--        WHEN safe_json_parse(convert_from(ra.args_decoded, 'UTF8'))->>'error' IS NULL
--        THEN safe_json_parse(convert_from(ra.args_decoded, 'UTF8'))->'metadata'->>'title'
--        ELSE NULL 
--      END

-- For numeric extractions:
-- Instead of: (convert_from(ra.args_decoded, 'UTF8')::json->>'proposal_id')::NUMERIC
-- Use: CASE 
--        WHEN safe_json_parse(convert_from(ra.args_decoded, 'UTF8'))->>'error' IS NULL
--        THEN (safe_json_parse(convert_from(ra.args_decoded, 'UTF8'))->>'proposal_id')::NUMERIC
--        ELSE NULL 
--      END 