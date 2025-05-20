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