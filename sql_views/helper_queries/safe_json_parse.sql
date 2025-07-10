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
