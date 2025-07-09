-- Add migration script here
CREATE TABLE shifts (
    shift_id SERIAL PRIMARY KEY,
    shift_name varchar(50) NOT NULL,
    shift_start INTEGER NULL,
    shift_end int NOT NULL DEFAULT 0,
    shift_start_enroll TIMESTAMP NOT NULL,
    shift_end_enroll TIMESTAMP NOT NULL,
    shift_passday int NOT NULL DEFAULT 0,
    create_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);