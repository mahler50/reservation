use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    DatabaseError(sqlx::Error),

    #[error("Conflict with existing reservation: {0}")]
    ConflictReservation(String),

    #[error("Invalid start or end time for the reservation")]
    InvalidTimespan,

    #[error("Invalid user id: {0}")]
    InvalidUserId(String),

    #[error("Invalid resource id: {0}")]
    InvalidResourceId(String),

    #[error("unknown error")]
    Unknown,
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(e) => {
                let err: &PgDatabaseError = e.downcast_ref();
                match (err.code(), err.schema(), err.table()) {
                    ("23P01", Some("rsvp"), Some("reservations")) => {
                        Error::ConflictReservation(err.detail().unwrap().to_string())
                    }
                    _ => Error::DatabaseError(sqlx::Error::Database(e)),
                }
            }
            _ => Error::DatabaseError(e),
        }
    }
}

// TODO: parse conflict error to extract more details
// pub struct ReservationConflictInfo {
//     a: ReservationWindow,
//     b: ReservationWindow,
// }

// pub struct ReservationWindow {
//     rid: String,
//    start: DateTime<Utc>,
//     end: DateTime<Utc>,
// }
