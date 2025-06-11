use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReservationError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Invalid start or end time for the reservation")]
    InvalidTimespan,
    #[error("unknown error")]
    Unknown,
}
