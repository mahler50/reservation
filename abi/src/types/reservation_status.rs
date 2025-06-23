use std::fmt;

use crate::{ReservationStatus, RsvpStatus};

impl From<RsvpStatus> for ReservationStatus {
    fn from(value: RsvpStatus) -> Self {
        match value {
            RsvpStatus::Unknown => ReservationStatus::Unknown,
            RsvpStatus::Pending => ReservationStatus::Pending,
            RsvpStatus::Confirmed => ReservationStatus::Confirmed,
            RsvpStatus::Blocked => ReservationStatus::Blocked,
        }
    }
}

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
