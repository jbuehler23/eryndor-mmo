//! Helper functions for UI formatting.

/// Format a Unix timestamp (seconds) to a human-readable UTC time string
pub fn format_timestamp(timestamp: i64) -> String {
    use std::time::{UNIX_EPOCH, Duration};

    let _datetime = UNIX_EPOCH + Duration::from_secs(timestamp as u64);

    #[cfg(not(target_family = "wasm"))]
    {
        use chrono::{DateTime, Utc};
        let dt: DateTime<Utc> = _datetime.into();
        dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }

    #[cfg(target_family = "wasm")]
    {
        format!("Unix: {}", timestamp)
    }
}

/// Get current local time as a formatted string
pub fn format_local_time() -> String {
    #[cfg(not(target_family = "wasm"))]
    {
        use chrono::Local;
        let local_time = Local::now();
        local_time.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    #[cfg(target_family = "wasm")]
    {
        let timestamp_ms = js_sys::Date::now() as i64 / 1000;
        format!("Unix: {}", timestamp_ms)
    }
}
