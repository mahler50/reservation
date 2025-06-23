use crate::{ReservationId, ReservationManager, Rsvp};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row, postgres::types::PgRange, types::Uuid};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, mut rsvp: abi::Reservation) -> Result<abi::Reservation, abi::Error> {
        // Validate the reservation.
        rsvp.validate()?;

        // Convert the start and end times to UTC.
        let timespan: PgRange<DateTime<Utc>> = rsvp.get_timespan().into();

        let status = abi::ReservationStatus::try_from(rsvp.status)
            .unwrap_or(abi::ReservationStatus::Pending);

        // execute the SQL query to insert the reservation and return the reservation ID.
        let id: Uuid = sqlx::query("INSERT INTO rsvp.reservations (user_id, resource_id, timespan, note, status) VALUES ($1, $2, $3, $4, $5::rsvp.reservation_status) RETURNING id")
            .bind(rsvp.user_id.clone())
            .bind(rsvp.resource_id.clone())
            .bind(timespan)
            .bind(rsvp.note.clone())
            .bind(status.to_string())
            .fetch_one(&self.pool)
            .await?.get(0);

        rsvp.id = id.to_string();

        Ok(rsvp)
    }

    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, abi::Error> {
        let id = Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        // if current status is `pending`, change it to `confirmed`, otherwie do nothing.
        let rsvp: abi::Reservation = sqlx::query_as(
            "UPDATE rsvp.reservations SET status = 'CONFIRMED' WHERE id = $1::uuid AND status = 'PENDING' RETURNING *"
        ).bind(id).fetch_one(&self.pool).await?;

        Ok(rsvp)
    }

    async fn update_note(
        &self,
        _id: ReservationId,
        _note: String,
    ) -> Result<abi::Reservation, abi::Error> {
        // Implementation for updating reservation note
        unimplemented!()
    }

    async fn delete(&self, _id: ReservationId) -> Result<(), abi::Error> {
        // Implementation for deleting a reservation
        unimplemented!()
    }

    async fn get(&self, _id: ReservationId) -> Result<abi::Reservation, abi::Error> {
        // Implementation for getting a reservation by ID
        unimplemented!()
    }

    async fn query(
        &self,
        _query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, abi::Error> {
        // Implementation for querying reservations
        unimplemented!()
    }
}

impl ReservationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use abi::{ReservationConflict, ReservationConflictInfo, ReservationWindow};

    use super::*;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let manager = ReservationManager::new(migrated_pool.clone());
        // Create a reservation with a valid time window for 3 days.
        let rsvp = abi::Reservation::new_pending(
            "kobe",
            "room-114514",
            "2025-06-01T12:00:00-07:00".parse().unwrap(),
            "2025-06-03T12:00:00-07:00".parse().unwrap(),
            "I'll arrive at 10:00",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();
        assert!(!rsvp.id.is_empty());
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn conflict_reserve_should_reject() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp1 = abi::Reservation::new_pending(
            "kobe",
            "room-114514",
            "2025-06-01T12:00:00-07:00".parse().unwrap(),
            "2025-06-03T12:00:00-07:00".parse().unwrap(),
            "hello.",
        );
        let rsvp2 = abi::Reservation::new_pending(
            "man",
            "room-114514",
            "2025-06-02T12:00:00-07:00".parse().unwrap(),
            "2025-06-05T12:00:00-07:00".parse().unwrap(),
            "hi.",
        );

        let rsvp1 = manager.reserve(rsvp1).await.unwrap();
        assert!(!rsvp1.id.is_empty());
        let err = manager.reserve(rsvp2).await.unwrap_err();

        let info = ReservationConflictInfo::Parsed(ReservationConflict {
            new: ReservationWindow {
                rid: "room-114514".to_string(),
                start: "2025-06-02 19:00:00 UTC".parse().unwrap(),
                end: "2025-06-05 19:00:00 UTC".parse().unwrap(),
            },
            old: ReservationWindow {
                rid: "room-114514".to_string(),
                start: "2025-06-01 19:00:00 UTC".parse().unwrap(),
                end: "2025-06-03 19:00:00 UTC".parse().unwrap(),
            },
        });

        assert_eq!(err, abi::Error::ConflictReservation(info));
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn change_status_should_work() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = abi::Reservation::new_pending(
            "kobe",
            "room-114514",
            "2025-06-01T12:00:00-07:00".parse().unwrap(),
            "2025-06-03T12:00:00-07:00".parse().unwrap(),
            "Man, what can I say!",
        );
        let rsvp = manager.reserve(rsvp).await.unwrap();
        assert_eq!(rsvp.status, abi::ReservationStatus::Pending as i32);
        let updated_rsvp = manager.change_status(rsvp.id.clone()).await.unwrap();
        assert_eq!(
            updated_rsvp.status,
            abi::ReservationStatus::Confirmed as i32
        );
        assert_eq!(updated_rsvp.id, rsvp.id);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn change_status_on_no_pending_rsvp_should_do_nothing() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = abi::Reservation::new_pending(
            "kobe",
            "room-114514",
            "2025-06-01T12:00:00-07:00".parse().unwrap(),
            "2025-06-03T12:00:00-07:00".parse().unwrap(),
            "Man, what can I say!",
        );
        let rsvp = manager.reserve(rsvp).await.unwrap();
        assert_eq!(rsvp.status, abi::ReservationStatus::Pending as i32);
        let updated_rsvp = manager.change_status(rsvp.id.clone()).await.unwrap();
        assert_eq!(
            updated_rsvp.status,
            abi::ReservationStatus::Confirmed as i32
        );
        let res = manager.change_status(rsvp.id.clone()).await.unwrap_err();
        assert_eq!(res, abi::Error::NotFound);
    }
}
