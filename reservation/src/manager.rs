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
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, abi::Error> {
        let id = Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        let rsvp: abi::Reservation = sqlx::query_as(
            "UPDATE rsvp.reservations SET note = $1 WHERE id = $2::uuid RETURNING *",
        )
        .bind(note)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(rsvp)
    }

    async fn delete(&self, id: ReservationId) -> Result<(), abi::Error> {
        let id = Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        // Execute the SQL query to delete the reservation by ID.
        let _rows_affected = sqlx::query("DELETE FROM rsvp.reservations WHERE id = $1::uuid")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(())
    }

    async fn get(&self, id: ReservationId) -> Result<abi::Reservation, abi::Error> {
        let id = Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        // Execute the SQL query to get the reservation by ID.
        let rsvp: abi::Reservation =
            sqlx::query_as("SELECT * FROM rsvp.reservations WHERE id = $1::uuid")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(rsvp)
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
    use chrono::FixedOffset;

    use super::*;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let manager = ReservationManager::new(migrated_pool.clone());
        // Create a reservation with a valid time window for 3 days.
        let rsvp = make_basic_reservation(&manager).await.unwrap();
        assert!(!rsvp.id.is_empty());
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn conflict_reserve_should_reject() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp1 = make_basic_reservation(&manager).await.unwrap();
        assert!(!rsvp1.id.is_empty());

        // new reservation
        let err = make_reservation(
            &manager,
            "man",
            "room-114514",
            "2025-06-02T12:00:00-07:00".parse().unwrap(),
            "2025-06-05T12:00:00-07:00".parse().unwrap(),
            "hi.",
        )
        .await
        .unwrap_err();

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
        let rsvp = make_basic_reservation(&manager).await.unwrap();
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
        let rsvp = make_basic_reservation(&manager).await.unwrap();
        assert_eq!(rsvp.status, abi::ReservationStatus::Pending as i32);
        let updated_rsvp = manager.change_status(rsvp.id.clone()).await.unwrap();
        assert_eq!(
            updated_rsvp.status,
            abi::ReservationStatus::Confirmed as i32
        );
        let res = manager.change_status(rsvp.id.clone()).await.unwrap_err();
        assert_eq!(res, abi::Error::NotFound);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn update_note_should_work() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = make_basic_reservation(&manager).await.unwrap();
        assert!(!rsvp.id.is_empty());

        let new_note = "Mamba out!".to_string();
        let updated_rsvp = manager
            .update_note(rsvp.id.clone(), new_note.clone())
            .await
            .unwrap();
        assert_eq!(updated_rsvp.note, new_note);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn delete_should_work() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = make_basic_reservation(&manager).await.unwrap();
        assert!(!rsvp.id.is_empty());

        // Delete the reservation.
        manager.delete(rsvp.id.clone()).await.unwrap();

        // Try to get the deleted reservation, should return NotFound error.
        let res = manager.get(rsvp.id.clone()).await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), abi::Error::NotFound);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn query_should_work() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = make_basic_reservation(&manager).await.unwrap();
        assert!(!rsvp.id.is_empty());

        // Query reservations by room ID.
        let get = manager.get(rsvp.id.clone()).await.unwrap();
        assert_eq!(rsvp, get);
    }

    /// Helper functions to create a reservation for testing.
    async fn make_basic_reservation(
        manager: &ReservationManager,
    ) -> Result<abi::Reservation, abi::Error> {
        make_reservation(
            manager,
            "kobe",
            "room-114514",
            "2025-06-01T12:00:00-07:00".parse().unwrap(),
            "2025-06-03T12:00:00-07:00".parse().unwrap(),
            "Man, what can I say!",
        )
        .await
    }

    async fn make_reservation(
        manager: &ReservationManager,
        uid: &str,
        rid: &str,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
        note: &str,
    ) -> Result<abi::Reservation, abi::Error> {
        let rsvp = abi::Reservation::new_pending(uid, rid, start, end, note);
        manager.reserve(rsvp).await
    }
}
