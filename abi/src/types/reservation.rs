use std::ops::Bound;

use chrono::{DateTime, FixedOffset, Utc};
use sqlx::{
    FromRow, Row,
    postgres::{PgRow, types::PgRange},
    types::Uuid,
};

use crate::{
    Error, Reservation, ReservationStatus, RsvpStatus, Validator,
    types::{get_time_range, vlidate_time_range},
    utils::utc_time_to_timestamp,
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

    pub fn get_timespan(&self) -> PgRange<DateTime<Utc>> {
        get_time_range(self.start.as_ref(), self.end.as_ref())
    }
}

impl Validator for Reservation {
    fn validate(&self) -> Result<(), Error> {
        if self.user_id.is_empty() {
            return Err(Error::InvalidUserId(self.user_id.clone()));
        }

        if self.resource_id.is_empty() {
            return Err(Error::InvalidResourceId(self.resource_id.clone()));
        }

        vlidate_time_range(self.start.as_ref(), self.end.as_ref())?;
        Ok(())
    }
}

impl FromRow<'_, PgRow> for Reservation {
    fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
        let id: Uuid = row.get("id");
        let status: RsvpStatus = row.get("status");
        let range: PgRange<DateTime<Utc>> = row.get("timespan");
        let range: NaiveRange<DateTime<Utc>> = range.into();
        // Range should always have a start and end time.
        assert!(range.start.is_some());
        assert!(range.end.is_some());
        let start = range.start.unwrap();
        let end = range.end.unwrap();

        Ok(Self {
            id: id.to_string(),
            user_id: row.get("user_id"),
            status: ReservationStatus::from(status) as i32,
            resource_id: row.get("resource_id"),
            start: Some(utc_time_to_timestamp(start)),
            end: Some(utc_time_to_timestamp(end)),
            note: row.get("note"),
        })
    }
}

struct NaiveRange<T> {
    start: Option<T>,
    end: Option<T>,
}

impl<T> From<PgRange<T>> for NaiveRange<T> {
    fn from(value: PgRange<T>) -> Self {
        let f = |b: Bound<T>| match b {
            Bound::Included(v) => Some(v),
            Bound::Excluded(v) => Some(v),
            Bound::Unbounded => None,
        };
        Self {
            start: f(value.start),
            end: f(value.end),
        }
    }
}
