use chrono::{DateTime, Utc};

pub fn bytes_to_string(bytes: &[u8]) -> String {
    if !bytes.is_empty() {
        let mut value = 0u128;
        for &byte in bytes {
            value = (value << 8) | (byte as u128);
        }
        value.to_string()
    } else {
        "0".to_string()
    }
}

pub fn format_timestamp(timestamp_nanosec: u64) -> String {
    let seconds = (timestamp_nanosec / 1_000_000_000) as i64;
    let nanos = (timestamp_nanosec % 1_000_000_000) as u32;
    let datetime = DateTime::<Utc>::from_timestamp(seconds, nanos).unwrap();
    datetime.format("%Y-%m-%d %H:%M:%S.%f").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_string_empty() {
        let result = bytes_to_string(&[]);
        assert_eq!(result, "0");
    }

    #[test]
    fn test_bytes_to_string_single_byte() {
        let result = bytes_to_string(&[42]);
        assert_eq!(result, "42");
    }

    #[test]
    fn test_bytes_to_string_multiple_bytes() {
        let result = bytes_to_string(&[1, 2, 3, 4]);
        assert_eq!(result, "16909060"); // 1*256^3 + 2*256^2 + 3*256 + 4
    }

    #[test]
    fn test_bytes_to_string_large_number() {
        let result = bytes_to_string(&[255, 255, 255, 255]);
        assert_eq!(result, "4294967295"); // 2^32 - 1
    }

    #[test]
    fn test_format_timestamp() {
        // Test with a known timestamp: 2022-01-01 00:00:00 UTC
        let timestamp_nanosec = 1640995200000000000;
        let result = format_timestamp(timestamp_nanosec);
        
        // The result should contain the date and time
        assert!(result.contains("2022-01-01"));
        assert!(result.contains("00:00:00"));
    }

    #[test]
    fn test_format_timestamp_with_nanoseconds() {
        // Test with nanoseconds: 2022-01-01 00:00:00.123456789 UTC
        let timestamp_nanosec = 1640995200123456789;
        let result = format_timestamp(timestamp_nanosec);
        
        // The result should contain the date, time, and nanoseconds
        assert!(result.contains("2022-01-01"));
        assert!(result.contains("00:00:00"));
        assert!(result.contains("123456789"));
    }
} 