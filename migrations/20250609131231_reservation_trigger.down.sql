DROP TRIGGER reservation_trigger on rsvp.reservations;
DROP FUNCTION rsvp.reservation_trigger();
DROP TABLE rsvp.reservations_changes CASCADE;
