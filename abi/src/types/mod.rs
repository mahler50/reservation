use std::ops::Bound;

use chrono::{DateTime, Utc};
use prost_types::Timestamp;
use sqlx::postgres::types::PgRange;

use crate::{Error, timestamp_to_utc_time};

mod reservation;
mod reservation_query;
mod reservation_status;

/// Validates the time range.
pub fn vlidate_time_range(start: Option<&Timestamp>, end: Option<&Timestamp>) -> Result<(), Error> {
    if start.is_none() || end.is_none() {
        return Err(Error::InvalidTimespan);
    }

    let start = start.unwrap();
    let end = end.unwrap();
    if start.seconds >= end.seconds {
        return Err(Error::InvalidTimespan);
    }

    Ok(())
}

/// Get pg datetime range from start and end timestamps.
pub fn get_time_range(
    start: Option<&Timestamp>,
    end: Option<&Timestamp>,
) -> PgRange<DateTime<Utc>> {
    let start = timestamp_to_utc_time(start.unwrap());
    let end = timestamp_to_utc_time(end.unwrap());

    PgRange {
        start: Bound::Included(start),
        end: Bound::Excluded(end),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn vlidate_time_range_should_work() {
        let start = Timestamp {
            seconds: 1,
            nanos: 0,
        };
        let end = Timestamp {
            seconds: 2,
            nanos: 0,
        };
        assert!(vlidate_time_range(Some(&start), Some(&end)).is_ok());
    }

    #[test]
    fn vlidate_time_range_should_reject_invalid_range() {
        let start = Timestamp {
            seconds: 2,
            nanos: 0,
        };
        let end = Timestamp {
            seconds: 1,
            nanos: 0,
        };
        assert!(vlidate_time_range(Some(&start), Some(&end)).is_err());
    }

    #[test]
    fn get_time_range_should_work() {
        let start = Timestamp {
            seconds: 1,
            nanos: 0,
        };
        let end = Timestamp {
            seconds: 2,
            nanos: 0,
        };
        let range = get_time_range(Some(&start), Some(&end));
        assert_eq!(range.start, Bound::Included(timestamp_to_utc_time(&start)));
        assert_eq!(range.end, Bound::Excluded(timestamp_to_utc_time(&end)));
    }
}
