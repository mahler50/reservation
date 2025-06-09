-- reservation change table
CREATE TABLE rsvp.reservations_changes (
    id SERIAL NOT NULL,
    reservation_id uuid NOT NULL,
    op rsvp.reservation_update_type NOT NULL
);

-- trigger for create/updarte/delete a reservation.
CREATE OR REPLACE FUNCTION rsvp.reservation_trigger() RETURNS trigger AS
$$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- update reservations_changes
        INSERT INTO rsvp.reservations_changes (reservation_id, op) VALUES (NEW.id, 'CREATE');
    ELSIF TG_OP = 'UPDATE' THEN
        -- if status changed, update reservations_changes
        IF OLD.status <> NEW.status THEN
            INSERT INTO rsvp.reservations_changes (reservation_id, op) VALUES (NEW.id, 'UPDATE');
        END IF;
    ELSIF TG_OP = 'DELETE' THEN
        -- update reservations_changes
        INSERT INTO rsvp.reservations_changes (reservation_id, op) VALUES (OLD.id, 'DELETE');
    END IF;
    -- notify the reservation change
    NOTIFY reservation_update;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER reservation_trigger
    AFTER INSERT OR UPDATE OR DELETE ON rsvp.reservations
    FOR EACH ROW EXECUTE FUNCTION rsvp.reservation_trigger();
