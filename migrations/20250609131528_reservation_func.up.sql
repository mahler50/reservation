-- If user_id id null, find all reservations for the resource in the given time range.
-- If reservation_id is null, find all reservations for the user in the given time range.
-- If both are null, find all reservations in the given time range.
CREATE OR REPLACE FUNCTION rsvp.query(uid text, rid text, during tstzrange, r_status rsvp.reservation_status, page integer default 1, page_size integer default 10, is_desc boolean default false) RETURNS TABLE (LIKE rsvp.reservations) as $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM rsvp.reservations r
    WHERE (uid IS NULL OR r.user_id = uid)
      AND (rid IS NULL OR r.resource_id = rid)
      AND r.status = r_status
      AND during @> r.timespan
    ORDER BY
        CASE WHEN is_desc THEN lower(r.timespan) END DESC,
        CASE WHEN NOT is_desc THEN lower(r.timespan) END ASC
    LIMIT page_size OFFSET (page - 1) * page_size;
END;
$$ LANGUAGE plpgsql;
