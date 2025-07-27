use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgRange;

use crate::{
    Error, ReservationQuery, Validator,
    types::{get_time_range, vlidate_time_range},
};

impl ReservationQuery {
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
