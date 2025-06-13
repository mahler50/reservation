use chrono::{DateTime, TimeZone, Utc};
use prost_types::Timestamp;

/// Converts a `prost_types::Timestamp` to a `chrono::DateTime<Utc>`.
pub fn timestamp_to_utc_time(ts: &Timestamp) -> DateTime<Utc> {
    Utc.timestamp_opt(ts.seconds, ts.nanos as u32).unwrap()
}

/// Converts a `chrono::DateTime<Utc>` to a `prost_types::Timestamp`.
pub fn utc_time_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
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

    #[test]
    fn test_utc_time_to_timestamp() {
        let dt = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let ts = utc_time_to_timestamp(dt);
        assert_eq!(ts.seconds, 1735689600);
        assert_eq!(ts.nanos, 0);
    }
}
