-- Add migration script here
CREATE TABLE wk_time (
    shift_id SERIAL PRIMARY KEY,
    shift_name varchar(50) NOT NULL,
    shift_start_time BIGINT NOT NULL,
    shift_end_time BIGINT NOT NULL,
    shift_start_enroll BIGINT NOT NULL,
    shift_end_enroll BIGINT NOT NULL,
    shift_dow INT NOT NULL,
    shift_passday SMALLINT NOT NULL DEFAULT 0,
    create_at BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer,
    update_at BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer
);