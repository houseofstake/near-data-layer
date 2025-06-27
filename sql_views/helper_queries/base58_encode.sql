/* This enables the base58_encode() function to work on the dev-node database */
CREATE OR REPLACE FUNCTION base58_encode(input TEXT) RETURNS TEXT AS $$
import base58
return base58.b58encode(bytes.fromhex(input)).decode('utf-8')
$$ LANGUAGE plpython3u; 