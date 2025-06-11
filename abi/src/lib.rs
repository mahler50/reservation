mod pb;

use chrono::{DateTime, TimeZone, Utc};
pub use pb::*;
use prost_types::Timestamp;

/// Converts a `prost_types::Timestamp` to a `chrono::DateTime<Utc>`.
pub fn timestamp_to_utc_time(ts: &Timestamp) -> DateTime<Utc> {
    Utc.timestamp_opt(ts.seconds, ts.nanos as u32).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_timestamp_to_utc_time() {
        let ts = Timestamp {
            seconds: 1735689600, // 2025-01-01T00:00:00Z
            nanos: 0,
        };
        let dt = timestamp_to_utc_time(&ts);
        assert_eq!(dt, Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap());
    }
}
