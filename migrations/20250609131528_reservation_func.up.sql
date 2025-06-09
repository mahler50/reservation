-- If user_id id null, find all reservations for the resource in the given time range.
-- If reservation_id is null, find all reservations for the user in the given time range.
-- If both are null, find all reservations in the given time range.
CREATE OR REPLACE FUNCTION rsvp.query(uid text, rid text, during tstzrange) RETURNS TABLE (LIKE rsvp.reservations) as $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM rsvp.reservations r
    WHERE (uid IS NULL OR r.user_id = uid)
      AND (rid IS NULL OR r.resource_id = rid)
      AND during @> r.timespan
    ORDER BY r.start_time;
END;
$$ LANGUAGE plpgsql;
