mod error;
mod pb;
mod types;
mod utils;

pub use error::{Error, ReservationConflict, ReservationConflictInfo, ReservationWindow};
pub use pb::*;
pub use utils::*;

/// Database representation of reservation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "reservation_status", rename_all = "UPPERCASE")]
pub enum RsvpStatus {
    Unknown,
    Pending,
    Confirmed,
    Blocked,
}
