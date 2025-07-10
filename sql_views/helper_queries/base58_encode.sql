CREATE OR REPLACE FUNCTION public.base58_encode(hex_string text)
 RETURNS text
 LANGUAGE plpgsql
 IMMUTABLE
AS $function$
DECLARE
    alphabet text := '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
    num bytea;
    result text := '';
    mod integer;
    leading_zeros integer := 0;
    i integer;
    byte_val integer;
BEGIN
    -- Handle empty input
    IF hex_string IS NULL OR hex_string = '' THEN
        RETURN '';
    END IF;

    -- Convert hex to bytea
    num := decode(hex_string, 'hex');
    
    -- Count leading zeros in bytea
    i := 1;
    WHILE i <= length(num) AND get_byte(num, i-1) = 0 LOOP
        leading_zeros := leading_zeros + 1;
        i := i + 1;
    END LOOP;

    -- Convert bytea to base58
    WHILE length(num) > 0 LOOP
        mod := 0;
        FOR i IN 1..length(num) LOOP
            mod := mod * 256 + get_byte(num, i-1);
            num := set_byte(num, i-1, mod / 58);
            mod := mod % 58;
        END LOOP;
        result := substr(alphabet, mod + 1, 1) || result;
        
        -- Remove leading zeros from num
        WHILE length(num) > 0 AND get_byte(num, 0) = 0 LOOP
            num := substring(num from 2);
        END LOOP;
    END LOOP;

    -- Add leading '1's for each leading zero byte
    FOR i IN 1..leading_zeros LOOP
        result := '1' || result;
    END LOOP;

    RETURN result;
END;
$function$
