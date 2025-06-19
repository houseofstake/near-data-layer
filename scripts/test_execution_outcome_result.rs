use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::Value;

fn main() {
    // Test with the specific transaction data provided
    let test_result_value = "W3sicm9vdCI6IjZnaFVvbTFublNDM1NxWWRnTjdOZU14VkI0dkZQclZOTmhhQmFtZGZ5a0pYIiwibGVuZ3RoIjoyNywiYmxvY2tfaGVpZ2h0IjoyMDE3ODc1MzN9LHsiVjAiOnsidXBkYXRlX3RpbWVzdGFtcCI6IjE3NTAyMzM3NTk4Njg3NDQxMDMiLCJ0b3RhbF92ZW5lYXJfYmFsYW5jZSI6eyJuZWFyX2JhbGFuY2UiOiIxMTc1NjI1NTkzODA2OTYyNzI5NDYxMDIwNDAiLCJleHRyYV92ZW5lYXJfYmFsYW5jZSI6IjEyNTM2MTM3NjM3MTU2Njk5MjE4NDQwMiJ9LCJ2ZW5lYXJfZ3Jvd3RoX2NvbmZpZyI6eyJGaXhlZFJhdGUiOnsiYW5udWFsX2dyb3d0aF9yYXRlX25zIjp7Im51bWVyYXRvciI6IjYiLCJkZW5vbWluYXRvciI6IjMxNTM2MDAwMDAwMDAwMDAwMDAifX19fV0=";
    
    println!("Testing execution outcome result processing...");
    println!("Receipt ID: EmAhjD7QDiwqdfHJDc8auAycNtiZz3KxAzAkzteHX9EV");
    println!("Block Height: 201857082");
    println!("Result Value (base64): {}", test_result_value);
    
    // Decode the base64 result
    match BASE64.decode(test_result_value) {
        Ok(decoded_bytes) => {
            println!("Decoded bytes length: {}", decoded_bytes.len());
            
            // Try to parse as UTF-8 string
            match String::from_utf8(decoded_bytes.clone()) {
                Ok(utf8_string) => {
                    println!("UTF-8 string: {}", utf8_string);
                    
                    // Try to parse as JSON
                    match serde_json::from_str::<Value>(&utf8_string) {
                        Ok(json_value) => {
                            println!("Parsed JSON successfully!");
                            println!("JSON: {}", serde_json::to_string_pretty(&json_value).unwrap());
                            
                            // Extract specific values
                            if let Some(array) = json_value.as_array() {
                                if array.len() >= 2 {
                                    if let Some(v0_obj) = array[1].get("V0") {
                                        if let Some(total_balance) = v0_obj.get("total_venear_balance") {
                                            if let Some(near_balance) = total_balance.get("near_balance") {
                                                println!("Near Balance: {}", near_balance);
                                            }
                                            if let Some(extra_balance) = total_balance.get("extra_venear_balance") {
                                                println!("Extra Balance: {}", extra_balance);
                                            }
                                        }
                                        if let Some(growth_config) = v0_obj.get("venear_growth_config") {
                                            println!("Growth Config: {}", serde_json::to_string_pretty(growth_config).unwrap());
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("Failed to parse as JSON: {}", e);
                            println!("String representation: {:?}", utf8_string);
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to decode as UTF-8: {}", e);
                    println!("Hex representation: 0x{}", hex::encode(&decoded_bytes));
                }
            }
        }
        Err(e) => {
            println!("Failed to decode base64: {}", e);
        }
    }
} 