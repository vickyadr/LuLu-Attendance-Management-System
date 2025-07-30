-- Add migration script here
CREATE TABLE schedules (
    schedule_id SERIAL PRIMARY KEY,
    schedule_name varchar(50) NOT NULL,
    schedule_shift_id INTEGER NULL,
    schedule_dom INTEGER NULL,
    schedule_parrent INTEGER NOT NULL,
    create_at BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer,
    update_at BIGINT NOT NULL DEFAULT DATE_PART('epoch'::text, CURRENT_TIMESTAMP)::integer
);