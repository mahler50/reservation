use std::fmt;

use crate::ReservationStatus;

impl fmt::Display for ReservationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReservationStatus::Blocked => write!(f, "BLOCKED"),
            ReservationStatus::Confirmed => write!(f, "CONFIRMED"),
            ReservationStatus::Pending => write!(f, "PENDING"),
            ReservationStatus::Unknown => write!(f, "UNKNOWN"),
        }
    }
}
