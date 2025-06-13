CREATE TYPE rsvp.reservation_status AS ENUM (
    'UNKNOWN',
    'PENDING',
    'CONFIRMED',
    'BLOCKED'
);
CREATE TYPE rsvp.reservation_update_type as ENUM (
    'UNKNOWN',
    'CREATE',
    'UPDATE',
    'DELETE'
);

CREATE TABLE rsvp.reservations (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    user_id VARCHAR(64) NOT NULL,
    status rsvp.reservation_status NOT NULL DEFAULT 'PENDING',

    resource_id VARCHAR(64) NOT NULL,
    timespan tstzrange NOT NULL,

    note TEXT,

    -- Ensure that no two reservations overlap for the same resource
    CONSTRAINT reservations_pkey PRIMARY KEY (id),
    CONSTRAINT reservations_resource_exclusion
        EXCLUDE USING GIST (resource_id WITH =, timespan WITH &&)
);
CREATE INDEX reservations_user_id_idx ON rsvp.reservations (user_id);
CREATE INDEX reservations_resource_id_idx ON rsvp.reservations (resource_id);
