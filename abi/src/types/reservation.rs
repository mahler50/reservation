use std::ops::Range;

use chrono::{DateTime, FixedOffset, Utc};

use crate::{
    Error, Reservation, ReservationStatus,
    utils::{timestamp_to_utc_time, utc_time_to_timestamp},
};

impl Reservation {
    /// Creates a new pending reservation with the given parameters.
    pub fn new_pending(
        uid: impl Into<String>,
        rid: impl Into<String>,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
        note: impl Into<String>,
    ) -> Self {
        Self {
            id: "".to_string(),
            user_id: uid.into(),
            resource_id: rid.into(),
            start: Some(utc_time_to_timestamp(start.with_timezone(&Utc))),
            end: Some(utc_time_to_timestamp(end.with_timezone(&Utc))),
            note: note.into(),
            status: ReservationStatus::Pending as i32,
        }
    }

    /// Validates the reservation.
    pub fn validate(&self) -> Result<(), Error> {
        if self.user_id.is_empty() {
            return Err(Error::InvalidUserId(self.user_id.clone()));
        }

        if self.resource_id.is_empty() {
            return Err(Error::InvalidResourceId(self.resource_id.clone()));
        }

        if self.start.is_none() || self.end.is_none() {
            return Err(Error::InvalidTimespan);
        }
        let start = timestamp_to_utc_time(self.start.as_ref().unwrap());
        let end = timestamp_to_utc_time(self.end.as_ref().unwrap());
        if start >= end {
            return Err(Error::InvalidTimespan);
        }
        Ok(())
    }

    pub fn get_timespan(&self) -> Range<DateTime<Utc>> {
        let start = timestamp_to_utc_time(self.start.as_ref().unwrap());
        let end = timestamp_to_utc_time(self.end.as_ref().unwrap());
        start..end
    }
}
