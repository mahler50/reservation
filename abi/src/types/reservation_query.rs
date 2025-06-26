use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgRange;

use crate::{
    Error, ReservationQuery, ReservationStatus, Validator,
    types::{get_time_range, vlidate_time_range},
    utc_time_to_timestamp,
};

#[allow(clippy::too_many_arguments)]
impl ReservationQuery {
    pub fn new(
        user_id: impl Into<String>,
        resource_id: impl Into<String>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        status: ReservationStatus,
        page_size: i32,
        page: i32,
        desc: bool,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            resource_id: resource_id.into(),
            start: Some(utc_time_to_timestamp(start)),
            end: Some(utc_time_to_timestamp(end)),
            status: status as i32,
            page_size,
            page,
            desc,
        }
    }

    /// Returns the timespan as a PostgreSQL range.
    pub fn get_timespan(&self) -> PgRange<DateTime<Utc>> {
        get_time_range(self.start.as_ref(), self.end.as_ref())
    }
}

impl Validator for ReservationQuery {
    fn validate(&self) -> Result<(), Error> {
        vlidate_time_range(self.start.as_ref(), self.end.as_ref())?;

        Ok(())
    }
}
