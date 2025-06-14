mod manager;
use async_trait::async_trait;

use sqlx::PgPool;

pub type ReservationId = String;

#[derive(Debug)]
pub struct ReservationManager {
    pool: PgPool,
}

#[async_trait]
pub trait Rsvp {
    /// Make a reservation.
    async fn reserve(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, abi::Error>;
    /// Change reservation status.
    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, abi::Error>;
    /// Update reservation note.
    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, abi::Error>;
    /// Delete reservation.
    async fn delete(&self, id: ReservationId) -> Result<(), abi::Error>;
    /// Get reservation by id.
    async fn get(&self, id: ReservationId) -> Result<abi::Reservation, abi::Error>;
    /// Query reservations.
    async fn query(
        &self,
        query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, abi::Error>;
}
